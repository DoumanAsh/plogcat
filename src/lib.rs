#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

pub mod cli;
pub mod errors;
pub mod color;
mod parser;
pub use parser::{parse, LogCatLine};
