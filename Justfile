default:
  @just --choose --chooser=sk

test:
  cargo test

lint:
  cargo clippy

demo:
  cargo run --example hash_map
  rm -rf hash_map_example

bench:
  #!/usr/bin/env bash
  
  set -euo pipefail
  
  set -x

  cargo bench
  rm -rf naive_*_benchmark

ci: test lint bench