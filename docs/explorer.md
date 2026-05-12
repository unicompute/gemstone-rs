# Explorer Guide

`gemstone-rs-explorer` is a local-only HTTP explorer built on the Rust API. It
is useful for proving the browser, inspect, eval, and codegen surfaces before
embedding richer tooling in VS Code or another frontend.

## Install and Run

```bash
cargo install gemstone-rs-explorer
gemstone-rs-explorer --port 8787
```

From a checkout:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
```

Open:

```text
http://127.0.0.1:8787/
```

The home page is a small browser UI over the same JSON endpoints. It can:

- browse dictionaries, classes, protocols, methods, and source
- run doctor/status checks
- inspect BridgeRoot and list keys
- run codegen sample/discover/preview/diff/check/generate from an editable
  config path
- load and save the selected codegen config file; saving is still gated by
  `--allow-write`, accepts a POST body for larger configs, and validates the
  config before writing
- render generated wrapper source, generated mapping config, colored unified
  diff output, and a side-by-side diff in a dedicated detail pane
- remember the current browser fields locally across reloads
- put/remove BridgeRoot values with explicit string/symbol key policy and
  string/small-int/bool value policy when `--allow-write` is enabled

Screenshot:

![gemstone-rs explorer home](assets/explorer-home.png)

Refresh it with:

```bash
make screenshots
```

See [Screenshot Workflow](screenshots.md) for the repeatable capture process.

## Safety Defaults

The explorer:

- binds to `127.0.0.1` by default
- rejects non-loopback hosts
- starts read-only
- requires `--allow-eval` before workspace evaluation
- requires `--allow-write` before codegen writes
- reads GemStone credentials from the same `GS_*` environment as the CLI

Do not expose it publicly without adding authentication and transport security.

## Browse Endpoints

Run these from a second terminal:

```bash
curl -s http://127.0.0.1:8787/health
curl -s http://127.0.0.1:8787/api/config
curl -s http://127.0.0.1:8787/api/doctor
curl -s 'http://127.0.0.1:8787/api/doctor?live=1'
curl -s http://127.0.0.1:8787/api/status
curl -s http://127.0.0.1:8787/api/browse/dictionaries
curl -s 'http://127.0.0.1:8787/api/browse/classes?dictionary=UserGlobals'
curl -s 'http://127.0.0.1:8787/api/browse/protocols?class=Object&meta=0'
curl -s 'http://127.0.0.1:8787/api/browse/methods?class=Object&protocol=--%20all%20--&meta=0'
curl -s 'http://127.0.0.1:8787/api/browse/source?class=Object&selector=printString&meta=0'
curl -s 'http://127.0.0.1:8787/api/inspect?oop=20'
```

Expected shapes:

```json
{"status":"ok"}
{"host":"127.0.0.1","port":8787,"readOnly":true,"allowEval":false}
{"success":true,"environment":{"GS_PASSWORD":{"status":"set","masked":true}}}
{"connected":true,"sessionId":12345,"needsCommit":false,"inTransaction":false}
{"success":true,"dictionaries":["UserGlobals"]}
```

## Eval

Eval is disabled by default:

```bash
gemstone-rs-explorer --allow-eval
```

Then:

```bash
curl -s 'http://127.0.0.1:8787/api/eval?source=3%20%2B%204'
```

Expected:

```json
{"type":"smallInt","value":7}
```

## Codegen Endpoints

The home page includes a Codegen Workflow panel. Set `Config path`, optionally
enter a mapped Rust type and GemStone class, then use:

- `Load Config` to read the selected config file into the editor
- `Save Config` to POST the editor contents, validate them, and write the file
  when the explorer was started with `--allow-write`
- `Sample Config` to view the starter config format
- `Discover Mapping` to ask a live stone for a mapping config proposal
- `Preview` to inspect generated Rust wrappers without writing files
- `Diff` to compare generated output with the committed file; the detail pane
  shows both the exact unified diff and a side-by-side view for review
- `Check` to fail when generated output is stale
- `Generate` to write wrappers when the explorer was started with
  `--allow-write`

Read-only endpoints:

```bash
curl -s http://127.0.0.1:8787/api/codegen/sample
curl -s 'http://127.0.0.1:8787/api/codegen/config?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/check?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/discover-mapping?mapped=BookingDraft&class=Object'
```

Expected check result when generated output is current:

```json
{"success":true,"exists":true,"upToDate":true}
```

Write-gated endpoint:

```bash
gemstone-rs-explorer --allow-write
```

```bash
curl -s 'http://127.0.0.1:8787/api/codegen/generate?config=examples/codegen/gemstone-rs.codegen'
curl -s -X POST \
  --data-binary @examples/codegen/gemstone-rs.codegen \
  'http://127.0.0.1:8787/api/codegen/config/save?config=examples/codegen/draft.codegen'
```

Expected:

```json
{"success":true,"output":"examples/codegen/generated/gemstone_wrappers.rs","bytes":1234}
{"success":true,"config":"examples/codegen/draft.codegen","bytes":512}
```

## BridgeRoot Endpoints

BridgeRoot inspection and mapping-config preview use the same live session
configuration as the browser endpoints:

```bash
curl -s http://127.0.0.1:8787/api/bridge/root
curl -s http://127.0.0.1:8787/api/bridge/keys
curl -s 'http://127.0.0.1:8787/api/bridge/get?key=BookingDraft'
curl -s 'http://127.0.0.1:8787/api/bridge/get?key=BookingDraft&key_type=Symbol'
curl -s 'http://127.0.0.1:8787/api/bridge/mapping-config?mapped=BookingDraft'
```

The browser UI exposes `Bridge key type` and `Bridge value type` controls so
you can explicitly test string-key, symbol-key, string-value, small-int-value,
and bool-value mappings before encoding them in a reusable config file. It
persists the selected key, value, class, selector, config, and mapping fields in
browser local storage; use `Clear Saved Fields` to reset the page back to the
documented defaults.

Writes are disabled unless the explorer is started with `--allow-write`:

```bash
gemstone-rs-explorer --allow-write
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=WorkbenchDraft&value=hello'
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=WorkbenchCount&value=7&value_type=SmallInt'
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=WorkbenchApproved&value=true&value_type=Bool'
curl -s 'http://127.0.0.1:8787/api/bridge/remove?key=WorkbenchDraft'
```

Expected shapes:

```json
{"success":true,"name":"GemStoneRsBridgeRoot","oop":1234,"identityId":1}
{"success":true,"root":"GemStoneRsBridgeRoot","keys":[{"printString":"BookingDraft"}]}
{"oop":5678,"classOop":9012,"printString":"aDictionary(...)"}
{"success":true,"config":"mapped = BookingDraft ..."}
{"success":true,"root":"GemStoneRsBridgeRoot","key":"WorkbenchDraft","keyType":"String","valueType":"String","oop":3456}
```

## Product Direction

The explorer should remain a separate binary crate. The core `gemstone-rs`
library stays focused on safe GemStone API access, while the explorer proves
higher-level browsing and codegen workflows.

Good next steps:

- file-picker style config selection
- deeper VS Code webview integration
