use crate::MadeleineError;

/// Internal Result type alias.
pub type Result<T, E = MadeleineError> = std::result::Result<T, E>;
