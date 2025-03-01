use crate::app::{CrtClient, CrtClientBuilder, CrtClientError};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct InstallLogWatcherBuilder {
    crt_client: Arc<CrtClient>,
    config: InstallLogWatcherConfig,
}

#[derive(Clone, Debug)]
struct InstallLogWatcherConfig {
    pooling_delay: Duration,
    wait_for_clear_on_start: bool,
    fetch_last_log_on_stop: bool,
}

impl Default for InstallLogWatcherConfig {
    fn default() -> Self {
        Self {
            pooling_delay: Duration::from_millis(1000),
            wait_for_clear_on_start: false,
            fetch_last_log_on_stop: false,
        }
    }
}

pub struct InstallLogWatcherHandle {
    cancellation_token: CancellationToken,
    finish_notify: Arc<Notify>,
    _worker: JoinHandle<Result<(), CrtClientError>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallLogWatcherEvent<'a> {
    Clear,
    Append(&'a str),
}

impl InstallLogWatcherBuilder {
    pub fn new(crt_client: Arc<CrtClient>) -> Self {
        Self {
            crt_client,
            config: InstallLogWatcherConfig::default(),
        }
    }

    pub fn new_with_new_session(crt_client: &CrtClient) -> Result<Self, CrtClientError> {
        let new_client = Arc::new(
            CrtClientBuilder::new(crt_client.credentials().clone())
                .with_new_memory_cache()
                .use_net_framework_mode(crt_client.is_net_framework())
                .danger_accept_invalid_certs(crt_client.is_insecure())
                .build()?,
        );
        Ok(Self::new(new_client))
    }

    #[allow(dead_code)]
    pub fn pooling_delay(mut self, value: Duration) -> Self {
        self.config.pooling_delay = value;
        self
    }

    #[allow(dead_code)]
    pub fn wait_for_clear_on_start(mut self, value: bool) -> Self {
        self.config.wait_for_clear_on_start = value;
        self
    }

    pub fn fetch_last_log_on_stop(mut self, value: bool) -> Self {
        self.config.fetch_last_log_on_stop = value;
        self
    }

    pub fn start<H>(self, handler: H) -> InstallLogWatcherHandle
    where
        H: Fn(InstallLogWatcherEvent<'_>) + Send + Sync + 'static,
    {
        let cancellation_token = CancellationToken::new();
        let finish_notify = Arc::new(Notify::new());

        let worker = {
            let cancellation_token = cancellation_token.clone();
            let finish_notify = finish_notify.clone();

            tokio::spawn(async move {
                let result = InstallLogWatcher::new(
                    self.crt_client,
                    self.config,
                    handler,
                    cancellation_token,
                )
                .run()
                .await;

                finish_notify.notify_waiters();
                result
            })
        };

        InstallLogWatcherHandle {
            cancellation_token,
            finish_notify,
            _worker: worker,
        }
    }
}

impl InstallLogWatcherHandle {
    pub async fn wait_until_stopped(&self) {
        self.finish_notify.notified().await;
    }

    pub fn stop(&self) {
        self.cancellation_token.cancel();
    }
}

impl Drop for InstallLogWatcherHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

struct InstallLogWatcher<H> {
    crt_client: Arc<CrtClient>,
    config: InstallLogWatcherConfig,
    handler: H,
    cancellation_token: CancellationToken,
}

impl<H> InstallLogWatcher<H>
where
    H: Fn(InstallLogWatcherEvent<'_>) + Send + 'static,
{
    fn new(
        crt_client: Arc<CrtClient>,
        config: InstallLogWatcherConfig,
        handler: H,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            crt_client,
            config,
            handler,
            cancellation_token,
        }
    }

    async fn run(&self) -> Result<(), CrtClientError> {
        let mut last_log = String::new();
        let mut clear_received = !self.config.wait_for_clear_on_start;

        loop {
            let current_log = self.get_log_file().await?;
            let is_first_log_msg = current_log.is_empty() && !clear_received;
            let is_appending_log_msg = !last_log.is_empty() && !current_log.starts_with(&last_log);

            // Determine if a clear event should be emitted.
            if is_first_log_msg || is_appending_log_msg {
                (self.handler)(InstallLogWatcherEvent::Clear);
                clear_received = true;
            }

            // If clear has been received, calculate and emit the delta.
            if clear_received {
                let delta = Self::strip_delta(&last_log, &current_log);
                if !delta.is_empty() {
                    (self.handler)(InstallLogWatcherEvent::Append(delta));
                }
            }

            last_log = current_log;

            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    if self.config.fetch_last_log_on_stop && clear_received {
                        self.handle_final_log(&last_log).await?;
                    }
                    break;
                },
                _ = tokio::time::sleep(self.config.pooling_delay) => {},
            }
        }

        Ok(())
    }

    async fn get_log_file(&self) -> Result<String, CrtClientError> {
        self.crt_client
            .package_installer_service()
            .get_log_file()
            .await
    }

    fn strip_delta<'a>(last_log: &str, current_log: &'a str) -> &'a str {
        if last_log.is_empty() || !current_log.starts_with(last_log) {
            current_log
        } else {
            &current_log[last_log.len()..]
        }
    }

    async fn handle_final_log(&self, last_log: &str) -> Result<(), CrtClientError> {
        let final_log = self.get_log_file().await?;

        if final_log != last_log {
            if !last_log.is_empty() && !final_log.starts_with(last_log) {
                (self.handler)(InstallLogWatcherEvent::Clear);
            }

            let delta = Self::strip_delta(last_log, &final_log);

            if !delta.is_empty() {
                (self.handler)(InstallLogWatcherEvent::Append(delta));
            }
        }

        Ok(())
    }
}
