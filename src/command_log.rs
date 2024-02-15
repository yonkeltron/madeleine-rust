use std::path::Path;

use rusqlite::Connection;
use ulid::Ulid;

use crate::command::Command;
use crate::madeleine_error::MadeleineError;
use crate::result::Result;

/// Represents an append-only log of commands.
/// Backed by a stateful store on disk.
pub(crate) struct CommandLog {
  conn: Connection,
}

const CREATE_LOG_TABLE_SQL: &str = include_str!("queries/create_log_table.sql");
const INSERT_COMMIT_SQL: &str = include_str!("queries/insert_commit.sql");
const COUNT_COMMITS_SQL: &str = include_str!("queries/count_commits.sql");

impl CommandLog {
  /// Constructor function.
  pub fn new<P: AsRef<Path>>(store_dir: P) -> Result<Self, MadeleineError> {
    let conn = Connection::open(store_dir)?;

    conn.execute_batch(CREATE_LOG_TABLE_SQL)?;

    Ok(Self { conn })
  }

  /// Append a command to the log, serializing it first.
  pub fn append_command<'a, C: Command<'a>>(&self, command: C) -> Result<(), MadeleineError> {
    let ulid = Ulid::new().to_string();

    let serialized_command = serde_json::to_string(&command)?;

    self.conn.execute(
      INSERT_COMMIT_SQL,
      &[
        (":ulid", ulid.as_str()),
        (":data", serialized_command.as_str()),
      ],
    )?;

    Ok(())
  }

  /// Get the length of the underlying commit log.
  pub fn len(&self) -> Result<u64> {
    let extracted = self
      .conn
      .query_row(COUNT_COMMITS_SQL, [], |row| row.get(0))?;

    let res = if extracted > 0 {
      extracted + 1
    } else {
      extracted
    };

    Ok(res)
  }
}
