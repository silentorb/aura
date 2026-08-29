# 0003. File-first I/O

Date: 2026-08-29

## Context

The initial development environment is WSL/devcontainer, where real-time audio output is unreliable. The first deliverable is a program that generates audio and writes it to disk.

## Decision

Implement offline WAV export first via `aura-io-wav` (32-bit float). Add `aura-io-cpal` as a compile-time stub with sample conversion tests but no active playback. Default CLI output goes to the gitignored `output/` directory.

## Consequences

- Development and CI can validate audio generation without audio hardware.
- Real-time playback is deferred; `aura-io-cpal` must be completed before live output.
- FunDSP native WAV I/O (`Wave::save_wav32`) is sufficient for now; `hound` can be added later if CPAL interop requires it.
