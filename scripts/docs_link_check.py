#!/usr/bin/env python3
"""Validate local Markdown links without touching the network."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlparse


ROOT = Path(__file__).resolve().parents[1]

MARKDOWN_ROOTS = [
    ROOT / "README.md",
    ROOT / "docs",
    ROOT / "examples",
    ROOT / "vscode-gemstone-rs-workbench" / "README.md",
]

SKIP_DIRS = {
    ".git",
    "node_modules",
    "target",
    "docs/pdf",
    "examples/actix-service/target",
    "examples/axum-service/target",
}

LINK_RE = re.compile(r"(!?)\[[^\]]+\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
HEADING_RE = re.compile(r"^(#{1,6})\s+(.+?)\s*#*\s*$")
INLINE_MARKUP_RE = re.compile(r"[`*_~]")
PUNCTUATION_RE = re.compile(r"[^a-z0-9 _-]")
SPACE_RE = re.compile(r"\s+")


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def iter_markdown_files() -> list[Path]:
    files: list[Path] = []
    for root in MARKDOWN_ROOTS:
        if root.is_file():
            files.append(root)
            continue
        for path in root.rglob("*.md"):
            relative = rel(path)
            if any(relative == skip or relative.startswith(f"{skip}/") for skip in SKIP_DIRS):
                continue
            files.append(path)
    return sorted(set(files))


def github_slug(text: str) -> str:
    text = INLINE_MARKUP_RE.sub("", text)
    text = text.strip().lower()
    text = PUNCTUATION_RE.sub("", text)
    text = SPACE_RE.sub("-", text)
    return text.strip("-")


def markdown_anchors(path: Path) -> set[str]:
    anchors: set[str] = set()
    seen: dict[str, int] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        match = HEADING_RE.match(line)
        if not match:
            continue
        base = github_slug(match.group(2))
        if not base:
            continue
        count = seen.get(base, 0)
        seen[base] = count + 1
        anchors.add(base if count == 0 else f"{base}-{count}")
    return anchors


def raw_github_to_local(url: str) -> Path | None:
    parsed = urlparse(url)
    if parsed.netloc != "raw.githubusercontent.com":
        return None
    parts = [part for part in parsed.path.split("/") if part]
    if len(parts) < 4:
        return None
    owner, repo, _ref, *local = parts
    if owner != "unicompute" or repo != "gemstone-rs":
        return None
    return ROOT.joinpath(*local)


def resolve_local_target(markdown: Path, target: str) -> tuple[Path | None, str | None]:
    parsed = urlparse(target)
    if parsed.scheme in {"http", "https"}:
        return raw_github_to_local(target), parsed.fragment or None
    if parsed.scheme or target.startswith("mailto:") or target.startswith("data:"):
        return None, None

    clean_path = unquote(parsed.path)
    fragment = unquote(parsed.fragment) if parsed.fragment else None
    if not clean_path:
        return markdown, fragment
    return (markdown.parent / clean_path).resolve(), fragment


def check_link(markdown: Path, target: str, anchor_cache: dict[Path, set[str]]) -> str | None:
    path, fragment = resolve_local_target(markdown, target)
    if path is None:
        return None
    if not path.exists():
        return f"{rel(markdown)}: link {target!r}: missing {path}"
    if fragment and path.is_file() and path.suffix.lower() == ".md":
        anchors = anchor_cache.setdefault(path, markdown_anchors(path))
        wanted = github_slug(fragment)
        if wanted and wanted not in anchors:
            return f"{rel(markdown)}: link {target!r}: missing anchor #{fragment} in {rel(path)}"
    return None


def main() -> int:
    errors: list[str] = []
    checked = 0
    anchor_cache: dict[Path, set[str]] = {}

    for markdown in iter_markdown_files():
        text = markdown.read_text(encoding="utf-8")
        for _image_marker, target in LINK_RE.findall(text):
            checked += 1
            error = check_link(markdown, target, anchor_cache)
            if error:
                errors.append(error)

    if errors:
        for error in errors:
            print(f"docs-link-check: {error}", file=sys.stderr)
        return 1

    print(f"docs-link-check ok: {checked} local/external Markdown links scanned")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
