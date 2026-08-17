use crate::course::{Course, Position};
use crate::simulation::SimulationResult;

const PROGRESS_REWARD_PER_CELL: f64 = 30.0;
const UNIQUE_CELL_REWARD: f64 = 10.0;
const SUCCESSFUL_MOVE_REWARD: f64 = 0.0;

const REVISIT_PENALTY: f64 = 1.0;
const COLLISION_PENALTY: f64 = 1.0;

const STEPS_TO_CLOSEST_PENALTY: f64 = 2.0;

const GOAL_REWARD: f64 = 10_000.0;
const SOLUTION_STEP_PENALTY: f64 = 0.25;

#[derive(Clone, Debug)]
pub struct FitnessBreakdown {
    pub progress_cells: usize,
    pub progress_fitness: f64,

    pub steps_to_closest: usize,
    pub steps_to_closest_fitness: f64,

    pub unique_cells: usize,
    pub unique_cell_fitness: f64,

    pub successful_moves: usize,
    pub successful_move_fitness: f64,

    pub revisits: usize,
    pub revisit_fitness: f64,

    pub collisions: usize,
    pub collision_fitness: f64,

    pub goal_fitness: f64,
    pub solution_step_fitness: f64,

    pub total: f64,
}

pub fn calculate(
    result: &SimulationResult,
    course: &Course,
) -> FitnessBreakdown {
    let distances_to_goal =
        shortest_distances_to_goal(course);

    let start_index =
        course.start.y * course.width
            + course.start.x;

    let start_distance =
        distances_to_goal[start_index]
            .expect(
                "Course start must be reachable from goal",
            );

    let mut closest_distance =
        start_distance;

    let mut steps_to_closest =
        0;

    for (step, position) in
        result.path.iter().enumerate()
    {
        let index =
            position.y * course.width
                + position.x;

        let Some(distance) =
            distances_to_goal[index]
        else {
            continue;
        };

        /*
         * Only update when the actor achieves a strictly better
         * BFS distance than it has ever achieved before.
         *
         * This means repeated visits to the same closest point
         * do not affect steps_to_closest. Revisits are penalized
         * separately.
         */
        if distance < closest_distance {
            closest_distance = distance;
            steps_to_closest = step;
        }
    }

    let progress_cells =
        start_distance.saturating_sub(
            closest_distance,
        );

    let path_stats =
        calculate_path_stats(
            &result.path,
            course,
        );

    /*
     * Primary unsolved objective:
     * make real navigational progress according to the course's
     * actual traversable topology.
     */
    let progress_fitness =
        progress_cells as f64
            * PROGRESS_REWARD_PER_CELL;

    /*
     * Encourage getting to the best point efficiently, but keep
     * this penalty modest enough that necessary detours are still
     * viable.
     */
    let steps_to_closest_fitness =
        -(steps_to_closest as f64
            * STEPS_TO_CLOSEST_PENALTY);

    /*
     * Exploration is useful, but intentionally weaker than actual
     * progress toward the goal.
     */
    let unique_cell_fitness =
        path_stats.unique_cells as f64
            * UNIQUE_CELL_REWARD;

    /*
     * Generic movement has no intrinsic reward. A successful move
     * matters because it either discovers a new tile or contributes
     * toward progress.
     */
    let successful_move_fitness =
        path_stats.successful_moves as f64
            * SUCCESSFUL_MOVE_REWARD;

    /*
     * Repeatedly traversing known territory is discouraged, but the
     * penalty remains modest because legitimate backtracking may be
     * required.
     */
    let revisit_fitness =
        -(path_stats.revisits as f64
            * REVISIT_PENALTY);

    /*
     * Collisions are undesirable, but not catastrophically so.
     * A blind actor may need occasional failed actions while learning
     * obstacle structure.
     */
    let collision_fitness =
        -(result.collisions as f64
            * COLLISION_PENALTY);

    /*
     * Reaching the goal should dominate any unsolved behavior.
     */
    let goal_fitness =
        if result.reached_goal {
            GOAL_REWARD
        } else {
            0.0
        };

    /*
     * Once the goal is reached, prefer more efficient solutions.
     */
    let solution_step_fitness =
        if result.reached_goal {
            -(result.steps as f64
                * SOLUTION_STEP_PENALTY)
        } else {
            0.0
        };

    let total =
        progress_fitness
            + steps_to_closest_fitness
            + unique_cell_fitness
            + successful_move_fitness
            + revisit_fitness
            + collision_fitness
            + goal_fitness
            + solution_step_fitness;

    FitnessBreakdown {
        progress_cells,
        progress_fitness,

        steps_to_closest,
        steps_to_closest_fitness,

        unique_cells:
            path_stats.unique_cells,

        unique_cell_fitness,

        successful_moves:
            path_stats.successful_moves,

        successful_move_fitness,

        revisits:
            path_stats.revisits,

        revisit_fitness,

        collisions:
            result.collisions,

        collision_fitness,

        goal_fitness,
        solution_step_fitness,

        total,
    }
}

struct PathStats {
    unique_cells: usize,
    successful_moves: usize,
    revisits: usize,
}

fn calculate_path_stats(
    path: &[Position],
    course: &Course,
) -> PathStats {
    let mut visited =
        vec![
            false;
            course.width * course.height
        ];

    let mut unique_cells = 0;
    let mut revisits = 0;

    for position in path {
        let index =
            position.y * course.width
                + position.x;

        if visited[index] {
            revisits += 1;
        } else {
            visited[index] = true;
            unique_cells += 1;
        }
    }

    let successful_moves =
        path.len().saturating_sub(1);

    PathStats {
        unique_cells,
        successful_moves,
        revisits,
    }
}

fn shortest_distances_to_goal(
    course: &Course,
) -> Vec<Option<usize>> {
    use std::collections::VecDeque;

    let mut distances =
        vec![
            None;
            course.width * course.height
        ];

    let mut queue =
        VecDeque::new();

    let goal_index =
        course.goal.y * course.width
            + course.goal.x;

    distances[goal_index] =
        Some(0);

    queue.push_back(
        course.goal,
    );

    while let Some(current) =
        queue.pop_front()
    {
        let current_index =
            current.y * course.width
                + current.x;

        let current_distance =
            distances[current_index]
                .expect(
                    "Queued position must have a distance",
                );

        let neighbors = [
            current
                .x
                .checked_sub(1)
                .map(|x| Position {
                    x,
                    y: current.y,
                }),

            (current.x + 1
                < course.width)
                .then_some(
                    Position {
                        x: current.x + 1,
                        y: current.y,
                    },
                ),

            current
                .y
                .checked_sub(1)
                .map(|y| Position {
                    x: current.x,
                    y,
                }),

            (current.y + 1
                < course.height)
                .then_some(
                    Position {
                        x: current.x,
                        y: current.y + 1,
                    },
                ),
        ];

        for neighbor in
            neighbors.into_iter().flatten()
        {
            if !course.is_walkable(
                neighbor,
            ) {
                continue;
            }

            let neighbor_index =
                neighbor.y * course.width
                    + neighbor.x;

            if distances[
                neighbor_index
            ]
            .is_some()
            {
                continue;
            }

            distances[
                neighbor_index
            ] =
                Some(
                    current_distance + 1,
                );

            queue.push_back(
                neighbor,
            );
        }
    }

    distances
}