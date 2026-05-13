# gemstone-rs-explorer

Local-only web explorer for GemStone/S using `gemstone-rs`.

This is an early proving-ground crate. It intentionally uses only the Rust
standard library, binds to `127.0.0.1` by default, starts read-only, and requires
explicit `--allow-eval` for arbitrary workspace evaluation.

Run it with:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
```

Add a local auth token when you want a second guard around the loopback app,
especially before starting with `--allow-write`:

```bash
export GEMSTONE_RS_EXPLORER_TOKEN='replace-with-a-local-random-token'
cargo run -p gemstone-rs-explorer -- --port 8787 --auth-token-env GEMSTONE_RS_EXPLORER_TOKEN
```

Then pass `token=` for browser links or use a header from scripts:

```bash
curl -s 'http://127.0.0.1:8787/api/config?token=replace-with-a-local-random-token'
curl -s -H 'X-GemStone-RS-Token: replace-with-a-local-random-token' \
  http://127.0.0.1:8787/api/status
```

Then open:

```text
http://127.0.0.1:8787/
http://127.0.0.1:8787/api/status
http://127.0.0.1:8787/api/browse/dictionaries
http://127.0.0.1:8787/api/browse/classes?dictionary=UserGlobals
http://127.0.0.1:8787/api/browse/protocols?class=Object
http://127.0.0.1:8787/api/browse/methods?class=Object&protocol=--%20all%20--
http://127.0.0.1:8787/api/browse/source?class=Object
http://127.0.0.1:8787/api/codegen/sample
http://127.0.0.1:8787/api/codegen/configs?root=.
http://127.0.0.1:8787/api/codegen/profiles?profile_file=gemstone-rs.codegen-profiles.json
http://127.0.0.1:8787/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/check?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/bridge/root
http://127.0.0.1:8787/api/bridge/keys
http://127.0.0.1:8787/api/inspect?oop=20
```

Enable eval explicitly:

```bash
cargo run -p gemstone-rs-explorer -- --allow-eval
```

```text
http://127.0.0.1:8787/api/eval?source=3%20%2B%204
```

Planned next steps:

- add a richer frontend once the API endpoints are stable
- keep VS Code webview and CLI workflows aligned with the explorer API

Browse endpoints are read-only. They use the active user's symbol list and the
same dictionary/class lookup convention as the Python GemStone database
explorer:

```text
GET /api/browse/dictionaries
GET /api/browse/classes?dictionary=UserGlobals
GET /api/browse/protocols?class=Object&meta=0
GET /api/browse/methods?class=Object&protocol=--%20all%20--&meta=0
GET /api/browse/source?class=Object&selector=printString&meta=0
GET /api/codegen/sample
GET /api/codegen/configs?root=.
GET /api/codegen/profiles?profile_file=gemstone-rs.codegen-profiles.json
GET /api/codegen/profiles/check?profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
GET /api/codegen/preview?config=examples/codegen/gemstone-rs.codegen
GET /api/codegen/preview-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
GET /api/codegen/diff?config=examples/codegen/gemstone-rs.codegen
GET /api/codegen/diff-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
GET /api/codegen/check?config=examples/codegen/gemstone-rs.codegen
GET /api/codegen/check-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
```

Codegen generation is write-gated. Start with `--allow-write` before using:

```text
GET /api/codegen/generate?config=examples/codegen/gemstone-rs.codegen
GET /api/codegen/generate-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json
POST /api/codegen/profiles/save?profile_file=gemstone-rs.codegen-profiles.json
GET /api/bridge/put?key=ExplorerDraft&value=hello
GET /api/bridge/put?key=ExplorerCount&value=7&value_type=SmallInt
GET /api/bridge/remove?key=ExplorerDraft
```

Project profile saves are schema-validated through the shared
`gemstone-rs profile validate` rules. Relative `config=` and `profile_file=`
write targets stay under the configured codegen root, `..` traversal is
rejected after URL decoding, and absolute write targets require
`--allow-absolute-write-paths`.

The home-page UI asks for confirmation before BridgeRoot writes, env-file
writes, profile saves, config saves, and codegen generate actions. The server
still enforces the final write policy: write endpoints return `403` unless the
process was started with `--allow-write`.
