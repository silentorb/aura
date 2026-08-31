# 0004. Imp integration for declarative DSP graphs

Date: 2026-08-29

Status: **Partially superseded** by [0006](./0006-imp-time-to-sample.md) and [0007](./0007-drop-fundsp.md).

## Context

Aura needs a declarative way to define synthesis graphs. [Imp](https://github.com/silentorb/imp-spec) provides a portable DAG model (`Graph`, node libraries, registries).

`imp_execution` evaluates collection/path graphs into tabular results — the wrong execution model for continuous audio buffers.

## Decision (historical)

1. Add **`aura-imp`** — Aura-specific Imp node libraries and registry helpers; depends on `imp_core_types` and `imp_registry` only.
2. ~~Provision **imp-rust** in the devcontainer image at build time (`COPY imp-rust /opt/imp-rust`); Cargo path deps point at `/opt/imp-rust/crates/*`.~~ **Superseded:** bind-mount sibling `imp-rust` at `.mnt/imp-rust`; Cargo path deps point at `.mnt/imp-rust/crates/*`.
3. Defer **`imp_execution`** for DSP.

## Current direction (see ADR 0006)

- Imp graphs denote **`Time → Sample`** pure functions.
- **`aura-integration::translate_graph`** composes `Sampler` closures; not FunDSP, not stock `imp_execution`.
- Node libraries: `aura.time`, `aura.dsp`, `aura.envelope`, `aura.music`.

## Consequences

- imp-rust is live-mounted from the host (`~/dev/imp-rust` → `.mnt/imp-rust`); changes are picked up without rebuilding the devcontainer image.
- Composition and music nodes extend `aura-imp`; translation delegates to domain crates.
