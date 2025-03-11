use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};

pub enum ChannelError {
    Closed,
}

/// Avtan Channel
pub struct AvtanChannel<T> {
    queue: Mutex<VecDeque<T>>,
    condvar: Condvar,
    closed: AtomicBool,
}

impl<T> AvtanChannel<T> {
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

mod avtan_furure {
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    };

    pub struct AvtanFuture {
        count: i32,
    }

    impl Future for AvtanFuture {
        type Output = i32;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.count += 1;
            println!("polling with result: {}", self.count);
            std::thread::sleep(Duration::from_secs(1));
            if self.count < 5 {
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(self.count)
            }
        }
    }
}
