#!/usr/bin/env node
/**
 * 将 Release 正文中的「变更日志 / 变更说明」同步到 latest.json 的 notes 字段。
 *
 * 用法:
 *   node scripts/sync-latest-notes.mjs --body release-body.md --file latest.json \
 *     [--fallback "Anoterm v0.2.0"]
 *
 * 说明:
 * - 匹配 "## 变更日志" / "## 变更说明" / "## 更新内容" / "## Changelog" 标题
 * - 取该标题到下一个二级标题之间的内容作为 notes
 * - 未找到对应章节时使用 --fallback
 */
import { readFileSync, writeFileSync } from "node:fs";

function parseArgs(argv) {
  const args = {};
  for (let i = 2; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, "");
    args[key] = argv[i + 1];
  }
  return args;
}

const args = parseArgs(process.argv);
const bodyFile = args.body;
const manifestFile = args.file ?? "latest.json";

if (!bodyFile) {
  console.error("缺少必填参数: --body <release-body.md>");
  process.exit(1);
}

const body = readFileSync(bodyFile, "utf-8").replace(/\r\n?/g, "\n");
const headingRe = /^##[^\n]*(?:变更日志|变更说明|更新内容|changelog)[^\n]*$/im;
const nextHeadingRe = /^##(?:\s|$)/m;

let notes = "";
const match = headingRe.exec(body);
if (match) {
  const rest = body.slice(match.index + match[0].length).replace(/^\n+/, "");
  const next = nextHeadingRe.exec(rest);
  notes = (next ? rest.slice(0, next.index) : rest).trim();
}

if (!notes) {
  notes = (args.fallback ?? "").trim();
  if (notes) console.warn("未找到变更日志章节，使用 fallback 内容");
}

if (!notes) {
  console.error("Release 正文中没有可用的更新内容");
  process.exit(1);
}

const manifest = JSON.parse(readFileSync(manifestFile, "utf-8"));
manifest.notes = notes;
writeFileSync(manifestFile, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`已更新 ${manifestFile} 的 notes（${notes.split("\n").length} 行）`);
