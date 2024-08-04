use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use base64::prelude::*;
use chrono::{DateTime, TimeZone, Utc};
use dashmap::DashMap;
use delay_timer::prelude::{DelayTimer, Task, TaskBuilder};
use if_chain::if_chain;
use lazy_static::lazy_static;
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use regex::Regex;
use rustls::sign::CertifiedKey;
use small_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, NewAccount,
    NewOrder, OrderStatus,
};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

use crate::config::{get_host_path_from_domain, ACMEConfig, ACMEType, Config};
use entity::storage::{CertInfo};
use crate::domain_storage::DomainStorage;
use crate::tls::load_ssl_file;

const ACME_DIR: &str = "acme";
const CHALLENGE_DIR: &str = "challenge";
pub const ACME_CHALLENGE: &str = "/.well-known/acme-challenge/";

const CERTIFICATE_PRIVATE_KEY_REGEX_STR: &str =
    "^certificate_(stage|prod|ci)_(?P<domain>.*)\\.key$";
//"[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})+$";

lazy_static! {
    pub static ref PRIVATE_KEY_NAME_REGEX: Regex =
        Regex::new(CERTIFICATE_PRIVATE_KEY_REGEX_STR).unwrap();
}

pub type ChallengePath = Arc<RwLock<Option<PathBuf>>>;

#[derive(Clone)]
struct ACMEProvider {
    path: Arc<PathBuf>,
    acme_type: ACMEType,
    account: Account,
}

impl ACMEProvider {
    fn init(acme_config: ACMEConfig, default_file_dir: &Path) -> anyhow::Result<Self> {
        let path = match acme_config.dir {
            Some(path) => {
                let path = PathBuf::from(path);
                if path.exists() {
                    path
                } else {
                    bail!("acme dir path: {path:?} does not exists")
                }
            }
            None => {
                let path = default_file_dir.join(ACME_DIR);
                if !path.exists() {
                    fs::create_dir(&path)?;
                }
                path
            }
        };
        let emails = acme_config.emails;

        //let stage = true;//acme_config.stage;
        let acme_type = acme_config.acme_type;
        let account_file_path = Self::get_account_file_path(&emails, &acme_type, &path);
        let emails: Vec<&str> = emails.iter().map(std::ops::Deref::deref).collect();
        let account = Self::init_or_get_account(
            &account_file_path,
            &acme_type,
            &emails,
            acme_config.ci_ca_path.as_ref(),
        )?;
        let challenge_path = path.join(CHALLENGE_DIR);
        if !challenge_path.exists() {
            fs::create_dir(&challenge_path)?
        }

        Ok(Self {
            path: Arc::new(path),
            acme_type, //acme_config.stage,
            account,
        })
    }

    fn get_account_file_path(emails: &[String], acme_type: &ACMEType, dir: &PathBuf) -> PathBuf {
        let url = acme_type.url();
        let email = format!("{}_{}", url, emails.join(","));
        let file_name = BASE64_URL_SAFE_NO_PAD.encode(email);
        let file_name = format!("account_{}_{file_name}", acme_type.as_str());
        dir.join(file_name)
    }
    fn init_or_get_account(
        path: &PathBuf,
        acme_type: &ACMEType,
        emails: &[&str],
        ci_ca_path: Option<&String>,
    ) -> anyhow::Result<Account> {
        let agent = if matches!(acme_type, ACMEType::CI) {
            let mut roots = rustls::RootCertStore::empty();
            let mut reader = std::io::BufReader::new(File::open(ci_ca_path.unwrap())?);
            let cert = rustls_pemfile::certs(&mut reader).map(|v| v.unwrap());
            roots.add_parsable_certificates(cert);
            let tls_config = rustls::ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth();
            ureq::builder()
                .https_only(true)
                .tls_config(Arc::new(tls_config))
                .build()
        } else {
            ureq::builder().https_only(true).build()
        };

        let account = if path.exists() {
            let file = File::open(path)?;
            let credentials: AccountCredentials = serde_json::from_reader(file)?;
            info!("get acme account from file: {path:?}");
            Account::from_credentials_with_agent(credentials, agent)?
        } else {
            let (account, credentials) = Account::create_with_agent(
                &NewAccount {
                    contact: emails,
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                acme_type.url(),
                None,
                agent,
            )?;
            let file = File::create(path)?;
            serde_json::to_writer_pretty(file, &credentials)?;
            info!("create acme account file: {path:?} and write credentials");
            account
        };
        Ok(account)
    }

    async fn create_order_and_auth(
        &self,
        domain: String,
        challenge_path: Arc<PathBuf>,
        alias: Option<Vec<String>>,
    ) -> anyhow::Result<(PathBuf, PathBuf)> {
        //
        let identifier = Identifier::Dns(domain.clone());
        let mut identifiers = alias.map(|list| list.into_iter().map(Identifier::Dns).collect::<Vec<Identifier>>()).unwrap_or_else(|| vec![]);
        identifiers.push(identifier);
        let mut order = self.account.new_order(&NewOrder {
            identifiers: &identifiers,
        })?;
        let state = order.state();
        debug!("domain:{domain} order state:{:#?}", state);
        assert!(matches!(state.status, OrderStatus::Pending));
        let authorizations = order.authorizations()?;
        let mut names = vec![];
        for authz in &authorizations {
            match authz.status {
                AuthorizationStatus::Pending => {}
                AuthorizationStatus::Valid => continue,
                _ => {
                    warn!("authorization : {authz:#?}")
                },
            }
            let challenge = authz
                .challenges
                .iter()
                .find(|c| c.r#type == ChallengeType::Http01)
                .ok_or_else(|| anyhow!("no http01 challenge found for domain:{domain}"))?;
            let Identifier::Dns(identifier) = &authz.identifier;
            let token = challenge.token.clone();

            let key_authorization = order.key_authorization(challenge);
            let challenge_domain_token_path = get_challenge_path(&challenge_path, identifier, &token);
            fs::write(challenge_domain_token_path, key_authorization.as_str())?;
            names.push(identifier.clone());
            order.set_challenge_ready(&challenge.url)?;
        }
        // get token
        let mut retries: u32 = 0;
        let state = loop {
            tokio::time::sleep(Duration::from_secs((1 << (retries + 1)).min(10))).await;
            retries += 1;
            if let Err(e) = order.refresh() {
                warn!("domain: {domain} order refresh failure at {retries}: {e}");
                continue;
            }
            let state = order.state();
            if let OrderStatus::Ready | OrderStatus::Invalid | OrderStatus::Valid = state.status {
                info!("domain: {domain} order state: {:#?}", state);
                break state;
            }
            if retries > 10 {
                error!("domain: {domain} order is not ready {state:#?} {retries}");
                bail!("domain: {domain} order is not ready")
            }
        };
        if state.status == OrderStatus::Invalid {
            bail!("domain: {domain} order is invalid")
        }

        let mut params = CertificateParams::new(names);
        params.distinguished_name = DistinguishedName::new();
        let cert = Certificate::from_params(params).unwrap();
        let csr = cert.serialize_request_der()?;
        order
            .finalize(&csr)
            .with_context(|| format!("{domain} csr failure"))?;

        let mut retries: u32 = 0;
        let cert_chain_pem = loop {
            match order.certificate() {
                Ok(Some(cert_chain_pem)) => break cert_chain_pem,
                _ => sleep(Duration::from_secs(1)).await,
            }
            retries += 1;
            warn!("domain: {domain} order certificate failure at {retries}");
            if retries > 20 {
                bail!("domain {domain} cert not received")
            }
        };

        debug!("domain: {domain} get cert successful, public cert {cert_chain_pem}");
        let private_key = cert.serialize_private_key_pem();
        let (public_cert_path, private_key_path) = self.get_certificate_file_names(&domain);

        let mut private_key_file = File::create(&private_key_path)
            .with_context(|| format!("create file {private_key_path:?} failure"))?;
        let mut pubic_cert_file = File::create(&public_cert_path)
            .with_context(|| format!("create file {public_cert_path:?} failure"))?;

        private_key_file
            .write_all(private_key.as_bytes())
            .with_context(|| format!("write file {private_key_path:?} failure"))?;
        pubic_cert_file
            .write_all(cert_chain_pem.as_bytes())
            .with_context(|| format!("write file {public_cert_path:?} failure"))?;

        Ok((public_cert_path, private_key_path))
    }

    fn get_certificate_file_names(&self, domain: &str) -> (PathBuf, PathBuf) {
        //let env = self.acme
        let env = self.acme_type.as_str();
        (
            self.path.join(format!("certificate_{env}_{domain}.pem")),
            self.path.join(format!("certificate_{env}_{domain}.key")),
        )
    }
}

//#[derive(Debug)]
pub struct RefreshDomainMessage(pub Vec<String>);
pub struct ReloadACMEState {
    provider: Arc<ACMEProvider>,
    disabled_hosts: Arc<HashSet<String>>,
    alias_hosts: Arc<HashMap<String, Vec<String>>>,
    hosts: Arc<HashSet<String>>,
    pub challenge_path: Arc<PathBuf>,
}

pub struct ReloadACMEStateMessage(pub Option<ReloadACMEState>);

#[derive(Clone)]
pub struct ACMEManager {
    pub sender: Sender<RefreshDomainMessage>,
    pub certificate_map: Arc<DashMap<String, Arc<CertifiedKey>>>,
    pub challenge_dir: Arc<RwLock<Option<PathBuf>>>,
}

impl ACMEManager {
    pub fn init(
        config: &Config,
        domain_storage: Arc<DomainStorage>,
        reload_rx: Option<Receiver<ReloadACMEStateMessage>>,
        delay_timer: &DelayTimer,
    ) -> anyhow::Result<Self> {
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<RefreshDomainMessage>(2);

        let certificate_map = Arc::new(DashMap::new());
        let acme_config = config.https.as_ref().and_then(|x| x.acme.clone());
        let reload_acme_state = if let Some(acme_config) = acme_config {
            Some(Self::init_acme_provider_and_certificate(
                config,
                acme_config,
                domain_storage,
                certificate_map.clone(),
            )?)
        } else {
            None
        };

        let challenge_path = reload_acme_state
            .as_ref()
            .map(|r| r.challenge_path.as_ref().clone());

        let _certificate_map = certificate_map.clone();
        tokio::spawn(async move {
            let mut reload_acme_state = reload_acme_state;
            async fn refresh(
                refresh_domains: Vec<String>,
                reload_acme_state: &Option<ReloadACMEState>,
                _certificate_map: Arc<DashMap<String, Arc<CertifiedKey>>>,
            ) {
                if let Some(ReloadACMEState {
                    provider,
                    disabled_hosts,
                    hosts,
                    challenge_path,
                    alias_hosts,
                }) = reload_acme_state
                {
                    let refresh_domains = if refresh_domains.is_empty() {
                        hosts.iter().map(|v| v.to_string()).collect()
                    } else {
                        refresh_domains
                    };
                    let refresh_domains: HashSet<String> = refresh_domains
                        .into_iter()
                        .filter_map(|domain| {
                            if disabled_hosts.contains(&domain) {
                                return None;
                            }
                            match _certificate_map.get(&domain) {
                                None => Some(domain),
                                Some(certified_key) => {
                                    let need_refresh = cert_need_refresh(&certified_key)
                                        .unwrap_or_else(|e| {
                                            warn!("get {domain} cert info failure:{e}");
                                            true
                                        });
                                    if need_refresh {
                                        Some(domain)
                                    } else {
                                        None
                                    }
                                }
                            }
                        })
                        .collect();
                    // TODO: handle error
                    let _ = ACMEManager::renewal_certificates(
                        provider.clone(),
                        _certificate_map.clone(),
                        refresh_domains,
                        challenge_path.clone(),
                        alias_hosts.clone(),
                    )
                    .await;
                }
            }
            if let Some(mut reload_rx) = reload_rx {
                loop {
                    tokio::select! {
                        Some(RefreshDomainMessage(refresh_domains)) = receiver.recv() => {
                           refresh(refresh_domains, &reload_acme_state, _certificate_map.clone()).await
                        }
                        Some(ReloadACMEStateMessage(state)) = reload_rx.recv() => {
                            reload_acme_state = state;
                        }
                    }
                }
            } else {
                while let Some(RefreshDomainMessage(refresh_domains)) = receiver.recv().await {
                    refresh(
                        refresh_domains,
                        &reload_acme_state,
                        _certificate_map.clone(),
                    )
                    .await
                }
            }
        });

        delay_timer
            .add_task(Self::create_daily_trigger_task(sender.clone())?)
            .with_context(|| "add daily check cert job fail")?;

        let _sender = sender.clone();
        // trigger
        tokio::spawn(async move {
            sleep(Duration::from_secs(3)).await;
            let _ = _sender.send(RefreshDomainMessage(vec![])).await;
        });
        Ok(Self {
            sender,
            certificate_map,
            challenge_dir: Arc::new(RwLock::new(challenge_path)),
        })
    }

    pub fn init_acme_provider_and_certificate(
        config: &Config,
        acme_config: ACMEConfig,
        domain_storage: Arc<DomainStorage>,
        certificate_map: Arc<DashMap<String, Arc<CertifiedKey>>>,
    ) -> anyhow::Result<ReloadACMEState> {
        let provider = ACMEProvider::init(acme_config, &PathBuf::from(&config.file_dir))?;

        let challenge_path = provider.path.join(CHALLENGE_DIR);
        if !challenge_path.exists() {
            fs::create_dir(&challenge_path)?;
        }
        let disable_https_hosts: HashSet<String> = config
            .domains
            .iter()
            .filter_map(|v| {
                if v.https
                    .as_ref()
                    .map(|v| v.disable_acme)
                    .unwrap_or_else(|| false)
                {
                    Some(get_host_path_from_domain(&v.domain).0.to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut hosts: HashSet<String> = domain_storage
            .get_domain_info()?
            .into_iter()
            .filter_map(|info| {
                let domain = info.domain;
                let host = get_host_path_from_domain(&domain).0;
                if disable_https_hosts.contains(host) {
                    None
                } else {
                    Some(host.to_string())
                }
            })
            .collect();

        let certificates = get_certificates_files(&provider.acme_type, &provider.path);

        for (domain, cert) in certificates {
            if !disable_https_hosts.contains(&domain) {
                // prevent certificates get dirty file when reload
                if !certificate_map.contains_key(&domain) {
                    certificate_map.insert(domain.clone(), Arc::new(cert));
                }
                hosts.insert(domain);
            }
        }
        let mut alias_map = HashMap::new();
        for domain in &config.domains {
            if let Some(alias) = domain.alias.as_ref() {
                if !alias.is_empty() {
                    alias_map.insert(domain.domain.clone(), alias.iter().map(|x|x.clone()).collect());
                }
            }
        }
        Ok(ReloadACMEState {
            provider: Arc::new(provider),
            hosts: Arc::new(hosts),
            disabled_hosts: Arc::new(disable_https_hosts),
            challenge_path: Arc::new(challenge_path),
            alias_hosts: Arc::new(alias_map),
        })
    }

    async fn renewal_certificates(
        provider: Arc<ACMEProvider>,
        certificate_map: Arc<DashMap<String, Arc<CertifiedKey>>>,
        renewal_domains: HashSet<String>,
        challenge_path: Arc<PathBuf>,
        alias_map: Arc<HashMap<String, Vec<String>>>,
    ) {
        for domain in renewal_domains {
            debug!("{domain} begin to get cert");
            let alias = alias_map.get(&domain).map(|x|x.clone());
            match provider
                .create_order_and_auth(domain.clone(), challenge_path.clone(), alias)
                .await
            {
                Ok((public_cert, private_key)) => {
                    certificate_map.insert(
                        domain.clone(),
                        Arc::new(load_ssl_file(&public_cert, &private_key).unwrap()),
                    );
                    info!("{domain} renewal cert successfully")
                }
                Err(err) => {
                    warn!("{domain} renewal cert failure: {err}")
                }
            }
            sleep(Duration::from_secs(20)).await
        }
    }

    fn create_daily_trigger_task(sender: Sender<RefreshDomainMessage>) -> anyhow::Result<Task> {
        let task = move || {
            let _ = sender.send(RefreshDomainMessage(vec![]));
        };

        Ok(TaskBuilder::default()
            .set_frequency_repeated_by_days(1)
            .set_task_id(2)
            .set_maximum_parallel_runnable_num(1)
            .spawn_routine(task)?)
    }
    pub async fn add_new_domain(&self, host: &str) {
        let _ = self
            .sender
            .send(RefreshDomainMessage(vec![host.to_string()]))
            .await;
    }

    pub async fn get_cert_data(&self, host: Option<&String>) -> anyhow::Result<Vec<CertInfo>> {
        let result = match host {
            Some(host) => match self.certificate_map.get(host) {
                Some(value) => {
                    let [begin, end] = get_cert_validate_time(&value)?;
                    vec![CertInfo {
                        begin,
                        end,
                        host: host.to_owned(),
                    }]
                }
                None => vec![],
            },
            None => self
                .certificate_map
                .iter()
                .filter_map(|item| match get_cert_validate_time(&item) {
                    Ok([begin, end]) => Some(CertInfo {
                        begin,
                        end,
                        host: item.key().to_string(),
                    }),
                    Err(error) => {
                        warn!("get {} cert fail:{error}", item.key());
                        None
                    }
                })
                .collect(),
        };
        Ok(result)
    }
}

fn get_cert_validate_time(certified_key: &CertifiedKey) -> anyhow::Result<[DateTime<Utc>; 2]> {
    let cert = certified_key.end_entity_cert()?;
    let (_, cert) = x509_parser::parse_x509_certificate(cert.as_ref())?;
    let validity = cert.validity();
    Ok([validity.not_before, validity.not_after]
        .map(|t| Utc.timestamp_opt(t.timestamp(), 0).unwrap()))
}

fn cert_need_refresh(certified_key: &CertifiedKey) -> anyhow::Result<bool> {
    let [begin, end] = get_cert_validate_time(certified_key)?;
    let now = Utc::now();
    Ok(now < begin || now > end || end - now < chrono::Duration::days(9))
}

fn get_certificates_files(acme_type: &ACMEType, path: &PathBuf) -> Vec<(String, CertifiedKey)> {
    let env = format!("certificate_{}_", acme_type.as_str());
    WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entity| {
            if_chain! {
                if let Some(entity) = entity.ok();
                if let Some(metadata) = entity.metadata().ok();
                if metadata.is_file();
                if let Some(file_name) = entity.file_name().to_str();
                if file_name.starts_with(&env);// must keep this to filter different env
                if let Some(r) = PRIVATE_KEY_NAME_REGEX.captures(file_name);
                if let Some(domain) = r.name("domain").map(|v|v.as_str());
                let cert_path = path.join(format!("{env}{domain}.pem"));
                if cert_path.exists();
                then {
                    let private_key_path = entity.path().to_path_buf();
                    match load_ssl_file(&cert_path, &private_key_path) {
                        Ok(key) => {
                            Some((domain.to_string(),key))
                        }
                        Err(e) => {
                            warn!("load {domain} cert failure: {e}");
                            None
                        }
                    }
                }else {
                    None
                }
            }
        })
        .collect()
}

pub fn get_challenge_path(prefix: &Path, host: &str, token: &str) -> PathBuf {
    prefix.join(format!("{host}_{token}.token"))
}

#[cfg(test)]
mod test {
    use base64::prelude::*;
    use std::path::PathBuf;
    use x509_parser::nom::AsBytes;

    use crate::acme::PRIVATE_KEY_NAME_REGEX;
    use crate::config::ACMEType;
    use crate::tls::load_ssl_file;
    use crate::LOCAL_HOST;

    #[test]
    fn test_account_name() {
        let email = "_";
        let hash = BASE64_URL_SAFE_NO_PAD.encode(email);
        println!("hash: {hash}")
    }

    #[test]
    fn test_private_key_regex() {
        assert!(PRIVATE_KEY_NAME_REGEX.is_match("certificate_stage_www.example.com.key"));
        assert!(!PRIVATE_KEY_NAME_REGEX.is_match("certificate_stag_www.example.com.key"));

        let r = PRIVATE_KEY_NAME_REGEX
            .captures("certificate_stage_www.example.com.key")
            .unwrap();
        let r = r.name("domain").unwrap().as_str();
        assert_eq!(r, "www.example.com");
    }

    #[test]
    fn test_load_ssl_file() {
        let certified_key = load_ssl_file(
            &PathBuf::from(format!("tests/data/{LOCAL_HOST}.pem")),
            &PathBuf::from(format!("tests/data/{LOCAL_HOST}.key")),
        )
        .unwrap();

        let z = certified_key.end_entity_cert().unwrap().as_bytes();

        let (_, z) = x509_parser::parse_x509_certificate(z).unwrap();
        println!("{:?}", z.validity);
    }

    #[ignore]
    #[test]
    fn test_load_ci_file() {
        let certified_key = load_ssl_file(
            &PathBuf::from(format!(
                "../tests/data/web/acme/certificate_ci_{LOCAL_HOST}.pem"
            )),
            &PathBuf::from(format!(
                "../tests/data/web/acme/certificate_ci_{LOCAL_HOST}.key"
            )),
        )
        .unwrap();

        let z = certified_key.end_entity_cert().unwrap().as_bytes();

        let (_, z) = x509_parser::parse_x509_certificate(z).unwrap();
        println!("{:?}", z.validity);
    }

    #[test]
    fn test_matches() {
        let r = &ACMEType::CI;
        let r = matches!(r, ACMEType::CI);
        assert!(r);
    }

    #[test]
    fn get_challenge_path_safe_test() {
        let root = PathBuf::from("/tmp");
        let challenge_token_path = super::get_challenge_path(&root, "www.example.com", "/../");
        println!("{}", challenge_token_path.display().to_string());
        assert!(challenge_token_path.canonicalize().is_err());
    }
}
