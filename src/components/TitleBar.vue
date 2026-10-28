<template>
  <header
    class="z-100 flex h-10 shrink-0 items-center justify-between border-b border-border bg-background pl-4 shadow-sm select-none"
    @mousedown="handleDrag"
  >
    <div class="flex items-center gap-2.5">
      <button
        type="button"
        class="-ml-1.5 flex cursor-pointer items-center gap-2.5 rounded-md px-1.5 py-1 transition-colors hover:bg-muted/60"
        title="返回首页"
        @click="emit('home')"
      >
        <span class="flex h-7 w-7 items-center justify-center overflow-hidden">
          <img :src="appIcon" alt="Anoterm" class="block h-6 w-6 rounded-md object-cover" />
        </span>
        <span class="text-sm font-semibold">Anoterm</span>
      </button>
    </div>

    <div class="flex-1 self-stretch"></div>

    <div class="flex items-center self-stretch">
      <TooltipProvider :delay-duration="0">
        <div class="mr-2 flex items-center gap-1.5">
          <Tooltip>
            <TooltipTrigger as-child>
              <Button variant="ghost" size="icon" class="size-8" @click="openGithub">
                <svg class="size-4" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                  <path
                    d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"
                  />
                </svg>
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">GitHub</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger as-child>
              <Button
                variant="ghost"
                size="icon"
                class="size-8"
                :disabled="checkingUpdate"
                @click="emit('checkUpdate')"
              >
                <Loader2 v-if="checkingUpdate" class="size-4 animate-spin" />
                <RefreshCw v-else class="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">检查更新</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger as-child>
              <Button variant="ghost" size="icon" class="size-8" @click="emit('toggleTheme')">
                <Sun v-if="isDark" class="size-4" />
                <Moon v-else class="size-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">{{ isDark ? "切换亮色" : "切换暗色" }}</TooltipContent>
          </Tooltip>
        </div>

        <div class="flex items-center self-stretch">
          <button class="window-control" title="最小化" @click="minimize">
            <Minus class="size-4" />
          </button>
          <button
            class="window-control"
            :title="isMaximized ? '还原' : '最大化'"
            @click="toggleMaximize"
          >
            <Copy v-if="isMaximized" class="size-4" />
            <Maximize v-else class="size-4" />
          </button>
          <button class="window-control close" title="关闭" @click="close">
            <X class="size-4" />
          </button>
        </div>
      </TooltipProvider>
    </div>
  </header>
</template>

<script setup lang="ts">
/**
 * 自定义标题栏（窗口无系统装饰）：
 * - 左侧：Logo、标题
 * - 右侧：GitHub、检查更新、主题切换、窗口控制（最小化/最大化/关闭）
 * - 空白区域按下可拖动窗口（button 等选择器内的控件除外），双击最大化
 */
import { onMounted, ref } from "vue";
import { Copy, Loader2, Maximize, Minus, Moon, RefreshCw, Sun, X } from "@lucide/vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Button } from "./ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import appIcon from "../assets/app-icon.png";

defineProps<{ isDark: boolean; checkingUpdate?: boolean }>();

const emit = defineEmits<{
  home: [];
  toggleTheme: [];
  checkUpdate: [];
}>();

const GITHUB_URL = "https://github.com/youngtseup/anoterm";

function openGithub() {
  if ("__TAURI_INTERNALS__" in window) {
    openUrl(GITHUB_URL).catch(() => window.open(GITHUB_URL, "_blank"));
  } else {
    window.open(GITHUB_URL, "_blank");
  }
}

const isMaximized = ref(false);
let appWindow: ReturnType<typeof getCurrentWindow> | null = null;

onMounted(async () => {
  if (!("__TAURI_INTERNALS__" in window)) return;
  appWindow = getCurrentWindow();
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    /* ignore */
  }
});

function minimize() {
  appWindow?.minimize();
}

async function toggleMaximize() {
  try {
    await appWindow?.toggleMaximize();
    isMaximized.value = (await appWindow?.isMaximized()) ?? false;
    if (!isMaximized.value) {
      await new Promise((r) => setTimeout(r, 120));
      await appWindow?.center();
    }
  } catch {
    /* ignore */
  }
}

function close() {
  appWindow?.close();
}

const NO_DRAG_SELECTOR =
  "button, input, textarea, select, a, [contenteditable='true'], [data-no-drag]";

function handleDrag(e: MouseEvent) {
  if (e.buttons !== 1) return;
  const target = e.target as HTMLElement;
  if (target.closest(NO_DRAG_SELECTOR)) return;
  if (e.detail === 2) {
    toggleMaximize();
    return;
  }
  appWindow?.startDragging();
}
</script>

<style scoped>
.window-control {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--color-muted-foreground);
  cursor: pointer;
  transition:
    background 0.15s,
    color 0.15s;
}

.window-control:hover {
  background: rgba(0, 0, 0, 0.08);
  color: var(--color-foreground);
}

.dark .window-control:hover {
  background: rgba(255, 255, 255, 0.1);
}

.window-control.close:hover {
  background: #e81123;
  color: #ffffff;
}
</style>
