#!/usr/bin/env node
/**
 * 从 Release 的签名文件（*.sig）合成自动更新清单 latest.json。
 *
 * 用法:
 *   node scripts/make-latest-json.mjs --version 0.2.0 --repo owner/repo \
 *     --sigs-dir sigs --out latest.json [--tag v0.2.0] [--notes "..."]
 *
 * 说明:
 * - Windows：NSIS 安装器 `.exe` 即更新包（`createUpdaterArtifacts: true`）
 * - macOS：通用二进制 `.app.tar.gz`，同时供 darwin-aarch64 / darwin-x86_64
 * - Linux：`.AppImage` 即更新包，按文件名区分 amd64 / aarch64
 * - 任一平台缺失即退出码 1（避免发布缺平台的清单）
 */
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

function parseArgs(argv) {
  const args = {};
  for (let i = 2; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, "");
    args[key] = argv[i + 1];
  }
  return args;
}

const args = parseArgs(process.argv);
const version = args.version;
const repo = args.repo;
const tag = args.tag ?? `v${version}`;
const sigsDir = args["sigs-dir"] ?? "sigs";
const outFile = args.out ?? "latest.json";
const notes = args.notes ?? `Anoterm ${version}`;

if (!version || !repo) {
  console.error("缺少必填参数: --version <版本> --repo <owner/repo>");
  process.exit(1);
}

const sigFiles = readdirSync(sigsDir).filter((f) => f.endsWith(".sig"));
const bySuffix = (suffix) => sigFiles.filter((f) => f.endsWith(suffix));

const signature = (file) => readFileSync(join(sigsDir, file), "utf-8").trim();
const entry = (file) => ({
  signature: signature(file),
  url: `https://github.com/${repo}/releases/download/${tag}/${file.slice(0, -".sig".length)}`,
});

const platforms = {};
const missing = [];

// Windows：NSIS 安装器 .exe（同时写基础键与 -nsis 后缀键）
const win = bySuffix("-setup.exe.sig");
if (win.length) {
  platforms["windows-x86_64"] = entry(win[0]);
  platforms["windows-x86_64-nsis"] = entry(win[0]);
} else {
  missing.push("windows-x86_64 (-setup.exe.sig)");
}

// macOS：通用二进制 .app.tar.gz（同时写基础键与 -app 后缀键）
const mac = bySuffix(".app.tar.gz.sig");
if (mac.length) {
  for (const key of ["darwin-aarch64", "darwin-x86_64", "darwin-aarch64-app", "darwin-x86_64-app"]) {
    platforms[key] = entry(mac[0]);
  }
} else {
  missing.push("darwin (universal .app.tar.gz.sig)");
}

// Linux：AppImage（同时写基础键与 -appimage 后缀键）
const linuxSigs = bySuffix(".AppImage.sig");
const linuxX64 = linuxSigs.find((f) => /_(amd64|x86_64)\.AppImage\.sig$/.test(f));
const linuxArm = linuxSigs.find((f) => /_aarch64\.AppImage\.sig$/.test(f));
if (linuxX64) {
  platforms["linux-x86_64"] = entry(linuxX64);
  platforms["linux-x86_64-appimage"] = entry(linuxX64);
} else {
  missing.push("linux-x86_64 (…_amd64.AppImage.sig)");
}
if (linuxArm) {
  platforms["linux-aarch64"] = entry(linuxArm);
  platforms["linux-aarch64-appimage"] = entry(linuxArm);
} else {
  missing.push("linux-aarch64 (…_aarch64.AppImage.sig)");
}

if (missing.length) {
  console.error("缺少以下平台的更新签名，无法生成 latest.json：");
  for (const item of missing) console.error(`  - ${item}`);
  console.error(`已找到的签名文件：${sigFiles.join(", ") || "(无)"}`);
  process.exit(1);
}

const manifest = {
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms,
};
writeFileSync(outFile, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`已生成 ${outFile}（平台：${Object.keys(platforms).join(", ")}）`);
