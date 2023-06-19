
#![feature(array_windows)]
mod primatives;
mod classes;
pub use memory;
pub use memory::memory_macros as memory_macros;
pub mod prelude;

#[cfg(feature = "egui")]
mod egui;