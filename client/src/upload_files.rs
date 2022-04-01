use crate::API;
use anyhow::anyhow;
use console::style;
use futures::future::Either;
use futures::StreamExt;
use if_chain::if_chain;
use spa_server::admin_server::request::UpdateUploadingStatusOption;
use spa_server::domain_storage::{
    md5_file, GetDomainPositionStatus, ShortMetaData, UploadingStatus,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

pub fn upload_files(
    api: API,
    domain: String,
    version: Option<u32>,
    path: PathBuf,
    parallel: u32,
) -> anyhow::Result<()> {
    //println!("now is {:?}", fs::canonicalize("./")?);
    let path = fs::canonicalize(path)?;
    //println!("the upload path is {:?}", &path);
    println!("the upload path is {:?}", &path);
    if !path.is_dir() {
        return Err(anyhow!("{:?} is not a directory", path));
    }

    let prefix_path = path.to_str().unwrap().to_string();
    let version = get_upload_version(&api, &domain, version)?;
    println!("Begin to fetch server file metadata with md5, you may need to wait if there are large number of files.");
    let server_metadata = api.get_file_metadata(&domain, version)?;
    if !server_metadata.is_empty() {
        println!(
            "There are {} files already in server",
            server_metadata.len()
        );
    } else {
        println!("There are no files in server");
    }
    let server_metadata = server_metadata
        .into_iter()
        .map(|x| (x.path.clone(), x))
        .collect::<HashMap<String, ShortMetaData>>();

    let mut byte_buffer = vec![0u8; 1024 * 1024];
    let uploading_files = WalkDir::new(path)
        .into_iter()
        .filter_map(|entity| {
            if_chain! {
                if let Some(entity) = entity.ok();
                if let Some(metadata) = entity.metadata().ok();
                if metadata.is_file();
                if let Some(path) = entity.path().to_str().map(|x|x.to_string());
                then {
                    let key = path.replace(&prefix_path,"");
                    if server_metadata.get(&key).filter(|x|
                        x.length == metadata.len() &&
                        md5_file(entity.path(), &mut byte_buffer).filter(|md5|md5 == &x.md5).is_some()
                    ).is_none() {
                        Some((key, entity.path().to_path_buf()))
                    } else {
                        None
                    }
                }else {
                    None
                }
            }
        })
        .collect::<Vec<(String, PathBuf)>>();
    if uploading_files.is_empty() {
        return Err(anyhow!("There is no file to uploading"));
    }
    let uploading_file_count = uploading_files.len();
    println!(
        "{}",
        style(format!(
            "there are {} files to upload",
            uploading_file_count
        ))
        .green()
    );

    api.change_uploading_status(UpdateUploadingStatusOption {
        domain: domain.clone(),
        version,
        status: UploadingStatus::Uploading,
    })?;
    println!(
        "{}",
        style(format!(
            "Prepare files to upload and change {}:{} status:Uploading",
            &domain, version
        ))
        .green()
    );

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(parallel as usize)
        .enable_all()
        .build()?;

    println!(
        "{}",
        style(format!("Tokio init {} workers", parallel)).green()
    );
    let api = Arc::new(api);
    let domain: std::borrow::Cow<'static, str> = domain.into();
    let str_version: std::borrow::Cow<'static, str> = version.to_string().into();

    let process_count = Arc::new(AtomicU64::new(1));
    let upload_result = rt.block_on(async {
        let api = api.clone();
        futures::stream::iter(uploading_files.into_iter().map(|(key, path)| {
            let key: std::borrow::Cow<'static, str> = key.into();
            let r = retry_upload(
                api.as_ref(),
                domain.clone(),
                str_version.clone(),
                key,
                path,
                process_count.clone(),
            );
            r
        }))
        .buffer_unordered(parallel as usize)
        .map(|result| match result {
            Either::Left((key, count)) => {
                eprintln!("({}/{}) {} [Fail]", count, uploading_file_count, key);
                Some(key)
            }
            Either::Right((key, count)) => {
                println!("({}/{}) {} [Success]", count, uploading_file_count, key);
                None
            }
        })
        .collect::<Vec<Option<String>>>()
        .await
    });
    let fail_keys: Vec<String> = upload_result.into_iter().filter_map(|x| x).collect();
    if !fail_keys.is_empty() {
        return Err(anyhow!(
            "There are {} file(s) uploaded fail.",
            fail_keys.len()
        ));
    } else {
        api.change_uploading_status(UpdateUploadingStatusOption {
            domain: domain.clone().to_string(),
            version,
            status: UploadingStatus::Finish,
        })?;
    }
    Ok(())
}

async fn retry_upload<T: Into<Cow<'static, str>> + Clone>(
    api: &API,
    domain: T,
    version: T,
    key: T,
    path: PathBuf,
    count: Arc<AtomicU64>,
) -> Either<(String, u64), (String, u64)> {
    for retry in (0..3).into_iter() {
        let r = api
            .upload_file(domain.clone(), version.clone(), key.clone(), path.clone())
            .await;
        if r.is_ok() {
            let count = count.fetch_add(1, Ordering::SeqCst);
            return Either::Right((key.into().to_string(), count));
        }
    }
    let count = count.fetch_add(1, Ordering::SeqCst);
    Either::Left((key.into().to_string(), count))
}

fn get_upload_version(api: &API, domain: &str, version: Option<u32>) -> anyhow::Result<u32> {
    if let Some(version) = version {
        Ok(version)
    } else {
        let resp = api.get_upload_position(domain)?;
        match resp.status {
            GetDomainPositionStatus::NewDomain => {
                println!("domain:{} is new in server!", domain);
            }
            _ => {}
        };
        Ok(resp.version)
    }
}
