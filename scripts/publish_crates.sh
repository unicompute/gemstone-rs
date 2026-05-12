#!/usr/bin/env bash
set -euo pipefail

DRY_RUN="${DRY_RUN:-0}"
IS_DRY_RUN=0
if [[ "$DRY_RUN" == "1" || "$DRY_RUN" == "true" ]]; then
  IS_DRY_RUN=1
fi

manifest_for_crate() {
  case "$1" in
    gemstone-gci) echo "crates/gemstone-gci/Cargo.toml" ;;
    gemstone-rs-macros) echo "crates/gemstone-rs-macros/Cargo.toml" ;;
    gemstone-rs) echo "crates/gemstone-rs/Cargo.toml" ;;
    gemstone-rs-cli) echo "crates/gemstone-rs-cli/Cargo.toml" ;;
    gemstone-rs-explorer) echo "crates/gemstone-rs-explorer/Cargo.toml" ;;
    *) echo "unknown crate: $1" >&2; return 2 ;;
  esac
}

crate_version() {
  sed -n 's/^version = "\(.*\)"/\1/p' "$(manifest_for_crate "$1")" | head -n 1
}

crate_version_exists() {
  local package="$1"
  local version="$2"
  curl -fsSL "https://crates.io/api/v1/crates/${package}/${version}" >/dev/null 2>&1
}

publish_crate() {
  local package="$1"
  local version
  version="$(crate_version "$package")"
  if [[ "$IS_DRY_RUN" == "1" ]]; then
    cargo package --list -p "$package" --allow-dirty >/dev/null
    echo "package file list ok: $package"
  else
    if crate_version_exists "$package" "$version"; then
      echo "skipping $package $version; already published"
      return 0
    fi
    set +e
    output="$(cargo publish -p "$package" 2>&1)"
    status=$?
    set -e
    printf '%s\n' "$output"
    if [[ "$status" != "0" ]]; then
      if grep -q "already exists on crates.io index" <<<"$output"; then
        echo "skipping $package $version; already published"
        return 0
      fi
      return "$status"
    fi
    sleep 30
  fi
}

publish_crate gemstone-gci
publish_crate gemstone-rs-macros
publish_crate gemstone-rs
publish_crate gemstone-rs-cli
publish_crate gemstone-rs-explorer
