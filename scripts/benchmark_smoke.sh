#!/usr/bin/env bash
set -euo pipefail

ITERATIONS="${ITERATIONS:-20}"
EXPR="${1:-3 + 4}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT"

echo "gemstone-rs benchmark smoke"
echo "iterations=$ITERATIONS"
echo "expr=$EXPR"
echo "This is a coarse CLI smoke benchmark, not a microbenchmark."

start="$(date +%s)"
for _ in $(seq 1 "$ITERATIONS"); do
  cargo run -q -p gemstone-rs-cli -- eval "$EXPR" >/dev/null
done
end="$(date +%s)"
elapsed=$((end - start))

echo "elapsed_seconds=$elapsed"
if [[ "$elapsed" -gt 0 ]]; then
  echo "evals_per_second=$((ITERATIONS / elapsed))"
fi
