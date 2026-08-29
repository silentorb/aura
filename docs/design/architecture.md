# Architecture

High-level system overview for Aura. Components and boundaries are placeholders until defined.

## High-level components

| Component | Role |
|-----------|------|
| Composer | TBD — procedural generation and structuring of musical material |
| Synthesizer | TBD — sound generation and timbral control |
| Scheduler | TBD — temporal organization and event dispatch |
| I/O | TBD — audio output and external interfaces |

## Module boundaries

TBD — to be defined as the crate structure emerges.

## Considerations

Factors to weigh during design. Not decisions until recorded as ADRs.

- **Real-time audio** — low-latency output may constrain architecture
- **Determinism** — procedural composition may require reproducible results from a given seed
- **Modularity** — composition, synthesis, and sequencing may be separable concerns
- **Performance** — synthesis and rendering are likely CPU-intensive

## Open questions

TBD
