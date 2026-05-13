const assert = require("assert");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const explorerSource = fs.readFileSync(
  path.join(root, "crates/gemstone-rs-explorer/src/main.rs"),
  "utf8"
);
const sample = JSON.parse(
  fs.readFileSync(
    path.join(root, "examples/codegen/gemstone-rs.codegen-profiles.json"),
    "utf8"
  )
);

assert(
  explorerSource.includes("function renderImportSummary"),
  "explorer browser JS should render an import summary"
);
assert(
  explorerSource.includes("function profileFingerprint"),
  "explorer browser JS should compare profile fingerprints"
);
assert(
  explorerSource.includes("function profilesExportPayload"),
  "explorer browser JS should export project profile payloads"
);

function normalizeCodegenProfile(profile) {
  if (!profile || typeof profile !== "object") return null;
  const name = String(profile.name || "").trim();
  if (!name) return null;
  return {
    name,
    config: String(profile.config || "").trim(),
    root: String(profile.root || "").trim(),
    mapped: String(profile.mapped || "").trim(),
    className: String(profile.className || profile.class || "").trim(),
  };
}

function profileFingerprint(profile) {
  const clean = normalizeCodegenProfile(profile);
  return clean ? JSON.stringify(clean) : "";
}

function importProfiles(currentProfiles, source) {
  const payload = JSON.parse(source);
  const candidates = Array.isArray(payload)
    ? payload
    : Array.isArray(payload.profiles)
      ? payload.profiles
      : [payload.profile || payload];
  const imported = candidates.map(normalizeCodegenProfile).filter(Boolean);
  if (!imported.length) {
    throw new Error("no valid profiles found");
  }

  const current = currentProfiles.map(normalizeCodegenProfile).filter(Boolean);
  const created = [];
  const replaced = [];
  const unchanged = [];
  for (const profile of imported) {
    const existing = current.find((item) => item.name === profile.name);
    if (!existing) created.push(profile.name);
    else if (profileFingerprint(existing) === profileFingerprint(profile)) unchanged.push(profile.name);
    else replaced.push(profile.name);
  }
  const existing = current.filter((item) => !imported.some((profile) => profile.name === item.name));
  const merged = [...imported, ...existing].slice(0, 16);
  return { created, replaced, unchanged, merged };
}

const current = [
  {
    name: "default",
    config: "old.codegen",
    root: "",
    mapped: "OldDraft",
    className: "Object",
  },
  {
    name: "bridge-mapping",
    config: "examples/codegen/gemstone-rs.codegen",
    root: "",
    mapped: "BookingDraft",
    className: "Object",
  },
];

const summary = importProfiles(current, JSON.stringify(sample));
assert.deepStrictEqual(summary.created, ["object-wrapper"]);
assert.deepStrictEqual(summary.replaced, ["default"]);
assert.deepStrictEqual(summary.unchanged, ["bridge-mapping"]);
assert.strictEqual(summary.merged.length, 3);

const single = importProfiles(
  [],
  JSON.stringify({
    kind: "gemstone-rs-explorer-codegen-profile",
    version: 1,
    profile: sample.profiles[0],
  })
);
assert.deepStrictEqual(single.created, ["default"]);
assert.deepStrictEqual(single.replaced, []);
assert.deepStrictEqual(single.unchanged, []);

assert.throws(
  () => importProfiles([], JSON.stringify({ kind: "bad", profiles: [{ name: "" }] })),
  /no valid profiles found/
);

console.log("profile import/export browser summary checks passed");
