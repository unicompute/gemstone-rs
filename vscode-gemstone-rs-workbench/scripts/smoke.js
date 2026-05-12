const assert = require("assert");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
const packageLock = JSON.parse(fs.readFileSync(path.join(root, "package-lock.json"), "utf8"));
const extensionSource = fs.readFileSync(path.join(root, "src", "extension.js"), "utf8");
const readme = fs.readFileSync(path.join(root, "README.md"), "utf8");

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

console.log(`gemstone-rs Workbench smoke checks passed for ${packageJson.version}`);
