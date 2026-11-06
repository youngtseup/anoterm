<template>
  <div
    v-for="handle in visibleHandles"
    :key="handle.direction"
    class="window-resize-handle"
    :class="handle.className"
    aria-hidden="true"
    @mousedown="startResize($event, handle.direction)"
  />
</template>

<script setup lang="ts">
/**
 * 无边框窗口的边缘缩放热区（参照 openidea 的处理方式）：
 * - 在窗口四边和四角放置透明热区，按下后调用原生 startResizeDragging，
 *   由系统负责最小尺寸、贴边吸附等行为
 * - 窗口最大化 / 全屏时不渲染，边缘交还给内容
 * - macOS 使用系统原生标题栏，已有缩放边缘，无需添加
 */
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

type ResizeDirection =
  "East" | "North" | "NorthEast" | "NorthWest" | "South" | "SouthEast" | "SouthWest" | "West";

const EDGES: { direction: ResizeDirection; className: string }[] = [
  { direction: "North", className: "n" },
  { direction: "South", className: "s" },
  { direction: "East", className: "e" },
  { direction: "West", className: "w" },
  { direction: "NorthEast", className: "ne" },
  { direction: "NorthWest", className: "nw" },
  { direction: "SouthEast", className: "se" },
  { direction: "SouthWest", className: "sw" },
];

/** macOS 使用系统原生窗口控制，无需自定义缩放热区 */
const isMac = navigator.userAgent.toLowerCase().includes("mac");

const enabled = ref(false);
/** 最大化或全屏时没有可缩放的边缘（初始化时先视为不可缩放，等状态同步后再显示） */
const sized = ref(true);

let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
let unlisten: (() => void) | null = null;

const visibleHandles = computed(() => (enabled.value && !sized.value ? EDGES : []));

async function syncSized() {
  if (!appWindow) return;
  try {
    const [maximized, fullscreen] = await Promise.all([
      appWindow.isMaximized(),
      appWindow.isFullscreen(),
    ]);
    sized.value = maximized || fullscreen;
  } catch {
    // 状态读取失败时按普通窗口处理，至少保留缩放能力
    sized.value = false;
  }
}

onMounted(async () => {
  if (isMac || !("__TAURI_INTERNALS__" in window)) return;
  appWindow = getCurrentWindow();
  enabled.value = true;
  await syncSized();
  unlisten = await appWindow.onResized(syncSized);
});

onBeforeUnmount(() => {
  unlisten?.();
});

function startResize(event: MouseEvent, direction: ResizeDirection) {
  if (event.button !== 0) return;
  event.preventDefault();
  appWindow?.startResizeDragging(direction).catch(() => {});
}
</script>

<style scoped>
.window-resize-handle {
  position: fixed;
  z-index: 9999;
  pointer-events: auto;
}

.n,
.s {
  left: 8px;
  right: 8px;
  height: 6px;
}

.e,
.w {
  top: 8px;
  bottom: 8px;
  width: 6px;
}

.ne,
.nw,
.se,
.sw {
  width: 12px;
  height: 12px;
}

.n {
  top: 0;
  cursor: ns-resize;
}

.s {
  bottom: 0;
  cursor: ns-resize;
}

.e {
  right: 0;
  cursor: ew-resize;
}

.w {
  left: 0;
  cursor: ew-resize;
}

.ne {
  top: 0;
  right: 0;
  cursor: nesw-resize;
}

.nw {
  top: 0;
  left: 0;
  cursor: nwse-resize;
}

.se {
  bottom: 0;
  right: 0;
  cursor: nwse-resize;
}

.sw {
  bottom: 0;
  left: 0;
  cursor: nesw-resize;
}
</style>
