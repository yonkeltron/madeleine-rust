use std::cell::RefCell;
use std::path::PathBuf;

use commitlog::*;
use ulid::Ulid;

use crate::command::Command;
use crate::madeleine_error::MadeleineError;

/// Represents an append-only log of commands.
/// Backed by a stateful store on disk.
pub(crate) struct CommandLog {
  commit_log: RefCell<CommitLog>,
}

impl CommandLog {
  /// Constructor function.
  pub fn new(store_dir: PathBuf) -> Result<Self, MadeleineError> {
    let opts = LogOptions::new(store_dir);
    let commit_log = RefCell::new(CommitLog::new(opts)?);

    Ok(Self { commit_log })
  }

  /// Append a command to the log, serializing it first.
  pub fn append_command<'a, C: Command<'a>>(&self, command: C) -> Result<Offset, MadeleineError> {
    let log_entry = (Ulid::new(), command);

    let serialized_command = serde_json::to_string(&log_entry)?;

    let mut commit_log = self.commit_log.try_borrow_mut()?;

    let offset = commit_log.append_msg(&serialized_command)?;

    Ok(offset)
  }

  /// Get the length of the underlying commit log.
  pub fn len(&self) -> u64 {
    let extracted = self.commit_log.borrow().last_offset().unwrap_or(0);

    if extracted > 0 {
      extracted + 1
    } else {
      extracted
    }
  }
}
