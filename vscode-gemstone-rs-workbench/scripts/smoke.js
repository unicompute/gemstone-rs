const assert = require("assert");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
const packageLock = JSON.parse(fs.readFileSync(path.join(root, "package-lock.json"), "utf8"));
const extensionSource = fs.readFileSync(path.join(root, "src", "extension.js"), "utf8");
const readme = fs.readFileSync(path.join(root, "README.md"), "utf8");
const repoRoot = path.resolve(root, "..");
const rootSchema = fs.readFileSync(
  path.join(repoRoot, "schemas/gemstone-rs.codegen-profiles.schema.json"),
  "utf8"
);
const extensionSchema = fs.readFileSync(
  path.join(root, "schemas/gemstone-rs.codegen-profiles.schema.json"),
  "utf8"
);

function includes(list, value) {
  return Array.isArray(list) && list.includes(value);
}

const requiredCommands = [
  "gemstoneRs.openExplorerWebview",
  "gemstoneRs.launchExplorer",
  "gemstoneRs.codegenPreview",
  "gemstoneRs.codegenDiff",
  "gemstoneRs.codegenCheck",
  "gemstoneRs.codegenGenerate",
  "gemstoneRs.codegenPreviewProfile",
  "gemstoneRs.codegenDiffProfile",
  "gemstoneRs.codegenCheckProfile",
  "gemstoneRs.codegenGenerateProfile",
  "gemstoneRs.loadProjectProfiles",
  "gemstoneRs.saveProjectProfiles",
  "gemstoneRs.exportCodegenProfile",
  "gemstoneRs.showSampleProjectProfiles",
  "gemstoneRs.createProjectProfiles",
  "gemstoneRs.validateProjectProfiles",
  "gemstoneRs.listProjectProfiles",
  "gemstoneRs.showProjectProfile",
  "gemstoneRs.resolveProjectProfile",
  "gemstoneRs.checkProjectProfiles",
  "gemstoneRs.generateMappingConfig",
  "gemstoneRs.previewBridgeRoot",
  "gemstoneRs.runGeneratedMappingExample",
];

for (const command of requiredCommands) {
  assert(
    includes(packageJson.activationEvents, `onCommand:${command}`),
    `${command} is missing from activationEvents`
  );
  assert(
    packageJson.contributes.commands.some((entry) => entry.command === command),
    `${command} is missing from contributes.commands`
  );
  assert(extensionSource.includes(`"${command}"`), `${command} is not registered in extension.js`);
}

assert.strictEqual(
  packageLock.packages[""].version,
  packageJson.version,
  "package-lock root version must match package.json"
);

assert(extensionSource.includes("function explorerWebviewHtml"), "webview HTML helper is missing");
assert(extensionSource.includes("<iframe"), "webview should embed the explorer in an iframe");
assert(extensionSource.includes("escapeHtml(url)"), "webview URL must be escaped");
assert(extensionSource.includes("GemStone RS: Launch Explorer first"), "webview launch hint is missing");
assert(readme.includes("Open Explorer Webview"), "README should mention the webview command");
assert(readme.includes("Codegen Check Profile"), "README should mention profile-driven codegen");
assert(readme.includes("Create Project Profiles"), "README should mention profile creation");
assert(readme.includes("Validate Project Profiles"), "README should mention profile validation");
assert(readme.includes("List Project Profiles"), "README should mention profile listing");
assert(readme.includes("Check Project Profiles"), "README should mention profile checking");
assert(readme.includes("Resolve Project Profile"), "README should mention profile resolution");
assert(extensionSource.includes("showQuickPick"), "profile commands should offer a QuickPick");
assert(
  extensionSource.includes('"profile", "check", "--json"'),
  "profile checking should use JSON output"
);
assert(
  extensionSource.includes("Project profile freshness"),
  "profile checking should render a readable summary"
);
assert.deepStrictEqual(
  JSON.parse(extensionSchema),
  JSON.parse(rootSchema),
  "extension profile schema must match the repository schema"
);
assert(
  packageJson.contributes.jsonValidation.some((entry) =>
    entry.fileMatch.includes("gemstone-rs.codegen-profiles.json")
  ),
  "package.json should contribute JSON validation for profile files"
);

console.log(`gemstone-rs Workbench smoke checks passed for ${packageJson.version}`);
