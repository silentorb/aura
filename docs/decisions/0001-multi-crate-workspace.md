# 0001. Multi-crate workspace

Date: 2026-08-29

## Context

Aura will grow to encompass composition, synthesis, scheduling, and I/O. A single crate with modules would create tight coupling and unclear dependency boundaries as the codebase expands.

## Decision

Organize the Rust source as a Cargo workspace of many small crates under `crates/`, with dependency direction flowing inward: applications → I/O → render → dsp → sample.

## Consequences

- Crate boundaries enforce discipline; adding a dependency requires an explicit `Cargo.toml` entry.
- Build times may increase slightly due to crate overhead.
- Cross-crate APIs must be designed deliberately; internal details stay private per crate.
- New features land in the smallest crate that owns the concern.
