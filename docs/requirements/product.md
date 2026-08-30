# Product

Product definition for Aura — procedural music composition and synthesis software.

## Vision

Aura is a library-first framework for procedural music. Specialized programs compose Aura crates to produce audio; notation and synthesis integrate through explicit, documented pipelines rather than hidden coupling.

## Goals

- Express music as procedural, reusable patterns rather than destructive stored sequences
- Sample each note as an independent instrument instance by default
- Support deterministic offline rendering with optional future-aware scheduling
- Keep integration at the surface; core crates stay focused

See [`notation-synthesis.md`](../design/notation-synthesis.md) for integration rules.

## Non-goals

- General-purpose CLI tool in v1 (`aura-cli` deferred)
- Live CPAL playback until the dev environment supports it reliably
- Full Imp-graph-per-note instruments in v1

## Core domains

| Domain | Description |
|--------|-------------|
| Composition | Notation types (`aura-composition`) and generators (`aura-composer`) |
| Sequencing | Offline scheduling (`aura-scheduler`); live dispatch deferred |
| Instrumentation | Per-note sampling and mix-down (`aura-instrumentation`) |
| Synthesis | FunDSP-based audio graph construction (`aura-dsp`, `aura-imp`) |
| Rendering | Offline WAV export; whole-graph and per-note paths |

### Rendering (initial scope)

- Generate audio offline via FunDSP or per-note PCM mix-down.
- Write 32-bit float WAV files to an untracked `output/` directory via `aura-demo`.
- Real-time playback via CPAL is planned but deferred.

### Integration requirements

1. Per-note instrument instances with per-note parameters and envelopes
2. Future-aware sampling during offline render via `ScheduleContext`
3. Deterministic, seedable offline output
4. Functional notation generation over stored sequences
5. Modular, nestable, library-organized patterns
6. Long-term: holistic song-as-function via reducers (target, not v1)

## Open questions

- Live vs offline parity when lookahead is unavailable
- Imp vs direct FunDSP for simple instruments
