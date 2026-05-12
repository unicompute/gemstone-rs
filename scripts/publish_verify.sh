#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.1.0}"
PUBLISHER="${VSCODE_PUBLISHER:-unicompute}"
EXTENSION="${VSCODE_EXTENSION:-gemstone-rs-workbench}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/gemstone-rs-install.XXXXXX")"

cleanup() {
  rm -rf "$INSTALL_ROOT"
}
trap cleanup EXIT

require() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 2
  }
}

check_crate_version() {
  local crate="$1"
  local url="https://crates.io/api/v1/crates/${crate}/${VERSION}"
  printf 'checking %s %s on crates.io... ' "$crate" "$VERSION"
  curl -fsSL "$url" >/dev/null
  printf 'ok\n'
}

check_marketplace_version() {
  local expected
  expected="$(node -e "console.log(require('$ROOT/vscode-gemstone-rs-workbench/package.json').version)")"
  printf 'checking Marketplace %s.%s version... ' "$PUBLISHER" "$EXTENSION"
  local actual
  actual="$(
    cd "$ROOT/vscode-gemstone-rs-workbench"
    npx @vscode/vsce show "${PUBLISHER}.${EXTENSION}" --json \
      | node -e "let data=''; process.stdin.on('data', c => data += c); process.stdin.on('end', () => console.log(JSON.parse(data).versions[0].version));"
  )"
  if [[ "$actual" != "$expected" ]]; then
    echo "expected $expected, got $actual" >&2
    exit 1
  fi
  printf 'ok (%s)\n' "$actual"
}

require cargo
require curl
require node
require npx

for crate in gemstone-gci gemstone-rs gemstone-rs-cli gemstone-rs-explorer; do
  check_crate_version "$crate"
done

echo "installing CLI packages into $INSTALL_ROOT"
CARGO_INSTALL_ROOT="$INSTALL_ROOT" cargo install gemstone-rs-cli --version "$VERSION"
CARGO_INSTALL_ROOT="$INSTALL_ROOT" cargo install gemstone-rs-explorer --version "$VERSION"

"$INSTALL_ROOT/bin/gemstone-rs" --help >/dev/null
"$INSTALL_ROOT/bin/gemstone-rs-explorer" --help >/dev/null
echo "installed binaries respond to --help"

check_marketplace_version
echo "publish verification complete"
