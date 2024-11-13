use std::fs;
use std::path::PathBuf;

use rusqlite::{params, Connection};
use ulid::Ulid;

use crate::command::Command;
use crate::madeleine_error::MadeleineError;
use crate::madeleine_result::Result;

const CREATE_TABLE_SQL: &str = include_str!("queries/create_command_log_table.sql");
const INSERT_COMMAND_SQL: &str = include_str!("queries/insert_command.sql");
const COUNT_COMMANDS_SQL: &str = include_str!("queries/count_commands.sql");

/// Represents an append-only log of commands.
/// Backed by a stateful store on disk.
pub(crate) struct CommandLog {
  storage: Connection,
}

impl CommandLog {
  /// Constructor function.
  pub fn new(store_dir: PathBuf) -> Result<Self, MadeleineError> {
    fs::create_dir_all(&store_dir)?;

    let storage_path = store_dir.join("madeleine.db");
    let storage = Connection::open(storage_path)?;

    storage.execute(CREATE_TABLE_SQL, params![])?;

    Ok(Self { storage })
  }

  /// Append a command to the log, serializing it first.
  pub fn append_command<'a, C: Command<'a>>(&self, command: C) -> Result<()> {
    let serialized_command = serde_json::to_string(&command)?;

    let ulid = Ulid::new().to_string();

    let inserted = self.storage.execute(
      INSERT_COMMAND_SQL,
      &[(":command", &serialized_command), (":ulid", &ulid)],
    )?;

    if inserted < 1 {
      Err(MadeleineError::CommandLogAppendError(String::from(
        "Unable to INSERT command into storage",
      )))
    } else {
      Ok(())
    }
  }

  /// Get the length of the underlying commit log.
  pub fn len(&self) -> Result<u64> {
    let extracted = self
      .storage
      .query_row(COUNT_COMMANDS_SQL, params![], |row| row.get("count"))?;

    Ok(extracted)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use assert_fs::prelude::*;

  #[test]
  fn test_init_command_log() {
    let temp_dir = assert_fs::TempDir::new().expect("unable to create temp dir in test");

    temp_dir
      .child("madeleine.db")
      .assert(predicates::path::missing());

    let _command_log = CommandLog::new(temp_dir.path().to_path_buf())
      .expect("unable to instantiate command log in test");

    temp_dir
      .child("madeleine.db")
      .assert(predicates::path::exists());
  }
}
