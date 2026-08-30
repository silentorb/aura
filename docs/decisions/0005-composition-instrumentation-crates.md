# 0005. Composition, scheduler, and instrumentation crates

Date: 2026-08-30

## Context

Aura needs notation types, procedural generators, temporal scheduling, and per-note instrument sampling before the full notation→audio pipeline can be exercised. The architecture doc named Composer and Scheduler components but left crate boundaries unset. A standalone integration doc ([`notation-synthesis.md`](../design/notation-synthesis.md)) defines how notation and synthesis work together.

## Decision

1. Add domain crates:
   - **`aura-composition`** — notation types and musical-time utilities
   - **`aura-composer`** — procedural generators
   - **`aura-scheduler`** — offline beat→frame scheduling and `ScheduleContext` (lookahead)
   - **`aura-instrumentation`** — `Instrument` trait, per-note sampling, mix-down

2. Add **`aura-demo`** as the surface integration reference program. It declares all wired dependencies explicitly and writes reference WAV output. **`aura-cli`** remains unchanged and deferred.

3. Push orchestration to the surface. Core crates do not hide wiring behind transitive dependencies. An indirect dependency is treated as a hidden direct dependency.

4. v1 instruments use direct FunDSP primitives (`aura-dsp::sine_hz`). Imp graph per note is deferred.

## Consequences

- Clear separation: what (composition) → when (scheduler) → how (instrumentation) → wiring (demo).
- New features land in the smallest owning crate.
- `aura-demo` is the smoke-test entry point (`cargo run -p aura-demo`), not `aura-cli`.
- Live CPAL scheduling remains a follow-up within `aura-scheduler` or a future extension.
