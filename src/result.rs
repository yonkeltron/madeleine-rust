use crate::madeleine_error::MadeleineError;

pub type Result<T, E = MadeleineError> = core::result::Result<T, E>;
