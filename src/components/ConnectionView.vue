<template>
  <div class="connection-view">
    <TerminalView
      v-if="hasSsh"
      v-show="activeSub === 'ssh'"
      :tab-id="tabId"
      :active="active && activeSub === 'ssh'"
      @close="emit('close', $event)"
    />
    <SftpView v-if="hasSftp" v-show="activeSub === 'sftp'" :tab-id="tabId" :ready="ready" />
  </div>
</template>

<script setup lang="ts">
/**
 * 连接面板：一个外层 tab 的会话宿主。
 * - 负责建立/断开会话（ssh_connect / ssh_disconnect），子视图共享该会话
 * - 挂载顺序：子组件先 onMounted → 本组件再连接，保证终端事件监听已就绪
 * - 连接成功 emit connected（App 刷新最近连接），失败则关闭 tab
 * - TerminalView / SftpView 用 v-show 切换，常驻保活
 */
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Connection, SubKind } from "../types";
import TerminalView from "./TerminalView.vue";
import SftpView from "./SftpView.vue";

const props = defineProps<{
  connection: Connection;
  tabId: string;
  active?: boolean;
  activeSub: SubKind;
}>();
const emit = defineEmits<{
  close: [tabId: string];
  connected: [tabId: string];
  error: [message: string];
}>();

const hasSsh = computed(
  () => props.connection.type === "ssh" || props.connection.type === "ssh_sftp",
);
const hasSftp = computed(
  () => props.connection.type === "sftp" || props.connection.type === "ssh_sftp",
);

let disposed = false;
/** 会话建立完成（供 SftpView 开始加载） */
const ready = ref(false);

/** 挂载时建立会话（按连接类型打开 shell / sftp 通道） */
onMounted(async () => {
  try {
    await invoke("ssh_connect", {
      connId: props.tabId,
      connectionId: props.connection.id,
      host: props.connection.host,
      port: props.connection.port,
      username: props.connection.username,
      password: props.connection.password,
      authType: props.connection.authType,
      privateKeyPath: props.connection.privateKeyPath,
      keyPassphrase: props.connection.keyPassphrase,
      name: props.connection.name,
      remark: props.connection.remark,
      sessionType: props.connection.type,
    });
    ready.value = true;
    emit("connected", props.tabId);
  } catch (err) {
    if (disposed) return;
    const msg = typeof err === "string" ? err : ((err as Error)?.message ?? "连接失败");
    emit("error", msg);
    emit("close", props.tabId);
  }
});

/** 卸载时断开会话（后端清理状态与数据泵任务） */
onBeforeUnmount(() => {
  disposed = true;
  invoke("ssh_disconnect", { connId: props.tabId }).catch(() => {});
});
</script>

<style scoped>
/* 背景跟随主题；终端区恒为深色，由 TerminalView 自己覆盖 */
.connection-view {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--color-background);
}
</style>
