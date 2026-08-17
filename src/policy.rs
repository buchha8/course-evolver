use rand::Rng;
use rand::SeedableRng;

use crate::course::{Course, Position};

const INPUT_COUNT: usize = 9;
const HIDDEN_COUNT: usize = 4;
const ACTION_COUNT: usize = 4;

const INPUT_HIDDEN_WEIGHT_COUNT: usize = INPUT_COUNT * HIDDEN_COUNT;
const HIDDEN_HIDDEN_WEIGHT_COUNT: usize = HIDDEN_COUNT * HIDDEN_COUNT;
const HIDDEN_ACTION_WEIGHT_COUNT: usize = HIDDEN_COUNT * ACTION_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub position_x: f32,
    pub position_y: f32,

    pub goal_x: f32,
    pub goal_y: f32,

    pub previous_action: Option<Action>,
    pub previous_action_succeeded: bool,
}

impl Observation {
    fn inputs(&self) -> [f32; INPUT_COUNT] {
        let previous_up =
            (self.previous_action == Some(Action::Up)) as u8 as f32;

        let previous_down =
            (self.previous_action == Some(Action::Down)) as u8 as f32;

        let previous_left =
            (self.previous_action == Some(Action::Left)) as u8 as f32;

        let previous_right =
            (self.previous_action == Some(Action::Right)) as u8 as f32;

        [
            self.position_x,
            self.position_y,
            self.goal_x,
            self.goal_y,
            previous_up,
            previous_down,
            previous_left,
            previous_right,
            self.previous_action_succeeded as u8 as f32,
        ]
    }
}

pub trait Policy: Send {
    fn choose_action(&mut self, observation: &Observation) -> Action;
}

impl<P: Policy + ?Sized> Policy for Box<P> {
    fn choose_action(&mut self, observation: &Observation) -> Action {
        (**self).choose_action(observation)
    }
}

#[derive(Clone, Debug)]
pub struct Genome {
    pub input_hidden_weights: [f32; INPUT_HIDDEN_WEIGHT_COUNT],
    pub hidden_hidden_weights: [f32; HIDDEN_HIDDEN_WEIGHT_COUNT],
    pub hidden_biases: [f32; HIDDEN_COUNT],

    pub hidden_action_weights: [f32; HIDDEN_ACTION_WEIGHT_COUNT],
    pub action_biases: [f32; ACTION_COUNT],
}

impl Genome {
    pub fn random() -> Self {
        let mut rng = rand::rngs::StdRng::from_os_rng();

        Self {
            input_hidden_weights: std::array::from_fn(|_| {
                rng.random_range(-1.0..=1.0)
            }),

            hidden_hidden_weights: std::array::from_fn(|_| {
                rng.random_range(-1.0..=1.0)
            }),

            hidden_biases: std::array::from_fn(|_| {
                rng.random_range(-1.0..=1.0)
            }),

            hidden_action_weights: std::array::from_fn(|_| {
                rng.random_range(-1.0..=1.0)
            }),

            action_biases: std::array::from_fn(|_| {
                rng.random_range(-1.0..=1.0)
            }),
        }
    }
}

pub struct RecurrentPolicy {
    genome: Genome,
    hidden: [f32; HIDDEN_COUNT],
}

impl RecurrentPolicy {
    pub fn new(genome: Genome) -> Self {
        Self {
            genome,
            hidden: [0.0; HIDDEN_COUNT],
        }
    }

    fn update_hidden(&mut self, inputs: &[f32; INPUT_COUNT]) {
        let previous_hidden = self.hidden;
        let mut next_hidden = [0.0; HIDDEN_COUNT];

        for hidden_index in 0..HIDDEN_COUNT {
            let mut value = self.genome.hidden_biases[hidden_index];

            for input_index in 0..INPUT_COUNT {
                let weight_index =
                    hidden_index * INPUT_COUNT + input_index;

                value += inputs[input_index]
                    * self.genome.input_hidden_weights[weight_index];
            }

            for previous_hidden_index in 0..HIDDEN_COUNT {
                let weight_index =
                    hidden_index * HIDDEN_COUNT
                        + previous_hidden_index;

                value += previous_hidden[previous_hidden_index]
                    * self.genome.hidden_hidden_weights[weight_index];
            }

            next_hidden[hidden_index] = value.tanh();
        }

        self.hidden = next_hidden;
    }

    fn action_scores(&self) -> [f32; ACTION_COUNT] {
        let mut scores = self.genome.action_biases;

        for action_index in 0..ACTION_COUNT {
            for hidden_index in 0..HIDDEN_COUNT {
                let weight_index =
                    action_index * HIDDEN_COUNT + hidden_index;

                scores[action_index] += self.hidden[hidden_index]
                    * self.genome.hidden_action_weights[weight_index];
            }
        }

        scores
    }
}

impl Policy for RecurrentPolicy {
    fn choose_action(&mut self, observation: &Observation) -> Action {
        let inputs = observation.inputs();

        self.update_hidden(&inputs);

        let scores = self.action_scores();

        let mut best_action = 0;
        let mut best_score = scores[0];

        for action_index in 1..ACTION_COUNT {
            if scores[action_index] > best_score {
                best_score = scores[action_index];
                best_action = action_index;
            }
        }

        match best_action {
            0 => Action::Up,
            1 => Action::Down,
            2 => Action::Left,
            _ => Action::Right,
        }
    }
}

pub fn observe(
    course: &Course,
    position: Position,
    previous_action: Option<Action>,
    previous_action_succeeded: bool,
) -> Observation {
    let width_scale = (course.width - 1) as f32;
    let height_scale = (course.height - 1) as f32;

    Observation {
        position_x: position.x as f32 / width_scale,
        position_y: position.y as f32 / height_scale,

        goal_x:
            (course.goal.x as f32 - position.x as f32) / width_scale,

        goal_y:
            (course.goal.y as f32 - position.y as f32) / height_scale,

        previous_action,
        previous_action_succeeded,
    }
}