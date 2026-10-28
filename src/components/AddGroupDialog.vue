<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-[380px]">
      <DialogHeader>
        <DialogTitle>{{ initialName ? "重命名分组" : "新增分组" }}</DialogTitle>
        <DialogDescription class="sr-only">
          {{ initialName ? "重命名分组" : "新增分组" }}
        </DialogDescription>
      </DialogHeader>
      <Input v-model="name" placeholder="请输入分组名称" @keyup.enter="confirm" />
      <DialogFooter>
        <Button variant="outline" @click="open = false">取消</Button>
        <Button @click="confirm">确定</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
/**
 * 新增/重命名分组对话框（按 initialName 区分模式）。
 */
import { computed, ref, watch } from "vue";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { Input } from "./ui/input";

const props = defineProps<{
  modelValue: boolean;
  initialName?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  submit: [name: string];
}>();

const open = computed({
  get: () => props.modelValue,
  set: (v: boolean) => emit("update:modelValue", v),
});

const name = ref("");

watch(
  () => props.modelValue,
  (visible) => {
    if (visible) {
      name.value = props.initialName ?? "";
    }
  },
);

function confirm() {
  if (name.value.trim()) {
    emit("submit", name.value.trim());
    open.value = false;
  }
}
</script>
