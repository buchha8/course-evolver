use std::sync::Arc;

use rand::Rng;

use crate::course::Course;
use crate::job::Job;
use crate::policy::{Genome, RecurrentPolicy};

const EVALUATION_COURSE_COUNT: usize = 3;

const ELITE_FRACTION: f32 = 0.05;
const PARENT_POOL_FRACTION: f32 = 0.30;
const RANDOM_IMMIGRANT_FRACTION: f32 = 0.10;

const MUTATION_RATE: f64 = 0.10;
const MUTATION_AMOUNT: f32 = 0.35;

const MIN_WEIGHT: f32 = -3.0;
const MAX_WEIGHT: f32 = 3.0;

#[derive(Clone, Debug)]
pub struct Individual {
    pub id: u64,
    pub genome: Genome,
}

pub struct Evolution {
    population: Vec<Individual>,
    generation: usize,
    next_individual_id: u64,
    next_job_id: u64,
    max_steps: usize,
}

impl Evolution {
    pub fn new(
        population_size: usize,
        max_steps: usize,
    ) -> Self {
        let mut next_individual_id = 0;

        let population = (0..population_size)
            .map(|_| {
                let individual = Individual {
                    id: next_individual_id,
                    genome: Genome::random(),
                };

                next_individual_id += 1;

                individual
            })
            .collect();

        Self {
            population,
            generation: 0,
            next_individual_id,
            next_job_id: 0,
            max_steps,
        }
    }

    pub fn create_jobs(&mut self) -> Vec<Job> {
        let courses: Vec<Arc<Course>> =
            (0..EVALUATION_COURSE_COUNT)
                .map(|_| {
                    Arc::new(
                        Course::random(50, 50),
                    )
                })
                .collect();

        let mut jobs = Vec::with_capacity(
            self.population.len()
                * EVALUATION_COURSE_COUNT,
        );

        for individual in &self.population {
            for (course_index, course) in
                courses.iter().enumerate()
            {
                let policy =
                    RecurrentPolicy::new(
                        individual.genome.clone(),
                    );

                let job = Job::new(
                    self.next_job_id,
                    self.generation,
                    individual.id,
                    course_index,
                    Arc::clone(course),
                    Box::new(policy),
                    self.max_steps,
                );

                self.next_job_id += 1;

                jobs.push(job);
            }
        }

        jobs
    }

    pub fn advance_generation(
        &mut self,
        evaluations: &[(u64, f64)],
    ) {
        assert_eq!(
            evaluations.len(),
            self.population.len(),
            "Every individual must have an aggregate fitness"
        );

        let mut ranked_evaluations =
            evaluations.to_vec();

        ranked_evaluations.sort_by(
            |a, b| b.1.total_cmp(&a.1),
        );

        let ranked_population:
            Vec<Individual> =
            ranked_evaluations
                .iter()
                .map(
                    |(individual_id, _)| {
                        self.population
                            .iter()
                            .find(
                                |individual| {
                                    individual.id
                                        == *individual_id
                                },
                            )
                            .expect(
                                "Evaluation references unknown individual",
                            )
                            .clone()
                    },
                )
                .collect();

        let population_size =
            self.population.len();

        let elite_count = (
            population_size as f32
                * ELITE_FRACTION
        )
            .ceil() as usize;

        let elite_count =
            elite_count.clamp(
                1,
                population_size,
            );

        let parent_pool_size = (
            population_size as f32
                * PARENT_POOL_FRACTION
        )
            .ceil() as usize;

        let parent_pool_size =
            parent_pool_size.clamp(
                1,
                population_size,
            );

        let immigrant_count = (
            population_size as f32
                * RANDOM_IMMIGRANT_FRACTION
        )
            .ceil() as usize;

        let immigrant_count =
            immigrant_count.min(
                population_size
                    - elite_count,
            );

        let child_target =
            population_size
                - immigrant_count;

        let mut next_population =
            Vec::with_capacity(
                population_size,
            );

        // Preserve the strongest individuals exactly.
        for elite in ranked_population
            .iter()
            .take(elite_count)
        {
            next_population.push(
                elite.clone(),
            );
        }

        let mut rng = rand::rng();

        // Fill most of the population with mutated
        // descendants of strong parents.
        while next_population.len()
            < child_target
        {
            let parent =
                &ranked_population[
                    rng.random_range(
                        0..parent_pool_size,
                    )
                ];

            let mut child_genome =
                parent.genome.clone();

            mutate(
                &mut child_genome,
                &mut rng,
            );

            let child = Individual {
                id: self.next_individual_id,
                genome: child_genome,
            };

            self.next_individual_id += 1;

            next_population.push(child);
        }

        // Inject completely new genomes to maintain diversity.
        while next_population.len()
            < population_size
        {
            let immigrant =
                Individual {
                    id: self.next_individual_id,
                    genome: Genome::random(),
                };

            self.next_individual_id += 1;

            next_population.push(
                immigrant,
            );
        }

        self.population =
            next_population;

        self.generation += 1;
    }

    pub fn generation(&self) -> usize {
        self.generation
    }
}

fn mutate(
    genome: &mut Genome,
    rng: &mut impl Rng,
) {
    for weight in genome
        .input_hidden_weights
        .iter_mut()
        .chain(
            genome
                .hidden_hidden_weights
                .iter_mut(),
        )
        .chain(
            genome
                .hidden_biases
                .iter_mut(),
        )
        .chain(
            genome
                .hidden_action_weights
                .iter_mut(),
        )
        .chain(
            genome
                .action_biases
                .iter_mut(),
        )
    {
        if rng.random_bool(
            MUTATION_RATE,
        ) {
            *weight +=
                rng.random_range(
                    -MUTATION_AMOUNT
                        ..=MUTATION_AMOUNT,
                );

            *weight =
                weight.clamp(
                    MIN_WEIGHT,
                    MAX_WEIGHT,
                );
        }
    }
}