use crate::API;
use anyhow::anyhow;
use console::style;
use futures::StreamExt;
use if_chain::if_chain;
use spa_server::admin_server::request::UpdateUploadingStatusOption;
use spa_server::domain_storage::{md5_file, DomainInfo, ShortMetaData, UploadingStatus};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::fmt::layer;
use walkdir::WalkDir;

pub fn upload_files(
    api: API,
    domain: String,
    version: u32,
    path: PathBuf,
    parallel: u32,
) -> anyhow::Result<()> {
    if path.is_dir() {
        return Err(anyhow!("{:?} is not a directory", path));
    }
    let prefix_path = path.to_str().unwrap().to_string();

    println!("Begin to fetch server file metadata with md5,\nyou may need to wait if there are large number of files.");
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
    let version: std::borrow::Cow<'static, str> = version.to_string().into();

    async fn retry_upload<T: Into<Cow<'static, str>> + Clone>(
        api: &API,
        domain: T,
        version: T,
        key: T,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        for retry in (0..3).into_iter() {
            let r = api
                .upload_file(domain.clone(), version.clone(), key.clone(), path.clone())
                .await;
            if r.is_ok() {
                break;
            }
        }
        Ok(())
    }

    rt.block_on(async move {
        let upload_result =
            futures::stream::iter(uploading_files.into_iter().map(|(key, path)| {
                let key: std::borrow::Cow<'static, str> = key.into();
                let r = retry_upload(api.as_ref(), domain.clone(), version.clone(), key, path);
                r
                //retry(api.clone().as_ref(), &key, &path)
            }))
            .buffer_unordered(parallel as usize)
            .collect::<Vec<anyhow::Result<()>>>()
            .await;

        //for (key, path) in uploading_files.into_iter() {}
    });

    Ok(())
}
