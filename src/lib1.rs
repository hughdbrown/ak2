use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
use std::time::Duration;

struct WorkQueue<T> {
    queue: Arc<(Mutex<VecDeque<T>>, Condvar)>,
    max_size: usize,
}

impl<T> WorkQueue<T> {
    pub fn new(max_size: usize) -> Self {
        WorkQueue {
            queue: Arc::new((
                Mutex::new(VecDeque::new()),
                Condvar::new()
            )),
            max_size,
        }
    }

    pub fn push(&self, item: T, timeout: Duration) -> Result<(), T> {
        let (lock, cvar) = &*self.queue;
        
        let mut queue = lock.lock().unwrap();
        let deadline = std::time::Instant::now() + timeout;
        
        while queue.len() >= self.max_size {
            let wait_time = deadline.checked_duration_since(std::time::Instant::now())
                .ok_or_else(|| item)?;
                
            let (new_queue, timeout_result) = cvar
                .wait_timeout(queue, wait_time)
                .unwrap();
            queue = new_queue;
            
            if timeout_result.timed_out() {
                return Err(item);
            }
        }
        
        queue.push_back(item);
        cvar.notify_one();
        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let (lock, cvar) = &*self.queue;
        let mut queue = lock.lock().unwrap();
        
        let item = queue.pop_front();
        if item.is_some() {
            cvar.notify_one();
        }
        
        item
    }
}
