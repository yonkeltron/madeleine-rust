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
  /// Errors relating to appending commands to the command log's commit log.
  #[error("Commit Log Append error")]
  CommitLogAppendError(#[from] commitlog::AppendError),
  /// Errors relating to reading from the command log's commit log.
  #[error("Commit Log Read error")]
  CommitLogReadError(#[from] commitlog::ReadError),
  /// Internal error related to mutable borrowing.
  #[error("Mutable Borrow error")]
  BorrowMutError(#[from] std::cell::BorrowMutError),
  /// Internal error related to borrowing.
  #[error("Borrow error")]
  BorrowError(#[from] std::cell::BorrowError),
}
