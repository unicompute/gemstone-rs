#!/usr/bin/env python3
"""Validate committed visual assets and Markdown image references."""

from __future__ import annotations

import json
import re
import struct
import sys
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import urlparse


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

REQUIRED_ASSETS = [
    ROOT / "docs/assets/gemstone-rs-graphic.png",
    ROOT / "docs/assets/explorer-home.png",
    ROOT / "docs/assets/workbench-codegen-edit-flow.png",
    ROOT / "docs/assets/workbench-codegen-edit-flow.gif",
    ROOT / "vscode-gemstone-rs-workbench/media/activity.svg",
    ROOT / "vscode-gemstone-rs-workbench/media/gemstone-rs-graphic.png",
    ROOT / "vscode-gemstone-rs-workbench/media/icon.png",
]

IMAGE_RE = re.compile(r"!\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")


@dataclass(frozen=True)
class ImageInfo:
    kind: str
    width: int | None = None
    height: int | None = None


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


def parse_image(data: bytes, path: Path) -> ImageInfo:
    if data.startswith(b"\x89PNG\r\n\x1a\n"):
        if len(data) < 24:
            raise ValueError("truncated PNG header")
        width, height = struct.unpack(">II", data[16:24])
        return ImageInfo("png", width, height)
    if data.startswith((b"GIF87a", b"GIF89a")):
        if len(data) < 10:
            raise ValueError("truncated GIF header")
        width, height = struct.unpack("<HH", data[6:10])
        return ImageInfo("gif", width, height)
    if data.startswith(b"\xff\xd8\xff"):
        return ImageInfo("jpeg")
    if path.suffix.lower() == ".svg":
        text = data[:512].decode("utf-8", errors="ignore").lower()
        if "<svg" not in text:
            raise ValueError("SVG file does not contain <svg")
        return ImageInfo("svg")
    raise ValueError("unrecognized image format")


def validate_image(path: Path) -> ImageInfo:
    if not path.exists():
        raise ValueError("missing")
    if not path.is_file():
        raise ValueError("not a file")
    data = path.read_bytes()
    if not data:
        raise ValueError("empty")
    info = parse_image(data, path)
    if info.width is not None and info.height is not None:
        if info.width <= 0 or info.height <= 0:
            raise ValueError("invalid dimensions")
    return info


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


def resolve_markdown_image(markdown: Path, target: str) -> Path | None:
    if target.startswith("#") or target.startswith("data:"):
        return None
    if target.startswith("http://") or target.startswith("https://"):
        return raw_github_to_local(target)
    clean = target.split("#", 1)[0].split("?", 1)[0]
    return (markdown.parent / clean).resolve()


def check_markdown_references(errors: list[str]) -> int:
    checked = 0
    for markdown in iter_markdown_files():
        text = markdown.read_text(encoding="utf-8")
        for match in IMAGE_RE.finditer(text):
            target = match.group(1)
            path = resolve_markdown_image(markdown, target)
            if path is None:
                continue
            checked += 1
            try:
                validate_image(path)
            except ValueError as err:
                errors.append(f"{rel(markdown)}: image {target!r}: {err}")
    return checked


def check_required_assets(errors: list[str]) -> int:
    checked = 0
    for asset in REQUIRED_ASSETS:
        checked += 1
        try:
            validate_image(asset)
        except ValueError as err:
            errors.append(f"{rel(asset)}: {err}")
    return checked


def check_vscode_icon(errors: list[str]) -> int:
    package_path = ROOT / "vscode-gemstone-rs-workbench/package.json"
    package = json.loads(package_path.read_text(encoding="utf-8"))
    icon = package.get("icon")
    if not isinstance(icon, str) or not icon:
        errors.append("vscode-gemstone-rs-workbench/package.json: icon is required")
        return 0
    icon_path = package_path.parent / icon
    try:
        info = validate_image(icon_path)
    except ValueError as err:
        errors.append(f"{rel(icon_path)}: package icon: {err}")
        return 1
    if info.kind != "png":
        errors.append(f"{rel(icon_path)}: package icon must be PNG, got {info.kind}")
    if info.width is not None and info.height is not None:
        if info.width < 128 or info.height < 128:
            errors.append(f"{rel(icon_path)}: package icon should be at least 128x128")
    return 1


def main() -> int:
    errors: list[str] = []
    checked = 0
    checked += check_required_assets(errors)
    checked += check_vscode_icon(errors)
    checked += check_markdown_references(errors)

    if errors:
        for error in errors:
            print(f"visual-asset-check: {error}", file=sys.stderr)
        return 1

    print(f"visual-asset-check ok: {checked} image references/assets")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
