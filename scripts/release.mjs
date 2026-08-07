#!/usr/bin/env node


import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const OUT = path.join(ROOT, "dist", "release");

const BASE_URL = "https://zsync.eu/zyntaxai/releases";

const args = process.argv.slice(2);
const notes = valueOf("--notes");
const skipBuild = args.includes("--skip-build");
const triple = valueOf("--target");

const BUNDLE = triple
  ? path.join(ROOT, "target", triple, "release", "bundle")
  : path.join(ROOT, "target", "release", "bundle");

const version = JSON.parse(
  fs.readFileSync(path.join(ROOT, "src-tauri", "tauri.conf.json"), "utf8"),
).version;

const ARCH = triple
  ? { x86_64: "x86_64", aarch64: "aarch64" }[triple.split("-")[0]]
  : { x64: "x86_64", arm64: "aarch64" }[process.arch];
const OS = { linux: "linux", win32: "windows", darwin: "darwin" }[process.platform];
if (!ARCH || !OS) fail(`unsupported platform: ${triple ?? `${process.platform}/${process.arch}`}`);
const TARGET = `${OS}-${ARCH}`;


const PATTERNS = {
  linux: { dir: "appimage", ext: ".AppImage" },


  windows: { dir: "nsis", ext: "-setup.exe" },
  darwin: { dir: "macos", ext: ".app.tar.gz" },
}[OS];

if (!skipBuild) {
  console.log(`Building ZyntaxAI ${version} for ${TARGET}…`);
  if (!process.env.TAURI_SIGNING_PRIVATE_KEY) {
    fail(
      "no signing key. Set TAURI_SIGNING_PRIVATE_KEY to the key or its path\n" +
        "(and TAURI_SIGNING_PRIVATE_KEY_PASSWORD, empty if it has none).\n" +
        "Without it the bundle has no signature and the updater will reject it.",
    );
  }

  const cli = path.join(ROOT, "node_modules", "@tauri-apps", "cli", "tauri.js");
  if (!fs.existsSync(cli)) fail(`the Tauri CLI is missing at ${cli} — run pnpm install first`);

  execFileSync(process.execPath, [cli, "build", ...(triple ? ["--target", triple] : [])], {
    cwd: ROOT,
    stdio: "inherit",
    env: {
      ...process.env,
      ...(OS === "linux" ? { NO_STRIP: "true" } : {}),
    },
  });
}

const dir = path.join(BUNDLE, PATTERNS.dir);
const artifact = fs
  .readdirSync(dir)
  .filter((name) => name.endsWith(PATTERNS.ext))
  .sort()
  .pop();
if (!artifact) fail(`no ${PATTERNS.ext} bundle in ${dir}`);

const signaturePath = path.join(dir, `${artifact}.sig`);
if (!fs.existsSync(signaturePath)) {
  fail(
    `${artifact} has no .sig beside it.\n` +
      "Either the build ran without a signing key, or createUpdaterArtifacts\n" +
      "is off in tauri.conf.json. An unsigned artifact can never be installed.",
  );
}

const published =
  OS === "darwin" ? `ZyntaxAI_${version}_${ARCH}.app.tar.gz` : artifact;

const versionDir = path.join(OUT, version);
fs.mkdirSync(versionDir, { recursive: true });
fs.copyFileSync(path.join(dir, artifact), path.join(versionDir, published));
fs.copyFileSync(signaturePath, path.join(versionDir, `${published}.sig`));

for (const extra of collectInstallers()) {
  fs.copyFileSync(extra, path.join(versionDir, path.basename(extra)));
}

const manifestPath = path.join(OUT, "latest.json");
let manifest = { version, notes: notes ?? "", pub_date: new Date().toISOString(), platforms: {} };

if (fs.existsSync(manifestPath)) {
  const existing = JSON.parse(fs.readFileSync(manifestPath, "utf8"));


  if (existing.version === version) {
    manifest = { ...existing, notes: notes ?? existing.notes, platforms: existing.platforms ?? {} };
  }
}

manifest.platforms[TARGET] = {
  signature: fs.readFileSync(signaturePath, "utf8").trim(),
  url: `${BASE_URL}/${version}/${encodeURIComponent(published)}`,
};

fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);

const built = fs.readdirSync(versionDir);
console.log(`\nRelease ${version} — ${TARGET}\n`);
for (const name of built) {
  const size = fs.statSync(path.join(versionDir, name)).size;
  console.log(`  ${name}  (${(size / 1024 / 1024).toFixed(1)} MB)`);
  console.log(`    sha256 ${sha256(path.join(versionDir, name))}`);
}
console.log(`\nPlatforms in the manifest: ${Object.keys(manifest.platforms).join(", ")}`);
console.log(`\nUpload:\n  ${path.relative(ROOT, versionDir)}/  →  ${BASE_URL}/${version}/`);
console.log(`  ${path.relative(ROOT, manifestPath)}  →  https://zsync.eu/zyntaxai/latest.json`);
console.log(
  "\nUpload the artifacts before the manifest. The other order advertises a\n" +
    "download that is not there yet.",
);

function collectInstallers() {
  const wanted = [".deb", ".rpm", ".AppImage", ".msi", "-setup.exe", ".dmg"];
  const found = [];
  const walk = (current) => {
    if (!fs.existsSync(current)) return;
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const full = path.join(current, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (wanted.some((ext) => entry.name.endsWith(ext))) found.push(full);
    }
  };
  walk(BUNDLE);
  return found;
}

function sha256(file) {
  return createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function valueOf(flag) {
  const index = args.indexOf(flag);
  return index === -1 ? undefined : args[index + 1];
}

function fail(message) {
  console.error(`\nrelease: ${message}\n`);
  process.exit(1);
}
