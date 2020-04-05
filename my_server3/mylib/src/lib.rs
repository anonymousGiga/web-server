use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            //let job = receiver.lock().unwrap().recv().unwrap();
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("worker {} receive a job", id);
                    job();
                },
                Message::Terminate => {
                    println!("Worker {} receive terminate", id);
                    break;
                },
            }
        });

        Worker { 
            id, 
            thread: Some(thread), 
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    //sender: mpsc::Sender<Job>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;
enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver)); //线程安全

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        //self.sender.send(job).unwrap();
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        //send terminate message to all workers
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        //wait all workers terminate
        for worker in &mut self.workers {
            //wait for worker thread terminate

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
