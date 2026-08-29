# Product

Product definition for Aura — procedural music composition and synthesis software.

## Vision

TBD

## Goals

TBD

## Non-goals

TBD

## Core domains

| Domain | Description |
|--------|-------------|
| Composition | TBD — procedural generation and structuring of musical material |
| Synthesis | FunDSP-based audio graph construction (`aura-dsp`) |
| Sequencing | TBD — temporal organization and playback of musical events |
| Rendering | Offline WAV export to disk; real-time CPAL playback deferred |

### Rendering (initial scope)

- Generate audio offline via FunDSP `Wave::render`.
- Write 32-bit float WAV files to an untracked `output/` directory.
- Real-time playback via CPAL is planned but deferred until the dev environment supports audio output reliably.

## Open questions

TBD
