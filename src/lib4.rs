use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Wake};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;


use std::sync::mpsc::SyncSender; // added
use std::sync::mpsc::Receiver; // added
use std::sync::mpsc::sync_channel; // added

struct Task {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    task_sender: SyncSender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.task_sender.send(self.clone()).expect("Too many tasks");
    }
}

struct Executor {
    task_queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
    task_sender: SyncSender<Arc<Task>>,
    task_receiver: Receiver<Arc<Task>>,
}

impl Executor {
    pub fn new() -> Self {
        let (task_sender, task_receiver) = sync_channel(10000);
        Executor {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            task_sender,
            task_receiver,
        }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Box::pin(future),
            task_sender: self.task_sender.clone(),
        });

        self.task_queue.lock().unwrap().push_back(task);
    }

    pub fn run(&self) {
        loop {
            while let Ok(task) = self.task_receiver.try_recv() {
                self.task_queue.lock().unwrap().push_back(task);
            }

            let task = match self.task_queue.lock().unwrap().pop_front() {
                Some(task) => task,
                None => break,
            };

            let waker = task_to_waker(task.clone());
            let mut context = Context::from_waker(&waker);

            match unsafe { Pin::new_unchecked(&mut task.future) }.poll(&mut context) {
                Poll::Ready(()) => {}
                Poll::Pending => {
                    self.task_queue.lock().unwrap().push_back(task);
                }
            }
        }
    }
}
