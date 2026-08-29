# 0002. Audio stack: DASP, FunDSP, CPAL

Date: 2026-08-29

## Context

Aura requires both real-time audio playback and non-real-time processing with disk serialization. The stack must support code reuse between both paths and integrate with the Rust audio ecosystem.

## Decision

Adopt three libraries with distinct roles:

| Library | Role |
|---------|------|
| **DASP** (`dasp_sample`, `dasp_signal`) | Sample type traits and signal abstractions; bridges FunDSP `f32` output to CPAL sample formats |
| **FunDSP** | Audio synthesis and DSP graph construction; offline rendering via `Wave::render`, real-time via `AudioUnit::process` |
| **CPAL** | Cross-platform real-time audio I/O |

FunDSP `prelude32` imports are confined to `aura-dsp` so other crates depend on Aura abstractions, not FunDSP internals.

## Consequences

- Synthesis graphs are portable between offline and real-time paths.
- FunDSP version upgrades require changes primarily in `aura-dsp` and `aura-render`.
- CPAL pulls platform audio dependencies (e.g. ALSA on Linux); the devcontainer must provide them.
