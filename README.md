# Course Evolver

Course Evolver is a Rust experiment in evolutionary optimization, concurrent job orchestration, and simple recurrent neural-network policies.

A population of agents attempts to navigate randomly generated obstacle courses. Each agent's behavior is controlled by a small recurrent neural network whose weights form its heritable genome. Individuals are evaluated, selected based on fitness, mutated, and carried forward across generations.

The project is primarily intended as an exploration of orchestrating computationally expensive workloads rather than as an optimized maze-solving system.

![Example 1](https://github.com/buchha8/Past-Projects/blob/main/Rust%20Projects/course-evolver/course-evolver-1.gif)

![Example 2](https://github.com/buchha8/Past-Projects/blob/main/Rust%20Projects/course-evolver/course-evolver-2.gif)

## Features

- Evolutionary population of neural-network policies
- Recurrent hidden state for behavior with memory
- Multi-course evaluation of each individual
- Concurrent simulation jobs using a bounded worker pool
- Randomly generated obstacle courses
- Fitness-based selection and mutation
- Fitness signals for:
  - progress toward the goal
  - unique cells explored
  - time required to reach the closest point
  - revisited cells
  - collisions
  - successful course completion
  - solution length
- Generation history retained for inspection
- Macroquad GUI for viewing completed simulations
- Playback controls for inspecting agent behavior
- Per-course fitness breakdown in the GUI

## Architecture

The project separates orchestration, evolution, simulation, and visualization into independent modules:

- `main.rs` — application entry point and main GUI/update loop
- `controller.rs` — coordinates generations and the overall run
- `scheduler.rs` — manages bounded concurrent execution of simulation jobs
- `job.rs` — defines individual simulation jobs and their results
- `evolution.rs` — owns the evolving population, selection, and mutation
- `policy.rs` — recurrent neural-network policy and heritable weights
- `simulation.rs` — executes an agent against a course
- `fitness.rs` — calculates fitness and detailed fitness components
- `course.rs` — course representation and random course generation
- `history.rs` — stores completed generation/individual results for inspection
- `app.rs` — GUI state and user input
- `renderer.rs` — renders courses, playback, and statistics

## Evolution

Each individual contains a heritable recurrent neural-network policy.

The policy receives limited information about the agent's current state rather than a complete representation of the course. Its recurrent hidden state allows behavior to depend on previous experience.

The network weights are inherited between generations. Higher-fitness individuals are preferentially selected to produce mutated descendants.

Hidden-state values themselves are temporary simulation state and are not inherited. The weights controlling how observations affect hidden state, and how hidden state affects future actions, are inherited.

## Fitness

Fitness currently combines several signals.

Progress toward the goal is measured using actual shortest-path distance through the course rather than simple geometric distance. Exploration is rewarded through unique cells visited, while revisits and collisions are penalized.

The number of moves required to achieve an individual's closest point to the goal is also penalized. This discourages agents from collecting exploration rewards indefinitely before eventually making useful progress.

Reaching the goal provides a large additional reward, with faster solutions preferred.

The individual fitness components are displayed separately in the GUI to make tuning and unexpected evolutionary behavior easier to inspect.

## Multi-Course Evaluation

Individuals can be evaluated against multiple courses in the same generation.

Each individual receives an aggregate fitness across its course results. This reduces dependence on a single favorable course and provides a path toward increasingly computationally expensive evaluation workloads.

## Orchestration

Simulation jobs are CPU-bound and can be executed independently.

Rather than starting every simulation simultaneously, `scheduler.rs` limits the number of active jobs. This allows population size and multi-course evaluation to scale without creating an unbounded number of concurrent workers.

The worker count can be tuned independently from population size and generation count.

## Running

A standard development build can be run with:

    cargo run

For substantially better simulation performance, use an optimized build:

    cargo run --release

Rust debug builds are significantly slower for the CPU-heavy simulation workload.

## Configuration

The main workload parameters are currently configured in `main.rs`, including:

- worker count
- population size
- maximum simulation steps
- generation count

Evolutionary parameters, mutation behavior, neural-network dimensions, course generation, and fitness weights are defined in their corresponding modules.

## Status

This is an experimental project and remains a work in progress.

The current system successfully demonstrates:

- concurrent simulation orchestration
- evolutionary inheritance and mutation
- recurrent policies
- multi-course evaluation
- generation history
- visual playback and inspection
- nontrivial evolved navigation behavior

Policy quality is still inconsistent, and fitness design, mutation behavior, network architecture, and course evaluation remain areas for experimentation.