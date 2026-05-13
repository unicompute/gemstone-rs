#!/usr/bin/env python3
"""Verify assets downloaded from a GitHub Release against SHA256SUMS."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKSUM_RE = re.compile(r"^([0-9a-fA-F]{64})\s+\*?(.+?)\s*$")


def read_vscode_version() -> str:
    package_json = ROOT / "vscode-gemstone-rs-workbench/package.json"
    return json.loads(package_json.read_text(encoding="utf-8"))["version"]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def parse_checksums(path: Path) -> dict[str, str]:
    entries: dict[str, str] = {}
    errors: list[str] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        match = CHECKSUM_RE.match(line)
        if not match:
            errors.append(f"{path}:{line_number}: invalid SHA256SUMS line")
            continue
        digest, name = match.groups()
        basename = Path(name).name
        if basename in entries:
            errors.append(f"{path}:{line_number}: duplicate release asset name {basename}")
            continue
        entries[basename] = digest.lower()
    if errors:
        raise ValueError("\n".join(errors))
    return entries


def expected_assets() -> list[str]:
    vscode_version = read_vscode_version()
    assets = ["SHA256SUMS", f"gemstone-rs-workbench-{vscode_version}.vsix"]
    assets.extend(pdf.name for pdf in sorted((ROOT / "docs/pdf").glob("*.pdf")))
    return assets


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("asset_dir", help="directory containing downloaded release assets")
    args = parser.parse_args(argv)

    asset_dir = Path(args.asset_dir).resolve()
    checksum_path = asset_dir / "SHA256SUMS"
    errors: list[str] = []

    if not asset_dir.is_dir():
        errors.append(f"asset directory does not exist: {asset_dir}")
    if not checksum_path.exists():
        errors.append(f"missing downloaded SHA256SUMS: {checksum_path}")

    expected = expected_assets()
    for name in expected:
        path = asset_dir / name
        if not path.exists():
            errors.append(f"missing downloaded release asset: {name}")
        elif path.stat().st_size == 0:
            errors.append(f"downloaded release asset is empty: {name}")

    checksum_entries: dict[str, str] = {}
    if checksum_path.exists():
        try:
            checksum_entries = parse_checksums(checksum_path)
        except ValueError as exc:
            errors.extend(str(exc).splitlines())

    for name in expected:
        if name == "SHA256SUMS":
            continue
        digest = checksum_entries.get(name)
        if digest is None:
            errors.append(f"SHA256SUMS is missing downloaded asset {name}")
            continue
        path = asset_dir / name
        if path.exists() and sha256(path) != digest:
            errors.append(f"SHA256 mismatch for downloaded asset {name}")

    for name in checksum_entries:
        if name not in expected:
            errors.append(f"SHA256SUMS lists unexpected release asset {name}")

    if errors:
        for error in errors:
            print(f"downloaded-release-check: {error}", file=sys.stderr)
        return 1

    print(f"downloaded-release-check ok: {len(expected)} assets")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
