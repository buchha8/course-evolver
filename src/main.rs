mod app;
mod controller;
mod course;
mod evolution;
mod fitness;
mod history;
mod job;
mod policy;
mod renderer;
mod scheduler;
mod simulation;

use app::App;
use controller::RunController;
use macroquad::prelude::{next_frame, Conf};

const WORKER_COUNT: usize = 4;
const POPULATION_SIZE: usize = 100;
const MAX_STEPS: usize = 2_000;
const GENERATION_COUNT: usize = 100;

fn window_conf() -> Conf {
    Conf {
        window_title: "Course Evolver".to_owned(),
        window_width: 1000,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut controller =
        RunController::new(
            WORKER_COUNT,
            POPULATION_SIZE,
            MAX_STEPS,
            GENERATION_COUNT,
        );

    let mut app = App::new();

    loop {
        controller.update();

        app.update(
            controller.history(),
        );

        renderer::render(
            &app,
            controller.history(),
        );

        next_frame().await;
    }
}