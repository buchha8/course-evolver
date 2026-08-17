use macroquad::prelude::*;

use crate::app::App;
use crate::course::{Cell, Position};
use crate::history::{
    GenerationRecord,
    IndividualRecord,
};
use crate::job::JobResult;

const CELL_SIZE: f32 = 12.0;

pub fn render(
    app: &App,
    history: &[GenerationRecord],
) {
    clear_background(WHITE);

    let Some(generation_index) =
        app.selected_generation
    else {
        draw_text(
            "Waiting for generation results...",
            30.0,
            50.0,
            28.0,
            BLACK,
        );

        return;
    };

    let generation =
        &history[generation_index];

    let Some(individual_index) =
        app.selected_individual
    else {
        return;
    };

    let individual =
        &generation.individuals[
            individual_index
        ];

    if individual.results.is_empty() {
        return;
    }

    let course_index =
        app.selected_course.min(
            individual.results.len() - 1,
        );

    let result =
        &individual.results[
            course_index
        ];

    draw_course(result);
    draw_path(app, result);

    draw_information(
        app,
        history,
        generation_index,
        generation,
        individual,
        individual_index,
        result,
        course_index,
    );
}

fn draw_course(
    result: &JobResult,
) {
    let course = &result.course;

    for y in 0..course.height {
        for x in 0..course.width {
            let position =
                Position { x, y };

            let color =
                if position == course.start {
                    GREEN
                } else if position == course.goal {
                    RED
                } else {
                    match course.cell(position) {
                        Cell::Open =>
                            LIGHTGRAY,

                        Cell::Obstacle =>
                            DARKGRAY,
                    }
                };

            draw_rectangle(
                x as f32 * CELL_SIZE,
                y as f32 * CELL_SIZE,
                CELL_SIZE - 1.0,
                CELL_SIZE - 1.0,
                color,
            );
        }
    }
}

fn draw_path(
    app: &App,
    result: &JobResult,
) {
    let path =
        &result.simulation.path;

    if path.is_empty() {
        return;
    }

    let playback_index =
        app.playback_index.min(
            path.len() - 1,
        );

    let visible_path =
        &path[..=playback_index];

    for window in visible_path.windows(2) {
        let first = window[0];
        let second = window[1];

        let x1 =
            first.x as f32 * CELL_SIZE
                + CELL_SIZE / 2.0;

        let y1 =
            first.y as f32 * CELL_SIZE
                + CELL_SIZE / 2.0;

        let x2 =
            second.x as f32 * CELL_SIZE
                + CELL_SIZE / 2.0;

        let y2 =
            second.y as f32 * CELL_SIZE
                + CELL_SIZE / 2.0;

        draw_line(
            x1,
            y1,
            x2,
            y2,
            2.0,
            BLUE,
        );
    }

    let current =
        path[playback_index];

    let agent_x =
        current.x as f32 * CELL_SIZE
            + CELL_SIZE / 2.0;

    let agent_y =
        current.y as f32 * CELL_SIZE
            + CELL_SIZE / 2.0;

    draw_circle(
        agent_x,
        agent_y,
        CELL_SIZE * 0.35,
        BLUE,
    );
}

fn draw_information(
    app: &App,
    history: &[GenerationRecord],
    generation_index: usize,
    generation: &GenerationRecord,
    individual: &IndividualRecord,
    individual_index: usize,
    result: &JobResult,
    course_index: usize,
) {
    let panel_x =
        result.course.width as f32
            * CELL_SIZE
            + 20.0;

    let best_fitness =
        generation
            .best_aggregate_fitness()
            .unwrap_or(0.0);

    let mean_fitness =
        generation
            .mean_aggregate_fitness()
            .unwrap_or(0.0);

    let solved_courses =
        generation.total_courses_solved();

    let total_courses =
        generation
            .individuals
            .iter()
            .map(|individual| {
                individual.results.len()
            })
            .sum::<usize>();

    draw_text(
        "Course Evolver",
        panel_x,
        25.0,
        26.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Generation: {} / {}",
            generation_index + 1,
            history.len()
        ),
        panel_x,
        52.0,
        18.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Best aggregate: {:.2}",
            best_fitness
        ),
        panel_x,
        74.0,
        18.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Mean aggregate: {:.2}",
            mean_fitness
        ),
        panel_x,
        96.0,
        18.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Course solves: {} / {}",
            solved_courses,
            total_courses
        ),
        panel_x,
        118.0,
        18.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Individual: {} / {}  ID {}",
            individual_index + 1,
            generation.individuals.len(),
            individual.individual_id
        ),
        panel_x,
        150.0,
        18.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Aggregate fitness: {:.2}",
            individual.aggregate_fitness
        ),
        panel_x,
        172.0,
        18.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Course: {} / {}",
            course_index + 1,
            individual.results.len()
        ),
        panel_x,
        194.0,
        18.0,
        BLACK,
    );

    draw_text(
        &format!(
            "COURSE FITNESS: {:.2}",
            result.fitness
        ),
        panel_x,
        224.0,
        20.0,
        BLACK,
    );

    let breakdown =
        &result.fitness_breakdown;

    draw_text(
        &format!(
            "Progress: {} cells   {:+.2}",
            breakdown.progress_cells,
            breakdown.progress_fitness
        ),
        panel_x,
        252.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Steps to closest: {}   {:+.2}",
            breakdown.steps_to_closest,
            breakdown.steps_to_closest_fitness
        ),
        panel_x,
        274.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Unique cells: {}    {:+.2}",
            breakdown.unique_cells,
            breakdown.unique_cell_fitness
        ),
        panel_x,
        296.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Successful moves: {}  {:+.2}",
            breakdown.successful_moves,
            breakdown.successful_move_fitness
        ),
        panel_x,
        318.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Revisits: {}        {:+.2}",
            breakdown.revisits,
            breakdown.revisit_fitness
        ),
        panel_x,
        340.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Collisions: {}      {:+.2}",
            breakdown.collisions,
            breakdown.collision_fitness
        ),
        panel_x,
        362.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Goal reward:       {:+.2}",
            breakdown.goal_fitness
        ),
        panel_x,
        384.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Solution steps:    {:+.2}",
            breakdown.solution_step_fitness
        ),
        panel_x,
        406.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Reached goal: {}   Steps: {}",
            result.simulation.reached_goal,
            result.simulation.steps
        ),
        panel_x,
        438.0,
        17.0,
        BLACK,
    );

    draw_text(
        &format!(
            "Playback: {}   Speed: {:.0}/s",
            app.playback_index,
            app.playback_speed
        ),
        panel_x,
        460.0,
        17.0,
        BLACK,
    );

    draw_text(
        if app.playing {
            "Playing"
        } else {
            "Paused"
        },
        panel_x,
        482.0,
        17.0,
        BLACK,
    );

    draw_text(
        "PgUp/PgDn: generation",
        panel_x,
        518.0,
        15.0,
        BLACK,
    );

    draw_text(
        "Left/Right: individual",
        panel_x,
        538.0,
        15.0,
        BLACK,
    );

    draw_text(
        "B: best individual + course",
        panel_x,
        558.0,
        15.0,
        BLACK,
    );

    draw_text(
        ", / . : course",
        panel_x,
        578.0,
        15.0,
        BLACK,
    );

    draw_text(
        "Space: pause   R: restart",
        panel_x,
        598.0,
        15.0,
        BLACK,
    );
}