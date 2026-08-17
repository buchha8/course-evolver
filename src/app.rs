use macroquad::prelude::{
    get_frame_time,
    is_key_pressed,
    KeyCode,
};

use crate::history::GenerationRecord;

pub struct App {
    pub selected_generation: Option<usize>,
    pub selected_individual: Option<usize>,
    pub selected_course: usize,

    pub playback_index: usize,
    pub playing: bool,
    pub playback_speed: f32,

    playback_accumulator: f32,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected_generation: None,
            selected_individual: None,
            selected_course: 0,

            playback_index: 0,
            playing: true,
            playback_speed: 30.0,

            playback_accumulator: 0.0,
        }
    }

    pub fn update(
        &mut self,
        history: &[GenerationRecord],
    ) {
        if history.is_empty() {
            self.selected_generation = None;
            self.selected_individual = None;
            return;
        }

        if self.selected_generation.is_none() {
            self.selected_generation = Some(0);
            self.select_best_individual(&history[0]);
        }

        if is_key_pressed(KeyCode::PageUp) {
            self.select_previous_generation(history);
        }

        if is_key_pressed(KeyCode::PageDown) {
            self.select_next_generation(history);
        }

        let generation_index =
            self.selected_generation.unwrap();

        let generation =
            &history[generation_index];

        if generation.individuals.is_empty() {
            self.selected_individual = None;
            return;
        }

        if self.selected_individual.is_none() {
            self.select_best_individual(generation);
        }

        if is_key_pressed(KeyCode::Left) {
            self.select_previous_individual(
                generation.individuals.len(),
            );
        }

        if is_key_pressed(KeyCode::Right) {
            self.select_next_individual(
                generation.individuals.len(),
            );
        }

        if is_key_pressed(KeyCode::B) {
            self.select_best_individual(generation);
        }

        let individual_index =
            self.selected_individual.unwrap();

        let individual =
            &generation.individuals[individual_index];

        if is_key_pressed(KeyCode::Comma) {
            self.select_previous_course(
                individual.results.len(),
            );
        }

        if is_key_pressed(KeyCode::Period) {
            self.select_next_course(
                individual.results.len(),
            );
        }

        if is_key_pressed(KeyCode::Space) {
            self.playing = !self.playing;
        }

        if is_key_pressed(KeyCode::R) {
            self.restart_playback();
        }

        if is_key_pressed(KeyCode::Up) {
            self.playback_speed =
                (self.playback_speed * 2.0)
                    .min(1000.0);
        }

        if is_key_pressed(KeyCode::Down) {
            self.playback_speed =
                (self.playback_speed / 2.0)
                    .max(1.0);
        }

        let result =
            &individual.results[
                self.selected_course
            ];

        let path_length =
            result.simulation.path.len();

        if path_length == 0 {
            return;
        }

        if self.playback_index >= path_length {
            self.playback_index =
                path_length - 1;
        }

        if !self.playing {
            return;
        }

        self.playback_accumulator +=
            get_frame_time()
                * self.playback_speed;

        while self.playback_accumulator >= 1.0 {
            if self.playback_index + 1
                >= path_length
            {
                self.playing = false;
                self.playback_accumulator = 0.0;
                break;
            }

            self.playback_index += 1;
            self.playback_accumulator -= 1.0;
        }
    }

    fn select_previous_generation(
        &mut self,
        history: &[GenerationRecord],
    ) {
        let current =
            self.selected_generation.unwrap_or(0);

        let next =
            if current == 0 {
                history.len() - 1
            } else {
                current - 1
            };

        self.selected_generation = Some(next);

        self.select_best_individual(
            &history[next],
        );
    }

    fn select_next_generation(
        &mut self,
        history: &[GenerationRecord],
    ) {
        let current =
            self.selected_generation.unwrap_or(0);

        let next =
            (current + 1) % history.len();

        self.selected_generation = Some(next);

        self.select_best_individual(
            &history[next],
        );
    }

    fn select_previous_individual(
        &mut self,
        individual_count: usize,
    ) {
        let current =
            self.selected_individual.unwrap_or(0);

        let next =
            if current == 0 {
                individual_count - 1
            } else {
                current - 1
            };

        self.selected_individual = Some(next);

        // Intentionally keep selected_course unchanged so
        // the same course can be compared across individuals.
        self.restart_playback();
    }

    fn select_next_individual(
        &mut self,
        individual_count: usize,
    ) {
        let current =
            self.selected_individual.unwrap_or(0);

        let next =
            (current + 1) % individual_count;

        self.selected_individual = Some(next);

        // Intentionally keep selected_course unchanged so
        // the same course can be compared across individuals.
        self.restart_playback();
    }

    fn select_best_individual(
        &mut self,
        generation: &GenerationRecord,
    ) {
        self.selected_individual =
            generation.best_individual_index();

        self.select_best_course(generation);
    }

    fn select_best_course(
        &mut self,
        generation: &GenerationRecord,
    ) {
        let Some(individual_index) =
            self.selected_individual
        else {
            self.selected_course = 0;
            return;
        };

        let individual =
            &generation.individuals[
                individual_index
            ];

        self.selected_course =
            individual
                .best_course_index()
                .unwrap_or(0);

        self.restart_playback();
    }

    fn select_previous_course(
        &mut self,
        course_count: usize,
    ) {
        if course_count == 0 {
            return;
        }

        if self.selected_course == 0 {
            self.selected_course =
                course_count - 1;
        } else {
            self.selected_course -= 1;
        }

        self.restart_playback();
    }

    fn select_next_course(
        &mut self,
        course_count: usize,
    ) {
        if course_count == 0 {
            return;
        }

        self.selected_course =
            (self.selected_course + 1)
                % course_count;

        self.restart_playback();
    }

    fn restart_playback(&mut self) {
        self.playback_index = 0;
        self.playback_accumulator = 0.0;
        self.playing = true;
    }
}