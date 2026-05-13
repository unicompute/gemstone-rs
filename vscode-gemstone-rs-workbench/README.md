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
  "gemstoneRs.explorerAuthToken": "",
  "gemstoneRs.codegenConfig": "gemstone-rs.codegen",
  "gemstoneRs.bridgeRoot": "GemStoneRsBridgeRoot"
}
```

For a source checkout:

```json
{
  "gemstoneRs.checkoutPath": "/path/to/gemstone-rs",
  "gemstoneRs.useCargo": true,
  "gemstoneRs.codegenConfig": "examples/codegen/gemstone-rs.codegen",
  "gemstoneRs.bridgeRoot": "GemStoneRsBridgeRoot"
}
```

## Commands

- `GemStone RS: Verify Setup`
- `GemStone RS: Verify Live Setup`
- `GemStone RS: Verify Strict Setup`
- `GemStone RS: Run Setup Assistant`
- `GemStone RS: Doctor`
- `GemStone RS: Show Environment Template`
- `GemStone RS: Copy Environment Template`
- `GemStone RS: Write .env.gemstone-rs`
- `GemStone RS: Eval Smalltalk`
- `GemStone RS: Browse Dictionaries`
- `GemStone RS: Browse Classes`
- `GemStone RS: Show Example Commands`
- `GemStone RS: Codegen Init`
- `GemStone RS: Codegen Discover`
- `GemStone RS: Generate Mapping Config`
- `GemStone RS: Preview BridgeRoot`
- `GemStone RS: List BridgeRoot Keys`
- `GemStone RS: Put BridgeRoot String`
- `GemStone RS: Put BridgeRoot Symbol`
- `GemStone RS: Put BridgeRoot SmallInt`
- `GemStone RS: Put BridgeRoot Bool`
- `GemStone RS: Remove BridgeRoot Key`
- `GemStone RS: Run Generated Mapping Example`
- `GemStone RS: Codegen Preview`
- `GemStone RS: Codegen Diff`
- `GemStone RS: Codegen Check`
- `GemStone RS: Codegen Explain`
- `GemStone RS: Codegen Generate`
- `GemStone RS: Codegen Preview Profile`
- `GemStone RS: Codegen Diff Profile`
- `GemStone RS: Codegen Check Profile`
- `GemStone RS: Codegen Explain Profile`
- `GemStone RS: Codegen Generate Profile`
- `GemStone RS: Load Project Profiles`
- `GemStone RS: Save Project Profiles`
- `GemStone RS: Export Codegen Profile`
- `GemStone RS: Show Sample Project Profiles`
- `GemStone RS: Create Project Profiles`
- `GemStone RS: Validate Project Profiles`
- `GemStone RS: List Project Profiles`
- `GemStone RS: Show Project Profile`
- `GemStone RS: Resolve Project Profile`
- `GemStone RS: Check Project Profiles`
- `GemStone RS: Generate Explorer Auth Token`
- `GemStone RS: Clear Explorer Auth Token`
- `GemStone RS: Launch Explorer`
- `GemStone RS: Open Explorer Webview`
- `GemStone RS: Open Method Source`
- `GemStone RS: Open Codegen Docs`

`Codegen Preview` opens generated Rust wrappers in an untitled Rust editor.
`Codegen Diff` opens a generated diff. `Codegen Generate` shows that diff
before writing, then opens the generated file after a successful write. The
`Codegen Explain` renders the structured `codegen explain --json` summary with
classes, selectors, return types, mapped fields, and generated test stubs. The
profile variants run the same preview/diff/check/explain/generate loop from a
named project profile in `gemstone-rs.codegen-profiles.json`.
`Show Example Commands` uses `gemstone-rs examples list --json` to show the
same curated example map that the CLI exposes, then lets you run a selected
example in a terminal, copy its command, or open the examples guide. This is
the Rust workbench equivalent of the gemstone-py examples launcher.
`Load Project Profiles`, `Save Project Profiles`, and `Export Codegen Profile`
open the local explorer workflow where project profile JSON can be loaded,
schema-validated, summarized, and saved under `--allow-write`. `Show Sample
Project Profiles` opens the built-in `gemstone-rs profile sample` output in an
untitled JSON editor, `Create Project Profiles` writes that sample with
`gemstone-rs profile init`, and `Validate Project Profiles` runs
`gemstone-rs profile validate` directly and shows the result in the GemStone RS
output panel. `List Project Profiles`, `Show Project Profile`, and `Resolve
Project Profile` render parsed profile details and resolved config paths.
`Check Project Profiles` renders the shared `profile check --json` report with
per-profile freshness, output paths, and errors.
through `gemstone-rs profile list/show/resolve`. `Check Project Profiles` runs
`gemstone-rs profile check --json` across every profile and renders an
aggregate summary with ok, stale, and error counts before a commit. The result
message can copy the report or open the profile file. Profile commands use a
QuickPick populated from the project profile file. The extension also
contributes JSON validation for files named
`gemstone-rs.codegen-profiles.json`.
`Generate Mapping Config` asks the live stone for a starter `BridgeMapped`
config. `Preview BridgeRoot` opens the explorer BridgeRoot endpoint.
`List BridgeRoot Keys` runs `gemstone-rs bridge keys` and shows the current
root keys in the output panel.
`Put BridgeRoot String`, `Put BridgeRoot Symbol`, `Put BridgeRoot SmallInt`,
`Put BridgeRoot Bool`, and `Remove BridgeRoot Key` run explicit committed
BridgeRoot writes through the CLI after prompting for a key, String/Symbol key
type, and value. They pass the configured `gemstoneRs.bridgeRoot` setting as
`--root`, so teams can use a project-specific bridge dictionary instead of the
default `GemStoneRsBridgeRoot`.
`Run Generated Mapping Example` starts the checked-in mapping demo in a
terminal.
`Open Explorer Webview` embeds the running loopback explorer inside VS Code.
The webview wraps the explorer in a workbench shell: the iframe remains the
full browser UI, while the side inspector can query explorer status, setup
checks, project profile freshness, Codegen preview/diff/check JSON, and
BridgeRoot keys. Workbench buttons can hand off to native VS Code commands for
previewing wrappers, opening diffs, generating with confirmation, checking
project profiles, opening docs, and opening generated output files.
If `gemstoneRs.explorerAuthToken` is set, `Launch Explorer` passes
`--auth-token-env GEMSTONE_RS_EXPLORER_TOKEN` and all browser/webview URLs
include the matching `token=` query parameter. Use `Generate Explorer Auth
Token` to create a local random token, store it in VS Code settings, and copy
it to the clipboard. Use `Clear Explorer Auth Token` to return to the default
loopback-only, no-token mode.
`Verify Setup` and `Doctor` run `gemstone-rs doctor`, so VS Code reports the
same masked environment and GCI-library checks as the terminal. `Verify Live
Setup` runs `gemstone-rs doctor --live` when credentials and a reachable stone
are available. `Verify Strict Setup` runs `gemstone-rs doctor --strict` for
CI-style validation of explicit stone and GCI library settings. All setup
checks report the active checkout, CLI, explorer, codegen config, and profile
paths, configured BridgeRoot, plus whether the configured `gemstoneRs.envFile`
exists. When the env file exists, Workbench passes `--env-file` automatically
to CLI commands and explorer startup. Their result actions can copy the setup
report, copy a safe `GS_*` environment export script with a placeholder
password, or open the extension settings. `Show Environment Template`, `Copy
Environment Template`, and `Write .env.gemstone-rs` use the CLI
`gemstone-rs env sample/write` commands so VS Code and terminal setup stay on
the same safe template.
`Run Setup Assistant` calls the local explorer endpoint `/api/setup/assistant`
and renders the env-file, GemStone configuration, GCI library, codegen config,
and project profile checks in the GemStone RS output panel. Start the explorer
first with `GemStone RS: Launch Explorer`.

## Sidebar

Open the GemStone RS activity bar item to use the sidebar tree:

- `Dictionaries` expands live dictionaries, classes, protocols, and methods.
- Selecting a method opens its GemStone source through `gemstone-rs browse source`.
- `Codegen Config` exposes Discover, Preview, Diff, Check, Generate, profile
  preview/diff/check/generate, and Docs.
- `Codegen Config` also exposes Load Project Profiles, Save Project Profiles,
  Export Codegen Profile, Show Sample Project Profiles, Create Project
  Profiles, Validate Project Profiles, List Project Profiles, Show Project
  Profile, Resolve Project Profile, Check Project Profiles, Generate Mapping Config, Preview
  BridgeRoot, List BridgeRoot Keys, Put BridgeRoot String, Put BridgeRoot
  Symbol, Put BridgeRoot SmallInt, Put BridgeRoot Bool, Remove BridgeRoot Key,
  and Run Generated Mapping Example.
- `Explorer` exposes Doctor, Verify Setup, Verify Live Setup, Verify Strict
  Setup, Eval Smalltalk, Show Example Commands, Show/Copy/Write Environment
  Template, Launch Explorer, Open Explorer Webview, Generate Explorer Auth
  Token, and Clear Explorer Auth Token.

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
npm run check
npm run test:smoke
npm run package -- --out gemstone-rs-workbench-0.3.1.vsix
```
