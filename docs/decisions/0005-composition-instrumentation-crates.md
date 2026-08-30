# 0005. Composition, scheduler, and instrumentation crates

Date: 2026-08-30

## Context

Aura needs notation types, procedural generators, temporal scheduling, and per-note instrument sampling before the full notation→audio pipeline can be exercised. A standalone integration doc ([`notation-synthesis.md`](../design/notation-synthesis.md)) defines how notation and synthesis work together.

## Decision

1. Add domain crates:
   - **`aura-composition`** — notation types and musical-time utilities
   - **`aura-composer`** — procedural generators
   - **`aura-scheduler`** — offline beat→frame scheduling and `ScheduleContext` (lookahead)
   - **`aura-instrumentation`** — `Instrument` trait, per-note sampling, mix-down

2. Surface integration via **`aura-integration`** (library) and a thin graph-driven **`aura-cli`**. Demo graphs live under [`demos/`](../../demos/).

3. Push orchestration to the surface. Core crates do not hide wiring behind transitive dependencies.

4. Imp graphs express **`Time → Sample`** functions; music nodes delegate to composition crates at translate time.

## Consequences

- Clear separation: what (composition) → when (scheduler) → how (instrumentation/synthesis) → wiring (integration / CLI).
- Smoke test: `./demos/render-all.sh` or `cargo run -p aura-cli -- --graph ... --output ...`.
- Live CPAL scheduling remains a follow-up.
