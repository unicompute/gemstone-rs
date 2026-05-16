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

for (const args of [
  ["compare", "gemstone-py", "--json"],
  ["compare", "gemstone-py", "--status", "--json"],
  ["compare", "gemstone-py", "--scorecard", "--json"],
  ["compare", "gemstone-py", "--parity", "--json"],
  ["compare", "gemstone-js", "--gaps", "--json"],
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

console.log("gemstone-rs codegen and comparison schema checks passed");

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
      assertBatch(value.batch, `${context}.batch`);
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
  assertBatch(value.nextBatch, `${context}.nextBatch`);
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
  assertBatch(value.nextBatch, `${context}.nextBatch`);
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
