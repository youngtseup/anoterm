/**
 * SFTP 传输进度、同名冲突与取消：
 * - 启动批量上传/下载，监听后端进度与冲突事件（按 transferId 过滤）
 * - 同一实例同时只保留一个传输会话
 */

import { computed, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  sftpCancelTransfer,
  sftpDownloadItems,
  sftpResolveConflict,
  sftpUploadItems,
  type ConflictChoice,
  type TransferConflict,
  type TransferItem,
  type TransferOutcome,
} from "../api";

/** 进度事件负载（字段与后端 camelCase 对齐） */
interface ProgressPayload {
  transferId: string;
  current: string;
  transferred: number;
  total: number;
}

export interface TransferState {
  transferId: string;
  current: string;
  transferred: number;
  total: number;
}

export interface StartOptions {
  /** 跳过冲突弹窗，直接使用指定策略（如系统保存框已确认覆盖） */
  conflictPolicy?: ConflictChoice;
}

let seq = 0;

export function useSftpTransfer() {
  const state = ref<TransferState | null>(null);
  const cancelling = ref(false);
  const running = computed(() => state.value !== null);
  /** 当前等待用户处理的同名冲突 */
  const conflict = ref<TransferConflict | null>(null);
  let unlisten: UnlistenFn | null = null;
  let unlistenConflict: UnlistenFn | null = null;

  /** 启动传输；返回 done / cancelled，异常交给调用方处理 */
  async function start(
    tabId: string,
    direction: "upload" | "download",
    items: TransferItem[],
    options?: StartOptions,
  ): Promise<TransferOutcome> {
    const transferId = `xfer-${Date.now().toString(36)}-${++seq}`;
    state.value = { transferId, current: "", transferred: 0, total: 0 };
    cancelling.value = false;
    conflict.value = null;
    try {
      unlisten = await listen<ProgressPayload>("sftp-transfer-progress", (ev) => {
        if (ev.payload.transferId !== transferId || !state.value) return;
        state.value.current = ev.payload.current;
        state.value.transferred = ev.payload.transferred;
        state.value.total = ev.payload.total;
      });
      unlistenConflict = await listen<TransferConflict>("sftp-transfer-conflict", (ev) => {
        if (ev.payload.transferId !== transferId) return;
        conflict.value = ev.payload;
      });
      return direction === "upload"
        ? await sftpUploadItems(transferId, tabId, items, options?.conflictPolicy)
        : await sftpDownloadItems(transferId, tabId, items, options?.conflictPolicy);
    } finally {
      unlisten?.();
      unlisten = null;
      unlistenConflict?.();
      unlistenConflict = null;
      state.value = null;
      cancelling.value = false;
      conflict.value = null;
    }
  }

  /** 请求取消当前传输（后端在分块边界退出并清理半成品） */
  async function cancel() {
    const current = state.value;
    if (!current || cancelling.value) return;
    cancelling.value = true;
    try {
      await sftpCancelTransfer(current.transferId);
    } catch {
      cancelling.value = false;
    }
  }

  /** 回复当前同名冲突；applyAll 表示对本次传输后续冲突应用同一策略 */
  async function resolveConflict(choice: ConflictChoice, applyAll: boolean) {
    const current = conflict.value;
    if (!current) return;
    conflict.value = null;
    try {
      await sftpResolveConflict(current.transferId, choice, applyAll);
    } catch {
      /* ignore */
    }
  }

  return { state, cancelling, running, conflict, start, cancel, resolveConflict };
}
