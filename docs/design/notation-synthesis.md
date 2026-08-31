# Notation and synthesis

How musical notation becomes audio in Aura. This document governs integration across composition, scheduling, instrumentation, synthesis, and rendering crates.

## Purpose and scope

Aura separates **what** music is (notation), **when** it happens (scheduling), **how** it sounds (instrumentation and synthesis), and **where** it goes (I/O). Surface programs load Imp graphs via [`aura-integration`](../../crates/aura-integration) and [`aura-cli`](../../crates/aura-cli).

See also [`architecture.md`](architecture.md) and [`imp-execution.md`](imp-execution.md).

## Glossary

| Term | Meaning |
|------|---------|
| **Notation** | Types describing musical material: semitones, time, events, scores, chords |
| **Semitone** | Discrete key pitch on the 12-TET chromatic scale; not a frequency |
| **Chord** | Root, voiced tones, and optional bass — built from intervals and scale degrees |
| **ChordProgression** | Timed sequence of chords on a beat grid; a **data signal** sampled per frame via `ChordSignal` (with wired tempo) |
| **Loopable** | Type constraint (`Loopable`) satisfied by `score` and `chord_progression`; required type argument for the generic `loop` node |
| **loop** | Generic Imp node `loop<T: Loopable>` — marks a score or progression as unboundedly looped (modulus over cycle length at playback) |
| **Tempo** | Beats per minute; a **data signal** (`tempo` port type) composed at translate time |
| **TimeSignature** | Beats per bar and beat unit; a **data signal** (`time_signature` port type) composed at translate time |
| **Data signal** | Structured translate-time or sample-time value (e.g. `Score`, `ChordProgression`, `Tempo`, `TimeSignature`), distinct from scalar `Sampler` outputs |
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
4. **Notation path** — `arpeggio`, `drum_grid`, `epic_minor_progression`, `loop`, `constant_tempo`, `constant_time_signature`, and related nodes build scores, progressions, tempo, and meter; generators emit **one cycle**; `loop` compositors repeat via modulus for the render duration.
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

Every aspect of music production should be expressible as **modular, reusable patterns** organized into libraries. Patterns support nested abstraction and composition. Generators (e.g. `arpeggio`, `drum_grid`) emit one finite cycle; the generic `loop<T: Loopable>` node applies unbounded repetition by modulus. Render duration is external (CLI `--duration`).

### Rule 6: Holistic song functions (target)

Long term, an entire song should be describable as a **single function**, with granular aspects integrated via reducers, blurring the line between notation and audio. Initial iterations need not fully realize this paradigm.

### Rule 7: Chord input default

By default, instruments resolve harmony **once at note-on** (`start_beats` / `start_frame`). They may optionally sample `ChordSignal` per frame (e.g. pitch glide across chord boundaries). The arpeggio demo uses the note-on default: semitone is baked into each `NoteEvent` at translate time.

## Drum lanes in Imp graphs

Percussion in the arpeggio demo uses **separate scores per lane** (kick and snare each get their own `drum_grid` → `note_at_time` chain) because different hits need different synthesis subgraphs. Lanes are mixed at sample time with an `add` node.

- **Kick** — `exponential_sweep_sine` (closed-form pitch drop) × `linear_adsr` amplitude envelope.
- **Snare** — deterministic noise through a **stateless high-pass** (`noise(t) - noise(t - dt)`), not an IIR filter, to stay compatible with seekable pure `f(t)` sampling ([ADR 0007](../decisions/0007-drop-fundsp.md)).

Drum hits use `NoteEvent` timing only; semitone is unused on drum lanes.

## Ontology

Aura models music in its own terms: semitones, scale degrees, intervals, chords, and keys. Frequency (`Hz`) is derived at synthesis boundaries via `Semitone::to_hz()`. MIDI compatibility, if ever needed, is an adapter concern — not the source of truth for notation types.

## Crate responsibility matrix

| Crate | Role |
|-------|------|
| [`aura-composition`](../../crates/aura-composition) | Notation types, beat↔second conversion |
| [`aura-composer`](../../crates/aura-composer) | Procedural generators (e.g. arpeggios, drum grids) |
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
