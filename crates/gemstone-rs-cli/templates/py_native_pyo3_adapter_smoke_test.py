import json

import gemstone_py_native
import gemstone_py_native_compat


def test_capabilities_json_names_contract():
    capabilities = json.loads(gemstone_py_native.capabilities_json())
    assert capabilities["name"] == "gemstone-py-native adapter contract"
    assert capabilities["contractVersion"] == 1
    assert "eval" in capabilities["operations"]
    assert "perform" in capabilities["operations"]


def test_dry_run_smoke_json_passes():
    report = json.loads(gemstone_py_native.smoke_dry_run_json())
    assert report["ok"] is True
    assert report["dryRun"] is True
    assert report["contractVersion"] == 1


def test_samples_json_covers_value_and_error_shapes():
    report = json.loads(gemstone_py_native.samples_json())
    value_kinds = {entry["value"]["kind"] for entry in report["values"]}
    error_kinds = {entry["error"]["kind"] for entry in report["errors"]}
    assert {"nil", "bool", "smallInt", "char", "string", "symbol", "oop"} <= value_kinds
    assert {"missingConfig", "illegalOop", "unexpectedType", "mapping"} <= error_kinds


def test_migration_json_tracks_python_wrapper_work():
    report = json.loads(gemstone_py_native.migration_json())
    assert report["targetPackage"] == "gemstone-py-native"
    assert report["contractVersion"] == 1
    assert report["pendingCount"] >= 1
    step_ids = {entry["id"] for entry in report["steps"]}
    assert "wrap_py_native_session" in step_ids
    assert "preserve_python_api" in step_ids


def test_compatibility_json_tracks_python_shim_work():
    report = json.loads(gemstone_py_native.compatibility_json())
    assert report["module"] == "gemstone_py_native_compat"
    assert report["sessionClass"] == "NativeCompatibilitySession"
    assert report["handleClass"] == "OopHandle"
    methods = {entry["pythonMethod"]: entry for entry in report["methods"]}
    assert methods["eval_value"]["pythonReturn"] == "dict"
    assert methods["eval_oop"]["pythonReturn"] == "OopHandle"
    assert methods["eval_smallint"]["pythonReturn"] == "int"
    assert methods["perform_oop"]["nativeMethod"] == "NativeSession.perform_raw_oop"
    assert methods["perform_value"]["nativeMethod"] == "NativeSession.perform_json"
    assert methods["value_to_oop_symbol"]["pythonReturn"] == "OopHandle"


def test_conformance_json_tracks_backend_surface():
    report = json.loads(gemstone_py_native.conformance_json())
    assert report["targetPackage"] == "gemstone-py-native"
    assert "compatibility_json" in report["moduleFunctions"]
    assert "conformance_json" in report["moduleFunctions"]
    assert "handoff_json" in report["moduleFunctions"]
    assert "eval_json" in report["nativeSessionMethods"]
    assert "perform_raw_oop" in report["nativeSessionMethods"]
    assert "perform_json" in report["nativeSessionMethods"]
    assert "perform_oop" in report["compatibilityMethods"]
    assert "perform_value" in report["compatibilityMethods"]
    fixture_paths = {entry["path"] for entry in report["fixtures"]}
    assert "examples/py-native/gemstone-rs.py-native-conformance.json" in fixture_paths
    assert "examples/py-native/gemstone-rs.py-native-handoff.json" in fixture_paths


def test_handoff_json_tracks_downstream_acceptance():
    report = json.loads(gemstone_py_native.handoff_json())
    assert report["targetPackage"] == "gemstone-py-native"
    artifact_names = {entry["name"] for entry in report["artifacts"]}
    assert {"capabilities", "samples", "smoke", "migration", "compatibility", "conformance"} <= artifact_names
    acceptance_ids = {entry["id"] for entry in report["acceptance"]}
    assert "fixtures_current" in acceptance_ids
    assert "live_native_backend_green" in acceptance_ids


def test_native_session_exposes_core_adapter_methods():
    expected = {
        "login_from_env",
        "session_id",
        "eval_repr",
        "eval_json",
        "eval_smallint",
        "eval_oop",
        "execute",
        "resolve",
        "value_to_oop_nil",
        "value_to_oop_bool",
        "value_to_oop_smallint",
        "value_to_oop_char",
        "value_to_oop_string",
        "value_to_oop_symbol",
        "value_to_oop_raw",
        "perform_raw_oop",
        "perform_json",
        "new_string",
        "new_symbol",
        "fetch_string",
        "global_get",
        "global_put_raw",
        "global_put_string",
        "global_put_smallint",
        "add_to_export_set",
        "remove_from_export_set",
        "needs_commit",
        "in_transaction",
        "commit",
        "abort",
        "logout",
    }
    missing = {name for name in expected if not hasattr(gemstone_py_native.NativeSession, name)}
    assert missing == set()


def test_compatibility_report_documents_return_policy():
    report = gemstone_py_native_compat.compatibility_report()
    assert report["module"] == "gemstone_py_native_compat"
    assert report["handleClass"] == "OopHandle"
    assert "typed helpers are opt-in" in report["returnPolicy"]


def test_oop_handle_wraps_raw_oop_values():
    handle = gemstone_py_native_compat.OopHandle(58)
    assert handle.raw == 58
    assert gemstone_py_native_compat.raw_oop(handle) == 58
    assert gemstone_py_native_compat.raw_oop(58) == 58
    assert gemstone_py_native_compat.raw_oops([handle, 60]) == [58, 60]
