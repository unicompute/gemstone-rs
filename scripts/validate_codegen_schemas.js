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
  "gemstone-rs.compare.schema.json",
  "gemstone-rs.py-native.schema.json",
  "gemstone-rs.py-native-samples.schema.json",
  "gemstone-rs.py-native-smoke.schema.json",
  "gemstone-rs.py-native-migration.schema.json",
  "gemstone-rs.py-native-compat.schema.json",
  "gemstone-rs.py-native-conformance.schema.json",
  "gemstone-rs.py-native-handoff.schema.json",
  "gemstone-rs.py-native-publish-receipt.schema.json",
  "gemstone-rs.py-native-check-all.schema.json",
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
const pyNativeFixture = readJson("examples/py-native/gemstone-rs.py-native.json");
assertPyNativeCapabilities(pyNativeFixture);
const pyNativeOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "capabilities", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
const pyNativeCapabilities = JSON.parse(lastJsonLine(pyNativeOutput));
assertPyNativeCapabilities(pyNativeCapabilities);
assert.deepStrictEqual(
  pyNativeCapabilities,
  pyNativeFixture,
  "py-native capabilities output drifted from examples/py-native/gemstone-rs.py-native.json"
);
const pyNativeSamplesFixture = readJson("examples/py-native/gemstone-rs.py-native-samples.json");
assertPyNativeSamples(pyNativeSamplesFixture);
const pyNativeSamplesOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "samples", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
const pyNativeSamples = JSON.parse(lastJsonLine(pyNativeSamplesOutput));
assertPyNativeSamples(pyNativeSamples);
assert.deepStrictEqual(
  pyNativeSamples,
  pyNativeSamplesFixture,
  "py-native samples output drifted from examples/py-native/gemstone-rs.py-native-samples.json"
);
const pyNativeSmokeFixture = readJson("examples/py-native/gemstone-rs.py-native-smoke.json");
assertPyNativeSmoke(pyNativeSmokeFixture, { dryRun: true });
const pyNativeSmokeOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "smoke", "--dry-run", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
const pyNativeSmoke = JSON.parse(lastJsonLine(pyNativeSmokeOutput));
assertPyNativeSmoke(pyNativeSmoke, { dryRun: true });
assert.deepStrictEqual(
  pyNativeSmoke,
  pyNativeSmokeFixture,
  "py-native smoke output drifted from examples/py-native/gemstone-rs.py-native-smoke.json"
);
const pyNativeMigrationOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "migration", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
assertPyNativeMigration(JSON.parse(lastJsonLine(pyNativeMigrationOutput)));
const pyNativeCompatibilityFixture = readJson("examples/py-native/gemstone-rs.py-native-compat.json");
assertPyNativeCompatibility(pyNativeCompatibilityFixture);
const pyNativeCompatibilityOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "compatibility", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
const pyNativeCompatibility = JSON.parse(lastJsonLine(pyNativeCompatibilityOutput));
assertPyNativeCompatibility(pyNativeCompatibility);
assert.deepStrictEqual(
  pyNativeCompatibility,
  pyNativeCompatibilityFixture,
  "py-native compatibility output drifted from examples/py-native/gemstone-rs.py-native-compat.json"
);
const pyNativeConformanceFixture = readJson("examples/py-native/gemstone-rs.py-native-conformance.json");
assertPyNativeConformance(pyNativeConformanceFixture);
const pyNativeConformanceOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "conformance", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
const pyNativeConformance = JSON.parse(lastJsonLine(pyNativeConformanceOutput));
assertPyNativeConformance(pyNativeConformance);
assert.deepStrictEqual(
  pyNativeConformance,
  pyNativeConformanceFixture,
  "py-native conformance output drifted from examples/py-native/gemstone-rs.py-native-conformance.json"
);
const pyNativeHandoffFixture = readJson("examples/py-native/gemstone-rs.py-native-handoff.json");
assertPyNativeHandoff(pyNativeHandoffFixture);
const pyNativeHandoffOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "handoff", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
const pyNativeHandoff = JSON.parse(lastJsonLine(pyNativeHandoffOutput));
assertPyNativeHandoff(pyNativeHandoff);
assert.deepStrictEqual(
  pyNativeHandoff,
  pyNativeHandoffFixture,
  "py-native handoff output drifted from examples/py-native/gemstone-rs.py-native-handoff.json"
);
const pyNativePublishReceiptFixture = readJson(
  "examples/py-native/gemstone-rs.py-native-publish-receipt.json"
);
assertPyNativePublishReceipt(pyNativePublishReceiptFixture);
const pyNativePublishReceiptOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "publish-receipt", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
const pyNativePublishReceipt = JSON.parse(lastJsonLine(pyNativePublishReceiptOutput));
assertPyNativePublishReceipt(pyNativePublishReceipt);
assert.deepStrictEqual(
  pyNativePublishReceipt,
  pyNativePublishReceiptFixture,
  "py-native publish receipt output drifted from examples/py-native/gemstone-rs.py-native-publish-receipt.json"
);
const pyNativeCheckAllOutput = childProcess.execFileSync(
  "cargo",
  ["run", "-p", "gemstone-rs-cli", "--", "py-native", "check-all", "--json"],
  {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  }
);
assertPyNativeCheckAll(JSON.parse(lastJsonLine(pyNativeCheckAllOutput)));

for (const args of [
  ["compare", "gemstone-py", "--json"],
  ["compare", "gemstone-py", "--status", "--json"],
  ["compare", "gemstone-py", "--scorecard", "--json"],
  ["compare", "gemstone-py", "--parity", "--json"],
  ["compare", "all", "--json"],
  ["compare", "all", "--status", "--json"],
  ["compare", "all", "--scorecard", "--json"],
  ["compare", "all", "--parity", "--json"],
  ["compare", "all", "--gaps", "--json"],
  ["compare", "all", "--next", "--json"],
  ["compare", "all", "--totals", "--json"],
  ["compare", "all", "--batches", "--json"],
]) {
  assertCompare(
    JSON.parse(lastJsonLine(runGemstoneRs(args))),
    `gemstone-rs ${args.join(" ")}`
  );
}

console.log("gemstone-rs codegen, comparison, and py-native schema checks passed");

function runGemstoneRs(args) {
  return childProcess.execFileSync("cargo", ["run", "-p", "gemstone-rs-cli", "--", ...args], {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  });
}

function lastJsonLine(output) {
  const lines = output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const line = lines.reverse().find((candidate) => candidate.startsWith("{"));
  assert(line, "expected a JSON line from CLI command");
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
      assert(Array.isArray(method.argTypes));
      assert(Array.isArray(method.arguments));
      assert.strictEqual(method.args.length, method.argTypes.length);
      assert.strictEqual(method.args.length, method.arguments.length);
      for (const arg of method.arguments) {
        assert.strictEqual(typeof arg.name, "string");
        assert(["Oop", "String", "Symbol", "SmallInt", "Bool"].includes(arg.type));
        assert.strictEqual(typeof arg.rustType, "string");
      }
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

function assertPyNativeCapabilities(value) {
  assert.strictEqual(value.name, "gemstone-py-native adapter contract");
  assert.strictEqual(typeof value.contractVersion, "number");
  assert(value.contractVersion >= 1);
  assert.strictEqual(typeof value.threading, "string");
  assert(value.threading.includes("non-Send/non-Sync"));
  assertStringSet(
    value.operations,
    [
      "login",
      "logout",
      "eval",
      "eval_oop",
      "execute",
      "resolve",
      "value_to_oop",
      "perform",
      "new_string",
      "new_symbol",
      "fetch_string",
      "global_get",
      "global_put",
      "commit",
      "abort",
      "needs_commit",
      "in_transaction",
      "add_to_export_set",
      "remove_from_export_set",
    ],
    "py-native.operations"
  );
  assertStringSet(
    value.valueKinds,
    ["nil", "bool", "smallInt", "char", "string", "symbol", "oop"],
    "py-native.valueKinds"
  );
  assertStringSet(
    value.errorKinds,
    [
      "gci",
      "missingEnvironment",
      "missingConfig",
      "nul",
      "notLoggedIn",
      "gemStone",
      "illegalOop",
      "unexpectedType",
      "mapping",
      "workerStopped",
      "workerPanicked",
      "negativeSize",
      "argumentCountTooLarge",
    ],
    "py-native.errorKinds"
  );
  assert(value.oopConstants && typeof value.oopConstants === "object");
  for (const field of ["nil", "true", "false", "smallint7", "charA"]) {
    assert.strictEqual(
      typeof value.oopConstants[field],
      "number",
      `py-native.oopConstants.${field}`
    );
  }
}

function assertPyNativeSmoke(value, options = {}) {
  assert.strictEqual(typeof value.ok, "boolean", "py-native-smoke.ok");
  assert.strictEqual(typeof value.dryRun, "boolean", "py-native-smoke.dryRun");
  if (options.dryRun !== undefined) {
    assert.strictEqual(value.dryRun, options.dryRun, "py-native-smoke.dryRun expected value");
  }
  assert.strictEqual(typeof value.contractVersion, "number", "py-native-smoke.contractVersion");
  assert(value.contractVersion >= 1, "py-native-smoke.contractVersion");
  assert(Array.isArray(value.steps), "py-native-smoke.steps");
  assert(value.steps.length > 0, "py-native-smoke.steps should not be empty");

  const names = new Set();
  for (const [index, step] of value.steps.entries()) {
    assert.strictEqual(typeof step.name, "string", `py-native-smoke.steps[${index}].name`);
    assert(!names.has(step.name), `py-native-smoke.steps duplicate ${step.name}`);
    names.add(step.name);
    assert.strictEqual(typeof step.ok, "boolean", `py-native-smoke.steps[${index}].ok`);
    assert.strictEqual(typeof step.detail, "string", `py-native-smoke.steps[${index}].detail`);
  }
  for (const required of [
    "capabilities",
    "oop_constants",
    "value_conversion",
    "config_error_mapping",
    "structured_error_mapping",
  ]) {
    assert(names.has(required), `py-native-smoke.steps missing ${required}`);
  }
}

function assertPyNativeMigration(value) {
  assert.strictEqual(typeof value.contractVersion, "number", "py-native-migration.contractVersion");
  assert(value.contractVersion >= 1, "py-native-migration.contractVersion");
  assert.strictEqual(value.targetPackage, "gemstone-py-native", "py-native-migration.targetPackage");
  assert.strictEqual(typeof value.status, "string", "py-native-migration.status");
  assert.strictEqual(typeof value.doneCount, "number", "py-native-migration.doneCount");
  assert.strictEqual(typeof value.pendingCount, "number", "py-native-migration.pendingCount");
  assert(Array.isArray(value.steps), "py-native-migration.steps");
  assert(value.steps.length > 0, "py-native-migration.steps should not be empty");

  const ids = new Set();
  for (const [index, step] of value.steps.entries()) {
    assert.strictEqual(typeof step.id, "string", `py-native-migration.steps[${index}].id`);
    assert(!ids.has(step.id), `py-native-migration.steps duplicate ${step.id}`);
    ids.add(step.id);
    assert.strictEqual(typeof step.title, "string", `py-native-migration.steps[${index}].title`);
    assert(["done", "pending"].includes(step.status), `py-native-migration.steps[${index}].status`);
    assert.strictEqual(typeof step.detail, "string", `py-native-migration.steps[${index}].detail`);
    assert.strictEqual(typeof step.verify, "string", `py-native-migration.steps[${index}].verify`);
  }

  for (const required of [
    "scaffold_pyo3_adapter",
    "wrap_py_native_session",
    "preserve_python_api",
    "live_backend_smoke",
    "publish_wheels",
  ]) {
    assert(ids.has(required), `py-native-migration.steps missing ${required}`);
  }
}

function assertPyNativeCompatibility(value) {
  assert.strictEqual(typeof value.contractVersion, "number", "py-native-compat.contractVersion");
  assert(value.contractVersion >= 1, "py-native-compat.contractVersion");
  assert.strictEqual(value.module, "gemstone_py_native_compat", "py-native-compat.module");
  assert.strictEqual(
    value.sessionClass,
    "NativeCompatibilitySession",
    "py-native-compat.sessionClass"
  );
  assert.strictEqual(value.handleClass, "OopHandle", "py-native-compat.handleClass");
  assert.strictEqual(typeof value.returnPolicy, "string", "py-native-compat.returnPolicy");
  assert(value.returnPolicy.includes("typed helpers are opt-in"), "py-native-compat.returnPolicy");
  assert(Array.isArray(value.methods), "py-native-compat.methods");
  assert(value.methods.length > 0, "py-native-compat.methods should not be empty");

  const names = new Set();
  for (const [index, method] of value.methods.entries()) {
    assert.strictEqual(typeof method.pythonMethod, "string", `py-native-compat.methods[${index}].pythonMethod`);
    assert(!names.has(method.pythonMethod), `py-native-compat duplicate ${method.pythonMethod}`);
    names.add(method.pythonMethod);
    assert.strictEqual(typeof method.nativeMethod, "string", `py-native-compat.methods[${index}].nativeMethod`);
    assert.strictEqual(typeof method.nativeReturn, "string", `py-native-compat.methods[${index}].nativeReturn`);
    assert.strictEqual(typeof method.pythonReturn, "string", `py-native-compat.methods[${index}].pythonReturn`);
    assert.strictEqual(typeof method.note, "string", `py-native-compat.methods[${index}].note`);
  }

  for (const required of [
    "login_from_env",
    "eval_value",
    "eval_oop",
    "eval_smallint",
    "perform_oop",
    "perform_value",
    "new_symbol",
    "global_get",
    "global_put_oop",
    "value_to_oop_symbol",
    "add_to_export_set",
    "commit",
    "abort",
    "logout",
  ]) {
    assert(names.has(required), `py-native-compat.methods missing ${required}`);
  }
}

function assertPyNativeConformance(value) {
  assert.strictEqual(typeof value.contractVersion, "number", "py-native-conformance.contractVersion");
  assert(value.contractVersion >= 1, "py-native-conformance.contractVersion");
  assert.strictEqual(value.targetPackage, "gemstone-py-native", "py-native-conformance.targetPackage");
  assert.strictEqual(typeof value.status, "string", "py-native-conformance.status");
  assertStringSet(
    value.moduleFunctions,
    [
      "capabilities_json",
      "samples_json",
      "smoke_dry_run_json",
      "migration_json",
      "compatibility_json",
      "conformance_json",
      "handoff_json",
    ],
    "py-native-conformance.moduleFunctions"
  );
  assertStringSet(
    value.nativeSessionMethods,
    [
      "login_from_env",
      "session_id",
      "eval_repr",
      "eval_json",
      "eval_smallint",
      "eval_oop",
      "execute",
      "resolve",
      "value_to_oop_nil",
      "value_to_oop_bool",
      "value_to_oop_smallint",
      "value_to_oop_char",
      "value_to_oop_string",
      "value_to_oop_symbol",
      "value_to_oop_raw",
      "perform_raw_oop",
      "perform_json",
      "new_string",
      "new_symbol",
      "fetch_string",
      "global_get",
      "global_put_raw",
      "global_put_string",
      "global_put_smallint",
      "add_to_export_set",
      "remove_from_export_set",
      "needs_commit",
      "in_transaction",
      "commit",
      "abort",
      "logout",
    ],
    "py-native-conformance.nativeSessionMethods"
  );
  assertStringSet(
    value.compatibilityMethods,
    [
      "login_from_env",
      "eval_value",
      "eval_oop",
      "perform_oop",
      "perform_value",
      "new_symbol",
      "global_get",
      "global_put_oop",
      "value_to_oop_symbol",
      "add_to_export_set",
      "commit",
      "abort",
      "logout",
    ],
    "py-native-conformance.compatibilityMethods"
  );
  assert(Array.isArray(value.fixtures), "py-native-conformance.fixtures");
  const fixturePaths = new Set();
  for (const [index, fixture] of value.fixtures.entries()) {
    assert.strictEqual(typeof fixture.path, "string", `py-native-conformance.fixtures[${index}].path`);
    assert.strictEqual(typeof fixture.command, "string", `py-native-conformance.fixtures[${index}].command`);
    assert.strictEqual(typeof fixture.purpose, "string", `py-native-conformance.fixtures[${index}].purpose`);
    fixturePaths.add(fixture.path);
  }
  for (const required of [
    "examples/py-native/gemstone-rs.py-native.json",
    "examples/py-native/gemstone-rs.py-native-compat.json",
    "examples/py-native/gemstone-rs.py-native-conformance.json",
    "examples/py-native/gemstone-rs.py-native-handoff.json",
  ]) {
    assert(fixturePaths.has(required), `py-native-conformance.fixtures missing ${required}`);
  }
  assert(Array.isArray(value.scaffoldFiles), "py-native-conformance.scaffoldFiles");
  const scaffoldPaths = new Set();
  for (const [index, file] of value.scaffoldFiles.entries()) {
    assert.strictEqual(typeof file.path, "string", `py-native-conformance.scaffoldFiles[${index}].path`);
    assert.strictEqual(typeof file.purpose, "string", `py-native-conformance.scaffoldFiles[${index}].purpose`);
    scaffoldPaths.add(file.path);
  }
  for (const required of ["src/lib.rs", "python/gemstone_py_native_compat.py", "tests/test_smoke.py"]) {
    assert(scaffoldPaths.has(required), `py-native-conformance.scaffoldFiles missing ${required}`);
  }
}

function assertPyNativeHandoff(value) {
  assert.strictEqual(typeof value.contractVersion, "number", "py-native-handoff.contractVersion");
  assert(value.contractVersion >= 1, "py-native-handoff.contractVersion");
  assert.strictEqual(value.targetPackage, "gemstone-py-native", "py-native-handoff.targetPackage");
  assert.strictEqual(value.adapterModule, "gemstone_rs::py_native", "py-native-handoff.adapterModule");
  assert.strictEqual(typeof value.scaffold, "string", "py-native-handoff.scaffold");
  assert.strictEqual(typeof value.status, "string", "py-native-handoff.status");
  assert(Array.isArray(value.artifacts), "py-native-handoff.artifacts");
  assert(value.artifacts.length > 0, "py-native-handoff.artifacts should not be empty");

  const artifactNames = new Set();
  for (const [index, artifact] of value.artifacts.entries()) {
    assert.strictEqual(typeof artifact.name, "string", `py-native-handoff.artifacts[${index}].name`);
    assert(!artifactNames.has(artifact.name), `py-native-handoff.artifacts duplicate ${artifact.name}`);
    artifactNames.add(artifact.name);
    assert.strictEqual(typeof artifact.path, "string", `py-native-handoff.artifacts[${index}].path`);
    assert.strictEqual(typeof artifact.schema, "string", `py-native-handoff.artifacts[${index}].schema`);
    assert.strictEqual(typeof artifact.command, "string", `py-native-handoff.artifacts[${index}].command`);
    assert.strictEqual(typeof artifact.checkCommand, "string", `py-native-handoff.artifacts[${index}].checkCommand`);
    assert.strictEqual(typeof artifact.purpose, "string", `py-native-handoff.artifacts[${index}].purpose`);
  }
  for (const required of [
    "capabilities",
    "samples",
    "smoke",
    "migration",
    "compatibility",
    "conformance",
    "publish-receipt",
  ]) {
    assert(artifactNames.has(required), `py-native-handoff.artifacts missing ${required}`);
  }

  assert(Array.isArray(value.acceptance), "py-native-handoff.acceptance");
  assert(value.acceptance.length > 0, "py-native-handoff.acceptance should not be empty");
  const acceptanceIds = new Set();
  for (const [index, criterion] of value.acceptance.entries()) {
    assert.strictEqual(typeof criterion.id, "string", `py-native-handoff.acceptance[${index}].id`);
    assert(!acceptanceIds.has(criterion.id), `py-native-handoff.acceptance duplicate ${criterion.id}`);
    acceptanceIds.add(criterion.id);
    assert.strictEqual(typeof criterion.required, "boolean", `py-native-handoff.acceptance[${index}].required`);
    assert.strictEqual(typeof criterion.verify, "string", `py-native-handoff.acceptance[${index}].verify`);
  }
  for (const required of [
    "scaffold_compiles",
    "fixtures_current",
    "python_return_policy_preserved",
    "live_native_backend_green",
    "wheels_after_live_green",
  ]) {
    assert(acceptanceIds.has(required), `py-native-handoff.acceptance missing ${required}`);
  }
}

function assertPyNativePublishReceipt(value) {
  assert.strictEqual(
    typeof value.contractVersion,
    "number",
    "py-native-publish-receipt.contractVersion"
  );
  assert(value.contractVersion >= 1, "py-native-publish-receipt.contractVersion");
  assert.strictEqual(
    value.targetPackage,
    "gemstone-py-native",
    "py-native-publish-receipt.targetPackage"
  );
  assert.strictEqual(value.rustCore, "gemstone-rs", "py-native-publish-receipt.rustCore");
  assert.strictEqual(typeof value.releaseTag, "string", "py-native-publish-receipt.releaseTag");
  assert.strictEqual(typeof value.status, "string", "py-native-publish-receipt.status");
  assert(Array.isArray(value.targets), "py-native-publish-receipt.targets");
  assert(value.targets.length >= 2, "py-native-publish-receipt.targets should include TestPyPI and PyPI");

  const indexes = new Set();
  for (const [index, target] of value.targets.entries()) {
    assert.strictEqual(typeof target.index, "string", `py-native-publish-receipt.targets[${index}].index`);
    assert(!indexes.has(target.index), `py-native-publish-receipt duplicate target ${target.index}`);
    indexes.add(target.index);
    assert.strictEqual(target.package, "gemstone-py-native", `py-native-publish-receipt.targets[${index}].package`);
    assert.strictEqual(typeof target.version, "string", `py-native-publish-receipt.targets[${index}].version`);
    assert.strictEqual(target.workflow, "Native Wheels", `py-native-publish-receipt.targets[${index}].workflow`);
    assert.strictEqual(typeof target.runId, "number", `py-native-publish-receipt.targets[${index}].runId`);
    assert.strictEqual(typeof target.runUrl, "string", `py-native-publish-receipt.targets[${index}].runUrl`);
    assert(target.runUrl.includes("/actions/runs/"), `py-native-publish-receipt.targets[${index}].runUrl`);
    assert.strictEqual(typeof target.createdAt, "string", `py-native-publish-receipt.targets[${index}].createdAt`);
    assert.strictEqual(typeof target.verifiedAt, "string", `py-native-publish-receipt.targets[${index}].verifiedAt`);
    assert.strictEqual(target.conclusion, "success", `py-native-publish-receipt.targets[${index}].conclusion`);
    assert.strictEqual(typeof target.installCommand, "string", `py-native-publish-receipt.targets[${index}].installCommand`);
    assert.strictEqual(typeof target.verification, "string", `py-native-publish-receipt.targets[${index}].verification`);
    assert(target.verification.includes("RustCoreSession"), `py-native-publish-receipt.targets[${index}].verification`);
  }

  assert(indexes.has("TestPyPI"), "py-native-publish-receipt.targets missing TestPyPI");
  assert(indexes.has("PyPI"), "py-native-publish-receipt.targets missing PyPI");
}

function assertPyNativeCheckAll(value) {
  assert.strictEqual(value.ok, true, "py-native-check-all.ok");
  assert.strictEqual(typeof value.contractVersion, "number", "py-native-check-all.contractVersion");
  assert(value.contractVersion >= 1, "py-native-check-all.contractVersion");
  assert(
    value.root === null || typeof value.root === "string",
    "py-native-check-all.root"
  );
  assert.strictEqual(typeof value.stepCount, "number", "py-native-check-all.stepCount");
  assert.strictEqual(typeof value.okCount, "number", "py-native-check-all.okCount");
  assert.strictEqual(typeof value.errorCount, "number", "py-native-check-all.errorCount");
  assert(Array.isArray(value.steps), "py-native-check-all.steps");
  assert.strictEqual(value.stepCount, value.steps.length, "py-native-check-all.stepCount");
  assert.strictEqual(
    value.okCount + value.errorCount,
    value.stepCount,
    "py-native-check-all summary counts"
  );

  const names = new Set();
  for (const [index, step] of value.steps.entries()) {
    assert.strictEqual(typeof step.name, "string", `py-native-check-all.steps[${index}].name`);
    assert(!names.has(step.name), `py-native-check-all duplicate ${step.name}`);
    names.add(step.name);
    assert.strictEqual(typeof step.path, "string", `py-native-check-all.steps[${index}].path`);
    assert.strictEqual(typeof step.command, "string", `py-native-check-all.steps[${index}].command`);
    assert.strictEqual(typeof step.ok, "boolean", `py-native-check-all.steps[${index}].ok`);
    assert(
      step.error === null || typeof step.error === "string",
      `py-native-check-all.steps[${index}].error`
    );
  }
  for (const required of [
    "capabilities",
    "samples",
    "smoke",
    "compatibility",
    "conformance",
    "handoff",
    "publish-receipt",
  ]) {
    assert(names.has(required), `py-native-check-all.steps missing ${required}`);
  }
}

function assertPyNativeSamples(value) {
  assert.strictEqual(typeof value.contractVersion, "number", "py-native-samples.contractVersion");
  assert(value.contractVersion >= 1, "py-native-samples.contractVersion");
  assert(Array.isArray(value.values), "py-native-samples.values");
  assert(Array.isArray(value.errors), "py-native-samples.errors");

  const valueNames = new Set();
  for (const [index, entry] of value.values.entries()) {
    assert.strictEqual(typeof entry.name, "string", `py-native-samples.values[${index}].name`);
    assert(!valueNames.has(entry.name), `py-native-samples.values duplicate ${entry.name}`);
    valueNames.add(entry.name);
    assert(entry.value && typeof entry.value === "object", `py-native-samples.values[${index}].value`);
    assert(
      ["nil", "bool", "smallInt", "char", "string", "symbol", "oop"].includes(entry.value.kind),
      `py-native-samples.values[${index}].value.kind`
    );
  }
  for (const required of ["nil", "true", "smallint", "char", "string", "symbol", "oop"]) {
    assert(valueNames.has(required), `py-native-samples.values missing ${required}`);
  }

  const errorNames = new Set();
  for (const [index, entry] of value.errors.entries()) {
    assert.strictEqual(typeof entry.name, "string", `py-native-samples.errors[${index}].name`);
    assert(!errorNames.has(entry.name), `py-native-samples.errors duplicate ${entry.name}`);
    errorNames.add(entry.name);
    assert(entry.error && typeof entry.error === "object", `py-native-samples.errors[${index}].error`);
    assert.strictEqual(typeof entry.error.kind, "string", `py-native-samples.errors[${index}].error.kind`);
    assert.strictEqual(typeof entry.error.message, "string", `py-native-samples.errors[${index}].error.message`);
  }
  for (const required of ["missingConfig", "illegalOop", "unexpectedType", "mapping"]) {
    assert(errorNames.has(required), `py-native-samples.errors missing ${required}`);
  }
}

function assertStringSet(actual, required, context) {
  assert(Array.isArray(actual), context);
  const values = new Set(actual);
  assert.strictEqual(values.size, actual.length, `${context}: duplicate entries`);
  for (const value of actual) {
    assert.strictEqual(typeof value, "string", `${context}: entries must be strings`);
  }
  for (const value of required) {
    assert(values.has(value), `${context}: missing ${value}`);
  }
}

function assertCompare(value, context) {
  assert(["gemstone-py", "gemstone-js", "all"].includes(value.comparison), context);
  assert(["summary", "status", "scorecard", "parity", "gaps", "next", "totals", "batches"].includes(value.view), context);

  if (value.comparison === "all") {
    assert(Array.isArray(value.comparisons), `${context}: comparisons`);
    assert(value.comparisons.length > 0, `${context}: comparisons should not be empty`);
    if (value.view === "totals" || value.view === "batches") {
      assertTotals(value, context);
    }
    for (const [index, comparison] of value.comparisons.entries()) {
      assertCompareEntry(comparison, value.view, `${context}.comparisons[${index}]`);
    }
    return;
  }

  assertCompareEntry(value, value.view, context);
}

function assertCompareEntry(value, view, context) {
  assert(["gemstone-py", "gemstone-js", "all"].includes(value.comparison), context);
  switch (view) {
    case "summary":
      assert(Array.isArray(value.rows), `${context}: rows`);
      assert(value.rows.length > 0, `${context}: rows should not be empty`);
      for (const [index, row] of value.rows.entries()) {
        assertCompareRow(row, value.comparison, `${context}.rows[${index}]`);
      }
      break;
    case "status":
      assertStatus(value, context);
      break;
    case "scorecard":
      assertScorecard(value, context);
      break;
    case "parity":
      assertParity(value, context);
      break;
    case "gaps":
      assert(Array.isArray(value.gaps), `${context}: gaps`);
      assert(value.gaps.length > 0, `${context}: gaps should not be empty`);
      for (const [index, gap] of value.gaps.entries()) {
        assertCompareGap(gap, value.comparison, `${context}.gaps[${index}]`);
      }
      break;
    case "next":
      assertOptionalBatch(value.batch, `${context}.batch`);
      assertGenericGap(value.gap, `${context}.gap`);
      break;
    case "totals":
      assertTotals(value, context);
      break;
    case "batches":
      assertTotals(value, context);
      assert(Array.isArray(value.batches), `${context}: batches`);
      assert.strictEqual(value.totalBatches, value.batches.length, `${context}: batch count`);
      for (const [index, batch] of value.batches.entries()) {
        assertBatch(batch, `${context}.batches[${index}]`);
      }
      break;
    default:
      throw new Error(`${context}: unknown compare view ${view}`);
  }
}

function assertStatus(value, context) {
  assert.strictEqual(typeof value.answer, "string", `${context}.answer`);
  assertTotals(value.remaining, `${context}.remaining`);
  assert(value.parity && typeof value.parity === "object", `${context}.parity`);
  assert.strictEqual(typeof value.parity.gemstonePyScore, "number", `${context}.parity.gemstonePyScore`);
  assert.strictEqual(typeof value.parity.projectScore, "number", `${context}.parity.projectScore`);
  assert.strictEqual(typeof value.parity.maxScore, "number", `${context}.parity.maxScore`);
  assert.strictEqual(typeof value.parity.scoreGap, "number", `${context}.parity.scoreGap`);
  assertOptionalBatch(value.nextBatch, `${context}.nextBatch`);
  assertGenericGap(value.topGap, `${context}.topGap`);
  assert(value.commands && typeof value.commands === "object", `${context}.commands`);
  for (const field of ["scorecard", "parity", "batches", "totals"]) {
    assert.strictEqual(typeof value.commands[field], "string", `${context}.commands.${field}`);
  }
}

function assertParity(value, context) {
  assert.strictEqual(typeof value.project, "string", `${context}.project`);
  assert(value.overall && typeof value.overall === "object", `${context}.overall`);
  assert.strictEqual(typeof value.overall.gemstonePyScore, "number", `${context}.overall.gemstonePyScore`);
  assert.strictEqual(typeof value.overall.projectScore, "number", `${context}.overall.projectScore`);
  assert.strictEqual(typeof value.overall.maxScore, "number", `${context}.overall.maxScore`);
  assert.strictEqual(typeof value.overall.scoreGap, "number", `${context}.overall.scoreGap`);
  assert(Array.isArray(value.rows), `${context}.rows`);
  assert(value.rows.length > 0, `${context}.rows should not be empty`);
  for (const [index, row] of value.rows.entries()) {
    assert.strictEqual(typeof row.area, "string", `${context}.rows[${index}].area`);
    assert.strictEqual(typeof row.gemstonePyScore, "number", `${context}.rows[${index}].gemstonePyScore`);
    assert.strictEqual(typeof row.projectScore, "number", `${context}.rows[${index}].projectScore`);
    assert.strictEqual(typeof row.leader, "string", `${context}.rows[${index}].leader`);
    assert.strictEqual(typeof row.status, "string", `${context}.rows[${index}].status`);
    assert.strictEqual(typeof row.nextAction, "string", `${context}.rows[${index}].nextAction`);
  }
}

function assertScorecard(value, context) {
  assert.strictEqual(typeof value.answer, "string", `${context}.answer`);
  for (const field of [
    "gemstonePyUseWhen",
    "projectUseWhen",
    "gemstonePyStrengths",
    "projectStrengths",
  ]) {
    assert(Array.isArray(value[field]), `${context}.${field}`);
    assert(value[field].length > 0, `${context}.${field} should not be empty`);
    for (const [index, item] of value[field].entries()) {
      assert.strictEqual(typeof item, "string", `${context}.${field}[${index}]`);
    }
  }
  assertTotals(value.remaining, `${context}.remaining`);
  assertOptionalBatch(value.nextBatch, `${context}.nextBatch`);
  assertGenericGap(value.topGap, `${context}.topGap`);
}

function assertCompareRow(row, comparison, context) {
  assert.strictEqual(typeof row.topic, "string", `${context}.topic`);
  assert.strictEqual(typeof row.gemstonePy, "string", `${context}.gemstonePy`);
  assert.strictEqual(typeof row.recommendation, "string", `${context}.recommendation`);
  if (comparison === "gemstone-js") {
    assert.strictEqual(typeof row.gemstoneJs, "string", `${context}.gemstoneJs`);
  } else {
    assert.strictEqual(typeof row.gemstoneRs, "string", `${context}.gemstoneRs`);
  }
}

function assertCompareGap(gap, comparison, context) {
  assert.strictEqual(typeof gap.priority, "string", `${context}.priority`);
  assert.strictEqual(typeof gap.area, "string", `${context}.area`);
  assert.strictEqual(typeof gap.gemstonePyStrength, "string", `${context}.gemstonePyStrength`);
  assert.strictEqual(typeof gap.nextAction, "string", `${context}.nextAction`);
  assert.strictEqual(typeof gap.verifyWith, "string", `${context}.verifyWith`);
  if (comparison === "gemstone-js") {
    assert.strictEqual(typeof gap.gemstoneJsGap, "string", `${context}.gemstoneJsGap`);
  } else {
    assert.strictEqual(typeof gap.gemstoneRsGap, "string", `${context}.gemstoneRsGap`);
  }
}

function assertGenericGap(gap, context) {
  assert.strictEqual(typeof gap.priority, "string", `${context}.priority`);
  assert.strictEqual(typeof gap.area, "string", `${context}.area`);
  assert.strictEqual(typeof gap.gemstonePyStrength, "string", `${context}.gemstonePyStrength`);
  assert.strictEqual(typeof gap.project, "string", `${context}.project`);
  assert.strictEqual(typeof gap.projectGap, "string", `${context}.projectGap`);
  assert.strictEqual(typeof gap.nextAction, "string", `${context}.nextAction`);
  assert.strictEqual(typeof gap.verifyWith, "string", `${context}.verifyWith`);
}

function assertTotals(value, context) {
  assert.strictEqual(typeof value.totalBatches, "number", `${context}.totalBatches`);
  assert.strictEqual(typeof value.hoursMin, "number", `${context}.hoursMin`);
  assert.strictEqual(typeof value.hoursMax, "number", `${context}.hoursMax`);
  assert(value.totalBatches >= 0, `${context}: totalBatches must be non-negative`);
  assert(value.hoursMin >= 0, `${context}: hoursMin must be non-negative`);
  assert(value.hoursMax >= value.hoursMin, `${context}: hoursMax must be >= hoursMin`);
}

function assertBatch(batch, context) {
  assert.strictEqual(typeof batch.number, "number", `${context}.number`);
  assert.strictEqual(typeof batch.focus, "string", `${context}.focus`);
  assert.strictEqual(typeof batch.hoursMin, "number", `${context}.hoursMin`);
  assert.strictEqual(typeof batch.hoursMax, "number", `${context}.hoursMax`);
  assert.strictEqual(typeof batch.outcome, "string", `${context}.outcome`);
  assert.strictEqual(typeof batch.verifyWith, "string", `${context}.verifyWith`);
  assert(batch.hoursMax >= batch.hoursMin, `${context}: hoursMax must be >= hoursMin`);
}

function assertOptionalBatch(batch, context) {
  if (batch === null) {
    return;
  }
  assertBatch(batch, context);
}
