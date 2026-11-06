<template>
  <!-- 更新对话框（安装中不可关闭，下载中可隐藏到后台） -->
  <Dialog v-model:open="open">
    <DialogContent
      class="sm:max-w-md"
      :show-close-button="updatePhase !== 'installing'"
      @escape-key-down="updatePhase === 'installing' && $event.preventDefault()"
    >
      <DialogHeader>
        <DialogTitle>发现新版本 v{{ updateInfo?.version }}</DialogTitle>
        <div class="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
          <span>
            当前版本
            <span class="font-mono text-foreground">v{{ updateInfo?.currentVersion }}</span>
          </span>
          <ArrowRight class="size-4 shrink-0" />
          <span>
            最新版本
            <span class="font-mono text-primary">v{{ updateInfo?.version }}</span>
          </span>
        </div>
        <DialogDescription
          class="max-h-48 overflow-y-auto whitespace-pre-wrap rounded-md border border-border bg-muted/40 p-3 leading-relaxed"
        >
          {{ updateNotes }}
        </DialogDescription>
      </DialogHeader>

      <div v-if="updatePhase !== 'confirm'" class="space-y-2">
        <div class="flex items-center justify-between gap-3 text-sm">
          <span class="flex items-center gap-2 text-muted-foreground">
            <Loader2 v-if="updatePhase !== 'downloaded'" class="size-4 animate-spin" />
            <CircleCheck v-else class="size-4 text-primary" />
            {{ updateStatusLabel }}
          </span>
          <span class="shrink-0 font-medium tabular-nums">{{ updateProgressText }}</span>
        </div>
        <div
          class="h-2 overflow-hidden rounded-full bg-muted"
          role="progressbar"
          aria-label="更新下载进度"
          aria-valuemin="0"
          aria-valuemax="100"
          :aria-valuenow="
            updateTotal > 0 || updatePhase !== 'downloading' ? updateProgress : undefined
          "
        >
          <div
            v-if="updateTotal > 0 || updatePhase !== 'downloading'"
            class="h-full rounded-full bg-primary transition-[width] duration-150"
            :style="{ width: `${updateProgress}%` }"
          />
          <div v-else class="h-full w-1/3 animate-pulse rounded-full bg-primary/60" />
        </div>
        <div class="flex justify-between gap-3 text-xs text-muted-foreground tabular-nums">
          <span v-if="updateTotal > 0">
            {{ formatBytes(updateDownloaded) }} / {{ formatBytes(updateTotal) }}
          </span>
          <span v-else-if="updatePhase === 'downloading'">
            已下载 {{ formatBytes(updateDownloaded) }}
          </span>
          <span v-if="updatePhase === 'downloaded'">更新包已就绪</span>
        </div>
      </div>

      <DialogFooter v-if="updatePhase !== 'installing'">
        <template v-if="updatePhase === 'confirm'">
          <Button variant="outline" @click="open = false">暂不更新</Button>
          <Button :disabled="checkingUpdate" @click="downloadUpdate">下载更新</Button>
        </template>
        <template v-else-if="updatePhase === 'downloading'">
          <Button variant="outline" @click="cancelDownload">取消</Button>
          <Button variant="outline" @click="open = false">隐藏并后台下载</Button>
        </template>
        <template v-else>
          <Button variant="outline" @click="open = false">稍后安装</Button>
          <Button @click="installUpdate">立即安装</Button>
        </template>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
/**
 * 应用更新弹窗：
 * - 检查更新、后台下载、下载完成后手动安装
 * - 通过 status 事件向标题栏同步检查/下载状态
 */
import { computed, markRaw, ref, watchEffect } from "vue";
import { toast } from "vue-sonner";
import { ArrowRight, CircleCheck, Loader2 } from "@lucide/vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { notifyError } from "../api";
import { formatBytes } from "../lib/utils";
import type { UpdateStatus, UpdateStatusPayload } from "../types";

type UpdatePhase = "confirm" | "downloading" | "downloaded" | "installing";

const emit = defineEmits<{ status: [payload: UpdateStatusPayload] }>();

const open = ref(false);
const checkingUpdate = ref(false);
const updatePhase = ref<UpdatePhase>("confirm");
const updateProgress = ref(0);
const updateTotal = ref(0);
const updateDownloaded = ref(0);
/** 下载代次：取消后自增，使旧下载回调失效 */
let downloadGeneration = 0;
/** Update 实例使用 WeakMap 私有字段，不能被 Vue 响应式代理包裹（否则调用 update 方法时报错） */
const updateInfo = ref<{
  version: string;
  currentVersion: string;
  body?: string;
  update: Update;
} | null>(null);

/** 标题栏更新按钮状态：仅在有更新任务时显示下载/就绪标记 */
const updateStatus = computed<UpdateStatus>(() => {
  if (!updateInfo.value || updatePhase.value === "confirm") return "idle";
  return updatePhase.value;
});

const updateProgressText = computed(() =>
  updateTotal.value > 0 || updatePhase.value !== "downloading"
    ? `${updateProgress.value}%`
    : "下载中",
);

const updateStatusLabel = computed(() => {
  switch (updatePhase.value) {
    case "downloading":
      return "正在下载更新…";
    case "downloaded":
      return "更新包已下载完成";
    case "installing":
      return "正在安装更新，请稍候…";
    default:
      return "";
  }
});

/** 过滤仅包含版本号的 notes，避免与上方的版本对比行重复 */
const updateNotes = computed(() => {
  const info = updateInfo.value;
  const body = info?.body?.trim();
  if (!info || !body) return "暂无";

  const version = info.version.replace(/^v/i, "").toLowerCase();
  const compact = body.replace(/\s+/g, " ").toLowerCase();
  const versionOnly = new Set([
    version,
    `v${version}`,
    `anoterm ${version}`,
    `anoterm v${version}`,
    `anoterm ${version} 更新`,
    `anoterm v${version} 更新`,
  ]);
  return versionOnly.has(compact) ? "暂无" : body;
});

watchEffect(() => {
  emit("status", {
    checking: checkingUpdate.value,
    status: updateStatus.value,
    progress:
      updatePhase.value === "downloading" && updateTotal.value > 0 ? updateProgress.value : null,
  });
});

/** 判断当前是否为 Windows（Windows 下安装完成后应用会自动退出运行安装器） */
function isWindowsPlatform() {
  return navigator.userAgent.toLowerCase().includes("win");
}

function resetUpdateProgress() {
  updateProgress.value = 0;
  updateTotal.value = 0;
  updateDownloaded.value = 0;
}

async function checkForUpdates() {
  if (checkingUpdate.value) return;
  // 已有下载/安装任务：直接重新打开对话框，避免重新检查丢失当前进度
  if (updateInfo.value && updatePhase.value !== "confirm") {
    open.value = true;
    return;
  }
  checkingUpdate.value = true;
  try {
    const update = await check();
    const previous = updateInfo.value;
    if (previous) {
      previous.update.close().catch(() => {});
    }
    if (!update) {
      updateInfo.value = null;
      updatePhase.value = "confirm";
      resetUpdateProgress();
      toast.success("当前已是最新版本");
      return;
    }
    updateInfo.value = {
      version: update.version,
      currentVersion: update.currentVersion,
      body: update.body ?? undefined,
      update: markRaw(update),
    };
    updatePhase.value = "confirm";
    resetUpdateProgress();
    open.value = true;
  } catch (e) {
    notifyError(e);
  } finally {
    checkingUpdate.value = false;
  }
}

/** 仅下载更新包，不自动安装；用户可关闭对话框继续后台下载 */
async function downloadUpdate() {
  const info = updateInfo.value;
  if (!info || updatePhase.value !== "confirm") return;
  const generation = ++downloadGeneration;
  updatePhase.value = "downloading";
  resetUpdateProgress();
  try {
    await info.update.download((event) => {
      if (generation !== downloadGeneration) return;
      if (event.event === "Started") {
        updateTotal.value = event.data.contentLength ?? 0;
      } else if (event.event === "Progress") {
        updateDownloaded.value += event.data.chunkLength;
        if (updateTotal.value > 0) {
          updateProgress.value = Math.min(
            99,
            Math.floor((updateDownloaded.value / updateTotal.value) * 100),
          );
        }
      }
    });
    if (generation !== downloadGeneration) {
      // 已取消：清理下载完成后才产生的缓存资源
      info.update.close().catch(() => {});
      return;
    }
    updateProgress.value = 100;
    updatePhase.value = "downloaded";
    toast.success("更新包已下载完成，点击标题栏更新按钮即可安装");
  } catch (e) {
    if (generation !== downloadGeneration) {
      info.update.close().catch(() => {});
      return;
    }
    updateInfo.value = null;
    updatePhase.value = "confirm";
    resetUpdateProgress();
    open.value = false;
    info.update.close().catch(() => {});
    notifyError(e);
  }
}

/** 取消下载：使当前下载回调失效并释放 Update 资源 */
function cancelDownload() {
  const info = updateInfo.value;
  if (!info || updatePhase.value !== "downloading") return;
  downloadGeneration += 1;
  updateInfo.value = null;
  updatePhase.value = "confirm";
  resetUpdateProgress();
  open.value = false;
  info.update.close().catch(() => {});
  toast.info("已取消更新下载");
}

/** 用户手动确认后才执行安装 */
async function installUpdate() {
  const info = updateInfo.value;
  if (!info || updatePhase.value !== "downloaded") return;
  updatePhase.value = "installing";
  try {
    await info.update.install();
    // Windows 下安装器启动后应用会自动退出；macOS/Linux 需手动重启
    if (!isWindowsPlatform()) {
      toast.success("更新已安装，正在重启应用…");
      await relaunch();
    }
    open.value = false;
  } catch (e) {
    updatePhase.value = "downloaded";
    notifyError(e);
  }
}

defineExpose({ checkForUpdates });
</script>
