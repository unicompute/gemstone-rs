import json

import gemstone_py_native


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
