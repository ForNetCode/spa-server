use crate::acme::ACMEManager;
use crate::config::{get_host_path_from_domain, AdminConfig};
use crate::domain_storage::DomainStorage;
use crate::hot_reload::HotReloadManager;
use crate::with;
use delay_timer::prelude::*;
use entity::request::{
    DeleteDomainVersionOption, DomainWithOptVersionOption, DomainWithVersionOption,
    GetDomainOption, GetDomainPositionOption, UpdateUploadingStatusOption, UploadFileOption,
};
use hyper::{Body, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use warp::multipart::FormData;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

pub struct AdminServer {
    conf: Arc<AdminConfig>,
    domain_storage: Arc<DomainStorage>,
    reload_manager: Arc<HotReloadManager>,
    acme_manager: Arc<ACMEManager>,
    delay_timer: DelayTimer,
    host_alias: Arc<HashMap<String, String>>,
}

impl AdminServer {
    pub fn new(
        conf: &AdminConfig,
        domain_storage: Arc<DomainStorage>,
        reload_manager: HotReloadManager,
        acme_manager: Arc<ACMEManager>,
        delay_timer: DelayTimer,
        host_alias: Arc<HashMap<String, String>>,
    ) -> Self {
        AdminServer {
            conf: Arc::new(conf.clone()),
            domain_storage,
            reload_manager: Arc::new(reload_manager),
            acme_manager,
            delay_timer,
            host_alias,
        }
    }

    fn routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        self.auth().and(
            warp::get()
                .and(
                    self.get_domain_info()
                        .or(self.get_domain_upload_path())
                        .or(self.get_files_metadata())
                        .or(self.get_acme_cert_info()),
                )
                .or(warp::post().and(
                    self.update_domain_version()
                        .or(self.reload_server())
                        .or(self.change_upload_status())
                        .or(self.upload_file())
                        .or(self.revoke_version())
                        .or(self.remove_domain_version()),
                )),
        )
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let bind_address =
            SocketAddr::from_str(&format!("{}:{}", &self.conf.addr, &self.conf.port)).unwrap();
        warp::serve(self.routes()).run(bind_address).await;
        if let Some(cron_config) = &self.conf.deprecated_version_delete {
            self.delay_timer.add_task(build_async_job(
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

    fn get_domain_info(&self) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
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
            .and(with(self.host_alias.clone()))
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
        let admin_config = self.conf.clone();
        let reload_manager = self.reload_manager.clone();
        let acme_manager = self.acme_manager.clone();

        warp::path("reload")
            .and(warp::path::end())
            .and(with(reload_manager))
            .and(with(admin_config))
            .and(with(acme_manager))
            .and_then(service::reload_server)
    }

    fn change_upload_status(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path!("files" / "upload_status")
            .and(with(self.domain_storage.clone()))
            .and(with(self.acme_manager.clone()))
            .and(
                warp::body::content_length_limit(1024 * 16)
                    .and(warp::body::json::<UpdateUploadingStatusOption>()),
            )
            .and(with(self.host_alias.clone()))
            .and_then(service::change_upload_status)
    }

    fn check_alias(domain: &str, host_alias: Arc<HashMap<String, String>>) -> Option<Response> {
        let (host, _) = get_host_path_from_domain(domain);
        if let Some(original_host) = host_alias.get(host) {
            return Some(bad_resp(format!(
                "should not use alias domain, please use {original_host}"
            )));
        }
        None
    }

    fn upload_file(&self) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        async fn handler(
            query: UploadFileOption,
            form: FormData,
            storage: Arc<DomainStorage>,
            host_alias: Arc<HashMap<String, String>>,
        ) -> Result<Response, Infallible> {
            if let Some(resp) = AdminServer::check_alias(&query.domain, host_alias) {
                return Ok(resp);
            }
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
            .and(with(self.host_alias.clone()))
            .and_then(handler)
    }

    fn get_files_metadata(
        &self,
    ) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
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

    fn get_acme_cert_info(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path!("cert" / "acme")
            .and(with(self.acme_manager.clone()))
            .and(warp::query::<GetDomainOption>())
            .and_then(service::get_acme_cert_info)
    }

    fn revoke_version(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        warp::path!("files" / "revoke_version")
            .and(with(self.domain_storage.clone()))
            .and(warp::body::json::<DomainWithVersionOption>())
            .and_then(service::revoke_version)
    }
}

pub mod service {
    use crate::acme::ACMEManager;
    use crate::domain_storage::{DomainStorage, URI_REGEX};
    use crate::service::not_found;
    use crate::{AdminConfig, HotReloadManager};
    use anyhow::{anyhow, Context};
    use bytes::Buf;
    use entity::request::{
        DeleteDomainVersionOption, DomainWithOptVersionOption, DomainWithVersionOption,
        GetDomainOption, GetDomainPositionFormat, GetDomainPositionOption,
        UpdateUploadingStatusOption, UploadFileOption,
    };
    use entity::storage::DomainInfo;
    use futures_util::{StreamExt, TryStreamExt};
    use hyper::Body;
    use std::collections::HashMap;
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
                    return if let Some(data) = domain_info.iter().find(|x| x.domain == domain) {
                        Ok(warp::reply::json(&[data]).into_response())
                    } else {
                        Ok(warp::reply::json::<[&DomainInfo; 0]>(&[]).into_response())
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
        host_alias: Arc<HashMap<String, String>>,
    ) -> Response {
        if let Some(resp) = super::AdminServer::check_alias(&option.domain, host_alias) {
            return resp;
        }
        if URI_REGEX.is_match(&option.domain) {
            match storage.get_upload_position(&option.domain) {
                Ok(ret) => {
                    if option.format == GetDomainPositionFormat::Json {
                        warp::reply::json(&ret).into_response()
                    } else {
                        ret.path.to_string_lossy().to_string().into_response()
                    }
                }
                Err(err) => {
                    let mut resp = StatusCode::BAD_REQUEST.into_response();
                    *resp.body_mut() = Body::from(err.to_string());
                    resp
                }
            }
        } else {
            StatusCode::BAD_REQUEST.into_response()
        }
    }

    pub(super) async fn reload_server(
        reload_manager: Arc<HotReloadManager>,
        admin_config: Arc<AdminConfig>,
        acme_manager: Arc<ACMEManager>,
    ) -> Result<Response, Infallible> {
        let resp = match crate::reload_server(&admin_config, reload_manager.as_ref(), acme_manager)
            .await
        {
            Ok(_) => Response::default(),
            Err(e) => {
                let mut resp = Response::new(Body::from(format!("error:{}", e)));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        };
        Ok(resp)
    }

    pub(super) async fn change_upload_status(
        storage: Arc<DomainStorage>,
        acme_manager: Arc<ACMEManager>,
        param: UpdateUploadingStatusOption,
        host_alias: Arc<HashMap<String, String>>,
    ) -> Result<Response, Infallible> {
        if let Some(resp) = super::AdminServer::check_alias(&param.domain, host_alias) {
            return Ok(resp);
        }
        let resp = match storage
            .update_uploading_status(param.domain, param.version, param.status, &acme_manager)
            .await
        {
            Ok(_) => Response::default(),
            Err(e) => {
                let mut resp = Response::new(Body::from(e.to_string()));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        };
        Ok(resp)
    }

    pub(super) async fn update_file(
        query: UploadFileOption,
        form: FormData,
        storage: Arc<DomainStorage>,
    ) -> anyhow::Result<Response> {
        if let Err(e) = storage.check_if_can_upload(&query.domain) {
            let mut resp = StatusCode::BAD_REQUEST.into_response();
            *resp.body_mut() = Body::from(e.to_string());
            return Ok(resp);
        }
        let mut parts = form.into_stream();
        if let Some(Ok(part)) = parts.next().await {
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
        Err(anyhow!("bad params, please check the api doc: https://github.com/fornetcode/spa-server/blob/master/docs/guide/sap-server-api.md"))
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
    pub(super) fn remove_domain_version(
        storage: Arc<DomainStorage>,
        query: DeleteDomainVersionOption,
    ) -> Response {
        let domains_info = if let Some(domain) = query.domain {
            storage
                .get_domain_info_by_domain(&domain)
                .map(|v| vec![v])
                .unwrap_or_default()
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
                    max_version -= max_reserve;
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
    pub(super) async fn get_acme_cert_info(
        acme_manager: Arc<ACMEManager>,
        query: GetDomainOption,
    ) -> Result<Response, Infallible> {
        let resp = match acme_manager.get_cert_data(query.domain.as_ref()).await {
            Ok(data) => warp::reply::json(&data).into_response(),
            Err(err) => {
                let mut resp = Response::new(Body::from(err.to_string()));
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                resp
            }
        };
        Ok(resp)
    }

    //TODO: when delete and revoke occur currently. would have problems.
    pub(super) async fn revoke_version(
        domain_storage: Arc<DomainStorage>,
        query: DomainWithVersionOption,
    ) -> Result<Response, Infallible> {
        let DomainWithVersionOption { domain, version } = query;
        let resp = match domain_storage.get_domain_info_by_domain(&domain) {
            Some(info) => {
                if info
                    .current_version
                    .is_some_and(|current_version| current_version > version)
                    && info.versions.contains(&version)
                {
                    match domain_storage
                        .upload_domain_with_version(domain, Some(version))
                        .await
                    {
                        Ok(_) => Response::default(),
                        Err(e) => {
                            let mut resp = StatusCode::BAD_REQUEST.into_response();
                            *resp.body_mut() = Body::from(e.to_string());
                            resp
                        }
                    }
                } else {
                    not_found()
                }
            }
            None => not_found(),
        };
        Ok(resp)
    }
}

fn bad_resp(text: String) -> Response {
    let mut resp = StatusCode::BAD_REQUEST.into_response();
    *resp.body_mut() = Body::from(text);
    resp
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
    use chrono::prelude::*;
    use delay_timer::entity::DelayTimerBuilder;
    use delay_timer::prelude::TaskBuilder;
    use std::time::Duration;

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
            let time = NaiveDateTime::from_timestamp_opt(time, 0).unwrap();
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
