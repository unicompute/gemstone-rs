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

- browse classes, methods, protocols, and source with structured JSON
- preview Rust/Python codegen output
- compare generated output against files
- add a richer frontend once the API endpoints are stable
