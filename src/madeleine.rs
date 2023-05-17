use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

use commitlog::Offset;
use serde::{Deserialize, Serialize};

use crate::command::Command;
use crate::command_log::CommandLog;
use crate::madeleine_error::MadeleineError;

const COMMAND_LOG_DIR_NAME: &str = "command_log";
const SNAPSHOT_FILE_SUFFIX: &str = "snapshot";

/// Top-level struct providing the public interface for transparent object persistence.
pub struct Madeleine<SystemState: Clone + for<'a> Deserialize<'a> + Serialize> {
  command_log: CommandLog,
  internal_state: RefCell<SystemState>,
  location_dir_path: PathBuf,
}

impl<SystemState: Clone + for<'a> Deserialize<'a> + Serialize> Madeleine<SystemState> {
  /// Generalized constructor.
  pub fn new<C>(location_dir_path: PathBuf, constructor: C) -> Result<Self, MadeleineError>
  where
    C: FnOnce() -> SystemState,
  {
    let log_dir = location_dir_path.join(COMMAND_LOG_DIR_NAME);
    let command_log = CommandLog::new(log_dir)?;
    let internal_state = RefCell::new(constructor());

    Ok(Self {
      command_log,
      internal_state,
      location_dir_path,
    })
  }

  /// Resume from existing instance on disk.
  pub fn resume(location_dir_path: PathBuf) -> Result<Self, MadeleineError> {
    // Read snapshot file if it exists.
    let snapshot_id_path = snapshot_id_file_path(location_dir_path.clone());

    if snapshot_id_path.is_file() {
      let snapshot_file_path_to_read = {
        let raw_snapshot_id = fs::read(&snapshot_id_path)?;
        let snapshot_id: usize = serde_json::from_slice(&raw_snapshot_id)?;
        snapshot_file_path(snapshot_id, location_dir_path.clone())
      };

      let raw_state = fs::read(snapshot_file_path_to_read)?;
      let hydrated_state: SystemState = serde_json::from_slice(&raw_state)?;
      let constructor = || hydrated_state;
      let madeleine = Madeleine::new(location_dir_path, constructor)?;

      Ok(madeleine)
    } else {
      Err(MadeleineError::SnapshotError(String::from(
        "No snapshots found",
      )))
    }

    // If it exists, read the last snapshot and spin up a new Madeleine with that as the initial state.
    // Then we replay all the commands in the log, if any exist.
  }

  /// Execute the command on the business object and update the application state.
  /// Then, log the command.
  pub fn execute_command<'a, C>(&self, command: C) -> Result<Offset, MadeleineError>
  where
    C: Command<'a, SystemState = SystemState> + Serialize + Deserialize<'a>,
  {
    self
      .internal_state
      .replace_with(|old| command.execute(old.to_owned()));

    self.command_log.append_command(command)
  }

  /// Consume the instance and return its internal state.
  pub fn into_inner(self) -> SystemState {
    self.internal_state.into_inner()
  }

  /// Run a closure passed a reference to the instance's internal state.
  pub fn tap<T, O>(&self, func: O) -> T
  where
    O: Fn(SystemState) -> T,
  {
    let val = self.internal_state.borrow();

    func(val.clone())
  }

  /// Gets the length of the command history.
  pub fn len(&self) -> u64 {
    self.command_log.len()
  }

  /// Determine if the instance has an empty command history.
  pub fn is_empty(&self) -> bool {
    self.command_log.len() == 0
  }

  /// Take and persist a snapshot of the internal state.
  pub fn take_snapshot(&self) -> Result<usize, MadeleineError> {
    let state = self.internal_state.try_borrow()?;
    let next_snapshot_id = self.next_snapshot_id()?;
    let location = snapshot_file_path(next_snapshot_id, self.location_dir_path.clone());

    let serialized = serde_json::to_string(&*state)?;
    fs::write(location, serialized)?;

    write_snapshot_id_file(
      self.location_dir_path.join(SNAPSHOT_FILE_SUFFIX),
      next_snapshot_id,
    )?;

    Ok(0)
  }

  /// Determine the next snapshot id in sequence.
  pub fn next_snapshot_id(&self) -> Result<usize, MadeleineError> {
    let snapshot_file_path = snapshot_id_file_path(self.location_dir_path.clone());

    if snapshot_file_path.is_file() {
      let raw = fs::read(snapshot_file_path)?;
      let parsed: usize = serde_json::from_slice(&raw)?;

      Ok(parsed + 1)
    } else {
      Ok(0)
    }
  }
}

fn snapshot_file_path(snapshot_id: usize, location_dir_path: PathBuf) -> PathBuf {
  let snapshot_file_name = format!("{}.{}", snapshot_id, SNAPSHOT_FILE_SUFFIX);
  location_dir_path.join(snapshot_file_name)
}

fn snapshot_id_file_path(location_dir_path: PathBuf) -> PathBuf {
  location_dir_path.join(SNAPSHOT_FILE_SUFFIX)
}

/// Write the snapshot ID file
fn write_snapshot_id_file(
  snapshot_file_id_name: PathBuf,
  current_snapshot_id: usize,
) -> Result<usize, MadeleineError> {
  let raw = serde_json::to_string(&current_snapshot_id)?;
  fs::write(snapshot_file_id_name, raw)?;

  Ok(current_snapshot_id)
}

#[cfg(test)]
mod tests {
  use super::*;

  use assert_fs::prelude::*;
  use predicates::prelude::*;
  use pretty_assertions::assert_eq;

  use std::collections::HashMap;

  #[derive(Debug, Clone, Deserialize, Serialize)]
  enum Action {
    Increment(String, usize),
    Decrement(String, usize),
  }

  impl Command<'_> for Action {
    type SystemState = HashMap<String, usize>;

    fn execute(&self, old_state: Self::SystemState) -> Self::SystemState {
      let mut new_state = old_state.clone();

      match self {
        Self::Increment(key, amount) => new_state
          .entry(key.to_string())
          .and_modify(|e| *e += amount)
          .or_insert(*amount),
        Self::Decrement(key, amount) => new_state
          .entry(key.to_string())
          .and_modify(|e| *e -= amount)
          .or_insert(*amount),
      };

      new_state
    }
  }

  #[track_caller]
  fn make_test_madeleine<T, C>(constructor: C) -> Madeleine<T>
  where
    C: Fn() -> T,
    T: Clone + for<'a> Deserialize<'a> + Serialize,
  {
    let temp_dir = assert_fs::TempDir::new().expect("unable to create temp dir in test");

    let log_path = temp_dir.path().join("test_log");

    Madeleine::new(log_path, constructor).expect("unable to instantiate madeleine in test")
  }

  #[test]
  fn test_new_creates_command_log() {
    let temp_dir = assert_fs::TempDir::new().expect("unable to create temp dir in test");

    let log_path = temp_dir.path().join("test_log");

    let state = 0;

    temp_dir
      .child("test_log")
      .assert(predicate::path::missing());

    Madeleine::new(log_path, &|| state).expect("unable to instantiate madeleine in test");

    temp_dir.child("test_log").assert(predicate::path::exists());
  }

  #[test]
  fn test_into_inner() {
    let state = 42;
    let madeleine = make_test_madeleine(|| state);

    assert_eq!(state, madeleine.into_inner());
  }

  #[test]
  fn test_new_sets_constructor_result_as_state() {
    let madeleine = make_test_madeleine(|| 41 + 1);

    assert_eq!(42, madeleine.into_inner());
  }

  #[test]
  fn test_execute_command_once() {
    let madeleine = make_test_madeleine(|| {
      let state: HashMap<String, usize> = HashMap::new();

      state
    });

    let action = Action::Increment("panda".to_string(), 613);

    madeleine
      .execute_command(action)
      .expect("unable to execute increment action in test");

    let state = madeleine.into_inner();

    let actual = state.get("panda");

    let val = 613 as usize;

    let expected = Some(&val);

    assert_eq!(expected, actual);
  }

  #[test]
  fn test_execute_command_many() {
    let madeleine = make_test_madeleine(|| {
      let state: HashMap<String, usize> = HashMap::new();

      state
    });

    for _i in 0..613 {
      let action = Action::Increment("panda".to_string(), 1);

      madeleine
        .execute_command(action)
        .expect("unable to execute increment action in test");
    }

    let state = madeleine.into_inner();

    let actual = state.get("panda");

    let val = 613 as usize;

    let expected = Some(&val);

    assert_eq!(expected, actual);
  }

  #[test]
  fn test_tap() {
    let madeleine = make_test_madeleine(|| {
      let state: HashMap<String, usize> = HashMap::new();

      state
    });

    let internal_start = madeleine.tap(|state| state.get("panda").map(|v| v.to_owned()));

    assert_eq!(internal_start, None);

    for _i in 0..613 {
      let action = Action::Increment("panda".to_string(), 1);

      madeleine
        .execute_command(action)
        .expect("unable to execute increment action in test");
    }

    let internal_mid = madeleine.tap(|state| state.get("panda").unwrap_or(&0).to_owned());

    assert_eq!(internal_mid, 613);

    for _i in 0..613 {
      let action = Action::Decrement("panda".to_string(), 1);

      madeleine
        .execute_command(action)
        .expect("unable to execute decrement action in test");
    }

    let state = madeleine.into_inner();

    let actual = state.get("panda");

    let val = 0;

    let expected = Some(&val);

    assert_eq!(expected, actual);
  }

  #[test]
  fn test_len_with_empty() {
    let madeleine = make_test_madeleine(|| {
      let state: HashMap<String, usize> = HashMap::new();

      state
    });

    let actual = madeleine.len();

    assert_eq!(actual, 0);
  }

  #[test]
  fn test_len_with_some_commands() {
    let madeleine = make_test_madeleine(|| {
      let state: HashMap<String, usize> = HashMap::new();

      state
    });

    let length_at_start = madeleine.len();

    assert_eq!(length_at_start, 0);

    for _i in 0..613 {
      let action = Action::Increment("panda".to_string(), 1);

      madeleine
        .execute_command(action)
        .expect("unable to execute increment action in test");
    }

    let actual = madeleine.len();

    assert_eq!(actual, 613);
  }

  #[test]
  fn test_next_snapshot_id_first() {
    let madeleine = make_test_madeleine(|| {
      let state: HashMap<String, usize> = HashMap::new();

      state
    });

    let actual = madeleine
      .next_snapshot_id()
      .expect("unable to determine next snapshot id in test");

    assert_eq!(actual, 0);
  }

  #[test]
  fn test_next_snapshot_id_subsequent() {
    let temp_dir = assert_fs::TempDir::new().expect("unable to create temp dir in test");

    let log_path = temp_dir.path().join("test_store");

    let state = 0;

    let madeleine =
      Madeleine::new(log_path, &|| state).expect("unable to instantiate madeleine in test");

    temp_dir
      .child("test_store")
      .child(SNAPSHOT_FILE_SUFFIX)
      .assert(predicate::path::missing());

    let actual_fresh = madeleine
      .next_snapshot_id()
      .expect("unable to determine next snapshot id in test");

    assert_eq!(actual_fresh, 0);

    madeleine
      .take_snapshot()
      .expect("unable to take snapshot in test");

    temp_dir
      .child("test_store")
      .child(SNAPSHOT_FILE_SUFFIX)
      .assert(predicate::path::exists());

    let actual_after_one = madeleine
      .next_snapshot_id()
      .expect("unable to determine next snapshot id in test");

    assert_eq!(actual_after_one, 1);

    madeleine
      .take_snapshot()
      .expect("unable to take snapshot in test");

    let actual_after_two = madeleine
      .next_snapshot_id()
      .expect("unable to determine next snapshot id in test");

    assert_eq!(actual_after_two, 2);
  }

  #[test]
  fn test_basic_resume() {
    let temp_dir = assert_fs::TempDir::new().expect("unable to create temp dir in test");

    let store_path = temp_dir.path().join("test_store");

    let madeleine = Madeleine::new(store_path.clone(), || {
      let state: HashMap<String, usize> = HashMap::new();

      state
    })
    .expect("unable to instantiate madeleine in test");

    for _i in 0..613 {
      let action = Action::Increment("panda".to_string(), 1);

      madeleine
        .execute_command(action)
        .expect("unable to execute increment action in test");
    }

    madeleine
      .take_snapshot()
      .expect("unable to take snapshot in test");

    let expected = madeleine.into_inner();

    let new_madeleine: Madeleine<HashMap<String, usize>> =
      Madeleine::resume(store_path).expect("unable to resume madeleine in test");

    let actual = new_madeleine.into_inner();

    assert_eq!(actual, expected);
  }

  // #[test]
  // fn test_complex_resume() {
  //   let temp_dir = assert_fs::TempDir::new().expect("unable to create temp dir in test");

  //   let store_path = temp_dir.path().join("test_store");

  //   let madeleine = Madeleine::new(store_path.clone(), || {
  //     let state: HashMap<String, usize> = HashMap::new();

  //     state
  //   })
  //   .expect("unable to instantiate madeleine in test");

  //   for _i in 0..613 {
  //     let action = Action::Increment("panda".to_string(), 1);

  //     madeleine
  //       .execute_command(action)
  //       .expect("unable to execute increment action in test");
  //   }

  //   madeleine
  //     .take_snapshot()
  //     .expect("unable to take snapshot in test");

  //   for _i in 0..613 {
  //     let action = Action::Decrement("panda".to_string(), 1);

  //     madeleine
  //       .execute_command(action)
  //       .expect("unable to execute decrement action in test");
  //   }

  //   let expected = madeleine.into_inner();

  //   let new_madeleine: Madeleine<HashMap<String, usize>> =
  //     Madeleine::resume(store_path).expect("unable to resume madeleine in test");

  //   let actual = new_madeleine.into_inner();

  //   assert_eq!(actual, expected);
  // }
}
