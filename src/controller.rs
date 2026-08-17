use std::collections::VecDeque;

use crate::evolution::Evolution;
use crate::history::GenerationRecord;
use crate::job::{Job, JobResult};
use crate::scheduler::Scheduler;

pub struct RunController {
    scheduler: Scheduler,
    evolution: Evolution,

    pending_jobs: VecDeque<Job>,
    current_results: Vec<JobResult>,
    history: Vec<GenerationRecord>,

    expected_jobs: usize,
    completed_jobs: usize,

    generation_limit: usize,
    finished: bool,
}

impl RunController {
    pub fn new(
        worker_count: usize,
        population_size: usize,
        max_steps: usize,
        generation_limit: usize,
    ) -> Self {
        assert!(generation_limit > 0);

        let mut controller = Self {
            scheduler:
                Scheduler::new(worker_count),

            evolution:
                Evolution::new(
                    population_size,
                    max_steps,
                ),

            pending_jobs:
                VecDeque::new(),

            current_results:
                Vec::new(),

            history:
                Vec::new(),

            expected_jobs: 0,
            completed_jobs: 0,

            generation_limit,
            finished: false,
        };

        controller.queue_generation();

        controller
    }

    pub fn update(&mut self) {
        if self.finished {
            return;
        }

        self.submit_pending_jobs();
        self.collect_results();
    }

    pub fn history(
        &self,
    ) -> &[GenerationRecord] {
        &self.history
    }

    fn queue_generation(&mut self) {
        let jobs =
            self.evolution.create_jobs();

        self.expected_jobs =
            jobs.len();

        self.completed_jobs = 0;

        self.current_results.clear();

        self.pending_jobs.extend(jobs);

        println!(
            "Starting generation {} with {} jobs",
            self.evolution.generation(),
            self.expected_jobs,
        );
    }

    fn submit_pending_jobs(&mut self) {
        while let Some(job) =
            self.pending_jobs.pop_front()
        {
            match self.scheduler.submit(job) {
                Ok(()) => {}

                Err(job) => {
                    self.pending_jobs
                        .push_front(job);

                    break;
                }
            }
        }
    }

    fn collect_results(&mut self) {
        while let Some(result) =
            self.scheduler.try_receive()
        {
            println!(
                "Generation {} individual {} course {} (job {}) completed: fitness = {:.2}",
                result.generation,
                result.individual_id,
                result.course_index,
                result.job_id,
                result.fitness,
            );

            self.current_results
                .push(result);

            self.completed_jobs += 1;

            if self.completed_jobs
                == self.expected_jobs
            {
                self.finish_generation();
                break;
            }
        }
    }

    fn finish_generation(&mut self) {
        let generation = self
            .current_results
            .first()
            .map(
                |result| result.generation,
            )
            .expect(
                "Completed generation has no results",
            );

        let results =
            std::mem::take(
                &mut self.current_results,
            );

        let record =
            GenerationRecord::new(
                results,
            );

        let best_fitness = record
            .individuals
            .iter()
            .map(
                |individual| {
                    individual
                        .aggregate_fitness
                },
            )
            .max_by(
                |a, b| a.total_cmp(b),
            )
            .expect(
                "Completed generation has no individuals",
            );

        println!(
            "Generation {generation} completed. Best aggregate fitness: {best_fitness:.2}"
        );

        let evaluations:
            Vec<(u64, f64)> =
            record
                .individuals
                .iter()
                .map(
                    |individual| {
                        (
                            individual
                                .individual_id,
                            individual
                                .aggregate_fitness,
                        )
                    },
                )
                .collect();

        let completed_generation_count =
            self.history.len() + 1;

        if completed_generation_count
            < self.generation_limit
        {
            self.evolution
                .advance_generation(
                    &evaluations,
                );
        }

        self.history.push(record);

        if completed_generation_count
            >= self.generation_limit
        {
            self.finished = true;

            println!(
                "Evolution run completed after {} generations.",
                completed_generation_count
            );

            return;
        }

        self.queue_generation();
    }
}