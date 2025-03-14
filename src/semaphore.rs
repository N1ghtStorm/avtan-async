use std::sync::{Condvar, Mutex};

/// A counting semaphore implementation that can be used for controlling access
/// to a shared resource by multiple threads.
pub struct Semaphore {
    /// The current number of available permits
    permits: Mutex<usize>,
    /// Condition variable to signal waiting threads
    condvar: Condvar,
}

impl Semaphore {
    /// Creates a new semaphore with the specified number of permits.
    pub fn new(permits: usize) -> Self {
        Semaphore {
            permits: Mutex::new(permits),
            condvar: Condvar::new(),
        }
    }

    /// Acquires a permit, blocking until one is available.
    pub fn acquire(&self) {
        let mut permits = self.permits.lock().unwrap();
        while *permits == 0 {
            permits = self.condvar.wait(permits).unwrap();
        }
        *permits -= 1;
    }

    /// Tries to acquire a permit without blocking.
    /// Returns true if a permit was acquired, false otherwise.
    pub fn try_acquire(&self) -> bool {
        let mut permits = self.permits.lock().unwrap();
        if *permits > 0 {
            *permits -= 1;
            true
        } else {
            false
        }
    }

    /// Releases a permit, potentially unblocking a waiting thread.
    pub fn release(&self) {
        let mut permits = self.permits.lock().unwrap();
        *permits += 1;
        self.condvar.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_semaphore() {
        let semaphore = Arc::new(Semaphore::new(2));
        let mut handles = vec![];

        for i in 0..5 {
            let sem = Arc::clone(&semaphore);
            let handle = thread::spawn(move || {
                sem.acquire();
                println!("Thread {} acquired a permit", i);
                // Simulate some work
                thread::sleep(std::time::Duration::from_millis(100));
                println!("Thread {} releasing a permit", i);
                sem.release();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
