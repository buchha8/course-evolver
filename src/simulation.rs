use std::sync::Arc;

use crate::course::{Course, Position};
use crate::policy::{observe, Action, Policy};

pub struct Agent {
    pub position: Position,
    pub path: Vec<Position>,
    pub collisions: usize,

    pub previous_action: Option<Action>,
    pub previous_action_succeeded: bool,
}

impl Agent {
    pub fn new(start: Position) -> Self {
        Self {
            position: start,
            path: vec![start],
            collisions: 0,

            previous_action: None,
            previous_action_succeeded: false,
        }
    }
}

pub struct Simulation<P: Policy> {
    pub course: Arc<Course>,
    pub policy: P,
    pub agent: Agent,
    pub steps: usize,
    pub max_steps: usize,
    pub finished: bool,
    pub reached_goal: bool,
}

impl<P: Policy> Simulation<P> {
    pub fn new(
        course: Arc<Course>,
        policy: P,
        max_steps: usize,
    ) -> Self {
        let start = course.start;

        Self {
            course,
            policy,
            agent: Agent::new(start),
            steps: 0,
            max_steps,
            finished: false,
            reached_goal: false,
        }
    }

    pub fn step(&mut self) {
        if self.finished {
            return;
        }

        if self.agent.position == self.course.goal {
            self.finished = true;
            self.reached_goal = true;
            return;
        }

        if self.steps >= self.max_steps {
            self.finished = true;
            return;
        }

        let observation = observe(
            &self.course,
            self.agent.position,
            self.agent.previous_action,
            self.agent.previous_action_succeeded,
        );

        let action = self.policy.choose_action(&observation);

        let target = match action {
            Action::Up => {
                if self.agent.position.y == 0 {
                    None
                } else {
                    Some(Position {
                        x: self.agent.position.x,
                        y: self.agent.position.y - 1,
                    })
                }
            }

            Action::Down => {
                let y = self.agent.position.y + 1;

                if y >= self.course.height {
                    None
                } else {
                    Some(Position {
                        x: self.agent.position.x,
                        y,
                    })
                }
            }

            Action::Left => {
                if self.agent.position.x == 0 {
                    None
                } else {
                    Some(Position {
                        x: self.agent.position.x - 1,
                        y: self.agent.position.y,
                    })
                }
            }

            Action::Right => {
                let x = self.agent.position.x + 1;

                if x >= self.course.width {
                    None
                } else {
                    Some(Position {
                        x,
                        y: self.agent.position.y,
                    })
                }
            }
        };

        let movement_succeeded = target
            .map(|position| self.course.is_walkable(position))
            .unwrap_or(false);

        if movement_succeeded {
            let target = target.expect(
                "Successful movement must have a valid target",
            );

            self.agent.position = target;
            self.agent.path.push(target);
        } else {
            self.agent.collisions += 1;
        }

        self.agent.previous_action = Some(action);
        self.agent.previous_action_succeeded = movement_succeeded;

        self.steps += 1;

        if self.agent.position == self.course.goal {
            self.finished = true;
            self.reached_goal = true;
        } else if self.steps >= self.max_steps {
            self.finished = true;
        }
    }

    pub fn run_to_completion(&mut self) {
        while !self.finished {
            self.step();
        }
    }

    pub fn result(&self) -> SimulationResult {
        SimulationResult {
            reached_goal: self.reached_goal,
            steps: self.steps,
            collisions: self.agent.collisions,
            path: self.agent.path.clone(),
        }
    }
}

pub struct SimulationResult {
    pub reached_goal: bool,
    pub steps: usize,
    pub collisions: usize,
    pub path: Vec<Position>,
}