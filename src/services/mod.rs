mod tui;

pub use self::tui::Tui;

use std::{
    sync::{
        atomic::{self, AtomicBool, Ordering},
        Arc,
    },
    thread,
};

/// A service that is a loop that does something
pub trait LoopService: Send + Sync {
    fn run_iteration(&mut self) -> eyre::Result<()>;
}

#[derive(Clone)]
pub struct ServiceControl {
    stop_all: Arc<AtomicBool>,
}

impl ServiceControl {
    pub fn new() -> Self {
        Self {
            stop_all: Default::default(),
        }
    }

    // Notify all spawned service instances to shutdown
    pub fn send_stop_to_all(&self) {
        self.stop_all.store(true, Ordering::SeqCst);
    }

    pub fn spawn_loop(&self, mut service: impl LoopService + 'static) -> JoinHandle {
        self.spawn_loop_raw(move || service.run_iteration())
    }

    pub fn spawn_loop_raw<F>(&self, mut f: F) -> JoinHandle
    where
        F: FnMut() -> eyre::Result<()> + Send + Sync + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));

        JoinHandle::new(
            stop.clone(),
            thread::spawn({
                let stop_all = self.stop_all.clone();
                move || match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    while !stop.load(atomic::Ordering::SeqCst)
                        && !stop_all.load(atomic::Ordering::SeqCst)
                    {
                        if let Err(e) = f() {
                            stop_all.store(true, atomic::Ordering::SeqCst);
                            return Err(e);
                        }
                    }
                    Ok(())
                })) {
                    Err(_e) => {
                        stop_all.store(true, atomic::Ordering::SeqCst);
                        eyre::bail!("service panicked");
                    }
                    Ok(res) => res,
                }
            }),
        )
    }
}

/// Simple thread join wrapper that joins the thread on drop
///
/// TODO: Would it be better to have it set the `stop` flag toc terminate all threads
/// on drop?
pub struct JoinHandle {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<eyre::Result<()>>>,
}

impl JoinHandle {
    fn new(stop: Arc<AtomicBool>, handle: thread::JoinHandle<eyre::Result<()>>) -> Self {
        JoinHandle {
            stop,
            thread: Some(handle),
        }
    }
}

impl JoinHandle {
    fn join_mut(&mut self) -> eyre::Result<()> {
        if let Some(h) = self.thread.take() {
            h.join()
                .map_err(|e| eyre::format_err!("join failed: {:?}", e))?
        } else {
            Ok(())
        }
    }

    #[allow(unused)]
    pub fn join(mut self) -> eyre::Result<()> {
        self.join_mut()
    }
}

impl Drop for JoinHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.join_mut().expect("not failed")
    }
}
