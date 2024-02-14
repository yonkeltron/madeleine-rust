#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! Transparent object persistence in the tradition of Ruby's [`madeleine` gem](https://github.com/ghostganz/madeleine).
//! In turn, that's inspired by Java's earlier [Prevalayer](https://prevayler.org/).

/// Module containing types and logic for Command implementations.
pub mod command;
mod command_log;
/// High-level public interface.
pub mod madeleine;
/// Error type.
pub mod madeleine_error;
/// Result type.
pub mod result;

pub use crate::command::Command;
pub use crate::madeleine::Madeleine;
pub use crate::madeleine_error::MadeleineError;
