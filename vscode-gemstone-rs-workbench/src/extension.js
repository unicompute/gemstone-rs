const childProcess = require("child_process");
const fs = require("fs");
const path = require("path");
const vscode = require("vscode");

let output;

function activate(context) {
  output = vscode.window.createOutputChannel("GemStone RS");
  context.subscriptions.push(output);

  register(context, "gemstoneRs.verifySetup", verifySetup);
  register(context, "gemstoneRs.eval", evalSmalltalk);
  register(context, "gemstoneRs.browseDictionaries", browseDictionaries);
  register(context, "gemstoneRs.browseClasses", browseClasses);
  register(context, "gemstoneRs.codegenInit", codegenInit);
  register(context, "gemstoneRs.codegenPreview", codegenPreview);
  register(context, "gemstoneRs.codegenCheck", codegenCheck);
  register(context, "gemstoneRs.codegenGenerate", codegenGenerate);
  register(context, "gemstoneRs.launchExplorer", launchExplorer);
  register(context, "gemstoneRs.openCodegenDocs", openCodegenDocs);
}

function deactivate() {}

function register(context, command, callback) {
  context.subscriptions.push(vscode.commands.registerCommand(command, callback));
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
  const result = await runAndShow(["codegen", "generate", configPath], { allowFailure: true });
  if (result.code !== 0) {
    return;
  }

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
