//! Thread pool implementation
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    EndYourselfMortal,
    DoThis(Job),
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl ThreadPool {
    /// Makes a new ThreadPool with `nthreads` threads
    pub fn new(nthreads: usize) -> ThreadPool {
        assert!(nthreads > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::new();
        for tid in 0..nthreads {
            workers.push(Worker::new(tid, receiver.clone()));
        }
        ThreadPool { workers, sender }
    }

    pub fn push<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::DoThis(job)).unwrap();
    }

    /// Call when done putting work into the queue
    pub fn done(&self) {
        for _ in &self.workers {
            self.sender.send(Message::EndYourselfMortal).unwrap();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in self.workers.iter_mut() {
            println!("stopping {}", worker.id);
            worker.thread.take().map(|thread| thread.join().unwrap());
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        Worker {
            id,
            thread: Some(thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::EndYourselfMortal => break,
                    Message::DoThis(job) => job(),
                }
            })),
        }
    }
}
