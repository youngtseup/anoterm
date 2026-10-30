<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-[460px]">
      <DialogHeader>
        <DialogTitle>{{ initial ? "修改连接" : "新增连接" }}</DialogTitle>
        <DialogDescription class="sr-only">
          {{ initial ? "修改连接信息" : "创建新连接" }}
        </DialogDescription>
      </DialogHeader>

      <div class="grid gap-4">
        <div class="grid grid-cols-[80px_1fr] items-center gap-3">
          <Label>名称</Label>
          <Input v-model="form.name" placeholder="连接名称" />
        </div>
        <div class="grid grid-cols-[80px_1fr] items-center gap-3">
          <Label>类型</Label>
          <Select v-model="form.type">
            <SelectTrigger class="w-full">
              <SelectValue placeholder="选择类型" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="ssh_sftp">SSH &amp; SFTP</SelectItem>
              <SelectItem value="ssh">SSH</SelectItem>
              <SelectItem value="sftp">SFTP</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div class="grid grid-cols-[80px_1fr] items-center gap-3">
          <Label>主机</Label>
          <Input v-model="form.host" placeholder="127.0.0.1" />
        </div>
        <div class="grid grid-cols-[80px_1fr] items-center gap-3">
          <Label>端口</Label>
          <Input
            v-model="portText"
            type="number"
            min="1"
            max="65535"
            placeholder="22"
            @blur="syncPort"
          />
        </div>
        <div class="grid grid-cols-[80px_1fr] items-center gap-3">
          <Label>用户名</Label>
          <Input v-model="form.username" placeholder="请输入用户名" />
        </div>
        <div class="grid grid-cols-[80px_1fr] items-center gap-3">
          <Label>认证方式</Label>
          <RadioGroup v-model="form.authType" class="flex items-center gap-4">
            <div class="flex items-center gap-2">
              <RadioGroupItem id="auth-password" value="password" />
              <Label for="auth-password" class="font-normal">密码</Label>
            </div>
            <div class="flex items-center gap-2">
              <RadioGroupItem id="auth-key" value="key" />
              <Label for="auth-key" class="font-normal">私钥</Label>
            </div>
          </RadioGroup>
        </div>

        <div
          v-if="form.authType === 'password'"
          class="grid grid-cols-[80px_1fr] items-center gap-3"
        >
          <Label>密码</Label>
          <div class="relative">
            <Input
              v-model="form.password"
              :type="showPassword ? 'text' : 'password'"
              class="pr-9"
              :placeholder="initial ? '留空则不修改（已加密保存）' : '请输入密码'"
            />
            <button
              type="button"
              class="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              @click="showPassword = !showPassword"
            >
              <EyeOff v-if="showPassword" class="size-4" />
              <Eye v-else class="size-4" />
            </button>
          </div>
        </div>

        <template v-else>
          <div class="grid grid-cols-[80px_1fr] items-center gap-3">
            <Label>私钥文件</Label>
            <div class="flex gap-2">
              <Input
                v-model="form.privateKeyPath"
                class="flex-1"
                placeholder="私钥路径（OpenSSH / PEM / PPK）"
              />
              <Button type="button" variant="outline" @click="pickKeyFile">选择文件</Button>
            </div>
          </div>
          <div class="grid grid-cols-[80px_1fr] items-center gap-3">
            <Label>口令</Label>
            <div class="relative">
              <Input
                v-model="form.keyPassphrase"
                :type="showPassword ? 'text' : 'password'"
                class="pr-9"
                placeholder="私钥口令（无口令留空）"
              />
              <button
                type="button"
                class="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                @click="showPassword = !showPassword"
              >
                <EyeOff v-if="showPassword" class="size-4" />
                <Eye v-else class="size-4" />
              </button>
            </div>
          </div>
        </template>

        <div class="grid grid-cols-[80px_1fr] items-center gap-3">
          <Label>备注</Label>
          <Textarea v-model="form.remark" :rows="2" placeholder="备注信息（可选）" />
        </div>
        <div class="grid grid-cols-[80px_1fr] items-center gap-3">
          <Label>分组</Label>
          <div class="relative">
            <Select v-model="groupValue">
              <SelectTrigger class="w-full">
                <SelectValue placeholder="请选择分组" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="g in selectableGroups" :key="g.id" :value="g.id">
                  {{ g.name }}
                </SelectItem>
              </SelectContent>
            </Select>
            <button
              v-if="form.groupId"
              type="button"
              class="absolute top-1/2 right-9 z-10 -translate-y-1/2 rounded p-0.5 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              title="清除分组"
              aria-label="清除分组"
              @click="clearGroup"
            >
              <X class="size-3.5" />
            </button>
          </div>
        </div>
      </div>

      <!-- 表单校验错误 -->
      <p v-if="errorMsg" class="text-[13px] text-destructive">{{ errorMsg }}</p>

      <!-- 测试连接失败详情 -->
      <div
        v-if="testErrorMsg"
        class="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/10 p-2.5 text-[13px] text-destructive"
      >
        <CircleAlert class="mt-0.5 size-4 shrink-0" />
        <span class="break-all whitespace-pre-wrap">{{ testErrorMsg }}</span>
      </div>

      <DialogFooter>
        <div class="flex items-center gap-2 sm:mr-auto" aria-live="polite">
          <Button variant="outline" :disabled="testing" @click="testConnection">
            <Loader2 v-if="testing" class="size-4 animate-spin" />
            {{ testing ? "测试中…" : "测试连接" }}
          </Button>
          <span
            v-if="testState === 'success'"
            class="flex items-center gap-1 text-[13px] text-emerald-600 dark:text-emerald-400"
          >
            <CircleCheck class="size-4" />
            连接成功
          </span>
          <span
            v-else-if="testState === 'fail'"
            class="flex items-center gap-1 text-[13px] text-destructive"
          >
            <CircleX class="size-4" />
            连接失败
          </span>
        </div>
        <Button variant="outline" @click="open = false">取消</Button>
        <Button :disabled="testing" @click="confirm">确定</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
/**
 * 新增/修改连接对话框。
 * - 打开时根据 initial 回填表单（为空则按新增模式）
 * - 认证方式：密码 / 私钥（私钥仅存路径，口令加密存储）
 * - 提交校验：名称必填；分组可选；私钥模式还需私钥文件路径
 */
import { computed, ref, watch } from "vue";
import { CircleAlert, CircleCheck, CircleX, Eye, EyeOff, Loader2, X } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { EMPTY_CONN_INPUT, isUngroupedGroup } from "../types";
import type { AddConnInput, Connection, Group } from "../types";
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
import { Label } from "./ui/label";
import { RadioGroup, RadioGroupItem } from "./ui/radio-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { Textarea } from "./ui/textarea";

const props = defineProps<{
  modelValue: boolean;
  groups: Group[];
  initial?: Connection | null;
  presetGroupId?: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  submit: [input: AddConnInput];
}>();

const open = computed({
  get: () => props.modelValue,
  set: (v: boolean) => emit("update:modelValue", v),
});

const form = ref<AddConnInput>({ ...EMPTY_CONN_INPUT });
const portText = ref(String(EMPTY_CONN_INPUT.port));
/** 空字符串不是合法选项，未分组时用 undefined 让 Select 展示占位文本 */
const groupValue = computed<string | undefined>({
  get: () => form.value.groupId || undefined,
  set: (value) => {
    form.value.groupId = value ?? "";
  },
});

function clearGroup() {
  form.value.groupId = "";
}
const selectableGroups = computed(() => props.groups.filter((g) => !isUngroupedGroup(g)));
const showPassword = ref(false);
const errorMsg = ref("");
/** 测试连接进行中 */
const testing = ref(false);
/** 测试连接状态：idle / testing / success / fail */
const testState = ref<"idle" | "testing" | "success" | "fail">("idle");
/** 测试连接失败详情 */
const testErrorMsg = ref("");

/**
 * 编辑模式的密码/口令掩码占位。
 * 密码与私钥口令均加密存库，前端不接触明文；提交时若仍为掩码则传空，
 * 后端 update_connection 遇到空值会保留原值。
 */
const PASSWORD_MASK = "********";

/** 端口输入同步：失焦/提交时将字符串解析并夹取到 [1, 65535] */
function syncPort() {
  const n = parseInt(portText.value, 10);
  if (Number.isNaN(n)) {
    portText.value = String(form.value.port);
    return;
  }
  const clamped = Math.min(65535, Math.max(1, n));
  portText.value = String(clamped);
  form.value.port = clamped;
}

/** 选择私钥文件（返回绝对路径） */
async function pickKeyFile() {
  const selected = await openDialog({ multiple: false, directory: false, title: "选择私钥文件" });
  if (typeof selected === "string") form.value.privateKeyPath = selected;
}

// 打开时回填：编辑模式用 initial，新增模式用空模板 + 预选分组
watch(
  () => props.modelValue,
  (visible) => {
    if (visible) {
      const init = props.initial;
      form.value = init
        ? {
            groupId: init.groupId,
            name: init.name,
            type: init.type,
            host: init.host,
            port: init.port,
            username: init.username,
            password: PASSWORD_MASK,
            remark: init.remark,
            authType: init.authType || "password",
            privateKeyPath: init.privateKeyPath,
            keyPassphrase: init.keyPassphrase ? PASSWORD_MASK : "",
          }
        : {
            ...EMPTY_CONN_INPUT,
            groupId: props.presetGroupId ?? "",
          };
      portText.value = String(form.value.port);
      showPassword.value = false;
      errorMsg.value = "";
      testing.value = false;
      testState.value = "idle";
      testErrorMsg.value = "";
    }
  },
);

// 用户修改任意字段后清除上一次测试结果与校验错误，避免陈旧状态误导
watch(
  [form, portText],
  () => {
    if (testing.value) return;
    testState.value = "idle";
    testErrorMsg.value = "";
    errorMsg.value = "";
  },
  { deep: true, flush: "sync" },
);

/** 测试连接：不保存配置，直接尝试建立并断开 SSH 会话 */
async function testConnection() {
  syncPort();
  testErrorMsg.value = "";
  if (!form.value.host.trim()) {
    testState.value = "idle";
    errorMsg.value = "请输入主机地址";
    return;
  }
  if (form.value.authType === "key" && !form.value.privateKeyPath.trim()) {
    testState.value = "idle";
    errorMsg.value = "请选择私钥文件";
    return;
  }
  errorMsg.value = "";
  testState.value = "testing";
  testing.value = true;
  try {
    // 掩码值置空：编辑模式下由后端使用已保存的密码/口令
    const payload = { ...form.value };
    if (payload.password === PASSWORD_MASK) payload.password = "";
    if (payload.keyPassphrase === PASSWORD_MASK) payload.keyPassphrase = "";
    await invoke("ssh_test_connection", {
      host: payload.host,
      port: payload.port,
      username: payload.username,
      password: payload.password,
      authType: payload.authType,
      privateKeyPath: payload.privateKeyPath,
      keyPassphrase: payload.keyPassphrase,
      connectionId: props.initial?.id ?? null,
    });
    testState.value = "success";
  } catch (e) {
    const msg = typeof e === "string" ? e : ((e as Error)?.message ?? "测试连接失败");
    testState.value = "fail";
    testErrorMsg.value = msg;
  } finally {
    testing.value = false;
  }
}

function confirm() {
  syncPort();
  testErrorMsg.value = "";
  if (!form.value.name.trim()) {
    errorMsg.value = "请输入连接名称";
    return;
  }
  if (form.value.authType === "key" && !form.value.privateKeyPath.trim()) {
    errorMsg.value = "请选择私钥文件";
    return;
  }
  errorMsg.value = "";
  // 编辑模式未修改的密码/口令：置空提交，由后端保留原值
  const input = { ...form.value };
  if (input.password === PASSWORD_MASK) input.password = "";
  if (input.keyPassphrase === PASSWORD_MASK) input.keyPassphrase = "";
  emit("submit", input);
  open.value = false;
}
</script>
