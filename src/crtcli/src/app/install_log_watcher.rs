use crate::app::{CrtClient, CrtClientBuilder, CrtClientError};
use futures::FutureExt;
use futures::future::{BoxFuture, Shared};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct InstallLogWatcherBuilder {
    crt_client: Arc<CrtClient>,
    config: InstallLogWatcherConfig,
}

#[derive(Clone, Debug)]
struct InstallLogWatcherConfig {
    polling_delay: Duration,
    wait_for_clear_on_start: bool,
    fetch_last_log_on_stop: bool,
}

impl Default for InstallLogWatcherConfig {
    fn default() -> Self {
        Self {
            polling_delay: Duration::from_millis(1000),
            wait_for_clear_on_start: false,
            fetch_last_log_on_stop: false,
        }
    }
}

pub struct InstallLogWatcherHandle {
    cancellation_token: CancellationToken,
    worker: Shared<BoxFuture<'static, ()>>,
}

#[derive(Debug, Clone)]
pub enum InstallLogWatcherEvent<'a> {
    Clear,
    Append(&'a str),
    FetchError(Arc<CrtClientError>),
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
    pub fn polling_delay(mut self, value: Duration) -> Self {
        self.config.polling_delay = value;
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

        let worker = {
            let cancellation_token = cancellation_token.clone();

            tokio::spawn(async move {
                let _ = InstallLogWatcher::new(
                    self.crt_client,
                    self.config,
                    handler,
                    cancellation_token,
                )
                .run()
                .await;
            })
            .map(|_| {
                // Ignoring any result with any error, e.g. JoinError, because it is not implement Clone
            })
        };

        InstallLogWatcherHandle {
            cancellation_token,
            worker: worker.boxed().shared(),
        }
    }
}

impl InstallLogWatcherHandle {
    pub async fn wait_until_stopped(&self) {
        self.worker.clone().await;
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

    // state
    last_log: String,
    clear_received: bool,
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
            last_log: String::new(),
            clear_received: !config.wait_for_clear_on_start,
            crt_client,
            config,
            handler,
            cancellation_token,
        }
    }

    async fn run(&mut self) -> Result<(), CrtClientError> {
        loop {
            self.fetch_and_handle_log_event().await;

            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    if self.config.fetch_last_log_on_stop && self.clear_received {
                        self.fetch_and_handle_log_event().await
                    }
                    break;
                },
                _ = tokio::time::sleep(self.config.polling_delay) => {},
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

    async fn fetch_and_handle_log_event(&mut self) {
        let current_log = match self.get_log_file().await {
            Ok(log) => log,
            Err(e) => {
                (self.handler)(InstallLogWatcherEvent::FetchError(Arc::new(e)));
                return;
            }
        };

        let is_first_log_msg = current_log.is_empty() && !self.clear_received;
        let is_appending_log_msg =
            !self.last_log.is_empty() && !current_log.starts_with(&self.last_log);

        // Determine if a clear event should be emitted.
        if is_first_log_msg || is_appending_log_msg {
            self.clear_received = true;
            (self.handler)(InstallLogWatcherEvent::Clear);
        }

        // If clear has been received, calculate and emit the delta.
        if self.clear_received {
            let delta = Self::strip_delta(&self.last_log, &current_log);
            if !delta.is_empty() {
                (self.handler)(InstallLogWatcherEvent::Append(delta));
            }
        }

        self.last_log = current_log;
    }
}
