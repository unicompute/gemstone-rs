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
  "gemstone-rs.profile-check.schema.json",
  "gemstone-rs.compare.schema.json",
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
  "gemstoneRs.showExampleCommands",
  "gemstoneRs.codegenPreview",
  "gemstoneRs.codegenDiff",
  "gemstoneRs.codegenCheck",
  "gemstoneRs.codegenExplain",
  "gemstoneRs.openGeneratedOutput",
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
  "gemstoneRs.putBridgeRootSymbol",
  "gemstoneRs.putBridgeRootSmallInt",
  "gemstoneRs.putBridgeRootBool",
  "gemstoneRs.generateExplorerAuthToken",
  "gemstoneRs.clearExplorerAuthToken",
  "gemstoneRs.compareGemstonePyStatus",
  "gemstoneRs.compareAllStatus",
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
assert(extensionSource.includes("gemstone-rs Explorer Workbench"), "webview should have a workbench wrapper");
assert(extensionSource.includes("Live Inspector"), "webview should include a live inspector");
assert(
  extensionSource.includes("data-probe=\"/api/codegen/profiles/check\""),
  "webview should expose profile status"
);
assert(extensionSource.includes("renderProfileStatus"), "webview should render profile status as structured content");
assert(extensionSource.includes("data-profile-action"), "webview profile status should expose row actions");
assert(extensionSource.includes("runProfileProbe"), "webview profile actions should call explorer profile endpoints");
assert(extensionSource.includes("renderCodegenExplain"), "webview should render codegen explain summaries");
assert(extensionSource.includes("renderBridgeValue"), "webview should render BridgeRoot values as structured content");
assert(extensionSource.includes("data-probe=\"/api/codegen/output\""), "webview should expose generated output reads");
assert(extensionSource.includes("data-command=\"gemstoneRs.openGeneratedOutput\""), "webview should expose generated output file opens");
assert(extensionSource.includes("renderDiff"), "webview should render generated diffs");
assert(extensionSource.includes("renderSetupAssistant"), "webview should render setup assistant steps");
assert(extensionSource.includes("renderBridgeKeys"), "webview should render BridgeRoot key summaries");
assert(extensionSource.includes("profile-table"), "webview should include profile/status tables");
assert(extensionSource.includes("status-pill"), "webview should include status badges");
assert(
  extensionSource.includes("data-probe=\"/api/bridge/root\""),
  "webview should expose BridgeRoot inspection"
);
assert(
  extensionSource.includes("data-command=\"gemstoneRs.codegenGenerate\""),
  "webview should expose generated-file actions"
);
assert(
  extensionSource.includes("data-command=\"gemstoneRs.compareGemstonePyStatus\""),
  "webview should expose comparison status actions"
);
assert(extensionSource.includes("handleExplorerWebviewMessage"), "webview should handle VS Code command messages");
assert(extensionSource.includes("escapeHtml(url)"), "webview URL must be escaped");
assert(extensionSource.includes("GemStone RS: Launch Explorer first"), "webview launch hint is missing");
assert(extensionSource.includes("Copy Env Script"), "verify setup should offer environment script copy");
assert(extensionSource.includes("Open Settings"), "verify setup should offer settings shortcut");
assert(extensionSource.includes('"doctor", "--live"'), "verify live setup should run doctor --live");
assert(extensionSource.includes('"doctor", "--strict"'), "verify strict setup should run doctor --strict");
assert(extensionSource.includes("setupAssistantUrl"), "Workbench should build a setup assistant URL");
assert(extensionSource.includes("explorerUrl"), "Workbench should build token-aware explorer URLs");
assert(extensionSource.includes("explorerAuthArgs"), "Workbench should pass explorer auth token arguments");
assert(
  extensionSource.includes("explorerTerminalEnv"),
  "Workbench should pass explorer auth token through terminal environment"
);
assert(
  extensionSource.includes("crypto.randomBytes"),
  "Workbench should generate explorer auth tokens locally"
);
assert(extensionSource.includes("state.authToken"), "webview should preserve auth token on explorer API calls");
assert(extensionSource.includes("/api/setup/assistant"), "Workbench should call the explorer setup assistant");
assert(extensionSource.includes("withEnvFileArgs"), "Workbench commands should pass --env-file when configured");
assert(extensionSource.includes("envFileExists"), "Workbench should report the env-file setting");
assert(extensionSource.includes('"env", "sample"'), "environment template commands should call env sample");
assert(extensionSource.includes('"env", "write"'), "environment write command should call env write");
assert(extensionSource.includes('"examples", "list", "--json"'), "Workbench should read the CLI examples index");
assert(extensionSource.includes("formatExamplesReport"), "Workbench should render the examples index");
assert(readme.includes("Show Example Commands"), "README should mention examples discovery");
assert(extensionSource.includes("put-string"), "BridgeRoot string command should use put-string");
assert(extensionSource.includes('"bridge", command, key, value'), "BridgeRoot scalar commands should share the CLI command path");
assert(extensionSource.includes("pickBridgeKeyType"), "BridgeRoot commands should prompt for String or Symbol key type");
assert(extensionSource.includes('"--key-type", keyType'), "BridgeRoot commands should pass --key-type to the CLI");
assert(extensionSource.includes("bridgeRootArgs"), "BridgeRoot commands should pass the configured root");
assert(extensionSource.includes("explorerUrlWithParams"), "BridgeRoot explorer URLs should pass root=");
assert(extensionSource.includes("BridgeRoot:"), "sidebar should show the configured BridgeRoot");
assert(extensionSource.includes("put-symbol"), "BridgeRoot symbol command should use put-symbol");
assert(extensionSource.includes("put-smallint"), "BridgeRoot SmallInt command should use put-smallint");
assert(extensionSource.includes("put-bool"), "BridgeRoot Bool command should use put-bool");
assert(!extensionSource.includes("GS_PASSWORD='change-me'"), "extension should not hard-code env secrets");
assert(readme.includes("Open Explorer Webview"), "README should mention the webview command");
assert(readme.includes("Generate Explorer Auth Token"), "README should mention auth token generation");
assert(readme.includes("Show Environment Template"), "README should mention environment template display");
assert(readme.includes("Write .env.gemstone-rs"), "README should mention environment template writes");
assert(readme.includes("Verify Live Setup"), "README should mention live setup verification");
assert(readme.includes("Verify Strict Setup"), "README should mention strict setup verification");
assert(readme.includes("Run Setup Assistant"), "README should mention setup assistant");
assert(readme.includes("Codegen Check Profile"), "README should mention profile-driven codegen");
assert(readme.includes("Codegen Explain"), "README should mention codegen explain");
assert(readme.includes("Codegen Explain Profile"), "README should mention profile codegen explain");
assert(readme.includes("Open Generated Output"), "README should mention generated output opening");
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
  extensionSource.includes('"compare", target, "--status", "--json"'),
  "comparison status should use JSON output"
);
assert(
  extensionSource.includes("formatComparisonStatusReport"),
  "comparison status should render a readable summary"
);
assert(readme.includes("Compare with gemstone-py"), "README should mention gemstone-py comparison");
assert(readme.includes("Show All Comparison Status"), "README should mention aggregate comparison status");
assert(
  extensionSource.includes("Project profile freshness"),
  "profile checking should render a readable summary"
);
assert(
  extensionSource.includes("upToDate"),
  "profile checking should render freshness details"
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
  packageJson.contributes.jsonValidation.some((entry) =>
    entry.fileMatch.some((pattern) => pattern.includes("gemstone-rs.profile-check"))
  ),
  "package.json should contribute JSON validation for profile check reports"
);
assert(
  packageJson.contributes.jsonValidation.some((entry) =>
    entry.fileMatch.some((pattern) => pattern.includes("gemstone-rs.compare"))
  ),
  "package.json should contribute JSON validation for compare reports"
);
assert(
  packageJson.contributes.configuration.properties["gemstoneRs.envFile"],
  "package.json should expose gemstoneRs.envFile"
);
assert(
  packageJson.contributes.configuration.properties["gemstoneRs.explorerAuthToken"],
  "package.json should expose gemstoneRs.explorerAuthToken"
);
assert(
  packageJson.contributes.configuration.properties["gemstoneRs.bridgeRoot"],
  "package.json should expose gemstoneRs.bridgeRoot"
);

console.log(`gemstone-rs Workbench smoke checks passed for ${packageJson.version}`);
