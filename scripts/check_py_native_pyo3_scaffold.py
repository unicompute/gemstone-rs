#!/usr/bin/env python3
"""Verify the generated PyO3 adapter scaffold against the local Rust core."""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path


def is_network_failure(output: str) -> bool:
    return any(
        needle in output
        for needle in [
            "Could not resolve host",
            "failed to download",
            "spurious network error",
            "download of config.json failed",
        ]
    )


def run(cmd: list[str], *, cwd: Path, retry_offline: bool = False) -> subprocess.CompletedProcess[str]:
    print("+ " + " ".join(cmd))
    result = subprocess.run(
        cmd,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.returncode == 0:
        if result.stdout.strip():
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
        return result

    if retry_offline and "--offline" not in cmd and is_network_failure(result.stdout):
        offline_cmd = [*cmd, "--offline"]
        print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
        print("+ " + " ".join(offline_cmd))
        offline = subprocess.run(
            offline_cmd,
            cwd=cwd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        if offline.returncode == 0:
            if offline.stdout.strip():
                print(offline.stdout, end="" if offline.stdout.endswith("\n") else "\n")
            return offline
        print(offline.stdout, end="" if offline.stdout.endswith("\n") else "\n")
        offline.check_returncode()

    print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
    result.check_returncode()
    return result


def patch_local_dependency(cargo_toml: Path, repo_root: Path) -> None:
    text = cargo_toml.read_text()
    replacement = f'gemstone-rs = {{ path = "{repo_root / "crates" / "gemstone-rs"}" }}'
    patched = re.sub(r'(?m)^gemstone-rs = "[^"]+"$', replacement, text, count=1)
    if patched == text:
        raise RuntimeError(f"could not find gemstone-rs dependency in {cargo_toml}")
    cargo_toml.write_text(patched)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Scaffold and compile the gemstone-py-native PyO3 starter project."
    )
    parser.add_argument(
        "--target",
        type=Path,
        default=Path("/tmp/gemstone-rs-scaffold-py-native-pyo3-adapter-check"),
        help="temporary scaffold target directory",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="pass --offline to cargo check/run",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    target = args.target.resolve()
    if target.exists():
        shutil.rmtree(target)

    run(
        [
            "cargo",
            "run",
            "-p",
            "gemstone-rs-cli",
            "--",
            "examples",
            "scaffold",
            "py_native_pyo3_adapter",
            str(target),
            "--force",
        ],
        cwd=repo_root,
    )
    patch_local_dependency(target / "Cargo.toml", repo_root)

    cargo_check = [
        "cargo",
        "check",
        "--manifest-path",
        str(target / "Cargo.toml"),
    ]
    cargo_run = [
        "cargo",
        "run",
        "--manifest-path",
        str(target / "Cargo.toml"),
    ]
    if args.offline:
        cargo_check.append("--offline")
        cargo_run.append("--offline")

    run(cargo_check, cwd=repo_root, retry_offline=not args.offline)
    run_result = run(cargo_run, cwd=repo_root, retry_offline=not args.offline)
    lib_source = (target / "src" / "lib.rs").read_text()
    pyproject_source = (target / "pyproject.toml").read_text()
    compat_source = (target / "python" / "gemstone_py_native_compat.py").read_text()
    for needle in [
        "samples_json:",
        "smoke_json:",
        "migration_json:",
        "compatibility_json:",
        "conformance_json:",
        "handoff_json:",
        '"targetPackage":"gemstone-py-native"',
        '"id":"wrap_py_native_session"',
        '"id":"fixtures_current"',
        '"module":"gemstone_py_native_compat"',
        '"pythonMethod":"eval_value"',
        '"pythonMethod":"eval_oop"',
        '"pythonMethod":"perform_value"',
        '"pythonMethod":"value_to_oop_symbol"',
        '"moduleFunctions":["capabilities_json"',
        '"nativeSessionMethods":["login_from_env"',
        '"eval_json"',
        '"perform_json"',
    ]:
        if needle not in run_result.stdout:
            raise RuntimeError(f"scaffold cargo run output missing {needle!r}")
    for method in [
        "fn eval_json(",
        "fn eval_oop(",
        "fn execute(",
        "fn resolve(",
        "fn value_to_oop_nil(",
        "fn value_to_oop_bool(",
        "fn value_to_oop_smallint(",
        "fn value_to_oop_char(",
        "fn value_to_oop_string(",
        "fn value_to_oop_symbol(",
        "fn value_to_oop_raw(",
        "fn perform_raw_oop(",
        "fn perform_json(",
        "fn global_put_string(",
        "fn new_symbol(",
        "fn commit(",
        "fn abort(",
        "fn compatibility_json()",
        "fn conformance_json()",
        "fn handoff_json()",
    ]:
        if method not in lib_source:
            raise RuntimeError(f"scaffold src/lib.rs missing {method!r}")
    for needle in [
        'python-source = "python"',
    ]:
        if needle not in pyproject_source:
            raise RuntimeError(f"scaffold pyproject.toml missing {needle!r}")
    for needle in [
        "class NativeCompatibilitySession",
        "class OopHandle",
        "def eval_value",
        "def perform_value",
        "def value_to_oop_symbol",
        "def compatibility_report",
        "compatibility_json",
        "def raw_oop",
    ]:
        if needle not in compat_source:
            raise RuntimeError(f"scaffold compatibility shim missing {needle!r}")
    print(f"verified PyO3 scaffold at {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
