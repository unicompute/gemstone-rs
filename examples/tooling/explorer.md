# Local Explorer Example

This example proves `gemstone-rs-explorer`, the local HTTP explorer built on
top of `gemstone-rs`.

## Start

From a source checkout:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
```

From installed tools:

```bash
cargo install gemstone-rs-explorer
gemstone-rs-explorer --port 8787
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
curl -s 'http://127.0.0.1:8787/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen'
curl -s 'http://127.0.0.1:8787/api/codegen/check?config=examples/codegen/gemstone-rs.codegen'
```

Expected check result when generated output is current:

```json
{"success":true,"exists":true,"upToDate":true}
```

Generation is disabled until you opt in:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787 --allow-write
curl -s 'http://127.0.0.1:8787/api/codegen/generate?config=examples/codegen/gemstone-rs.codegen'
```

## Eval Is Opt-In

Eval is disabled by default. To test it:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787 --allow-eval
curl -s 'http://127.0.0.1:8787/api/eval?source=3%20%2B%204'
```

Do not expose this service publicly without authentication and TLS.
