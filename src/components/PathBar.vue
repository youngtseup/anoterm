<template>
  <div
    ref="barEl"
    class="relative flex h-8 items-center rounded-md border bg-transparent shadow-xs transition-[color,box-shadow]"
    :class="editing ? 'border-ring ring-3 ring-ring/50' : 'border-input hover:border-ring/50'"
  >
    <HardDrive
      v-if="kind === 'local'"
      class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
    />
    <Folder
      v-else
      class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
    />

    <template v-if="editing">
      <Input
        v-model="draft"
        class="h-8 border-0 bg-transparent pr-8 pl-8 text-sm shadow-none focus-visible:ring-0"
        :placeholder="placeholder"
        @keyup.enter="submit"
        @keydown.esc="cancelEdit"
        @blur="cancelEdit"
      />
      <button
        v-if="draft"
        type="button"
        class="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
        title="清空"
        aria-label="清空"
        @mousedown.prevent
        @click="draft = ''"
      >
        <X class="size-4" />
      </button>
    </template>

    <template v-else>
      <div
        ref="crumbsEl"
        class="flex h-full min-w-0 flex-1 items-center gap-0.5 overflow-x-auto pr-1 pl-8 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
        role="navigation"
        aria-label="路径"
        @click.self="startEdit"
      >
        <template v-for="(segment, index) in segments" :key="segment.path">
          <div data-path-crumb class="flex shrink-0 items-center">
            <PathCrumb
              :segment="segment"
              :current="index === segments.length - 1"
              :active-child-path="segments[index + 1]?.path"
              :list-dirs="listDirs"
              :list-drives="listDrives"
              @navigate="navigate"
            />
          </div>
          <div
            v-if="index === 0 && segments.length > 1"
            data-path-more
            class="shrink-0 items-center"
            style="display: none"
          >
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <button
                  type="button"
                  class="rounded px-1.5 py-0.5 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                  title="显示隐藏的上级目录"
                  aria-label="显示隐藏的上级目录"
                >
                  …
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" class="max-h-64 min-w-40">
                <DropdownMenuItem
                  v-for="hidden in hiddenSegments"
                  :key="hidden.path"
                  :title="hidden.path"
                  @click="navigate(hidden.path)"
                >
                  <Folder class="text-amber-500" />
                  <span class="truncate">{{ hidden.name }}</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </template>
        <button
          v-if="segments.length === 0"
          type="button"
          class="h-full px-2 text-left text-sm text-muted-foreground"
          @click="startEdit"
        >
          {{ placeholder }}
        </button>
      </div>
      <button
        type="button"
        class="mr-1.5 shrink-0 rounded p-1 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        title="编辑路径"
        aria-label="编辑路径"
        @click="startEdit"
      >
        <Pencil class="size-3.5" />
      </button>
    </template>
  </div>
</template>

<script setup lang="ts">
/**
 * 文件面板地址栏：
 * - 默认展示可点击的面包屑，每段（含当前目录）右侧箭头可浏览子文件夹（「此电脑」可切换盘符）
 * - 宽度不足时仿 Windows 资源管理器隐藏前几级目录，以「…」菜单展示，始终保留根目录与末尾几级
 * - 点击空白处或铅笔进入编辑，回车跳转、Esc/失焦取消
 */
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { Folder, HardDrive, Pencil, X } from "@lucide/vue";
import type { SftpEntry } from "../types";
import { localBreadcrumbs, normalizeLocalPath, remoteBreadcrumbs } from "../lib/paths";
import { Input } from "./ui/input";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import PathCrumb from "./PathCrumb.vue";

const props = defineProps<{
  kind: "local" | "remote";
  path: string;
  /** 列出指定路径下的内容（由父级按本地/远端选择实现） */
  listDirs: (path: string) => Promise<SftpEntry[]>;
  /** 列出可用盘符（Windows） */
  listDrives: () => Promise<string[]>;
}>();

const emit = defineEmits<{ navigate: [path: string] }>();

const editing = ref(false);
const draft = ref(props.path);
const barEl = ref<HTMLDivElement | null>(null);
const crumbsEl = ref<HTMLDivElement | null>(null);

const placeholder = computed(() =>
  props.kind === "local" ? "输入本地路径，回车跳转" : "输入路径，回车跳转",
);

const segments = computed(() =>
  props.kind === "local" ? localBreadcrumbs(props.path) : remoteBreadcrumbs(props.path),
);

/** 首个可见面包屑下标；0 表示全部可见，其余值表示 [1, hiddenFrom) 被折叠进「…」 */
const hiddenFrom = ref(0);

const hiddenSegments = computed(() =>
  hiddenFrom.value > 1 ? segments.value.slice(1, hiddenFrom.value) : [],
);

let resizeObserver: ResizeObserver | null = null;
let rafId = 0;

onMounted(() => {
  void nextTick(updateOverflow);
  if (typeof ResizeObserver === "undefined" || !barEl.value) return;
  resizeObserver = new ResizeObserver(scheduleUpdateOverflow);
  resizeObserver.observe(barEl.value);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  if (rafId) cancelAnimationFrame(rafId);
});

function scheduleUpdateOverflow() {
  if (rafId) cancelAnimationFrame(rafId);
  rafId = requestAnimationFrame(updateOverflow);
}

/**
 * 按可用宽度折叠面包屑：保留根目录，从末尾尽量多保留，中间以「…」替代。
 * 先恢复全部可见再测量，保证折叠程度可随宽度变化回退。
 */
function updateOverflow() {
  rafId = 0;
  const bar = crumbsEl.value;
  if (!bar) return;
  const crumbs = Array.from(bar.querySelectorAll<HTMLElement>("[data-path-crumb]"));
  const marker = bar.querySelector<HTMLElement>("[data-path-more]");
  if (crumbs.length === 0) {
    hiddenFrom.value = 0;
    return;
  }
  for (const el of crumbs) el.style.display = "";
  if (marker) marker.style.display = "none";

  const style = getComputedStyle(bar);
  const gap = Number.parseFloat(style.columnGap) || 0;
  const available =
    bar.clientWidth -
    (Number.parseFloat(style.paddingLeft) || 0) -
    (Number.parseFloat(style.paddingRight) || 0);
  const widths = crumbs.map((el) => el.getBoundingClientRect().width);
  const total = widths.reduce((sum, width) => sum + width, 0) + gap * (widths.length - 1);
  if (total <= available || crumbs.length <= 2 || !marker) {
    hiddenFrom.value = 0;
    return;
  }

  marker.style.display = "flex";
  const markerWidth = marker.getBoundingClientRect().width;
  const last = widths.length - 1;
  let visibleFrom = last;
  let used = widths[0] + gap + markerWidth + gap + widths[last];
  for (let i = last - 1; i >= 1; i--) {
    if (used + gap + widths[i] > available) break;
    used += gap + widths[i];
    visibleFrom = i;
  }
  if (visibleFrom <= 1) {
    marker.style.display = "none";
    hiddenFrom.value = 0;
    return;
  }
  for (let i = 1; i < visibleFrom; i++) crumbs[i].style.display = "none";
  hiddenFrom.value = visibleFrom;
}

/** 进入编辑并全选当前路径 */
function startEdit() {
  if (editing.value) return;
  draft.value = props.path;
  editing.value = true;
  void nextTick(() => {
    const input = barEl.value?.querySelector("input");
    input?.focus();
    input?.select();
  });
}

/** 取消编辑，恢复当前路径（面包屑重挂载后由 watch 重新测量） */
function cancelEdit() {
  if (!editing.value) return;
  editing.value = false;
  draft.value = props.path;
}

/** 编辑态回车：规范化后跳转；目标与当前一致时直接退出 */
function submit() {
  const raw = draft.value.trim();
  if (!raw) return;
  const target =
    props.kind === "local" ? normalizeLocalPath(raw) : raw.startsWith("/") ? raw : "/" + raw;
  if (!target) return;
  if (target === props.path) {
    editing.value = false;
    return;
  }
  emit("navigate", target);
}

/** 面包屑点击（父级加载成功后 path 更新，编辑态由 watch 退出） */
function navigate(path: string) {
  if (path === props.path) return;
  emit("navigate", path);
}

/** 当前目录变化：同步输入框、退出编辑并重新计算面包屑折叠 */
watch(
  () => props.path,
  (path) => {
    draft.value = path;
    editing.value = false;
    void nextTick(() => {
      scrollToEnd();
      updateOverflow();
    });
  },
);

/** 退出编辑态后面包屑重新挂载，需重新测量宽度 */
watch(editing, (value) => {
  if (value) return;
  void nextTick(() => {
    scrollToEnd();
    updateOverflow();
  });
});

function scrollToEnd() {
  if (crumbsEl.value) crumbsEl.value.scrollLeft = crumbsEl.value.scrollWidth;
}
</script>
