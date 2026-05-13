const assert = require("assert");
const childProcess = require("child_process");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");

const schemaNames = [
  "gemstone-rs.codegen.schema.json",
  "gemstone-rs.codegen-explain.schema.json",
  "gemstone-rs.codegen-profiles.schema.json",
  "gemstone-rs.profile-check.schema.json",
];

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

for (const schemaName of schemaNames) {
  const schema = readJson(path.join("schemas", schemaName));
  assert.strictEqual(schema.$schema, "https://json-schema.org/draft/2020-12/schema");
  assert(schema.$id.includes(schemaName), `${schemaName} $id should include file name`);

  const packaged = readJson(path.join("vscode-gemstone-rs-workbench", "schemas", schemaName));
  assert.deepStrictEqual(packaged, schema, `${schemaName} packaged copy is stale`);
}

const explainOutput = childProcess.execFileSync(
  "cargo",
  [
    "run",
    "-p",
    "gemstone-rs-cli",
    "--",
    "codegen",
    "explain",
    "--json",
    "examples/codegen/gemstone-rs.codegen",
  ],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);

assertExplain(JSON.parse(lastJsonLine(explainOutput)));
const profileExplainOutput = childProcess.execFileSync(
  "cargo",
  [
    "run",
    "-p",
    "gemstone-rs-cli",
    "--",
    "codegen",
    "explain-profile",
    "--json",
    "default",
    "examples/codegen/gemstone-rs.codegen-profiles.json",
  ],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
assertExplain(JSON.parse(lastJsonLine(profileExplainOutput)));
const profileCheckOutput = childProcess.execFileSync(
  "cargo",
  [
    "run",
    "-p",
    "gemstone-rs-cli",
    "--",
    "profile",
    "check",
    "--json",
    "examples/codegen/gemstone-rs.codegen-profiles.json",
  ],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
assertProfileCheck(JSON.parse(lastJsonLine(profileCheckOutput)));
assertProfiles(readJson("examples/codegen/gemstone-rs.codegen-profiles.json"));

console.log("gemstone-rs codegen schema checks passed");

function lastJsonLine(output) {
  const lines = output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const line = lines.reverse().find((candidate) => candidate.startsWith("{"));
  assert(line, "expected a JSON line from codegen explain --json");
  return line;
}

function assertExplain(value) {
  assert.strictEqual(typeof value.output, "string");
  assert(Array.isArray(value.testStubs));
  assert(Array.isArray(value.classes));
  assert(Array.isArray(value.mapped));

  for (const cls of value.classes) {
    assert.strictEqual(typeof cls.name, "string");
    assert.strictEqual(typeof cls.dictionary, "string");
    assert.strictEqual(typeof cls.className, "string");
    assert.strictEqual(typeof cls.meta, "boolean");
    assert(Array.isArray(cls.methods));
    for (const method of cls.methods) {
      assert.strictEqual(typeof method.selector, "string");
      assert(Array.isArray(method.args));
      assert.strictEqual(typeof method.return, "string");
      assert(method.doc === null || typeof method.doc === "string");
    }
  }

  for (const mapped of value.mapped) {
    assert.strictEqual(typeof mapped.name, "string");
    assert(mapped.doc === null || typeof mapped.doc === "string");
    assert(Array.isArray(mapped.fields));
    for (const field of mapped.fields) {
      assert.strictEqual(typeof field.name, "string");
      assert.strictEqual(typeof field.type, "string");
      assert.strictEqual(typeof field.key, "string");
      assert(["String", "Symbol"].includes(field.keyType));
    }
  }
}

function assertProfiles(value) {
  assert.strictEqual(value.kind, "gemstone-rs-explorer-codegen-profiles");
  assert.strictEqual(value.version, 1);
  assert(Array.isArray(value.profiles));
  const names = new Set();
  for (const [index, profile] of value.profiles.entries()) {
    assert.strictEqual(typeof profile.name, "string", `profiles[${index}].name`);
    assert(profile.name.trim(), `profiles[${index}].name must not be empty`);
    assert(!names.has(profile.name), `duplicate profile ${profile.name}`);
    names.add(profile.name);
    for (const key of ["config", "root", "mapped", "className"]) {
      if (profile[key] !== undefined) {
        assert.strictEqual(typeof profile[key], "string", `profiles[${index}].${key}`);
      }
    }
  }
}

function assertProfileCheck(value) {
  assert.strictEqual(value.success, true);
  assert.strictEqual(typeof value.ok, "boolean");
  assert.strictEqual(typeof value.path, "string");
  assert.strictEqual(typeof value.profileFile, "string");
  assert.strictEqual(typeof value.profileCount, "number");
  assert.strictEqual(typeof value.okCount, "number");
  assert.strictEqual(typeof value.staleCount, "number");
  assert.strictEqual(typeof value.errorCount, "number");
  assert(Array.isArray(value.profiles));
  assert.strictEqual(value.profileCount, value.profiles.length);
  for (const [index, profile] of value.profiles.entries()) {
    assert.strictEqual(typeof profile.name, "string", `profiles[${index}].name`);
    assert.strictEqual(typeof profile.ok, "boolean", `profiles[${index}].ok`);
    assert(
      profile.config === null || typeof profile.config === "string",
      `profiles[${index}].config`
    );
    assert(
      profile.output === null || typeof profile.output === "string",
      `profiles[${index}].output`
    );
    assert.strictEqual(typeof profile.exists, "boolean", `profiles[${index}].exists`);
    assert.strictEqual(typeof profile.upToDate, "boolean", `profiles[${index}].upToDate`);
    assert(
      profile.error === null || typeof profile.error === "string",
      `profiles[${index}].error`
    );
  }
}
