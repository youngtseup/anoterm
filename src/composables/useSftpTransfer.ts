/**
 * SFTP 传输进度与取消：
 * - 启动批量上传/下载，监听后端进度事件（按 transferId 过滤）
 * - 同一实例同时只保留一个传输会话
 */

import { computed, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  sftpCancelTransfer,
  sftpDownloadItems,
  sftpUploadItems,
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

let seq = 0;

export function useSftpTransfer() {
  const state = ref<TransferState | null>(null);
  const cancelling = ref(false);
  const running = computed(() => state.value !== null);
  let unlisten: UnlistenFn | null = null;

  /** 启动传输；返回 done / cancelled，异常交给调用方处理 */
  async function start(
    tabId: string,
    direction: "upload" | "download",
    items: TransferItem[],
  ): Promise<TransferOutcome> {
    const transferId = `xfer-${Date.now().toString(36)}-${++seq}`;
    state.value = { transferId, current: "", transferred: 0, total: 0 };
    cancelling.value = false;
    try {
      unlisten = await listen<ProgressPayload>("sftp-transfer-progress", (ev) => {
        if (ev.payload.transferId !== transferId || !state.value) return;
        state.value.current = ev.payload.current;
        state.value.transferred = ev.payload.transferred;
        state.value.total = ev.payload.total;
      });
      return direction === "upload"
        ? await sftpUploadItems(transferId, tabId, items)
        : await sftpDownloadItems(transferId, tabId, items);
    } finally {
      unlisten?.();
      unlisten = null;
      state.value = null;
      cancelling.value = false;
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

  return { state, cancelling, running, start, cancel };
}
