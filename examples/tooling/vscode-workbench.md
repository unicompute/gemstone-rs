# VS Code Workbench Example

This example proves the VS Code workflow around the `gemstone-rs` CLI,
codegen, and local explorer.

## Install

From Marketplace:

```bash
code --install-extension unicompute.gemstone-rs-workbench
```

From a local VSIX:

```bash
code --install-extension vscode-gemstone-rs-workbench/gemstone-rs-workbench-0.2.4.vsix --force
```

## Source Checkout Settings

Use these settings when the current workspace is not the `gemstone-rs` checkout:

```json
{
  "gemstoneRs.checkoutPath": "/path/to/gemstone-rs",
  "gemstoneRs.useCargo": true,
  "gemstoneRs.codegenConfig": "examples/codegen/gemstone-rs.codegen"
}
```

Use these settings for installed binaries:

```json
{
  "gemstoneRs.cliPath": "gemstone-rs",
  "gemstoneRs.explorerPath": "gemstone-rs-explorer",
  "gemstoneRs.codegenConfig": "gemstone-rs.codegen"
}
```

## Sidebar Walkthrough

Open the GemStone RS activity bar item.

1. Expand `Dictionaries`.
2. Expand a dictionary such as `UserGlobals`.
3. Expand a class.
4. Expand a protocol.
5. Select a method to open its source.

This exercises:

- `gemstone-rs browse dictionaries`
- `gemstone-rs browse classes`
- `gemstone-rs browse protocols`
- `gemstone-rs browse methods`
- `gemstone-rs browse source`

## Codegen Walkthrough

From the `Codegen Config` tree:

1. Run `Preview Wrappers`.
2. Run `Diff Generated Output`.
3. Run `Check Freshness`.
4. Run `Generate Wrappers`.
5. Run `Open Codegen Docs`.

`Generate Wrappers` shows the generated diff before writing if output would
change.

## Explorer Walkthrough

From the `Explorer` tree:

1. Run `Verify Setup`.
2. Run `Eval Smalltalk` with `3 + 4`.
3. Run `Launch Explorer`.

`Launch Explorer` starts `gemstone-rs-explorer` in a VS Code terminal and opens
the loopback URL.
