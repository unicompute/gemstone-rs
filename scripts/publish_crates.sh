#!/usr/bin/env bash
set -euo pipefail

DRY_RUN="${DRY_RUN:-0}"
IS_DRY_RUN=0
if [[ "$DRY_RUN" == "1" || "$DRY_RUN" == "true" ]]; then
  IS_DRY_RUN=1
fi

publish_crate() {
  local package="$1"
  if [[ "$IS_DRY_RUN" == "1" ]]; then
    cargo package --list -p "$package" --allow-dirty >/dev/null
    echo "package file list ok: $package"
  else
    cargo publish -p "$package"
    sleep 30
  fi
}

publish_crate gemstone-gci
publish_crate gemstone-rs-macros
publish_crate gemstone-rs
publish_crate gemstone-rs-cli
publish_crate gemstone-rs-explorer
