<template>
  <AlertDialog :open="!!conflict">
    <AlertDialogContent @escape-key-down="choose('cancel')">
      <AlertDialogHeader>
        <AlertDialogTitle>名称冲突</AlertDialogTitle>
        <AlertDialogDescription>
          {{ conflict?.direction === "upload" ? "上传" : "下载" }}「{{
            conflict?.name
          }}」时，目标位置已存在同名{{ conflict?.existingIsDir ? "文件夹" : "文件" }}。
        </AlertDialogDescription>
      </AlertDialogHeader>

      <div v-if="conflict" class="space-y-3 text-[13px]">
        <div class="rounded-md border border-border bg-muted/40 p-2 font-mono text-xs break-all">
          <div><span class="text-muted-foreground">源：</span>{{ conflict.source }}</div>
          <div class="mt-1">
            <span class="text-muted-foreground">目标：</span>{{ conflict.dest }}
          </div>
        </div>
        <p class="text-muted-foreground">
          当前为{{ conflict.isDir ? "文件夹" : "文件" }}，目标为{{
            conflict.existingIsDir ? "文件夹" : "文件"
          }}。
          <template v-if="conflict.isDir !== conflict.existingIsDir">
            选择「覆盖」将先删除目标后再复制。
          </template>
          <template v-else-if="conflict.isDir"> 选择「覆盖」将删除目标文件夹并整体替换。 </template>
        </p>
        <label class="flex items-center justify-between gap-3 select-none">
          <span>应用于后续冲突</span>
          <Switch v-model="applyAll" />
        </label>
      </div>

      <AlertDialogFooter class="sm:justify-between">
        <Button variant="outline" @click="choose('cancel')">取消传输</Button>
        <div class="flex flex-wrap justify-end gap-2">
          <Button variant="outline" @click="choose('skip')">跳过</Button>
          <Button variant="outline" @click="choose('keepBoth')">保留两者</Button>
          <Button @click="choose('overwrite')">覆盖</Button>
        </div>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>

<script setup lang="ts">
/** 同名冲突弹窗：覆盖 / 跳过 / 保留两者 / 取消传输 */
import { ref, watch } from "vue";
import type { ConflictChoice, TransferConflict } from "../api";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import { Button } from "./ui/button";
import { Switch } from "./ui/switch";

const props = defineProps<{ conflict: TransferConflict | null }>();
const emit = defineEmits<{ resolve: [choice: ConflictChoice, applyAll: boolean] }>();

/** 是否把当前选择应用到本次传输后续冲突 */
const applyAll = ref(false);

watch(
  () => props.conflict,
  () => {
    applyAll.value = false;
  },
);

function choose(choice: ConflictChoice) {
  emit("resolve", choice, choice === "cancel" ? false : applyAll.value);
}
</script>
