use std::thread::{self, JoinHandle};
use std::time::Instant;

use tokio::runtime::Builder;
use tokio::sync::{mpsc, Semaphore};

use crate::job::{Job, JobResult};

pub struct Scheduler {
    job_sender: Option<mpsc::Sender<Job>>,
    result_receiver: mpsc::Receiver<JobResult>,
    runtime_thread: Option<JoinHandle<()>>,
}

impl Scheduler {
    pub fn new(concurrency: usize) -> Self {
        assert!(concurrency > 0);

        let queue_capacity = concurrency * 4;

        let (job_sender, job_receiver) =
            mpsc::channel::<Job>(queue_capacity);

        let (result_sender, result_receiver) =
            mpsc::channel::<JobResult>(queue_capacity);

        let runtime_thread = thread::spawn(move || {
            let runtime = Builder::new_multi_thread()
                .worker_threads(concurrency)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");

            runtime.block_on(run_scheduler(
                job_receiver,
                result_sender,
                concurrency,
            ));
        });

        Self {
            job_sender: Some(job_sender),
            result_receiver,
            runtime_thread: Some(runtime_thread),
        }
    }

    pub fn submit(&self, job: Job) -> Result<(), Job> {
        let sender = self
            .job_sender
            .as_ref()
            .expect("Scheduler has been shut down");

        match sender.try_send(job) {
            Ok(()) => Ok(()),

            Err(mpsc::error::TrySendError::Full(job)) => Err(job),

            Err(mpsc::error::TrySendError::Closed(_)) => {
                panic!("Scheduler runtime has stopped");
            }
        }
    }

    pub fn try_receive(&mut self) -> Option<JobResult> {
        self.result_receiver.try_recv().ok()
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.job_sender.take();

        if let Some(runtime_thread) = self.runtime_thread.take() {
            let _ = runtime_thread.join();
        }
    }
}

async fn run_scheduler(
    mut job_receiver: mpsc::Receiver<Job>,
    result_sender: mpsc::Sender<JobResult>,
    concurrency: usize,
) {
    let semaphore =
        std::sync::Arc::new(Semaphore::new(concurrency));

    let mut tasks = tokio::task::JoinSet::new();

    while let Some(job) = job_receiver.recv().await {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Scheduler semaphore closed");

        let result_sender = result_sender.clone();

        tasks.spawn(async move {
            let job_id = job.id;

            let execution =
                tokio::task::spawn_blocking(move || {
                    let thread_id =
                        std::thread::current().id();

                    let start = Instant::now();

                    println!(
                        "Job {job_id} START on thread {thread_id:?}"
                    );

                    let result = job.execute();

                    println!(
                        "Job {job_id} END on thread {thread_id:?} after {:?}",
                        start.elapsed()
                    );

                    result
                })
                .await;

            drop(permit);

            match execution {
                Ok(result) => {
                    let _ = result_sender.send(result).await;
                }

                Err(error) => {
                    eprintln!(
                        "Job {job_id} failed to execute: {error}"
                    );
                }
            }
        });
    }

    while let Some(result) = tasks.join_next().await {
        if let Err(error) = result {
            eprintln!("Scheduler task failed: {error}");
        }
    }
}