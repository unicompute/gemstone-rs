const childProcess = require("child_process");
const fs = require("fs");
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
  register(context, "gemstoneRs.eval", evalSmalltalk);
  register(context, "gemstoneRs.browseDictionaries", browseDictionaries);
  register(context, "gemstoneRs.browseClasses", browseClasses);
  register(context, "gemstoneRs.codegenInit", codegenInit);
  register(context, "gemstoneRs.codegenDiscover", codegenDiscover);
  register(context, "gemstoneRs.codegenPreview", codegenPreview);
  register(context, "gemstoneRs.codegenDiff", codegenDiff);
  register(context, "gemstoneRs.codegenCheck", codegenCheck);
  register(context, "gemstoneRs.codegenGenerate", codegenGenerate);
  register(context, "gemstoneRs.launchExplorer", launchExplorer);
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
        actionNode("Preview Wrappers", "gemstoneRs.codegenPreview"),
        actionNode("Diff Generated Output", "gemstoneRs.codegenDiff"),
        actionNode("Check Freshness", "gemstoneRs.codegenCheck"),
        actionNode("Generate Wrappers", "gemstoneRs.codegenGenerate"),
        actionNode("Open Codegen Docs", "gemstoneRs.openCodegenDocs"),
      ];
    }

    if (element.type === "explorer") {
      return [
        actionNode("Verify Setup", "gemstoneRs.verifySetup"),
        actionNode("Eval Smalltalk", "gemstoneRs.eval"),
        actionNode("Launch Explorer", "gemstoneRs.launchExplorer"),
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
  const cfg = settings();
  output.clear();
  output.appendLine("gemstone-rs Workbench setup");
  output.appendLine(`cwd: ${cfg.cwd}`);
  output.appendLine(`useCargo: ${cfg.useCargo}`);
  output.appendLine(`cliPath: ${cfg.cliPath}`);
  output.appendLine(`explorerPath: ${cfg.explorerPath}`);
  output.appendLine(`codegenConfig: ${cfg.codegenConfig}`);
  output.appendLine(`codegenConfigExists: ${fs.existsSync(resolvePath(cfg.codegenConfig, cfg.cwd))}`);
  output.appendLine("");

  const result = await runCli(["--help"], { allowFailure: true });
  output.append(result.stdout);
  output.append(result.stderr);
  output.show(true);

  if (result.code === 0) {
    vscode.window.showInformationMessage("gemstone-rs CLI is available.");
  } else {
    vscode.window.showWarningMessage("gemstone-rs CLI check failed. See GemStone RS output.");
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
  const terminal = vscode.window.createTerminal({
    name: "gemstone-rs Explorer",
    cwd: cfg.cwd,
  });
  if (cfg.useCargo) {
    terminal.sendText(
      `cargo run -p gemstone-rs-explorer -- --host ${shellQuote(cfg.explorerHost)} --port ${cfg.explorerPort}`
    );
  } else {
    terminal.sendText(
      `${shellQuote(cfg.explorerPath)} --host ${shellQuote(cfg.explorerHost)} --port ${cfg.explorerPort}`
    );
  }
  terminal.show();
  vscode.env.openExternal(vscode.Uri.parse(`http://${cfg.explorerHost}:${cfg.explorerPort}/`));
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
  const commandArgs = cfg.useCargo ? ["run", "-p", "gemstone-rs-cli", "--", ...args] : args;

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
    codegenConfig: cfg.get("codegenConfig", "gemstone-rs.codegen"),
    explorerHost: cfg.get("explorerHost", "127.0.0.1"),
    explorerPort: cfg.get("explorerPort", 8787),
  };
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

module.exports = {
  activate,
  deactivate,
};
