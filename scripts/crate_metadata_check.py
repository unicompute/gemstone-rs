#!/usr/bin/env python3
"""Check crates.io package metadata and publish order."""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

CRATE_ORDER = [
    "gemstone-gci",
    "gemstone-rs-macros",
    "gemstone-rs",
    "gemstone-rs-cli",
    "gemstone-rs-explorer",
]

CRATE_PATHS = {
    "gemstone-gci": "crates/gemstone-gci",
    "gemstone-rs-macros": "crates/gemstone-rs-macros",
    "gemstone-rs": "crates/gemstone-rs",
    "gemstone-rs-cli": "crates/gemstone-rs-cli",
    "gemstone-rs-explorer": "crates/gemstone-rs-explorer",
}

EXPECTED_BINS = {
    "gemstone-rs-cli": "gemstone-rs",
    "gemstone-rs-explorer": "gemstone-rs-explorer",
}

LOCAL_DEPENDENCIES = {
    "gemstone-rs": ["gemstone-gci", "gemstone-rs-macros"],
    "gemstone-rs-cli": ["gemstone-rs"],
    "gemstone-rs-explorer": ["gemstone-rs"],
}


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_toml(path: Path) -> dict:
    return tomllib.loads(read_text(path))


def workspace_value(package: dict, key: str, workspace_package: dict) -> object | None:
    value = package.get(key)
    if isinstance(value, dict) and value.get("workspace") is True:
        return workspace_package.get(key)
    return value


def main() -> int:
    errors: list[str] = []

    def check(condition: bool, message: str) -> None:
        if not condition:
            errors.append(message)

    workspace = read_toml(ROOT / "Cargo.toml")
    workspace_package = workspace["workspace"]["package"]
    members = workspace["workspace"]["members"]
    expected_members = [CRATE_PATHS[name] for name in CRATE_ORDER]
    check(
        sorted(members) == sorted(expected_members),
        "workspace members do not match the publishable crate set",
    )

    workspace_dependencies = workspace["workspace"]["dependencies"]
    publish_script = read_text(ROOT / "scripts/publish_crates.sh")
    published_order = re.findall(r"^publish_crate ([^\s]+)$", publish_script, re.MULTILINE)
    check(published_order == CRATE_ORDER, "scripts/publish_crates.sh publish order is stale")

    for crate in CRATE_ORDER:
        crate_dir = ROOT / CRATE_PATHS[crate]
        manifest_path = crate_dir / "Cargo.toml"
        manifest = read_toml(manifest_path)
        package = manifest["package"]

        check(package.get("name") == crate, f"{manifest_path}: package.name must be {crate}")
        check(bool(package.get("version")), f"{manifest_path}: package.version is required")

        description = package.get("description", "")
        check(isinstance(description, str) and 10 <= len(description) <= 200, f"{crate}: description must be 10-200 chars")
        check("\n" not in description, f"{crate}: description must be one line")

        for key in ["edition", "license", "homepage", "repository", "keywords", "categories"]:
            check(
                workspace_value(package, key, workspace_package) == workspace_package[key],
                f"{crate}: package.{key} should inherit workspace {key}",
            )

        keywords = workspace_value(package, "keywords", workspace_package)
        categories = workspace_value(package, "categories", workspace_package)
        check(isinstance(keywords, list) and 0 < len(keywords) <= 5, f"{crate}: crates.io allows 1-5 keywords")
        check(isinstance(categories, list) and 0 < len(categories) <= 5, f"{crate}: crates.io allows 1-5 categories")

        readme = package.get("readme")
        check(readme == "README.md", f"{crate}: package.readme should be README.md")
        readme_path = crate_dir / "README.md"
        if readme_path.exists():
            readme_text = read_text(readme_path)
            check(crate in readme_text, f"{crate}: README should mention the crate/package name")
            check(len(readme_text.strip()) >= 100, f"{crate}: README is too small for crates.io")
        else:
            check(False, f"{crate}: README.md is missing")

        expected_bin = EXPECTED_BINS.get(crate)
        if expected_bin:
            bins = manifest.get("bin", [])
            names = [item.get("name") for item in bins]
            check(expected_bin in names, f"{crate}: expected binary {expected_bin}")
        else:
            check("bin" not in manifest, f"{crate}: unexpected [[bin]] section")

        if crate == "gemstone-rs-macros":
            check(manifest.get("lib", {}).get("proc-macro") is True, "gemstone-rs-macros must be a proc-macro crate")

        dependencies = manifest.get("dependencies", {})
        for dependency in LOCAL_DEPENDENCIES.get(crate, []):
            check(
                dependencies.get(dependency, {}).get("workspace") is True,
                f"{crate}: dependency {dependency} should use workspace = true",
            )
            check(dependency in workspace_dependencies, f"{crate}: dependency {dependency} missing from workspace dependencies")

    if errors:
        for error in errors:
            print(f"crate-metadata-check: {error}", file=sys.stderr)
        return 1

    print(f"crate-metadata-check ok: {len(CRATE_ORDER)} crates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
