<template>
  <div class="terminal-view">
    <div class="terminal-body" ref="containerRef"></div>
    <div class="terminal-indicators" aria-hidden="true">
      <span class="terminal-indicator" :class="{ active: hasSelection }"></span>
      <span class="terminal-indicator" :class="{ active: hasClipboardText }"></span>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * SSH 终端视图（xterm.js）：
 * - 会话由 ConnectionView 建立，本组件只负责渲染与交互
 * - 监听 ssh-output / ssh-closed 事件（以 tabId 区分归属）
 * - 复制粘贴组合键按平台区分：macOS 用 Cmd+C/V，Windows/Linux 用 Ctrl+C/V
 * - 复制组合键仅在有选区时生效，否则 Ctrl+C 作为中断信号交给远端
 */
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { Terminal, type IDisposable } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { toast } from "vue-sonner";
import { TERMINAL_MIN_CONTRAST, buildTerminalTheme, useTheme } from "../composables/useTheme";

const props = defineProps<{ tabId: string; active?: boolean }>();
const emit = defineEmits<{ close: [tabId: string] }>();

const containerRef = ref<HTMLElement | null>(null);
const { isDark } = useTheme();

/**
 * macOS 的剪贴板组合键是 Cmd，Ctrl+C 必须原样发给远端作 SIGINT；
 * Windows/Linux 沿用 Ctrl+C（有选区时复制）与 Ctrl+V。
 */
const isMac = navigator.userAgent.toLowerCase().includes("mac");

const term = new Terminal({
  cursorBlink: true,
  fontSize: 14,
  fontFamily: '"Cascadia Mono", Consolas, Menlo, monospace',
  minimumContrastRatio: TERMINAL_MIN_CONTRAST,
  theme: buildTerminalTheme(),
});
const fit = new FitAddon();

let unlistenOutput: UnlistenFn | null = null;
let unlistenClosed: UnlistenFn | null = null;
let disposed = false;
let ro: ResizeObserver | null = null;

const hasSelection = ref(false);
const hasClipboardText = ref(false);
let selectionDisposable: IDisposable | null = null;
let clipboardTimer: number | null = null;
let clipboardReadInFlight = false;

/** 后端 base64 输出解码为字节 */
function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

/** 终端输入 → 后端 */
function sendData(data: string) {
  if (disposed) return;
  invoke("ssh_send", { connId: props.tabId, data }).catch(() => {});
}

/** 窗口尺寸变化 → 后端（远端 PTY 同步） */
function sendResize(cols: number, rows: number) {
  if (disposed) return;
  invoke("ssh_resize", { connId: props.tabId, cols, rows }).catch(() => {});
}

/** 自适应尺寸并通知远端 */
function fitAndResize() {
  try {
    fit.fit();
    sendResize(term.cols, term.rows);
  } catch {
    /* ignore */
  }
}

/** 复制选中文本到剪贴板 */
function copySelection() {
  const text = term.getSelection();
  if (!text) return;
  navigator.clipboard
    ?.writeText(text)
    .then(() => {
      hasClipboardText.value = true;
    })
    .catch(() => {});
}

/** 从剪贴板粘贴到终端 */
async function pasteClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    if (text) {
      hasClipboardText.value = true;
      term.paste(text);
    }
  } catch {
    /* ignore */
  }
}

function syncSelectionState() {
  hasSelection.value = term.hasSelection();
}

async function readClipboard(): Promise<string> {
  return (await navigator.clipboard?.readText()) ?? "";
}

/** 刷新“剪贴板有文本”状态；读取失败时保留上一次结果 */
async function refreshClipboardState() {
  if (clipboardReadInFlight) return;
  clipboardReadInFlight = true;
  try {
    const text = await readClipboard();
    hasClipboardText.value = text.length > 0;
  } catch {
    /* ignore */
  } finally {
    clipboardReadInFlight = false;
  }
}

/** 只有当前激活的终端需要监听剪贴板变化 */
function startClipboardWatch() {
  stopClipboardWatch();
  if (!props.active) return;
  void refreshClipboardState();
  clipboardTimer = window.setInterval(() => {
    if (document.hasFocus()) void refreshClipboardState();
  }, 1500);
}

function stopClipboardWatch() {
  if (clipboardTimer !== null) {
    window.clearInterval(clipboardTimer);
    clipboardTimer = null;
  }
}

function handleWindowFocus() {
  if (props.active) void refreshClipboardState();
}

/**
 * 自定义按键处理。
 * 注意 xterm 语义：返回 false 阻止 xterm 继续处理，返回 true 放行。
 * keydown/keypress 都会触发本回调，需只在 keydown 处理，避免双击触发。
 */
function handleKey(event: KeyboardEvent): boolean {
  if (event.type !== "keydown") return true;
  const key = event.key.toLowerCase();
  const clipboardCombo =
    !event.altKey && (isMac ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey);

  if (!clipboardCombo) return true;

  // 复制组合键：有选中文本时复制，否则作为中断信号交给远端
  if (key === "c") {
    if (term.hasSelection()) {
      copySelection();
      return false;
    }
    return true;
  }

  // 粘贴组合键：粘贴剪贴板（不走终端输入）
  if (key === "v") {
    event.preventDefault();
    pasteClipboard();
    return false;
  }

  return true;
}

onMounted(async () => {
  term.loadAddon(fit);
  term.open(containerRef.value!);
  fit.fit();

  term.onData((data) => sendData(data));
  term.onResize(({ cols, rows }) => sendResize(cols, rows));
  term.attachCustomKeyEventHandler(handleKey);
  selectionDisposable = term.onSelectionChange(syncSelectionState);
  window.addEventListener("focus", handleWindowFocus);
  if (props.active) startClipboardWatch();

  // 监听本 tab 的终端输出（会话由 ConnectionView 先行建立）
  unlistenOutput = await listen<{ conn_id: string; data: string }>("ssh-output", (e) => {
    if (disposed) return;
    if (e.payload.conn_id !== props.tabId) return;
    term.write(b64ToBytes(e.payload.data));
  });

  // 远端断开时提示并关闭 tab
  unlistenClosed = await listen<{ conn_id: string }>("ssh-closed", (e) => {
    if (disposed) return;
    if (e.payload.conn_id !== props.tabId) return;
    toast.warning("连接已断开");
    disposed = true;
    emit("close", props.tabId);
  });

  // 容器尺寸变化（窗口缩放/侧边栏收起）时自适应
  ro = new ResizeObserver(() => fitAndResize());
  ro.observe(containerRef.value!);

  if (props.active && !disposed) term.focus();
});

onBeforeUnmount(() => {
  disposed = true;
  stopClipboardWatch();
  window.removeEventListener("focus", handleWindowFocus);
  selectionDisposable?.dispose();
  selectionDisposable = null;
  unlistenOutput?.();
  unlistenClosed?.();
  ro?.disconnect();
  term.dispose();
});

watch(
  () => props.active,
  (active) => {
    if (active) {
      requestAnimationFrame(() => fitAndResize());
      syncSelectionState();
      startClipboardWatch();
    } else {
      stopClipboardWatch();
    }
  },
);

// 主题切换：xterm 对 theme 做引用比较，必须整体赋一个新对象才会生效
watch(isDark, () => {
  term.options.theme = buildTerminalTheme();
});
</script>

<style scoped>
/* 角色网格之外的留白也跟随主题，避免白色终端外围残留深色边框 */
.terminal-view {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--color-background);
}

.terminal-indicators {
  position: absolute;
  right: 16px;
  bottom: 10px;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 6px;
  pointer-events: none;
}

.terminal-indicator {
  width: 8px;
  height: 8px;
  border: 1px solid var(--color-muted-foreground);
  border-radius: 9999px;
  background: transparent;
  transition:
    background-color 0.15s ease,
    border-color 0.15s ease;
}

.terminal-indicator.active {
  border-color: var(--color-foreground);
  background: var(--color-foreground);
}

.terminal-body {
  flex: 1;
  min-height: 0;
  padding: 6px 0 6px 8px;
  overflow: hidden;
}

.terminal-body :deep(.xterm) {
  height: 100%;
}

/* xterm 6 的 viewport 仍是默认黑底，且比滚动容器高出几个像素，浅色主题下会露出底部黑线 */
.terminal-body :deep(.xterm .xterm-viewport) {
  background-color: var(--color-background);
}
</style>
