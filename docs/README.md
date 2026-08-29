# Documentation

This directory is the source of truth for Aura. Requirements and design docs define what the software should be; code must conform to them.

## Docs govern code

When documentation and implementation disagree:

1. Update the code to match the doc, or
2. Explicitly revise the doc to reflect an intentional change.

Never let code silently drift from documented requirements or design.

## Directory map

| Directory | Purpose |
|-----------|---------|
| [`requirements/`](requirements/) | Product definition, goals, and behavioral requirements |
| [`design/`](design/) | Architecture, module boundaries, and system design |
| [`decisions/`](decisions/) | Architecture decision records (ADRs) — dated, append-only |

## When to update

Update docs when:

- Adding or changing a feature
- Changing observable behavior
- Making an architectural choice
- Receiving a repeated agent or user instruction that defines persistent project intent

## Authoring style

- Write imperatively and specifically.
- Record dated decisions in `decisions/` as ADRs.
- Do not duplicate content from the root [`AGENTS.md`](../AGENTS.md) — link to it instead.
- Prefer updating an existing doc over creating a new one unless the topic warrants separation.
