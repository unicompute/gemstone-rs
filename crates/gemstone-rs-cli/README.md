# gemstone-rs-cli

Command line tools for `gemstone-rs`.

The binary name is `gemstone-rs`:

```bash
cargo run -p gemstone-rs-cli -- doctor
cargo run -p gemstone-rs-cli -- doctor --live
cargo run -p gemstone-rs-cli -- doctor --strict
cargo run -p gemstone-rs-cli -- doctor --env-file .env.gemstone-rs --live
cargo run -p gemstone-rs-cli -- doctor --json
cargo run -p gemstone-rs-cli -- hello
cargo run -p gemstone-rs-cli -- hello --json
cargo run -p gemstone-rs-cli -- compare gemstone-py
cargo run -p gemstone-rs-cli -- compare gemstone-py --json
cargo run -p gemstone-rs-cli -- compare gemstone-py --gaps
cargo run -p gemstone-rs-cli -- compare gemstone-py --gaps --json
cargo run -p gemstone-rs-cli -- env sample
cargo run -p gemstone-rs-cli -- env write .env.gemstone-rs
cargo run -p gemstone-rs-cli -- examples list
cargo run -p gemstone-rs-cli -- examples map
cargo run -p gemstone-rs-cli -- examples show quickstart
cargo run -p gemstone-rs-cli -- examples run codegen_preview --dry-run
cargo run -p gemstone-rs-cli -- examples scaffold quickstart /tmp/gemstone-rs-quickstart --force
cargo run -p gemstone-rs-cli -- examples scaffold http_service /tmp/gemstone-rs-http-service --force
cargo run -p gemstone-rs-cli -- eval --env-file .env.gemstone-rs "3 + 4"
cargo run -p gemstone-rs-cli -- browse dictionaries
cargo run -p gemstone-rs-cli -- browse classes UserGlobals
cargo run -p gemstone-rs-cli -- browse protocols Object
cargo run -p gemstone-rs-cli -- browse methods Object "-- all --"
cargo run -p gemstone-rs-cli -- browse source Object printString
cargo run -p gemstone-rs-cli -- inspect oop 20
cargo run -p gemstone-rs-cli -- bridge root
cargo run -p gemstone-rs-cli -- bridge keys
cargo run -p gemstone-rs-cli -- bridge get BookingDraft --symbol
cargo run -p gemstone-rs-cli -- bridge inspect BookingDraft --symbol
cargo run -p gemstone-rs-cli -- bridge put-string WorkbenchDraft "hello from Rust"
cargo run -p gemstone-rs-cli -- bridge put-symbol WorkbenchState ready
cargo run -p gemstone-rs-cli -- bridge put-smallint WorkbenchCount 7
cargo run -p gemstone-rs-cli -- bridge put-bool WorkbenchReady true
cargo run -p gemstone-rs-cli -- bridge sample-config BookingDraft
cargo run -p gemstone-rs-cli -- codegen init
cargo run -p gemstone-rs-cli -- codegen preview examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen diff examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen check examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen explain examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen explain --json examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen generate examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen check-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- codegen discover examples/codegen/discovered.codegen Object
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
cargo run -p gemstone-rs-cli -- codegen generate-profile default examples/codegen/gemstone-rs.codegen-profiles.json
```

The CLI uses the same GemStone environment variables as the library:

```bash
export GS_LIB=/opt/gemstone/product/lib
export GS_STONE=gs64stone
export GS_USERNAME=DataCurator
export GS_PASSWORD=swordfish
```

Use `gemstone-rs env sample` to print a copy-pasteable shell export template.
It reuses current non-secret values when they are set and always prints password
placeholders instead of real secrets. Use `gemstone-rs env write` to write the
same template to `.env.gemstone-rs`; pass a path to choose another file, and
pass `--force` to overwrite an existing file. `doctor --env-file` and
`eval --env-file` load that file for one command, which is useful in CI and in
shells where you do not want to source the file globally.

`hello` is the fastest no-GemStone sanity check. It mirrors
`gemstone-examples hello` from gemstone-py and prints the CLI version, target
OS, target architecture, and executable path. `compare gemstone-py` prints a
compact comparison summary; add `--gaps` for the prioritized catch-up report
and `--json` when an editor, CI job, or release script needs structured output.

`doctor` is the first command to run on a new machine. Without `--live`, it
checks environment and GCI library resolution, including whether `libgcirpc`
came from explicit config, `GS_LIB_PATH`, `GS_LIB`, or `GEMSTONE/lib`, plus the
path or directory searched. With `--live`, it logs in and asserts `3 + 4 == 7`.
Failures include remediation hints for missing credentials, dynamic library
loading, and stone connectivity. `--strict` is useful in CI because it fails
when the stone name or GCI library source is only coming from defaults. Add
`--json` for scripts, CI, and editor integrations.

The `examples` commands provide a gemstone-py-style curated example index from
the installed CLI:

```bash
gemstone-rs examples list
gemstone-rs examples list --json
gemstone-rs examples map
gemstone-rs examples map --json
gemstone-rs examples show quickstart
gemstone-rs examples run codegen_preview --dry-run
```

`list` and `show` tell you which Cargo example to run, whether it needs a live
stone, and what API surface it proves. `run` launches the selected Cargo
example when you are in a `gemstone-rs` source checkout. Use `--dry-run` in CI
or docs checks to print the exact command without compiling or connecting to a
stone. `scaffold <name> [path]` writes a standalone Cargo project from an
installed template; the first templates are `quickstart` and `http_service`.
`map` is the gemstone-rs equivalent of `gemstone-examples plan3-map`: it ties
Rust crates, examples, docs, and gemstone-py reference points together for each
feature stream.

The `bridge` commands inspect the default `GemStoneRsBridgeRoot` dictionary:

```bash
gemstone-rs bridge root
gemstone-rs bridge keys
gemstone-rs bridge get BookingDraft --symbol
gemstone-rs bridge inspect BookingDraft --symbol
gemstone-rs bridge put-string WorkbenchDraft "hello from Rust"
gemstone-rs bridge put-symbol WorkbenchState ready
gemstone-rs bridge put-smallint WorkbenchCount 7
gemstone-rs bridge put-bool WorkbenchReady true
gemstone-rs bridge remove WorkbenchDraft
gemstone-rs bridge sample-config BookingDraft
```

`bridge put-string`, `bridge put-symbol`, `bridge put-smallint`, and
`bridge put-bool` commit common scalar values without spelling out `--type`.
The generic `bridge put` remains available when scripts prefer
`--type String|Symbol|SmallInt|Bool`. Use `--symbol` or `--key-type Symbol`
when the BridgeRoot key is a GemStone Symbol instead of a String. Scripts can
also use equals-style options such as `--root=DemoRoot`,
`--key-type=Symbol`, and `--type=SmallInt`.

`codegen diff` previews generated changes before writing. `codegen check` is
suitable for CI. In this repository it verifies:

```bash
cargo run -p gemstone-rs-cli -- codegen check examples/codegen/gemstone-rs.codegen
cargo run -p gemstone-rs-cli -- codegen check-profile default examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile validate examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile validate --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile list --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile show default --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile resolve default --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile check examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile check --json examples/codegen/gemstone-rs.codegen-profiles.json
```
