const childProcess = require("child_process");
const crypto = require("crypto");
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
  register(context, "gemstoneRs.showExampleCommands", showExampleCommands);
  register(context, "gemstoneRs.codegenInit", codegenInit);
  register(context, "gemstoneRs.codegenDiscover", codegenDiscover);
  register(context, "gemstoneRs.generateMappingConfig", generateMappingConfig);
  register(context, "gemstoneRs.previewBridgeRoot", previewBridgeRoot);
  register(context, "gemstoneRs.listBridgeRootKeys", listBridgeRootKeys);
  register(context, "gemstoneRs.putBridgeRootString", putBridgeRootString);
  register(context, "gemstoneRs.putBridgeRootSymbol", putBridgeRootSymbol);
  register(context, "gemstoneRs.putBridgeRootSmallInt", putBridgeRootSmallInt);
  register(context, "gemstoneRs.putBridgeRootBool", putBridgeRootBool);
  register(context, "gemstoneRs.removeBridgeRootKey", removeBridgeRootKey);
  register(context, "gemstoneRs.runGeneratedMappingExample", runGeneratedMappingExample);
  register(context, "gemstoneRs.codegenPreview", codegenPreview);
  register(context, "gemstoneRs.codegenDiff", codegenDiff);
  register(context, "gemstoneRs.codegenCheck", codegenCheck);
  register(context, "gemstoneRs.codegenExplain", codegenExplain);
  register(context, "gemstoneRs.openCodegenConfig", openCodegenConfig);
  register(context, "gemstoneRs.openProjectProfiles", openProjectProfiles);
  register(context, "gemstoneRs.openGeneratedOutput", openGeneratedOutput);
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
  register(context, "gemstoneRs.generateExplorerAuthToken", generateExplorerAuthToken);
  register(context, "gemstoneRs.clearExplorerAuthToken", clearExplorerAuthToken);
  register(context, "gemstoneRs.openExplorerWebview", openExplorerWebview);
  register(context, "gemstoneRs.openMethodSource", openMethodSource);
  register(context, "gemstoneRs.openCodegenDocs", openCodegenDocs);
  register(context, "gemstoneRs.validatePyNativeContract", validatePyNativeContract);
  register(context, "gemstoneRs.validatePyNativeSamplesFixture", validatePyNativeSamplesFixture);
  register(context, "gemstoneRs.validatePyNativeSmokeFixture", validatePyNativeSmokeFixture);
  register(context, "gemstoneRs.runPyNativeSmoke", runPyNativeSmoke);
  register(context, "gemstoneRs.showPyNativeMigrationPlan", showPyNativeMigrationPlan);
  register(context, "gemstoneRs.validatePyNativeConformanceFixture", validatePyNativeConformanceFixture);
  register(context, "gemstoneRs.validatePyNativeHandoffBundle", validatePyNativeHandoffBundle);
  register(context, "gemstoneRs.showPyNativeHandoffBundle", showPyNativeHandoffBundle);
  register(context, "gemstoneRs.validatePyNativePublishReceipt", validatePyNativePublishReceipt);
  register(context, "gemstoneRs.showPyNativePublishReceipt", showPyNativePublishReceipt);
  register(context, "gemstoneRs.validatePyNativeSharedCoreGate", validatePyNativeSharedCoreGate);
  register(context, "gemstoneRs.compareGemstonePyStatus", compareGemstonePyStatus);
  register(context, "gemstoneRs.compareAllStatus", compareAllStatus);
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
        node("Comparison", "comparison", vscode.TreeItemCollapsibleState.Expanded),
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
      const cfg = settings();
      const configPath = cfg.codegenConfig;
      return [
        node(`Config: ${configPath}`, "message", vscode.TreeItemCollapsibleState.None),
        node(`BridgeRoot: ${cfg.bridgeRoot}`, "message", vscode.TreeItemCollapsibleState.None),
        actionNode("Discover from Live Stone", "gemstoneRs.codegenDiscover"),
        actionNode("Generate Mapping Config", "gemstoneRs.generateMappingConfig"),
        actionNode("Preview BridgeRoot", "gemstoneRs.previewBridgeRoot"),
        actionNode("List BridgeRoot Keys", "gemstoneRs.listBridgeRootKeys"),
        actionNode("Put BridgeRoot String", "gemstoneRs.putBridgeRootString"),
        actionNode("Put BridgeRoot Symbol", "gemstoneRs.putBridgeRootSymbol"),
        actionNode("Put BridgeRoot SmallInt", "gemstoneRs.putBridgeRootSmallInt"),
        actionNode("Put BridgeRoot Bool", "gemstoneRs.putBridgeRootBool"),
        actionNode("Remove BridgeRoot Key", "gemstoneRs.removeBridgeRootKey"),
        actionNode("Run Generated Mapping Example", "gemstoneRs.runGeneratedMappingExample"),
        actionNode("Preview Wrappers", "gemstoneRs.codegenPreview"),
        actionNode("Diff Generated Output", "gemstoneRs.codegenDiff"),
        actionNode("Check Freshness", "gemstoneRs.codegenCheck"),
        actionNode("Explain Config", "gemstoneRs.codegenExplain"),
        actionNode("Open Codegen Config", "gemstoneRs.openCodegenConfig"),
        actionNode("Open Project Profiles", "gemstoneRs.openProjectProfiles"),
        actionNode("Open Generated Output", "gemstoneRs.openGeneratedOutput"),
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
        actionNode("Validate py-native Contract", "gemstoneRs.validatePyNativeContract"),
        actionNode("Validate py-native Samples Fixture", "gemstoneRs.validatePyNativeSamplesFixture"),
        actionNode("Validate py-native Smoke Fixture", "gemstoneRs.validatePyNativeSmokeFixture"),
        actionNode("Run py-native Smoke", "gemstoneRs.runPyNativeSmoke"),
        actionNode("Show py-native Migration Plan", "gemstoneRs.showPyNativeMigrationPlan"),
        actionNode("Validate py-native Conformance Fixture", "gemstoneRs.validatePyNativeConformanceFixture"),
        actionNode("Validate py-native Handoff Bundle", "gemstoneRs.validatePyNativeHandoffBundle"),
        actionNode("Show py-native Handoff Bundle", "gemstoneRs.showPyNativeHandoffBundle"),
        actionNode("Validate py-native Publish Receipt", "gemstoneRs.validatePyNativePublishReceipt"),
        actionNode("Show py-native Publish Receipt", "gemstoneRs.showPyNativePublishReceipt"),
        actionNode("Validate py-native Shared Core Gate", "gemstoneRs.validatePyNativeSharedCoreGate"),
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
        actionNode("Show Example Commands", "gemstoneRs.showExampleCommands"),
        actionNode("Generate Explorer Auth Token", "gemstoneRs.generateExplorerAuthToken"),
        actionNode("Clear Explorer Auth Token", "gemstoneRs.clearExplorerAuthToken"),
        actionNode("Launch Explorer", "gemstoneRs.launchExplorer"),
        actionNode("Open Explorer Webview", "gemstoneRs.openExplorerWebview"),
      ];
    }

    if (element.type === "comparison") {
      return [
        actionNode("Compare with gemstone-py", "gemstoneRs.compareGemstonePyStatus"),
        actionNode("Show All Comparison Status", "gemstoneRs.compareAllStatus"),
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
  const url = new URL(explorerUrl(cfg, "/api/setup/assistant"));
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
    `bridgeRoot: ${cfg.bridgeRoot}`,
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

async function showExampleCommands() {
  const result = await runCli(["examples", "list", "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs examples list returned invalid JSON.");
  if (!report) {
    return;
  }
  const examples = Array.isArray(report.examples) ? report.examples : [];
  const reportText = formatExamplesReport(result, examples);
  output.clear();
  output.append(reportText);
  output.show(true);

  if (examples.length === 0) {
    vscode.window.showWarningMessage("No gemstone-rs examples were reported by the CLI.");
    return;
  }
  const item = await vscode.window.showQuickPick(examples.map(exampleQuickPickItem), {
    title: "gemstone-rs Examples",
    placeHolder: "Select an example command",
    matchOnDescription: true,
    matchOnDetail: true,
  });
  if (!item) {
    return;
  }
  const action = await vscode.window.showInformationMessage(
    item.example.requiresLive
      ? `${item.example.name} requires a live GemStone/S stone.`
      : `${item.example.name} can run without a live stone.`,
    "Run in Terminal",
    "Copy Command",
    "Open Examples Guide"
  );
  if (action === "Run in Terminal") {
    const terminal = vscode.window.createTerminal({
      name: `gemstone-rs ${item.example.name}`,
      cwd: settings().cwd,
    });
    terminal.sendText(item.example.command);
    terminal.show();
  } else if (action === "Copy Command") {
    await vscode.env.clipboard.writeText(item.example.command);
  } else if (action === "Open Examples Guide") {
    await openExamplesGuide();
  }
}

function formatExamplesReport(result, examples) {
  const lines = [commandLine(result).trimEnd(), "gemstone-rs examples", ""];
  for (const example of examples) {
    lines.push(`${example.name}\t${example.category || "-"}\tlive=${Boolean(example.requiresLive)}`);
    lines.push(`  ${example.command || "-"}`);
    lines.push(`  ${example.description || "-"}`);
  }
  if (result.stderr.trim()) {
    lines.push("", result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function exampleQuickPickItem(example) {
  return {
    label: example.name || "(unnamed)",
    description: `${example.category || "-"} live=${Boolean(example.requiresLive)}`,
    detail: `${example.command || "-"} - ${example.description || ""}`,
    example,
  };
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
  const url = explorerUrlWithParams(cfg, "/api/bridge/root", { root: cfg.bridgeRoot });
  vscode.env.openExternal(vscode.Uri.parse(url));
  output.clear();
  output.appendLine(`Opened ${url}`);
  output.appendLine("Start the local explorer first if the browser reports that the page is unavailable.");
  output.show(true);
}

async function listBridgeRootKeys() {
  await runAndShow(["bridge", "keys", ...bridgeRootArgs()], { allowFailure: true });
}

async function putBridgeRootString() {
  await putBridgeRootScalar({
    title: "Put BridgeRoot String",
    prompt: "String value",
    defaultKey: "WorkbenchDraft",
    defaultValue: "hello from VS Code",
    command: "put-string",
  });
}

async function putBridgeRootSymbol() {
  await putBridgeRootScalar({
    title: "Put BridgeRoot Symbol",
    prompt: "Symbol value",
    defaultKey: "WorkbenchState",
    defaultValue: "ready",
    command: "put-symbol",
  });
}

async function putBridgeRootSmallInt() {
  await putBridgeRootScalar({
    title: "Put BridgeRoot SmallInt",
    prompt: "SmallInt value",
    defaultKey: "WorkbenchCount",
    defaultValue: "7",
    command: "put-smallint",
  });
}

async function putBridgeRootBool() {
  await putBridgeRootScalar({
    title: "Put BridgeRoot Bool",
    prompt: "Bool value",
    defaultKey: "WorkbenchReady",
    defaultValue: "true",
    command: "put-bool",
  });
}

async function putBridgeRootScalar({ title, prompt, defaultKey, defaultValue, command }) {
  const key = await vscode.window.showInputBox({
    title,
    prompt: "BridgeRoot key to write",
    value: defaultKey,
  });
  if (!key) {
    return;
  }
  const keyType = await pickBridgeKeyType(title);
  if (!keyType) {
    return;
  }
  const value = await vscode.window.showInputBox({
    title,
    prompt: `${prompt} for ${key}`,
    value: defaultValue,
  });
  if (value === undefined) {
    return;
  }
  const result = await runAndShow(
    ["bridge", command, key, value, "--key-type", keyType, ...bridgeRootArgs()],
    { allowFailure: true }
  );
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
  const keyType = await pickBridgeKeyType("Remove BridgeRoot Key");
  if (!keyType) {
    return;
  }
  const choice = await vscode.window.showWarningMessage(
    `Remove ${key} (${keyType} key) from ${settings().bridgeRoot}?`,
    { modal: true },
    "Remove"
  );
  if (choice !== "Remove") {
    return;
  }
  const result = await runAndShow(
    ["bridge", "remove", key, "--key-type", keyType, ...bridgeRootArgs()],
    { allowFailure: true }
  );
  if (result.code === 0) {
    explorerProvider?.refresh();
  }
}

async function pickBridgeKeyType(title) {
  const choice = await vscode.window.showQuickPick(["String", "Symbol"], {
    title,
    placeHolder: "BridgeRoot key type",
  });
  return choice;
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

async function openCodegenConfig() {
  await openPathInEditor(settings().codegenConfig);
}

async function openProjectProfiles() {
  await openPathInEditor(settings().codegenProfiles);
}

async function openGeneratedOutput() {
  const configPath = await askConfigPath();
  if (!configPath) {
    return;
  }
  const result = await runCli(["codegen", "explain", "--json", configPath], { allowFailure: true });
  const report = parseJsonCommandResult(
    result,
    "gemstone-rs codegen explain returned invalid JSON."
  );
  if (!report) {
    return;
  }
  if (!report.output) {
    vscode.window.showWarningMessage("The codegen config did not report an output file.");
    return;
  }
  await openPathInEditor(report.output);
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

async function compareGemstonePyStatus() {
  await runComparisonStatus("gemstone-py");
}

async function compareAllStatus() {
  await runComparisonStatus("all");
}

async function runComparisonStatus(target) {
  const result = await runCli(["compare", target, "--status", "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(
    result,
    `gemstone-rs compare ${target} --status returned invalid JSON.`
  );
  if (!report) {
    return;
  }

  const reportText = showComparisonStatusReport(result, report);
  const message = comparisonStatusMessage(report);
  const action = await vscode.window.showInformationMessage(
    message,
    "Copy Report",
    "Copy JSON",
    "Open Comparison Guide"
  );
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
  } else if (action === "Copy JSON") {
    await vscode.env.clipboard.writeText(`${JSON.stringify(report, null, 2)}\n`);
  } else if (action === "Open Comparison Guide") {
    await openComparisonGuide();
  }
}

function showComparisonStatusReport(result, report) {
  const reportText = formatComparisonStatusReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);
  return reportText;
}

function formatComparisonStatusReport(result, report) {
  const lines = [commandLine(result).trimEnd()];
  if (report.comparison === "all" && Array.isArray(report.comparisons)) {
    lines.push("gemstone-rs comparison status");
    lines.push(`total: ${Number(report.totalBatches || 0)} batches, ${Number(report.hoursMin || 0)}-${Number(report.hoursMax || 0)} hours`);
    lines.push("");
    for (const entry of report.comparisons) {
      appendComparisonStatusEntry(lines, entry);
    }
  } else {
    lines.push("gemstone-rs status vs gemstone-py");
    appendComparisonStatusEntry(lines, report);
  }
  if (result.stderr.trim()) {
    lines.push("", result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function appendComparisonStatusEntry(lines, entry) {
  const parity = entry.parity || {};
  const remaining = entry.remaining || {};
  const nextBatch = entry.nextBatch || {};
  const topGap = entry.topGap || {};
  const commands = entry.commands || {};
  const projectLabel = entry.comparison === "gemstone-js" ? "gemstone-js" : "gemstone-rs";
  lines.push(`${entry.comparison || "comparison"}`);
  lines.push(`  answer: ${entry.answer || "-"}`);
  lines.push(
    `  parity: gemstone-py ${Number(parity.gemstonePyScore || 0)}/${Number(parity.maxScore || 0)}; ${projectLabel} ${Number(parity.projectScore || 0)}/${Number(parity.maxScore || 0)}; gap ${Number(parity.scoreGap || 0)}`
  );
  lines.push(
    `  remaining: ${Number(remaining.totalBatches || 0)} batches, ${Number(remaining.hoursMin || 0)}-${Number(remaining.hoursMax || 0)} hours`
  );
  if (nextBatch.focus) {
    lines.push(
      `  next batch: ${nextBatch.number || "-"} ${nextBatch.focus} (${Number(nextBatch.hoursMin || 0)}-${Number(nextBatch.hoursMax || 0)} hours)`
    );
  }
  if (topGap.area) {
    lines.push(`  top gap: ${topGap.priority || "-"} ${topGap.area}`);
    lines.push(`  next action: ${topGap.nextAction || "-"}`);
  }
  if (commands.scorecard || commands.parity || commands.batches) {
    lines.push("  commands:");
    if (commands.scorecard) {
      lines.push(`    ${commands.scorecard}`);
    }
    if (commands.parity) {
      lines.push(`    ${commands.parity}`);
    }
    if (commands.batches) {
      lines.push(`    ${commands.batches}`);
    }
  }
  lines.push("");
}

function comparisonStatusMessage(report) {
  if (report.comparison === "all") {
    return `Comparison status: ${Number(report.totalBatches || 0)} batches, ${Number(report.hoursMin || 0)}-${Number(report.hoursMax || 0)} hours.`;
  }
  const parity = report.parity || {};
  const remaining = report.remaining || {};
  return `gemstone-rs vs gemstone-py: ${Number(parity.projectScore || 0)}/${Number(parity.maxScore || 0)} parity, ${Number(remaining.totalBatches || 0)} batches remain.`;
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
  const authArgs = explorerAuthArgs(cfg);
  const authText = authArgs.length ? ` ${authArgs.map(shellQuote).join(" ")}` : "";
  const terminal = vscode.window.createTerminal({
    name: "gemstone-rs Explorer",
    cwd: cfg.cwd,
    env: explorerTerminalEnv(cfg),
  });
  if (cfg.useCargo) {
    terminal.sendText(
      `cargo run -p gemstone-rs-explorer --${envText}${authText} --host ${shellQuote(cfg.explorerHost)} --port ${cfg.explorerPort}`
    );
  } else {
    terminal.sendText(
      `${shellQuote(cfg.explorerPath)}${envText}${authText} --host ${shellQuote(cfg.explorerHost)} --port ${cfg.explorerPort}`
    );
  }
  terminal.show();
  vscode.env.openExternal(vscode.Uri.parse(explorerUrl(cfg)));
}

async function generateExplorerAuthToken() {
  const token = crypto.randomBytes(24).toString("base64url");
  const target = vscode.workspace.workspaceFolders?.length
    ? vscode.ConfigurationTarget.Workspace
    : vscode.ConfigurationTarget.Global;
  await vscode.workspace.getConfiguration("gemstoneRs").update("explorerAuthToken", token, target);
  await vscode.env.clipboard.writeText(token);
  explorerProvider?.refresh();
  output.clear();
  output.appendLine("Generated gemstone-rs explorer auth token.");
  output.appendLine(`scope: ${target === vscode.ConfigurationTarget.Workspace ? "workspace" : "global"}`);
  output.appendLine("The token was copied to the clipboard and stored in gemstoneRs.explorerAuthToken.");
  output.appendLine("Launch Explorer will pass it via GEMSTONE_RS_EXPLORER_TOKEN.");
  output.show(true);
  const action = await vscode.window.showInformationMessage(
    "Generated gemstone-rs explorer auth token.",
    "Launch Explorer",
    "Open Settings",
    "Copy Token"
  );
  if (action === "Launch Explorer") {
    launchExplorer();
  } else if (action === "Open Settings") {
    await vscode.commands.executeCommand(
      "workbench.action.openSettings",
      "gemstoneRs.explorerAuthToken"
    );
  } else if (action === "Copy Token") {
    await vscode.env.clipboard.writeText(token);
  }
}

async function clearExplorerAuthToken() {
  const cfg = settings();
  if (!cfg.explorerAuthToken) {
    vscode.window.showInformationMessage("No gemstone-rs explorer auth token is configured.");
    return;
  }
  const choice = await vscode.window.showWarningMessage(
    "Clear gemstoneRs.explorerAuthToken?",
    { modal: true },
    "Clear Token"
  );
  if (choice !== "Clear Token") {
    return;
  }
  const target = vscode.workspace.workspaceFolders?.length
    ? vscode.ConfigurationTarget.Workspace
    : vscode.ConfigurationTarget.Global;
  await vscode.workspace.getConfiguration("gemstoneRs").update("explorerAuthToken", "", target);
  explorerProvider?.refresh();
  output.clear();
  output.appendLine("Cleared gemstone-rs explorer auth token.");
  output.show(true);
}

function openExplorerWebview() {
  const cfg = settings();
  const url = explorerUrl(cfg);
  const panel = vscode.window.createWebviewPanel(
    "gemstoneRsExplorerWebview",
    "gemstone-rs Explorer",
    vscode.ViewColumn.Active,
    {
      enableScripts: true,
      retainContextWhenHidden: true,
    }
  );
  panel.webview.onDidReceiveMessage((message) =>
    handleExplorerWebviewMessage(message, url, panel.webview)
  );
  panel.webview.html = explorerWebviewHtml(url, cfg);
  output.clear();
  output.appendLine(`Opened gemstone-rs Explorer webview for ${url}`);
  output.appendLine("Run GemStone RS: Launch Explorer first if the webview cannot connect.");
  output.show(true);
  probeExplorerHealth(url);
}

async function handleExplorerWebviewMessage(message, url, webview) {
  if (!message || typeof message !== "object") {
    return;
  }
  if (message.command === "openExternal") {
    await vscode.env.openExternal(vscode.Uri.parse(String(message.url || url)));
    return;
  }
  if (message.command === "openPath") {
    const filePath = String(message.path || "").trim();
    if (filePath) {
      await openPathInEditor(filePath);
    }
    return;
  }
  if (message.command === "openDocument") {
    const content = String(message.content || "");
    if (content) {
      const document = await vscode.workspace.openTextDocument({
        content,
        language: String(message.language || "plaintext"),
      });
      await vscode.window.showTextDocument(document, { preview: true });
    }
    return;
  }
  if (message.command === "saveGeneratedOutput") {
    await saveGeneratedOutputFromWebview(
      String(message.path || ""),
      String(message.content || ""),
      webview
    );
    return;
  }
  if (message.command === "runWorkbenchCommand") {
    await runWorkbenchCommand(String(message.id || ""));
  }
}

async function saveGeneratedOutputFromWebview(filePath, content, webview) {
  const cfg = settings();
  const trimmedPath = filePath.trim();
  if (!trimmedPath) {
    vscode.window.showWarningMessage("No generated output path is available to save.");
    webview?.postMessage({ command: "generatedOutputSaved", ok: false, error: "missing output path" });
    return;
  }

  const fullPath = path.resolve(resolvePath(trimmedPath, cfg.cwd));
  const root = path.resolve(cfg.cwd);
  if (!isPathInside(fullPath, root)) {
    const message = `Refusing to save outside the configured checkout: ${fullPath}`;
    vscode.window.showErrorMessage(message);
    webview?.postMessage({ command: "generatedOutputSaved", ok: false, path: fullPath, error: message });
    return;
  }

  const choice = await vscode.window.showWarningMessage(
    `Save edited generated output to ${path.relative(root, fullPath) || fullPath}?`,
    { modal: true },
    "Save"
  );
  if (choice !== "Save") {
    webview?.postMessage({ command: "generatedOutputSaved", ok: false, path: fullPath, error: "save cancelled" });
    return;
  }

  try {
    await fs.promises.mkdir(path.dirname(fullPath), { recursive: true });
    await fs.promises.writeFile(fullPath, content, "utf8");
    const document = await vscode.workspace.openTextDocument(fullPath);
    await vscode.window.showTextDocument(document, { preview: false });
    vscode.window.showInformationMessage(`Saved generated output: ${path.relative(root, fullPath) || fullPath}`);
    webview?.postMessage({
      command: "generatedOutputSaved",
      ok: true,
      path: fullPath,
      bytes: Buffer.byteLength(content, "utf8"),
    });
  } catch (error) {
    vscode.window.showErrorMessage(`Could not save generated output: ${error.message}`);
    webview?.postMessage({ command: "generatedOutputSaved", ok: false, path: fullPath, error: error.message });
  }
}

function isPathInside(filePath, root) {
  const relative = path.relative(root, filePath);
  return Boolean(relative) && !relative.startsWith("..") && !path.isAbsolute(relative);
}

async function probeExplorerHealth(url) {
  const healthUrl = new URL("/health", url).toString();
  try {
    await httpGetJson(healthUrl);
  } catch (error) {
    const action = await vscode.window.showWarningMessage(
      "gemstone-rs Explorer is not reachable yet.",
      "Launch Explorer",
      "Open Browser",
      "Copy URL"
    );
    if (action === "Launch Explorer") {
      launchExplorer();
    } else if (action === "Open Browser") {
      await vscode.env.openExternal(vscode.Uri.parse(url));
    } else if (action === "Copy URL") {
      await vscode.env.clipboard.writeText(url);
    }
  }
}

async function runWorkbenchCommand(commandId) {
  const allowed = new Set([
    "gemstoneRs.launchExplorer",
    "gemstoneRs.verifySetup",
    "gemstoneRs.verifyLiveSetup",
    "gemstoneRs.verifyStrictSetup",
    "gemstoneRs.runSetupAssistant",
    "gemstoneRs.checkProjectProfiles",
    "gemstoneRs.codegenPreview",
    "gemstoneRs.codegenDiff",
    "gemstoneRs.codegenCheck",
    "gemstoneRs.codegenExplain",
    "gemstoneRs.openCodegenConfig",
    "gemstoneRs.openProjectProfiles",
    "gemstoneRs.openGeneratedOutput",
    "gemstoneRs.codegenGenerate",
    "gemstoneRs.codegenPreviewProfile",
    "gemstoneRs.codegenDiffProfile",
    "gemstoneRs.codegenCheckProfile",
    "gemstoneRs.codegenExplainProfile",
    "gemstoneRs.codegenGenerateProfile",
    "gemstoneRs.previewBridgeRoot",
    "gemstoneRs.listBridgeRootKeys",
    "gemstoneRs.openCodegenDocs",
    "gemstoneRs.validatePyNativeContract",
    "gemstoneRs.validatePyNativeSamplesFixture",
    "gemstoneRs.validatePyNativeSmokeFixture",
    "gemstoneRs.runPyNativeSmoke",
    "gemstoneRs.showPyNativeMigrationPlan",
    "gemstoneRs.validatePyNativeConformanceFixture",
    "gemstoneRs.validatePyNativeHandoffBundle",
    "gemstoneRs.showPyNativeHandoffBundle",
    "gemstoneRs.validatePyNativePublishReceipt",
    "gemstoneRs.showPyNativePublishReceipt",
    "gemstoneRs.validatePyNativeSharedCoreGate",
    "gemstoneRs.compareGemstonePyStatus",
    "gemstoneRs.compareAllStatus",
  ]);
  if (!allowed.has(commandId)) {
    vscode.window.showWarningMessage(`Unsupported gemstone-rs webview command: ${commandId}`);
    return;
  }
  await vscode.commands.executeCommand(commandId);
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

async function validatePyNativeContract() {
  const fixturePath = await vscode.window.showInputBox({
    title: "Validate py-native Contract",
    prompt: "Path to gemstone-rs.py-native.json",
    value: settings().pyNativeFixture,
  });
  if (!fixturePath) {
    return;
  }
  const result = await runCli(["py-native", "check", fixturePath, "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native check returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeContractReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const message = report.ok
    ? `py-native contract is current: ${report.path || fixturePath}`
    : `py-native contract drifted: ${report.path || fixturePath}`;
  const action = report.ok && result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report", "Open Fixture")
    : await vscode.window.showErrorMessage(message, "Copy Report", "Open Fixture");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native contract report.");
  } else if (action === "Open Fixture") {
    await openPathInEditor(fixturePath);
  }
}

async function validatePyNativeSamplesFixture() {
  const fixturePath = await vscode.window.showInputBox({
    title: "Validate py-native Samples Fixture",
    prompt: "Path to gemstone-rs.py-native-samples.json",
    value: settings().pyNativeSamplesFixture,
  });
  if (!fixturePath) {
    return;
  }
  const result = await runCli(["py-native", "check-samples", fixturePath, "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native check-samples returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeSamplesFixtureReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const message = report.ok
    ? `py-native samples fixture is current: ${report.path || fixturePath}`
    : `py-native samples fixture drifted: ${report.path || fixturePath}`;
  const action = report.ok && result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report", "Open Fixture")
    : await vscode.window.showErrorMessage(message, "Copy Report", "Open Fixture");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native samples fixture report.");
  } else if (action === "Open Fixture") {
    await openPathInEditor(fixturePath);
  }
}

async function validatePyNativeSmokeFixture() {
  const fixturePath = await vscode.window.showInputBox({
    title: "Validate py-native Smoke Fixture",
    prompt: "Path to gemstone-rs.py-native-smoke.json",
    value: settings().pyNativeSmokeFixture,
  });
  if (!fixturePath) {
    return;
  }
  const result = await runCli(["py-native", "check-smoke", fixturePath, "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native check-smoke returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeSmokeFixtureReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const message = report.ok
    ? `py-native smoke fixture is current: ${report.path || fixturePath}`
    : `py-native smoke fixture drifted: ${report.path || fixturePath}`;
  const action = report.ok && result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report", "Open Fixture")
    : await vscode.window.showErrorMessage(message, "Copy Report", "Open Fixture");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native smoke fixture report.");
  } else if (action === "Open Fixture") {
    await openPathInEditor(fixturePath);
  }
}

async function runPyNativeSmoke() {
  const mode = await vscode.window.showQuickPick(
    [
      {
        label: "Dry Run",
        description: "No GemStone login",
        args: ["--dry-run"],
      },
      {
        label: "Live GemStone",
        description: "Uses configured GS_* environment",
        args: [],
      },
    ],
    {
      title: "Run py-native Smoke",
      placeHolder: "Choose whether to run without or with a live GemStone login",
    }
  );
  if (!mode) {
    return;
  }

  const result = await runCli(["py-native", "smoke", ...mode.args, "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native smoke returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeSmokeReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const message = report.ok
    ? `py-native smoke passed: ${Array.isArray(report.steps) ? report.steps.length : 0} steps.`
    : "py-native smoke failed. See GemStone RS output.";
  const action = report.ok && result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report")
    : await vscode.window.showErrorMessage(message, "Copy Report");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native smoke report.");
  }
}

async function showPyNativeMigrationPlan() {
  const result = await runCli(["py-native", "migration", "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native migration returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeMigrationReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const stepCount = Array.isArray(report.steps) ? report.steps.length : 0;
  const message = `py-native migration plan: ${report.doneCount ?? 0} done, ${report.pendingCount ?? 0} pending across ${stepCount} steps.`;
  const action = result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report")
    : await vscode.window.showErrorMessage(message, "Copy Report");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native migration report.");
  }
}

async function validatePyNativeConformanceFixture() {
  const fixturePath = await vscode.window.showInputBox({
    title: "Validate py-native Conformance Fixture",
    prompt: "Path to gemstone-rs.py-native-conformance.json",
    value: settings().pyNativeConformanceFixture,
  });
  if (!fixturePath) {
    return;
  }
  const result = await runCli(["py-native", "check-conformance", fixturePath, "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native check-conformance returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeConformanceFixtureReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const message = report.ok
    ? `py-native conformance fixture is current: ${report.path || fixturePath}`
    : `py-native conformance fixture drifted: ${report.path || fixturePath}`;
  const action = report.ok && result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report", "Open Fixture")
    : await vscode.window.showErrorMessage(message, "Copy Report", "Open Fixture");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native conformance fixture report.");
  } else if (action === "Open Fixture") {
    await openPathInEditor(fixturePath);
  }
}

async function validatePyNativeHandoffBundle() {
  const fixturePath = await vscode.window.showInputBox({
    title: "Validate py-native Handoff Bundle",
    prompt: "Path to gemstone-rs.py-native-handoff.json",
    value: settings().pyNativeHandoffFixture,
  });
  if (!fixturePath) {
    return;
  }
  const result = await runCli(["py-native", "check-handoff", fixturePath, "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native check-handoff returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeHandoffCheckReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const message = report.ok
    ? `py-native handoff bundle is current: ${report.path || fixturePath}`
    : `py-native handoff bundle drifted: ${report.path || fixturePath}`;
  const action = report.ok && result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report", "Open Fixture")
    : await vscode.window.showErrorMessage(message, "Copy Report", "Open Fixture");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native handoff bundle report.");
  } else if (action === "Open Fixture") {
    await openPathInEditor(fixturePath);
  }
}

async function showPyNativeHandoffBundle() {
  const result = await runCli(["py-native", "handoff", "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native handoff returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeHandoffBundleReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const artifactCount = Array.isArray(report.artifacts) ? report.artifacts.length : 0;
  const acceptanceCount = Array.isArray(report.acceptance) ? report.acceptance.length : 0;
  const message = `py-native handoff bundle: ${artifactCount} artifacts, ${acceptanceCount} acceptance criteria.`;
  const action = result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report")
    : await vscode.window.showErrorMessage(message, "Copy Report");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native handoff bundle.");
  }
}

async function validatePyNativePublishReceipt() {
  const fixturePath = await vscode.window.showInputBox({
    title: "Validate py-native Publish Receipt",
    prompt: "Path to gemstone-rs.py-native-publish-receipt.json",
    value: settings().pyNativePublishReceiptFixture,
  });
  if (!fixturePath) {
    return;
  }
  const result = await runCli(["py-native", "check-publish-receipt", fixturePath, "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native check-publish-receipt returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativePublishReceiptCheckReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const message = report.ok
    ? `py-native publish receipt is current: ${report.path || fixturePath}`
    : `py-native publish receipt drifted: ${report.path || fixturePath}`;
  const action = report.ok && result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report", "Open Fixture")
    : await vscode.window.showErrorMessage(message, "Copy Report", "Open Fixture");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native publish receipt report.");
  } else if (action === "Open Fixture") {
    await openPathInEditor(fixturePath);
  }
}

async function showPyNativePublishReceipt() {
  const result = await runCli(["py-native", "publish-receipt", "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native publish-receipt returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativePublishReceiptReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const targetCount = Array.isArray(report.targets) ? report.targets.length : 0;
  const message = `py-native publish receipt: ${targetCount} targets for ${report.releaseTag || "unknown release"}.`;
  const action = result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report")
    : await vscode.window.showErrorMessage(message, "Copy Report");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native publish receipt.");
  }
}

async function validatePyNativeSharedCoreGate() {
  const result = await runCli(["py-native", "check-all", "--json"], { allowFailure: true });
  const report = parseJsonCommandResult(result, "gemstone-rs py-native check-all returned invalid JSON.");
  if (!report) {
    return;
  }
  const reportText = formatPyNativeSharedCoreGateReport(result, report);
  output.clear();
  output.append(reportText);
  output.show(true);

  const message = report.ok
    ? `py-native shared-core gate passed: ${report.okCount ?? 0}/${report.stepCount ?? 0} checks.`
    : `py-native shared-core gate failed: ${report.errorCount ?? 0} errors.`;
  const action = report.ok && result.code === 0
    ? await vscode.window.showInformationMessage(message, "Copy Report")
    : await vscode.window.showErrorMessage(message, "Copy Report");
  if (action === "Copy Report") {
    await vscode.env.clipboard.writeText(reportText);
    vscode.window.showInformationMessage("Copied py-native shared-core gate report.");
  }
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

function formatPyNativeContractReport(result, report) {
  const lines = [
    commandLine(result).trimEnd(),
    "py-native contract check",
    `path: ${report.path || "-"}`,
    `ok: ${Boolean(report.ok)}`,
    `contractVersion: ${report.contractVersion || "-"}`,
  ];
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativeSmokeFixtureReport(result, report) {
  const lines = [
    commandLine(result).trimEnd(),
    "py-native smoke fixture check",
    `path: ${report.path || "-"}`,
    `ok: ${Boolean(report.ok)}`,
    `contractVersion: ${report.contractVersion || "-"}`,
  ];
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativeSamplesFixtureReport(result, report) {
  const lines = [
    commandLine(result).trimEnd(),
    "py-native samples fixture check",
    `path: ${report.path || "-"}`,
    `ok: ${Boolean(report.ok)}`,
    `contractVersion: ${report.contractVersion || "-"}`,
    `valueCount: ${report.valueCount ?? "-"}`,
    `errorCount: ${report.errorCount ?? "-"}`,
  ];
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativeSmokeReport(result, report) {
  const steps = Array.isArray(report.steps) ? report.steps : [];
  const lines = [
    commandLine(result).trimEnd(),
    "py-native adapter smoke",
    `ok: ${Boolean(report.ok)}`,
    `dryRun: ${Boolean(report.dryRun)}`,
    `contractVersion: ${report.contractVersion || "-"}`,
    "",
  ];
  for (const step of steps) {
    lines.push(`${step.ok ? "ok" : "error"}\t${step.name || "(unnamed)"}`);
    lines.push(`  ${step.detail || ""}`);
  }
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativeMigrationReport(result, report) {
  const steps = Array.isArray(report.steps) ? report.steps : [];
  const lines = [
    commandLine(result).trimEnd(),
    "py-native gemstone-py migration",
    `targetPackage: ${report.targetPackage || "-"}`,
    `contractVersion: ${report.contractVersion || "-"}`,
    `status: ${report.status || "-"}`,
    `progress: ${report.doneCount ?? 0} done, ${report.pendingCount ?? 0} pending`,
    "",
  ];
  for (const step of steps) {
    lines.push(`${step.status || "unknown"}\t${step.id || "(unnamed)"}`);
    lines.push(`  ${step.title || ""}`);
    lines.push(`  ${step.detail || ""}`);
    lines.push(`  verify: ${step.verify || ""}`);
  }
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativeConformanceFixtureReport(result, report) {
  const lines = [
    commandLine(result).trimEnd(),
    "py-native conformance fixture check",
    `path: ${report.path || "-"}`,
    `ok: ${Boolean(report.ok)}`,
    `contractVersion: ${report.contractVersion || "-"}`,
    `moduleFunctions: ${report.moduleFunctionCount ?? "-"}`,
    `nativeSessionMethods: ${report.sessionMethodCount ?? "-"}`,
    `compatibilityMethods: ${report.compatibilityMethodCount ?? "-"}`,
    `fixtures: ${report.fixtureCount ?? "-"}`,
    `scaffoldFiles: ${report.scaffoldFileCount ?? "-"}`,
  ];
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativeHandoffCheckReport(result, report) {
  const lines = [
    commandLine(result).trimEnd(),
    "py-native handoff bundle check",
    `path: ${report.path || "-"}`,
    `ok: ${Boolean(report.ok)}`,
    `contractVersion: ${report.contractVersion || "-"}`,
    `artifacts: ${report.artifactCount ?? "-"}`,
    `acceptance: ${report.acceptanceCount ?? "-"}`,
  ];
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativeHandoffBundleReport(result, report) {
  const artifacts = Array.isArray(report.artifacts) ? report.artifacts : [];
  const acceptance = Array.isArray(report.acceptance) ? report.acceptance : [];
  const lines = [
    commandLine(result).trimEnd(),
    "py-native gemstone-py handoff",
    `targetPackage: ${report.targetPackage || "-"}`,
    `adapterModule: ${report.adapterModule || "-"}`,
    `scaffold: ${report.scaffold || "-"}`,
    `contractVersion: ${report.contractVersion || "-"}`,
    `status: ${report.status || "-"}`,
    "",
    "artifacts:",
  ];
  for (const artifact of artifacts) {
    lines.push(`  ${artifact.name || "(unnamed)"}`);
    lines.push(`    path: ${artifact.path || "-"}`);
    lines.push(`    schema: ${artifact.schema || "-"}`);
    lines.push(`    command: ${artifact.command || "-"}`);
    lines.push(`    check: ${artifact.checkCommand || "-"}`);
    lines.push(`    purpose: ${artifact.purpose || "-"}`);
  }
  lines.push("");
  lines.push("acceptance:");
  for (const criterion of acceptance) {
    lines.push(`  ${criterion.id || "(unnamed)"}${criterion.required ? " [required]" : ""}`);
    lines.push(`    verify: ${criterion.verify || "-"}`);
  }
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativePublishReceiptCheckReport(result, report) {
  const lines = [
    commandLine(result).trimEnd(),
    "py-native publish receipt check",
    `path: ${report.path || "-"}`,
    `ok: ${Boolean(report.ok)}`,
    `contractVersion: ${report.contractVersion || "-"}`,
    `releaseTag: ${report.releaseTag || "-"}`,
    `targets: ${report.targetCount ?? "-"}`,
  ];
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativePublishReceiptReport(result, report) {
  const targets = Array.isArray(report.targets) ? report.targets : [];
  const lines = [
    commandLine(result).trimEnd(),
    "py-native publish receipt",
    `targetPackage: ${report.targetPackage || "-"}`,
    `rustCore: ${report.rustCore || "-"}`,
    `releaseTag: ${report.releaseTag || "-"}`,
    `contractVersion: ${report.contractVersion || "-"}`,
    `status: ${report.status || "-"}`,
    "",
    "targets:",
  ];
  for (const target of targets) {
    lines.push(`  ${target.index || "(unnamed)"} ${target.package || "-"} ${target.version || "-"}`);
    lines.push(`    workflow: ${target.workflow || "-"} #${target.runId ?? "-"}`);
    lines.push(`    conclusion: ${target.conclusion || "-"}`);
    lines.push(`    created: ${target.createdAt || "-"}`);
    lines.push(`    verified: ${target.verifiedAt || "-"}`);
    lines.push(`    run: ${target.runUrl || "-"}`);
    lines.push(`    install: ${target.installCommand || "-"}`);
    lines.push(`    checks: ${target.verification || "-"}`);
  }
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatPyNativeSharedCoreGateReport(result, report) {
  const steps = Array.isArray(report.steps) ? report.steps : [];
  const lines = [
    commandLine(result).trimEnd(),
    "py-native shared-core gate",
    `root: ${report.root || "."}`,
    `ok: ${Boolean(report.ok)}`,
    `summary: ${report.okCount ?? 0} ok, ${report.errorCount ?? 0} errors, ${report.stepCount ?? steps.length} total`,
    "",
  ];
  for (const step of steps) {
    lines.push(`${step.ok ? "ok" : "error"}\t${step.name || "(unnamed)"}`);
    lines.push(`  path: ${step.path || "-"}`);
    lines.push(`  command: ${step.command || "-"}`);
    if (step.error) {
      lines.push(`  error: ${step.error}`);
    }
  }
  if (result.stderr.trim()) {
    lines.push("");
    lines.push(result.stderr.trimEnd());
  }
  return `${lines.join("\n")}\n`;
}

function formatProfileCheckReport(result, report) {
  const profiles = Array.isArray(report.profiles) ? report.profiles : [];
  const lines = [
    commandLine(result).trimEnd(),
    "Project profile freshness",
    `path: ${report.profileFile || report.path || "-"}`,
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
    lines.push(`  exists: ${Boolean(profile.exists)} upToDate: ${Boolean(profile.upToDate)}`);
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
  const url = explorerUrl(cfg);
  openExplorerWebview();
  output.appendLine("");
  output.appendLine(`Profile workflow: ${title}`);
  output.appendLine(instruction);
  output.appendLine(`Explorer URL: ${url}`);
  output.appendLine("Run GemStone RS: Launch Explorer first if the webview cannot connect.");
  output.show(true);
}

function explorerWebviewHtml(url, cfg = settings()) {
  const escaped = escapeHtml(url);
  const parsedUrl = new URL(url);
  const baseUrl = parsedUrl.origin;
  const state = {
    baseUrl,
    homeUrl: url,
    authToken: cfg.explorerAuthToken,
    codegenConfig: cfg.codegenConfig,
    codegenProfiles: cfg.codegenProfiles,
    bridgeRoot: cfg.bridgeRoot,
  };
  return `<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>gemstone-rs Explorer</title>
<style>
* { box-sizing: border-box; }
body { margin: 0; font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color: var(--vscode-foreground); background: var(--vscode-editor-background); }
button, input { font: inherit; }
button { border: 1px solid var(--vscode-button-border, transparent); background: var(--vscode-button-secondaryBackground); color: var(--vscode-button-secondaryForeground); border-radius: 4px; padding: 5px 8px; cursor: pointer; text-align: left; }
button.primary { background: var(--vscode-button-background); color: var(--vscode-button-foreground); }
button:hover { background: var(--vscode-button-secondaryHoverBackground); }
a { color: var(--vscode-textLink-foreground); }
.shell { display: grid; grid-template-rows: auto 1fr; height: 100vh; }
.bar { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 8px 10px; border-bottom: 1px solid var(--vscode-panel-border); }
.title { display: flex; flex-direction: column; gap: 2px; min-width: 220px; }
.title strong { font-size: 13px; }
.title span { color: var(--vscode-descriptionForeground); font-size: 11px; }
.bar-actions { display: flex; flex-wrap: wrap; gap: 6px; justify-content: flex-end; }
.layout { display: grid; grid-template-columns: minmax(260px, 330px) 1fr; min-height: 0; }
.rail { border-right: 1px solid var(--vscode-panel-border); overflow: auto; padding: 10px; display: flex; flex-direction: column; gap: 12px; }
.group { display: grid; gap: 6px; }
.group h2 { color: var(--vscode-descriptionForeground); font-size: 11px; font-weight: 600; letter-spacing: 0; margin: 0 0 2px; text-transform: uppercase; }
.field { display: grid; gap: 3px; color: var(--vscode-descriptionForeground); font-size: 11px; }
.field input { width: 100%; border: 1px solid var(--vscode-input-border, transparent); background: var(--vscode-input-background); color: var(--vscode-input-foreground); border-radius: 4px; padding: 5px 6px; }
.inspector { min-height: 160px; max-height: 34vh; overflow: auto; border: 1px solid var(--vscode-panel-border); border-radius: 4px; padding: 8px; background: var(--vscode-editorWidget-background); color: var(--vscode-editorWidget-foreground); font-size: 11px; }
.inspector pre { margin: 0; white-space: pre-wrap; font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace; }
.inspector.error { color: var(--vscode-errorForeground); }
.inspector.rich { white-space: normal; }
.result-title { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin: 0 0 8px; color: var(--vscode-foreground); font-size: 12px; font-weight: 700; }
.muted { color: var(--vscode-descriptionForeground); }
.summary-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(110px, 1fr)); gap: 6px; margin: 8px 0; }
.metric { border: 1px solid var(--vscode-panel-border); border-radius: 4px; padding: 6px; background: var(--vscode-editor-background); }
.metric strong { display: block; color: var(--vscode-foreground); font-size: 13px; }
.metric span { color: var(--vscode-descriptionForeground); }
.profile-table { width: 100%; border-collapse: collapse; margin-top: 8px; }
.profile-table th, .profile-table td { border: 1px solid var(--vscode-panel-border); padding: 5px; text-align: left; vertical-align: top; }
.profile-table th { color: var(--vscode-descriptionForeground); font-weight: 600; }
.profile-actions { display: flex; flex-wrap: wrap; gap: 4px; }
.status-pill { display: inline-block; border-radius: 999px; padding: 1px 6px; font-weight: 700; }
.status-ok { color: var(--vscode-testing-iconPassed); }
.status-stale { color: var(--vscode-testing-iconQueued); }
.status-error { color: var(--vscode-errorForeground); }
.diff-line { display: block; white-space: pre-wrap; font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace; }
.diff-add { color: var(--vscode-gitDecoration-addedResourceForeground); }
.diff-remove { color: var(--vscode-gitDecoration-deletedResourceForeground); }
.diff-meta { color: var(--vscode-descriptionForeground); }
.key-list { display: grid; gap: 4px; margin: 8px 0 0; }
.key-row { display: grid; grid-template-columns: minmax(120px, 1fr) auto; gap: 6px; border: 1px solid var(--vscode-panel-border); border-radius: 4px; padding: 5px; }
.browse-list { display: grid; gap: 4px; margin-top: 8px; }
.browse-row { display: grid; grid-template-columns: minmax(120px, 1fr) auto; align-items: center; gap: 6px; border: 1px solid var(--vscode-panel-border); border-radius: 4px; padding: 5px; }
.browse-row button { padding: 3px 6px; }
.source-actions { display: flex; flex-wrap: wrap; gap: 6px; margin: 6px 0; }
.source-editor { width: 100%; min-height: 360px; box-sizing: border-box; resize: vertical; font-family: var(--vscode-editor-font-family); font-size: var(--vscode-editor-font-size); color: var(--vscode-editor-foreground); background: var(--vscode-editor-background); border: 1px solid var(--vscode-panel-border); border-radius: 4px; padding: 8px; }
.save-state { color: var(--vscode-descriptionForeground); margin-top: 6px; }
.content { min-width: 0; min-height: 0; display: grid; grid-template-rows: auto 1fr; }
.iframe-tabs { display: flex; gap: 6px; padding: 8px; border-bottom: 1px solid var(--vscode-panel-border); overflow-x: auto; }
iframe { display: block; width: 100%; height: 100%; border: 0; background: white; }
@media (max-width: 760px) {
  .layout { grid-template-columns: 1fr; }
  .rail { border-right: 0; border-bottom: 1px solid var(--vscode-panel-border); max-height: 46vh; }
}
</style>
</head>
<body>
<div class="shell">
<div class="bar">
  <div class="title">
    <strong>gemstone-rs Explorer Workbench</strong>
    <span id="status">Explorer URL: ${escaped}</span>
  </div>
  <div class="bar-actions">
    <button class="primary" data-command="gemstoneRs.launchExplorer">Launch Explorer</button>
    <button data-action="refresh">Refresh</button>
    <button data-action="open-browser">Open Browser</button>
  </div>
</div>
<div class="layout">
  <aside class="rail">
    <div class="group">
      <h2>Project</h2>
      <label class="field">Codegen config<input id="codegenConfig" value="${escapeHtml(cfg.codegenConfig)}"></label>
      <label class="field">Profile file<input id="codegenProfiles" value="${escapeHtml(cfg.codegenProfiles)}"></label>
      <label class="field">BridgeRoot<input id="bridgeRoot" value="${escapeHtml(cfg.bridgeRoot)}"></label>
      <button data-command="gemstoneRs.openCodegenDocs">Open Codegen Docs</button>
      <button data-command="gemstoneRs.validatePyNativeContract">Validate py-native Contract</button>
      <button data-command="gemstoneRs.validatePyNativeSamplesFixture">Validate py-native Samples Fixture</button>
      <button data-command="gemstoneRs.validatePyNativeSmokeFixture">Validate py-native Smoke Fixture</button>
      <button data-command="gemstoneRs.runPyNativeSmoke">Run py-native Smoke</button>
      <button data-command="gemstoneRs.showPyNativeMigrationPlan">Show py-native Migration Plan</button>
      <button data-command="gemstoneRs.validatePyNativeConformanceFixture">Validate py-native Conformance Fixture</button>
      <button data-command="gemstoneRs.validatePyNativeHandoffBundle">Validate py-native Handoff Bundle</button>
      <button data-command="gemstoneRs.showPyNativeHandoffBundle">Show py-native Handoff Bundle</button>
      <button data-command="gemstoneRs.validatePyNativePublishReceipt">Validate py-native Publish Receipt</button>
      <button data-command="gemstoneRs.showPyNativePublishReceipt">Show py-native Publish Receipt</button>
      <button data-command="gemstoneRs.validatePyNativeSharedCoreGate">Validate py-native Shared Core Gate</button>
      <button data-command="gemstoneRs.openCodegenConfig">Open Codegen Config</button>
      <button data-command="gemstoneRs.openProjectProfiles">Open Project Profiles</button>
      <button data-command="gemstoneRs.checkProjectProfiles">Check Project Profiles in VS Code</button>
    </div>
    <div class="group">
      <h2>Live Browse</h2>
      <label class="field">Dictionary<input id="browseDictionary" value="UserGlobals"></label>
      <label class="field">Class<input id="browseClass" value="Object"></label>
      <label class="field">Protocol<input id="browseProtocol" value="-- all --"></label>
      <label class="field">Selector<input id="browseSelector" value="printString"></label>
      <button data-browse="/api/browse/dictionaries">Dictionaries</button>
      <button data-browse="/api/browse/classes">Classes</button>
      <button data-browse="/api/browse/protocols">Protocols</button>
      <button data-browse="/api/browse/methods">Methods</button>
      <button data-browse="/api/browse/source">Open Source</button>
    </div>
    <div class="group">
      <h2>Comparison</h2>
      <button data-command="gemstoneRs.compareGemstonePyStatus">Compare with gemstone-py</button>
      <button data-command="gemstoneRs.compareAllStatus">Show All Comparison Status</button>
    </div>
    <div class="group">
      <h2>Live Inspector</h2>
      <button data-probe="/api/status">Explorer Status</button>
      <button data-probe="/api/setup/assistant">Setup Assistant</button>
      <button data-probe="/api/codegen/profiles/check">Profile Status</button>
      <button data-probe="/api/bridge/root">BridgeRoot</button>
      <button data-probe="/api/bridge/keys">BridgeRoot Keys</button>
      <button data-open-last>Open Last Output File</button>
    </div>
    <div class="group">
      <h2>Codegen</h2>
      <button data-probe="/api/codegen/explain">Explain Config</button>
      <button data-probe="/api/codegen/preview">Preview/Edit Generated Wrappers</button>
      <button data-probe="/api/codegen/output">Read/Edit Generated Output</button>
      <button data-probe="/api/codegen/diff">Diff Generated Wrappers</button>
      <button data-probe="/api/codegen/check">Check Freshness</button>
      <button data-command="gemstoneRs.codegenPreview">Preview in Editor</button>
      <button data-command="gemstoneRs.codegenDiff">Diff in Editor</button>
      <button data-command="gemstoneRs.openCodegenConfig">Open Config File</button>
      <button data-command="gemstoneRs.openGeneratedOutput">Open Output File</button>
      <button data-command="gemstoneRs.codegenGenerate">Generate with Confirmation</button>
    </div>
    <div class="group">
      <h2>Profile Codegen</h2>
      <button data-probe="/api/codegen/explain-profile">Explain Profile</button>
      <button data-probe="/api/codegen/preview-profile">Preview/Edit Profile</button>
      <button data-probe="/api/codegen/output-profile">Read/Edit Profile Output</button>
      <button data-probe="/api/codegen/diff-profile">Diff Profile</button>
      <button data-probe="/api/codegen/check-profile">Check Profile</button>
      <button data-command="gemstoneRs.codegenPreviewProfile">Preview Profile in Editor</button>
      <button data-command="gemstoneRs.codegenGenerateProfile">Generate Profile with Confirmation</button>
      <button data-command="gemstoneRs.openProjectProfiles">Open Profile File</button>
    </div>
    <div id="inspector" class="inspector"><pre>Use the inspector buttons to query the running explorer. The iframe remains the full explorer UI.</pre></div>
  </aside>
  <main class="content">
    <div class="iframe-tabs">
      <button data-nav="/">Home</button>
      <button data-nav="/#browse">Browse</button>
      <button data-nav="/#bridge-codegen">BridgeRoot</button>
      <button data-nav="/#codegen-workflow">Codegen Workflow</button>
      <button data-nav="/api/codegen/profiles/check">Profile Status JSON</button>
    </div>
    <iframe id="explorerFrame" src="${escaped}" title="gemstone-rs Explorer"></iframe>
  </main>
</div>
</div>
<script>
const vscode = acquireVsCodeApi();
const state = ${jsLiteral(state)};
const frame = document.getElementById('explorerFrame');
const inspector = document.getElementById('inspector');
const status = document.getElementById('status');
let lastOutputFile = '';

function configValue(id) {
  return document.getElementById(id).value.trim();
}

function apiUrl(path) {
  const url = new URL(path, state.baseUrl + '/');
  if (state.authToken && !url.searchParams.has('token')) {
    url.searchParams.set('token', state.authToken);
  }
  const config = configValue('codegenConfig') || state.codegenConfig;
  const profiles = configValue('codegenProfiles') || state.codegenProfiles;
  const bridgeRoot = configValue('bridgeRoot') || state.bridgeRoot;
  if (path.includes('/api/codegen/') && !url.searchParams.has('config')) {
    url.searchParams.set('config', config);
  }
  if ((path.includes('profile') || path.includes('/api/setup/assistant')) && !url.searchParams.has('profile_file')) {
    url.searchParams.set('profile_file', profiles);
  }
  if (path.includes('-profile') && !url.searchParams.has('profile')) {
    url.searchParams.set('profile', 'default');
  }
  if (path.includes('/api/bridge/') && bridgeRoot && !url.searchParams.has('root')) {
    url.searchParams.set('root', bridgeRoot);
  }
  return url;
}

function browseUrl(path) {
  const url = apiUrl(path);
  const dictionary = configValue('browseDictionary') || 'UserGlobals';
  const className = configValue('browseClass') || 'Object';
  const protocol = configValue('browseProtocol') || '-- all --';
  const selector = configValue('browseSelector') || 'printString';
  if (path.includes('/api/browse/classes')) {
    url.searchParams.set('dictionary', dictionary);
  }
  if (path.includes('/api/browse/protocols')) {
    url.searchParams.set('dictionary', dictionary);
    url.searchParams.set('class', className);
  }
  if (path.includes('/api/browse/methods')) {
    url.searchParams.set('dictionary', dictionary);
    url.searchParams.set('class', className);
    url.searchParams.set('protocol', protocol);
  }
  if (path.includes('/api/browse/source')) {
    url.searchParams.set('dictionary', dictionary);
    url.searchParams.set('class', className);
    url.searchParams.set('selector', selector);
  }
  return url;
}

function setInspector(text, isError) {
  inspector.className = isError ? 'inspector error' : 'inspector';
  inspector.innerHTML = '<pre>' + escapeHtml(text) + '</pre>';
}

function setInspectorHtml(html, isError) {
  inspector.className = isError ? 'inspector rich error' : 'inspector rich';
  inspector.innerHTML = html;
}

function recordOutputPath(parsed) {
  if (!parsed || typeof parsed !== 'object') return;
  if (typeof parsed.output === 'string') {
    lastOutputFile = parsed.output;
  }
  if (parsed.explain && typeof parsed.explain.output === 'string') {
    lastOutputFile = parsed.explain.output;
  }
  if (Array.isArray(parsed.profiles)) {
    const firstOutput = parsed.profiles.find(profile => profile && typeof profile.output === 'string');
    if (firstOutput) lastOutputFile = firstOutput.output;
  }
}

function renderProbeResult(parsed, ok) {
  if (typeof parsed === 'string') {
    setInspector(parsed, !ok);
  } else if (parsed && typeof parsed === 'object') {
    recordOutputPath(parsed);
    if (Array.isArray(parsed.profiles) && typeof parsed.profileCount === 'number') {
      renderProfileStatus(parsed, ok);
    } else if (Array.isArray(parsed.dictionaries)) {
      renderBrowseList('Dictionaries', parsed.dictionaries, 'dictionary', ok);
    } else if (Array.isArray(parsed.classes)) {
      renderBrowseList('Classes', parsed.classes, 'class', ok);
    } else if (Array.isArray(parsed.protocols)) {
      renderBrowseList('Protocols', parsed.protocols, 'protocol', ok);
    } else if (Array.isArray(parsed.methods)) {
      renderBrowseList('Methods', parsed.methods, 'selector', ok);
    } else if (parsed.view === 'status' && (parsed.remaining || Array.isArray(parsed.comparisons))) {
      renderComparisonStatus(parsed, ok);
    } else if (Array.isArray(parsed.steps)) {
      renderSetupAssistant(parsed, ok);
    } else if (parsed.explain && typeof parsed.explain === 'object') {
      renderCodegenExplain(parsed.explain, ok);
    } else if (typeof parsed.diff === 'string') {
      renderDiff(parsed.diff || 'No generated output changes.', ok);
    } else if (typeof parsed.source === 'string' && typeof parsed.output === 'string') {
      renderGeneratedSource(parsed, ok);
    } else if (typeof parsed.config === 'string') {
      setInspectorHtml(resultTitle('Codegen Config') + '<pre>' + escapeHtml(parsed.config) + '</pre>', !ok);
    } else if (typeof parsed.source === 'string') {
      renderBrowseSource(parsed, ok);
    } else if (Array.isArray(parsed.keys)) {
      renderBridgeKeys(parsed, ok);
    } else if (parsed.value && typeof parsed.value === 'object') {
      renderBridgeValue(parsed, ok);
    } else if (parsed.name && typeof parsed.oop !== 'undefined' && typeof parsed.identityId !== 'undefined') {
      renderBridgeRoot(parsed, ok);
    } else if (typeof parsed.error === 'string') {
      setInspectorHtml(resultTitle('Explorer Error') + '<pre>' + escapeHtml(parsed.error) + '</pre>', true);
    } else {
      renderJsonResult(parsed, ok);
    }
  } else {
    setInspector(String(parsed), !ok);
  }
}

function resultTitle(title, extra) {
  return '<div class="result-title"><span>' + escapeHtml(title) + '</span><span class="muted">' + escapeHtml(extra || '') + '</span></div>';
}

function metric(label, value) {
  return '<div class="metric"><strong>' + escapeHtml(value) + '</strong><span>' + escapeHtml(label) + '</span></div>';
}

function renderProfileStatus(data, ok) {
  const profiles = Array.isArray(data.profiles) ? data.profiles : [];
  const rows = profiles.map(profile => {
    const name = profile.name || '(unnamed)';
    const status = profile.ok ? 'ok' : (profile.error ? 'error' : 'stale');
    const statusClass = profile.ok ? 'status-ok' : (profile.error ? 'status-error' : 'status-stale');
    return '<tr>' +
      '<td><strong>' + escapeHtml(name) + '</strong></td>' +
      '<td><span class="status-pill ' + statusClass + '">' + escapeHtml(status) + '</span></td>' +
      '<td>' + escapeHtml(profile.config || '-') + '</td>' +
      '<td>' + escapeHtml(profile.output || '-') + '</td>' +
      '<td>' + escapeHtml('exists=' + Boolean(profile.exists) + ' upToDate=' + Boolean(profile.upToDate)) + (profile.error ? '<br><span class="status-error">' + escapeHtml(profile.error) + '</span>' : '') + '</td>' +
      '<td><div class="profile-actions">' +
        profileActionButton(name, 'preview') +
        profileActionButton(name, 'diff') +
        profileActionButton(name, 'check') +
        profileActionButton(name, 'generate') +
      '</div></td>' +
    '</tr>';
  }).join('');
  const table = rows
    ? '<table class="profile-table"><thead><tr><th>Profile</th><th>Status</th><th>Config</th><th>Output</th><th>Freshness</th><th>Actions</th></tr></thead><tbody>' + rows + '</tbody></table>'
    : '<p class="muted">No project profiles reported.</p>';
  setInspectorHtml(
    resultTitle('Project Profile Status', data.profileFile || data.path || '-') +
    '<div class="summary-grid">' +
      metric('ok', Number(data.okCount || 0)) +
      metric('stale', Number(data.staleCount || 0)) +
      metric('errors', Number(data.errorCount || 0)) +
      metric('total', Number(data.profileCount || profiles.length)) +
    '</div>' +
    table,
    !ok
  );
  inspector.querySelectorAll('button[data-profile-action]').forEach(button => {
    button.addEventListener('click', () => runProfileProbe(button.dataset.profile, button.dataset.profileAction));
  });
}

function profileActionButton(profile, action) {
  return '<button data-profile-action="' + escapeHtml(action) + '" data-profile="' + escapeHtml(profile) + '">' + escapeHtml(action) + '</button>';
}

function runProfileProbe(profile, action) {
  const pathByAction = {
    preview: '/api/codegen/preview-profile',
    diff: '/api/codegen/diff-profile',
    check: '/api/codegen/check-profile',
    generate: '/api/codegen/generate-profile'
  };
  const path = pathByAction[action];
  if (!path) return;
  const url = new URL(path, state.baseUrl + '/');
  url.searchParams.set('profile', profile);
  probe(url.pathname + url.search);
}

function renderCodegenExplain(explain, ok) {
  const classes = Array.isArray(explain.classes) ? explain.classes : [];
  const mapped = Array.isArray(explain.mapped) ? explain.mapped : [];
  const classRows = classes.map(cls => {
    const methods = Array.isArray(cls.methods) ? cls.methods : [];
    return '<tr><td><strong>' + escapeHtml(cls.name || cls.className || '-') + '</strong></td><td>' + methods.length + '</td><td>' + escapeHtml(methods.map(method => method.selector).join(', ') || '-') + '</td></tr>';
  }).join('');
  const mappedRows = mapped.map(entry => {
    const fields = Array.isArray(entry.fields) ? entry.fields : [];
    return '<tr><td><strong>' + escapeHtml(entry.name || '-') + '</strong></td><td>' + fields.length + '</td><td>' + escapeHtml(fields.map(field => field.name + ':' + field.type).join(', ') || '-') + '</td></tr>';
  }).join('');
  setInspectorHtml(
    resultTitle('Codegen Explain', explain.output || '-') +
    '<div class="summary-grid">' +
      metric('classes', classes.length) +
      metric('mappings', mapped.length) +
      metric('test stubs', (explain.testStubs || []).length) +
    '</div>' +
    '<p><strong>Output:</strong> ' + escapeHtml(explain.output || '-') + '</p>' +
    '<p><strong>Test stubs:</strong> ' + escapeHtml((explain.testStubs || []).join(', ') || '-') + '</p>' +
    '<table class="profile-table"><thead><tr><th>Class</th><th>Methods</th><th>Selectors</th></tr></thead><tbody>' + classRows + '</tbody></table>' +
    '<table class="profile-table"><thead><tr><th>Mapped Type</th><th>Fields</th><th>Field Types</th></tr></thead><tbody>' + mappedRows + '</tbody></table>',
    !ok
  );
}

function renderDiff(diff, ok) {
  const lines = diff.split('\\n').map(line => {
    let cls = 'diff-line';
    if (line.startsWith('+') && !line.startsWith('+++')) cls += ' diff-add';
    else if (line.startsWith('-') && !line.startsWith('---')) cls += ' diff-remove';
    else if (line.startsWith('@@') || line.startsWith('diff ') || line.startsWith('---') || line.startsWith('+++')) cls += ' diff-meta';
    return '<span class="' + cls + '">' + escapeHtml(line || ' ') + '</span>';
  }).join('');
  setInspectorHtml(resultTitle('Generated Diff') + lines, !ok);
}

function renderGeneratedSource(data, ok) {
  const outputPath = data.output || lastOutputFile || '';
  const title = data.profile ? 'Profile Generated Output' : 'Generated Output';
  const source = data.source || '';
  if (outputPath) lastOutputFile = outputPath;
  setInspectorHtml(
    resultTitle(title, outputPath || 'no output path') +
    '<div class="source-actions">' +
      '<button data-open-generated-file>Open Output File</button>' +
      '<button data-open-generated-draft>Open Editable Draft</button>' +
      '<button data-save-generated-output' + (outputPath ? '' : ' disabled') + '>Save Edited Output</button>' +
    '</div>' +
    '<textarea id="generatedSourceEditor" class="source-editor" spellcheck="false">' + escapeHtml(source) + '</textarea>' +
    '<div id="generatedSaveState" class="save-state">Edit the generated wrappers here, then save to the configured output file or open a draft editor.</div>',
    !ok
  );
  const editor = document.getElementById('generatedSourceEditor');
  const saveState = document.getElementById('generatedSaveState');
  inspector.querySelector('[data-open-generated-file]')?.addEventListener('click', () => {
    if (!outputPath) {
      saveState.textContent = 'No output file has been reported yet.';
      return;
    }
    vscode.postMessage({ command: 'openPath', path: outputPath });
  });
  inspector.querySelector('[data-open-generated-draft]')?.addEventListener('click', () => {
    vscode.postMessage({
      command: 'openDocument',
      language: 'rust',
      content: editor.value
    });
  });
  inspector.querySelector('[data-save-generated-output]')?.addEventListener('click', () => {
    if (!outputPath) {
      saveState.textContent = 'No output file has been reported yet.';
      return;
    }
    saveState.textContent = 'Waiting for VS Code save confirmation...';
    vscode.postMessage({
      command: 'saveGeneratedOutput',
      path: outputPath,
      content: editor.value
    });
  });
  editor.addEventListener('input', () => {
    saveState.textContent = 'Edited in webview. Save writes the configured generated output file.';
  });
}

function renderSetupAssistant(data, ok) {
  const rows = (data.steps || []).map(step => {
    const good = Boolean(step.ok);
    return '<tr><td><span class="status-pill ' + (good ? 'status-ok' : 'status-error') + '">' + (good ? 'ok' : 'needs attention') + '</span></td><td><strong>' + escapeHtml(step.name || '-') + '</strong></td><td>' + escapeHtml(step.detail || '-') + '</td><td>' + escapeHtml(step.action || '-') + '</td></tr>';
  }).join('');
  setInspectorHtml(
    resultTitle('Setup Assistant') +
    '<table class="profile-table"><thead><tr><th>Status</th><th>Step</th><th>Detail</th><th>Action</th></tr></thead><tbody>' + rows + '</tbody></table>',
    !ok
  );
}

function renderBridgeRoot(data, ok) {
  setInspectorHtml(
    resultTitle('BridgeRoot') +
    '<div class="summary-grid">' +
      metric('name', data.name || '-') +
      metric('oop', String(data.oop || '-')) +
      metric('identity id', String(data.identityId || '-')) +
    '</div>',
    !ok
  );
}

function renderBridgeKeys(data, ok) {
  const keys = Array.isArray(data.keys) ? data.keys : [];
  const rows = keys.map(key =>
    '<div class="key-row"><span><strong>' + escapeHtml(key.printString || '-') + '</strong><br><span class="muted">oop=' + escapeHtml(key.oop || '-') + ' class=' + escapeHtml(key.classOop || '-') + '</span></span><span class="muted">identity=' + escapeHtml(key.identityId || '-') + '</span></div>'
  ).join('');
  setInspectorHtml(
    resultTitle('BridgeRoot Keys', data.root || '-') +
    (rows ? '<div class="key-list">' + rows + '</div>' : '<p class="muted">No keys reported.</p>'),
    !ok
  );
}

function renderBrowseList(title, values, targetField, ok) {
  const rows = values.map(value =>
    '<div class="browse-row"><span>' + escapeHtml(value) + '</span><button data-browse-pick="' + escapeHtml(targetField) + '" data-value="' + escapeHtml(value) + '">Use</button></div>'
  ).join('');
  setInspectorHtml(
    resultTitle(title, values.length + ' result' + (values.length === 1 ? '' : 's')) +
    (rows ? '<div class="browse-list">' + rows + '</div>' : '<p class="muted">No values reported.</p>'),
    !ok
  );
  inspector.querySelectorAll('button[data-browse-pick]').forEach(button => {
    button.addEventListener('click', () => {
      const value = button.dataset.value || '';
      const field = button.dataset.browsePick;
      if (field === 'dictionary') document.getElementById('browseDictionary').value = value;
      else if (field === 'class') document.getElementById('browseClass').value = value;
      else if (field === 'protocol') document.getElementById('browseProtocol').value = value;
      else if (field === 'selector') document.getElementById('browseSelector').value = value;
    });
  });
}

function renderBrowseSource(data, ok) {
  const title = [data.class, data.selector].filter(Boolean).join('>>') || 'GemStone Source';
  setInspectorHtml(
    resultTitle('Method Source', title) +
    '<div class="source-actions"><button data-open-source-editor>Open Source in Editor</button></div>' +
    '<pre>' + escapeHtml(data.source) + '</pre>',
    !ok
  );
  const button = inspector.querySelector('button[data-open-source-editor]');
  if (button) {
    button.addEventListener('click', () => vscode.postMessage({
      command: 'openDocument',
      language: 'smalltalk',
      content: data.source
    }));
  }
}

function renderBridgeValue(data, ok) {
  const value = data.value || {};
  setInspectorHtml(
    resultTitle('BridgeRoot Value', data.root || '-') +
    '<div class="summary-grid">' +
      metric('key', data.key || '-') +
      metric('key type', data.keyType || '-') +
      metric('oop', String(value.oop || '-')) +
      metric('class oop', String(value.classOop || '-')) +
    '</div>' +
    '<p><strong>printString:</strong></p><pre>' + escapeHtml(value.printString || '-') + '</pre>',
    !ok
  );
}

function renderComparisonStatus(data, ok) {
  const entries = Array.isArray(data.comparisons) ? data.comparisons : [data];
  const cards = entries.map(entry => {
    const parity = entry.parity || {};
    const remaining = entry.remaining || {};
    const nextBatch = entry.nextBatch || {};
    const topGap = entry.topGap || {};
    return '<div class="metric">' +
      '<strong>' + escapeHtml(entry.comparison || 'comparison') + '</strong>' +
      '<p>' + escapeHtml(entry.answer || '-') + '</p>' +
      '<p>Parity: gemstone-py ' + Number(parity.gemstonePyScore || 0) + '/' + Number(parity.maxScore || 0) + '; ' + escapeHtml(parity.project || topGap.project || 'gemstone-rs') + ' ' + Number(parity.projectScore || 0) + '/' + Number(parity.maxScore || 0) + '; gap ' + Number(parity.scoreGap || 0) + '</p>' +
      '<p>Remaining: ' + Number(remaining.totalBatches || 0) + ' batches, ' + Number(remaining.hoursMin || 0) + '-' + Number(remaining.hoursMax || 0) + ' hours</p>' +
      '<p>Next: ' + escapeHtml(nextBatch.focus || '-') + '</p>' +
      '<p>Top gap: ' + escapeHtml((topGap.priority || '-') + ' ' + (topGap.area || '-')) + '</p>' +
    '</div>';
  }).join('');
  setInspectorHtml(resultTitle('Comparison Status') + cards, !ok);
}

function renderJsonResult(data, ok) {
  setInspectorHtml(resultTitle(data.success === false ? 'Explorer Response' : 'Explorer JSON') + '<pre>' + escapeHtml(JSON.stringify(data, null, 2)) + '</pre>', !ok || data.success === false);
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

async function probe(path) {
  const url = apiUrl(path);
  status.textContent = 'GET ' + url.pathname + url.search;
  setInspector('Loading ' + url.href + ' ...', false);
  try {
    const response = await fetch(url.href);
    const text = await response.text();
    let parsed;
    try {
      parsed = JSON.parse(text);
    } catch {
      parsed = text;
    }
    renderProbeResult(parsed, response.ok);
  } catch (error) {
    setInspector('Could not reach explorer at ' + state.baseUrl + '\\n' + error.message + '\\n\\nRun GemStone RS: Launch Explorer first.', true);
  }
}

function navigate(path) {
  const url = new URL(path, state.baseUrl + '/');
  if (state.authToken && !url.searchParams.has('token')) {
    url.searchParams.set('token', state.authToken);
  }
  frame.src = url.href;
}

async function browseProbe(path) {
  const url = browseUrl(path);
  status.textContent = 'GET ' + url.pathname + url.search;
  setInspector('Loading ' + url.href + ' ...', false);
  try {
    const response = await fetch(url.href);
    const text = await response.text();
    let parsed;
    try {
      parsed = JSON.parse(text);
    } catch {
      parsed = text;
    }
    renderProbeResult(parsed, response.ok);
  } catch (error) {
    setInspector('Could not reach explorer at ' + state.baseUrl + '\\n' + error.message + '\\n\\nRun GemStone RS: Launch Explorer first.', true);
  }
}

document.querySelectorAll('[data-command]').forEach(button => {
  button.addEventListener('click', () => vscode.postMessage({
    command: 'runWorkbenchCommand',
    id: button.dataset.command
  }));
});

document.querySelectorAll('[data-probe]').forEach(button => {
  button.addEventListener('click', () => probe(button.dataset.probe));
});

document.querySelectorAll('[data-browse]').forEach(button => {
  button.addEventListener('click', () => browseProbe(button.dataset.browse));
});

document.querySelectorAll('[data-nav]').forEach(button => {
  button.addEventListener('click', () => navigate(button.dataset.nav));
});

document.querySelector('[data-action="refresh"]').addEventListener('click', () => {
  frame.src = frame.src;
});

document.querySelector('[data-action="open-browser"]').addEventListener('click', () => {
  vscode.postMessage({ command: 'openExternal', url: state.homeUrl });
});

document.querySelector('[data-open-last]').addEventListener('click', () => {
  if (!lastOutputFile) {
    setInspector('No output file has been reported yet. Run Check, Explain, Preview, or Diff first.', true);
    return;
  }
  vscode.postMessage({ command: 'openPath', path: lastOutputFile });
});

window.addEventListener('message', event => {
  const message = event.data || {};
  if (message.command !== 'generatedOutputSaved') return;
  const saveState = document.getElementById('generatedSaveState');
  if (!saveState) return;
  if (message.ok) {
    saveState.textContent = 'Saved ' + Number(message.bytes || 0) + ' bytes to ' + (message.path || 'generated output') + '.';
  } else {
    saveState.textContent = 'Save did not complete: ' + (message.error || 'unknown error');
  }
});
</script>
</body>
</html>`;
}

function jsLiteral(value) {
  return JSON.stringify(value).replace(/</g, "\\u003c");
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

async function openComparisonGuide() {
  const cfg = settings();
  const docsPath = resolvePath("docs/gemstone-py-vs-gemstone-rs.md", cfg.cwd);
  if (fs.existsSync(docsPath)) {
    const document = await vscode.workspace.openTextDocument(docsPath);
    await vscode.window.showTextDocument(document, { preview: true });
  } else {
    vscode.env.openExternal(vscode.Uri.parse("https://github.com/unicompute/gemstone-rs/blob/main/docs/gemstone-py-vs-gemstone-rs.md"));
  }
}

async function openExamplesGuide() {
  const cfg = settings();
  const docsPath = resolvePath("examples/README.md", cfg.cwd);
  if (fs.existsSync(docsPath)) {
    const document = await vscode.workspace.openTextDocument(docsPath);
    await vscode.window.showTextDocument(document, { preview: true });
  } else {
    vscode.env.openExternal(vscode.Uri.parse("https://github.com/unicompute/gemstone-rs/tree/main/examples"));
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
    pyNativeFixture: cfg.get("pyNativeFixture", "examples/py-native/gemstone-rs.py-native.json"),
    pyNativeSamplesFixture: cfg.get("pyNativeSamplesFixture", "examples/py-native/gemstone-rs.py-native-samples.json"),
    pyNativeSmokeFixture: cfg.get("pyNativeSmokeFixture", "examples/py-native/gemstone-rs.py-native-smoke.json"),
    pyNativeConformanceFixture: cfg.get("pyNativeConformanceFixture", "examples/py-native/gemstone-rs.py-native-conformance.json"),
    pyNativeHandoffFixture: cfg.get("pyNativeHandoffFixture", "examples/py-native/gemstone-rs.py-native-handoff.json"),
    pyNativePublishReceiptFixture: cfg.get("pyNativePublishReceiptFixture", "examples/py-native/gemstone-rs.py-native-publish-receipt.json"),
    bridgeRoot: cfg.get("bridgeRoot", "GemStoneRsBridgeRoot").trim() || "GemStoneRsBridgeRoot",
    explorerHost: cfg.get("explorerHost", "127.0.0.1"),
    explorerPort: cfg.get("explorerPort", 8787),
    explorerAuthToken: cfg.get("explorerAuthToken", "").trim(),
  };
}

function explorerUrl(cfg = settings(), route = "/") {
  const url = new URL(route, `http://${cfg.explorerHost}:${cfg.explorerPort}/`);
  if (cfg.explorerAuthToken) {
    url.searchParams.set("token", cfg.explorerAuthToken);
  }
  return url.toString();
}

function explorerUrlWithParams(cfg, route, params) {
  const url = new URL(explorerUrl(cfg, route));
  for (const [name, value] of Object.entries(params || {})) {
    if (value) {
      url.searchParams.set(name, value);
    }
  }
  return url.toString();
}

function bridgeRootArgs(cfg = settings()) {
  return ["--root", cfg.bridgeRoot];
}

function explorerAuthArgs(cfg = settings()) {
  return cfg.explorerAuthToken ? ["--auth-token-env", "GEMSTONE_RS_EXPLORER_TOKEN"] : [];
}

function explorerTerminalEnv(cfg = settings()) {
  return cfg.explorerAuthToken ? { GEMSTONE_RS_EXPLORER_TOKEN: cfg.explorerAuthToken } : {};
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
