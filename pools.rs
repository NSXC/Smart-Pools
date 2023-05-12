use std::sync::{Arc, Mutex};
use std::thread;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(num_threads: usize) -> ThreadPool {
        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(num_threads);

        for id in 0..num_threads {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Job::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Job::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<std::sync::mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            match job {
                Job::Task(f) => f(),
                Job::Terminate => break,
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

enum Job {
    Task(Box<dyn FnOnce() + Send + 'static>),
    Terminate,
}

impl Job {
    fn new<F>(f: F) -> Job
    where
        F: FnOnce() + Send + 'static,
    {
        Job::Task(Box::new(f))
    }
}

fn main() {
    let pool = ThreadPool::new(4);

    for i in 0..1000 {//1000 loops
        pool.execute(move || {
            println!("Task {} started", i);//print when task has started
            let mut num_threads = 1;
            if i % 2 == 0 {
                num_threads = 2;
            }
            if num_threads > 1 {
                let subpool = ThreadPool::new(num_threads);
                for j in 0..num_threads {
                    subpool.execute(move || {
                        println!("Sub-task {} of Task {} started", j, i);//print sub task
                    });
                }
            } else {
                println!("Still running");//fall back message
            }

            println!("Task {} finished", i);//when task is done
        });
    }
}
