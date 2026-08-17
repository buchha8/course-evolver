use std::collections::BTreeMap;

use crate::job::JobResult;

pub struct IndividualRecord {
    pub individual_id: u64,
    pub aggregate_fitness: f64,
    pub results: Vec<JobResult>,
}

impl IndividualRecord {
    pub fn best_course_index(&self) -> Option<usize> {
        self.results
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.fitness.total_cmp(&b.fitness)
            })
            .map(|(index, _)| index)
    }

    pub fn solved_course_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| {
                result.simulation.reached_goal
            })
            .count()
    }
}

pub struct GenerationRecord {
    pub individuals: Vec<IndividualRecord>,
}

impl GenerationRecord {
    pub fn new(
        results: Vec<JobResult>,
    ) -> Self {
        let mut grouped:
            BTreeMap<u64, Vec<JobResult>> =
            BTreeMap::new();

        for result in results {
            grouped
                .entry(result.individual_id)
                .or_default()
                .push(result);
        }

        let mut individuals =
            Vec::with_capacity(grouped.len());

        for (
            individual_id,
            mut results,
        ) in grouped
        {
            results.sort_by_key(
                |result| result.course_index,
            );

            let aggregate_fitness =
                results
                    .iter()
                    .map(|result| result.fitness)
                    .sum::<f64>()
                    / results.len() as f64;

            individuals.push(
                IndividualRecord {
                    individual_id,
                    aggregate_fitness,
                    results,
                },
            );
        }

        Self {
            individuals,
        }
    }

    pub fn best_individual_index(
        &self,
    ) -> Option<usize> {
        self.individuals
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.aggregate_fitness
                    .total_cmp(&b.aggregate_fitness)
            })
            .map(|(index, _)| index)
    }

    pub fn best_aggregate_fitness(
        &self,
    ) -> Option<f64> {
        self.individuals
            .iter()
            .map(|individual| {
                individual.aggregate_fitness
            })
            .max_by(|a, b| a.total_cmp(b))
    }

    pub fn mean_aggregate_fitness(
        &self,
    ) -> Option<f64> {
        if self.individuals.is_empty() {
            return None;
        }

        Some(
            self.individuals
                .iter()
                .map(|individual| {
                    individual.aggregate_fitness
                })
                .sum::<f64>()
                / self.individuals.len() as f64,
        )
    }

    pub fn total_courses_solved(
        &self,
    ) -> usize {
        self.individuals
            .iter()
            .map(|individual| {
                individual.solved_course_count()
            })
            .sum()
    }
}