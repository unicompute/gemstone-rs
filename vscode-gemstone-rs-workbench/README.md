# gemstone-rs Workbench

![gemstone-rs Workbench graphic](media/gemstone-rs-graphic.png)

VS Code commands for `gemstone-rs` CLI, codegen, and local explorer workflows.

The extension is intentionally thin: the Rust CLI remains the stable contract,
and VS Code provides command palette actions, output panels, and generated-file
previews. The GemStone RS activity bar view also exposes a browsable tree for
dictionaries, classes, protocols, methods, and codegen actions.

## Setup

For an installed CLI:

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

## Commands

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

`Codegen Preview` opens generated Rust wrappers in an untitled Rust editor.
`Codegen Diff` opens a generated diff. `Codegen Generate` shows that diff
before writing, then opens the generated file after a successful write.
`Generate Mapping Config` asks the live stone for a starter `BridgeMapped`
config. `Preview BridgeRoot` opens the explorer BridgeRoot endpoint.
`Run Generated Mapping Example` starts the checked-in mapping demo in a
terminal.

## Sidebar

Open the GemStone RS activity bar item to use the sidebar tree:

- `Dictionaries` expands live dictionaries, classes, protocols, and methods.
- Selecting a method opens its GemStone source through `gemstone-rs browse source`.
- `Codegen Config` exposes Discover, Preview, Diff, Check, Generate, and Docs.
- `Codegen Config` also exposes Generate Mapping Config, Preview BridgeRoot,
  and Run Generated Mapping Example.
- `Explorer` exposes Verify Setup, Eval Smalltalk, and Launch Explorer.

The tree uses the same settings as the command palette actions. If
`gemstoneRs.useCargo` is true, commands run through the local checkout with
`cargo run -p gemstone-rs-cli -- ...`.

## Package Locally

From the repository root:

```bash
make vscode-package
```

Or from this directory:

```bash
npm ci
npm run package -- --out gemstone-rs-workbench-0.2.1.vsix
```

## Later

Embedding `gemstone-rs-explorer` as a VS Code webview should be a later feature
release. The current extension starts the explorer externally and opens the
local URL.
