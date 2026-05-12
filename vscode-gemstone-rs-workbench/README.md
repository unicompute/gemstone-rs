# gemstone-rs Workbench

VS Code commands for `gemstone-rs` CLI, codegen, and local explorer workflows.

The extension is intentionally thin: the Rust CLI remains the stable contract,
and VS Code provides command palette actions, output panels, and generated-file
previews.

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
- `GemStone RS: Codegen Preview`
- `GemStone RS: Codegen Check`
- `GemStone RS: Codegen Generate`
- `GemStone RS: Launch Explorer`
- `GemStone RS: Open Codegen Docs`

`Codegen Preview` opens generated Rust wrappers in an untitled Rust editor.
`Codegen Generate` writes the generated file and opens it.

## Later

Embedding `gemstone-rs-explorer` as a VS Code webview should be a later feature
release. The current extension starts the explorer externally and opens the
local URL.
