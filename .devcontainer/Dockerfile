FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# System deps for CPAL/ALSA (offline rendering does not need these, but the stack will).
RUN apt-get update -qq \
 && apt-get install -y -qq libasound2-dev pkg-config \
 && rm -rf /var/lib/apt/lists/*

# Bake imp-rust at a stable image path (build context = parent of aura/).
COPY imp-rust /opt/imp-rust
