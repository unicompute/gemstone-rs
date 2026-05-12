#!/usr/bin/env bash
set -euo pipefail

DRY_RUN="${DRY_RUN:-0}"
ARGS=()
if [[ "$DRY_RUN" == "1" || "$DRY_RUN" == "true" ]]; then
  ARGS+=(--dry-run)
fi

publish_crate() {
  local package="$1"
  cargo publish -p "$package" "${ARGS[@]}"
  if [[ ${#ARGS[@]} -eq 0 ]]; then
    sleep 30
  fi
}

publish_crate gemstone-gci
publish_crate gemstone-rs
publish_crate gemstone-rs-cli
publish_crate gemstone-rs-explorer
