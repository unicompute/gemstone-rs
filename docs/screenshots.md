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

The target writes:

```text
docs/assets/explorer-home.png
docs/assets/workbench-codegen-edit-flow.png
docs/assets/workbench-codegen-edit-flow.gif
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

Validate all committed visual assets and Markdown image references:

```bash
make visual-asset-check
```

That check verifies the VSIX icon path, required Marketplace/GitHub assets,
local Markdown image links, and repository `raw.githubusercontent.com` image
links that point back into this project.

Validate local Markdown links and same-repository raw GitHub links:

```bash
make docs-link-check
```

That check is offline; it does not probe public websites, but it does catch
missing local files, missing directories, and broken Markdown heading anchors.

The screenshot path does not require GemStone credentials because the explorer
home page renders before live endpoints are called. Live browse/codegen data
still requires the normal `GS_*` environment.

## Workbench Marketplace Images

When the VS Code webview changes, refresh `docs/assets/explorer-home.png`,
`docs/assets/workbench-codegen-edit-flow.png`, and
`docs/assets/workbench-codegen-edit-flow.gif`, then capture any additional
Marketplace/GitHub GIFs from VS Code with the same visible flow:

1. Run `GemStone RS: Launch Explorer`.
2. Run `GemStone RS: Open Explorer Webview`.
3. Click `Preview/Edit Generated Wrappers`.
4. Show the generated-source editor with `Open Output File`, `Open Editable
   Draft`, and `Save Edited Output`.
5. Capture a still screenshot or short GIF for the Marketplace listing.

The committed still image and GIF show the embedded explorer, the Codegen
panel, and the editable generated-output pane so users can see the
review/edit/save loop without reading the command list. Capture an additional
manual GIF only when you need to show the real VS Code confirmation prompt.

## After Capture

Regenerate PDFs after refreshing screenshots:

```bash
python3 docs/build_pdf_docs.py
```

For a release pass, run:

```bash
make visual-asset-check
make docs-link-check
make verify
make vscode-package
```
