use crate::Config;
use anyhow::anyhow;
use std::sync::Arc;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{oneshot, Mutex};

pub struct HotReloadManager(Arc<Mutex<HotReloadState>>);

impl Clone for HotReloadManager {
    fn clone(&self) -> Self {
        HotReloadManager(self.0.clone())
    }
}

impl HotReloadManager {
    pub fn init(config: &Config) -> (HotReloadManager, Option<Receiver<()>>, Option<Receiver<()>>) {
        let (state, http, https) = HotReloadState::init(config);
        (HotReloadManager(Arc::new(Mutex::new(state))), http, https)
    }
    // this could only reload once. once error happens, need to restart.
    pub async fn reload(&self, state: HotReloadState) -> anyhow::Result<()> {
        let mut lock = self.0.lock().await;
        lock.reload()?;
        *lock = state;
        Ok(())
    }
}

pub struct HotReloadState {
    http_signal: Option<Sender<()>>,
    https_signal: Option<Sender<()>>,
}

impl HotReloadState {
    pub fn init(config: &Config) -> (Self, Option<Receiver<()>>, Option<Receiver<()>>) {
        // enable http
        let (http_signal, http_rx) = if config.port > 0 {
            let (tx, rx) = oneshot::channel::<()>();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        let (https_signal, https_rx) = if config.https.is_some() {
            let (tx, rx) = oneshot::channel::<()>();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        let manager = HotReloadState {
            http_signal,
            https_signal,
        };
        (manager, http_rx, https_rx)
    }

    fn reload(&mut self) -> anyhow::Result<()> {
        if let Some(signal) = self.http_signal.take() {
            let handle_http = match signal.send(()) {
                Ok(_) => {
                    tracing::debug!("send http stop signal success");
                    Ok(())
                }
                Err(_) => {
                    tracing::error!("send http stop signal error, need to restart");
                    Err(anyhow!("send http stop signal error, need to restart"))
                }
            };
            handle_http?;
        }

        if let Some(signal) = self.https_signal.take() {
            let handle_https = match signal.send(()) {
                Ok(_) => {
                    tracing::debug!("send https stop signal success");
                    Ok(())
                }
                Err(_) => {
                    tracing::error!("send https stop signal error, need to restart");
                    Err(anyhow!("send https stop signal error, need to restart"))
                }
            };
            handle_https?;
        }

        Ok(())
    }
}
