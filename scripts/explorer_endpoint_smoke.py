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
    run_one_smoke(auth_token=None)
    run_one_smoke(auth_token="smoke-token")
    return 0


def run_one_smoke(auth_token: str | None) -> None:
    try:
        port = free_port()
    except PermissionError as error:
        if os.environ.get("REQUIRE_EXPLORER_SMOKE") in {"1", "true", "TRUE", "yes"}:
            raise
        print(f"skipping explorer endpoint smoke: cannot bind loopback socket ({error})")
        return
    command = [
        "cargo",
        "run",
        "-p",
        "gemstone-rs-explorer",
        "--",
        "--port",
        str(port),
        "--codegen-root",
        str(ROOT),
    ]
    if auth_token:
        command.extend(["--auth-token", auth_token])
    process = subprocess.Popen(
        command,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        base = f"http://127.0.0.1:{port}"
        wait_for_health(base, process)
        assert_json(f"{base}/health", {"status": "ok"})
        if auth_token:
            status_code, denied = get_json_with_status(f"{base}/api/config")
            assert status_code == 401
            assert "auth token" in denied["error"]
            assert_json(f"{base}/api/config?token={auth_token}", {"authRequired": True})
            assert_json(
                f"{base}/api/config",
                {"authRequired": True},
                headers={"X-GemStone-RS-Token": auth_token},
            )
            suffix = f"?token={auth_token}"
            separator = "&"
        else:
            assert_json(f"{base}/api/config", {"authRequired": False})
            suffix = ""
            separator = "?"

        root_html = get_text(f"{base}/{suffix}")
        assert "gemstone-rs Explorer" in root_html
        assert "browseStatus" in root_html
        assert "metaClass" in root_html
        assert "renderSourceDetail" in root_html
        assert "copyDetailText" in root_html

        status_code, status = get_json_with_status(f"{base}/api/status{suffix}")
        assert status_code in {200, 500, 503}
        assert "connected" in status

        comparison = get_json(f"{base}/api/compare/gemstone-py/status{suffix}")
        assert comparison["comparison"] == "gemstone-py"
        assert comparison["view"] == "status"
        assert comparison["remaining"]["totalBatches"] == 5
        assert comparison["parity"]["project"] == "gemstone-rs"

        combined = get_json(f"{base}/api/compare/all/status{suffix}")
        assert combined["comparison"] == "all"
        assert combined["totalBatches"] == 11
        assert len(combined["comparisons"]) == 2

        profiles = get_json(
            f"{base}/api/codegen/profiles/check"
            f"{suffix}{separator}profile_file=examples/codegen/gemstone-rs.codegen-profiles.json"
        )
        assert profiles["success"] is True
        assert profiles["ok"] is True
        assert profiles["profileCount"] == 3
        assert profiles["okCount"] == 3

        preview = get_json(
            f"{base}/api/codegen/preview-profile"
            f"{suffix}{separator}profile=default"
            "&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json"
        )
        assert preview["success"] is True
        assert "pub struct Object" in preview["source"]

        output = get_json(
            f"{base}/api/codegen/output-profile"
            f"{suffix}{separator}profile=default"
            "&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json"
        )
        assert output["success"] is True
        assert output["exists"] is True
        assert output["output"].endswith("gemstone_wrappers.rs")
        assert "pub struct Object" in output["source"]

        mode = "auth" if auth_token else "open"
        print(f"explorer endpoint smoke checks passed on {base} ({mode})")
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


def get_json(url: str, headers: dict[str, str] | None = None) -> dict:
    status, value = get_json_with_status(url, headers=headers)
    assert 200 <= status < 300, f"{url}: expected 2xx, got {status}"
    return value


def get_text(url: str, headers: dict[str, str] | None = None) -> str:
    request = urllib.request.Request(url, headers=headers or {})
    with urllib.request.urlopen(request, timeout=5) as response:
        assert 200 <= response.status < 300, f"{url}: expected 2xx, got {response.status}"
        return response.read().decode("utf-8")


def get_json_with_status(
    url: str, headers: dict[str, str] | None = None
) -> tuple[int, dict]:
    request = urllib.request.Request(url, headers=headers or {})
    try:
        with urllib.request.urlopen(request, timeout=5) as response:
            status = response.status
            payload = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        status = error.code
        payload = error.read().decode("utf-8")
    value = json.loads(payload)
    assert isinstance(value, dict), f"expected object from {url}"
    return status, value


def assert_json(url: str, expected: dict, headers: dict[str, str] | None = None) -> None:
    actual = get_json(url, headers=headers)
    for key, value in expected.items():
        assert actual.get(key) == value, f"{url}: expected {key}={value}, got {actual.get(key)}"


if __name__ == "__main__":
    sys.exit(main())
