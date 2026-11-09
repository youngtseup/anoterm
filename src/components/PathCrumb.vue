<template>
  <div class="flex shrink-0 items-center">
    <button
      v-if="!current"
      type="button"
      class="max-w-[220px] truncate rounded px-1.5 py-0.5 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
      :title="segment.drives ? '切换盘符' : segment.path"
      @click="onCrumbClick"
    >
      {{ segment.name }}
    </button>
    <span
      v-else
      class="max-w-[220px] truncate px-1.5 py-0.5 text-foreground"
      :title="segment.path"
      aria-current="page"
    >
      {{ segment.name }}
    </span>

    <DropdownMenu :open="menuOpen" @update:open="onMenuOpenChange">
      <DropdownMenuTrigger as-child>
        <button
          type="button"
          class="rounded p-0.5 text-muted-foreground/60 transition-colors hover:bg-accent hover:text-foreground"
          :title="segment.drives ? '切换盘符' : '浏览子文件夹'"
          :aria-label="segment.drives ? '切换盘符' : '浏览子文件夹'"
        >
          <ChevronRight class="size-3.5" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" class="max-h-64 min-w-40">
        <DropdownMenuLabel class="max-w-[260px] truncate">
          {{ mode === "drives" ? "切换盘符" : segment.name }}
        </DropdownMenuLabel>
        <DropdownMenuItem v-if="loading" disabled>
          <Loader2 class="animate-spin" />
          加载中…
        </DropdownMenuItem>
        <DropdownMenuItem v-else-if="failed" disabled>加载失败，请重新打开</DropdownMenuItem>
        <DropdownMenuItem v-else-if="items.length === 0" disabled>
          {{ mode === "drives" ? "没有可用盘符" : "没有子文件夹" }}
        </DropdownMenuItem>
        <template v-else>
          <DropdownMenuItem
            v-for="item in items"
            :key="item.path"
            :title="item.path"
            @click="emit('navigate', item.path)"
          >
            <HardDrive v-if="mode === 'drives'" class="text-muted-foreground" />
            <Folder v-else class="text-amber-500" />
            <span
              class="truncate"
              :class="item.path === activeChildPath ? 'font-semibold text-foreground' : ''"
            >
              {{ item.name }}
            </span>
          </DropdownMenuItem>
        </template>
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>

<script setup lang="ts">
/**
 * 地址栏中的单个路径段：
 * - 左键点击普通段跳转；「此电脑」段左键列出盘符
 * - 每段右侧箭头列出该路径下的子文件夹（当前段同样支持，「此电脑」列出盘符）
 * 每次打开菜单都重新加载，避免展示过期的目录内容。
 */
import { ref } from "vue";
import { ChevronRight, Folder, HardDrive, Loader2 } from "@lucide/vue";
import type { PathSegment } from "../lib/paths";
import type { SftpEntry } from "../types";
import { notifyError } from "../api";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";

const props = defineProps<{
  segment: PathSegment;
  /** 是否为当前目录（当前段不可点击跳转，但可通过右侧箭头浏览子文件夹） */
  current: boolean;
  /** 当前实际进入的下一级路径，用于菜单中高亮 */
  activeChildPath?: string;
  listDirs: (path: string) => Promise<SftpEntry[]>;
  /** 列出可用盘符（Windows） */
  listDrives: () => Promise<string[]>;
}>();

const emit = defineEmits<{ navigate: [path: string] }>();

type MenuMode = "dirs" | "drives";
interface CrumbItem {
  name: string;
  path: string;
}

const menuOpen = ref(false);
const mode = ref<MenuMode>(props.segment.drives ? "drives" : "dirs");
const loading = ref(false);
const failed = ref(false);
const items = ref<CrumbItem[]>([]);
const nameCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
let requestId = 0;

async function loadItems() {
  const id = ++requestId;
  loading.value = true;
  failed.value = false;
  try {
    let list: CrumbItem[];
    if (mode.value === "drives") {
      list = (await props.listDrives()).map((root) => ({
        name: root.replace(/[\\/]+$/, "") || root,
        path: root,
      }));
    } else {
      list = (await props.listDirs(props.segment.path))
        .filter((entry) => entry.isDir)
        .map((entry) => ({ name: entry.name, path: entry.path }))
        .sort((a, b) => nameCollator.compare(a.name, b.name));
    }
    if (id !== requestId) return;
    items.value = list;
  } catch (e) {
    if (id !== requestId) return;
    failed.value = true;
    notifyError(e);
  } finally {
    if (id === requestId) loading.value = false;
  }
}

function onMenuOpenChange(value: boolean) {
  menuOpen.value = value;
  if (value) void loadItems();
}

/** 普通段左键跳转；「此电脑」左键打开盘符菜单 */
function onCrumbClick() {
  if (props.segment.drives) {
    menuOpen.value = true;
    void loadItems();
  } else {
    emit("navigate", props.segment.path);
  }
}
</script>
