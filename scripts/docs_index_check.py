#!/usr/bin/env python3
"""Check that public documentation indexes list the main guides."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class RequiredGuide:
    label: str
    root_target: str
    docs_target: str


REQUIRED_GUIDES = (
    RequiredGuide("Setup Guide", "docs/setup-guide.md", "setup-guide.md"),
    RequiredGuide("Examples Guide", "docs/examples-guide.md", "examples-guide.md"),
    RequiredGuide("Feature Map", "docs/feature-map.md", "feature-map.md"),
    RequiredGuide("User Manual", "docs/user-manual.md", "user-manual.md"),
    RequiredGuide("Cookbook", "docs/cookbook.md", "cookbook.md"),
    RequiredGuide(
        "gemstone-py vs gemstone-rs",
        "docs/gemstone-py-vs-gemstone-rs.md",
        "gemstone-py-vs-gemstone-rs.md",
    ),
    RequiredGuide(
        "gemstone-js vs gemstone-py",
        "docs/gemstone-js-vs-gemstone-py.md",
        "gemstone-js-vs-gemstone-py.md",
    ),
    RequiredGuide("Object Mapping", "docs/object-mapping.md", "object-mapping.md"),
    RequiredGuide("Codegen Guide", "docs/codegen.md", "codegen.md"),
    RequiredGuide("Codegen Profile Schema", "docs/profile-schema.md", "profile-schema.md"),
    RequiredGuide("Explorer Guide", "docs/explorer.md", "explorer.md"),
    RequiredGuide("VS Code Workbench", "docs/vscode-workbench.md", "vscode-workbench.md"),
    RequiredGuide("Screenshot Workflow", "docs/screenshots.md", "screenshots.md"),
    RequiredGuide(
        "Performance and Safety",
        "docs/performance-safety.md",
        "performance-safety.md",
    ),
    RequiredGuide(
        "Shared Core Integration",
        "docs/shared-core-integration.md",
        "shared-core-integration.md",
    ),
    RequiredGuide("Medium Article", "docs/medium-article.md", "medium-article.md"),
    RequiredGuide(
        "Funny Introduction",
        "docs/funny-introduction/README.md",
        "funny-introduction/README.md",
    ),
    RequiredGuide("Release Checklist", "docs/release-checklist.md", "release-checklist.md"),
)

LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")


def markdown_links(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    return {target.split("#", 1)[0] for _label, target in LINK_RE.findall(text)}


def check_index(path: Path, targets: list[str], errors: list[str]) -> None:
    links = markdown_links(path)
    for target in targets:
        if target not in links:
            errors.append(f"{path.relative_to(ROOT)} missing link to {target}")


def check_files_exist(errors: list[str]) -> None:
    for guide in REQUIRED_GUIDES:
        root_path = ROOT / guide.root_target
        if not root_path.exists():
            errors.append(f"missing guide file {guide.root_target}")


def main() -> int:
    errors: list[str] = []
    check_files_exist(errors)
    check_index(ROOT / "README.md", [guide.root_target for guide in REQUIRED_GUIDES], errors)
    check_index(ROOT / "docs/README.md", [guide.docs_target for guide in REQUIRED_GUIDES], errors)

    if errors:
        for error in errors:
            print(f"docs-index-check: {error}", file=sys.stderr)
        return 1

    print(f"docs-index-check ok: {len(REQUIRED_GUIDES)} guides listed in README.md and docs/README.md")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
