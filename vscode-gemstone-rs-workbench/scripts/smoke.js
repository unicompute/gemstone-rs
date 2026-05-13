const assert = require("assert");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
const packageLock = JSON.parse(fs.readFileSync(path.join(root, "package-lock.json"), "utf8"));
const extensionSource = fs.readFileSync(path.join(root, "src", "extension.js"), "utf8");
const readme = fs.readFileSync(path.join(root, "README.md"), "utf8");
const repoRoot = path.resolve(root, "..");
const schemaNames = [
  "gemstone-rs.codegen.schema.json",
  "gemstone-rs.codegen-profiles.schema.json",
  "gemstone-rs.codegen-explain.schema.json",
];

function includes(list, value) {
  return Array.isArray(list) && list.includes(value);
}

const requiredCommands = [
  "gemstoneRs.openExplorerWebview",
  "gemstoneRs.launchExplorer",
  "gemstoneRs.verifyLiveSetup",
  "gemstoneRs.verifyStrictSetup",
  "gemstoneRs.runSetupAssistant",
  "gemstoneRs.showEnvironmentTemplate",
  "gemstoneRs.copyEnvironmentTemplate",
  "gemstoneRs.writeEnvironmentTemplate",
  "gemstoneRs.codegenPreview",
  "gemstoneRs.codegenDiff",
  "gemstoneRs.codegenCheck",
  "gemstoneRs.codegenExplain",
  "gemstoneRs.codegenGenerate",
  "gemstoneRs.codegenPreviewProfile",
  "gemstoneRs.codegenDiffProfile",
  "gemstoneRs.codegenCheckProfile",
  "gemstoneRs.codegenExplainProfile",
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
assert(extensionSource.includes("Copy Env Script"), "verify setup should offer environment script copy");
assert(extensionSource.includes("Open Settings"), "verify setup should offer settings shortcut");
assert(extensionSource.includes('"doctor", "--live"'), "verify live setup should run doctor --live");
assert(extensionSource.includes('"doctor", "--strict"'), "verify strict setup should run doctor --strict");
assert(extensionSource.includes("setupAssistantUrl"), "Workbench should build a setup assistant URL");
assert(extensionSource.includes("/api/setup/assistant"), "Workbench should call the explorer setup assistant");
assert(extensionSource.includes("withEnvFileArgs"), "Workbench commands should pass --env-file when configured");
assert(extensionSource.includes("envFileExists"), "Workbench should report the env-file setting");
assert(extensionSource.includes('"env", "sample"'), "environment template commands should call env sample");
assert(extensionSource.includes('"env", "write"'), "environment write command should call env write");
assert(!extensionSource.includes("GS_PASSWORD='change-me'"), "extension should not hard-code env secrets");
assert(readme.includes("Open Explorer Webview"), "README should mention the webview command");
assert(readme.includes("Show Environment Template"), "README should mention environment template display");
assert(readme.includes("Write .env.gemstone-rs"), "README should mention environment template writes");
assert(readme.includes("Verify Live Setup"), "README should mention live setup verification");
assert(readme.includes("Verify Strict Setup"), "README should mention strict setup verification");
assert(readme.includes("Run Setup Assistant"), "README should mention setup assistant");
assert(readme.includes("Codegen Check Profile"), "README should mention profile-driven codegen");
assert(readme.includes("Codegen Explain"), "README should mention codegen explain");
assert(readme.includes("Codegen Explain Profile"), "README should mention profile codegen explain");
assert(readme.includes("Create Project Profiles"), "README should mention profile creation");
assert(readme.includes("Validate Project Profiles"), "README should mention profile validation");
assert(readme.includes("List Project Profiles"), "README should mention profile listing");
assert(readme.includes("Check Project Profiles"), "README should mention profile checking");
assert(readme.includes("Resolve Project Profile"), "README should mention profile resolution");
assert(extensionSource.includes("showQuickPick"), "profile commands should offer a QuickPick");
assert(
  extensionSource.includes('"codegen", "explain", "--json"'),
  "codegen explain should use JSON output"
);
assert(
  extensionSource.includes('"codegen", "explain-profile", "--json"'),
  "profile codegen explain should use JSON output"
);
assert(
  extensionSource.includes("Codegen config explanation"),
  "codegen explain should render a readable summary"
);
assert(
  extensionSource.includes('"profile", "check", "--json"'),
  "profile checking should use JSON output"
);
assert(
  extensionSource.includes("Project profile freshness"),
  "profile checking should render a readable summary"
);
assert(
  extensionSource.includes("Copy Report"),
  "profile checking should offer report copy action"
);
assert(
  extensionSource.includes("Open Profile File"),
  "profile checking should offer profile file open action"
);
for (const schemaName of schemaNames) {
  const rootSchema = fs.readFileSync(path.join(repoRoot, "schemas", schemaName), "utf8");
  const extensionSchema = fs.readFileSync(path.join(root, "schemas", schemaName), "utf8");
  assert.deepStrictEqual(
    JSON.parse(extensionSchema),
    JSON.parse(rootSchema),
    `extension ${schemaName} must match the repository schema`
  );
}
assert(
  packageJson.contributes.jsonValidation.some((entry) =>
    entry.fileMatch.some((pattern) => pattern.includes("gemstone-rs.codegen"))
  ),
  "package.json should contribute JSON validation for codegen config files"
);
assert(
  packageJson.contributes.jsonValidation.some((entry) =>
    entry.fileMatch.some((pattern) => pattern.includes("gemstone-rs.codegen-profiles.json"))
  ),
  "package.json should contribute JSON validation for profile files"
);
assert(
  packageJson.contributes.configuration.properties["gemstoneRs.envFile"],
  "package.json should expose gemstoneRs.envFile"
);

console.log(`gemstone-rs Workbench smoke checks passed for ${packageJson.version}`);
