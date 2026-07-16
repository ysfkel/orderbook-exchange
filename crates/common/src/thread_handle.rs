use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

pub struct ThreadHandle {
    pub run: Arc<AtomicBool>,
    pub thread: Option<JoinHandle<()>>,
}

impl ThreadHandle {
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.run.store(false, Ordering::Release);
        if let Some(t) = self.thread.take() {
            if t.join().is_err() {
                tracing::error!("matching-engine thread panicked");
            }
        }
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}
