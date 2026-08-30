# 0004. Imp integration for declarative DSP graphs

Date: 2026-08-29

## Context

Aura needs a declarative way to define synthesis graphs. [Imp](https://github.com/silentorb/imp-spec) provides a portable DAG model (`Graph`, node libraries, registries). FunDSP remains the audio execution engine (see [0002-audio-stack.md](./0002-audio-stack.md)).

`imp_execution` evaluates collection/path graphs into tabular results — the wrong execution model for continuous audio buffers.

## Decision

1. Add **`aura-imp`** — Aura-specific Imp node libraries and registry helpers; depends on `imp_core_types` and `imp_registry` only (no FunDSP).
2. Extend **`aura-dsp`** with `compile_graph` — lowers Imp `Graph` + `Registry` to `Box<dyn AudioUnit>` at compile time.
3. Provision **imp-rust** in the devcontainer image at build time (`COPY imp-rust /opt/imp-rust`); Cargo path deps point at `/opt/imp-rust/crates/*`.
4. Defer **`imp_execution`** for DSP; buffer-oriented Imp execution is a future imp-rust concern.

Initial DSP library: `sine_hz` node with `control` frequency input and `audio_mono` output.

## Consequences

- Waveforms can be defined as Imp graphs and rendered through the existing offline pipeline.
- FunDSP imports remain confined to `aura-dsp` and downstream render/I/O crates.
- imp-rust changes require rebuilding the devcontainer image.
- Composition and non-DSP Imp uses can extend `aura-imp` with additional node libraries later.
