use rayon::prelude::*;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

struct ParallelProcessor<T, U> {
    sender: SyncSender<T>,
    receiver: Receiver<U>,
    worker_count: usize,
}

impl<T, U> ParallelProcessor<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    pub fn new<F>(worker_count: usize, processing_fn: F) -> Self
    where
        F: Fn(T) -> U + Send + Sync + 'static,
    {
        let (input_tx, input_rx) = sync_channel(worker_count * 2);
        let (output_tx, output_rx) = sync_channel(worker_count * 2);
        
        let processing_fn = Arc::new(processing_fn);
        
        for _ in 0..worker_count {
            let input_rx = input_rx.clone();
            let output_tx = output_tx.clone();
            let processing_fn = processing_fn.clone();
            
            thread::spawn(move || {
                while let Ok(item) = input_rx.recv() {
                    let result = processing_fn(item);
                    if output_tx.send(result).is_err() {
                        break;
                    }
                }
            });
        }

        ParallelProcessor {
            sender: input_tx,
            receiver: output_rx,
            worker_count,
        }
    }

    pub fn process_stream<I>(&self, input: I) -> impl Iterator<Item = U>
    where
        I: IntoIterator<Item = T>,
    {
        let sender = self.sender.clone();
        thread::spawn(move || {
            for item in input {
                if sender.send(item).is_err() {
                    break;
                }
            }
        });

        (0..self.worker_count).map(move |_| {
            self.receiver.recv().expect("Worker thread died")
        })
    }
}
