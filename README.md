# Hi, I'm Madeleine

**WARNING**: I am very early alpha and not even feature-complete yet!
Do not use me, but feel free to chip in to lend a hand.

I'm a [Rust](https://www.rust-lang.org/) library for building stateful applications with persistent state containers.
My inspiration comes from Ruby's [`madeleine` gem](https://github.com/ghostganz/madeleine) and, transitively, the earlier Java [Prevalayer library](https://prevayler.org/).

## Overview

Using Madeleine, you model applications purely as predefined operations on a protected data structure.
These operations are called _commands_ and which may mutate the state of a protected data structure called a _system_.
Using this model, every command which gets executed is transparently serialized and added to an append-only log in which their order is retained.
Since the commands are the _only_ things capable of altering the system, this constitutes a complete history of how the application's current state came to be.
As every command log is persisted to disk, an application can exit or crash with the understanding that it may replay the command log on resumption without apparent effect.
In other words, it could conceivably pick back up right where it left off.

To see it in action, check out the `examples` directory for sample code.

## Feature Roadmap

- [x] Main, top-level `Madeleine` interface
- [x] Example code
- [x] Snapshot logic
- [x] Command logging
- [ ] Persistence
  - [x] Commands
  - [x] Snapshots
- [ ] Resumption/rehydration from
  - [ ] Command log
  - [x] Snapshots
  - [ ] Mixed scenarios involving commands following a snapshot
- [ ] Garbage collection
- [x] Benchmarks
- [x] Integration tests

### Wish list

- [ ] Concurrency/parallelism support
- [ ] Async interfaces
- [ ] Background processing
- [ ] Storage compression
- [ ] Additional and pluggable storage formats, possibly through [features](https://doc.rust-lang.org/cargo/reference/features.html):
  - [x] JSON
  - [ ] TOML
  - [ ] CBOR
