"""Python compatibility helpers for the gemstone-py-native PyO3 starter.

The extension module intentionally exposes a direct native surface. This file
shows how the Python package layer can keep stable, explicit return behavior on
top of that native surface.
"""

from __future__ import annotations

from dataclasses import dataclass
import json
from typing import Dict, Iterable, List, Union

import gemstone_py_native


@dataclass(frozen=True)
class OopHandle:
    """Explicit Python handle for a GemStone object identity."""

    raw: int


OopLike = Union[OopHandle, int]


def raw_oop(value: OopLike) -> int:
    """Return the raw GemStone OOP integer from a handle or raw integer."""

    if isinstance(value, OopHandle):
        return value.raw
    if isinstance(value, int):
        return value
    raise TypeError(f"expected OopHandle or int, got {type(value).__name__}")


def raw_oops(values: Iterable[OopLike]) -> List[int]:
    """Return raw GemStone OOP integers from a sequence of handles."""

    return [raw_oop(value) for value in values]


class NativeCompatibilitySession:
    """Compatibility wrapper above gemstone_py_native.NativeSession.

    Methods that return GemStone object identity wrap raw OOP integers in
    OopHandle. Typed helpers remain explicit opt-in methods.
    """

    def __init__(self, native: gemstone_py_native.NativeSession) -> None:
        self._native = native

    @classmethod
    def login_from_env(cls) -> "NativeCompatibilitySession":
        return cls(gemstone_py_native.NativeSession.login_from_env())

    def session_id(self) -> int:
        return self._native.session_id()

    def eval_repr(self, source: str) -> str:
        return self._native.eval_repr(source)

    def eval_smallint(self, source: str) -> int:
        return self._native.eval_smallint(source)

    def eval_oop(self, source: str) -> OopHandle:
        return OopHandle(self._native.eval_oop(source))

    def execute(self, source: str) -> OopHandle:
        return OopHandle(self._native.execute(source))

    def resolve(self, name: str) -> OopHandle:
        return OopHandle(self._native.resolve(name))

    def perform_oop(
        self,
        receiver: OopLike,
        selector: str,
        args: Iterable[OopLike] = (),
    ) -> OopHandle:
        return OopHandle(
            self._native.perform_raw_oop(raw_oop(receiver), selector, raw_oops(args))
        )

    def new_string(self, value: str) -> OopHandle:
        return OopHandle(self._native.new_string(value))

    def fetch_string(self, oop: OopLike) -> str:
        return self._native.fetch_string(raw_oop(oop))

    def global_get(self, symbol_name: str) -> OopHandle:
        return OopHandle(self._native.global_get(symbol_name))

    def global_put_oop(self, symbol_name: str, value: OopLike) -> None:
        self._native.global_put_raw(symbol_name, raw_oop(value))

    def global_put_string(self, symbol_name: str, value: str) -> None:
        self._native.global_put_string(symbol_name, value)

    def global_put_smallint(self, symbol_name: str, value: int) -> None:
        self._native.global_put_smallint(symbol_name, value)

    def add_to_export_set(self, oop: OopLike) -> None:
        self._native.add_to_export_set(raw_oop(oop))

    def remove_from_export_set(self, oop: OopLike) -> None:
        self._native.remove_from_export_set(raw_oop(oop))

    def needs_commit(self) -> bool:
        return self._native.needs_commit()

    def in_transaction(self) -> bool:
        return self._native.in_transaction()

    def commit(self) -> None:
        self._native.commit()

    def abort(self) -> None:
        self._native.abort()

    def logout(self) -> None:
        self._native.logout()


def compatibility_report() -> Dict[str, object]:
    """Return the Rust migration report plus Python return-policy notes."""

    report = json.loads(gemstone_py_native.migration_json())
    report["compatLayer"] = {
        "module": "gemstone_py_native_compat",
        "session": "NativeCompatibilitySession",
        "oopHandle": "OopHandle",
        "returnPolicy": (
            "object identity returns OopHandle, raw native OOPs stay below the "
            "package boundary, and typed helpers are opt-in"
        ),
    }
    return report
