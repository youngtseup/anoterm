<template>
  <div class="flex min-h-0 flex-1 flex-col bg-muted/50">
    <TransferBar
      v-if="transferState"
      :current="transferState.current"
      :transferred="transferState.transferred"
      :total="transferState.total"
      :cancelling="transferCancelling"
      @cancel="cancelTransfer"
    />
    <div class="flex min-h-0 flex-1">
      <FilePane
        v-show="layout === 'dual'"
        ref="localPane"
        kind="local"
        :layout="layout"
        :ready="ready"
        :busy="transferRunning"
        class="w-1/2 min-w-0 border-r border-border"
        @transfer="uploadToRemote"
      />
      <FilePane
        ref="remotePane"
        kind="remote"
        :layout="layout"
        :tab-id="tabId"
        :ready="ready"
        :busy="transferRunning"
        class="min-w-0 flex-1"
        @transfer="downloadToLocal"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * SFTP 视图宿主：
 * - 双栏（默认）：左本地 / 右远端，负责两侧之间的传输编排、进度条与取消
 * - 单栏：仅远端浏览器（本地栏隐藏但保活，切换布局不丢路径与选择）
 */
import { useTemplateRef } from "vue";
import { toast } from "vue-sonner";
import type { SftpEntry } from "../types";
import { notifyError, type TransferItem } from "../api";
import { joinLocal, joinRemote } from "../lib/paths";
import { useSftpLayout } from "../composables/useSftpLayout";
import { useSftpTransfer } from "../composables/useSftpTransfer";
import FilePane from "./FilePane.vue";
import TransferBar from "./TransferBar.vue";

const props = defineProps<{ tabId: string; ready: boolean }>();

const { layout } = useSftpLayout();
const {
  state: transferState,
  cancelling: transferCancelling,
  running: transferRunning,
  start: startTransfer,
  cancel: cancelTransfer,
} = useSftpTransfer();
const localPane = useTemplateRef<InstanceType<typeof FilePane>>("localPane");
const remotePane = useTemplateRef<InstanceType<typeof FilePane>>("remotePane");

/** 双栏：本地选中项 → 远端当前目录 */
async function uploadToRemote(entries: SftpEntry[]) {
  const target = remotePane.value;
  if (entries.length === 0 || !target || !target.cwd || transferRunning.value) return;
  if (!props.ready) {
    toast.warning("SFTP 会话尚未就绪");
    return;
  }
  const remoteCwd = target.cwd;
  const items: TransferItem[] = entries.map((entry) => ({
    source: entry.path,
    dest: joinRemote(remoteCwd, entry.name),
    isDir: entry.isDir,
  }));
  try {
    const outcome = await startTransfer(props.tabId, "upload", items);
    if (outcome === "done") {
      toast.success(entries.length > 1 ? `已上传 ${entries.length} 项` : "上传完成");
    } else {
      toast.info("已取消上传");
    }
    await target.refresh();
  } catch (e) {
    notifyError(e);
  }
}

/** 双栏：远端选中项 → 本地当前目录 */
async function downloadToLocal(entries: SftpEntry[]) {
  const target = localPane.value;
  if (entries.length === 0 || !target || !target.cwd || transferRunning.value) return;
  if (!props.ready) {
    toast.warning("SFTP 会话尚未就绪");
    return;
  }
  const localCwd = target.cwd;
  const items: TransferItem[] = entries.map((entry) => ({
    source: entry.path,
    dest: joinLocal(localCwd, entry.name),
    isDir: entry.isDir,
  }));
  try {
    const outcome = await startTransfer(props.tabId, "download", items);
    if (outcome === "done") {
      toast.success(entries.length > 1 ? `已下载 ${entries.length} 项` : "下载完成");
    } else {
      toast.info("已取消下载");
    }
    await target.refresh();
  } catch (e) {
    notifyError(e);
  }
}
</script>
