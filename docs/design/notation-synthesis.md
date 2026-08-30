# Notation and synthesis

How musical notation becomes audio in Aura. This document governs integration across composition, scheduling, instrumentation, synthesis, and rendering crates.

## Purpose and scope

Aura separates **what** music is (notation), **when** it happens (scheduling), **how** it sounds (instrumentation and synthesis), and **where** it goes (I/O). Surface programs load Imp graphs via [`aura-integration`](../../crates/aura-integration) and [`aura-cli`](../../crates/aura-cli).

See also [`architecture.md`](architecture.md) and [`imp-execution.md`](imp-execution.md).

## Glossary

| Term | Meaning |
|------|---------|
| **Notation** | Types describing musical material: pitch, time, events, scores |
| **Composition** | Procedural generators that produce notation |
| **Scheduling** | Converting musical time to sample-frame timelines |
| **Instrumentation** | Per-note instrument instances, envelopes, and mix-down |
| **Synthesis** | Pure Time → Sample DSP and Imp graph translation |
| **Rendering** | Sampling `f(t)` over a duration; WAV export |

## Pipeline

```mermaid
flowchart LR
  graph[Imp graph JSON] --> integration[aura-integration]
  integration --> composition[aura-composition]
  integration --> scheduler[aura-scheduler]
  integration --> dsp[aura-dsp]
  integration --> wav[aura-io-wav]
```

1. **Imp graph** — denotes `Time → Sample`; music nodes delegate to composition crates at translate time.
2. **Translation** — `translate_graph` composes pure `Sampler` closures.
3. **Sampling** — `sample_graph` applies `f(t)` per frame over the render duration.
4. **Notation path** — `minor_arpeggio` and related nodes build scores; schedule tables drive note gates at sample time.
5. **Output** — CLI or demo scripts write WAV files.

## Integration rules

### Rule 1: Per-note instances

By default, each note is sampled as a **distinct instrument instance**, independent of other notes. Instrument parameter changes are **per note**, not global. Like MIDI velocity, each note may carry custom parameters and its own envelope.

### Rule 2: Future-aware sampling

When possible (primarily during non-real-time rendering), samplers receive the full scheduled timeline via `ScheduleContext`, allowing instruments to react to **future** notes—not only the current event. Live MIDI recording may not provide full lookahead; live and offline output may diverge in those cases.

### Rule 3: Deterministic offline rendering

Non-real-time rendering must be **100% deterministic and reproducible**. Randomized samples use seeds combined with time-from-start. The v1 sine instrument path is deterministic by construction.

### Rule 4: Functional notation

Aura favors **functional notation generation** over a fixed, destructive sequence of stored notes. DAWs often apply loops and filters on top of stored MIDI; Aura inverts that—dynamic, abstract generators form the foundation, with optional embedded snippets of concrete notation.

### Rule 5: Modular nested patterns

Every aspect of music production should be expressible as **modular, reusable patterns** organized into libraries. Patterns support nested abstraction and composition.

### Rule 6: Holistic song functions (target)

Long term, an entire song should be describable as a **single function**, with granular aspects integrated via reducers, blurring the line between notation and audio. Initial iterations need not fully realize this paradigm.

## Crate responsibility matrix

| Crate | Role |
|-------|------|
| [`aura-composition`](../../crates/aura-composition) | Notation types, beat↔second conversion |
| [`aura-composer`](../../crates/aura-composer) | Procedural generators (e.g. arpeggios) |
| [`aura-scheduler`](../../crates/aura-scheduler) | Offline timeline, frame offsets, `ScheduleContext` |
| [`aura-instrumentation`](../../crates/aura-instrumentation) | `Instrument` trait, per-note render, mix-down |
| [`aura-imp`](../../crates/aura-imp) | Imp node libraries and JSON helpers |
| [`aura-dsp`](../../crates/aura-dsp) | Pure Time → Sample DSP functions |
| [`aura-render`](../../crates/aura-render) | Offline sampling via `Sampler` |
| [`aura-sample`](../../crates/aura-sample) | Sample rate, seconds→frames |
| [`aura-integration`](../../crates/aura-integration) | Graph translation and render glue |
| [`aura-cli`](../../crates/aura-cli) | Thin graph-driven CLI |

Dependency direction flows inward. **Integration belongs at the surface**—`aura-integration`, CLI, and demo scripts declare every crate they wire together directly.

## Integration at the surface

An indirect dependency is a hidden direct dependency. Core library crates expose focused APIs and must not bury orchestration. `aura-integration` and thin surface programs own the wiring: load graph → translate → sample → write WAV.

## Open questions

- Live CPAL scheduler parity with offline lookahead semantics
- Graph-owned duration vs external render parameters
- Buffer sampling optimizations preserving FP semantics
