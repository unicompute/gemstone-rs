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
    for needle in [
        "samples_json:",
        "smoke_json:",
        "migration_json:",
        '"targetPackage":"gemstone-py-native"',
        '"id":"wrap_py_native_session"',
    ]:
        if needle not in run_result.stdout:
            raise RuntimeError(f"scaffold cargo run output missing {needle!r}")
    print(f"verified PyO3 scaffold at {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
