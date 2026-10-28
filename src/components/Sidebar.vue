<template>
  <aside
    class="flex flex-col overflow-hidden border-border bg-muted/50 transition-[width,min-width] duration-200 ease-in-out"
    :class="collapsed ? 'w-0 min-w-0' : 'w-[260px] min-w-[260px] border-r'"
  >
    <div class="px-4 pt-3 pb-1">
      <Button size="sm" class="w-full" @click="emit('add-conn')">
        <Plug class="size-4" />
        新增连接
      </Button>
    </div>

    <div
      class="group-header-row relative flex items-center px-4 pt-1.5 pb-2.5"
      data-drop-root
      :class="{ 'drop-root': dropTarget?.kind === 'root' }"
    >
      <span class="text-[13px] font-semibold tracking-wide text-muted-foreground uppercase">
        连接管理
      </span>
      <button
        type="button"
        class="ml-auto flex size-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        title="新增分组"
        aria-label="新增分组"
        @click="emit('add-group')"
      >
        <FolderPlus class="size-4" />
      </button>
    </div>

    <div class="sidebar-content flex-1 overflow-y-auto px-2 pb-2">
      <div v-if="ungroupedConnections.length > 0" class="mb-1">
        <div
          class="connection-root-item relative flex cursor-pointer items-center gap-2 rounded-md px-2.5 py-2 transition-colors select-none hover:bg-card"
          :class="[
            activeConn === conn.id ? 'bg-card shadow-sm' : '',
            connDropClass(conn.id),
            { 'drag-source': isConnDragSource(conn.id) },
          ]"
          v-for="conn in ungroupedConnections"
          :key="`root-${conn.id}`"
          :data-conn-id="conn.id"
          :data-parent-group="UNGROUPED_GROUP_ID"
          title="点击打开，长按拖动"
          @click="handleConnClick(conn)"
          @pointerdown="
            onPointerDown($event, {
              kind: 'conn',
              connId: conn.id,
              groupId: UNGROUPED_GROUP_ID,
            })
          "
          @contextmenu.prevent="onConnContextMenu($event, conn)"
        >
          <span class="size-[7px] shrink-0 rounded-full bg-muted-foreground"></span>
          <span
            class="truncate text-[13px]"
            :class="activeConn === conn.id ? 'font-medium' : 'text-muted-foreground'"
          >
            {{ conn.name }}
          </span>
        </div>
      </div>

      <div class="mb-0.5" v-for="group in realGroups" :key="group.id">
        <div
          class="group-row relative flex cursor-pointer items-center gap-2 rounded-md px-2.5 py-2 transition-colors select-none hover:bg-card"
          :class="[groupDropClass(group.id), { 'drag-source': isGroupDragSource(group.id) }]"
          :data-group-id="group.id"
          title="点击展开/收起，长按拖动排序"
          @click="handleGroupClick(group)"
          @pointerdown="onPointerDown($event, { kind: 'group', groupId: group.id })"
          @contextmenu.prevent="onGroupContextMenu($event, group)"
        >
          <ChevronRight
            class="size-3.5 text-muted-foreground transition-transform duration-200"
            :class="{ 'rotate-90': group.open }"
          />
          <span class="flex-1 truncate text-[13.5px] font-medium">{{ group.name }}</span>
          <span class="rounded-full bg-background px-1.5 text-[11px] text-muted-foreground">
            {{ group.connections.length }}
          </span>
        </div>

        <div
          class="connection-list relative ml-2.5 overflow-hidden pl-2.5 transition-[max-height] duration-200"
          :class="group.open ? 'max-h-[800px]' : 'max-h-0'"
        >
          <div
            class="connection-item relative flex cursor-pointer items-center gap-2 rounded-md px-2.5 py-[7px] transition-colors select-none hover:bg-card"
            :class="[
              activeConn === conn.id ? 'bg-card shadow-sm' : '',
              connDropClass(conn.id),
              { 'drag-source': isConnDragSource(conn.id) },
            ]"
            v-for="conn in group.connections"
            :key="conn.id"
            :data-conn-id="conn.id"
            :data-parent-group="group.id"
            title="点击打开，长按拖动"
            @click="handleConnClick(conn)"
            @pointerdown="
              onPointerDown($event, {
                kind: 'conn',
                connId: conn.id,
                groupId: group.id,
              })
            "
            @contextmenu.prevent="onConnContextMenu($event, conn)"
          >
            <span class="size-[7px] shrink-0 rounded-full bg-muted-foreground"></span>
            <span
              class="truncate text-[13px]"
              :class="activeConn === conn.id ? 'font-medium' : 'text-muted-foreground'"
            >
              {{ conn.name }}
            </span>
          </div>
        </div>
      </div>

      <div
        v-if="realGroups.length === 0 && ungroupedConnections.length === 0"
        class="px-3 py-6 text-center text-[13px] text-muted-foreground"
      >
        暂无连接，点击「新增连接」创建
      </div>
    </div>

    <!-- 右键菜单（锚定在鼠标坐标的零尺寸触发元素上） -->
    <DropdownMenu v-model:open="menuOpen">
      <DropdownMenuTrigger as-child>
        <div class="fixed size-0" :style="{ left: `${menuPos.x}px`, top: `${menuPos.y}px` }" />
      </DropdownMenuTrigger>
      <DropdownMenuContent side="right" align="start" :side-offset="0" class="min-w-36">
        <DropdownMenuItem
          v-for="item in menuItems"
          :key="item.label"
          :variant="item.danger ? 'destructive' : 'default'"
          @click="runMenuCommand(item.command)"
        >
          {{ item.label }}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  </aside>
</template>

<script setup lang="ts">
/**
 * 侧边栏：分组树形展示连接。
 * - 分组可展开/收起，连接以树形缩进展示（每行一个连接）
 * - 长按左键拖动分组排序，或拖动连接在组内/跨组/顶层排序
 * - 未归属分组的连接直接显示在所有分组上方的树顶层
 * - 分组/连接右键菜单使用 DropdownMenu（受控 open + 鼠标坐标锚点）
 */
import { computed, onBeforeUnmount, ref } from "vue";
import { ChevronRight, FolderPlus, Plug } from "@lucide/vue";
import {
  UNGROUPED_GROUP_ID,
  isUngroupedGroup,
  type Connection,
  type ContextMenuItem,
  type Group,
  type TreeOrder,
} from "../types";
import { Button } from "./ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";

const props = defineProps<{
  groups: Group[];
  activeConn: string | null;
  collapsed?: boolean;
}>();

const emit = defineEmits<{
  select: [id: string];
  "add-group": [];
  "rename-group": [id: string, name: string];
  "delete-group": [id: string, name: string];
  "add-conn": [groupId?: string];
  "edit-conn": [conn: Connection];
  "delete-conn": [id: string, name: string];
  reorder: [order: TreeOrder];
}>();

// ---------- 右键菜单状态 ----------
const menuOpen = ref(false);
const menuPos = ref({ x: 0, y: 0 });
const menuItems = ref<ContextMenuItem[]>([]);
let menuRunner: ((cmd: string) => void) | null = null;

function openMenu(e: MouseEvent, items: ContextMenuItem[], runner: (cmd: string) => void) {
  menuItems.value = items;
  menuRunner = runner;
  menuPos.value = { x: e.clientX, y: e.clientY };
  menuOpen.value = true;
}

function runMenuCommand(cmd: string) {
  menuRunner?.(cmd);
}

// ---------- 菜单命令 ----------
/** 展开/收起分组 */
function toggleGroup(group: Group) {
  group.open = !group.open;
}

function handleGroupClick(group: Group) {
  if (Date.now() < ignoreClickUntil) return;
  toggleGroup(group);
}

function handleConnClick(conn: Connection) {
  if (Date.now() < ignoreClickUntil) return;
  emit("select", conn.id);
}

function onGroupContextMenu(e: MouseEvent, group: Group) {
  openMenu(
    e,
    [
      { label: "新增连接", command: "add-conn" },
      { label: "重命名", command: "rename-group" },
      { label: "删除分组", command: "delete-group", danger: true },
    ],
    (cmd: string) => {
      if (cmd === "add-conn") emit("add-conn", group.id);
      else if (cmd === "rename-group") emit("rename-group", group.id, group.name);
      else if (cmd === "delete-group") emit("delete-group", group.id, group.name);
    },
  );
}

function onConnContextMenu(e: MouseEvent, conn: Connection) {
  openMenu(
    e,
    [
      { label: "修改", command: "edit-conn" },
      { label: "删除", command: "delete-conn", danger: true },
    ],
    (cmd: string) => {
      if (cmd === "edit-conn") emit("edit-conn", conn);
      else if (cmd === "delete-conn") emit("delete-conn", conn.id, conn.name);
    },
  );
}

// ---------- 长按拖动 ----------
const LONG_PRESS_MS = 150;
const MOVE_CANCEL_PX = 2;
const CLICK_SUPPRESS_MS = 300;

type DragSource =
  { kind: "group"; groupId: string } | { kind: "conn"; connId: string; groupId: string };

type DropTarget =
  | { kind: "root" }
  | { kind: "group"; groupId: string; edge: "before" | "after" | "inside" }
  | { kind: "conn"; connId: string; groupId: string; edge: "before" | "after" };

interface PressState {
  x: number;
  y: number;
  source: DragSource;
  element: HTMLElement;
  pointerId: number;
}

const dragging = ref<DragSource | null>(null);
const dropTarget = ref<DropTarget | null>(null);
let pressState: PressState | null = null;
let pressTimer: number | null = null;
let ignoreClickUntil = 0;
let targetFrame: number | null = null;
let pendingTargetX = 0;
let pendingTargetY = 0;

const realGroups = computed(() => props.groups.filter((g) => !isUngroupedGroup(g)));

const ungroupedConnections = computed(
  () => props.groups.find((g) => isUngroupedGroup(g))?.connections ?? [],
);

function isGroupDragSource(id: string): boolean {
  return dragging.value?.kind === "group" && dragging.value.groupId === id;
}

function isConnDragSource(id: string): boolean {
  return dragging.value?.kind === "conn" && dragging.value.connId === id;
}

function groupDropClass(id: string): string {
  const target = dropTarget.value;
  if (target?.kind !== "group" || target.groupId !== id) return "";
  if (target.edge === "before") return "drop-before";
  if (target.edge === "after") return "drop-after";
  return "drop-into";
}

function connDropClass(id: string): string {
  const target = dropTarget.value;
  if (target?.kind !== "conn" || target.connId !== id) return "";
  return target.edge === "before" ? "drop-before" : "drop-after";
}

function cleanupPress() {
  if (pressTimer !== null) {
    window.clearTimeout(pressTimer);
    pressTimer = null;
  }
  const state = pressState;
  pressState = null;
  if (state) {
    try {
      if (state.element.hasPointerCapture(state.pointerId)) {
        state.element.releasePointerCapture(state.pointerId);
      }
    } catch {
      /* ignore */
    }
  }
  if (targetFrame !== null) {
    window.cancelAnimationFrame(targetFrame);
    targetFrame = null;
  }
  dragging.value = null;
  dropTarget.value = null;
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  window.removeEventListener("pointermove", onPointerMove);
  window.removeEventListener("pointerup", onPointerUp);
  window.removeEventListener("pointercancel", onPointerCancel);
}

function onPointerDown(event: PointerEvent, source: DragSource) {
  if (event.button !== 0 || dragging.value) return;
  if (source.kind === "group" && source.groupId === UNGROUPED_GROUP_ID) return;

  cleanupPress();
  const element = event.currentTarget as HTMLElement;
  pressState = {
    x: event.clientX,
    y: event.clientY,
    source,
    element,
    pointerId: event.pointerId,
  };
  try {
    element.setPointerCapture(event.pointerId);
  } catch {
    /* ignore */
  }
  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
  window.addEventListener("pointercancel", onPointerCancel);

  pressTimer = window.setTimeout(() => {
    pressTimer = null;
    if (!pressState) return;
    dragging.value = pressState.source;
    document.body.style.cursor = "grabbing";
    document.body.style.userSelect = "none";
  }, LONG_PRESS_MS);
}

function onPointerMove(event: PointerEvent) {
  if (!pressState) return;
  if (!dragging.value) {
    if (Math.hypot(event.clientX - pressState.x, event.clientY - pressState.y) > MOVE_CANCEL_PX) {
      cleanupPress();
    }
    return;
  }
  pendingTargetX = event.clientX;
  pendingTargetY = event.clientY;
  if (targetFrame !== null) return;
  targetFrame = window.requestAnimationFrame(() => {
    targetFrame = null;
    updateDropTarget(pendingTargetX, pendingTargetY);
  });
}

function onPointerUp(event: PointerEvent) {
  const source = dragging.value;
  if (source && event.type === "pointerup") {
    updateDropTarget(event.clientX, event.clientY);
    ignoreClickUntil = Date.now() + CLICK_SUPPRESS_MS;
    if (dropTarget.value) applyDrop(source, dropTarget.value);
  }
  cleanupPress();
}

function onPointerCancel() {
  cleanupPress();
}

function updateDropTarget(x: number, y: number) {
  const source = dragging.value;
  if (!source) return;
  const element = document.elementFromPoint(x, y) as HTMLElement | null;
  if (!element) {
    dropTarget.value = null;
    return;
  }

  if (source.kind === "group") {
    const groupEl = element.closest<HTMLElement>("[data-group-id]");
    if (!groupEl) {
      dropTarget.value = null;
      return;
    }
    const groupId = groupEl.dataset.groupId ?? "";
    if (!groupId || groupId === UNGROUPED_GROUP_ID || groupId === source.groupId) {
      dropTarget.value = null;
      return;
    }
    const rect = groupEl.getBoundingClientRect();
    dropTarget.value = {
      kind: "group",
      groupId,
      edge: y < rect.top + rect.height / 2 ? "before" : "after",
    };
    return;
  }

  const rootEl = element.closest<HTMLElement>("[data-drop-root]");
  if (rootEl) {
    dropTarget.value = { kind: "root" };
    return;
  }

  const connEl = element.closest<HTMLElement>("[data-conn-id]");
  if (connEl) {
    const connId = connEl.dataset.connId ?? "";
    const groupId = connEl.dataset.parentGroup ?? "";
    if (!connId || connId === source.connId) {
      dropTarget.value = null;
      return;
    }
    const rect = connEl.getBoundingClientRect();
    dropTarget.value = {
      kind: "conn",
      connId,
      groupId,
      edge: y < rect.top + rect.height / 2 ? "before" : "after",
    };
    return;
  }

  const groupEl = element.closest<HTMLElement>("[data-group-id]");
  if (groupEl) {
    const groupId = groupEl.dataset.groupId ?? "";
    if (!groupId) {
      dropTarget.value = null;
      return;
    }
    dropTarget.value = { kind: "group", groupId, edge: "inside" };
    return;
  }

  dropTarget.value = null;
}

function buildOrder(groupOrder: Group[], connsByGroup: Map<string, Connection[]>): TreeOrder {
  const connections: TreeOrder["connections"] = [];
  for (const group of groupOrder) {
    for (const conn of connsByGroup.get(group.id) ?? []) {
      connections.push({ id: conn.id, groupId: group.id });
    }
  }
  for (const conn of connsByGroup.get(UNGROUPED_GROUP_ID) ?? []) {
    connections.push({ id: conn.id, groupId: "" });
  }
  return { groupIds: groupOrder.map((g) => g.id), connections };
}

function applyDrop(source: DragSource, target: DropTarget) {
  const realGroups = props.groups.filter((g) => !isUngroupedGroup(g));
  const connsByGroup = new Map<string, Connection[]>(
    props.groups.map((g) => [g.id, [...g.connections]]),
  );

  if (source.kind === "group") {
    if (target.kind !== "group" || target.edge === "inside") return;
    const dragged = realGroups.find((g) => g.id === source.groupId);
    if (!dragged) return;
    const ordered = realGroups.filter((g) => g.id !== source.groupId);
    const targetIndex = ordered.findIndex((g) => g.id === target.groupId);
    if (targetIndex < 0) return;
    ordered.splice(target.edge === "before" ? targetIndex : targetIndex + 1, 0, dragged);
    emit("reorder", buildOrder(ordered, connsByGroup));
    return;
  }

  const sourceConns = connsByGroup.get(source.groupId);
  if (!sourceConns) return;
  const sourceIndex = sourceConns.findIndex((c) => c.id === source.connId);
  if (sourceIndex < 0) return;
  const [draggedConn] = sourceConns.splice(sourceIndex, 1);

  let destinationId: string;
  let insertIndex: number;
  if (target.kind === "root") {
    // 落在“连接管理”标题上：移到所有分组之外的最顶层
    destinationId = UNGROUPED_GROUP_ID;
    insertIndex = 0;
  } else if (target.kind === "group") {
    // 落在分组标题上：插入该组第一个位置，横线显示在标题下方
    destinationId = target.groupId;
    insertIndex = 0;
  } else {
    destinationId = target.groupId;
    const targetConns = connsByGroup.get(destinationId) ?? [];
    const targetIndex = targetConns.findIndex((c) => c.id === target.connId);
    if (targetIndex < 0) return;
    insertIndex = target.edge === "before" ? targetIndex : targetIndex + 1;
  }

  let destinationConns = connsByGroup.get(destinationId);
  if (!destinationConns) {
    destinationConns = [];
    connsByGroup.set(destinationId, destinationConns);
  }
  destinationConns.splice(insertIndex, 0, draggedConn);

  const destinationGroup = props.groups.find((g) => g.id === destinationId);
  if (destinationGroup) destinationGroup.open = true;

  emit("reorder", buildOrder(realGroups, connsByGroup));
}

onBeforeUnmount(cleanupPress);
</script>

<style scoped>
.sidebar-content::-webkit-scrollbar {
  width: 4px;
}
.sidebar-content::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 2px;
}

.connection-list::before {
  content: "";
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 1px;
  border-left: 1px dashed var(--color-border);
}

.connection-item::before {
  content: "";
  width: 8px;
  height: 1px;
  flex-shrink: 0;
  background: var(--color-border);
}

.group-row.drop-before::after,
.connection-item.drop-before::after,
.connection-root-item.drop-before::after,
.group-row.drop-after::after,
.group-row.drop-into::after,
.connection-item.drop-after::after,
.connection-root-item.drop-after::after {
  content: "";
  position: absolute;
  left: 4px;
  right: 4px;
  height: 2px;
  border-radius: 9999px;
  background: var(--color-primary);
  pointer-events: none;
}

.drop-before::after {
  top: 0;
}

.drop-after::after,
.drop-into::after {
  bottom: 0;
}

.group-header-row.drop-root::after {
  content: "";
  position: absolute;
  left: 12px;
  right: 12px;
  bottom: 2px;
  height: 2px;
  border-radius: 9999px;
  background: var(--color-primary);
  pointer-events: none;
}

.drag-source {
  opacity: 0.4;
  pointer-events: none;
}
</style>
