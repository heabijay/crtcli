use crate::app::{CrtClient, CrtClientBuilder, CrtClientError};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{spawn, JoinHandle};
use std::time::Duration;

pub struct InstallLogWatcher {
    crt_client: Arc<CrtClient>,
    pooling_delay: Duration,
    wait_for_clear_on_start: bool,
    fetch_last_log_on_stop: bool,
}

pub struct InstallLogWatcherHandle {
    stop_signal: Sender<()>,
    worker_finished: Arc<(Mutex<bool>, Condvar)>,
    _worker: JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub enum InstallLogWatcherEvent<'a> {
    Clear,
    Append(&'a str),
}

impl InstallLogWatcher {
    pub fn new(crt_client: Arc<CrtClient>) -> Self {
        Self {
            crt_client,
            pooling_delay: Duration::from_millis(1000),
            wait_for_clear_on_start: false,
            fetch_last_log_on_stop: false,
        }
    }

    pub fn new_with_new_session(crt_client: &CrtClient) -> Result<Self, CrtClientError> {
        Ok(Self {
            crt_client: Arc::new(
                CrtClientBuilder::new(crt_client.credentials().clone())
                    .with_new_memory_cache()
                    .use_net_framework_mode(crt_client.is_net_framework())
                    .danger_accept_invalid_certs(crt_client.is_insecure())
                    .build()?,
            ),
            pooling_delay: Duration::from_millis(1000),
            wait_for_clear_on_start: false,
            fetch_last_log_on_stop: false,
        })
    }

    #[allow(dead_code)]
    pub fn pooling_delay(mut self, value: Duration) -> Self {
        self.pooling_delay = value;
        self
    }

    #[allow(dead_code)]
    pub fn wait_for_clear_on_start(mut self, value: bool) -> Self {
        self.wait_for_clear_on_start = value;
        self
    }

    pub fn fetch_last_log_on_stop(mut self, value: bool) -> Self {
        self.fetch_last_log_on_stop = value;
        self
    }

    pub fn start<H>(self, handler: H) -> InstallLogWatcherHandle
    where
        H: Fn(InstallLogWatcherEvent<'_>) + Send + 'static,
    {
        let (stop_signal_tx, stop_signal_rx) = channel();
        let worker_finished = Arc::new((Mutex::new(false), Condvar::new()));
        let worker_finished_clone = Arc::clone(&worker_finished);

        let worker = spawn(move || {
            if let Err(e) = self.run_worker(handler, stop_signal_rx, worker_finished_clone) {
                eprintln!("InstallLogWatcher worker error: {}", e);
            }
        });

        InstallLogWatcherHandle {
            stop_signal: stop_signal_tx,
            worker_finished,
            _worker: worker,
        }
    }

    fn run_worker<H>(
        &self,
        handler: H,
        stop_signal: Receiver<()>,
        worker_finished: Arc<(Mutex<bool>, Condvar)>,
    ) -> Result<(), CrtClientError>
    where
        H: Fn(InstallLogWatcherEvent<'_>) + Send + 'static,
    {
        let mut last_log = String::new();
        let mut clear_received = !self.wait_for_clear_on_start;

        loop {
            let current_log = self.crt_client.package_installer_service().get_log_file()?;

            if current_log.is_empty() && !clear_received {
                handler(InstallLogWatcherEvent::Clear);
                clear_received = true;
            } else if current_log != last_log {
                if !last_log.is_empty() && !current_log.starts_with(&last_log) {
                    handler(InstallLogWatcherEvent::Clear);
                    clear_received = true;
                }

                if clear_received {
                    let delta = if last_log.is_empty() || !current_log.starts_with(&last_log) {
                        &current_log
                    } else {
                        &current_log[last_log.len()..]
                    };

                    if !delta.is_empty() {
                        handler(InstallLogWatcherEvent::Append(delta));
                    }
                }
            }

            last_log = current_log;

            if stop_signal.recv_timeout(self.pooling_delay).is_ok() {
                if self.fetch_last_log_on_stop {
                    let final_log = self.crt_client.package_installer_service().get_log_file()?;
                    if final_log != last_log {
                        if !last_log.is_empty() && !final_log.starts_with(&last_log) {
                            handler(InstallLogWatcherEvent::Clear);
                        }
                        let delta = if last_log.is_empty() || !final_log.starts_with(&last_log) {
                            &final_log
                        } else {
                            &final_log[last_log.len()..]
                        };

                        if !delta.is_empty() {
                            handler(InstallLogWatcherEvent::Append(delta));
                        }
                    }
                }
                break;
            }
        }

        // Notify that the worker has finished
        let (lock, cvar) = &*worker_finished;
        let mut finished = lock.lock().unwrap();
        *finished = true;
        cvar.notify_all();

        Ok(())
    }
}

impl InstallLogWatcherHandle {
    pub fn wait_until_stopped(&self) {
        let (lock, cvar) = &*self.worker_finished;
        let mut finished = lock.lock().unwrap();
        while !*finished {
            finished = cvar.wait(finished).unwrap();
        }
    }

    pub fn stop(&self) {
        let _ = self.stop_signal.send(());
    }
}

impl Drop for InstallLogWatcherHandle {
    fn drop(&mut self) {
        self.stop();
        // self.wait_until_stopped();
    }
}
