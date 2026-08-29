# Aura — Agent Instructions

## Project

Aura is procedural music composition and synthesis software, written primarily in Rust.

## Source of truth

[`docs/README.md`](docs/README.md) is the entry point for all project documentation. Requirements and design docs under `docs/` define what the software should be; code must conform to them. When docs and code disagree, update code or explicitly revise the doc — never silently drift.

## Persistent requirements

Treat user instructions that define product behavior, architecture, or conventions as durable requirements. Capture them in the appropriate doc under `docs/` before or alongside code changes. One-off task instructions are execution context only.

## Commands

```bash
cargo build
cargo test
cargo clippy
```

These apply once a Rust crate scaffold exists.

## Workspace

This workspace may mount multiple repositories. Each repo may have its own `AGENTS.md` and `docs/`. Prefer the nearest `AGENTS.md` and that repo's `docs/` when working in a given tree. This file governs Aura only.

## Where to look

- [`docs/requirements/`](docs/requirements/) — product definition and requirements
- [`docs/design/`](docs/design/) — architecture and system design
- [`docs/decisions/`](docs/decisions/) — architecture decision records (ADRs)
