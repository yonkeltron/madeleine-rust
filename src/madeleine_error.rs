use thiserror::Error;

use std::io;

/// Custom error type for Madeleine.
#[derive(Error, Debug)]
pub enum MadeleineError {
  /// Error related to File I/O and disk operations.
  #[error("File I/O error")]
  FileIOError(#[from] io::Error),
  /// Errors relating to snapshot files.
  #[error("Snapshot error: {0}")]
  SnapshotError(String),
  /// Any sort of serialization error (currently only JSON).
  #[error("Serialization error")]
  SerializationError(#[from] serde_json::Error),
  // #[error("Deserialization error")]
  // DeserializationError(#[from] serde_json::de::Error),
  /// Errors relating to the command log's storage.
  #[error("Command Log error")]
  CommandLogStorageError(#[from] rusqlite::Error),
  /// Error relating to appending to the command log.
  #[error("Command Log append error: {0}")]
  CommandLogAppendError(String),
}
