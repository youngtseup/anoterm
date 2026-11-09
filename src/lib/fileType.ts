/**
 * 文件面板「类型」归类：目录与无扩展名文件使用固定标签，其余按扩展名大写展示。
 */

import type { SftpEntry } from "../types";

/** 目录的类型标签 */
export const DIR_TYPE = "文件夹";
/** 无扩展名文件的类型标签 */
export const FILE_TYPE = "文件";

/** 取条目的类型标签（按扩展名，统一大写） */
export function entryType(entry: Pick<SftpEntry, "name" | "isDir">): string {
  if (entry.isDir) return DIR_TYPE;
  const name = entry.name;
  const dot = name.lastIndexOf(".");
  // 点开头（如 .gitignore）或结尾无内容时不视为扩展名
  if (dot <= 0 || dot === name.length - 1) return FILE_TYPE;
  return name.slice(dot + 1).toUpperCase();
}
