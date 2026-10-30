<template>
  <div class="flex h-screen flex-col">
    <TitleBar
      :is-dark="isDark"
      :checking-update="checkingUpdate"
      @home="goHome"
      @toggle-theme="toggleTheme"
      @check-update="handleCheckUpdate"
    />

    <div class="relative flex flex-1 overflow-hidden">
      <Sidebar
        :groups="groups"
        :active-conn="activeConnId"
        :collapsed="sidebarCollapsed"
        @select="handleSelect"
        @add-group="openAddGroup"
        @rename-group="openRenameGroup"
        @delete-group="handleDeleteGroup"
        @add-conn="openAddConn"
        @edit-conn="openEditConn"
        @delete-conn="handleDeleteConn"
        @reorder="handleReorder"
      />

      <!-- 侧边栏收起/展开按钮：展开时紧贴侧边栏右边线，收起时紧贴主界面左边缘 -->
      <div
        class="absolute top-1/2 z-20 flex -translate-y-1/2 items-center justify-center transition-[left] duration-200 ease-in-out"
        :style="{ left: sidebarCollapsed ? '0px' : '239px' }"
      >
        <button
          class="flex h-11 w-5 items-center justify-center rounded border border-border bg-muted/40 text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground"
          :title="sidebarCollapsed ? '展开侧边栏' : '收起侧边栏'"
          @click="sidebarCollapsed = !sidebarCollapsed"
        >
          <ArrowLeft v-if="!sidebarCollapsed" class="size-3.5" />
          <ArrowRight v-else class="size-3.5" />
        </button>
      </div>

      <main class="flex min-w-0 flex-1 flex-col overflow-hidden bg-muted/50">
        <!-- 有 tab 时显示标签栏；无激活 tab（尚未打开或点击 Logo 回首页）时显示仪表盘 -->
        <TabBar
          v-if="tabs.length > 0"
          :tabs="tabs"
          :active-tab-id="activeTabId"
          @select="selectTab"
          @set-sub="setSubTab"
          @close="closeTab"
          @close-left="closeLeft"
          @close-right="closeRight"
          @close-others="closeOthers"
          @close-all="closeAllTabs"
        />
        <div class="flex min-h-0 flex-1 flex-col">
          <Dashboard
            v-if="!activeTabId"
            :groups="groups"
            :recent-connections="recentConnections"
            :connected-count="connectedTabCount"
            @select-recent="handleSelectRecent"
            @clear-recent="handleClearRecent"
          />
          <!-- 所有 tab 常驻挂载（v-show 切换），保活终端与 SFTP 状态 -->
          <ConnectionView
            v-for="tab in tabs"
            :key="tab.tabId"
            v-show="tab.tabId === activeTabId"
            :connection="getConnection(tab.connectionId)!"
            :tab-id="tab.tabId"
            :active="tab.tabId === activeTabId"
            :active-sub="tab.activeSub"
            @close="closeTab"
            @connected="handleConnected"
            @error="handleConnectError"
          />
        </div>
      </main>
    </div>

    <!-- 对话框 -->
    <AddGroupDialog
      v-model="groupDialog.open"
      :initial-name="groupDialog.initName"
      @submit="handleGroupSubmit"
    />
    <AddConnectionDialog
      v-model="connDialog.open"
      :groups="groups"
      :initial="connDialog.target"
      :preset-group-id="connDialog.presetGroupId"
      @submit="handleConnSubmit"
    />

    <!-- 通用确认对话框 -->
    <AlertDialog v-model:open="confirmOpen">
      <AlertDialogContent @escape-key-down="resolveConfirm(false)">
        <AlertDialogHeader>
          <AlertDialogTitle>{{ confirmInfo?.title }}</AlertDialogTitle>
          <AlertDialogDescription>{{ confirmInfo?.message }}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="resolveConfirm(false)">取消</AlertDialogCancel>
          <AlertDialogAction
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            @click="resolveConfirm(true)"
          >
            {{ confirmInfo?.confirmText ?? "确认" }}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

    <!-- 连接失败弹窗 -->
    <AlertDialog v-model:open="connectErrorOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>连接失败</AlertDialogTitle>
          <AlertDialogDescription class="break-all whitespace-pre-wrap">
            {{ connectErrorMessage }}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogAction @click="connectErrorOpen = false">确定</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

    <!-- 主机密钥确认弹窗（首次连接 / 指纹变更） -->
    <AlertDialog :open="!!hostKeyInfo">
      <AlertDialogContent @escape-key-down="respondHostKey(false)">
        <AlertDialogHeader>
          <AlertDialogTitle>{{
            hostKeyInfo?.changed ? "主机密钥已变更" : "确认主机密钥"
          }}</AlertDialogTitle>
          <AlertDialogDescription>
            <template v-if="hostKeyInfo?.changed">
              警告：{{ hostKeyInfo?.host }}:{{ hostKeyInfo?.port }}
              的主机密钥与此前记录不一致，可能是服务器重装，也可能是中间人攻击，请确认后再继续。
            </template>
            <template v-else>
              首次连接 {{ hostKeyInfo?.host }}:{{ hostKeyInfo?.port }}，请核对指纹后再接受。
            </template>
          </AlertDialogDescription>
        </AlertDialogHeader>

        <div class="space-y-2 text-sm">
          <div><span class="text-muted-foreground">密钥类型：</span>{{ hostKeyInfo?.keyType }}</div>
          <div>
            <div class="text-muted-foreground">指纹（SHA256）：</div>
            <div
              class="mt-1 rounded-md border border-border bg-muted/40 p-2 font-mono text-xs break-all"
            >
              {{ hostKeyInfo?.fingerprint }}
            </div>
          </div>
          <div v-if="hostKeyInfo?.changed && hostKeyInfo?.oldFingerprint">
            <div class="text-muted-foreground">此前记录的指纹：</div>
            <div
              class="mt-1 rounded-md border border-destructive/30 bg-destructive/10 p-2 font-mono text-xs break-all text-destructive line-through"
            >
              {{ hostKeyInfo.oldFingerprint }}
            </div>
          </div>
        </div>

        <AlertDialogFooter>
          <AlertDialogCancel @click="respondHostKey(false)">取消</AlertDialogCancel>
          <AlertDialogAction
            :class="
              hostKeyInfo?.changed
                ? 'bg-destructive text-destructive-foreground hover:bg-destructive/90'
                : ''
            "
            @click="respondHostKey(true)"
          >
            {{ hostKeyInfo?.changed ? "仍要信任并更新" : "接受并保存" }}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

    <!-- 更新对话框（下载中不可关闭） -->
    <Dialog v-model:open="updateOpen">
      <DialogContent
        class="sm:max-w-md"
        :show-close-button="updatePhase === 'confirm'"
        @escape-key-down="updatePhase === 'downloading' && $event.preventDefault()"
      >
        <DialogHeader>
          <DialogTitle>发现新版本 v{{ updateInfo?.version }}</DialogTitle>
          <DialogDescription
            v-if="updateInfo?.body"
            class="max-h-48 overflow-y-auto whitespace-pre-wrap rounded-md border border-border bg-muted/40 p-3 text-sm leading-relaxed"
          >
            {{ updateInfo.body }}
          </DialogDescription>
          <DialogDescription v-else class="sr-only">发现新版本</DialogDescription>
        </DialogHeader>
        <div
          v-if="updatePhase === 'downloading'"
          class="flex items-center gap-3 text-sm text-muted-foreground"
        >
          <Loader2 class="size-4 animate-spin" />
          <span>正在下载更新 {{ updateProgress }}%</span>
        </div>
        <DialogFooter v-else>
          <Button variant="outline" @click="updateOpen = false">暂不更新</Button>
          <Button :disabled="checkingUpdate" @click="startUpdate">立即更新</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <Toaster :theme="isDark ? 'dark' : 'light'" position="bottom-right" :rich-colors="true" />
  </div>
</template>

<!--
  应用根组件：
  - 布局：标题栏 + 侧边栏 + 主内容区（tab 栏 + 连接面板 / 仪表盘）
  - 状态：分组/连接数据、tab 管理器（useTabs）、对话框开关、最近连接
  - 职责：数据加载与 CRUD、tab 生命周期编排，具体会话逻辑下沉到 ConnectionView
-->
<script setup lang="ts">
import { computed, markRaw, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { toast } from "vue-sonner";
import { ArrowLeft, ArrowRight, Loader2 } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import TitleBar from "./components/TitleBar.vue";
import Sidebar from "./components/Sidebar.vue";
import Dashboard from "./components/Dashboard.vue";
import ConnectionView from "./components/ConnectionView.vue";
import TabBar from "./components/TabBar.vue";
import AddGroupDialog from "./components/AddGroupDialog.vue";
import AddConnectionDialog from "./components/AddConnectionDialog.vue";
import { Button } from "./components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./components/ui/alert-dialog";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./components/ui/dialog";
import { Toaster } from "./components/ui/sonner";
import {
  clearRecent,
  createConnection,
  createGroup,
  deleteConnection,
  deleteGroup,
  listGroups,
  listRecent,
  notifyError,
  renameGroup,
  saveTreeOrder,
  updateConnection,
} from "./api";
import {
  isUngroupedGroup,
  subTabsForType,
  type AddConnInput,
  type Connection,
  type Group,
  type RecentItem,
  type TreeOrder,
} from "./types";
import { useTabs } from "./composables/useTabs";
import { useTheme } from "./composables/useTheme";

// ---------- 全局 UI 状态 ----------
const { isDark, toggleTheme } = useTheme();
const sidebarCollapsed = ref(false);

// ---------- 数据状态 ----------
const groups = ref<Group[]>([]);
const recentConnections = ref<RecentItem[]>([]);

// ---------- tab 管理器 ----------
const {
  tabs,
  activeTabId,
  activeConnId,
  getConnection,
  handleSelect,
  selectTab,
  goHome,
  setSubTab,
  closeTab,
  closeLeft,
  closeRight,
  closeOthers,
  closeAllTabs,
  removeTabs,
} = useTabs(groups, sidebarCollapsed);

/** 已建立会话的 tab 数（仪表盘“已连接”统计） */
const connectedTabCount = computed(() => tabs.value.filter((t) => t.connected).length);

// ---------- 对话框状态 ----------
/** 分组对话框：open 是否显示；targetId 为空表示新增，否则重命名该分组 */
const groupDialog = ref<{ open: boolean; targetId: string | null; initName: string }>({
  open: false,
  targetId: null,
  initName: "",
});
/** 连接对话框：target 为空表示新增，否则编辑该连接 */
const connDialog = ref<{ open: boolean; target: Connection | null; presetGroupId: string }>({
  open: false,
  target: null,
  presetGroupId: "",
});

// ---------- 确认对话框 ----------
const confirmOpen = ref(false);
const confirmInfo = ref<{
  title: string;
  message: string;
  confirmText: string;
  resolve: (ok: boolean) => void;
  resolved: boolean;
} | null>(null);

function confirmDialog(title: string, message: string, confirmText = "确认"): Promise<boolean> {
  return new Promise((resolve) => {
    confirmInfo.value = { title, message, confirmText, resolve, resolved: false };
    confirmOpen.value = true;
  });
}

function resolveConfirm(ok: boolean) {
  const info = confirmInfo.value;
  if (info && !info.resolved) {
    info.resolved = true;
    info.resolve(ok);
  }
  confirmOpen.value = false;
}

// ---------- 连接失败弹窗 ----------
const connectErrorOpen = ref(false);
const connectErrorMessage = ref("");

/** 会话建立成功：标记对应 tab 已连接，并刷新最近连接 */
function handleConnected(tabId: string) {
  const tab = tabs.value.find((t) => t.tabId === tabId);
  if (tab) tab.connected = true;
  refreshRecent();
}

/** 会话建立失败：弹窗展示错误（标签已由 ConnectionView 关闭） */
function handleConnectError(message: string) {
  connectErrorMessage.value = message;
  connectErrorOpen.value = true;
}

// ---------- 主机密钥确认弹窗 ----------
interface HostKeyVerify {
  connId: string;
  host: string;
  port: number;
  keyType: string;
  fingerprint: string;
  publicKey: string;
  changed: boolean;
  oldFingerprint: string | null;
}

const hostKeyInfo = ref<HostKeyVerify | null>(null);

/** 回应用户决策（接受并保存 / 取消） */
function respondHostKey(accept: boolean) {
  const info = hostKeyInfo.value;
  if (!info) return;
  hostKeyInfo.value = null;
  invoke("ssh_confirm_host_key", { connId: info.connId, accept }).catch(() => {});
}

// ---------- 数据加载 ----------
async function refresh() {
  try {
    const next = await listGroups();
    const openState = new Map(groups.value.map((g) => [g.id, g.open]));
    for (const group of next) {
      group.open = openState.get(group.id) ?? isUngroupedGroup(group);
    }
    groups.value = next;
  } catch (e) {
    notifyError(e);
  }
}

async function refreshRecent() {
  try {
    recentConnections.value = await listRecent();
  } catch (e) {
    notifyError(e);
  }
}

// ---------- 键盘快捷键 ----------
/** Ctrl+Tab / Ctrl+Shift+Tab：切换当前连接 tab 的 SSH/SFTP 子视图 */
function cycleSubTab(direction: 1 | -1) {
  if (!activeTabId.value) return;
  const tab = tabs.value.find((t) => t.tabId === activeTabId.value);
  if (!tab) return;
  const subs = subTabsForType(tab.connectionType);
  if (subs.length < 2) return;
  const idx = subs.indexOf(tab.activeSub);
  setSubTab(tab.tabId, subs[(idx + direction + subs.length) % subs.length]);
}

function onWindowKeydown(e: KeyboardEvent) {
  if (e.ctrlKey && !e.altKey && !e.metaKey && e.key === "Tab") {
    e.preventDefault();
    cycleSubTab(e.shiftKey ? -1 : 1);
  }
}

let unlistenHostKey: UnlistenFn | null = null;

onMounted(async () => {
  refresh();
  refreshRecent();
  window.addEventListener("keydown", onWindowKeydown, true);
  // 首次连接/指纹变更时后端发起的主机密钥确认
  unlistenHostKey = await listen<HostKeyVerify>("ssh-host-key-verify", (e) => {
    hostKeyInfo.value = e.payload;
  });
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onWindowKeydown, true);
  unlistenHostKey?.();
});

// 全部 tab 关闭回到仪表盘时刷新最近连接
watch(
  () => tabs.value.length,
  (n) => {
    if (n === 0) refreshRecent();
  },
);

// ---------- 分组操作 ----------
function openAddGroup() {
  groupDialog.value = { open: true, targetId: null, initName: "" };
}

function openRenameGroup(id: string, name: string) {
  groupDialog.value = { open: true, targetId: id, initName: name };
}

async function handleGroupSubmit(name: string) {
  const targetId = groupDialog.value.targetId;
  try {
    if (targetId) {
      await renameGroup(targetId, name);
      toast.success("分组已重命名");
    } else {
      await createGroup(name);
      toast.success("分组已创建");
    }
    await refresh();
  } catch (e) {
    notifyError(e);
  }
}

async function handleDeleteGroup(id: string, name: string) {
  const ok = await confirmDialog("删除分组", `确定删除分组「${name}」及其所有连接吗？`, "删除");
  if (!ok) return;
  try {
    await deleteGroup(id);
    toast.success("分组已删除");
    await refresh();
    // 清理已删除连接对应的打开 tab
    const ids = new Set<string>();
    for (const g of groups.value) for (const c of g.connections) ids.add(c.id);
    removeTabs((t) => !ids.has(t.connectionId));
  } catch (e) {
    notifyError(e);
  }
}

// ---------- 连接操作 ----------
function openAddConn(groupId?: string) {
  connDialog.value = { open: true, target: null, presetGroupId: groupId ?? "" };
}

function openEditConn(conn: Connection) {
  connDialog.value = { open: true, target: conn, presetGroupId: "" };
}

async function handleConnSubmit(input: AddConnInput) {
  try {
    const target = connDialog.value.target;
    if (target) {
      await updateConnection(target.id, input);
      toast.success("连接已更新");
    } else {
      await createConnection(input);
      toast.success("连接已创建");
    }
    await refresh();
  } catch (e) {
    notifyError(e);
  }
}

async function handleDeleteConn(id: string, name: string) {
  const ok = await confirmDialog("删除连接", `确定删除连接「${name}」吗？`, "删除");
  if (!ok) return;
  try {
    await deleteConnection(id);
    toast.success("连接已删除");
    await refresh();
    removeTabs((t) => t.connectionId === id);
  } catch (e) {
    notifyError(e);
  }
}

/** 拖拽结束：保存分组/连接顺序，再刷新以拿到后端持久化结果 */
async function handleReorder(order: TreeOrder) {
  try {
    await saveTreeOrder(order);
  } catch (e) {
    notifyError(e);
  } finally {
    await refresh();
  }
}

// ---------- 最近连接操作 ----------
/** 点击最近连接：连接仍存在则打开，否则提示 */
function handleSelectRecent(id: string) {
  const conn = getConnection(id);
  if (!conn) {
    toast.warning("该连接已不存在，可能已被删除");
    return;
  }
  handleSelect(id);
}

// ---------- 检查更新 ----------
const checkingUpdate = ref(false);
const updateOpen = ref(false);
const updatePhase = ref<"confirm" | "downloading">("confirm");
const updateProgress = ref(0);
/** Update 实例使用 WeakMap 私有字段，不能被 Vue 响应式代理包裹（否则调用 update 方法时报错） */
const updateInfo = ref<{ version: string; body?: string; update: Update } | null>(null);

/** 判断当前是否为 Windows（Windows 下安装完成后应用会自动退出运行安装器） */
function isWindowsPlatform() {
  return navigator.userAgent.toLowerCase().includes("win");
}

async function handleCheckUpdate() {
  if (checkingUpdate.value) return;
  checkingUpdate.value = true;
  try {
    const update = await check();
    if (!update) {
      toast.success("当前已是最新版本");
      return;
    }
    updateInfo.value = {
      version: update.version,
      body: update.body ?? undefined,
      update: markRaw(update),
    };
    updatePhase.value = "confirm";
    updateOpen.value = true;
  } catch (e) {
    notifyError(e);
  } finally {
    checkingUpdate.value = false;
  }
}

async function startUpdate() {
  const info = updateInfo.value;
  if (!info) return;
  updatePhase.value = "downloading";
  updateProgress.value = 0;
  let downloaded = 0;
  let total = 0;
  try {
    await info.update.downloadAndInstall((event) => {
      if (event.event === "Started") {
        total = event.data.contentLength ?? 0;
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        if (total > 0) {
          updateProgress.value = Math.min(100, Math.round((downloaded / total) * 100));
        }
      }
    });
    // Windows 下 install 会自动退出应用并启动安装器；macOS/Linux 需手动重启
    if (!isWindowsPlatform()) {
      toast.success("更新完成，正在重启应用…");
      await relaunch();
    }
    updateOpen.value = false;
  } catch (e) {
    updateOpen.value = false;
    notifyError(e);
  }
}

async function handleClearRecent() {
  try {
    await clearRecent();
    recentConnections.value = [];
    toast.success("最近连接已清空");
  } catch (e) {
    notifyError(e);
  }
}
</script>
