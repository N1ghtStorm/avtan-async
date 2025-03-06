use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use futures::task::noop_waker;

pub struct Executor {
    tasks: Arc<Mutex<VecDeque<Pin<Box<dyn Future<Output = ()> + Send>>>>>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn spawn(&self, task: impl Future<Output = ()> + Send + 'static) {
        self.tasks.lock().unwrap().push_back(Box::pin(task));
    }

    pub fn run(&self) {
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

pub async fn hello_async() {
    println!("Hello from async runtime!");
}
