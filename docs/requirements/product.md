# Product

Product definition for Aura — procedural music composition and synthesis software.

## Vision

Aura is a library-first framework for procedural music. Specialized programs compose Aura crates to produce audio; notation and synthesis integrate through explicit, documented pipelines rather than hidden coupling.

## Goals

- Express music as procedural, reusable patterns rather than destructive stored sequences
- Model synthesis as pure **Time → Sample** functions via Imp graphs
- Support deterministic offline rendering with optional future-aware scheduling
- Keep integration at the surface; core crates stay focused

See [`notation-synthesis.md`](../design/notation-synthesis.md) and [`imp-execution.md`](../design/imp-execution.md).

## Non-goals

- Live CPAL playback until the dev environment supports it reliably
- Stock `imp_execution` or FunDSP for Aura's primary sampler path

## Core domains

| Domain | Description |
|--------|-------------|
| Composition | Notation types (`aura-composition`) and generators (`aura-composer`) |
| Sequencing | Offline scheduling (`aura-scheduler`); live dispatch deferred |
| Instrumentation | Per-note sampling and mix-down (`aura-instrumentation`) |
| Synthesis | Imp graphs (`aura-imp`) and pure DSP (`aura-dsp`) |
| Integration | Graph translation and sampling (`aura-integration`) |
| Rendering | Offline WAV export via thin CLI and demo scripts |

### Rendering (initial scope)

- Load Imp JSON graphs; translate to `Sampler`; sample over a specified duration.
- Write 32-bit float WAV files to untracked `output/` via `aura-cli` or [`demos/render-all.sh`](../../demos/render-all.sh).
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
- Graph-owned duration vs external render parameters
- Buffer sampling optimizations preserving FP semantics
