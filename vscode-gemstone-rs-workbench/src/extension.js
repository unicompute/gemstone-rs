const childProcess = require("child_process");
const fs = require("fs");
const http = require("http");
const path = require("path");
const vscode = require("vscode");

let output;
let explorerProvider;

function activate(context) {
  output = vscode.window.createOutputChannel("GemStone RS");
  context.subscriptions.push(output);

  explorerProvider = new GemStoneTreeProvider();
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider("gemstoneRsExplorer", explorerProvider)
  );

  register(context, "gemstoneRs.refreshExplorer", () => explorerProvider.refresh());
  register(context, "gemstoneRs.verifySetup", verifySetup);
  register(context, "gemstoneRs.verifyLiveSetup", verifyLiveSetup);
  register(context, "gemstoneRs.verifyStrictSetup", verifyStrictSetup);
  register(context, "gemstoneRs.runSetupAssistant", runSetupAssistant);
  register(context, "gemstoneRs.doctor", doctor);
  register(context, "gemstoneRs.showEnvironmentTemplate", showEnvironmentTemplate);
  register(context, "gemstoneRs.copyEnvironmentTemplate", copyEnvironmentTemplate);
  register(context, "gemstoneRs.writeEnvironmentTemplate", writeEnvironmentTemplate);
  register(context, "gemstoneRs.eval", evalSmalltalk);
  register(context, "gemstoneRs.browseDictionaries", browseDictionaries);
  register(context, "gemstoneRs.browseClasses", browseClasses);
  register(context, "gemstoneRs.codegenInit", codegenInit);
  register(context, "gemstoneRs.codegenDiscover", codegenDiscover);
  register(context, "gemstoneRs.generateMappingConfig", generateMappingConfig);
  register(context, "gemstoneRs.previewBridgeRoot", previewBridgeRoot);
  register(context, "gemstoneRs.listBridgeRootKeys", listBridgeRootKeys);
  register(context, "gemstoneRs.putBridgeRootString", putBridgeRootString);
  register(context, "gemstoneRs.removeBridgeRootKey", removeBridgeRootKey);
  register(context, "gemstoneRs.runGeneratedMappingExample", runGeneratedMappingExample);
  register(context, "gemstoneRs.codegenPreview", codegenPreview);
  register(context, "gemstoneRs.codegenDiff", codegenDiff);
  register(context, "gemstoneRs.codegenCheck", codegenCheck);
  register(context, "gemstoneRs.codegenExplain", codegenExplain);
  register(context, "gemstoneRs.codegenGenerate", codegenGenerate);
  register(context, "gemstoneRs.codegenPreviewProfile", codegenPreviewProfile);
  register(context, "gemstoneRs.codegenDiffProfile", codegenDiffProfile);
  register(context, "gemstoneRs.codegenCheckProfile", codegenCheckProfile);
  register(context, "gemstoneRs.codegenExplainProfile", codegenExplainProfile);
  register(context, "gemstoneRs.codegenGenerateProfile", codegenGenerateProfile);
  register(context, "gemstoneRs.loadProjectProfiles", loadProjectProfiles);
  register(context, "gemstoneRs.saveProjectProfiles", saveProjectProfiles);
  register(context, "gemstoneRs.exportCodegenProfile", exportCodegenProfile);
  register(context, "gemstoneRs.showSampleProjectProfiles", showSampleProjectProfiles);
  register(context, "gemstoneRs.createProjectProfiles", createProjectProfiles);
  register(context, "gemstoneRs.validateProjectProfiles", validateProjectProfiles);
  register(context, "gemstoneRs.listProjectProfiles", listProjectProfiles);
  register(context, "gemstoneRs.showProjectProfile", showProjectProfile);
  register(context, "gemstoneRs.resolveProjectProfile", resolveProjectProfile);
  register(context, "gemstoneRs.checkProjectProfiles", checkProjectProfiles);
  register(context, "gemstoneRs.launchExplorer", launchExplorer);
  register(context, "gemstoneRs.openExplorerWebview", openExplorerWebview);
  register(context, "gemstoneRs.openMethodSource", openMethodSource);
  register(context, "gemstoneRs.openCodegenDocs", openCodegenDocs);
}

function deactivate() {}

function register(context, command, callback) {
  context.subscriptions.push(vscode.commands.registerCommand(command, callback));
}

class GemStoneTreeProvider {
  constructor() {
    this._onDidChangeTreeData = new vscode.EventEmitter();
    this.onDidChangeTreeData = this._onDidChangeTreeData.event;
  }

  refresh() {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(element) {
    return element;
  }

  async getChildren(element) {
    if (!element) {
      return [
        node("Dictionaries", "dictionaries", vscode.TreeItemCollapsibleState.Collapsed),
        node("Codegen Config", "codegen", vscode.TreeItemCollapsibleState.Expanded),
        node("Explorer", "explorer", vscode.TreeItemCollapsibleState.Collapsed),
      ];
    }

    if (element.type === "dictionaries") {
      return childrenFromCli(["browse", "dictionaries"], (dictionary) =>
        node(dictionary, "dictionary", vscode.TreeItemCollapsibleState.Collapsed, { dictionary })
      );
    }

    if (element.type === "dictionary") {
      return childrenFromCli(["browse", "classes", element.data.dictionary], (className) =>
        node(className, "class", vscode.TreeItemCollapsibleState.Collapsed, {
          dictionary: element.data.dictionary,
          className,
          meta: false,
        })
      );
    }

    if (element.type === "class") {
      return childrenFromCli(
        ["browse", "protocols", element.data.className, element.data.dictionary],
        (protocol) =>
          node(protocol, "protocol", vscode.TreeItemCollapsibleState.Collapsed, {
            dictionary: element.data.dictionary,
            className: element.data.className,
            meta: element.data.meta,
            protocol,
          })
      );
    }

    if (element.type === "protocol") {
      return childrenFromCli(
        [
          "browse",
          "methods",
          element.data.className,
          element.data.protocol,
          element.data.dictionary,
        ],
        (selector) =>
          node(selector, "method", vscode.TreeItemCollapsibleState.None, {
            dictionary: element.data.dictionary,
            className: element.data.className,
            meta: element.data.meta,
            selector,
          }, {
            command: "gemstoneRs.openMethodSource",
            title: "Open Method Source",
            arguments: [
              {
                dictionary: element.data.dictionary,
                className: element.data.className,
                meta: element.data.meta,
                selector,
              },
            ],
          })
      );
    }

    if (element.type === "codegen") {
      const configPath = settings().codegenConfig;
      return [
        node(`Config: ${configPath}`, "message", vscode.TreeItemCollapsibleState.None),
        actionNode("Discover from Live Stone", "gemstoneRs.codegenDiscover"),
        actionNode("Generate Mapping Config", "gemstoneRs.generateMappingConfig"),
        actionNode("Preview BridgeRoot", "gemstoneRs.previewBridgeRoot"),
        actionNode("List BridgeRoot Keys", "gemstoneRs.listBridgeRootKeys"),
        actionNode("Put BridgeRoot String", "gemstoneRs.putBridgeRootString"),
        actionNode("Remove BridgeRoot Key", "gemstoneRs.removeBridgeRootKey"),
        actionNode("Run Generated Mapping Example", "gemstoneRs.runGeneratedMappingExample"),
        actionNode("Preview Wrappers", "gemstoneRs.codegenPreview"),
        actionNode("Diff Generated Output", "gemstoneRs.codegenDiff"),
        actionNode("Check Freshness", "gemstoneRs.codegenCheck"),
        actionNode("Explain Config", "gemstoneRs.codegenExplain"),
        actionNode("Generate Wrappers", "gemstoneRs.codegenGenerate"),
        actionNode("Preview Profile Wrappers", "gemstoneRs.codegenPreviewProfile"),
        actionNode("Diff Profile Output", "gemstoneRs.codegenDiffProfile"),
        actionNode("Check Profile Freshness", "gemstoneRs.codegenCheckProfile"),
        actionNode("Explain Profile Config", "gemstoneRs.codegenExplainProfile"),
        actionNode("Generate Profile Wrappers", "gemstoneRs.codegenGenerateProfile"),
        actionNode("Load Project Profiles", "gemstoneRs.loadProjectProfiles"),
        actionNode("Save Project Profiles", "gemstoneRs.saveProjectProfiles"),
        actionNode("Export Codegen Profile", "gemstoneRs.exportCodegenProfile"),
        actionNode("Show Sample Project Profiles", "gemstoneRs.showSampleProjectProfiles"),
        actionNode("Create Project Profiles", "gemstoneRs.createProjectProfiles"),
        actionNode("Validate Project Profiles", "gemstoneRs.validateProjectProfiles"),
        actionNode("List Project Profiles", "gemstoneRs.listProjectProfiles"),
        actionNode("Show Project Profile", "gemstoneRs.showProjectProfile"),
        actionNode("Resolve Project Profile", "gemstoneRs.resolveProjectProfile"),
        actionNode("Check Project Profiles", "gemstoneRs.checkProjectProfiles"),
        actionNode("Open Codegen Docs", "gemstoneRs.openCodegenDocs"),
      ];
    }

    if (element.type === "explorer") {
      return [
        actionNode("Doctor", "gemstoneRs.doctor"),
        actionNode("Verify Setup", "gemstoneRs.verifySetup"),
        actionNode("Verify Live Setup", "gemstoneRs.verifyLiveSetup"),
        actionNode("Verify Strict Setup", "gemstoneRs.verifyStrictSetup"),
        actionNode("Run Setup Assistant", "gemstoneRs.runSetupAssistant"),
        actionNode("Show Environment Template", "gemstoneRs.showEnvironmentTemplate"),
        actionNode("Copy Environment Template", "gemstoneRs.copyEnvironmentTemplate"),
        actionNode("Write .env.gemstone-rs", "gemstoneRs.writeEnvironmentTemplate"),
        actionNode("Eval Smalltalk", "gemstoneRs.eval"),
        actionNode("Launch Explorer", "gemstoneRs.launchExplorer"),
        actionNode("Open Explorer Webview", "gemstoneRs.openExplorerWebview"),
      ];
    }

    return [];
  }
}

class GemStoneNode extends vscode.TreeItem {
  constructor(label, type, collapsibleState, data = {}, command = undefined) {
    super(label, collapsibleState);
    this.type = type;
    this.data = data;
    this.contextValue = type;
    this.command = command;
  }
}

function node(label, type, collapsibleState, data = {}, command = undefined) {
  return new GemStoneNode(label, type, collapsibleState, data, command);
}

function actionNode(label, command) {
  return node(label, "action", vscode.TreeItemCollapsibleState.None, {}, { command, title: label });
}

async function childrenFromCli(args, mapper) {
  const result = await runCli(args, { allowFailure: true });
  if (result.code !== 0) {
    return [
      node(
        `Command failed: gemstone-rs ${args.join(" ")}`,
        "message",
        vscode.TreeItemCollapsibleState.None
      ),
    ];
  }
  return stdoutLines(result.stdout).map(mapper);
}

async function verifySetup() {
  await runSetupCheck({
    args: ["doctor"],
    title: "gemstone-rs Workbench setup",
    success: "gemstone-rs setup check passed.",
    failure: "gemstone-rs setup check found issues. See GemStone RS output.",
  });
}

async function verifyLiveSetup() {
  await runSetupCheck({
    args: ["doctor", "--live"],
    title: "gemstone-rs Workbench live setup",
    success: "gemstone-rs live setup check passed.",
    failure: "gemstone-rs live setup check found issues. See GemStone RS output.",
  });
}

async function verifyStrictSetup() {
  await runSetupCheck({
    args: ["doctor", "--strict"],
    title: "gemstone-rs Workbench strict setup",
    success: "gemstone-rs strict setup check passed.",
    failure: "gemstone-rs strict setup check found issues. See GemStone RS output.",
  });
}

async function runSetupAssistant() {
  const cfg = settings();
  const url = setupAssistantUrl(cfg);
  let data;
  try {
    data = await httpGetJson(url);
  } catch (error) {
    const report = [
      "gemstone-rs Explorer setup assistant",
      `url: ${url}`,
      "",
      `error: ${error.message}`,
      "",
      "Start the explorer with GemStone RS: Launch Explorer, then run this command again.",
      "",
    ].join("\n");
    output.clear();
    output.append(report);
    output.show(true);
    const action = await vscode.window.showWarningMessage(
      "Setup Assistant could not reach the local explorer.",
      "Launch Explorer",
      "Open Settings",
      "Copy Report"
    );
    if (action === "Launch Explorer") {
      launchExplorer();
    } else if (action === "Open Settings") {
      await vscode.commands.executeCommand(
        "workbench.action.openSettings",
        "@ext:unicompute.gemstone-rs-workbench"
      );
    } else if (action === "Copy Report") {
      await vscode.env.clipboard.writeText(report);
    }
    return;
  }

  const report = formatSetupAssistantReport(url, data);
  output.clear();
  output.append(report);
  output.show(true);
  const action = await vscode.window.showInformationMessage(
    data.success ? "Setup Assistant checks passed." : "Setup Assistant found issues.",
    "Copy Report",
    "Open Explorer Webview",
    "Open Settings"
  );
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(report);
  } else if (action === "Open Explorer Webview") {
    openExplorerWebview();
  } else if (action === "Open Settings") {
    await vscode.commands.executeCommand(
      "workbench.action.openSettings",
      "@ext:unicompute.gemstone-rs-workbench"
    );
  }
}

function setupAssistantUrl(cfg) {
  const url = new URL(`http://${cfg.explorerHost}:${cfg.explorerPort}/api/setup/assistant`);
  url.searchParams.set("env_file", cfg.envFile || ".env.gemstone-rs");
  url.searchParams.set("config", cfg.codegenConfig);
  url.searchParams.set("profile_file", cfg.codegenProfiles);
  return url.toString();
}

function formatSetupAssistantReport(url, data) {
  const lines = [
    "gemstone-rs Explorer setup assistant",
    `url: ${url}`,
    `success: ${Boolean(data.success)}`,
    "",
  ];
  for (const step of data.steps || []) {
    lines.push(`${step.ok ? "OK" : "Needs attention"}: ${step.name || "(unnamed step)"}`);
    lines.push(`  ${step.detail || "-"}`);
    if (step.action) {
      lines.push(`  Action: ${step.action}`);
    }
    lines.push("");
  }
  if (!Array.isArray(data.steps)) {
    lines.push(JSON.stringify(data, null, 2), "");
  }
  return lines.join("\n");
}

async function runSetupCheck({ args, title, success, failure }) {
  const cfg = settings();
  const result = await runCli(args, { allowFailure: true });
  const reportText = formatSetupReport(cfg, result, title);
  output.clear();
  output.append(reportText);
  output.show(true);

  let action;
  if (result.code === 0) {
    action = await vscode.window.showInformationMessage(
      success,
      "Copy Report",
      "Copy Env Script",
      "Open Settings"
    );
  } else {
    action = await vscode.window.showWarningMessage(
      failure,
      "Copy Report",
      "Copy Env Script",
      "Open Settings"
    );
  }
  await handleSetupAction(action, reportText);
}

function formatSetupReport(cfg, result, title) {
  const lines = [
    title,
    `cwd: ${cfg.cwd}`,
    `useCargo: ${cfg.useCargo}`,
    `cliPath: ${cfg.cliPath}`,
    `explorerPath: ${cfg.explorerPath}`,
    `envFile: ${cfg.envFile}`,
    `envFileExists: ${envFileExists(cfg)}`,
    `codegenConfig: ${cfg.codegenConfig}`,
    `codegenProfiles: ${cfg.codegenProfiles}`,
    `codegenConfigExists: ${fs.existsSync(resolvePath(cfg.codegenConfig, cfg.cwd))}`,
    `codegenProfilesExists: ${fs.existsSync(resolvePath(cfg.codegenProfiles, cfg.cwd))}`,
    "",
    commandLine(result).trimEnd(),
  ];
  if (result.stdout.trim()) {
    lines.push(result.stdout.trimEnd());
  }
  if (result.stderr.trim()) {
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

async function handleSetupAction(action, reportText) {
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied gemstone-rs setup report.");
  } else if (action === "Copy Env Script") {
    await copyEnvironmentTemplate();
  } else if (action === "Open Settings") {
    await vscode.commands.executeCommand(
      "workbench.action.openSettings",
      "@ext:unicompute.gemstone-rs-workbench"
    );
  }
}

async function doctor() {
  await runAndShow(["doctor"], { allowFailure: true });
}

async function showEnvironmentTemplate() {
  const result = await runCli(["env", "sample"], { allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stderr);
  output.show(true);

  if (result.code !== 0) {
    vscode.window.showErrorMessage("gemstone-rs env sample failed. See GemStone RS output.");
    return;
  }

  const document = await vscode.workspace.openTextDocument({
    content: result.stdout,
    language: "shellscript",
  });
  await vscode.window.showTextDocument(document, { preview: true });
}

async function copyEnvironmentTemplate() {
  const result = await runCli(["env", "sample"], { allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stderr);
  output.show(true);

  if (result.code !== 0) {
    vscode.window.showErrorMessage("gemstone-rs env sample failed. See GemStone RS output.");
    return;
  }

  await vscode.env.clipboard.writeText(result.stdout);
  vscode.window.showInformationMessage("Copied gemstone-rs environment template.");
}

async function writeEnvironmentTemplate() {
  const target = await vscode.window.showInputBox({
    title: "Write gemstone-rs Environment Template",
    prompt: "Path for the safe GS_* shell template",
    value: ".env.gemstone-rs",
  });
  if (!target) {
    return;
  }
  let result = await runCli(["env", "write", target], { allowFailure: true });
  if (result.code !== 0 && /already exists/.test(`${result.stdout}\n${result.stderr}`)) {
    const choice = await vscode.window.showWarningMessage(
      `${target} already exists. Overwrite it?`,
      { modal: true },
      "Overwrite"
    );
    if (choice === "Overwrite") {
      result = await runCli(["env", "write", target, "--force"], { allowFailure: true });
    }
  }
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stdout);
  output.append(result.stderr);
  output.show(true);
  if (result.code === 0) {
    vscode.window.showInformationMessage(`Wrote ${target}.`);
    await openPathInEditor(target);
  } else {
    vscode.window.showErrorMessage("gemstone-rs env write failed. See GemStone RS output.");
  }
}

async function evalSmalltalk() {
  const source = await vscode.window.showInputBox({
    title: "Eval Smalltalk",
    prompt: "Smalltalk source to evaluate through gemstone-rs",
    value: "3 + 4",
  });
  if (!source) {
    return;
  }
  await runAndShow(["eval", source]);
}

async function browseDictionaries() {
  await runAndShow(["browse", "dictionaries"]);
}

async function browseClasses() {
  const dictionary = await vscode.window.showInputBox({
    title: "Browse Classes",
    prompt: "Dictionary name",
    value: "UserGlobals",
  });
  if (!dictionary) {
    return;
  }
  await runAndShow(["browse", "classes", dictionary]);
}

async function codegenInit() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  await runAndShow(["codegen", "init", configPath]);
  explorerProvider?.refresh();
}

async function codegenDiscover() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  const classText = await vscode.window.showInputBox({
    title: "Discover Codegen Config",
    prompt: "Classes to include, separated by spaces",
    value: "Object",
  });
  if (!classText) {
    return;
  }
  const classes = classText.split(/\s+/).map((value) => value.trim()).filter(Boolean);
  await runAndShow(["codegen", "discover", configPath, ...classes], { allowFailure: true });
  explorerProvider?.refresh();
}

async function generateMappingConfig() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  const mappedName = await vscode.window.showInputBox({
    title: "Generate Mapping Config",
    prompt: "Rust struct name for the BridgeRoot mapping",
    value: "BookingDraft",
  });
  if (!mappedName) {
    return;
  }
  const className = await vscode.window.showInputBox({
    title: "Generate Mapping Config",
    prompt: "GemStone class to inspect for field names",
    value: "Object",
  });
  if (!className) {
    return;
  }
  await runAndShow(["codegen", "discover-mapping", configPath, mappedName, className], {
    allowFailure: true,
  });
  explorerProvider?.refresh();
}

async function previewBridgeRoot() {
  const cfg = settings();
  const url = `http://${cfg.explorerHost}:${cfg.explorerPort}/api/bridge/root`;
  vscode.env.openExternal(vscode.Uri.parse(url));
  output.clear();
  output.appendLine(`Opened ${url}`);
  output.appendLine("Start the local explorer first if the browser reports that the page is unavailable.");
  output.show(true);
}

async function listBridgeRootKeys() {
  await runAndShow(["bridge", "keys"], { allowFailure: true });
}

async function putBridgeRootString() {
  const key = await vscode.window.showInputBox({
    title: "Put BridgeRoot String",
    prompt: "BridgeRoot key to write",
    value: "WorkbenchDraft",
  });
  if (!key) {
    return;
  }
  const value = await vscode.window.showInputBox({
    title: "Put BridgeRoot String",
    prompt: `String value for ${key}`,
    value: "hello from VS Code",
  });
  if (value === undefined) {
    return;
  }
  const result = await runAndShow(["bridge", "put", key, value, "--type", "String"], {
    allowFailure: true,
  });
  if (result.code === 0) {
    explorerProvider?.refresh();
  }
}

async function removeBridgeRootKey() {
  const key = await vscode.window.showInputBox({
    title: "Remove BridgeRoot Key",
    prompt: "BridgeRoot key to remove",
    value: "WorkbenchDraft",
  });
  if (!key) {
    return;
  }
  const choice = await vscode.window.showWarningMessage(
    `Remove ${key} from GemStoneRsBridgeRoot?`,
    { modal: true },
    "Remove"
  );
  if (choice !== "Remove") {
    return;
  }
  const result = await runAndShow(["bridge", "remove", key], { allowFailure: true });
  if (result.code === 0) {
    explorerProvider?.refresh();
  }
}

async function runGeneratedMappingExample() {
  const terminal = vscode.window.createTerminal({
    name: "gemstone-rs Generated Mapping",
    cwd: settings().cwd,
  });
  terminal.sendText("cargo run -p gemstone-rs --example generated_mapping_app");
  terminal.show();
}

async function codegenPreview() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  const result = await runCli(["codegen", "preview", configPath], { allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stderr);
  output.show(true);

  if (result.code !== 0) {
    vscode.window.showErrorMessage("gemstone-rs codegen preview failed. See GemStone RS output.");
    return;
  }

  const document = await vscode.workspace.openTextDocument({
    content: result.stdout,
    language: "rust",
  });
  await vscode.window.showTextDocument(document, { preview: true });
}

async function codegenDiff() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  const result = await runCli(["codegen", "diff", configPath], { allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stdout);
  output.append(result.stderr);
  output.show(true);

  if (result.stdout.trim()) {
    const document = await vscode.workspace.openTextDocument({
      content: result.stdout,
      language: "diff",
    });
    await vscode.window.showTextDocument(document, { preview: true });
  }

  if (result.code === 0) {
    vscode.window.showInformationMessage("gemstone-rs generated output is up to date.");
  } else if (!result.stdout.trim()) {
    vscode.window.showErrorMessage("gemstone-rs codegen diff failed. See GemStone RS output.");
  }
}

async function codegenCheck() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  await runAndShow(["codegen", "check", configPath], { allowFailure: true });
}

async function codegenExplain() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  await runCodegenExplain(["codegen", "explain", "--json", configPath], {
    title: "Codegen config explanation",
    sourcePath: configPath,
    openLabel: "Open Config",
  });
}

async function codegenGenerate() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  const diff = await runCli(["codegen", "diff", configPath], { allowFailure: true });
  if (diff.code !== 0 && diff.stdout.trim()) {
    const document = await vscode.workspace.openTextDocument({
      content: diff.stdout,
      language: "diff",
    });
    await vscode.window.showTextDocument(document, { preview: true });
    const choice = await vscode.window.showWarningMessage(
      "Generated wrappers differ from the current file. Generate anyway?",
      { modal: true },
      "Generate"
    );
    if (choice !== "Generate") {
      return;
    }
  } else if (diff.code !== 0) {
    output.clear();
    output.appendLine(commandLine(diff));
    output.append(diff.stdout);
    output.append(diff.stderr);
    output.show(true);
    vscode.window.showErrorMessage("gemstone-rs codegen diff failed. See GemStone RS output.");
    return;
  }

  const result = await runAndShow(["codegen", "generate", configPath], { allowFailure: true });
  if (result.code !== 0) {
    return;
  }
  explorerProvider?.refresh();

  const generated = parseGeneratedPath(result.stdout);
  if (!generated) {
    return;
  }
  const fullPath = resolvePath(generated, settings().cwd);
  if (fs.existsSync(fullPath)) {
    const document = await vscode.workspace.openTextDocument(fullPath);
    await vscode.window.showTextDocument(document, { preview: false });
  }
}

async function codegenPreviewProfile() {
  const args = await askProfileCodegenArgs("Codegen Preview Profile");
  if (!args) {
    return;
  }
  const result = await runCli(["codegen", "preview-profile", ...args], { allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stderr);
  output.show(true);

  if (result.code !== 0) {
    vscode.window.showErrorMessage("gemstone-rs codegen profile preview failed. See GemStone RS output.");
    return;
  }

  const document = await vscode.workspace.openTextDocument({
    content: result.stdout,
    language: "rust",
  });
  await vscode.window.showTextDocument(document, { preview: true });
}

async function codegenDiffProfile() {
  const args = await askProfileCodegenArgs("Codegen Diff Profile");
  if (!args) {
    return;
  }
  const result = await runCli(["codegen", "diff-profile", ...args], { allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stdout);
  output.append(result.stderr);
  output.show(true);

  if (result.stdout.trim()) {
    const document = await vscode.workspace.openTextDocument({
      content: result.stdout,
      language: "diff",
    });
    await vscode.window.showTextDocument(document, { preview: true });
  }

  if (result.code === 0) {
    vscode.window.showInformationMessage("gemstone-rs profile generated output is up to date.");
  } else if (!result.stdout.trim()) {
    vscode.window.showErrorMessage("gemstone-rs codegen profile diff failed. See GemStone RS output.");
  }
}

async function codegenCheckProfile() {
  const args = await askProfileCodegenArgs("Codegen Check Profile");
  if (!args) {
    return;
  }
  await runAndShow(["codegen", "check-profile", ...args], { allowFailure: true });
}

async function codegenExplainProfile() {
  const selection = await pickProjectProfile("Codegen Explain Profile");
  if (!selection) {
    return;
  }
  await runCodegenExplain(["codegen", "explain-profile", "--json", selection.name, selection.path], {
    title: `Codegen profile explanation: ${selection.name}`,
    sourcePath: selection.path,
    openLabel: "Open Profile File",
  });
}

async function codegenGenerateProfile() {
  const args = await askProfileCodegenArgs("Codegen Generate Profile");
  if (!args) {
    return;
  }
  const diff = await runCli(["codegen", "diff-profile", ...args], { allowFailure: true });
  if (diff.code !== 0 && diff.stdout.trim()) {
    const document = await vscode.workspace.openTextDocument({
      content: diff.stdout,
      language: "diff",
    });
    await vscode.window.showTextDocument(document, { preview: true });
    const choice = await vscode.window.showWarningMessage(
      "Profile-generated wrappers differ from the current file. Generate anyway?",
      { modal: true },
      "Generate"
    );
    if (choice !== "Generate") {
      return;
    }
  } else if (diff.code !== 0) {
    output.clear();
    output.appendLine(commandLine(diff));
    output.append(diff.stdout);
    output.append(diff.stderr);
    output.show(true);
    vscode.window.showErrorMessage("gemstone-rs codegen profile diff failed. See GemStone RS output.");
    return;
  }

  const result = await runAndShow(["codegen", "generate-profile", ...args], { allowFailure: true });
  if (result.code === 0) {
    explorerProvider?.refresh();
  }
}

async function runCodegenExplain(args, { title, sourcePath, openLabel }) {
  const result = await runCli(args, { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs codegen explain returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = showCodegenExplainReport(result, title, report);
  const classes = Array.isArray(report.classes) ? report.classes.length : 0;
  const mapped = Array.isArray(report.mapped) ? report.mapped.length : 0;
  const action = await vscode.window.showInformationMessage(
    `Codegen explain: ${classes} classes, ${mapped} mappings.`,
    "Copy Summary",
    "Copy JSON",
    "Open JSON",
    openLabel
  );
  if (action === "Copy Summary") {
    await vscode.env.clipboard.writeText(reportText);
  } else if (action === "Copy JSON") {
    await vscode.env.clipboard.writeText(JSON.stringify(report, null, 2));
  } else if (action === "Open JSON") {
    const document = await vscode.workspace.openTextDocument({
      content: `${JSON.stringify(report, null, 2)}\n`,
      language: "json",
    });
    await vscode.window.showTextDocument(document, { preview: true });
  } else if (action === openLabel) {
    await openPathInEditor(sourcePath);
  }
}

function showCodegenExplainReport(result, title, report) {
  const reportText = formatCodegenExplainReport(result, title, report);
  output.clear();
  output.append(reportText);
  output.show(true);
  return reportText;
}

function formatCodegenExplainReport(result, title, report) {
  const classes = Array.isArray(report.classes) ? report.classes : [];
  const mapped = Array.isArray(report.mapped) ? report.mapped : [];
  const testStubs = Array.isArray(report.testStubs) ? report.testStubs : [];
  const lines = [
    commandLine(result).trimEnd(),
    title,
    `output: ${report.output || "-"}`,
    `testStubs: ${testStubs.length ? testStubs.join(", ") : "-"}`,
    `classes: ${classes.length}`,
    `mapped: ${mapped.length}`,
    "",
  ];

  for (const cls of classes) {
    const methodCount = Array.isArray(cls.methods) ? cls.methods.length : 0;
    const target = cls.meta ? `${cls.className} class` : cls.className;
    lines.push(`class: ${cls.name || target || "(unnamed)"} methods=${methodCount}`);
    lines.push(`  target: ${target || "-"} dictionary=${cls.dictionary || "-"}`);
    for (const method of cls.methods || []) {
      const args = Array.isArray(method.args) ? method.args.join(", ") : "";
      lines.push(`  method: ${method.selector || "-"}(${args}) -> ${method.return || "Value"}`);
      if (method.doc) {
        lines.push(`    ${method.doc}`);
      }
    }
    lines.push("");
  }

  for (const mapping of mapped) {
    const fields = Array.isArray(mapping.fields) ? mapping.fields : [];
    lines.push(`mapped: ${mapping.name || "(unnamed)"} fields=${fields.length}`);
    if (mapping.doc) {
      lines.push(`  ${mapping.doc}`);
    }
    for (const field of fields) {
      lines.push(
        `  field: ${field.name || "-"} key=${field.key || "-"} keyType=${field.keyType || "-"} type=${field.type || "-"}`
      );
    }
    lines.push("");
  }

  if (result.stderr.trim()) {
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

async function openMethodSource(method) {
  if (!method?.className) {
    return;
  }
  const args = ["browse", "source", method.className, method.selector || "", method.dictionary || ""];
  if (method.meta) {
    args.push("--meta");
  }
  const result = await runCli(args, { allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stderr);
  output.show(true);

  if (result.code !== 0) {
    vscode.window.showErrorMessage("gemstone-rs browse source failed. See GemStone RS output.");
    return;
  }

  const document = await vscode.workspace.openTextDocument({
    content: result.stdout,
    language: "smalltalk",
  });
  await vscode.window.showTextDocument(document, { preview: true });
}

function launchExplorer() {
  const cfg = settings();
  const envArgs = envFileArgs(cfg);
  const envText = envArgs.length ? ` ${envArgs.map(shellQuote).join(" ")}` : "";
  const terminal = vscode.window.createTerminal({
    name: "gemstone-rs Explorer",
    cwd: cfg.cwd,
  });
  if (cfg.useCargo) {
    terminal.sendText(
      `cargo run -p gemstone-rs-explorer --${envText} --host ${shellQuote(cfg.explorerHost)} --port ${cfg.explorerPort}`
    );
  } else {
    terminal.sendText(
      `${shellQuote(cfg.explorerPath)}${envText} --host ${shellQuote(cfg.explorerHost)} --port ${cfg.explorerPort}`
    );
  }
  terminal.show();
  vscode.env.openExternal(vscode.Uri.parse(`http://${cfg.explorerHost}:${cfg.explorerPort}/`));
}

function openExplorerWebview() {
  const cfg = settings();
  const url = `http://${cfg.explorerHost}:${cfg.explorerPort}/`;
  const panel = vscode.window.createWebviewPanel(
    "gemstoneRsExplorerWebview",
    "gemstone-rs Explorer",
    vscode.ViewColumn.Active,
    {
      enableScripts: true,
      retainContextWhenHidden: true,
    }
  );
  panel.webview.html = explorerWebviewHtml(url);
  output.clear();
  output.appendLine(`Opened gemstone-rs Explorer webview for ${url}`);
  output.appendLine("Run GemStone RS: Launch Explorer first if the webview cannot connect.");
  output.show(true);
}

function loadProjectProfiles() {
  openExplorerProfileWorkflow(
    "Load Project Profiles",
    "Use the Codegen Workflow panel: set Config root if needed, then click Load Project Profiles."
  );
}

function saveProjectProfiles() {
  openExplorerProfileWorkflow(
    "Save Project Profiles",
    "Start the explorer with --allow-write, then use the Codegen Workflow panel to click Save Project Profiles."
  );
}

function exportCodegenProfile() {
  openExplorerProfileWorkflow(
    "Export Codegen Profile",
    "Use Profile name and Save Profile if needed, then click Export Profile and copy Profile JSON."
  );
}

async function showSampleProjectProfiles() {
  const result = await runCli(["profile", "sample"], { allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stderr);
  output.show(true);

  if (result.code !== 0) {
    vscode.window.showErrorMessage("gemstone-rs profile sample failed. See GemStone RS output.");
    return;
  }

  const document = await vscode.workspace.openTextDocument({
    content: result.stdout,
    language: "json",
  });
  await vscode.window.showTextDocument(document, { preview: true });
}

async function createProjectProfiles() {
  const profilePath = await vscode.window.showInputBox({
    title: "Create Project Profiles",
    prompt: "Path to write gemstone-rs.codegen-profiles.json",
    value: settings().codegenProfiles,
  });
  if (!profilePath) {
    return;
  }
  const result = await runAndShow(["profile", "init", profilePath], { allowFailure: true });
  if (result.code === 0) {
    explorerProvider?.refresh();
  }
}

async function validateProjectProfiles() {
  const profilePath = await vscode.window.showInputBox({
    title: "Validate Project Profiles",
    prompt: "Path to gemstone-rs.codegen-profiles.json",
    value: settings().codegenProfiles,
  });
  if (!profilePath) {
    return;
  }
  await runAndShow(["profile", "validate", profilePath], { allowFailure: true });
}

async function listProjectProfiles() {
  const profilePath = await vscode.window.showInputBox({
    title: "List Project Profiles",
    prompt: "Path to gemstone-rs.codegen-profiles.json",
    value: settings().codegenProfiles,
  });
  if (!profilePath) {
    return;
  }
  await runAndShow(["profile", "list", profilePath], { allowFailure: true });
}

async function showProjectProfile() {
  const selection = await pickProjectProfile("Show Project Profile");
  if (!selection) {
    return;
  }
  await runAndShow(["profile", "show", selection.name, selection.path], { allowFailure: true });
}

async function resolveProjectProfile() {
  const selection = await pickProjectProfile("Resolve Project Profile");
  if (!selection) {
    return;
  }
  await runAndShow(["profile", "resolve", selection.name, selection.path], { allowFailure: true });
}

async function checkProjectProfiles() {
  const profilePath = await vscode.window.showInputBox({
    title: "Check Project Profiles",
    prompt: "Path to gemstone-rs.codegen-profiles.json",
    value: settings().codegenProfiles,
  });
  if (!profilePath) {
    return;
  }
  const result = await runCli(["profile", "check", "--json", profilePath], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs profile check returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = showProfileCheckReport(result, report);
  const stale = Number(report.staleCount || 0);
  const errors = Number(report.errorCount || 0);
  let action;
  if (result.code === 0 && report.ok) {
    action = await vscode.window.showInformationMessage(
      `Project profiles are fresh: ${Number(report.okCount || 0)} ok.`,
      "Copy Report",
      "Open Profile File"
    );
  } else {
    action = await vscode.window.showErrorMessage(
      `Project profile check failed: ${stale} stale, ${errors} errors.`,
      "Copy Report",
      "Open Profile File"
    );
  }
  await handleProfileCheckAction(action, profilePath, reportText);
}

function parseJsonCommandResult(result, errorMessage) {
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    output.clear();
    output.appendLine(commandLine(result));
    output.append(result.stdout);
    output.append(result.stderr);
    output.appendLine(`JSON parse error: ${error.message}`);
    output.show(true);
    vscode.window.showErrorMessage(errorMessage);
    return undefined;
  }
}

function showProfileCheckReport(result, report) {
  const reportText = formatProfileCheckReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);
  return reportText;
}

function formatProfileCheckReport(result, report) {
  const profiles = Array.isArray(report.profiles) ? report.profiles : [];
  const lines = [
    commandLine(result).trimEnd(),
    "Project profile freshness",
    `path: ${report.path || "-"}`,
  ];
  const okCount = Number(report.okCount || 0);
  const staleCount = Number(report.staleCount || 0);
  const errorCount = Number(report.errorCount || 0);
  const profileCount = Number(report.profileCount || profiles.length);
  lines.push(
    `summary: ${okCount} ok, ${staleCount} stale, ${errorCount} errors, ${profileCount} total`
  );
  lines.push("");
  for (const profile of profiles) {
    const status = profileCheckStatus(profile);
    lines.push(`${status}\t${profile.name || "(unnamed)"}`);
    lines.push(`  config: ${profile.config || "-"}`);
    lines.push(`  output: ${profile.output || "-"}`);
    if (profile.error) {
      lines.push(`  error: ${profile.error}`);
    }
  }
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

async function handleProfileCheckAction(action, profilePath, reportText) {
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied project profile check report.");
  } else if (action === "Open Profile File") {
    await openPathInEditor(profilePath);
  }
}

function profileCheckStatus(profile) {
  if (profile.ok) {
    return "ok";
  }
  if (profile.error) {
    return "error";
  }
  return "stale";
}

async function openPathInEditor(filePath) {
  const fullPath = resolvePath(filePath, settings().cwd);
  try {
    const document = await vscode.workspace.openTextDocument(fullPath);
    await vscode.window.showTextDocument(document, { preview: true });
  } catch (error) {
    vscode.window.showErrorMessage(`Could not open ${fullPath}: ${error.message}`);
  }
}

function openExplorerProfileWorkflow(title, instruction) {
  const cfg = settings();
  const url = `http://${cfg.explorerHost}:${cfg.explorerPort}/`;
  openExplorerWebview();
  output.appendLine("");
  output.appendLine(`Profile workflow: ${title}`);
  output.appendLine(instruction);
  output.appendLine(`Explorer URL: ${url}`);
  output.appendLine("Run GemStone RS: Launch Explorer first if the webview cannot connect.");
  output.show(true);
}

function explorerWebviewHtml(url) {
  const escaped = escapeHtml(url);
  return `<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>gemstone-rs Explorer</title>
<style>
body { margin: 0; font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color: var(--vscode-foreground); background: var(--vscode-editor-background); }
.bar { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 8px 10px; border-bottom: 1px solid var(--vscode-panel-border); }
a { color: var(--vscode-textLink-foreground); }
iframe { display: block; width: 100vw; height: calc(100vh - 42px); border: 0; background: white; }
</style>
</head>
<body>
<div class="bar">
  <strong>gemstone-rs Explorer</strong>
  <a href="${escaped}">Open in Browser</a>
</div>
<iframe src="${escaped}" title="gemstone-rs Explorer"></iframe>
</body>
</html>`;
}

async function openCodegenDocs() {
  const cfg = settings();
  const docsPath = resolvePath("examples/codegen/README.md", cfg.cwd);
  if (fs.existsSync(docsPath)) {
    const document = await vscode.workspace.openTextDocument(docsPath);
    await vscode.window.showTextDocument(document, { preview: true });
  } else {
    vscode.env.openExternal(vscode.Uri.parse("https://github.com/unicompute/gemstone-rs/tree/main/examples/codegen"));
  }
}

async function askConfigPath() {
  return vscode.window.showInputBox({
    title: "gemstone-rs Codegen Config",
    prompt: "Path to gemstone-rs.codegen",
    value: settings().codegenConfig,
  });
}

async function askProfileCodegenArgs(title) {
  const selection = await pickProjectProfile(title);
  return selection ? [selection.name, selection.path] : undefined;
}

async function pickProjectProfile(title) {
  const profilePath = await vscode.window.showInputBox({
    title,
    prompt: "Path to gemstone-rs.codegen-profiles.json",
    value: settings().codegenProfiles,
  });
  if (!profilePath) {
    return undefined;
  }
  const result = await runCli(["profile", "list", "--json", profilePath], { allowFailure: true });
  if (result.code !== 0) {
    output.clear();
    output.appendLine(commandLine(result));
    output.append(result.stdout);
    output.append(result.stderr);
    output.show(true);
    vscode.window.showErrorMessage("gemstone-rs profile list failed. See GemStone RS output.");
    return undefined;
  }

  let parsed;
  try {
    parsed = JSON.parse(result.stdout);
  } catch (error) {
    output.clear();
    output.appendLine(commandLine(result));
    output.append(result.stdout);
    output.append(result.stderr);
    output.appendLine(`JSON parse error: ${error.message}`);
    output.show(true);
    vscode.window.showErrorMessage("gemstone-rs profile list returned invalid JSON.");
    return undefined;
  }

  const profiles = Array.isArray(parsed.profiles) ? parsed.profiles : [];
  if (profiles.length === 0) {
    vscode.window.showWarningMessage(`No profiles found in ${profilePath}.`);
    return undefined;
  }
  const item = await vscode.window.showQuickPick(profiles.map(profileQuickPickItem), {
    title,
    placeHolder: "Select a project profile",
    matchOnDescription: true,
    matchOnDetail: true,
  });
  return item ? { name: item.profile.name, path: profilePath } : undefined;
}

function profileQuickPickItem(profile) {
  const config = profile.config || "-";
  const root = profile.root === "" ? "\"\"" : profile.root || "-";
  const mapped = profile.mapped || "-";
  const className = profile.className || "-";
  return {
    label: profile.name || "(unnamed)",
    description: config,
    detail: `root=${root} mapped=${mapped} className=${className}`,
    profile,
  };
}

async function runAndShow(args, options = {}) {
  const result = await runCli(args, { ...options, allowFailure: true });
  output.clear();
  output.appendLine(commandLine(result));
  output.append(result.stdout);
  output.append(result.stderr);
  output.show(true);
  if (result.code !== 0 && !options.suppressError) {
    vscode.window.showErrorMessage(`gemstone-rs command failed: ${args.join(" ")}`);
  }
  return result;
}

function runCli(args, options = {}) {
  const cfg = settings();
  const command = cfg.useCargo ? "cargo" : cfg.cliPath;
  const argsWithEnv = withEnvFileArgs(args, cfg);
  const commandArgs = cfg.useCargo ? ["run", "-p", "gemstone-rs-cli", "--", ...argsWithEnv] : argsWithEnv;

  return new Promise((resolve, reject) => {
    const child = childProcess.spawn(command, commandArgs, {
      cwd: cfg.cwd,
      env: process.env,
      shell: false,
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.on("error", (error) => {
      if (options.allowFailure) {
        resolve({ command, args: commandArgs, code: 127, stdout, stderr: `${stderr}${error.message}\n` });
      } else {
        reject(error);
      }
    });
    child.on("close", (code) => {
      resolve({ command, args: commandArgs, code, stdout, stderr });
    });
  });
}

function httpGetJson(url) {
  return new Promise((resolve, reject) => {
    const request = http.get(url, (response) => {
      let body = "";
      response.setEncoding("utf8");
      response.on("data", (chunk) => {
        body += chunk;
      });
      response.on("end", () => {
        let data;
        try {
          data = JSON.parse(body);
        } catch (error) {
          reject(new Error(`Invalid JSON from ${url}: ${error.message}`));
          return;
        }
        if (response.statusCode >= 400 && !Array.isArray(data.steps)) {
          reject(new Error(data.error || `HTTP ${response.statusCode}`));
          return;
        }
        resolve(data);
      });
    });
    request.setTimeout(3000, () => {
      request.destroy(new Error(`Timed out connecting to ${url}`));
    });
    request.on("error", reject);
  });
}

function settings() {
  const cfg = vscode.workspace.getConfiguration("gemstoneRs");
  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();
  const checkoutPath = cfg.get("checkoutPath", "").trim();
  const cwd = checkoutPath || workspace;
  return {
    cwd,
    cliPath: cfg.get("cliPath", "gemstone-rs"),
    explorerPath: cfg.get("explorerPath", "gemstone-rs-explorer"),
    useCargo: cfg.get("useCargo", false),
    envFile: cfg.get("envFile", ".env.gemstone-rs"),
    codegenConfig: cfg.get("codegenConfig", "gemstone-rs.codegen"),
    codegenProfiles: cfg.get("codegenProfiles", "gemstone-rs.codegen-profiles.json"),
    explorerHost: cfg.get("explorerHost", "127.0.0.1"),
    explorerPort: cfg.get("explorerPort", 8787),
  };
}

function withEnvFileArgs(args, cfg = settings()) {
  if (args.some((arg) => arg === "--env-file" || String(arg).startsWith("--env-file="))) {
    return args;
  }
  return [...envFileArgs(cfg), ...args];
}

function envFileArgs(cfg = settings()) {
  const envFile = String(cfg.envFile || "").trim();
  if (!envFile || !envFileExists(cfg)) {
    return [];
  }
  return ["--env-file", envFile];
}

function envFileExists(cfg = settings()) {
  const envFile = String(cfg.envFile || "").trim();
  return Boolean(envFile && fs.existsSync(resolvePath(envFile, cfg.cwd)));
}

function resolvePath(value, cwd) {
  return path.isAbsolute(value) ? value : path.join(cwd, value);
}

function parseGeneratedPath(stdout) {
  const match = stdout.match(/^generated\s+(.+)$/m);
  return match ? match[1].trim() : "";
}

function stdoutLines(stdout) {
  return stdout
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function commandLine(result) {
  return `$ ${shellQuote(result.command)} ${result.args.map(shellQuote).join(" ")}\nexit ${result.code}\n`;
}

function shellQuote(value) {
  if (/^[A-Za-z0-9_./:=@+-]+$/.test(String(value))) {
    return String(value);
  }
  return `'${String(value).replace(/'/g, "'\\''")}'`;
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

module.exports = {
  activate,
  deactivate,
};
