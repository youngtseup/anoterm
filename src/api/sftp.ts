/**
 * SFTP 文件操作的 Tauri 命令封装。
 *
 * 所有命令以会话 id（前端 tabId）定位后端会话，
 * 远端路径均为服务器上的绝对路径。
 */

import { invoke } from "@tauri-apps/api/core";
import type { SftpEntry } from "../types";

/** 列出远端目录内容（目录排前，按名称排序） */
export async function sftpList(connId: string, path: string): Promise<SftpEntry[]> {
  return await invoke<SftpEntry[]>("sftp_list", { connId, path });
}

/** 创建远端目录 */
export async function sftpMkdir(connId: string, path: string): Promise<void> {
  await invoke("sftp_mkdir", { connId, path });
}

/** 重命名/移动远端文件或目录 */
export async function sftpRename(connId: string, oldPath: string, newPath: string): Promise<void> {
  await invoke("sftp_rename", { connId, oldPath, newPath });
}

/** 删除远端文件或目录（按 isDir 区分） */
export async function sftpRemove(connId: string, path: string, isDir: boolean): Promise<void> {
  await invoke("sftp_remove", { connId, path, isDir });
}

/** 传输项：目录会递归处理；dest 为最终目标路径（含名称） */
export interface TransferItem {
  source: string;
  dest: string;
  isDir: boolean;
}

/** 传输结果：取消属于正常结束，不算错误 */
export type TransferOutcome = "done" | "cancelled";

/** 同名冲突处理策略 */
export type ConflictChoice = "overwrite" | "skip" | "keepBoth" | "cancel";

/** 传输冲突事件负载（字段与后端 camelCase 对齐） */
export interface TransferConflict {
  transferId: string;
  direction: "upload" | "download";
  source: string;
  dest: string;
  name: string;
  isDir: boolean;
  existingIsDir: boolean;
}

/** 批量上传：本地 → 远端（后端发进度事件，可取消） */
export async function sftpUploadItems(
  transferId: string,
  connId: string,
  items: TransferItem[],
  conflictPolicy?: ConflictChoice,
): Promise<TransferOutcome> {
  return await invoke<TransferOutcome>("sftp_upload_items", {
    transferId,
    connId,
    items,
    conflictPolicy: conflictPolicy ?? null,
  });
}

/** 批量下载：远端 → 本地（后端发进度事件，可取消） */
export async function sftpDownloadItems(
  transferId: string,
  connId: string,
  items: TransferItem[],
  conflictPolicy?: ConflictChoice,
): Promise<TransferOutcome> {
  return await invoke<TransferOutcome>("sftp_download_items", {
    transferId,
    connId,
    items,
    conflictPolicy: conflictPolicy ?? null,
  });
}

/** 请求取消指定传输 */
export async function sftpCancelTransfer(transferId: string): Promise<void> {
  await invoke("sftp_cancel_transfer", { transferId });
}

/** 回复同名冲突处理选择 */
export async function sftpResolveConflict(
  transferId: string,
  choice: ConflictChoice,
  applyAll: boolean,
): Promise<void> {
  await invoke("sftp_resolve_conflict", { transferId, choice, applyAll });
}
