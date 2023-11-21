#![feature(test)]
#![feature(wrapping_next_power_of_two)]

#[cfg(test)]
mod tests;

mod mutators;
mod feedback;
mod executor;
mod input;
mod graph;
mod scheduler;
mod observer;

#[cfg(feature = "user-stats")]
mod stats;

pub mod prelude {
    pub use super::{
        executor::*,
        feedback::*,
        graph::*,
        input::*,
        mutators::*,
        observer::*,
        scheduler::*,
    };
}

pub mod tt;
