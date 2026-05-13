# Screenshot Workflow

The repository keeps screenshots in `docs/assets/` so the README, guides, PDFs,
Marketplace text, and articles can point at stable image paths.

## Explorer Screenshot

`docs/assets/explorer-home.png` is captured from the local explorer at a
1440x1500 viewport.

Install Playwright once:

```bash
python3 -m pip install playwright
python3 -m playwright install chromium
```

If Playwright is not installed, the capture script falls back to a local
Chrome, Chromium, or Microsoft Edge executable when one is available.

Then refresh the screenshot:

```bash
make screenshots
```

The target starts:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
```

It waits for:

```text
http://127.0.0.1:8787/health
```

Then it writes:

```text
docs/assets/explorer-home.png
```

If the explorer is already running, capture it without starting another server:

```bash
python3 scripts/capture_explorer_screenshots.py --no-server --url http://127.0.0.1:8787/
```

Run a dry path check without launching a browser:

```bash
python3 scripts/capture_explorer_screenshots.py --check
```

The screenshot path does not require GemStone credentials because the explorer
home page renders before live endpoints are called. Live browse/codegen data
still requires the normal `GS_*` environment.

## After Capture

Regenerate PDFs after refreshing screenshots:

```bash
python3 docs/build_pdf_docs.py
```

For a release pass, run:

```bash
make verify
make vscode-package
```
