#!/usr/bin/env python3
"""Smoke test Axum and Actix example HTTP routes."""

from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class HttpResult:
    status: int
    body: str
    headers: dict[str, str]


@dataclass(frozen=True)
class Service:
    name: str
    manifest: Path
    adapter: str


SERVICES = [
    Service("axum", ROOT / "examples/axum-service/Cargo.toml", "axum"),
    Service("actix", ROOT / "examples/actix-service/Cargo.toml", "actix"),
]


def free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def http_get(url: str, request_id: str = "gemstone-rs-smoke") -> HttpResult:
    request = urllib.request.Request(url, headers={"X-Request-Id": request_id})
    try:
        with urllib.request.urlopen(request, timeout=2.0) as response:
            headers = {key.lower(): value for key, value in response.headers.items()}
            return HttpResult(response.status, response.read().decode("utf-8"), headers)
    except urllib.error.HTTPError as err:
        headers = {key.lower(): value for key, value in err.headers.items()}
        return HttpResult(err.code, err.read().decode("utf-8"), headers)


def wait_for_local_health(base_url: str, process: subprocess.Popen[str]) -> None:
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        if process.poll() is not None:
            output = process.stdout.read() if process.stdout else ""
            raise AssertionError(f"service exited early with {process.returncode}\n{output}")
        try:
            result = http_get(f"{base_url}/health/local")
            if result.status == 200 and json.loads(result.body).get("ok") is True:
                return
        except Exception:
            pass
        time.sleep(0.2)
    raise AssertionError(f"timed out waiting for {base_url}/health/local")


def assert_json(result: HttpResult, expected_status: int) -> dict:
    if result.status != expected_status:
        raise AssertionError(f"expected HTTP {expected_status}, got {result.status}: {result.body}")
    try:
        return json.loads(result.body)
    except json.JSONDecodeError as err:
        raise AssertionError(f"response is not JSON: {result.body}") from err


def assert_diagnostics(service: Service, result: HttpResult, route: str) -> None:
    adapter = result.headers.get("x-gemstone-rs-adapter")
    if adapter != service.adapter:
        raise AssertionError(f"{service.name} adapter header mismatch: {adapter!r}")
    actual_route = result.headers.get("x-gemstone-rs-route")
    if actual_route != route:
        raise AssertionError(f"{service.name} route header mismatch: {actual_route!r}")
    request_id = result.headers.get("x-gemstone-rs-request-id")
    if request_id != "gemstone-rs-smoke":
        raise AssertionError(f"{service.name} request id header mismatch: {request_id!r}")
    method = result.headers.get("x-gemstone-rs-request-method")
    if method != "GET":
        raise AssertionError(f"{service.name} request method header mismatch: {method!r}")
    path = result.headers.get("x-gemstone-rs-request-path")
    expected_path = {
        "root": "/",
        "health.local": "/health/local",
        "health.gemstone": "/health/gemstone",
    }[route]
    if path != expected_path:
        raise AssertionError(f"{service.name} request path header mismatch: {path!r}")
    lifecycle = result.headers.get("x-gemstone-rs-request-lifecycle")
    if lifecycle != "received,handled":
        raise AssertionError(f"{service.name} lifecycle header mismatch: {lifecycle!r}")
    duration = result.headers.get("x-gemstone-rs-request-duration-us")
    if duration is None:
        raise AssertionError(f"{service.name} missing duration header")
    try:
        parsed_duration = int(duration)
    except ValueError as err:
        raise AssertionError(
            f"{service.name} duration header is not an integer: {duration!r}"
        ) from err
    if parsed_duration < 0:
        raise AssertionError(f"{service.name} duration header is negative: {duration!r}")
    middleware = result.headers.get("x-gemstone-rs-example-middleware")
    if middleware != service.adapter:
        raise AssertionError(f"{service.name} middleware header mismatch: {middleware!r}")


def check_routes(service: Service) -> None:
    port = free_port()
    base_url = f"http://127.0.0.1:{port}"
    command = [
        "cargo",
        "run",
        "--manifest-path",
        str(service.manifest),
        "--",
        "--host",
        "127.0.0.1",
        "--port",
        str(port),
        "--workers",
        "1",
    ]
    process = subprocess.Popen(
        command,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        wait_for_local_health(base_url, process)

        root_response = http_get(f"{base_url}/")
        assert_diagnostics(service, root_response, "root")
        root = assert_json(root_response, 200)
        if "name" not in root or root.get("endpoints", {}).get("gemstone") != "/health/gemstone":
            raise AssertionError(f"{service.name} root response is missing endpoint metadata: {root}")

        local_response = http_get(f"{base_url}/health/local")
        assert_diagnostics(service, local_response, "health.local")
        local = assert_json(local_response, 200)
        if local.get("ok") is not True:
            raise AssertionError(f"{service.name} local health is not ok: {local}")

        gemstone = http_get(f"{base_url}/health/gemstone")
        assert_diagnostics(service, gemstone, "health.gemstone")
        if os.environ.get("GS_RUN_LIVE_RUST") == "1":
            body = assert_json(gemstone, 200)
            if body.get("result") != 7:
                raise AssertionError(f"{service.name} live health did not return 7: {body}")
        elif gemstone.status == 200:
            body = assert_json(gemstone, 200)
            if body.get("result") != 7:
                raise AssertionError(f"{service.name} health returned unexpected success: {body}")
        else:
            body = assert_json(gemstone, 503)
            if "error" not in body:
                raise AssertionError(f"{service.name} missing error body for unavailable health: {body}")
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def main() -> int:
    for service in SERVICES:
        check_routes(service)
        print(f"{service.name} framework route smoke passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
