#![feature(test)]
#![feature(wrapping_next_power_of_two)]

#[cfg(test)]
mod tests;

mod executor;
mod feedback;
mod graph;
mod input;
mod mutators;
mod observer;
mod stats;

pub mod prelude {
    pub use super::{
        executor::*,
        feedback::*,
        input::*,
        mutators::*,
        observer::*,
        graph::*,
    };
}
pub mod tt;
