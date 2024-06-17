use crate::acme::{ReloadACMEState, ReloadACMEStateMessage};
use crate::Config;
use anyhow::anyhow;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver as MReceiver, Sender as MSender};
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{oneshot, Mutex};

pub struct HotReloadManager(
    Arc<Mutex<OneShotReloadState>>,
    MSender<ReloadACMEStateMessage>,
);

impl Clone for HotReloadManager {
    fn clone(&self) -> Self {
        HotReloadManager(self.0.clone(), self.1.clone())
    }
}

impl HotReloadManager {
    pub fn init(
        config: &Config,
    ) -> (
        HotReloadManager,
        Option<Receiver<()>>,
        Option<Receiver<()>>,
        MReceiver<ReloadACMEStateMessage>,
    ) {
        let (state, http, https) = OneShotReloadState::init(config);

        let (acme_signal, acme_rx) = tokio::sync::mpsc::channel::<ReloadACMEStateMessage>(1);

        (
            HotReloadManager(Arc::new(Mutex::new(state)), acme_signal),
            http,
            https,
            acme_rx,
        )
    }
    // this could only reload once. once error happens, need to restart.
    pub async fn reload(
        &self,
        state: OneShotReloadState,
        reload_acme: Option<ReloadACMEState>,
    ) -> anyhow::Result<()> {
        let _ = self.1.send(ReloadACMEStateMessage(reload_acme)).await;
        let mut lock = self.0.lock().await;
        lock.reload()?;
        *lock = state;

        Ok(())
    }
}

pub struct OneShotReloadState {
    http_signal: Option<Sender<()>>,
    https_signal: Option<Sender<()>>,
}

impl OneShotReloadState {
    pub fn init(config: &Config) -> (Self, Option<Receiver<()>>, Option<Receiver<()>>) {
        // enable http
        let (http_signal, http_rx) = if config.http.is_some() {
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

        let manager = OneShotReloadState {
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
