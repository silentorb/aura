# 0002. Audio stack: DASP, CPAL (FunDSP superseded)

Date: 2026-08-29

Status: **FunDSP role superseded** by [0007](./0007-drop-fundsp.md).

## Context

Aura requires both real-time audio playback and non-real-time processing with disk serialization. The stack must support code reuse between both paths and integrate with the Rust audio ecosystem.

## Decision

Adopt libraries with distinct roles:

| Library | Role |
|---------|------|
| **DASP** (`dasp_sample`, `dasp_signal`) | Sample type traits and signal abstractions; bridges PCM to CPAL sample formats |
| **CPAL** | Cross-platform real-time audio I/O (deferred in devcontainer) |
| **`aura-dsp`** | Pure `Time → Sample` functions (replaces FunDSP) |

Historical note: FunDSP was initially adopted for synthesis; removed because sequential `tick` state conflicts with Aura's FP sampling model.

## Consequences

- Synthesis is pure `Sampler::at(t)`; offline and future real-time paths share the same functions.
- CPAL pulls platform audio dependencies (e.g. ALSA on Linux); the devcontainer must provide them.
