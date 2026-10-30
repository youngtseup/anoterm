<template>
  <div
    class="tab-bar flex h-12 shrink-0 items-stretch overflow-x-auto overflow-y-hidden border-b border-border bg-background"
  >
    <div
      v-for="tab in tabs"
      :key="tab.tabId"
      class="tab flex shrink-0 cursor-pointer flex-col justify-between px-2 pb-1 pt-1 transition-colors select-none"
      :class="
        tab.tabId === activeTabId
          ? 'border-r border-border bg-muted/50 text-foreground shadow-[inset_0_2px_0_var(--color-primary)]'
          : 'border-r border-border bg-background text-muted-foreground'
      "
      @click="emit('select', tab.tabId)"
      @mousedown.middle.prevent="emit('close', tab.tabId)"
      @contextmenu.prevent="openMenu($event, tab.tabId)"
    >
      <div
        class="flex items-center gap-1"
        :title="`名称：${tab.name}\n主机：${tab.host}\n端口：${tab.port}\n用户名：${tab.username}\n备注：${tab.remark || '—'}`"
      >
        <span class="min-w-0 flex-1 truncate text-[12.5px]">{{ tab.name }}</span>
        <span
          class="flex size-4 shrink-0 items-center justify-center rounded hover:bg-black/10 hover:text-foreground dark:hover:bg-white/10"
          @click.stop="emit('close', tab.tabId)"
        >
          <X class="size-3" />
        </span>
      </div>
      <div v-if="subTabsForType(tab.connectionType).length > 1" class="mt-1 flex gap-1" @click.stop>
        <span
          v-for="st in subTabsForType(tab.connectionType)"
          :key="st"
          class="cursor-pointer rounded-full border px-2 py-0.5 text-[11px] leading-none"
          :class="
            tab.activeSub === st
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-border bg-muted text-muted-foreground'
          "
          @click.stop="emit('set-sub', tab.tabId, st)"
          @contextmenu="onPillContextmenu($event, st)"
        >
          {{ st === "ssh" ? "SSH" : "SFTP" }}
        </span>
      </div>
    </div>
  </div>

  <!-- 右键菜单（锚定在鼠标坐标的零尺寸触发元素上） -->
  <DropdownMenu v-model:open="menuOpen">
    <DropdownMenuTrigger as-child>
      <div class="fixed size-0" :style="{ left: `${menuPos.x}px`, top: `${menuPos.y}px` }" />
    </DropdownMenuTrigger>
    <DropdownMenuContent side="right" align="start" :side-offset="0" class="min-w-40">
      <template v-if="contextSupportsSftp">
        <DropdownMenuLabel>SFTP 布局</DropdownMenuLabel>
        <DropdownMenuRadioGroup :model-value="sftpLayout" @update:model-value="setSftpLayout">
          <DropdownMenuRadioItem value="dual">双栏布局</DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="single">单栏布局</DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
        <DropdownMenuSeparator />
      </template>
      <DropdownMenuItem @click="run('close')">关闭标签</DropdownMenuItem>
      <DropdownMenuItem @click="run('close-left')">关闭左侧标签</DropdownMenuItem>
      <DropdownMenuItem @click="run('close-right')">关闭右侧标签</DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem @click="run('close-others')">关闭其他标签</DropdownMenuItem>
      <DropdownMenuItem @click="run('close-all')">关闭所有标签</DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>

  <!-- SFTP 标签右键菜单：切换双栏/单栏布局 -->
  <DropdownMenu v-model:open="sftpMenuOpen">
    <DropdownMenuTrigger as-child>
      <div
        class="fixed size-0"
        :style="{ left: `${sftpMenuPos.x}px`, top: `${sftpMenuPos.y}px` }"
      />
    </DropdownMenuTrigger>
    <DropdownMenuContent side="right" align="start" :side-offset="0" class="min-w-36">
      <DropdownMenuLabel>文件浏览器布局</DropdownMenuLabel>
      <DropdownMenuRadioGroup :model-value="sftpLayout" @update:model-value="setSftpLayout">
        <DropdownMenuRadioItem value="dual">双栏布局</DropdownMenuRadioItem>
        <DropdownMenuRadioItem value="single">单栏布局</DropdownMenuRadioItem>
      </DropdownMenuRadioGroup>
    </DropdownMenuContent>
  </DropdownMenu>
</template>

<script setup lang="ts">
/**
 * 外层标签栏：
 * - 每个 tab 两行：名称 + SSH/SFTP 子视图切换 pill（仅多子视图类型显示）
 * - 中键关闭、右键菜单（关闭/左侧/右侧/其他/所有）
 * - SFTP pill 右键菜单：切换双栏/单栏布局
 */
import { computed, ref } from "vue";
import { X } from "@lucide/vue";
import type { SubKind, TerminalTab } from "../types";
import { subTabsForType } from "../types";
import { useSftpLayout } from "../composables/useSftpLayout";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";

const props = defineProps<{
  tabs: TerminalTab[];
  activeTabId: string | null;
}>();

const emit = defineEmits<{
  select: [tabId: string];
  "set-sub": [tabId: string, sub: SubKind];
  close: [tabId: string];
  "close-others": [tabId: string];
  "close-left": [tabId: string];
  "close-right": [tabId: string];
  "close-all": [];
}>();

const menuOpen = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const contextTabId = ref<string | null>(null);

function openMenu(e: MouseEvent, tabId: string) {
  contextTabId.value = tabId;
  menuPos.value = { x: e.clientX, y: e.clientY };
  menuOpen.value = true;
}

function run(cmd: string) {
  const tabId = contextTabId.value;
  if (!tabId) return;
  if (cmd === "close") emit("close", tabId);
  else if (cmd === "close-left") emit("close-left", tabId);
  else if (cmd === "close-right") emit("close-right", tabId);
  else if (cmd === "close-others") emit("close-others", tabId);
  else if (cmd === "close-all") emit("close-all");
}

// ---------- SFTP 布局切换 ----------
const { layout: sftpLayout } = useSftpLayout();
const sftpMenuOpen = ref(false);
const sftpMenuPos = ref({ x: 0, y: 0 });

/** 右键的标签是否包含 SFTP 视图 */
const contextSupportsSftp = computed(() => {
  const tab = props.tabs.find((t) => t.tabId === contextTabId.value);
  return !!tab && subTabsForType(tab.connectionType).includes("sftp");
});

/** 仅 SFTP pill 拦截右键；SSH pill 仍走标签栏菜单 */
function onPillContextmenu(ev: MouseEvent, sub: SubKind) {
  if (sub !== "sftp") return;
  ev.preventDefault();
  ev.stopPropagation();
  sftpMenuPos.value = { x: ev.clientX, y: ev.clientY };
  sftpMenuOpen.value = true;
}

function setSftpLayout(value: unknown) {
  if (value === "dual" || value === "single") sftpLayout.value = value;
}
</script>

<style scoped>
.tab-bar::-webkit-scrollbar {
  height: 0;
}
</style>
