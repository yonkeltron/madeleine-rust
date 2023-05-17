use serde::{Deserialize, Serialize};

/// This trait must be implemented by every command.
/// Specifically, every command (and its state) must be serializable and deserializable by serde.
/// A command's state must also be `Clone`.
pub trait Command<'a>: Serialize + Deserialize<'a> {
  /// The associated type which a Command must return. This must correspond to the type of the `Madeleine` instance's internal state.
  type SystemState: Serialize + Deserialize<'a> + Clone;

  /// Core logic for a Command, left to the implementor to specify.
  fn execute(&self, old_state: Self::SystemState) -> Self::SystemState;
}
