# Codegen Profile Schema

`gemstone-rs` project profile files make explorer and VS Code codegen workflows
repeatable. The default file name is:

```text
gemstone-rs.codegen-profiles.json
```

This repository includes a sample:

```text
examples/codegen/gemstone-rs.codegen-profiles.json
```

The JSON Schema is checked in at:

```text
schemas/gemstone-rs.codegen-profiles.schema.json
```

Validate the sample from a checkout:

```bash
cargo run -p gemstone-rs-cli -- profile sample
cargo run -p gemstone-rs-cli -- profile init gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile validate examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile validate --json examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile list examples/codegen/gemstone-rs.codegen-profiles.json
cargo run -p gemstone-rs-cli -- profile show default examples/codegen/gemstone-rs.codegen-profiles.json
```

For an installed CLI:

```bash
gemstone-rs profile sample
gemstone-rs profile init gemstone-rs.codegen-profiles.json
gemstone-rs profile validate gemstone-rs.codegen-profiles.json
gemstone-rs profile validate --json gemstone-rs.codegen-profiles.json
gemstone-rs profile list gemstone-rs.codegen-profiles.json
gemstone-rs profile show default gemstone-rs.codegen-profiles.json
```

Expected output:

```text
profile ok: examples/codegen/gemstone-rs.codegen-profiles.json (3 profiles: default, object-wrapper, bridge-mapping)
```

## Shape

```json
{
  "kind": "gemstone-rs-explorer-codegen-profiles",
  "version": 1,
  "profiles": [
    {
      "name": "default",
      "config": "examples/codegen/gemstone-rs.codegen",
      "root": "",
      "mapped": "BookingDraft",
      "className": "Object"
    }
  ]
}
```

Top-level fields:

- `kind` must be `gemstone-rs-explorer-codegen-profiles`.
- `version` must be `1`.
- `profiles` must be an array.

Profile fields:

- `name` is required, string-valued, non-empty, and unique.
- `config` is optional and string-valued.
- `root` is optional and string-valued.
- `mapped` is optional and string-valued.
- `className` is optional and string-valued.

Unknown top-level or profile fields are rejected. Non-string profile fields are
rejected.

## VS Code

Use `GemStone RS: Show Sample Project Profiles` to open the built-in sample in
an untitled JSON editor, `GemStone RS: Create Project Profiles` to write the
sample to a project file, and `GemStone RS: Validate Project Profiles` to run
the same CLI validator and show the result in the GemStone RS output panel.
`GemStone RS: List Project Profiles` and `GemStone RS: Show Project Profile`
render the parsed profile summary through the same CLI parser, so you can
check the active config/root/mapped/class fields without opening the explorer.
The workbench also contributes JSON validation for files named
`gemstone-rs.codegen-profiles.json`, using the packaged schema copy in
`vscode-gemstone-rs-workbench/schemas/`.

Without the extension, associate the schema with the profile file in VS Code:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["gemstone-rs.codegen-profiles.json"],
      "url": "./schemas/gemstone-rs.codegen-profiles.schema.json"
    }
  ]
}
```

## Explorer Safety

The explorer validates project profile JSON before saving. It also rejects
write paths containing `..` after URL decoding. Relative profile writes stay
inside the configured `--codegen-root`; absolute write targets require
`--allow-absolute-write-paths`.
