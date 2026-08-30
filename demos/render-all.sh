#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo build -p aura-cli --quiet

cargo run -p aura-cli --quiet -- \
  --graph demos/sine.json \
  --output output/sine.wav \
  --duration 10s \
  --sample-rate 44100

cargo run -p aura-cli --quiet -- \
  --graph demos/arpeggio.json \
  --output output/arpeggio.wav \
  --duration 2m \
  --tempo 120 \
  --sample-rate 44100

echo "Wrote output/sine.wav and output/arpeggio.wav"
