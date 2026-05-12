#!/usr/bin/env python3
"""Build gemstone-rs PDF docs from local Markdown sources."""

from __future__ import annotations

import html
import re
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

DOCS_DIR = Path(__file__).resolve().parent
PDF_DIR = DOCS_DIR / "pdf"
HTML_DIR = DOCS_DIR / ".pdf-build"
CSS_PATH = DOCS_DIR / "pdf-theme.css"


@dataclass(frozen=True)
class BuildTarget:
    slug: str
    title: str
    subtitle: str
    source_paths: tuple[Path, ...]
    cover_image: Path | None = None


def _normalize_target(target: str, source_dir: Path, known_stems: frozenset[str]) -> str:
    if target.startswith(("http://", "https://", "mailto:", "#")):
        return target
    path = target.split("#")[0]
    fragment = target[len(path) :]
    if path.endswith(".md") and Path(path).stem in known_stems:
        return f"#{Path(path).stem}{fragment}"
    return (source_dir / target).resolve().as_uri()


def _inline(text: str, source_dir: Path, known_stems: frozenset[str]) -> str:
    text = html.escape(text, quote=False)
    text = re.sub(
        r"!\[([^\]]*)\]\(([^)]+)\)",
        lambda m: (
            f'<img alt="{html.escape(m.group(1), quote=True)}" '
            f'src="{html.escape(_normalize_target(m.group(2), source_dir, known_stems), quote=True)}"/>'
        ),
        text,
    )
    text = re.sub(
        r"\[([^\]]+)\]\(([^)]+)\)",
        lambda m: (
            f'<a href="{html.escape(_normalize_target(m.group(2), source_dir, known_stems), quote=True)}">'
            f"{m.group(1)}</a>"
        ),
        text,
    )
    text = re.sub(r"`([^`]+)`", lambda m: f"<code>{m.group(1)}</code>", text)
    text = re.sub(r"\*\*([^*]+)\*\*", lambda m: f"<strong>{m.group(1)}</strong>", text)
    text = re.sub(r"\*([^*]+)\*", lambda m: f"<em>{m.group(1)}</em>", text)
    return text


def _render_table(lines: list[str], source_dir: Path, known_stems: frozenset[str]) -> str:
    rows = [[cell.strip() for cell in line.strip().strip("|").split("|")] for line in lines]
    out = ["<table><thead><tr>"]
    out.extend(f"<th>{_inline(cell, source_dir, known_stems)}</th>" for cell in rows[0])
    out.append("</tr></thead><tbody>")
    for row in rows[2:]:
        out.append("<tr>")
        out.extend(f"<td>{_inline(cell, source_dir, known_stems)}</td>" for cell in row)
        out.append("</tr>")
    out.append("</tbody></table>")
    return "".join(out)


def _render_markdown(
    text: str,
    source_dir: Path,
    anchor_prefix: str,
    known_stems: frozenset[str],
) -> tuple[str, list[tuple[int, str, str]]]:
    lines = text.splitlines()
    out: list[str] = []
    toc: list[tuple[int, str, str]] = []
    paragraph: list[str] = []
    list_stack: list[str] = []
    i = 0

    def flush_paragraph() -> None:
        nonlocal paragraph
        if paragraph:
            out.append(f"<p>{_inline(' '.join(paragraph), source_dir, known_stems)}</p>")
            paragraph = []

    def flush_lists() -> None:
        while list_stack:
            out.append(f"</{list_stack.pop()}>")

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if not stripped:
            flush_paragraph()
            flush_lists()
            i += 1
            continue

        if stripped.startswith("```"):
            flush_paragraph()
            flush_lists()
            fence_lines: list[str] = []
            i += 1
            while i < len(lines) and not lines[i].strip().startswith("```"):
                fence_lines.append(lines[i])
                i += 1
            out.append(f"<pre><code>{html.escape(chr(10).join(fence_lines))}</code></pre>")
            i += 1
            continue

        heading_match = re.match(r"^(#{1,6})\s+(.*)$", stripped)
        if heading_match:
            flush_paragraph()
            flush_lists()
            level = len(heading_match.group(1))
            content = heading_match.group(2).strip()
            base_anchor = re.sub(r"[^a-z0-9]+", "-", content.lower()).strip("-")
            anchor = f"{anchor_prefix}-{base_anchor}" if base_anchor else anchor_prefix
            toc.append((level, content, anchor))
            out.append(f'<h{level} id="{anchor}">{_inline(content, source_dir, known_stems)}</h{level}>')
            i += 1
            continue

        if stripped.startswith("> "):
            flush_paragraph()
            flush_lists()
            quote_lines = [stripped[2:]]
            i += 1
            while i < len(lines) and lines[i].strip().startswith("> "):
                quote_lines.append(lines[i].strip()[2:])
                i += 1
            out.append(f"<blockquote><p>{_inline(' '.join(quote_lines), source_dir, known_stems)}</p></blockquote>")
            continue

        if stripped.startswith("|") and i + 1 < len(lines):
            sep = lines[i + 1].strip()
            if sep.startswith("|") and re.fullmatch(r"[|\-: ]+", sep):
                flush_paragraph()
                flush_lists()
                table_lines = [lines[i], lines[i + 1]]
                i += 2
                while i < len(lines) and lines[i].strip().startswith("|"):
                    table_lines.append(lines[i])
                    i += 1
                out.append(_render_table(table_lines, source_dir, known_stems))
                continue

        ordered_match = re.match(r"^\d+\.\s+(.*)$", stripped)
        unordered_match = re.match(r"^-\s+(.*)$", stripped)
        if ordered_match or unordered_match:
            flush_paragraph()
            target = "ol" if ordered_match else "ul"
            if not list_stack or list_stack[-1] != target:
                flush_lists()
                list_stack.append(target)
                out.append(f"<{target}>")
            item_text = ordered_match.group(1) if ordered_match else unordered_match.group(1)
            out.append(f"<li>{_inline(item_text, source_dir, known_stems)}</li>")
            i += 1
            continue

        paragraph.append(stripped)
        i += 1

    flush_paragraph()
    flush_lists()
    return "".join(out), toc


def _toc_html(entries: Iterable[tuple[int, str, str]]) -> str:
    parts = ['<section class="toc"><h2>Contents</h2><ul>']
    for level, title, anchor in entries:
        parts.append(
            f'<li style="margin-left: {12 * (level - 1)}px;"><a href="#{anchor}">{html.escape(title)}</a></li>'
        )
    parts.append("</ul></section>")
    return "".join(parts)


def _build_html(target: BuildTarget) -> Path:
    HTML_DIR.mkdir(parents=True, exist_ok=True)
    known_stems = frozenset(path.stem for path in target.source_paths)
    body_parts: list[str] = []
    toc_entries: list[tuple[int, str, str]] = []

    for source_path in target.source_paths:
        rendered, toc = _render_markdown(
            source_path.read_text(encoding="utf-8"),
            source_path.parent,
            source_path.stem,
            known_stems,
        )
        toc_entries.extend(toc)
        body_parts.append(f'<div id="{source_path.stem}" style="height:0"></div>{rendered}')

    cover_html = ""
    if target.cover_image is not None:
        cover_html = (
            f'<img class="cover-art" src="{html.escape(target.cover_image.resolve().as_uri(), quote=True)}" '
            f'alt="{html.escape(target.title, quote=True)} cover art"/>'
        )

    html_text = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <title>{html.escape(target.title)}</title>
  <link rel="stylesheet" href="{CSS_PATH.name}"/>
</head>
<body>
  <main>
    <section class="title-page">
      <h1>{html.escape(target.title)}</h1>
      <div class="subtitle">{html.escape(target.subtitle)}</div>
      {cover_html}
      <div class="meta">Generated from the Markdown sources in gemstone-rs/docs.</div>
    </section>
    {_toc_html(toc_entries)}
    {''.join(body_parts)}
    <section class="doc-note">
      This PDF was generated locally from <code>docs/</code>.
      If you edit the Markdown sources, rerun <code>python docs/build_pdf_docs.py</code>.
    </section>
  </main>
</body>
</html>
"""
    html_path = HTML_DIR / f"{target.slug}.html"
    html_path.write_text(html_text, encoding="utf-8")
    (HTML_DIR / CSS_PATH.name).write_text(CSS_PATH.read_text(encoding="utf-8"), encoding="utf-8")
    return html_path


def _render_pdf(target: BuildTarget) -> Path:
    PDF_DIR.mkdir(parents=True, exist_ok=True)
    html_path = _build_html(target)
    pdf_path = PDF_DIR / f"{target.slug}.pdf"
    subprocess.run(["weasyprint", str(html_path), str(pdf_path)], cwd=str(HTML_DIR), check=True)
    return pdf_path


def main() -> int:
    if HTML_DIR.exists():
        shutil.rmtree(HTML_DIR)

    funny_dir = DOCS_DIR / "funny-introduction"
    cover = DOCS_DIR / "assets" / "gemstone-rs-graphic.png"
    targets = (
        BuildTarget("setup-guide", "gemstone-rs Setup Guide", "Install, configure, and complete the first GemStone login", (DOCS_DIR / "setup-guide.md",), cover),
        BuildTarget("user-manual", "gemstone-rs User Manual", "Core Rust API usage for sessions, OOPs, browser operations, and transactions", (DOCS_DIR / "user-manual.md",), cover),
        BuildTarget("examples-guide", "gemstone-rs Examples Guide", "A tour of Rust examples and tooling workflows", (DOCS_DIR / "examples-guide.md",), cover),
        BuildTarget("cookbook", "gemstone-rs Cookbook", "Task-focused Rust recipes for GemStone/S", (DOCS_DIR / "cookbook.md",), cover),
        BuildTarget("gemstone-py-vs-gemstone-rs", "gemstone-py vs gemstone-rs", "Install paths, use cases, maturity, and feature matrix", (DOCS_DIR / "gemstone-py-vs-gemstone-rs.md",), cover),
        BuildTarget("object-mapping", "gemstone-rs BridgeRoot and Object Mapping", "A first MagLev-style bridge-root mapping layer over OOPs", (DOCS_DIR / "object-mapping.md",), cover),
        BuildTarget("codegen", "gemstone-rs Codegen Guide", "Generating checked-in Rust wrappers for GemStone classes", (DOCS_DIR / "codegen.md",), cover),
        BuildTarget("explorer", "gemstone-rs Explorer Guide", "Local HTTP explorer endpoints and safety defaults", (DOCS_DIR / "explorer.md",), cover),
        BuildTarget("vscode-workbench", "gemstone-rs VS Code Workbench", "Sidebar browsing, codegen, and explorer launch workflows", (DOCS_DIR / "vscode-workbench.md",), cover),
        BuildTarget("performance-safety", "gemstone-rs Performance and Safety", "Benchmark guidance, GCI loading notes, and threading rules", (DOCS_DIR / "performance-safety.md",), cover),
        BuildTarget("shared-core-integration", "gemstone-rs Shared Core Integration", "How gemstone-py-native can wrap the Rust core", (DOCS_DIR / "shared-core-integration.md",), cover),
        BuildTarget("release-checklist", "gemstone-rs Release Checklist", "Crates, VSIX, GitHub release, and verification steps", (DOCS_DIR / "release-checklist.md",), cover),
        BuildTarget("medium-article", "Talking to GemStone/S from Rust", "A complete article-style guide to gemstone-rs", (DOCS_DIR / "medium-article.md",), cover),
        BuildTarget(
            "funny-introduction-book",
            "A Practical but Lighter Introduction to gemstone-rs",
            "Rust, GemStone/S, and the sharp edges you actually need to know about",
            (
                funny_dir / "README.md",
                funny_dir / "part-01-why-gemstone-rs-exists.md",
                funny_dir / "part-02-sessions-and-transactions.md",
                funny_dir / "part-03-oops-values-and-browser.md",
                funny_dir / "part-04-codegen-explorer-and-vscode.md",
            ),
            cover,
        ),
        BuildTarget(
            "core-guides-companion",
            "gemstone-rs Companion Manual",
            "Setup, user manual, examples, cookbook, codegen, explorer, and workbench in one PDF",
            (
                DOCS_DIR / "README.md",
                DOCS_DIR / "setup-guide.md",
                DOCS_DIR / "user-manual.md",
                DOCS_DIR / "examples-guide.md",
                DOCS_DIR / "cookbook.md",
                DOCS_DIR / "gemstone-py-vs-gemstone-rs.md",
                DOCS_DIR / "object-mapping.md",
                DOCS_DIR / "codegen.md",
                DOCS_DIR / "explorer.md",
                DOCS_DIR / "vscode-workbench.md",
                DOCS_DIR / "performance-safety.md",
                DOCS_DIR / "shared-core-integration.md",
                DOCS_DIR / "release-checklist.md",
            ),
            cover,
        ),
    )

    try:
        for target in targets:
            print(_render_pdf(target))
        return 0
    finally:
        if HTML_DIR.exists():
            shutil.rmtree(HTML_DIR)


if __name__ == "__main__":
    raise SystemExit(main())
