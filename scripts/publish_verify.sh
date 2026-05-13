#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.2.2}"
PUBLISHER="${VSCODE_PUBLISHER:-unicompute}"
EXTENSION="${VSCODE_EXTENSION:-gemstone-rs-workbench}"
REPO="${GITHUB_REPOSITORY:-unicompute/gemstone-rs}"
VERIFY_GITHUB_RELEASE="${VERIFY_GITHUB_RELEASE:-1}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VSIX_VERSION="$(node -e "console.log(require('$ROOT/vscode-gemstone-rs-workbench/package.json').version)")"
INSTALL_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/gemstone-rs-install.XXXXXX")"
RELEASE_DOWNLOAD_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/gemstone-rs-release-assets.XXXXXX")"

cleanup() {
  rm -rf "$INSTALL_ROOT"
  rm -rf "$RELEASE_DOWNLOAD_ROOT"
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

expected_github_release_assets() {
  printf '%s\n' "gemstone-rs-workbench-${VSIX_VERSION}.vsix"
  printf '%s\n' "SHA256SUMS"
  find "$ROOT/docs/pdf" -maxdepth 1 -type f -name '*.pdf' -print \
    | sort \
    | while IFS= read -r pdf; do
        basename "$pdf"
      done
}

require cargo
require curl
require node
require npx
if [[ "$VERIFY_GITHUB_RELEASE" != "0" && "$VERIFY_GITHUB_RELEASE" != "false" ]]; then
  require gh
  require python3
fi

for crate in gemstone-gci gemstone-rs-macros gemstone-rs gemstone-rs-cli gemstone-rs-explorer; do
  check_crate_version "$crate"
done

echo "installing CLI packages into $INSTALL_ROOT"
CARGO_INSTALL_ROOT="$INSTALL_ROOT" cargo install gemstone-rs-cli --version "$VERSION"
CARGO_INSTALL_ROOT="$INSTALL_ROOT" cargo install gemstone-rs-explorer --version "$VERSION"

"$INSTALL_ROOT/bin/gemstone-rs" --help >/dev/null
"$INSTALL_ROOT/bin/gemstone-rs-explorer" --help >/dev/null
echo "installed binaries respond to --help"

check_marketplace_version

check_github_release_assets() {
  local tag="${GITHUB_RELEASE_TAG:-v${VERSION}}"
  printf 'checking GitHub release %s assets... ' "$tag"
  local assets
  assets="$(gh release view "$tag" --repo "$REPO" --json assets --jq '.assets[].name')"
  local missing=0
  local expected_count=0
  while IFS= read -r expected; do
    expected_count=$((expected_count + 1))
    if ! grep -Fxq "$expected" <<<"$assets"; then
      echo "missing asset $expected" >&2
      missing=1
    fi
  done < <(expected_github_release_assets)
  if [[ "$missing" != "0" ]]; then
    echo "release assets found:" >&2
    echo "$assets" >&2
    exit 1
  fi
  while IFS= read -r expected; do
    gh release download "$tag" --repo "$REPO" --dir "$RELEASE_DOWNLOAD_ROOT" --pattern "$expected" --clobber >/dev/null
  done < <(expected_github_release_assets)
  python3 "$ROOT/scripts/verify_downloaded_release_assets.py" "$RELEASE_DOWNLOAD_ROOT"
  printf 'ok (%s expected assets)\n' "$expected_count"
}

if [[ "$VERIFY_GITHUB_RELEASE" != "0" && "$VERIFY_GITHUB_RELEASE" != "false" ]]; then
  check_github_release_assets
else
  echo "skipping GitHub release asset check"
fi

echo "publish verification complete"
