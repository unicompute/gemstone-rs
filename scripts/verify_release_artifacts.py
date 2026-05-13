#!/usr/bin/env python3
"""Verify release assets and SHA256SUMS before publishing a GitHub release."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKSUM_RE = re.compile(r"^([0-9a-fA-F]{64})\s+\*?(.+?)\s*$")


def rel(path: Path) -> str:
    return path.resolve().relative_to(ROOT).as_posix()


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
        digest, raw_name = match.groups()
        asset_path = Path(raw_name)
        if asset_path.is_absolute():
            try:
                name = asset_path.resolve().relative_to(ROOT).as_posix()
            except ValueError:
                errors.append(f"{path}:{line_number}: asset path is outside repository: {raw_name}")
                continue
        else:
            name = asset_path.as_posix()
        if name in entries:
            errors.append(f"{path}:{line_number}: duplicate checksum entry for {name}")
            continue
        entries[name] = digest.lower()
    if errors:
        raise ValueError("\n".join(errors))
    return entries


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--checksums",
        default="SHA256SUMS",
        help="checksum file to verify, relative to the repository root by default",
    )
    parser.add_argument(
        "--vsix",
        default=None,
        help="expected VSIX asset path; defaults to the package.json versioned VSIX",
    )
    args = parser.parse_args(argv)

    checksum_path = Path(args.checksums)
    if not checksum_path.is_absolute():
        checksum_path = ROOT / checksum_path

    vscode_version = read_vscode_version()
    vsix_path = Path(args.vsix) if args.vsix else Path(
        f"vscode-gemstone-rs-workbench/gemstone-rs-workbench-{vscode_version}.vsix"
    )
    if not vsix_path.is_absolute():
        vsix_path = ROOT / vsix_path

    errors: list[str] = []

    if not checksum_path.exists():
        errors.append(f"missing checksum file: {checksum_path}")
    if not vsix_path.exists():
        errors.append(f"missing VSIX asset: {vsix_path}")

    pdfs = sorted((ROOT / "docs/pdf").glob("*.pdf"))
    if not pdfs:
        errors.append("missing PDF assets under docs/pdf")

    required = [vsix_path, *pdfs]
    for path in required:
        if path.exists() and path.stat().st_size == 0:
            errors.append(f"release asset is empty: {rel(path)}")

    entries: dict[str, str] = {}
    if checksum_path.exists():
        try:
            entries = parse_checksums(checksum_path)
        except ValueError as exc:
            errors.extend(str(exc).splitlines())

    for path in required:
        if not path.exists():
            continue
        name = rel(path)
        expected = entries.get(name)
        if expected is None:
            errors.append(f"SHA256SUMS is missing {name}")
            continue
        actual = sha256(path)
        if actual != expected:
            errors.append(f"SHA256 mismatch for {name}: expected {expected}, got {actual}")

    for name in entries:
        path = ROOT / name
        if not path.exists():
            errors.append(f"SHA256SUMS lists missing file: {name}")
        elif path.stat().st_size == 0:
            errors.append(f"SHA256SUMS lists empty file: {name}")

    if errors:
        for error in errors:
            print(f"release-artifact-check: {error}", file=sys.stderr)
        return 1

    print(f"release-artifact-check ok: {len(required)} assets, {len(entries)} checksums")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
