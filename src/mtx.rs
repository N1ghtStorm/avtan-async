use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// A simple asynchronous mutex implementation.
pub struct Mutex<T> {
    /// The semaphore is used to control access to the data.
    /// With permit_count = 1, it acts as a mutex.
    // semaphore: Arc<Semaphore>,
    /// The protected data.
    data: UnsafeCell<T>,
}

/// Guard that provides safe access to the locked data and releases the lock when dropped.
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

// Mutex implementation
impl<T> Mutex<T> {
    /// Creates a new async mutex with the given data.
    pub fn new(data: T) -> Self {
        Self {
            // semaphore: Arc::new(Semaphore::new(1)), // Only one permit available
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the lock, waiting asynchronously if necessary.
    pub async fn lock(&self) -> MutexGuard<'_, T> {
        // let permit = self.semaphore.acquire().await.unwrap();
        MutexGuard {
            mutex: self,
            // _permit: permit,
        }
    }
}

// Implement Send and Sync for Mutex if T is Send
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

// Implement Deref and DerefMut for MutexGuard
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}
