use madeleine::{Command, Madeleine, MadeleineError};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

// Define a command called `Action` which has two variants corresponding to each of its operations.
#[derive(Debug, Clone, Deserialize, Serialize)]
enum Action {
  Increment(String, usize),
  Decrement(String, usize),
}

// Implement the `Command` trait for `Action`.
impl Command<'_> for Action {
  // The type of the system's internal state is a map from `String` to `usize`.
  type SystemState = HashMap<String, usize>;

  // The actual logic lives in this `execute` method.
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

pub fn main() -> Result<(), MadeleineError> {
  // Initialize the system.
  let madeleine = Madeleine::new("hash_map_example".into(), &|| {
    let state: HashMap<String, usize> = HashMap::new();

    state
  })?;

  println!("Instantiated Madeleine");

  let internal_start = madeleine.tap(|state| state.get("panda").map(|v| v.to_owned()));

  println!("Current value of 'panda': {:?}", internal_start);

  for i in 1..1024 {
    let action = Action::Increment("panda".to_string(), i);
    madeleine.execute_command(action)?;
  }

  println!("Finished increment run.");

  let internal_mid = madeleine.tap(|state| state.get("panda").unwrap_or(&0).to_owned());

  println!("Current value of 'panda': {}", internal_mid);

  for i in 1..1024 {
    let action = Action::Decrement("panda".to_string(), i);
    madeleine.execute_command(action)?;
  }

  println!("Finished decrement run.");

  // Tear down the system by extracting its internal state.
  let state = madeleine.into_inner();

  println!("Internal State: {:?}", state);

  Ok(())
}
