use crate::admin_server::request::{
    DeleteDomainVersionOption, DomainWithOptVersionOption, DomainWithVersionOption,
    GetDomainOption, GetDomainPositionOption, UpdateUploadingStatusOption, UploadFileOption,
};
use crate::config::AdminConfig;
use crate::domain_storage::DomainStorage;
use crate::hot_reload::HotReloadManager;
use crate::with;
use delay_timer::prelude::*;
use hyper::{Body, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use warp::multipart::FormData;
use warp::reply::Response;
use warp::{Filter, Rejection};

pub struct AdminServer {
    conf: AdminConfig,
    domain_storage: Arc<DomainStorage>,
    reload_manager: HotReloadManager,
}

impl AdminServer {
    pub fn new(
        conf: &AdminConfig,
        domain_storage: Arc<DomainStorage>,
        reload_manager: HotReloadManager,
    ) -> Self {
        AdminServer {
            conf: conf.clone(),
            domain_storage,
            reload_manager,
        }
    }

    fn routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        self.auth().and(
            (warp::get().and(
                self.get_domain_info()
                    .or(self.get_domain_upload_path())
                    .or(self.get_files_metadata()),
            ))
            .or(warp::post().and(
                self.update_domain_version()
                    .or(self.reload_server())
                    .or(self.change_upload_status())
                    .or(self.upload_file())
                    .or(self.remove_domain_version()),
            )),
        )
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
        warp::serve(self.routes()).run(bind_address).await;
        if let Some(cron_config) = &self.conf.deprecated_version_delete {
            let delay_timer = DelayTimerBuilder::default()
                .tokio_runtime_by_default()
                .build();
            delay_timer.add_task(build_async_job(
                self.domain_storage.clone(),
                Some(cron_config.max_reserve),
                &cron_config.cron,
            )?)?;
        }
        Ok(())
    }

    fn auth(&self) -> impl Filter<Extract = (), Error = Rejection> + Clone {
        // this will not trigger memory leak, be careful to use it
        warp::header::exact(
            "authorization",
            Box::leak(format!("Bearer {}", &self.conf.token).into_boxed_str()),
        )
    }

    fn get_domain_info(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path("status")
            .and(warp::query::<GetDomainOption>())
            .and(with(self.domain_storage.clone()))
            .and_then(service::get_domain_info)
    }

    fn get_domain_upload_path(
        &self,
    ) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
        warp::path!("upload" / "position")
            .and(warp::query::<GetDomainPositionOption>())
            .and(with(self.domain_storage.clone()))
            .map(service::get_upload_position)
    }

    fn update_domain_version(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path("update_version")
            .and(warp::path::end())
            .and(
                warp::body::content_length_limit(1024 * 16)
                    .and(warp::body::json::<DomainWithOptVersionOption>()),
            )
            .and(with(self.domain_storage.clone()))
            .and_then(service::update_domain_version)
    }

    fn reload_server(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        let admin_config = Arc::new(self.conf.clone());
        let reload_manager = Arc::new(self.reload_manager.clone());

        warp::path("reload")
            .and(warp::path::end())
            .and(with(reload_manager))
            .and(with(admin_config))
            .and_then(service::reload_server)
    }

    fn change_upload_status(
        &self,
    ) -> impl Filter<Extract = (Response,), Error = Rejection> + Clone {
        warp::path!("files" / "upload_status")
            .and(with(self.domain_storage.clone()))
            .and(
                warp::body::content_length_limit(1024 * 16)
                    .and(warp::body::json::<UpdateUploadingStatusOption>()),
            )
            .map(service::change_upload_status)
    }

    fn upload_file(&self) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        async fn handler(
            query: UploadFileOption,
            form: FormData,
            storage: Arc<DomainStorage>,
        ) -> Result<Response, Infallible> {
            let resp = service::update_file(query, form, storage)
                .await
                .unwrap_or_else(|e| {
                    let mut resp = Response::new(Body::from(e.to_string()));
                    *resp.status_mut() = StatusCode::BAD_REQUEST;
                    resp
                });
            Ok(resp)
        }
        warp::path!("file" / "upload")
            .and(warp::path::end())
            .and(warp::query::<UploadFileOption>())
            .and(warp::multipart::form().max_length(self.conf.max_upload_size))
            .and(with(self.domain_storage.clone()))
            .and_then(handler)
    }

    fn get_files_metadata(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path!("files" / "metadata")
            .and(with(self.domain_storage.clone()))
            .and(warp::query::<DomainWithVersionOption>())
            .map(service::get_files_metadata)
    }

    fn remove_domain_version(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path!("files" / "delete")
            .and(with(self.domain_storage.clone()))
            .and(warp::query::<DeleteDomainVersionOption>())
            .map(service::remove_domain_version)
    }
}

pub mod service {
    use crate::admin_server::request::{
        DeleteDomainVersionOption, DomainWithOptVersionOption, DomainWithVersionOption,
        GetDomainOption, GetDomainPositionFormat, GetDomainPositionOption,
        UpdateUploadingStatusOption, UploadFileOption,
    };
    use crate::domain_storage::{DomainStorage, URI_REGEX};
    use crate::{AdminConfig, HotReloadManager};
    use anyhow::{anyhow, Context};
    use bytes::Buf;
    use futures_util::{StreamExt, TryStreamExt};
    use hyper::Body;
    use std::convert::Infallible;
    use std::sync::Arc;
    use tracing::error;
    use warp::http::StatusCode;
    use warp::multipart::FormData;
    use warp::reply::Response;
    use warp::Reply;

    pub(super) async fn get_domain_info(
        option: GetDomainOption,
        storage: Arc<DomainStorage>,
    ) -> Result<Response, Infallible> {
        match storage.get_domain_info() {
            Ok(domain_info) => match option.domain {
                Some(domain) => {
                    if let Some(data) = domain_info.iter().find(|x| x.domain == domain) {
                        return Ok(warp::reply::json(data).into_response());
                    } else {
                        return Ok(StatusCode::NOT_FOUND.into_response());
                    }
                }
                _ => Ok(warp::reply::json(&domain_info).into_response()),
            },
            Err(e) => {
                let mut resp = Response::new(Body::from(e.to_string()));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                Ok(resp)
            }
        }
    }

    pub(super) async fn update_domain_version(
        option: DomainWithOptVersionOption,
        storage: Arc<DomainStorage>,
    ) -> Result<Response, Infallible> {
        match storage
            .upload_domain_with_version(option.domain.clone(), option.version)
            .await
        {
            Ok(version) => {
                let text = format!(
                    "domain:{} static web version has changed to {}",
                    option.domain, version
                );
                tracing::info!("{}", &text);
                Ok(text.into_response())
            }
            Err(e) => {
                error!("upload domain({}) version failure {:?}", option.domain, e);
                Ok(StatusCode::NOT_FOUND.into_response())
            }
        }
    }

    pub(super) fn get_upload_position(
        option: GetDomainPositionOption,
        storage: Arc<DomainStorage>,
    ) -> Response {
        if URI_REGEX.is_match(&option.domain) {
            let ret = storage.get_upload_position(&option.domain);
            if option.format == GetDomainPositionFormat::Json {
                warp::reply::json(&ret).into_response()
            } else {
                ret.path.to_string_lossy().to_string().into_response()
            }
        } else {
            StatusCode::BAD_REQUEST.into_response()
        }
    }

    pub(super) async fn reload_server(
        reload_manager: Arc<HotReloadManager>,
        admin_config: Arc<AdminConfig>,
    ) -> Result<Response, Infallible> {
        let resp = match crate::reload_server(&admin_config, reload_manager.as_ref()).await {
            Ok(_) => Response::default(),
            Err(e) => {
                let mut resp = Response::new(Body::from(format!("error:{}", e)));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        };
        Ok(resp)
    }

    pub(super) fn change_upload_status(
        storage: Arc<DomainStorage>,
        param: UpdateUploadingStatusOption,
    ) -> Response {
        match storage.update_uploading_status(param.domain, param.version, param.status) {
            Ok(_) => Response::default(),
            Err(e) => {
                let mut resp = Response::new(Body::from(e.to_string()));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        }
    }

    pub(super) async fn update_file(
        query: UploadFileOption,
        form: FormData,
        storage: Arc<DomainStorage>,
    ) -> anyhow::Result<Response> {
        let mut parts = form.into_stream();
        while let Some(Ok(part)) = parts.next().await {
            // let name = part.name();
            let file = part
                .stream()
                .try_fold(Vec::new(), |mut acc, buf| async move {
                    acc.extend_from_slice(buf.chunk());
                    Ok(acc)
                })
                .await
                .with_context(|| "get form failure")?;
            tracing::debug!(
                "uploading file: {:?}, {:?}, {:?}",
                &query.domain,
                &query.version,
                &query.path
            );
            storage.save_file(query.domain, query.version, query.path, file)?;
            return Ok(Response::default());
        }
        return Err(anyhow!("bad params, please check the api doc: https://github.com/fornetcode/spa-server/blob/master/docs/guide/sap-server-api.md"));
    }

    pub(super) fn get_files_metadata(
        storage: Arc<DomainStorage>,
        query: DomainWithVersionOption,
    ) -> Response {
        match storage.get_files_metadata(query.domain, query.version) {
            Ok(data) => warp::reply::json(&data).into_response(),
            Err(err) => {
                let mut resp = Response::new(Body::from(err.to_string()));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        }
    }

    // TODO: how to handle uploading versions
    // TODO: remove domain directory if there's no version dir.
    // TODO: handle multiple domain meta.
    pub(super) fn remove_domain_version(
        storage: Arc<DomainStorage>,
        query: DeleteDomainVersionOption,
    ) -> Response {
        let domains_info = if let Some(domain) = query.domain {
            storage
                .get_domain_info_by_domain(&domain)
                .map(|v| vec![v])
                .unwrap_or(vec![])
        } else {
            storage.get_domain_info().unwrap_or_else(|_| vec![])
        };
        for info in domains_info {
            let delete_versions = if let Some(max_reserve) = query.max_reserve {
                if let Some(mut max_version) =
                    info.current_version
                        .or(info.versions.iter().max().map(|x| *x))
                {
                    //TODO: fix it, get reserve versions by array index compare, rather than -.
                    max_version = max_version - max_reserve;
                    info.versions
                        .into_iter()
                        .filter(|v| *v < max_version)
                        .collect::<Vec<u32>>()
                } else {
                    vec![]
                }
            } else {
                // TODO: keep uploading versions ?
                let current_version = info.current_version.unwrap_or(u32::MAX);
                info.versions
                    .into_iter()
                    .filter(|version| *version != current_version)
                    .collect::<Vec<u32>>()
            };
            for version in delete_versions {
                let _ = storage.remove_domain_version(&info.domain, Some(version));
            }
        }
        Response::default()
    }
}

pub mod request {
    use crate::domain_storage::UploadingStatus;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct GetDomainOption {
        pub domain: Option<String>,
    }

    #[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
    pub enum GetDomainPositionFormat {
        Path,
        Json,
    }
    impl Default for GetDomainPositionFormat {
        fn default() -> Self {
            GetDomainPositionFormat::Path
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct GetDomainPositionOption {
        pub domain: String,
        //#[serde(default="crate::admin_server::request::GetDomainPositionFormat::Path")]
        #[serde(default)]
        pub format: GetDomainPositionFormat,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct UploadFileOption {
        pub domain: String,
        pub version: u32,
        pub path: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct DomainWithVersionOption {
        pub domain: String,
        pub version: u32,
    }

    #[derive(Deserialize, Serialize)]
    pub struct DomainWithOptVersionOption {
        pub domain: String,
        pub version: Option<u32>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct UpdateUploadingStatusOption {
        pub domain: String,
        pub version: u32,
        pub status: UploadingStatus,
    }

    #[derive(Deserialize, Serialize)]
    pub struct DeleteDomainVersionOption {
        pub domain: Option<String>,
        pub max_reserve: Option<u32>,
    }
}

fn build_async_job(
    domain_storage: Arc<DomainStorage>,
    max_reserve: Option<u32>,
    cron: &str,
) -> anyhow::Result<Task> {
    let domain_storage = domain_storage.clone();

    let body = move || {
        service::remove_domain_version(
            domain_storage.clone(),
            DeleteDomainVersionOption {
                domain: None,
                max_reserve,
            },
        );
    };
    let builder = TaskBuilder::default()
        .set_frequency_repeated_by_cron_str(cron)
        .set_task_id(1)
        .set_maximum_parallel_runnable_num(1)
        .spawn_routine(body)?;
    Ok(builder)
}
// TODO: the code structure is not friendly with Unit Test, need refactor it.
#[cfg(test)]
mod test {
    use crate::admin_server::request::DomainWithVersionOption;
    use chrono::prelude::*;
    use delay_timer::entity::DelayTimerBuilder;
    use delay_timer::prelude::TaskBuilder;
    use std::time::Duration;
    use warp::test::request;

    #[tokio::test]
    async fn update_domain_version_test() {
        let body = DomainWithVersionOption {
            domain: "self.noti.link".to_string(),
            version: 1,
        };
        /*
        let resp = request()
            .method("POST")
            .path("/update_version")
            .json(&body)
            .reply(&api)
            .await;*/
    }

    #[tokio::test]
    async fn delay_is_ok() {
        let body = move || {
            println!("run delay job");
        };
        let mut task = TaskBuilder::default()
            .set_frequency_repeated_by_cron_str("0 0 3 * * *")
            .set_task_id(1)
            .set_maximum_parallel_runnable_num(1)
            .spawn_routine(body)
            .unwrap();
        assert!(task.is_valid());
        for _ in 0..10 {
            let time = task.get_next_exec_timestamp().unwrap() as i64;
            let time = NaiveDateTime::from_timestamp(time, 0);
            let time: DateTime<Utc> = DateTime::from_utc(time, Utc);
            println!("{}", time.format("%Y-%m-%d %H:%M:%S"));
        }
        let delay_timer = DelayTimerBuilder::default()
            .tokio_runtime_by_default()
            .build();
        delay_timer.add_task(task).unwrap();

        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("finish");
    }
}
