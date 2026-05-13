#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.2.1}"
DRY_RUN="${DRY_RUN:-1}"
PUBLISH_CRATES="${PUBLISH_CRATES:-0}"
PUBLISH_VSIX="${PUBLISH_VSIX:-0}"
CREATE_GITHUB_RELEASE="${CREATE_GITHUB_RELEASE:-0}"
VERIFY_PUBLIC="${VERIFY_PUBLIC:-0}"
REFRESH_SCREENSHOTS="${REFRESH_SCREENSHOTS:-0}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VSIX_VERSION="$(node -p "require('$ROOT/vscode-gemstone-rs-workbench/package.json').version")"
VSIX="$ROOT/vscode-gemstone-rs-workbench/gemstone-rs-workbench-${VSIX_VERSION}.vsix"
CHECKSUMS="$ROOT/SHA256SUMS"
RELEASE_NOTES="$ROOT/release-notes.md"

cd "$ROOT"

if [[ "$REFRESH_SCREENSHOTS" == "1" || "$REFRESH_SCREENSHOTS" == "true" ]]; then
  echo "== screenshots =="
  make screenshots
else
  echo "skipping screenshot refresh; set REFRESH_SCREENSHOTS=1"
fi

echo "== verify =="
make verify

echo "== package VSIX ${VSIX_VERSION} =="
make vscode-package

echo "== checksums =="
{
  if [[ -f "$VSIX" ]]; then
    shasum -a 256 "$VSIX"
  fi
  find docs/pdf -type f -name '*.pdf' -print0 | sort -z | xargs -0 shasum -a 256
} > "$CHECKSUMS"
echo "wrote $CHECKSUMS"

if [[ "$PUBLISH_CRATES" == "1" || "$PUBLISH_CRATES" == "true" ]]; then
  echo "== publish crates =="
  DRY_RUN="$DRY_RUN" scripts/publish_crates.sh
else
  echo "skipping crates publish; set PUBLISH_CRATES=1"
fi

if [[ "$PUBLISH_VSIX" == "1" || "$PUBLISH_VSIX" == "true" ]]; then
  echo "== publish VSIX =="
  cd "$ROOT/vscode-gemstone-rs-workbench"
  npx @vscode/vsce publish --packagePath "$VSIX"
  cd "$ROOT"
else
  echo "skipping VSIX publish; set PUBLISH_VSIX=1"
fi

if [[ "$CREATE_GITHUB_RELEASE" == "1" || "$CREATE_GITHUB_RELEASE" == "true" ]]; then
  echo "== GitHub release =="
  cat > "$RELEASE_NOTES" <<NOTES
gemstone-rs ${VERSION}

Install:

\`\`\`bash
cargo add gemstone-rs
cargo install gemstone-rs-cli
cargo install gemstone-rs-explorer
\`\`\`

VS Code Workbench:

- https://marketplace.visualstudio.com/items?itemName=unicompute.gemstone-rs-workbench

Highlights:

- \`--env-file .env.gemstone-rs\` support across CLI and explorer startup.
- Workbench automatically passes the configured env file when it exists.
- \`codegen explain --json\` for structured Codegen summaries.
- Codegen schemas and generated-wrapper compile smoke checks.
- Explorer setup assistant for env, codegen config, project profiles, and strict setup flow.
- VS Code command for rendering the explorer setup assistant report.
- Clearer object-mapping path diagnostics for nested fields and arrays.

Release assets:

- VSIX package
- PDF documentation
- SHA256SUMS
NOTES
  git tag "v${VERSION}" || true
  git push origin "v${VERSION}" || true
  gh release create "v${VERSION}" "$VSIX" "$CHECKSUMS" docs/pdf/*.pdf \
    --title "gemstone-rs v${VERSION}" \
    --notes-file "$RELEASE_NOTES"
else
  echo "skipping GitHub release; set CREATE_GITHUB_RELEASE=1"
fi

if [[ "$VERIFY_PUBLIC" == "1" || "$VERIFY_PUBLIC" == "true" ]]; then
  echo "== public verification =="
  scripts/publish_verify.sh "$VERSION"
else
  echo "skipping public verification; set VERIFY_PUBLIC=1"
fi

echo "release wrapper complete"
