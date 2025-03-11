use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
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

pub struct AvtanSharedFuture {
    state: Arc<Mutex<SharedFutureState>>,
}
pub struct SharedFutureState {
    data: Option<Vec<u8>>,
    waker: Option<Waker>,
}

impl AvtanSharedFuture {
    pub fn new() -> (Self, Arc<Mutex<SharedFutureState>>) {
        let state = Arc::new(Mutex::new(SharedFutureState {
            data: None,
            waker: None,
        }));
        (
            AvtanSharedFuture {
                state: state.clone(),
            },
            state,
        )
    }
}

impl Future for AvtanSharedFuture {
    type Output = String;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Polling the future");
        let mut state = self.state.lock().unwrap();
        if state.data.is_some() {
            let data = state.data.take().unwrap();
            Poll::Ready(String::from_utf8(data).unwrap())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}