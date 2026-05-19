#!/usr/bin/env python3
"""Check release metadata stays in sync across gemstone-rs tooling."""

from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

CRATE_MANIFESTS = {
    "gemstone-gci": ROOT / "crates/gemstone-gci/Cargo.toml",
    "gemstone-rs-macros": ROOT / "crates/gemstone-rs-macros/Cargo.toml",
    "gemstone-rs": ROOT / "crates/gemstone-rs/Cargo.toml",
    "gemstone-rs-axum": ROOT / "crates/gemstone-rs-axum/Cargo.toml",
    "gemstone-rs-actix": ROOT / "crates/gemstone-rs-actix/Cargo.toml",
    "gemstone-rs-cli": ROOT / "crates/gemstone-rs-cli/Cargo.toml",
    "gemstone-rs-explorer": ROOT / "crates/gemstone-rs-explorer/Cargo.toml",
}

WORKSPACE_DEPS = [
    "gemstone-gci",
    "gemstone-rs-macros",
    "gemstone-rs",
    "gemstone-rs-axum",
    "gemstone-rs-actix",
]


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_toml(path: Path) -> dict:
    return tomllib.loads(read_text(path))


def read_json(path: Path) -> dict:
    return json.loads(read_text(path))


def first_match(pattern: str, text: str, label: str) -> str:
    match = re.search(pattern, text, re.MULTILINE)
    if not match:
        raise AssertionError(f"could not find {label}")
    return match.group(1)


def main() -> int:
    errors: list[str] = []

    def check(condition: bool, message: str) -> None:
        if not condition:
            errors.append(message)

    crate_versions: dict[str, str] = {}
    for crate, manifest in CRATE_MANIFESTS.items():
        value = read_toml(manifest)
        package = value["package"]
        check(package["name"] == crate, f"{manifest}: package.name is {package['name']!r}, expected {crate!r}")
        crate_versions[crate] = package["version"]

    release_version = crate_versions["gemstone-rs"]
    for crate, version in crate_versions.items():
        check(
            version == release_version,
            f"{crate} version {version} does not match gemstone-rs {release_version}",
        )

    workspace = read_toml(ROOT / "Cargo.toml")
    workspace_deps = workspace["workspace"]["dependencies"]
    for crate in WORKSPACE_DEPS:
        dep = workspace_deps[crate]
        check(
            dep["version"] == crate_versions[crate],
            f"workspace dependency {crate} version {dep['version']} does not match {crate_versions[crate]}",
        )
        expected_path = f"crates/{crate}"
        check(
            dep["path"] == expected_path,
            f"workspace dependency {crate} path {dep['path']!r} does not match {expected_path!r}",
        )

    lock_path = ROOT / "Cargo.lock"
    if lock_path.exists():
        lock = read_toml(lock_path)
        locked = {pkg["name"]: pkg["version"] for pkg in lock.get("package", []) if pkg["name"] in CRATE_MANIFESTS}
        for crate, version in crate_versions.items():
            check(crate in locked, f"Cargo.lock is missing package {crate}")
            if crate in locked:
                check(
                    locked[crate] == version,
                    f"Cargo.lock {crate} version {locked[crate]} does not match {version}",
                )

    makefile = read_text(ROOT / "Makefile")
    make_version = first_match(r"^VERSION \?= ([^\s]+)$", makefile, "Makefile VERSION")
    check(make_version == release_version, f"Makefile VERSION {make_version} does not match {release_version}")

    for script_name in ["release_all.sh", "publish_verify.sh"]:
        script = read_text(ROOT / f"scripts/{script_name}")
        script_version = first_match(r'^VERSION="\$\{1:-([^}]+)\}"$', script, f"{script_name} default VERSION")
        check(
            script_version == release_version,
            f"scripts/{script_name} default VERSION {script_version} does not match {release_version}",
        )

    release_workflow = read_text(ROOT / ".github/workflows/release.yml")
    workflow_version = first_match(r'^\s+default: "([^"]+)"$', release_workflow, "release workflow default version")
    check(
        workflow_version == release_version,
        f"release workflow default version {workflow_version} does not match {release_version}",
    )

    post_release_workflow = read_text(ROOT / ".github/workflows/post-release-verify.yml")
    post_release_version = first_match(
        r'^\s+default: "([^"]+)"$',
        post_release_workflow,
        "post-release workflow default version",
    )
    check(
        post_release_version == release_version,
        f"post-release workflow default version {post_release_version} does not match {release_version}",
    )

    ci_workflow = read_text(ROOT / ".github/workflows/ci.yml")
    smoke_versions = re.findall(r"scripts/release_all\.sh ([0-9]+\.[0-9]+\.[0-9]+)", ci_workflow)
    check(bool(smoke_versions), "CI release smoke does not call scripts/release_all.sh with an explicit version")
    for version in smoke_versions:
        check(version == release_version, f"CI release smoke version {version} does not match {release_version}")

    package_json = read_json(ROOT / "vscode-gemstone-rs-workbench/package.json")
    package_lock = read_json(ROOT / "vscode-gemstone-rs-workbench/package-lock.json")
    vscode_version = package_json["version"]
    check(
        package_lock["version"] == vscode_version,
        f"package-lock top-level version {package_lock['version']} does not match package.json {vscode_version}",
    )
    root_lock_package = package_lock.get("packages", {}).get("", {})
    check(
        root_lock_package.get("version") == vscode_version,
        f"package-lock root package version {root_lock_package.get('version')} does not match package.json {vscode_version}",
    )

    release_snippet_paths = [
        "README.md",
        "docs/release-checklist.md",
        "docs/setup-guide.md",
        "docs/medium-article.md",
    ]
    release_snippet_patterns = [
        (r"scripts/release_all\.sh ([0-9]+\.[0-9]+\.[0-9]+)", "release_all.sh snippet"),
        (r"scripts/publish_verify\.sh ([0-9]+\.[0-9]+\.[0-9]+)", "publish_verify.sh snippet"),
        (r"-f version=([0-9]+\.[0-9]+\.[0-9]+)", "workflow version input"),
    ]
    for relative_path in release_snippet_paths:
        text = read_text(ROOT / relative_path)
        for pattern, label in release_snippet_patterns:
            for version in re.findall(pattern, text):
                check(
                    version == release_version,
                    f"{relative_path}: {label} version {version} does not match {release_version}",
                )

    vsix_filename_paths = [
        "README.md",
        "docs/release-checklist.md",
        "docs/vscode-workbench.md",
    ]
    for relative_path in vsix_filename_paths:
        text = read_text(ROOT / relative_path)
        for version in re.findall(r"gemstone-rs-workbench-([0-9]+\.[0-9]+\.[0-9]+)\.vsix", text):
            check(
                version == vscode_version,
                f"{relative_path}: VSIX filename version {version} does not match {vscode_version}",
            )

    release_checklist = read_text(ROOT / "docs/release-checklist.md")
    notes_versions = re.findall(
        r"^## ([0-9]+\.[0-9]+\.[0-9]+) / Workbench ([0-9]+\.[0-9]+\.[0-9]+) Notes$",
        release_checklist,
        re.MULTILINE,
    )
    if not notes_versions:
        raise AssertionError("could not find release checklist current notes heading")
    check(
        (release_version, vscode_version) in notes_versions,
        f"release checklist is missing {release_version} / Workbench {vscode_version} notes",
    )

    cli_source = read_text(ROOT / "crates/gemstone-rs-cli/src/main.rs")
    for adapter_crate in ["gemstone-rs-axum", "gemstone-rs-actix"]:
        check(
            f'{adapter_crate} = "{{gemstone_rs_version}}"' in cli_source,
            f"CLI scaffold for {adapter_crate} should use {{gemstone_rs_version}}",
        )
        hardcoded_versions = re.findall(
            rf'{re.escape(adapter_crate)} = "([0-9]+\.[0-9]+\.[0-9]+)"',
            cli_source,
        )
        for version in hardcoded_versions:
            check(
                version == crate_versions[adapter_crate],
                f"CLI scaffold {adapter_crate} version {version} does not match {crate_versions[adapter_crate]}",
            )

    if errors:
        for error in errors:
            print(f"version-check: {error}", file=sys.stderr)
        return 1

    print(f"version-check ok: crates={release_version}, vscode={vscode_version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
