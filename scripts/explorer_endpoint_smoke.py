#!/usr/bin/env python3
"""Smoke-test the gemstone-rs explorer over real HTTP."""

from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    try:
        port = free_port()
    except PermissionError as error:
        if os.environ.get("REQUIRE_EXPLORER_SMOKE") in {"1", "true", "TRUE", "yes"}:
            raise
        print(f"skipping explorer endpoint smoke: cannot bind loopback socket ({error})")
        return 0
    process = subprocess.Popen(
        [
            "cargo",
            "run",
            "-p",
            "gemstone-rs-explorer",
            "--",
            "--port",
            str(port),
            "--codegen-root",
            str(ROOT),
        ],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        base = f"http://127.0.0.1:{port}"
        wait_for_health(base, process)
        assert_json(f"{base}/health", {"status": "ok"})
        status_code, status = get_json_with_status(f"{base}/api/status")
        assert status_code in {200, 500, 503}
        assert "connected" in status

        profiles = get_json(
            f"{base}/api/codegen/profiles/check"
            "?profile_file=examples/codegen/gemstone-rs.codegen-profiles.json"
        )
        assert profiles["success"] is True
        assert profiles["ok"] is True
        assert profiles["profileCount"] == 3
        assert profiles["okCount"] == 3

        preview = get_json(
            f"{base}/api/codegen/preview-profile"
            "?profile=default"
            "&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json"
        )
        assert preview["success"] is True
        assert "pub struct Object" in preview["source"]

        print(f"explorer endpoint smoke checks passed on {base}")
        return 0
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def wait_for_health(base: str, process: subprocess.Popen[str]) -> None:
    deadline = time.time() + 30
    last_error: Exception | None = None
    while time.time() < deadline:
        if process.poll() is not None:
            stderr = process.stderr.read() if process.stderr else ""
            raise RuntimeError(f"explorer exited early with {process.returncode}: {stderr}")
        try:
            get_json(f"{base}/health")
            return
        except (urllib.error.URLError, json.JSONDecodeError, AssertionError) as error:
            last_error = error
            time.sleep(0.25)
    raise TimeoutError(f"explorer did not become ready: {last_error}")


def get_json(url: str) -> dict:
    status, value = get_json_with_status(url)
    assert 200 <= status < 300, f"{url}: expected 2xx, got {status}"
    return value


def get_json_with_status(url: str) -> tuple[int, dict]:
    try:
        with urllib.request.urlopen(url, timeout=5) as response:
            status = response.status
            payload = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        status = error.code
        payload = error.read().decode("utf-8")
    value = json.loads(payload)
    assert isinstance(value, dict), f"expected object from {url}"
    return status, value


def assert_json(url: str, expected: dict) -> None:
    actual = get_json(url)
    for key, value in expected.items():
        assert actual.get(key) == value, f"{url}: expected {key}={value}, got {actual.get(key)}"


if __name__ == "__main__":
    sys.exit(main())
