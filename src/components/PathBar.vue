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
          <PathCrumb
            :segment="segment"
            :current="index === segments.length - 1"
            :active-child-path="segments[index + 1]?.path"
            :list-dirs="listDirs"
            :list-drives="listDrives"
            @navigate="navigate"
          />
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
 * - 点击空白处或铅笔进入编辑，回车跳转、Esc/失焦取消
 */
import { computed, nextTick, ref, watch } from "vue";
import { Folder, HardDrive, Pencil, X } from "@lucide/vue";
import type { SftpEntry } from "../types";
import { localBreadcrumbs, normalizeLocalPath, remoteBreadcrumbs } from "../lib/paths";
import { Input } from "./ui/input";
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

/** 取消编辑，恢复当前路径 */
function cancelEdit() {
  if (!editing.value) return;
  editing.value = false;
  draft.value = props.path;
  void nextTick(scrollToEnd);
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

/** 当前目录变化：同步输入框、退出编辑并滚动到路径末尾 */
watch(
  () => props.path,
  (path) => {
    draft.value = path;
    editing.value = false;
    void nextTick(scrollToEnd);
  },
);

function scrollToEnd() {
  if (crumbsEl.value) crumbsEl.value.scrollLeft = crumbsEl.value.scrollWidth;
}
</script>
