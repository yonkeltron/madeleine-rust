use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use madeleine::{Command, Madeleine};

#[derive(Debug, Clone, Deserialize, Serialize)]
enum Action {
  Increment(String, isize),
  Decrement(String, isize),
}

impl Command<'_> for Action {
  type SystemState = HashMap<String, isize>;

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

pub fn increment_benchmark(c: &mut Criterion) {
  let madeleine = Madeleine::new("naive_increment_benchmark".into(), &|| {
    let state: HashMap<String, isize> = HashMap::new();

    state
  })
  .expect("unable to instantiate madeleine in benchmark");

  c.bench_function("increment", |b| {
    b.iter(|| {
      let action = Action::Increment("panda".to_string(), black_box(20));
      madeleine
        .execute_command(action)
        .expect("unable to append command in benchmark")
    })
  });
}

pub fn decrement_benchmark(c: &mut Criterion) {
  let madeleine = Madeleine::new("naive_decrement_benchmark".into(), &|| {
    let state: HashMap<String, isize> = HashMap::new();

    state
  })
  .expect("unable to instantiate madeleine in benchmark");

  c.bench_function("decrement", |b| {
    b.iter(|| {
      let action = Action::Decrement("panda".to_string(), black_box(20));
      madeleine
        .execute_command(action)
        .expect("unable to append command in benchmark")
    })
  });
}

pub fn updown_benchmark(c: &mut Criterion) {
  let madeleine = Madeleine::new("naive_updown_benchmark".into(), &|| {
    let state: HashMap<String, isize> = HashMap::new();

    state
  })
  .expect("unable to instantiate madeleine in benchmark");

  c.bench_function("updown", |b| {
    b.iter(|| {
      for i in 1..1024 {
        let action = Action::Increment("panda".to_string(), black_box(i));
        madeleine
          .execute_command(action)
          .expect("unable to append command in benchmark");
      }

      for i in 1..1024 {
        let action = Action::Decrement("panda".to_string(), black_box(i));
        madeleine
          .execute_command(action)
          .expect("unable to append command in benchmark");
      }

      madeleine.tap(|state| state.get("panda").unwrap_or(&0).to_owned() + black_box(1))
    })
  });
}

pub fn tap_benchmark(c: &mut Criterion) {
  let madeleine = Madeleine::new("naive_tap_benchmark".into(), &|| {
    let state: HashMap<String, isize> = HashMap::new();

    state
  })
  .expect("unable to instantiate madeleine in benchmark");

  c.bench_function("updown", |b| {
    b.iter(|| madeleine.tap(|state| state.get("panda").unwrap_or(&0).to_owned() + black_box(1)))
  });
}

criterion_group!(
  benches,
  increment_benchmark,
  decrement_benchmark,
  updown_benchmark,
  tap_benchmark
);
criterion_main!(benches);
