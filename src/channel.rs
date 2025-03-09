use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, atomic::{AtomicBool, Ordering}};

pub enum ChannelError {
    Closed,
}

pub struct MPMCChannel<T> {
    queue: Mutex<VecDeque<T>>,
    condvar: Condvar,
    closed: AtomicBool,
}

impl<T> MPMCChannel<T> {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
            closed: AtomicBool::new(false),
        })
    }

    pub fn send(&self, item: T) -> Result<(), ChannelError> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(ChannelError::Closed);
        }

        let mut queue = self.queue.lock().unwrap();
        queue.push_back(item);
        self.condvar.notify_one();
        Ok(())
    }

    pub fn recv(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        loop {
            if let Some(item) = queue.pop_front() {
                return Some(item);
            }

            if self.closed.load(Ordering::SeqCst) {
                return None;
            }

            queue = self.condvar.wait(queue).unwrap();
        }
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.condvar.notify_all();
    }
}
