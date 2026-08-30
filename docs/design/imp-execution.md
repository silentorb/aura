# Imp graph execution model

How Aura evaluates Imp graphs for audio synthesis.

See [ADR 0006](../decisions/0006-imp-time-to-sample.md) and [ADR 0007](../decisions/0007-drop-fundsp.md).

## Time → Sample semantics

An Aura Imp graph denotes a pure function:

```text
f : Time → Sample
```

- **Time** — seconds from render start (`f64`).
- **Sample** — mono amplitude (`f32`).
- The **`time`** signal is wired explicitly in graphs (see demo JSON under [`demos/`](../../demos/)).

## Three layers

| Layer | Role |
|-------|------|
| Imp JSON graph | Syntax — nodes and edges describing function composition |
| `translate_graph` | Compiler — builds `Box<dyn Sampler>` once |
| `sample_graph` | Applicator — calls `f(t)` per frame over a duration |

This is **not** stock `imp_execution` (tabular row evaluation) and **not** FunDSP sequential `tick` state.

## Functional properties (v1)

- **Referential transparency** — `at(t)` depends only on `t` and graph parameters.
- **Seekability** — `at(t)` is valid without evaluating prior samples.
- **Composition** — node wiring corresponds to composing child samplers.

Translation produces nested closures (or equivalent) built once, then invoked per frame.

## Translate-time vs sample-time

| Kind | When | Examples |
|------|------|----------|
| Translate-time | During `translate_graph`; captured in closures | `minor_arpeggio` → `Score`; schedule tables |
| Sample-time | Inside `at(t)` | `sine(f, t)`, envelopes, note gates |

## Duration

Render duration is passed alongside the graph (`SampleSpec` / CLI `--duration`). Supported units: seconds (`10s`) and measures (`2m` with `--tempo`). Graph-owned duration is deferred.

## Deferred

- Higher-order graphs (graph produces graph)
- Buffer/block batching without changing `f(t)` semantics
- Stock `imp_execution` for audio
- Real-time CPAL sampler path
