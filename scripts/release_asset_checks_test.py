#!/usr/bin/env python3
"""Offline smoke tests for release asset verification helpers."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def expected_release_assets() -> list[str]:
    package_json = json.loads(
        (ROOT / "vscode-gemstone-rs-workbench/package.json").read_text(encoding="utf-8")
    )
    assets = ["SHA256SUMS", f"gemstone-rs-workbench-{package_json['version']}.vsix"]
    assets.extend(pdf.name for pdf in sorted((ROOT / "docs/pdf").glob("*.pdf")))
    return assets


def run_downloaded_check(asset_dir: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "scripts/verify_downloaded_release_assets.py", str(asset_dir)],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def write_fake_release(asset_dir: Path, assets: list[str]) -> None:
    lines: list[str] = []
    for name in assets:
        if name == "SHA256SUMS":
            continue
        data = f"fake release asset: {name}\n".encode("utf-8")
        (asset_dir / name).write_bytes(data)
        checksum_name = (
            f"vscode-gemstone-rs-workbench/{name}"
            if name.endswith(".vsix")
            else f"docs/pdf/{name}"
        )
        lines.append(f"{sha256(data)}  {checksum_name}\n")
    (asset_dir / "SHA256SUMS").write_text("".join(lines), encoding="utf-8")


def assert_ok(result: subprocess.CompletedProcess[str]) -> None:
    if result.returncode != 0:
        raise AssertionError(
            f"expected success, got {result.returncode}\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
        )


def assert_fails_with(result: subprocess.CompletedProcess[str], text: str) -> None:
    combined = f"{result.stdout}\n{result.stderr}"
    if result.returncode == 0 or text not in combined:
        raise AssertionError(
            f"expected failure containing {text!r}, got {result.returncode}\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
        )


def main() -> int:
    assets = expected_release_assets()
    if len(assets) < 3:
        raise AssertionError("expected VSIX plus generated PDF assets")

    with tempfile.TemporaryDirectory(prefix="gemstone-rs-release-assets-test.") as tmp:
        asset_dir = Path(tmp)
        write_fake_release(asset_dir, assets)
        assert_ok(run_downloaded_check(asset_dir))

        pdf_name = next(name for name in assets if name.endswith(".pdf"))
        (asset_dir / pdf_name).write_text("corrupted\n", encoding="utf-8")
        assert_fails_with(run_downloaded_check(asset_dir), "SHA256 mismatch")

        write_fake_release(asset_dir, assets)
        (asset_dir / pdf_name).unlink()
        assert_fails_with(run_downloaded_check(asset_dir), "missing downloaded release asset")

    print(f"release asset verifier smoke checks passed for {len(assets)} expected assets")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
