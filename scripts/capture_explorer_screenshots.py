#!/usr/bin/env python3
"""Capture repeatable gemstone-rs explorer screenshots."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = ROOT / "docs" / "assets" / "explorer-home.png"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", default="http://127.0.0.1:8787/")
    parser.add_argument("--port", type=int, default=8787)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--width", type=int, default=1440)
    parser.add_argument("--height", type=int, default=1500)
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument(
        "--no-server",
        action="store_true",
        help="Capture an already-running explorer instead of starting cargo run.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Validate paths and print the planned capture without starting a browser.",
    )
    return parser.parse_args()


def health_url(page_url: str) -> str:
    return page_url.rstrip("/") + "/health"


def wait_for_health(page_url: str, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    url = health_url(page_url)
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            with urllib.request.urlopen(url, timeout=1.0) as response:
                if response.status == 200:
                    return
        except (OSError, urllib.error.URLError) as err:
            last_error = err
        time.sleep(0.25)
    raise RuntimeError(f"timed out waiting for {url}: {last_error}")


def start_explorer(port: int) -> subprocess.Popen[str]:
    return subprocess.Popen(
        ["cargo", "run", "-p", "gemstone-rs-explorer", "--", "--port", str(port)],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )


def stop_process(process: subprocess.Popen[str] | None) -> None:
    if process is None or process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)


def capture_with_python_playwright(url: str, output: Path, width: int, height: int) -> bool:
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        return False

    with sync_playwright() as playwright:
        browser = playwright.chromium.launch()
        page = browser.new_page(viewport={"width": width, "height": height})
        page.goto(url, wait_until="networkidle")
        page.screenshot(path=str(output), full_page=True)
        browser.close()
    return True


def local_playwright_cli() -> list[str] | None:
    candidates = [
        ROOT / "node_modules" / ".bin" / "playwright",
        ROOT / "vscode-gemstone-rs-workbench" / "node_modules" / ".bin" / "playwright",
    ]
    for candidate in candidates:
        if candidate.exists():
            return [str(candidate)]
    command = shutil.which("playwright")
    if command:
        return [command]
    return None


def capture_with_playwright_cli(url: str, output: Path, width: int, height: int) -> bool:
    command = local_playwright_cli()
    if command is None:
        return False
    subprocess.run(
        [
            *command,
            "screenshot",
            f"--viewport-size={width},{height}",
            url,
            str(output),
        ],
        cwd=ROOT,
        check=True,
    )
    return True


def local_chrome_cli() -> list[str] | None:
    candidates = [
        Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
        Path("/Applications/Chromium.app/Contents/MacOS/Chromium"),
        Path("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return [str(candidate)]
    for command in ("google-chrome", "chromium", "chromium-browser", "microsoft-edge"):
        resolved = shutil.which(command)
        if resolved:
            return [resolved]
    return None


def capture_with_chrome(url: str, output: Path, width: int, height: int) -> bool:
    command = local_chrome_cli()
    if command is None:
        return False
    with tempfile.TemporaryDirectory(prefix="gemstone-rs-screenshot-") as user_data_dir:
        process = subprocess.Popen(
            [
                *command,
                "--headless=new",
                "--disable-background-networking",
                "--disable-component-update",
                "--disable-gpu",
                "--hide-scrollbars",
                "--no-first-run",
                f"--user-data-dir={user_data_dir}",
                f"--window-size={width},{height}",
                f"--screenshot={output}",
                url,
            ],
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            process.communicate(timeout=20)
        except subprocess.TimeoutExpired:
            process.terminate()
            try:
                process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.communicate(timeout=5)
            if output.exists() and output.stat().st_size > 0:
                return True
            return False
        if process.returncode != 0:
            return output.exists() and output.stat().st_size > 0
    return True


def capture(url: str, output: Path, width: int, height: int) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    if capture_with_python_playwright(url, output, width, height):
        return
    if capture_with_playwright_cli(url, output, width, height):
        return
    if capture_with_chrome(url, output, width, height):
        return
    raise RuntimeError(
        "Playwright is required for screenshot capture. Install either "
        "'python3 -m pip install playwright && python3 -m playwright install chromium' "
        "or a local Playwright CLI. A local Chrome/Chromium executable can also "
        "be used as a dependency-free fallback."
    )


def main() -> int:
    args = parse_args()
    output = args.output if args.output.is_absolute() else ROOT / args.output

    if args.check:
        print(f"url={args.url}")
        print(f"output={output}")
        print(f"viewport={args.width}x{args.height}")
        return 0

    process: subprocess.Popen[str] | None = None
    try:
        if not args.no_server:
            process = start_explorer(args.port)
        wait_for_health(args.url, args.timeout)
        capture(args.url, output, args.width, args.height)
        print(output)
        return 0
    finally:
        stop_process(process)


if __name__ == "__main__":
    raise SystemExit(main())
