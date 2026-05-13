# Local Explorer Example

This example proves `gemstone-rs-explorer`, the local HTTP explorer built on
top of `gemstone-rs`.

## Start

From a source checkout:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
cargo run -p gemstone-rs-explorer -- --env-file .env.gemstone-rs --port 8787
cargo run -p gemstone-rs-explorer -- --port 8787 --codegen-root /path/to/gemstone-rs
```

From installed tools:

```bash
cargo install gemstone-rs-explorer
gemstone-rs-explorer --port 8787
gemstone-rs-explorer --env-file .env.gemstone-rs --port 8787
```

Open:

```text
http://127.0.0.1:8787/
```

Screenshot:

![gemstone-rs explorer home](../../docs/assets/explorer-home.png)

## Verify Read-Only Endpoints

In a second terminal:

```bash
curl -s http://127.0.0.1:8787/health
curl -s http://127.0.0.1:8787/api/config
curl -s 'http://127.0.0.1:8787/api/setup/assistant?config=examples/codegen/gemstone-rs.codegen&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json'
curl -s http://127.0.0.1:8787/api/status
curl -s http://127.0.0.1:8787/api/browse/dictionaries
curl -s 'http://127.0.0.1:8787/api/browse/protocols?class=Object'
curl -s 'http://127.0.0.1:8787/api/browse/methods?class=Object&protocol=--%20all%20--'
curl -s 'http://127.0.0.1:8787/api/browse/source?class=Object&selector=printString'
curl -s 'http://127.0.0.1:8787/api/inspect?oop=20'
```

The explorer should bind only to loopback and report `readOnly: true` from
`/api/config`.

Expected response shapes:

```json
{"status":"ok"}
{"host":"127.0.0.1","port":8787,"readOnly":true,"allowEval":false}
{"connected":true,"sessionId":12345,"needsCommit":false,"inTransaction":false}
{"success":true,"dictionaries":["UserGlobals"]}
```

## Codegen Endpoints

```bash
curl -s http://127.0.0.1:8787/api/codegen/sample
curl -s 'http://127.0.0.1:8787/api/codegen/configs?root=.'
curl -s 'http://127.0.0.1:8787/api/codegen/profiles?profile_file=gemstone-rs.codegen-profiles.json'
curl -s 'http://127.0.0.1:8787/api/codegen/explain-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json'
curl -s 'http://127.0.0.1:8787/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/preview-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json'
curl -s 'http://127.0.0.1:8787/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/diff-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json'
curl -s 'http://127.0.0.1:8787/api/codegen/check?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/check-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json'
```

Expected check result when generated output is current:

```json
{"success":true,"root":".","configs":["examples/codegen/gemstone-rs.codegen"]}
{"success":true,"exists":true,"upToDate":true}
```

In the browser UI, use `Profile name` and `Save Profile` to save the current
config root, config path, mapped Rust type, and GemStone class as a local
profile. `Saved profiles` lets you switch back to that setup later without
retyping paths. Use `Export Profile` to copy a versioned JSON payload from
`Profile JSON`; paste that JSON into another explorer and use `Import Profile`
to merge it into that browser's saved profiles. Use `Load Project Profiles` to
read `gemstone-rs.codegen-profiles.json` from the codegen root, and `Save
Project Profiles` under `--allow-write` to commit the browser's saved profiles
back to that file. The server rejects invalid project profile schemas before
writing, including unsupported fields, duplicate names, non-string profile
fields, and traversal attempts in write paths. The browser reports which
imported profiles are new, replaced, or unchanged.

Validate the committed sample profile file without starting the explorer:

```bash
cargo run -p gemstone-rs-cli -- profile sample
cargo run -p gemstone-rs-cli -- profile init gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile validate examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile validate --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile list examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile show default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile resolve default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile check examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile check --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen preview-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen diff-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen check-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen explain-profile --json default examples/codegen/gemstone-rs.codegen-profiles.json
```

Generation is disabled until you opt in:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787 --allow-write
curl -s 'http://127.0.0.1:8787/api/codegen/generate?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/generate-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json'
```

The same write opt-in enables BridgeRoot edit endpoints:

```bash
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=ExplorerDraft&value=hello'
curl -s 'http://127.0.0.1:8787/api/bridge/put?key=ExplorerCount&value=7&value_type=SmallInt'
curl -s 'http://127.0.0.1:8787/api/bridge/remove?key=ExplorerDraft'
```

## Eval Is Opt-In

Eval is disabled by default. To test it:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787 --allow-eval
curl -s 'http://127.0.0.1:8787/api/eval?source=3%20%2B%204'
```

Do not expose this service publicly without authentication and TLS.
