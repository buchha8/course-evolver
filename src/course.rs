use rand::Rng;
use std::collections::VecDeque;
use std::fmt;

const MIN_BLOCK_COUNT: usize = 20;
const MAX_BLOCK_COUNT: usize = 35;

const MIN_BLOCK_SIZE: usize = 3;
const MAX_BLOCK_SIZE: usize = 10;

const MIN_OBSTACLE_DENSITY: f32 = 0.25;
const MAX_OBSTACLE_DENSITY: f32 = 0.50;

const MIN_PATH_LENGTH_FACTOR: f32 = 1.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Open,
    Obstacle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Course {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub start: Position,
    pub goal: Position,
}

impl Course {
    pub fn new(
        width: usize,
        height: usize,
        start: Position,
        goal: Position,
    ) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::Open; width * height],
            start,
            goal,
        }
    }

    pub fn random(width: usize, height: usize) -> Self {
        assert!(width >= MAX_BLOCK_SIZE);
        assert!(height >= MAX_BLOCK_SIZE);

        loop {
            let start = Position { x: 0, y: 0 };

            let goal = Position {
                x: width - 1,
                y: height - 1,
            };

            let mut course =
                Self::new(width, height, start, goal);

            let mut rng = rand::rng();

            let block_count =
                rng.random_range(
                    MIN_BLOCK_COUNT..=MAX_BLOCK_COUNT,
                );

            for _ in 0..block_count {
                course.add_random_block(&mut rng);
            }

            let density =
                course.obstacle_density();

            if density < MIN_OBSTACLE_DENSITY
                || density > MAX_OBSTACLE_DENSITY
            {
                continue;
            }

            let Some(shortest_path_length) =
                course.shortest_path_length()
            else {
                continue;
            };

            let direct_distance =
                manhattan_distance(start, goal);

            let minimum_path_length =
                (direct_distance as f32
                    * MIN_PATH_LENGTH_FACTOR)
                    .ceil() as usize;

            if shortest_path_length
                < minimum_path_length
            {
                continue;
            }

            return course;
        }
    }

    fn add_random_block(
        &mut self,
        rng: &mut impl Rng,
    ) {
        let width =
            rng.random_range(
                MIN_BLOCK_SIZE..=MAX_BLOCK_SIZE,
            );

        let height =
            rng.random_range(
                MIN_BLOCK_SIZE..=MAX_BLOCK_SIZE,
            );

        let x =
            rng.random_range(
                0..=self.width - width,
            );

        let y =
            rng.random_range(
                0..=self.height - height,
            );

        for dy in 0..height {
            for dx in 0..width {
                let position = Position {
                    x: x + dx,
                    y: y + dy,
                };

                if position != self.start
                    && position != self.goal
                {
                    self.set_cell(
                        position,
                        Cell::Obstacle,
                    );
                }
            }
        }
    }

    fn index(&self, position: Position) -> usize {
        position.y * self.width + position.x
    }

    pub fn cell(
        &self,
        position: Position,
    ) -> Cell {
        self.cells[self.index(position)]
    }

    pub fn set_cell(
        &mut self,
        position: Position,
        cell: Cell,
    ) {
        let index = self.index(position);
        self.cells[index] = cell;
    }

    pub fn is_walkable(
        &self,
        position: Position,
    ) -> bool {
        position.x < self.width
            && position.y < self.height
            && self.cell(position) == Cell::Open
    }

    fn obstacle_density(&self) -> f32 {
        let obstacle_count = self
            .cells
            .iter()
            .filter(|cell| {
                **cell == Cell::Obstacle
            })
            .count();

        obstacle_count as f32
            / self.cells.len() as f32
    }

    pub fn shortest_path_length(
        &self,
    ) -> Option<usize> {
        let mut distances =
            vec![usize::MAX; self.width * self.height];

        let mut queue = VecDeque::new();

        distances[self.index(self.start)] = 0;
        queue.push_back(self.start);

        while let Some(current) =
            queue.pop_front()
        {
            let current_distance =
                distances[self.index(current)];

            if current == self.goal {
                return Some(current_distance);
            }

            for neighbor in self.neighbors(current) {
                if !self.is_walkable(neighbor) {
                    continue;
                }

                let index =
                    self.index(neighbor);

                if distances[index] != usize::MAX {
                    continue;
                }

                distances[index] =
                    current_distance + 1;

                queue.push_back(neighbor);
            }
        }

        None
    }

    fn neighbors(
        &self,
        position: Position,
    ) -> Vec<Position> {
        let mut neighbors =
            Vec::with_capacity(4);

        if position.x > 0 {
            neighbors.push(Position {
                x: position.x - 1,
                y: position.y,
            });
        }

        if position.x + 1 < self.width {
            neighbors.push(Position {
                x: position.x + 1,
                y: position.y,
            });
        }

        if position.y > 0 {
            neighbors.push(Position {
                x: position.x,
                y: position.y - 1,
            });
        }

        if position.y + 1 < self.height {
            neighbors.push(Position {
                x: position.x,
                y: position.y + 1,
            });
        }

        neighbors
    }
}

fn manhattan_distance(
    a: Position,
    b: Position,
) -> usize {
    a.x.abs_diff(b.x)
        + a.y.abs_diff(b.y)
}

impl fmt::Display for Course {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let position =
                    Position { x, y };

                let character =
                    if position == self.start {
                        'S'
                    } else if position == self.goal {
                        'G'
                    } else {
                        match self.cell(position) {
                            Cell::Open => '.',
                            Cell::Obstacle => '#',
                        }
                    };

                write!(f, "{character}")?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}