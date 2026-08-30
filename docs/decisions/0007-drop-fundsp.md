# 0007. Drop FunDSP; pure aura-dsp Time → Sample functions

Date: 2026-08-30

## Context

FunDSP's `AudioUnit::tick` model is sequentially stateful (phase accumulators, filter history). Offline output is deterministic but not a pure `f(t)` — sampling at arbitrary time requires simulating prior samples. Aura's Imp integration targets explicit time and seekable FP sampling ([0006](./0006-imp-time-to-sample.md)).

## Decision

1. **Remove FunDSP** from the workspace dependency graph.
2. **`aura-dsp`** provides pure functions such as `sine(frequency, time)`.
3. **`aura-render`** samples `Sampler` implementations over a render interval.
4. **`aura-io-wav`** writes mono `f32` PCM directly (no FunDSP `Wave`).
5. Supersede the FunDSP role described in [0002](./0002-audio-stack.md); DASP and CPAL remain.

## Consequences

- No hidden DSP state between `Sampler::at(t)` calls.
- Real-time CPAL path must eventually sample the same pure functions (deferred).
- Prior `compile_graph` → FunDSP lowering is removed; Imp graphs translate to Aura samplers instead.
