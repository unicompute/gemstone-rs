# gemstone-rs Workbench

![gemstone-rs Workbench graphic](media/gemstone-rs-graphic.png)

VS Code commands for `gemstone-rs` CLI, codegen, and local explorer workflows.

The extension is intentionally thin: the Rust CLI remains the stable contract,
and VS Code provides command palette actions, output panels, and generated-file
previews. The GemStone RS activity bar view also exposes a browsable tree for
dictionaries, classes, protocols, methods, and codegen actions.

![gemstone-rs Workbench codegen edit flow](https://raw.githubusercontent.com/unicompute/gemstone-rs/main/docs/assets/workbench-codegen-edit-flow.gif)

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
- `GemStone RS: Open Codegen Config`
- `GemStone RS: Open Project Profiles`
- `GemStone RS: Open Generated Output`
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
- `GemStone RS: Validate py-native Contract`
- `GemStone RS: Validate py-native Samples Fixture`
- `GemStone RS: Validate py-native Smoke Fixture`
- `GemStone RS: Run py-native Smoke`
- `GemStone RS: Show py-native Migration Plan`
- `GemStone RS: Validate py-native Conformance Fixture`
- `GemStone RS: Validate py-native Handoff Bundle`
- `GemStone RS: Show py-native Handoff Bundle`
- `GemStone RS: Validate py-native Publish Receipt`
- `GemStone RS: Show py-native Publish Receipt`
- `GemStone RS: Validate py-native Shared Core Gate`
- `GemStone RS: Compare with gemstone-py`
- `GemStone RS: Show All Comparison Status`

`Codegen Preview` opens generated Rust wrappers in an untitled Rust editor.
`Codegen Diff` opens a generated diff. `Codegen Generate` shows that diff
before writing, then opens the generated file after a successful write. The
`Codegen Explain` renders the structured `codegen explain --json` summary with
classes, selectors, return types, mapped fields, and generated test stubs.
`Open Codegen Config`, `Open Project Profiles`, and `Open Generated Output`
open the current config, project profile file, and generated wrapper output in
VS Code editors so review and edits stay in the IDE. The profile variants run
the same preview/diff/check/explain/generate loop from a
named project profile in `gemstone-rs.codegen-profiles.json`.
`Validate py-native Contract` runs
`gemstone-rs py-native check --json` against `gemstoneRs.pyNativeFixture` so a
future `gemstone-py-native` wrapper can verify the Rust adapter contract from
the editor. `Validate py-native Samples Fixture` runs
`gemstone-rs py-native check-samples --json` against
`gemstoneRs.pyNativeSamplesFixture`, which gives wrapper work concrete value
and structured-error payloads to translate. `Validate py-native Smoke Fixture` runs
`gemstone-rs py-native check-smoke --json` against
`gemstoneRs.pyNativeSmokeFixture`, which catches drift in the dry-run smoke
fixture before Python adapter work consumes it. `Run py-native Smoke` runs
`gemstone-rs py-native smoke --json` in either dry-run or live mode and renders
each adapter step in the output panel. `Show py-native Migration Plan` runs
`gemstone-rs py-native migration --json` and renders the remaining
`gemstone-py-native` shared-core checklist in the same output panel. `Validate
py-native Conformance Fixture` runs `gemstone-rs py-native check-conformance
--json` against `gemstoneRs.pyNativeConformanceFixture` and verifies the PyO3
module functions, raw `NativeSession` methods, compatibility shim methods,
fixture paths, and scaffold files expected from the downstream wrapper,
including value-level methods such as `eval_json`, `perform_json`,
`new_symbol`, and `value_to_oop_*`.
`Validate py-native Handoff Bundle` runs `gemstone-rs py-native check-handoff
--json` against `gemstoneRs.pyNativeHandoffFixture`, and `Show py-native
Handoff Bundle` renders the downstream `gemstone-py-native` artifact list and
acceptance criteria from `gemstone-rs py-native handoff --json`. `Validate
py-native Publish Receipt` runs `gemstone-rs py-native
check-publish-receipt --json` against
`gemstoneRs.pyNativePublishReceiptFixture`, and `Show py-native Publish
Receipt` renders the verified TestPyPI/PyPI workflow run ids, install
commands, and package checks from `gemstone-rs py-native publish-receipt
--json`. `Validate py-native Shared Core Gate` runs `gemstone-rs py-native
check-all --json`, validates every checked-in py-native fixture in one command,
and renders a copyable downstream CI gate report.
`Show Example Commands` uses `gemstone-rs examples list --json` to show the
same curated example map that the CLI exposes, then lets you run a selected
example in a terminal, copy its command, or open the examples guide. This is
the Rust workbench equivalent of the gemstone-py examples launcher. From a
source checkout, the same examples can also be launched with
`gemstone-rs examples run <name>`.
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
full browser UI, while the side inspector renders structured setup checks,
live dictionary/class/protocol/method browsing, method source previews, project
profile freshness tables with Preview/Diff/Check/Generate buttons, Codegen
explain summaries, editable generated source, colorized diffs, BridgeRoot
identity/key/value summaries, and comparison status cards. Workbench buttons
can hand off to native VS Code commands for previewing wrappers, opening diffs,
opening config/profile/generated files, generating with confirmation, checking
project profiles, and opening docs. Generated wrapper preview/output responses
open in a webview editor with `Open Output File`, `Open Editable Draft`, and
`Save Edited Output`; saves stay inside the configured checkout and ask for
confirmation before writing. If the explorer is not running, the webview offers
Launch Explorer, Open Browser, and Copy URL fallbacks.
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
`Compare with gemstone-py` runs
`gemstone-rs compare gemstone-py --status --json` and renders the current
parity score, remaining batch count, next batch, and top gap without leaving
VS Code. `Show All Comparison Status` runs the aggregate status for every
comparison target, including the combined batch/hour estimate.

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
  Run Generated Mapping Example, and Open Generated Output.
- `Explorer` exposes Doctor, Verify Setup, Verify Live Setup, Verify Strict
  Setup, Eval Smalltalk, Show Example Commands, Show/Copy/Write Environment
  Template, Launch Explorer, Open Explorer Webview, Generate Explorer Auth
  Token, and Clear Explorer Auth Token.
- `Comparison` exposes the gemstone-py status report and the aggregate
  comparison status report from the same CLI JSON contract used by release
  verification.

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
