# Engineering Conventions for BPlusTree3

- No feature flags for internal experiments. We have no external users, so avoid `#[cfg(feature = ...)]` branches. Implement improvements directly (or in short‑lived local branches) and remove experimental code before merging.

- Performance work
  - Validate with existing Criterion benches and the large delete runner (`rust/src/bin/large_delete_benchmark.rs`).
  - For line‑level CPU hotspots, use the Instruments workload (`rust/src/bin/instruments_delete_target.rs`) and store traces under `rust/delete_profile.trace` (not committed).
  - Prefer targeted, localized changes that don’t regress insert/get/range performance.

- Coding style
  - Keep changes minimal and focused on the stated goal.
  - Reduce repeated arena lookups and redundant separator/key reads in hot paths.
  - Favor bulk moves and pre‑allocation over per‑element operations.

- Benchmarks to run for delete changes
  - `cd rust && cargo bench --bench comparison deletion`
  - `cd rust && cargo run --release --bin large_delete_benchmark`
  - Optional: record Instruments trace for confirmation of hotspot reductions.

- Hygiene before commit
  - Always remove dead code introduced by refactors.
  - Delete code as soon as it is dead.
  - Always format the workspace: `cd rust && cargo fmt --all`.
  - Always run all tests: `cargo test --workspace` (and benches if relevant).
