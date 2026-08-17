use std::sync::Arc;

use crate::course::Course;
use crate::fitness::{self, FitnessBreakdown};
use crate::policy::Policy;
use crate::simulation::{Simulation, SimulationResult};

pub struct Job {
    pub id: u64,
    pub generation: usize,
    pub individual_id: u64,
    pub course_index: usize,
    pub course: Arc<Course>,
    pub policy: Box<dyn Policy>,
    pub max_steps: usize,
}

pub struct JobResult {
    pub job_id: u64,
    pub generation: usize,
    pub individual_id: u64,
    pub course_index: usize,
    pub course: Arc<Course>,
    pub simulation: SimulationResult,

    pub fitness: f64,
    pub fitness_breakdown: FitnessBreakdown,
}

impl Job {
    pub fn new(
        id: u64,
        generation: usize,
        individual_id: u64,
        course_index: usize,
        course: Arc<Course>,
        policy: Box<dyn Policy>,
        max_steps: usize,
    ) -> Self {
        Self {
            id,
            generation,
            individual_id,
            course_index,
            course,
            policy,
            max_steps,
        }
    }

    pub fn execute(self) -> JobResult {
        let job_id = self.id;
        let generation = self.generation;
        let individual_id = self.individual_id;
        let course_index = self.course_index;

        let mut simulation =
            Simulation::new(
                self.course,
                self.policy,
                self.max_steps,
            );

        simulation.run_to_completion();

        let simulation_result =
            simulation.result();

        let fitness_breakdown =
            fitness::calculate(
                &simulation_result,
                simulation.course.as_ref(),
            );

        let fitness =
            fitness_breakdown.total;

        JobResult {
            job_id,
            generation,
            individual_id,
            course_index,
            course: simulation.course,
            simulation: simulation_result,

            fitness,
            fitness_breakdown,
        }
    }
}