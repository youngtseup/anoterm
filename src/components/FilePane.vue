<template>
  <div class="flex h-full min-h-0 min-w-0 flex-col bg-muted/50">
    <div class="flex flex-wrap items-center gap-2 border-b border-border bg-background px-3 py-2">
      <span
        v-if="layout === 'dual'"
        class="shrink-0 text-[11px] leading-none text-muted-foreground"
      >
        {{ kind === "local" ? "本地" : "远端" }}
      </span>
      <Button variant="outline" size="icon-sm" :disabled="isAtRoot" title="上级目录" @click="goUp">
        <ArrowUp class="size-4" />
      </Button>
      <Button variant="outline" size="icon-sm" title="刷新" @click="refresh">
        <RefreshCw class="size-4" />
      </Button>

      <div
        class="relative min-w-[120px] flex-1"
        :class="layout === 'single' ? 'max-w-[480px]' : ''"
      >
        <HardDrive
          v-if="kind === 'local'"
          class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
        />
        <Folder
          v-else
          class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
        />
        <Input
          v-model="pathInput"
          class="h-8 pr-8 pl-8"
          :placeholder="kind === 'local' ? '输入本地路径，回车跳转' : '输入路径，回车跳转'"
          @keyup.enter="goToPath"
          @focus="onPathFocus"
        />
        <button
          v-if="pathInput"
          type="button"
          class="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
          @click="pathInput = ''"
        >
          <X class="size-4" />
        </button>
      </div>

      <div v-if="layout === 'single'" class="flex-1"></div>

      <Button
        v-if="layout === 'dual'"
        variant="outline"
        size="icon-sm"
        title="新建文件夹"
        @click="mkdir"
      >
        <FolderPlus class="size-4" />
      </Button>
      <Button v-else variant="outline" size="sm" @click="mkdir">
        <FolderPlus class="size-4" />
        新建文件夹
      </Button>

      <Button
        v-if="kind === 'remote' && layout === 'single'"
        size="sm"
        :disabled="!!transferState"
        @click="uploadFromDialog"
      >
        <Loader2 v-if="transferState" class="size-4 animate-spin" />
        <Upload v-else class="size-4" />
        上传
      </Button>
      <Button v-else size="sm" :disabled="transferDisabled" @click="requestTransfer">
        <Loader2 v-if="busy" class="size-4 animate-spin" />
        <Upload v-else-if="kind === 'local'" class="size-4" />
        <Download v-else class="size-4" />
        {{ kind === "local" ? "上传" : "下载" }}
      </Button>
    </div>

    <TransferBar
      v-if="transferState"
      :current="transferState.current"
      :transferred="transferState.transferred"
      :total="transferState.total"
      :cancelling="transferCancelling"
      @cancel="cancelTransfer"
    />

    <div class="relative min-h-0 flex-1">
      <div class="h-full overflow-auto">
        <table class="w-full caption-bottom text-sm">
          <TableHeader class="sticky top-0 z-10 bg-muted/95 backdrop-blur">
            <TableRow class="hover:bg-transparent">
              <TableHead :class="layout === 'single' ? 'min-w-[260px]' : 'min-w-[140px]'"
                >名称</TableHead
              >
              <TableHead :class="layout === 'single' ? 'w-[100px]' : 'w-[80px]'">大小</TableHead>
              <TableHead :class="layout === 'single' ? 'w-[150px]' : 'w-[130px]'"
                >修改时间</TableHead
              >
              <TableHead
                v-if="kind === 'remote' && layout === 'single'"
                class="w-[180px] text-right"
                >操作</TableHead
              >
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="(row, index) in entries"
              :key="row.path"
              class="cursor-default"
              :class="
                layout === 'dual' && isSelected(row)
                  ? 'bg-primary/10 hover:bg-primary/10'
                  : 'hover:bg-muted/50'
              "
              @click="onRowClick($event, row, index)"
              @dblclick="onRowDblclick(row)"
              @contextmenu.prevent="onRowContextmenu($event, row, index)"
            >
              <TableCell>
                <div class="flex items-center gap-1.5">
                  <Folder v-if="row.isDir" class="size-4 shrink-0 text-amber-500" />
                  <File v-else class="size-4 shrink-0 text-muted-foreground" />
                  <span class="truncate">{{ row.name }}</span>
                </div>
              </TableCell>
              <TableCell class="text-muted-foreground">{{ formatSize(row) }}</TableCell>
              <TableCell class="text-muted-foreground">{{ formatTime(row) }}</TableCell>
              <TableCell v-if="kind === 'remote' && layout === 'single'" class="text-right">
                <div class="flex justify-end gap-1">
                  <Button
                    variant="link"
                    size="xs"
                    class="h-auto px-1.5"
                    @click="downloadViaDialog(row)"
                  >
                    {{ row.isDir ? "下载目录" : "下载" }}
                  </Button>
                  <Button variant="link" size="xs" class="h-auto px-1.5" @click="renameEntry(row)">
                    重命名
                  </Button>
                  <Button
                    variant="link"
                    size="xs"
                    class="h-auto px-1.5 text-destructive hover:text-destructive"
                    @click="removeEntry(row)"
                  >
                    删除
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          </TableBody>
        </table>

        <div
          v-if="!loading && entries.length === 0"
          class="px-5 py-16 text-center text-[13px] text-muted-foreground"
        >
          空目录
        </div>
      </div>

      <!-- 加载遮罩 -->
      <div
        v-if="loading"
        class="absolute inset-0 z-10 flex items-center justify-center bg-background/50"
      >
        <Loader2 class="size-6 animate-spin text-muted-foreground" />
      </div>
    </div>

    <!-- 行右键菜单 -->
    <DropdownMenu v-model:open="menuOpen">
      <DropdownMenuTrigger as-child>
        <div class="fixed size-0" :style="{ left: `${menuPos.x}px`, top: `${menuPos.y}px` }" />
      </DropdownMenuTrigger>
      <DropdownMenuContent side="right" align="start" :side-offset="0" class="min-w-36">
        <DropdownMenuItem v-if="menuEntry?.isDir" @click="doCtx('enter')"
          >进入文件夹</DropdownMenuItem
        >
        <DropdownMenuItem v-if="layout === 'dual'" @click="doCtx('transfer')">
          {{ transferMenuLabel }}
        </DropdownMenuItem>
        <DropdownMenuItem v-else @click="doCtx('download')">下载</DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem @click="doCtx('rename')">重命名</DropdownMenuItem>
        <DropdownMenuItem variant="destructive" @click="doCtx('remove')">删除</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>

    <!-- 输入对话框（新建文件夹/重命名） -->
    <Dialog v-model:open="promptOpen">
      <DialogContent class="sm:max-w-[380px]">
        <DialogHeader>
          <DialogTitle>{{ promptState?.title }}</DialogTitle>
          <DialogDescription class="sr-only">{{ promptState?.title }}</DialogDescription>
        </DialogHeader>
        <Input
          v-model="promptValue"
          :class="promptError && 'border-destructive focus-visible:ring-destructive/30'"
          @keyup.enter="confirmPrompt"
        />
        <p v-if="promptError" class="text-[13px] text-destructive">{{ promptError }}</p>
        <DialogFooter>
          <Button variant="outline" @click="promptOpen = false">取消</Button>
          <Button @click="confirmPrompt">{{ promptState?.okText ?? "确定" }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- 删除确认对话框 -->
    <Dialog v-model:open="deleteOpen">
      <DialogContent class="sm:max-w-[380px]">
        <DialogHeader>
          <DialogTitle>删除</DialogTitle>
          <DialogDescription>{{ deleteDescription }}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="deleteOpen = false">取消</Button>
          <Button variant="destructive" @click="doDelete">删除</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
/**
 * 通用文件面板（本地 / 远端）：
 * - 远端面板会话由 ConnectionView 建立（ready 后才加载），本组件只调用 sftp_* 命令
 * - 双栏模式：行多选（Ctrl/Shift），工具栏「上传/下载」或双击文件把选中项交给宿主传输
 * - 单栏模式（仅远端）：保留系统对话框上传/下载与行内操作链接
 * - 通用操作：地址栏跳转、新建文件夹、重命名、删除、右键菜单
 */
import { computed, onMounted, ref, watch } from "vue";
import { toast } from "vue-sonner";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  ArrowUp,
  Download,
  File,
  Folder,
  FolderPlus,
  HardDrive,
  Loader2,
  RefreshCw,
  Upload,
  X,
} from "@lucide/vue";
import type { SftpEntry } from "../types";
import type { SftpLayout } from "../composables/useSftpLayout";
import { useSftpTransfer } from "../composables/useSftpTransfer";
import {
  localHome,
  localList,
  localMkdir,
  localRemove,
  localRename,
  notifyError,
  sftpList,
  sftpMkdir,
  sftpRemove,
  sftpRename,
} from "../api";
import {
  isRootPath,
  joinLocal,
  joinRemote,
  localParent,
  normalizeLocalPath,
  remoteParent,
} from "../lib/paths";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { Input } from "./ui/input";
import { TableBody, TableCell, TableHead, TableHeader, TableRow } from "./ui/table";
import TransferBar from "./TransferBar.vue";

const props = defineProps<{
  kind: "local" | "remote";
  layout: SftpLayout;
  /** 远端会话 id（kind=remote 必传） */
  tabId?: string;
  /** 远端会话是否就绪（kind=remote） */
  ready?: boolean;
  /** 双栏传输进行中 */
  busy?: boolean;
}>();

const emit = defineEmits<{
  /** 请求把选中项传输到另一侧（双栏模式） */
  transfer: [entries: SftpEntry[]];
}>();

/** 当前目录 */
const cwd = ref("");
/** 当前目录内容 */
const entries = ref<SftpEntry[]>([]);
const loading = ref(false);
/** 单栏模式上传/下载（系统对话框）的进度与取消 */
const {
  state: transferState,
  cancelling: transferCancelling,
  start: startTransfer,
  cancel: cancelTransfer,
} = useSftpTransfer();
/** 地址栏输入值 */
const pathInput = ref("");
/** 双栏模式多选（按完整路径） */
const selected = ref<Set<string>>(new Set());
/** Shift 连选的锚点下标 */
const lastIndex = ref<number | null>(null);

const isAtRoot = computed(() => isRootPath(props.kind, cwd.value));
const selectedEntries = computed(() => entries.value.filter((e) => selected.value.has(e.path)));
const hasSelection = computed(() => selectedEntries.value.length > 0);
const transferDisabled = computed(() => !hasSelection.value || !!props.busy || !props.ready);

/** 当前条目的子路径（新建/重命名用） */
function childPath(name: string): string {
  return props.kind === "local" ? joinLocal(cwd.value, name) : joinRemote(cwd.value, name);
}

function clearSelection() {
  selected.value = new Set();
  lastIndex.value = null;
}

/**
 * 加载指定目录：成功才更新 cwd/entries（地址跳转失败时留在原目录）。
 */
async function load(path: string) {
  if (!path) return;
  loading.value = true;
  try {
    const list =
      props.kind === "local" ? await localList(path) : await sftpList(props.tabId!, path);
    cwd.value = path;
    pathInput.value = path;
    entries.value = list;
    clearSelection();
  } catch (e) {
    notifyError(e);
  } finally {
    loading.value = false;
  }
}

/** 地址栏聚焦时全选，方便直接输入新路径 */
function onPathFocus(ev: FocusEvent) {
  (ev.target as HTMLInputElement).select();
}

/** 地址栏回车：跳转到输入路径 */
function goToPath() {
  const raw = pathInput.value.trim();
  if (!raw) return;
  const target =
    props.kind === "local" ? normalizeLocalPath(raw) : raw.startsWith("/") ? raw : "/" + raw;
  if (target === cwd.value) return;
  load(target);
}

async function refresh() {
  await load(cwd.value);
}

/** 双击进入文件夹 */
function enterDir(entry: SftpEntry) {
  if (!entry.isDir) return;
  load(entry.path);
}

/** 返回上级目录 */
function goUp() {
  if (!cwd.value) return;
  load(props.kind === "local" ? localParent(cwd.value) : remoteParent(cwd.value));
}

// ---------- 选择（双栏模式） ----------
function isSelected(row: SftpEntry): boolean {
  return selected.value.has(row.path);
}

function onRowClick(ev: MouseEvent, row: SftpEntry, index: number) {
  if (props.layout !== "dual") return;
  const next = new Set(selected.value);
  if (ev.shiftKey && lastIndex.value !== null) {
    const [from, to] = [lastIndex.value, index].sort((a, b) => a - b);
    for (let i = from; i <= to; i++) next.add(entries.value[i].path);
  } else if (ev.ctrlKey || ev.metaKey) {
    if (next.has(row.path)) next.delete(row.path);
    else next.add(row.path);
    lastIndex.value = index;
  } else {
    next.clear();
    next.add(row.path);
    lastIndex.value = index;
  }
  selected.value = next;
}

/** 双击：文件夹进入；双栏模式下文件直接传输到另一侧 */
function onRowDblclick(row: SftpEntry) {
  if (row.isDir) return enterDir(row);
  if (props.layout === "dual") emit("transfer", [row]);
}

/** 工具栏「上传/下载」：把选中项交给宿主 */
function requestTransfer() {
  if (transferDisabled.value) return;
  emit("transfer", selectedEntries.value);
}

// ---------- 输入对话框（新建文件夹/重命名） ----------
const promptOpen = ref(false);
const promptValue = ref("");
const promptError = ref("");
const promptState = ref<{
  title: string;
  okText: string;
  handler: (v: string) => Promise<void>;
} | null>(null);

function promptInput(opts: {
  title: string;
  initial?: string;
  okText: string;
  handler: (v: string) => Promise<void>;
}) {
  promptState.value = { title: opts.title, okText: opts.okText, handler: opts.handler };
  promptValue.value = opts.initial ?? "";
  promptError.value = "";
  promptOpen.value = true;
}

function confirmPrompt() {
  const p = promptState.value;
  if (!p) return;
  const value = promptValue.value.trim();
  if (!value) {
    promptError.value = "名称不能为空";
    return;
  }
  promptOpen.value = false;
  p.handler(value).catch((e) => notifyError(e));
}

/** 新建文件夹（当前目录下） */
function mkdir() {
  promptInput({
    title: "新建文件夹",
    okText: "创建",
    handler: async (value) => {
      const path = childPath(value);
      if (props.kind === "local") await localMkdir(path);
      else await sftpMkdir(props.tabId!, path);
      toast.success("已创建");
      await refresh();
    },
  });
}

/** 重命名条目 */
function renameEntry(entry: SftpEntry) {
  promptInput({
    title: "重命名",
    initial: entry.name,
    okText: "确定",
    handler: async (value) => {
      const newPath = childPath(value);
      if (props.kind === "local") await localRename(entry.path, newPath);
      else await sftpRename(props.tabId!, entry.path, newPath);
      toast.success("已重命名");
      await refresh();
    },
  });
}

// ---------- 删除确认 ----------
const deleteOpen = ref(false);
const deleteTargets = ref<SftpEntry[]>([]);

const deleteDescription = computed(() => {
  const list = deleteTargets.value;
  if (list.length > 1) return `确定删除选中的 ${list.length} 项吗？目录及其内容将被删除。`;
  const entry = list[0];
  if (!entry) return "确定删除吗？";
  return `确定删除「${entry.name}」吗？${entry.isDir ? "目录及其内容将被删除。" : ""}`;
});

/** 双栏模式下删除整组选中项，单栏模式仅当前行 */
function removeEntry(entry: SftpEntry) {
  deleteTargets.value =
    props.layout === "dual" && isSelected(entry) && selectedEntries.value.length > 1
      ? selectedEntries.value
      : [entry];
  deleteOpen.value = true;
}

async function doDelete() {
  const targets = deleteTargets.value;
  if (targets.length === 0) return;
  deleteOpen.value = false;
  try {
    for (const entry of targets) {
      if (props.kind === "local") await localRemove(entry.path, entry.isDir);
      else await sftpRemove(props.tabId!, entry.path, entry.isDir);
    }
    toast.success(targets.length > 1 ? `已删除 ${targets.length} 项` : "已删除");
    await refresh();
  } catch (e) {
    notifyError(e);
  }
}

// ---------- 单栏模式：系统对话框下载 / 上传 ----------
/** 下载：文件选保存位置；文件夹选目录后递归下载（带进度/取消） */
async function downloadViaDialog(entry: SftpEntry) {
  if (props.kind !== "remote" || transferState.value) return;
  try {
    let dest: string;
    if (entry.isDir) {
      const dir = await open({ directory: true, title: "选择保存位置" });
      if (!dir) return;
      dest = dir;
    } else {
      const target = await save({ defaultPath: entry.name });
      if (!target) return;
      dest = target;
    }
    const outcome = await startTransfer(props.tabId!, "download", [
      { source: entry.path, dest, isDir: entry.isDir },
    ]);
    if (outcome === "done") {
      toast.success(entry.isDir ? "文件夹下载完成" : "下载完成");
    } else {
      toast.info("已取消下载");
    }
  } catch (e) {
    notifyError(e);
  }
}

/** 上传单个/多个文件到当前目录（单栏模式，带进度/取消） */
async function uploadFromDialog() {
  if (props.kind !== "remote" || transferState.value) return;
  try {
    const selectedPaths = await open({
      multiple: true,
      directory: false,
      title: "选择要上传的文件",
    });
    const files = Array.isArray(selectedPaths)
      ? selectedPaths
      : selectedPaths
        ? [selectedPaths]
        : [];
    if (files.length === 0) return;
    const items = files.map((f) => {
      const name = f.split(/[\\/]/).pop() ?? f;
      return { source: f, dest: joinRemote(cwd.value, name), isDir: false };
    });
    const outcome = await startTransfer(props.tabId!, "upload", items);
    if (outcome === "done") toast.success("上传完成");
    else toast.info("已取消上传");
    await refresh();
  } catch (e) {
    notifyError(e);
  }
}

// ---------- 右键菜单 ----------
const menuOpen = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const menuEntry = ref<SftpEntry | null>(null);

const transferMenuLabel = computed(() => {
  const entry = menuEntry.value;
  if (props.kind === "local") return entry?.isDir ? "上传文件夹" : "上传到右侧";
  return entry?.isDir ? "下载文件夹" : "下载到左侧";
});

function onRowContextmenu(ev: MouseEvent, row: SftpEntry, index: number) {
  if (props.layout === "dual" && !selected.value.has(row.path)) {
    selected.value = new Set([row.path]);
    lastIndex.value = index;
  }
  menuEntry.value = row;
  menuPos.value = { x: ev.clientX, y: ev.clientY };
  menuOpen.value = true;
}

/** 右键「传输」：右键行在选中集内则传全部选中项 */
function transferFromContext(entry: SftpEntry) {
  const list = selected.value.has(entry.path) ? selectedEntries.value : [entry];
  emit("transfer", list);
}

/** 执行右键菜单动作 */
function doCtx(action: "enter" | "transfer" | "download" | "rename" | "remove") {
  const entry = menuEntry.value;
  menuEntry.value = null;
  if (!entry) return;
  if (action === "enter") enterDir(entry);
  else if (action === "transfer") transferFromContext(entry);
  else if (action === "download") downloadViaDialog(entry);
  else if (action === "rename") renameEntry(entry);
  else removeEntry(entry);
}

/** 文件大小格式化 */
function formatSize(entry: SftpEntry): string {
  if (entry.isDir) return "—";
  const size = entry.size ?? 0;
  if (size < 1024) return size + " B";
  if (size < 1024 * 1024) return (size / 1024).toFixed(1) + " KB";
  if (size < 1024 * 1024 * 1024) return (size / (1024 * 1024)).toFixed(1) + " MB";
  return (size / (1024 * 1024 * 1024)).toFixed(2) + " GB";
}

/** 修改时间格式化 */
function formatTime(entry: SftpEntry): string {
  if (!entry.mtime) return "—";
  const d = new Date(entry.mtime * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

// 本地栏挂载后进入用户主目录
onMounted(async () => {
  if (props.kind !== "local") return;
  try {
    await load(await localHome());
  } catch (e) {
    notifyError(e);
    await load(navigator.userAgent.toLowerCase().includes("win") ? "C:\\" : "/");
  }
});

// 切回单栏时清空双栏选择
watch(
  () => props.layout,
  (value) => {
    if (value === "single") clearSelection();
  },
);

// 远端会话就绪后加载根目录
watch(
  () => props.ready,
  (r) => {
    if (props.kind === "remote" && r) load("/");
  },
  { immediate: true },
);

defineExpose({ cwd, refresh });
</script>
