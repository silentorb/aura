# Architecture Decision Records

Dated, append-only log of significant architectural decisions.

## Format

Each ADR is a numbered markdown file: `NNNN-short-title.md`.

```markdown
# NNNN. Title

Date: YYYY-MM-DD

## Context

What is the issue or forcing function?

## Decision

What was decided?

## Consequences

What becomes easier or harder as a result?
```

## Rules

- Do not edit or delete existing ADRs. Supersede with a new ADR if a decision changes.
- Keep decisions focused — one concern per record.
- Link related ADRs and design docs where helpful.

## Decision log

| ADR | Title | Date | Status |
|-----|-------|------|--------|
| [0001](0001-multi-crate-workspace.md) | Multi-crate workspace | 2026-08-29 | Accepted |
| [0002](0002-audio-stack.md) | Audio stack: DASP, FunDSP, CPAL | 2026-08-29 | Accepted |
| [0003](0003-file-first-io.md) | File-first I/O | 2026-08-29 | Accepted |
| [0004](0004-imp-integration.md) | Imp integration | 2026-08-29 | Accepted |
| [0005](0005-composition-instrumentation-crates.md) | Composition, scheduler, and instrumentation crates | 2026-08-30 | Accepted |
