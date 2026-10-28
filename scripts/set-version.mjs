#!/usr/bin/env node
/**
 * 设置应用版本号：一次同步四个位置
 *   package.json / src-tauri/tauri.conf.json / src-tauri/Cargo.toml / src-tauri/Cargo.lock
 *
 * 用法:
 *   pnpm run version 0.3.0      # 四个文件统一设为 0.3.0
 *   pnpm run version            # 无参数时按 package.json 当前版本对齐其余文件
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function read(rel) {
  return readFileSync(join(root, rel), "utf8");
}

/** 内容有变化才写盘，返回是否修改 */
function writeIfChanged(rel, next) {
  if (read(rel) === next) return false;
  writeFileSync(join(root, rel), next);
  console.log(`[version] ${rel}`);
  return true;
}

function readJson(rel) {
  return JSON.parse(read(rel));
}

function writeJson(rel, data) {
  return writeIfChanged(rel, `${JSON.stringify(data, null, 2)}\n`);
}

const arg = (process.argv[2] ?? "").trim().replace(/^v/, "");
const pkg = readJson("package.json");
const version = arg || pkg.version;

if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`[version] 无效版本号：${version || "(空)"}`);
  console.error("[version] 用法：pnpm run version 0.3.0");
  process.exit(1);
}

let changed = false;

// 1. package.json
if (pkg.version !== version) {
  pkg.version = version;
  changed = writeJson("package.json", pkg) || changed;
}

// 2. tauri.conf.json
{
  const rel = "src-tauri/tauri.conf.json";
  const conf = readJson(rel);
  if (conf.version !== version) {
    conf.version = version;
    changed = writeJson(rel, conf) || changed;
  }
}

// 3. Cargo.toml（首个 version 行即 [package] 版本）
{
  const rel = "src-tauri/Cargo.toml";
  const next = read(rel).replace(/^version = "[^"]*"$/m, () => `version = "${version}"`);
  changed = writeIfChanged(rel, next) || changed;
}

// 4. Cargo.lock（仅改根包版本块，依赖不动）
{
  const cargo = read("src-tauri/Cargo.toml");
  const name = cargo.match(/^name = "([^"]+)"/m)?.[1];
  if (name) {
    const rel = "src-tauri/Cargo.lock";
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const next = read(rel).replace(
      new RegExp(`(\\[\\[package\\]\\]\\nname = "${escaped}"\\nversion = ")[^"]*(")`),
      (_match, head, tail) => `${head}${version}${tail}`,
    );
    changed = writeIfChanged(rel, next) || changed;
  }
}

console.log(changed ? `[version] 已更新为 ${version}` : `[version] ${version}（无需修改）`);
