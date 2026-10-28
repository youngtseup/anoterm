/**
 * 多标签（tab）状态管理组合式函数。
 *
 * 每个 tab 对应一个后端会话（以 tabId 为会话 key），
 * 一个连接可同时打开多个 tab。负责：
 * - tab 列表与激活状态的维护
 * - 打开 / 关闭（单个、左侧、右侧、其他、全部）
 * - 子视图（SSH / SFTP）切换
 * - 连接删除后清理失效 tab
 */

import { computed, ref, type Ref } from "vue";
import type { Connection, Group, SubKind, TerminalTab } from "../types";
import { subTabsForType } from "../types";

/** tab id 自增序号，保证同一毫秒内创建的 id 不冲突 */
let tabSeq = 0;
function newTabId(): string {
  tabSeq += 1;
  return `tab-${Date.now().toString(36)}-${tabSeq}`;
}

/**
 * 初始化 tab 管理器。
 * @param groups 分组列表（用于按连接 id 查找连接）
 * @param sidebarCollapsed 侧边栏收起状态（打开 tab 时自动收起）
 */
export function useTabs(groups: Ref<Group[]>, sidebarCollapsed: Ref<boolean>) {
  /** 全部 tab */
  const tabs = ref<TerminalTab[]>([]);
  /** 当前激活 tab id */
  const activeTabId = ref<string | null>(null);

  /** 当前激活 tab 对应的连接 id（供侧边栏高亮） */
  const activeConnId = computed(() => {
    const tab = tabs.value.find((t) => t.tabId === activeTabId.value);
    return tab ? tab.connectionId : null;
  });

  /** 按连接 id 查找连接（跨分组搜索） */
  function getConnection(id: string): Connection | null {
    for (const g of groups.value) {
      const c = g.connections.find((x) => x.id === id);
      if (c) return c;
    }
    return null;
  }

  /** 打开连接：新建 tab 并激活（会话由 ConnectionView 挂载时建立） */
  function handleSelect(id: string) {
    const conn = getConnection(id);
    if (!conn) return;
    const tab: TerminalTab = {
      tabId: newTabId(),
      connectionId: id,
      name: conn.name,
      host: conn.host,
      port: conn.port,
      username: conn.username,
      remark: conn.remark,
      connectionType: conn.type,
      activeSub: subTabsForType(conn.type)[0],
      connected: false,
    };
    tabs.value.push(tab);
    activeTabId.value = tab.tabId;
    sidebarCollapsed.value = true;
  }

  /** 切换激活 tab */
  function selectTab(tabId: string) {
    activeTabId.value = tabId;
  }

  /** 返回首页（仪表盘）：保留已打开 tab 与会话，仅取消激活 */
  function goHome() {
    activeTabId.value = null;
    sidebarCollapsed.value = false;
  }

  /** 切换 tab 的子视图（SSH / SFTP） */
  function setSubTab(tabId: string, sub: SubKind) {
    const t = tabs.value.find((x) => x.tabId === tabId);
    if (t) t.activeSub = sub;
  }

  /** 关闭单个 tab，激活相邻 tab；全部关闭后回到仪表盘并展开侧边栏 */
  function closeTab(tabId: string) {
    const idx = tabs.value.findIndex((t) => t.tabId === tabId);
    if (idx === -1) return;
    tabs.value.splice(idx, 1);
    if (activeTabId.value === tabId) {
      if (tabs.value.length === 0) {
        activeTabId.value = null;
        sidebarCollapsed.value = false;
      } else {
        const next = tabs.value[Math.min(idx, tabs.value.length - 1)];
        activeTabId.value = next.tabId;
      }
    }
  }

  /** 关闭指定 tab 左侧的所有 tab */
  function closeLeft(tabId: string) {
    const idx = tabs.value.findIndex((t) => t.tabId === tabId);
    if (idx === -1) return;
    const removed = tabs.value.slice(0, idx);
    tabs.value = tabs.value.slice(idx);
    if (activeTabId.value && removed.some((t) => t.tabId === activeTabId.value)) {
      activeTabId.value = tabId;
    }
    if (tabs.value.length === 0) {
      activeTabId.value = null;
      sidebarCollapsed.value = false;
    }
  }

  /** 关闭指定 tab 右侧的所有 tab */
  function closeRight(tabId: string) {
    const idx = tabs.value.findIndex((t) => t.tabId === tabId);
    if (idx === -1) return;
    tabs.value = tabs.value.slice(0, idx + 1);
    if (activeTabId.value && !tabs.value.some((t) => t.tabId === activeTabId.value)) {
      activeTabId.value = tabId;
    }
    if (tabs.value.length === 0) {
      activeTabId.value = null;
      sidebarCollapsed.value = false;
    }
  }

  /** 关闭除指定 tab 外的所有 tab */
  function closeOthers(tabId: string) {
    tabs.value = tabs.value.filter((t) => t.tabId === tabId);
    activeTabId.value = tabId;
  }

  /** 关闭全部 tab */
  function closeAllTabs() {
    tabs.value = [];
    activeTabId.value = null;
    sidebarCollapsed.value = false;
  }

  /** 按条件移除 tab（如连接被删除时清理其打开的 tab） */
  function removeTabs(predicate: (t: TerminalTab) => boolean) {
    const doomed = tabs.value.filter(predicate);
    if (doomed.length === 0) return;
    for (const t of doomed) tabs.value.splice(tabs.value.indexOf(t), 1);
    if (activeTabId.value && !tabs.value.some((t) => t.tabId === activeTabId.value)) {
      activeTabId.value = tabs.value.length > 0 ? tabs.value[tabs.value.length - 1].tabId : null;
    }
    if (tabs.value.length === 0) {
      activeTabId.value = null;
      sidebarCollapsed.value = false;
    }
  }

  return {
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
  };
}
