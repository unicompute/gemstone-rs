#!/usr/bin/env python3
"""Capture a deterministic gemstone-rs Workbench screenshot for Marketplace docs."""

from __future__ import annotations

import tempfile
from pathlib import Path

from capture_explorer_screenshots import ROOT, capture


OUTPUT = ROOT / "docs" / "assets" / "workbench-codegen-edit-flow.png"


HTML = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>gemstone-rs Workbench Codegen Edit Flow</title>
<style>
:root {
  color-scheme: dark;
  --bg: #1e1e1e;
  --panel: #252526;
  --panel-2: #2d2d30;
  --border: #3f3f46;
  --text: #d4d4d4;
  --muted: #9da3ae;
  --accent: #d04437;
  --ok: #2ea043;
}
body { margin: 0; background: var(--bg); color: var(--text); font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
.shell { height: 900px; display: grid; grid-template-rows: 48px 1fr; }
.title { display: flex; align-items: center; justify-content: space-between; padding: 0 22px; background: #181818; border-bottom: 1px solid var(--border); }
.title strong { font-size: 18px; }
.title span { color: var(--muted); }
.layout { display: grid; grid-template-columns: 355px 1fr; min-height: 0; }
.rail { background: var(--panel); border-right: 1px solid var(--border); padding: 16px; overflow: hidden; }
.group { border: 1px solid var(--border); background: var(--panel-2); border-radius: 6px; padding: 12px; margin-bottom: 12px; }
.group h2 { margin: 0 0 10px; font-size: 14px; font-weight: 600; color: #ffffff; }
label { display: grid; gap: 5px; color: var(--muted); font-size: 12px; margin-bottom: 8px; }
input { background: #1b1b1c; color: var(--text); border: 1px solid var(--border); border-radius: 4px; padding: 7px 8px; }
button { background: var(--accent); color: white; border: 0; border-radius: 4px; padding: 7px 10px; margin: 0 5px 6px 0; font-weight: 600; }
button.secondary { background: #3a3d41; color: var(--text); border: 1px solid var(--border); }
.content { display: grid; grid-template-rows: 46px 1fr; min-width: 0; }
.tabs { display: flex; gap: 6px; align-items: center; padding: 8px 12px; background: #202020; border-bottom: 1px solid var(--border); }
.tabs button { background: #333338; padding: 6px 9px; }
.main { display: grid; grid-template-columns: 1fr 1.05fr; gap: 14px; padding: 16px; min-height: 0; }
.browser { border: 1px solid var(--border); border-radius: 6px; background: #f6f8fa; color: #24292f; padding: 18px; overflow: hidden; }
.browser h1 { margin: 0 0 16px; font-size: 22px; color: #24292f; }
.row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.card { border: 1px solid #d0d7de; border-radius: 6px; padding: 12px; background: white; min-height: 110px; }
.card h3 { margin: 0 0 8px; font-size: 15px; }
.list { display: grid; gap: 6px; }
.item { padding: 8px; background: #f6f8fa; border: 1px solid #d0d7de; border-radius: 4px; }
.inspector { border: 1px solid var(--border); border-radius: 6px; background: var(--panel); min-width: 0; overflow: hidden; }
.inspector-head { display: flex; align-items: center; justify-content: space-between; padding: 12px; border-bottom: 1px solid var(--border); }
.inspector-head strong { font-size: 16px; }
.path { color: var(--muted); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; }
.actions { padding: 10px 12px 0; }
textarea { box-sizing: border-box; width: calc(100% - 24px); height: 610px; margin: 10px 12px 6px; resize: none; border: 1px solid var(--border); border-radius: 4px; padding: 12px; background: #1b1b1c; color: #dcdcaa; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 13px; line-height: 1.42; }
.state { color: var(--muted); padding: 0 12px 12px; font-size: 12px; }
.pill { background: rgba(46,160,67,.18); color: #7ee787; border: 1px solid rgba(46,160,67,.55); border-radius: 999px; padding: 3px 8px; font-size: 12px; }
</style>
</head>
<body>
<div class="shell">
  <div class="title">
    <strong>gemstone-rs Explorer Workbench</strong>
    <span>http://127.0.0.1:8787/ <span class="pill">connected</span></span>
  </div>
  <div class="layout">
    <aside class="rail">
      <div class="group">
        <h2>Project</h2>
        <label>Codegen config<input value="examples/codegen/gemstone-rs.codegen"></label>
        <label>Profile file<input value="examples/codegen/gemstone-rs.codegen-profiles.json"></label>
        <button class="secondary">Open Codegen Config</button>
        <button class="secondary">Open Project Profiles</button>
      </div>
      <div class="group">
        <h2>Codegen</h2>
        <button>Preview/Edit Generated Wrappers</button>
        <button>Read/Edit Generated Output</button>
        <button class="secondary">Diff Generated Wrappers</button>
        <button class="secondary">Check Freshness</button>
      </div>
      <div class="group">
        <h2>Live Browse</h2>
        <label>Dictionary<input value="UserGlobals"></label>
        <label>Class<input value="Object"></label>
        <button class="secondary">Methods</button>
        <button class="secondary">Open Source</button>
      </div>
    </aside>
    <main class="content">
      <div class="tabs">
        <button>Home</button><button>Browse</button><button>Codegen Workflow</button><button>Profile Status JSON</button>
      </div>
      <div class="main">
        <section class="browser">
          <h1>gemstone-rs Explorer</h1>
          <div class="row">
            <div class="card"><h3>Codegen Workflow</h3><div class="list"><div class="item">Preview wrappers</div><div class="item">Diff generated output</div><div class="item">Check freshness</div></div></div>
            <div class="card"><h3>Live Browse</h3><div class="list"><div class="item">Dictionaries</div><div class="item">Classes</div><div class="item">Methods and source</div></div></div>
          </div>
        </section>
        <section class="inspector">
          <div class="inspector-head">
            <strong>Generated Output</strong>
            <span class="path">examples/codegen/generated/gemstone_wrappers.rs</span>
          </div>
          <div class="actions">
            <button class="secondary">Open Output File</button>
            <button class="secondary">Open Editable Draft</button>
            <button>Save Edited Output</button>
          </div>
          <textarea spellcheck="false">// @generated by gemstone-rs codegen. Do not edit by hand.
use gemstone_rs::{BridgeMapped, BridgeValue, Error, Oop, Result, Session, Value};

pub struct Object<'a> {
    session: &'a mut Session,
    oop: Oop,
}

impl<'a> Object<'a> {
    pub fn resolve(session: &'a mut Session) -> Result<Self> {
        let oop = session.resolve("Object")?;
        Ok(Self { session, oop })
    }

    /// Return the receiver printString.
    pub fn print_string(&mut self) -> Result<String> {
        let value = self.session.perform(self.oop, "printString", &[])?;
        match value {
            Value::String(text) => Ok(text),
            other => Err(Error::UnexpectedType {
                expected: "String",
                actual: format!("{other:?}"),
            }),
        }
    }
}</textarea>
          <div class="state">Edited in webview. Save writes the configured generated output file after confirmation.</div>
        </section>
      </div>
    </main>
  </div>
</div>
</body>
</html>
"""


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="gemstone-rs-workbench-shot-") as tmp:
        html = Path(tmp) / "workbench-codegen-edit-flow.html"
        html.write_text(HTML, encoding="utf-8")
        capture(html.as_uri(), OUTPUT, 1440, 900)
    print(OUTPUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
