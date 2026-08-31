#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo build -p aura-cli --quiet

cargo run -p aura-cli --quiet -- \
  --graph demos/demo-01.json \
  --output output/demo-01.wav \
  --duration 8m \
  --tempo 120 \
  --sample-rate 44100

echo "Wrote output/demo-01.wav"
