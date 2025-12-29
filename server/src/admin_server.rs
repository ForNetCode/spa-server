use crate::config::{AdminConfig, get_host_path_from_domain};
use crate::domain_storage::DomainStorage;
use delay_timer::prelude::*;
use salvo::oapi::OpenApi;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BearerValidator {
    token: String,
}

#[async_trait]
impl Handler for BearerValidator {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        match req.headers().get("Authorization") {
            Some(value) if value == &self.token => {
                ctrl.call_next(req, depot, res).await;
            }
            _ => {
                res.status_code(StatusCode::UNAUTHORIZED);
                ctrl.skip_rest();
            }
        }
    }
}

pub struct AdminServer {
    conf: Arc<AdminConfig>,
    domain_storage: Arc<DomainStorage>,
    delay_timer: DelayTimer,
    host_alias: Arc<HashMap<String, String>>,
}

impl AdminServer {
    pub fn new(
        conf: &AdminConfig,
        domain_storage: Arc<DomainStorage>,
        delay_timer: DelayTimer,
        host_alias: Arc<HashMap<String, String>>,
    ) -> Self {
        AdminServer {
            conf: Arc::new(conf.clone()),
            domain_storage,
            delay_timer,
            host_alias,
        }
    }

    fn routes(&self) -> Router {
        let api_router = Router::with_hoop(BearerValidator {
            token: format!("Bearer {}", self.conf.token),
        })
        .hoop(
            affix_state::inject(self.domain_storage.clone())
                .inject(self.conf.clone())
                .inject(self.host_alias.clone()),
        )
        .push(Router::with_path("status").get(service::get_domain_info))
        .push(Router::with_path("upload/position").get(service::get_upload_position))
        .push(Router::with_path("update_version").post(service::update_domain_version))
        .push(Router::with_path("files/upload_status").post(service::change_upload_status))
        .push(Router::with_path("file/upload").post(service::update_file))
        .push(Router::with_path("files/metadata").get(service::get_files_metadata))
        .push(Router::with_path("files/delete").post(service::remove_domain_version))
        .push(Router::with_path("files/revoke_version").post(service::revoke_version));

        let doc = OpenApi::new("SPA Server Admin API", "1.0.0").merge_router(&api_router);

        Router::new()
            .push(doc.into_router("/api-doc/openapi.json"))
            .push(SwaggerUi::new("/api-doc/openapi.json").into_router("swagger-ui"))
            .push(api_router)
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let listener = TcpListener::new((self.conf.addr.clone(), self.conf.port))
            .bind()
            .await;

        if let Some(cron_config) = &self.conf.deprecated_version_delete {
            self.delay_timer.add_task(build_async_job(
                self.domain_storage.clone(),
                Some(cron_config.max_reserve),
                &cron_config.cron,
            )?)?;
        }

        Server::new(listener).serve(self.routes()).await;

        Ok(())
    }

    fn check_alias(
        domain: &str,
        host_alias: Arc<HashMap<String, String>>,
        res: &mut Response,
    ) -> bool {
        let (host, _) = get_host_path_from_domain(domain);
        if let Some(original_host) = host_alias.get(host) {
            bad_resp(
                format!("should not use alias domain, please use {original_host}"),
                res,
            );
            return true;
        }
        false
    }
}

pub mod service {
    use crate::admin_server::bad_resp;
    use crate::domain_storage::{DomainStorage, uri_regex};
    use entity::request::{
        DeleteDomainVersionOption, DomainWithOptVersionOption, DomainWithVersionOption,
        GetDomainOption, GetDomainPositionFormat, GetDomainPositionOption,
        UpdateUploadingStatusOption, UploadFileOption,
    };
    use salvo::prelude::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tracing::error;

    /// Get domain info
    ///
    /// Returns information about domains. If domain query parameter is provided,
    /// returns info for that specific domain only.
    #[endpoint(
        responses(
            (status_code = 200, description = "Domain info retrieved successfully", body = Vec<entity::storage::DomainInfo>),
            (status_code = 400, description = "Bad request"),
            (status_code = 401, description = "Unauthorized")
        )
    )]
    pub async fn get_domain_info(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
        let option = req.parse_queries::<GetDomainOption>();
        match storage.get_domain_info() {
            Ok(domain_info) => {
                if let Ok(option) = option
                    && let Some(domain) = option.domain
                {
                    if let Some(data) = domain_info.iter().find(|x| x.domain == domain) {
                        res.render(Json(&[data]));
                    } else {
                        let data: Vec<&entity::storage::DomainInfo> = Vec::new();
                        res.render(Json(data));
                    }
                    return;
                }
                res.render(Json(domain_info));
            }
            Err(e) => {
                bad_resp(e.to_string(), res);
            }
        }
    }

    /// Update domain version
    ///
    /// Changes the current active version for a domain.
    #[endpoint(
        responses(
            (status_code = 200, description = "Version updated successfully", body = String),
            (status_code = 404, description = "Domain not found"),
            (status_code = 400, description = "Bad request"),
            (status_code = 401, description = "Unauthorized")
        ),
        request_body = DomainWithOptVersionOption
    )]
    pub async fn update_domain_version(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
        if let Ok(option) = req.parse_json::<DomainWithOptVersionOption>().await {
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
                    res.render(text);
                }
                Err(e) => {
                    error!("upload domain({}) version failure {:?}", option.domain, e);
                    res.status_code(StatusCode::NOT_FOUND);
                }
            }
        } else {
            res.status_code(StatusCode::BAD_REQUEST);
        }
    }

    /// Get upload position
    ///
    /// Returns the file system path for uploading files to a domain.
    /// Response can be either JSON (with format=json query param) or plain text path.
    #[endpoint(
        responses(
            (status_code = 200, description = "Upload position retrieved"),
            (status_code = 400, description = "Bad request"),
            (status_code = 401, description = "Unauthorized")
        )
    )]
    pub async fn get_upload_position(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
        let host_alias = depot.obtain::<Arc<HashMap<String, String>>>().unwrap();
        if let Ok(option) = req.parse_queries::<GetDomainPositionOption>() {
            if super::AdminServer::check_alias(&option.domain, host_alias.clone(), res) {
                return;
            }
            if uri_regex().is_match(&option.domain) {
                match storage.get_upload_position(&option.domain) {
                    Ok(ret) => {
                        if option.format == GetDomainPositionFormat::Json {
                            res.render(Json(ret));
                        } else {
                            res.render(ret.path.to_string_lossy().to_string());
                        }
                    }
                    Err(err) => {
                        bad_resp(err.to_string(), res);
                    }
                }
            } else {
                res.status_code(StatusCode::BAD_REQUEST);
            }
        } else {
            res.status_code(StatusCode::BAD_REQUEST);
        }
    }

    /// Change upload status
    ///
    /// Updates the uploading status for a specific domain version.
    #[endpoint(
        responses(
            (status_code = 200, description = "Upload status updated"),
            (status_code = 400, description = "Bad request"),
            (status_code = 401, description = "Unauthorized")
        ),
        request_body = UpdateUploadingStatusOption
    )]
    pub async fn change_upload_status(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
        let host_alias = depot.obtain::<Arc<HashMap<String, String>>>().unwrap();
        if let Ok(param) = req.parse_json::<UpdateUploadingStatusOption>().await {
            if super::AdminServer::check_alias(&param.domain, host_alias.clone(), res) {
                return;
            }
            match storage
                .update_uploading_status(param.domain, param.version, param.status)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    bad_resp(e.to_string(), res);
                }
            }
        } else {
            res.status_code(StatusCode::BAD_REQUEST);
        }
    }

    /// Upload file
    ///
    /// Uploads a single file to a domain version. Uses multipart/form-data.
    /// Requires query parameters: domain, version, path.
    #[endpoint(
        responses(
            (status_code = 200, description = "File uploaded successfully"),
            (status_code = 400, description = "Bad request"),
            (status_code = 401, description = "Unauthorized")
        )
    )]
    pub async fn update_file(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
        let host_alias = depot.obtain::<Arc<HashMap<String, String>>>().unwrap();
        let query = match req.parse_queries::<UploadFileOption>() {
            Ok(query) => query,
            Err(_e) => {
                bad_resp("invalid parameters".to_owned(), res);
                return;
            }
        };

        if super::AdminServer::check_alias(&query.domain, host_alias.clone(), res) {
            return;
        }
        if let Err(e) = storage.check_if_can_upload(&query.domain) {
            bad_resp(e.to_string(), res);
            return;
        }
        match req.file("file").await {
            Some(file) => {
                tracing::debug!(
                    "uploading file: {:?}, {:?}, {:?}",
                    &query.domain,
                    &query.version,
                    &query.path
                );
                match storage.save_file(query.domain, query.version, query.path, file.path()) {
                    Ok(_) => {
                        // do nothing.
                    }
                    Err(e) => {
                        error!("save upload file failure {}", e);
                        bad_resp(format!("save upload file failure: {e}"), res);
                    }
                }
            }
            None => {
                bad_resp("file not found".to_string(), res);
            }
        }
    }

    /// Get files metadata
    ///
    /// Returns metadata for all files in a specific domain version.
    #[endpoint(
        responses(
            (status_code = 200, description = "Files metadata retrieved", body = Vec<entity::storage::ShortMetaData>),
            (status_code = 400, description = "Bad request"),
            (status_code = 401, description = "Unauthorized")
        )
    )]
    pub async fn get_files_metadata(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
        if let Ok(query) = req.parse_queries::<DomainWithVersionOption>() {
            match storage.get_files_metadata(query.domain, query.version) {
                Ok(data) => res.render(Json(data)),
                Err(err) => {
                    bad_resp(err.to_string(), res);
                }
            }
        } else {
            res.status_code(StatusCode::BAD_REQUEST);
        }
    }

    pub(super) fn _remove_domain_version(
        domain: Option<String>,
        max_reserve: Option<u32>,
        storage: &Arc<DomainStorage>,
    ) {
        let domains_info = if let Some(domain) = domain {
            storage
                .get_domain_info_by_domain(&domain)
                .map(|v| vec![v])
                .unwrap_or_default()
        } else {
            storage.get_domain_info().unwrap_or_else(|_| vec![])
        };
        for info in domains_info {
            let delete_versions = if let Some(max_reserve) = max_reserve {
                if let Some(mut max_version) =
                    info.current_version.or(info.versions.iter().max().copied())
                {
                    //TODO: fix it, get reserve versions by array index compare, rather than -.
                    max_version -= max_reserve;
                    info.versions
                        .into_iter()
                        .filter(|v| *v <= max_version)
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
    }

    /// Remove domain version
    ///
    /// Deletes old versions for domains. Can delete specific domain versions
    /// or all domains' versions, optionally keeping a maximum number of versions.
    #[endpoint(
        responses(
            (status_code = 200, description = "Domain versions removed"),
            (status_code = 400, description = "Bad request"),
            (status_code = 401, description = "Unauthorized")
        ),
        request_body = DeleteDomainVersionOption
    )]
    pub async fn remove_domain_version(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        if let Ok(query) = req.parse_json::<DeleteDomainVersionOption>().await {
            let storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
            _remove_domain_version(query.domain, query.max_reserve, storage);
        } else {
            res.status_code(StatusCode::BAD_REQUEST);
        }
    }

    //TODO: when delete and revoke occur currently. would have problems.
    /// Revoke version
    ///
    /// Reverts to a previous version for a domain.
    #[endpoint(
        responses(
            (status_code = 200, description = "Version revoked successfully"),
            (status_code = 404, description = "Domain or version not found"),
            (status_code = 400, description = "Bad request"),
            (status_code = 401, description = "Unauthorized")
        ),
        request_body = DomainWithVersionOption
    )]
    pub async fn revoke_version(req: &mut Request, res: &mut Response, depot: &mut Depot) {
        let domain_storage = depot.obtain::<Arc<DomainStorage>>().unwrap();
        if let Ok(query) = req.parse_json::<DomainWithVersionOption>().await {
            let DomainWithVersionOption { domain, version } = query;
            match domain_storage.get_domain_info_by_domain(&domain) {
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
                            Ok(_) => {}
                            Err(e) => {
                                bad_resp(e.to_string(), res);
                            }
                        }
                    } else {
                        res.status_code(StatusCode::NOT_FOUND);
                    }
                }
                None => {
                    res.status_code(StatusCode::NOT_FOUND);
                }
            };
        } else {
            res.status_code(StatusCode::BAD_REQUEST);
        }
    }
}

fn bad_resp(text: String, res: &mut Response) {
    res.status_code(StatusCode::BAD_REQUEST);
    res.render(text);
}

use crate::admin_server::service::_remove_domain_version;

fn build_async_job(
    domain_storage: Arc<DomainStorage>,
    max_reserve: Option<u32>,
    cron: &str,
) -> anyhow::Result<Task> {
    let domain_storage = domain_storage.clone();
    let body = move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            _remove_domain_version(None, max_reserve, &domain_storage);
        });
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

            let time = DateTime::from_timestamp_millis(time).unwrap();
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
