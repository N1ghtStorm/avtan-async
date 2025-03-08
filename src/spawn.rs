use futures::task::ArcWake;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

// Task represents a future with its waker
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    wake_time: Option<Instant>,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

// Implement ArcWake for Task to enable proper waking
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // When a task is woken, push it back to the executor
        EXECUTOR.with(|e| {
            let executor = e.borrow();
            executor.push_task(arc_self.clone());
        });
    }
}

pub struct Executor {
    ready_tasks: Mutex<VecDeque<Arc<Task>>>,
    sleeping_tasks: Mutex<HashMap<Instant, Vec<Arc<Task>>>>,
}

// Thread-local executor instance
thread_local! {
    static EXECUTOR: std::cell::RefCell<Executor> = std::cell::RefCell::new(Executor::new());
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            ready_tasks: Mutex::new(VecDeque::new()),
            sleeping_tasks: Mutex::new(HashMap::new()),
        }
    }

    fn push_task(&self, task: Arc<Task>) {
        if let Some(wake_time) = task.wake_time {
            if wake_time <= Instant::now() {
                self.ready_tasks.lock().unwrap().push_back(task);
            } else {
                self.sleeping_tasks
                    .lock()
                    .unwrap()
                    .entry(wake_time)
                    .or_default()
                    .push(task);
            }
        } else {
            self.ready_tasks.lock().unwrap().push_back(task);
        }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Box::pin(future),
            wake_time: None,
        });
        self.push_task(task);
    }

    pub fn run(&self) {
        loop {
            // Wake up sleeping tasks
            let now = Instant::now();
            let mut ready = Vec::new();

            let mut sleeping = self.sleeping_tasks.lock().unwrap();
            let wake_times: Vec<_> = sleeping.keys().cloned().collect();

            for time in wake_times {
                if time <= now {
                    if let Some(tasks) = sleeping.remove(&time) {
                        ready.extend(tasks);
                    }
                }
            }
            drop(sleeping);

            // Add woken tasks to ready queue
            self.ready_tasks.lock().unwrap().extend(ready);

            // Process ready tasks
            while let Some(task) = self.ready_tasks.lock().unwrap().pop_front() {
                let waker = futures::task::waker_ref(&task);
                let mut cx = Context::from_waker(&waker);

                let mut future = unsafe {
                    // Safety: we only move the future temporarily and
                    // ensure it's moved back before the task is dropped
                    // let ptr = &task.future as *const _ as *mut _;
                    // &mut *ptr
                    todo!()
                };

                match future.as_mut().poll(&mut cx) {
                    Poll::Pending => {
                        // Task will be woken up by its waker when ready
                    }
                    Poll::Ready(()) => {
                        // Task is complete, it will be dropped
                    }
                }
            }

            if self.ready_tasks.lock().unwrap().is_empty()
                && self.sleeping_tasks.lock().unwrap().is_empty()
            {
                break;
            }
        }
    }
}

// Helper function to create a sleep future
pub struct Sleep {
    deadline: Instant,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.deadline {
            Poll::Ready(())
        } else {
            // Register waker to be called later
            let task = Arc::new(Task {
                future: Box::pin(std::future::ready(())),
                wake_time: Some(self.deadline),
            });
            EXECUTOR.with(|e| {
                e.borrow()
                    .sleeping_tasks
                    .lock()
                    .unwrap()
                    .entry(self.deadline)
                    .or_default()
                    .push(task);
            });
            Poll::Pending
        }
    }
}

pub fn sleep(duration: Duration) -> Sleep {
    Sleep {
        deadline: Instant::now() + duration,
    }
}

// Example async function
pub async fn hello_async() {
    println!("Hello");
    sleep(Duration::from_secs(1)).await;
    println!("World!");
}
