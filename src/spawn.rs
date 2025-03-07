use futures::task::noop_waker;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

pub struct Executor {
    tasks: Arc<Mutex<VecDeque<Pin<Box<dyn Future<Output = ()> + Send>>>>>,
}

impl Executor {

    #[inline(always)]
    pub fn new() -> Self {
        Executor {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    #[inline(always)]
    pub fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        self.tasks.lock().unwrap().push_back(Box::pin(task));
    }

    pub fn block_on(&self) {
        while let Some(mut task) = self.tasks.lock().unwrap().pop_front() {
            let waker = noop_waker();
            let mut context = Context::from_waker(&waker);
            match task.as_mut().poll(&mut context) {
                Poll::Pending => {
                    self.tasks.lock().unwrap().push_back(task);
                }
                Poll::Ready(()) => {}
            }
        }
    }
}

#[inline(always)]
pub async fn hello_async() {
    println!("Hello from async runtime!");
}
