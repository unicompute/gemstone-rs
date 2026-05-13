# Part 4: Codegen, Explorer, and VS Code

The first three steps got Rust connected to GemStone/S, showed how values move
across the boundary, and gave you browser calls for dictionaries, classes,
protocols, methods, and source. That is enough to build tools. It is not yet the
pleasant path for an application team that wants checked-in Rust wrappers and a
repeatable workflow.

This is where codegen, the local explorer, and the VS Code workbench become
useful.

## Codegen Is a Contract, Not a Shortcut

The goal of gemstone-rs codegen is not to hide GemStone/S. The goal is to turn a
known set of GemStone classes and methods into reviewed Rust source files.

Start with a config:

```text
output = src/generated/gemstone_wrappers.rs
class = Object
method = Object>>printString | return=String | doc=Return the receiver printString.
method = Object>>class | return=Oop | doc=Return the receiver class.
```

Preview first:

```bash
cargo run -p gemstone-rs-cli -- codegen preview examples/codegen/gemstone-rs.codegen
```

Check before committing:

```bash
cargo run -p gemstone-rs-cli -- codegen check
```

Generate when the preview is what you want:

```bash
cargo run -p gemstone-rs-cli -- codegen generate examples/codegen/gemstone-rs.codegen
```

The useful habit is simple: preview, diff, check, generate, then review the
generated file like any other source file.

## Discover From a Live Stone

When you are connected to a live stone, the CLI can create a starting config:

```bash
cargo run -p gemstone-rs-cli -- codegen discover examples/codegen/object.codegen Object
```

That gives you a concrete list to prune. For a real project, the codegen config
should be small and deliberate. Start with the classes your Rust code actually
calls. Add more only when the application needs them.

## Use the Explorer for Fast Feedback

The explorer is a local HTTP app that sits on top of the same Rust API:

```bash
cargo run -p gemstone-rs-explorer -- --port 8787
```

When you want an extra local guard, especially before enabling writes, run it
with a token:

```bash
export GEMSTONE_RS_EXPLORER_TOKEN='replace-with-a-local-random-token'
cargo run -p gemstone-rs-explorer -- --port 8787 --auth-token-env GEMSTONE_RS_EXPLORER_TOKEN
```

Then add `?token=replace-with-a-local-random-token` to browser URLs, or send
`X-GemStone-RS-Token` from scripts.

Open these endpoints while developing:

```text
http://127.0.0.1:8787/
http://127.0.0.1:8787/api/status
http://127.0.0.1:8787/api/browse/dictionaries
http://127.0.0.1:8787/api/browse/classes?dictionary=UserGlobals
http://127.0.0.1:8787/api/browse/methods?class=Object&protocol=--%20all%20--
http://127.0.0.1:8787/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen
http://127.0.0.1:8787/api/codegen/check?config=examples/codegen/gemstone-rs.codegen
```

Keep it bound to `127.0.0.1`. Treat eval and write operations as explicit local
developer actions.

## Use VS Code When You Want a Guided Workflow

The VS Code workbench is intentionally a thin layer over the Rust CLI and local
explorer. It should not become a second GemStone client. It should make the
common commands easier to discover.

Useful commands:

```text
GemStone RS: Configure Workbench
GemStone RS: Verify Workbench Setup
GemStone RS: Launch Explorer
GemStone RS: Run Codegen Check
GemStone RS: Preview Codegen Wrappers
GemStone RS: Generate Codegen Wrappers
GemStone RS: Open Codegen Docs
```

The intended codegen loop in VS Code is:

1. Configure the CLI path, checkout path, explorer path, and GemStone
   environment.
2. Verify the setup report.
3. Browse the stone.
4. Preview generated wrappers.
5. Review the diff.
6. Generate wrappers.
7. Run the Rust tests.

## The Practical Finish Line

The first useful milestone is not a huge generated API. It is a tiny, tested
wrapper that proves the path:

```rust
let object = session.resolve("Object")?;
let text = session.perform_string(object, "printString", &[])?;
assert!(!text.is_empty());
```

Once that works, the rest of the workflow is mostly scale: more classes, more
methods, better names, better docs, and stronger tests.
