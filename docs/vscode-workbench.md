# VS Code Workbench

`gemstone-rs Workbench` is the VS Code companion for the Rust CLI and local
explorer. It keeps the CLI as the stable contract and adds a sidebar, output
panel, generated-file previews, and codegen diff prompts.

Marketplace:

```text
https://marketplace.visualstudio.com/items?itemName=unicompute.gemstone-rs-workbench
```

## Setup

For installed CLI tools:

```json
{
  "gemstoneRs.cliPath": "gemstone-rs",
  "gemstoneRs.explorerPath": "gemstone-rs-explorer",
  "gemstoneRs.codegenConfig": "gemstone-rs.codegen"
}
```

For a source checkout:

```json
{
  "gemstoneRs.checkoutPath": "/path/to/gemstone-rs",
  "gemstoneRs.useCargo": true,
  "gemstoneRs.codegenConfig": "examples/codegen/gemstone-rs.codegen"
}
```

Use the same `GS_*` environment as the CLI.

## Sidebar Tree

Open the GemStone RS activity bar item.

The sidebar contains:

- `Dictionaries`
- `Codegen Config`
- `Explorer`

`Dictionaries` expands live dictionaries, classes, protocols, and methods.
Selecting a method opens its source in an untitled Smalltalk editor.

`Codegen Config` exposes:

- Discover from Live Stone
- Generate Mapping Config
- Preview BridgeRoot
- Run Generated Mapping Example
- Preview Wrappers
- Diff Generated Output
- Check Freshness
- Generate Wrappers
- Open Codegen Docs

`Explorer` exposes:

- Verify Setup
- Eval Smalltalk
- Launch Explorer

## Command Palette

Commands:

- `GemStone RS: Verify Setup`
- `GemStone RS: Eval Smalltalk`
- `GemStone RS: Browse Dictionaries`
- `GemStone RS: Browse Classes`
- `GemStone RS: Codegen Init`
- `GemStone RS: Codegen Discover`
- `GemStone RS: Generate Mapping Config`
- `GemStone RS: Preview BridgeRoot`
- `GemStone RS: Run Generated Mapping Example`
- `GemStone RS: Codegen Preview`
- `GemStone RS: Codegen Diff`
- `GemStone RS: Codegen Check`
- `GemStone RS: Codegen Generate`
- `GemStone RS: Launch Explorer`
- `GemStone RS: Open Method Source`
- `GemStone RS: Open Codegen Docs`

## Codegen Workflow

1. Set `gemstoneRs.codegenConfig`.
2. Run `GemStone RS: Codegen Discover` or edit the config manually.
3. Run `GemStone RS: Codegen Preview`.
4. Run `GemStone RS: Codegen Diff`.
5. Run `GemStone RS: Codegen Generate`.

`Codegen Generate` runs the diff first. If output would change, it opens the
diff and asks before writing.

## Object Mapping Workflow

Use `GemStone RS: Generate Mapping Config` when you want the live stone to
inspect a GemStone class and propose a `BridgeMapped` config. The command asks
for:

- config path
- Rust struct name
- GemStone class reference, such as `Object` or `UserGlobals:OkzBooking`

Use `GemStone RS: Preview BridgeRoot` after starting the local explorer. It
opens:

```text
http://127.0.0.1:8787/api/bridge/root
```

Use `GemStone RS: Run Generated Mapping Example` to run:

```bash
cargo run -p gemstone-rs --example generated_mapping_app
```

## Develop the Extension

```bash
cd vscode-gemstone-rs-workbench
npm ci
npm run check
npm run package -- --out gemstone-rs-workbench-0.2.0.vsix
```

From the repository root:

```bash
make vscode-package
```

## Later Feature Work

Embedding `gemstone-rs-explorer` as a VS Code webview should remain a feature
release because it changes the user experience and test surface. The current
extension intentionally launches the explorer externally and opens the local
URL.
