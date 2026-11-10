<script setup lang="ts">
import { PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from "reka-ui";
import { connectionAuthLabel, connectionTypeLabel, type ConnectionDetails } from "../types";

withDefaults(
  defineProps<{
    open: boolean;
    details: ConnectionDetails | null;
    side?: "top" | "right" | "bottom" | "left";
  }>(),
  { side: "right" },
);

const emit = defineEmits<{
  "update:open": [open: boolean];
}>();

const contentClass =
  "bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 max-h-(--reka-popper-available-height) w-72 origin-(--reka-popper-transform-origin) overflow-y-auto rounded-md border p-3 shadow-md outline-hidden";
</script>

<template>
  <PopoverRoot :open="open" @update:open="emit('update:open', $event)">
    <PopoverTrigger as-child>
      <slot />
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent
        v-if="details"
        :side="side"
        align="start"
        :side-offset="8"
        :class="contentClass"
      >
        <div class="space-y-3">
          <div class="min-w-0">
            <div class="flex items-center gap-1.5">
              <span class="truncate text-sm font-semibold">{{ details.name }}</span>
              <span
                v-if="details.deleted"
                class="shrink-0 rounded bg-destructive/10 px-1.5 py-0.5 text-[10px] leading-none text-destructive"
              >
                已删除
              </span>
            </div>
            <div class="mt-0.5 text-[11px] text-muted-foreground">
              {{ connectionTypeLabel(details.type) }}
            </div>
          </div>

          <div class="space-y-1.5 text-xs">
            <div class="flex gap-2">
              <span class="w-14 shrink-0 text-muted-foreground">地址</span>
              <span class="min-w-0 font-mono break-all">{{ details.host }}:{{ details.port }}</span>
            </div>
            <div class="flex gap-2">
              <span class="w-14 shrink-0 text-muted-foreground">用户名</span>
              <span class="min-w-0 break-all">{{ details.username || "—" }}</span>
            </div>
            <div class="flex gap-2">
              <span class="w-14 shrink-0 text-muted-foreground">连接方式</span>
              <span class="min-w-0 break-all">{{ connectionAuthLabel(details.authType) }}</span>
            </div>
            <div v-if="details.remark" class="flex gap-2">
              <span class="w-14 shrink-0 text-muted-foreground">备注</span>
              <span class="min-w-0 whitespace-pre-wrap break-words">{{ details.remark }}</span>
            </div>
          </div>

          <p v-if="details.deleted" class="text-[11px] text-muted-foreground">
            连接已删除，仅显示最近记录。
          </p>
        </div>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>
