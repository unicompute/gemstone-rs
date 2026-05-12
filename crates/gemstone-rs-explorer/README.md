# gemstone-rs-explorer

Local-only web explorer for GemStone/S using `gemstone-rs`.

This is an early proving-ground crate. It intentionally uses only the Rust
standard library, binds to `127.0.0.1` by default, starts read-only, and requires
explicit `--allow-eval` for arbitrary workspace evaluation.

Run it with:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
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

- preview Rust/Python codegen output
- compare generated output against files
- add a richer frontend once the API endpoints are stable

Browse endpoints are read-only. They use the active user's symbol list and the
same dictionary/class lookup convention as the Python GemStone database
explorer:

```text
GET /api/browse/dictionaries
GET /api/browse/classes?dictionary=UserGlobals
GET /api/browse/protocols?class=Object&meta=0
GET /api/browse/methods?class=Object&protocol=--%20all%20--&meta=0
GET /api/browse/source?class=Object&selector=printString&meta=0
```
