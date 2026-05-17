#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY_RUN=0
CHECK_ENV_ONLY=0
SKIP_FRAMEWORK=0
SKIP_EXAMPLES=0

usage() {
  cat <<'USAGE'
usage: scripts/live_smoke.sh [--dry-run] [--check-env-only] [--skip-framework] [--skip-examples]

Runs the live GemStone smoke lane used by release verification.

Required environment:
  GS_USERNAME
  GS_PASSWORD
  one of: GS_LIB_PATH, GS_LIB, GEMSTONE
  one of: GS_STONE, GS_STONE_NAME

Optional GemStone connection values such as GS_HOST, GS_NETLDI, and
GS_GEM_SERVICE are read by gemstone-rs when present.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --check-env-only)
      CHECK_ENV_ONLY=1
      shift
      ;;
    --skip-framework)
      SKIP_FRAMEWORK=1
      shift
      ;;
    --skip-examples)
      SKIP_EXAMPLES=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

missing=()
[[ -n "${GS_USERNAME:-}" ]] || missing+=("GS_USERNAME")
[[ -n "${GS_PASSWORD:-}" ]] || missing+=("GS_PASSWORD")
if [[ -z "${GS_LIB_PATH:-}" && -z "${GS_LIB:-}" && -z "${GEMSTONE:-}" ]]; then
  missing+=("GS_LIB_PATH or GS_LIB/GEMSTONE")
fi
if [[ -z "${GS_STONE:-}" && -z "${GS_STONE_NAME:-}" ]]; then
  missing+=("GS_STONE or GS_STONE_NAME")
fi

if [[ "$DRY_RUN" == "0" && "${#missing[@]}" -gt 0 ]]; then
  echo "live-smoke: missing GemStone environment:" >&2
  for name in "${missing[@]}"; do
    echo "  - $name" >&2
  done
  echo "live-smoke: write a template with \`cargo run -p gemstone-rs-cli -- env write .env.gemstone-rs\`." >&2
  echo "live-smoke: then source it or pass values as GitHub Actions secrets." >&2
  exit 2
fi

run() {
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  if [[ "$DRY_RUN" == "0" ]]; then
    "$@"
  fi
}

cd "$ROOT"

echo "== live GemStone smoke =="
if [[ "$DRY_RUN" == "1" ]]; then
  echo "dry-run: commands will be printed, not executed"
fi
if [[ "${#missing[@]}" -gt 0 ]]; then
  echo "dry-run: missing live environment values:"
  for name in "${missing[@]}"; do
    echo "  - $name"
  done
fi

if [[ "$CHECK_ENV_ONLY" == "1" ]]; then
  echo "live-smoke: environment check complete"
  exit 0
fi

run env GS_RUN_LIVE_RUST=1 cargo run -p gemstone-rs-cli -- doctor --strict --live
run env GS_RUN_LIVE_RUST=1 cargo test -p gemstone-rs live_ -- --test-threads=1

if [[ "$SKIP_EXAMPLES" == "0" ]]; then
  run env GS_RUN_LIVE_RUST=1 cargo run -p gemstone-rs --example live_smoke_cookbook
  run env GS_RUN_LIVE_RUST=1 cargo run -p gemstone-rs --example python_native_adapter
else
  echo "skipping live examples; --skip-examples was provided"
fi

if [[ "$SKIP_FRAMEWORK" == "0" ]]; then
  run env GS_RUN_LIVE_RUST=1 python3 scripts/framework_route_smoke.py --live
else
  echo "skipping live framework route smoke; --skip-framework was provided"
fi

echo "live GemStone smoke complete"
