# 0006. Imp graphs as Time → Sample functions

Date: 2026-08-30

## Context

Aura needs Imp graphs that express synthesis as pure functions of time. Stock `imp_execution` evaluates collection/path graphs to tabular rows — the wrong model for continuous audio. FunDSP uses sequential `tick` state, which conflicts with seekable, referentially transparent sampling (see [0007](./0007-drop-fundsp.md)).

## Decision

1. **Graph semantics** — an Aura Imp graph denotes `Time → Sample` (seconds → mono amplitude). The `time` signal is explicit in graphs.
2. **Translation** — `translate_graph` walks the DAG once and composes nested closures (`Sampler::at(t)`). Imp nodes are not interpreted per sample.
3. **Sampling** — `sample_graph` applies `f(t)` for each frame over a duration passed alongside the graph (CLI / `SampleSpec`).
4. **Custom Aura sampler** — implemented in `aura-integration`; not stock `imp_execution`.
5. **Node libraries** — `aura.time`, `aura.dsp`, `aura.envelope`, `aura.music` in `aura-imp`; translate hooks compose pure functions using `aura-dsp` and composition crates.

## Consequences

- Graph JSON is syntax; runtime is a composed pure function.
- Seekability and purity are testable (`at(t)` twice is identical; sparse `t` matches sequential pass).
- Duration is external to the graph for v1; graph-owned duration deferred.
- Higher-order graphs and buffer batching deferred.
