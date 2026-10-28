/**
 * 本地文件系统操作的 Tauri 命令封装（SFTP 双栏模式的本地侧）。
 */

import { invoke } from "@tauri-apps/api/core";
import type { SftpEntry } from "../types";

/** 用户主目录（本地栏初始路径） */
export async function localHome(): Promise<string> {
  return await invoke<string>("local_home");
}

/** 列出本地目录内容（目录排前，按名称排序） */
export async function localList(path: string): Promise<SftpEntry[]> {
  return await invoke<SftpEntry[]>("local_list", { path });
}

/** 创建本地目录 */
export async function localMkdir(path: string): Promise<void> {
  await invoke("local_mkdir", { path });
}

/** 重命名/移动本地文件或目录 */
export async function localRename(oldPath: string, newPath: string): Promise<void> {
  await invoke("local_rename", { oldPath, newPath });
}

/** 删除本地文件或目录（目录递归删除） */
export async function localRemove(path: string, isDir: boolean): Promise<void> {
  await invoke("local_remove", { path, isDir });
}
