<template>
  <div class="flex-1 overflow-y-auto px-8 py-7">
    <div class="mb-8 flex gap-[18px]">
      <div
        class="flex-1 rounded-lg border border-border/60 bg-card p-5.5 shadow-sm transition-shadow hover:shadow-md"
      >
        <div
          class="mb-3 inline-flex size-10 items-center justify-center rounded-[10px] bg-blue-500/10 text-blue-600 dark:bg-blue-400/10 dark:text-blue-400"
        >
          <Folder class="size-5" />
        </div>
        <div class="mb-2 text-[13px] text-muted-foreground">分组数</div>
        <div class="text-[32px] leading-none font-bold">{{ groupCount }}</div>
      </div>
      <div
        class="flex-1 rounded-lg border border-border/60 bg-card p-5.5 shadow-sm transition-shadow hover:shadow-md"
      >
        <div
          class="mb-3 inline-flex size-10 items-center justify-center rounded-[10px] bg-emerald-500/10 text-emerald-600 dark:bg-emerald-400/10 dark:text-emerald-400"
        >
          <Plug class="size-5" />
        </div>
        <div class="mb-2 text-[13px] text-muted-foreground">连接数</div>
        <div class="text-[32px] leading-none font-bold">{{ totalConnections }}</div>
      </div>
      <div
        class="flex-1 rounded-lg border border-border/60 bg-card p-5.5 shadow-sm transition-shadow hover:shadow-md"
      >
        <div
          class="mb-3 inline-flex size-10 items-center justify-center rounded-[10px] bg-amber-500/10 text-amber-600 dark:bg-amber-400/10 dark:text-amber-400"
        >
          <CircleCheck class="size-5" />
        </div>
        <div class="mb-2 text-[13px] text-muted-foreground">已连接</div>
        <div class="text-[32px] leading-none font-bold">{{ connectedCount }}</div>
      </div>
    </div>

    <div class="rounded-lg border border-border/60 bg-card shadow-sm">
      <div class="flex items-center justify-between border-b border-border/60 px-5 py-4">
        <span class="text-[15px] font-semibold">最近连接</span>
        <Button
          v-if="recentConnections.length > 0"
          variant="link"
          size="sm"
          class="text-destructive hover:text-destructive"
          @click="emit('clear-recent')"
        >
          清空
        </Button>
      </div>

      <div v-if="recentConnections.length > 0" class="py-1">
        <div
          class="flex cursor-pointer items-center justify-between px-5 py-3 transition-colors hover:bg-muted/60"
          v-for="(item, i) in recentConnections"
          :key="i"
          @click="emit('select-recent', item.id)"
        >
          <div class="flex items-center gap-3">
            <div
              class="flex size-9 shrink-0 items-center justify-center rounded-lg bg-blue-500/10 text-blue-600 dark:bg-blue-400/10 dark:text-blue-400"
            >
              <Monitor v-if="item.type === 'ssh'" class="size-4" />
              <FolderOpen v-else-if="item.type === 'sftp'" class="size-4" />
              <Coins v-else class="size-4" />
            </div>
            <div class="flex flex-col gap-0.5">
              <span class="text-[13.5px] font-medium">{{ item.name }}</span>
              <span class="text-xs text-muted-foreground"
                >{{ item.host }}:{{ item.port }} · {{ item.username }}</span
              >
            </div>
          </div>
          <span class="text-xs text-muted-foreground">{{ fmtTime(item.lastUsed) }}</span>
        </div>
      </div>
      <div v-else class="px-5 py-10 text-center text-[13px] text-muted-foreground">
        暂无最近连接记录
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * 仪表盘（无 tab 时显示）：
 * - 统计卡片：分组数 / 连接数 / 已连接数
 * - 最近连接列表：点击重新打开连接，可清空
 */
import { computed } from "vue";
import { CircleCheck, Coins, Folder, FolderOpen, Monitor, Plug } from "@lucide/vue";
import { isUngroupedGroup, type Group, type RecentItem } from "../types";
import { Button } from "./ui/button";

const props = defineProps<{
  groups: Group[];
  recentConnections: RecentItem[];
  connectedCount: number;
}>();

const emit = defineEmits<{
  "select-recent": [id: string];
  "clear-recent": [];
}>();

const groupCount = computed(() => props.groups.filter((group) => !isUngroupedGroup(group)).length);

const totalConnections = computed(() =>
  props.groups.reduce((sum, g) => sum + g.connections.length, 0),
);

/** 相对时间格式化（刚刚 / x分钟前 / x小时前 / x天前 / 日期） */
function fmtTime(ts: number): string {
  const diff = Math.floor(Date.now() / 1000) - ts;
  if (diff < 60) return "刚刚";
  if (diff < 3600) return Math.floor(diff / 60) + " 分钟前";
  if (diff < 86400) return Math.floor(diff / 3600) + " 小时前";
  if (diff < 86400 * 30) return Math.floor(diff / 86400) + " 天前";
  const d = new Date(ts * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}
</script>
