use crate::app::{CrtClient, CrtClientGenericError};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{spawn, JoinHandle};

pub struct InstallLogWatcher {
    crt_client: Arc<CrtClient>,
    handler: Option<fn(InstallLogWatcherEvent<'_>)>,

    pooling_delay: std::time::Duration,
    wait_for_clear_on_start: bool,
    fetch_last_log_on_stop: bool,
}

pub struct InstallLogWatcherHandle {
    cancellation_sender: Sender<()>,
    iteration_cvar: Arc<(Mutex<bool>, Condvar)>,
    _worker: JoinHandle<()>,
}

struct WorkerContext {
    crt_client: Arc<CrtClient>,
    handler: Option<fn(InstallLogWatcherEvent<'_>)>,
    cancellation_receiver: Receiver<()>,

    pooling_delay: std::time::Duration,
    wait_for_clear_on_start: bool,
    fetch_last_log_on_stop: bool,
    iteration_cvar: Arc<(Mutex<bool>, Condvar)>,
}

#[derive(Debug, Clone)]
pub enum InstallLogWatcherEvent<'c> {
    Clear(),
    Append(&'c str),
}

impl InstallLogWatcher {
    pub fn new(crt_client: Arc<CrtClient>) -> Self {
        Self {
            crt_client,
            pooling_delay: std::time::Duration::from_millis(1000),
            wait_for_clear_on_start: false,
            fetch_last_log_on_stop: false,
            handler: None,
        }
    }

    #[allow(dead_code)]
    pub fn pooling_delay(mut self, value: std::time::Duration) -> Self {
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

    pub fn with_handler(mut self, value: fn(InstallLogWatcherEvent<'_>)) -> Self {
        self.handler = Some(value);
        self
    }

    pub fn start(self) -> InstallLogWatcherHandle {
        let cancellation_channel = channel();
        let iteration_cvar = Arc::new((Mutex::new(false), Condvar::new()));

        let context = WorkerContext {
            crt_client: self.crt_client,
            handler: self.handler,

            cancellation_receiver: cancellation_channel.1,
            iteration_cvar: Arc::clone(&iteration_cvar),
            pooling_delay: self.pooling_delay,
            wait_for_clear_on_start: self.wait_for_clear_on_start,
            fetch_last_log_on_stop: self.fetch_last_log_on_stop,
        };

        InstallLogWatcherHandle {
            iteration_cvar,
            cancellation_sender: cancellation_channel.0,
            _worker: spawn(move || {
                context.main();
            }),
        }
    }
}

impl InstallLogWatcherHandle {
    pub fn wait_next_check_complete(&self) {
        let (lock, cvar) = &*self.iteration_cvar;
        let finished = lock.lock().unwrap();

        if !*finished {
            drop(cvar.wait(finished).unwrap())
        }
    }

    pub fn stop(&self) {
        let _ = self.cancellation_sender.send(());
    }
}

impl Drop for InstallLogWatcherHandle {
    fn drop(&mut self) {
        self.stop()
    }
}

impl WorkerContext {
    fn main(&self) {
        let mut iteration_ctx = WorkerIterationContext {
            current_log: None,
            timeout_received: false,
            clear_event_received: !self.wait_for_clear_on_start,
        };

        loop {
            let _ = process_iteration(self, &mut iteration_ctx);

            notify_iteration_complete(self);

            if iteration_ctx.timeout_received {
                break;
            }

            if self
                .cancellation_receiver
                .recv_timeout(self.pooling_delay)
                .is_ok()
            {
                iteration_ctx.timeout_received = true;

                if !self.fetch_last_log_on_stop {
                    break;
                }
            }
        }

        notify_iterations_finished(self);

        return;

        fn process_iteration(
            worker_ctx: &WorkerContext,
            iteration_ctx: &mut WorkerIterationContext,
        ) -> Result<(), CrtClientGenericError> {
            let log_file = worker_ctx
                .crt_client
                .package_installer_service()
                .get_log_file()?;

            if log_file.is_empty() && !iteration_ctx.clear_event_received {
                iteration_ctx.clear_event_received = true;
            }

            if iteration_ctx.current_log.is_none() {
                if !log_file.is_empty() {
                    handler_invoke(
                        worker_ctx,
                        iteration_ctx,
                        InstallLogWatcherEvent::Append(&log_file),
                    );
                }
            } else if log_file.starts_with(iteration_ctx.current_log.as_ref().unwrap()) {
                let delta = log_file
                    .strip_prefix(iteration_ctx.current_log.as_ref().unwrap())
                    .unwrap();

                if !delta.is_empty() {
                    handler_invoke(
                        worker_ctx,
                        iteration_ctx,
                        InstallLogWatcherEvent::Append(delta),
                    );
                }
            } else {
                handler_invoke(worker_ctx, iteration_ctx, InstallLogWatcherEvent::Clear());

                if !iteration_ctx.clear_event_received {
                    iteration_ctx.clear_event_received = true;
                }

                handler_invoke(
                    worker_ctx,
                    iteration_ctx,
                    InstallLogWatcherEvent::Append(&log_file),
                );
            }

            iteration_ctx.current_log = Some(log_file);

            Ok(())
        }

        fn handler_invoke(
            worker_ctx: &WorkerContext,
            iteration_ctx: &WorkerIterationContext,
            event: InstallLogWatcherEvent,
        ) {
            if let Some(handler) = worker_ctx.handler {
                if iteration_ctx.clear_event_received {
                    handler(event);
                }
            }
        }

        fn notify_iteration_complete(worker_ctx: &WorkerContext) {
            let (_, cvar) = &*worker_ctx.iteration_cvar;
            cvar.notify_all();
        }

        fn notify_iterations_finished(worker_ctx: &WorkerContext) {
            let (lock, cvar) = &*worker_ctx.iteration_cvar;
            let mut finished = lock.lock().unwrap();
            *finished = true;
            cvar.notify_all();
        }
    }
}

struct WorkerIterationContext {
    current_log: Option<String>,
    timeout_received: bool,
    clear_event_received: bool,
}
