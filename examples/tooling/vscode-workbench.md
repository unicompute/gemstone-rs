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
code --install-extension vscode-gemstone-rs-workbench/gemstone-rs-workbench-0.3.4.vsix --force
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

The embedded explorer webview has a matching `Live Browse` inspector. It can
load dictionaries, classes, protocols, methods, and source without leaving the
webview, then hand the method source to a VS Code editor.

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
4. Run `Explain Config`.
5. Run `Open Generated Output` when you want to edit the checked-in wrappers.
6. Run `Generate Wrappers`.
7. Run `Codegen Preview Profile`, `Codegen Diff Profile`, `Codegen Check Profile`, `Codegen Explain Profile`, or `Codegen Generate Profile` when you want a checked-in project profile to supply the config path.
8. Run `Validate py-native Contract`, `Validate py-native Samples Fixture`, or
   `Validate py-native Smoke Fixture` before adapter work that depends on
   checked-in Rust contract fixtures.
9. Run `Run py-native Smoke` when you want dry-run or live adapter smoke output
   in the VS Code output panel.
10. Run `Show py-native Migration Plan` when you want the remaining
    `gemstone-py-native` shared-core checklist in the output panel.
11. Run `Validate py-native Publish Receipt` or `Show py-native Publish
    Receipt` when you want verified TestPyPI/PyPI workflow runs in the output
    panel.
12. Run `Open Codegen Docs`.

`Generate Wrappers` shows the generated diff before writing if output would
change.
`Explain Config` renders the structured classes, methods, mapped fields, and
generated test stubs from `codegen explain --json`.
The embedded webview also has `Preview/Edit Generated Wrappers`,
`Read/Edit Generated Output`, `Preview/Edit Profile`, and `Read/Edit Profile
Output`. Those actions render generated Rust wrappers in an editable pane, with
buttons to open the output file, open the current text as an untitled draft, or
save the edited text back to the output path after confirmation.
The py-native actions call `gemstone-rs py-native check --json`,
`gemstone-rs py-native check-samples --json`,
`gemstone-rs py-native check-smoke --json`, and
`gemstone-rs py-native smoke --json`, and
`gemstone-rs py-native migration --json`, plus the publish-receipt commands,
so VS Code uses the same adapter contract, smoke, migration, and wheel-publish
reports as terminal and CI workflows.

## Explorer Walkthrough

From the `Explorer` tree:

1. Run `Verify Setup`.
2. Run `Verify Live Setup` when credentials and a reachable stone are ready.
3. Run `Verify Strict Setup` before release or CI validation.
4. Run `Launch Explorer`.
5. Run `Run Setup Assistant`.
6. Run `Eval Smalltalk` with `3 + 4`.
7. Run `Open Explorer Webview`.

`Verify Setup` shows the active checkout, CLI, explorer, env file, codegen
config, and project profile paths beside the `gemstone-rs doctor` output. If
`gemstoneRs.envFile` points at an existing file, Workbench passes `--env-file`
automatically to CLI commands and explorer startup. Use the result actions to
copy the report, copy a safe `GS_*` environment export script with a placeholder
password, or jump straight to the workbench settings. `Verify Live Setup` uses
the same report format but calls `gemstone-rs doctor --live`.
`Run Setup Assistant` reads the running explorer's `/api/setup/assistant`
endpoint and renders the env-file, GemStone configuration, GCI library, codegen
config, and project profile checks in the output panel.

`Launch Explorer` starts `gemstone-rs-explorer` in a VS Code terminal and opens
the loopback URL. `Open Explorer Webview` embeds the running local explorer in a
VS Code editor tab.

Inside the embedded explorer, use the `Codegen Workflow` panel to refresh the
known `.codegen` file picker, set `Config root` when the explorer started
outside the checkout, reuse a recent config path, load and save the selected
config file, preview wrappers, view diffs, run freshness checks, and discover a
BridgeRoot mapping config from a live class. Save a named profile when you want
to switch between root/config/mapping/class combinations, and export the
profile JSON when you want to share that setup with another browser or
teammate. When a profile belongs in the project, use `Load Project Profiles`
and `Save Project Profiles` with `gemstone-rs.codegen-profiles.json`, then use
`Export Codegen Profile` for a single shareable profile. Use `Show Sample
Project Profiles` to inspect the built-in profile JSON, `Create Project
Profiles` to write that sample into the project, and `Validate Project
Profiles` to run the CLI schema check from VS Code. `List Project Profiles`
and `Show Project Profile` inspect the parsed profile fields from VS Code, and
`Resolve Project Profile` shows the config path used by profile-driven codegen
without opening the explorer. `Check Project Profiles` runs the CLI freshness
check for every profile so stale generated wrappers are visible before commit.
Profile-specific commands use a QuickPick loaded from the project profile file.
Imports summarize new, replaced, and unchanged profiles. Use the
BridgeRoot key/value type selectors to test string keys, symbol keys, strings,
symbols, small integers, and bools before saving those choices in codegen
config. The webview inspector also has `BridgeRoot Keys`, `BridgeRoot Value`,
`BridgeRoot Shape`, and `BridgeRoot Mapping Preview` actions. `BridgeRoot Keys`
can copy a live key into the key field, `BridgeRoot Value` renders a nested
`BridgeValue` tree, `BridgeRoot Shape` shows relationship paths and repeated
identity groups, and `BridgeRoot Mapping Preview` proposes a starter mapping
for the selected key. The detail pane shows generated source, generated mapping
config, unified diff output, or side-by-side diff output, and the page
remembers the current fields locally across reloads. Config saves use a POST
body, so the editor can handle realistic config files instead of being limited
by URL length.
Workbench buttons can open the current codegen config, project profile file,
and generated output directly in VS Code for review or edits. The webview save
path is intentionally constrained to the configured checkout or active
workspace.
