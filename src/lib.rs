#[macro_use]
extern crate lazy_static;

extern crate time;

#[cfg(test)]
mod test;

mod timer;

pub use timer::{oneshot_ms, periodic_ms};
