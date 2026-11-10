/**
 * 全局类型定义与类型相关工具函数。
 */

/** 连接记录（与后端 connections 表对应，字段为 camelCase） */
export interface Connection {
  id: string;
  name: string;
  /** 连接类型：ssh / sftp / ssh_sftp */
  type: string;
  host: string;
  port: number;
  username: string;
  password: string;
  remark: string;
  groupId: string;
  /** 认证方式：password | key */
  authType: string;
  /** 私钥文件路径（引用，不复制内容入库） */
  privateKeyPath: string;
  /** 私钥口令（加密存储，可为空） */
  keyPassphrase: string;
}

/** 未分组连接在侧边栏中的虚拟分组 id（与后端保持一致） */
export const UNGROUPED_GROUP_ID = "__ungrouped__";

/** 分组（open 为前端侧边栏展开状态） */
export interface Group {
  id: string;
  name: string;
  open: boolean;
  connections: Connection[];
}

export function isUngroupedGroup(group: Pick<Group, "id">): boolean {
  return group.id === UNGROUPED_GROUP_ID;
}

/** 拖拽后的树顺序：分组 id 顺序 + 所有连接的归属与顺序 */
export interface TreeOrder {
  groupIds: string[];
  connections: { id: string; groupId: string }[];
}

/** 右键菜单项 */
export interface ContextMenuItem {
  label: string;
  command: string;
  danger?: boolean;
}

/** tab 的子视图类型：SSH 终端 / SFTP 文件浏览器 */
export type SubKind = "ssh" | "sftp";

/** 目录项（SFTP 列表返回，字段与后端 camelCase 对齐） */
export interface SftpEntry {
  name: string;
  path: string;
  isDir: boolean;
  isSymlink: boolean;
  size: number | null;
  /** 修改时间（Unix 秒） */
  mtime: number | null;
  permissions: number | null;
}

/** 一个打开的标签（对应后端一个会话，以 tabId 为会话 key） */
export interface TerminalTab {
  tabId: string;
  connectionId: string;
  name: string;
  host: string;
  port: number;
  username: string;
  remark: string;
  connectionType: string;
  /** 当前显示的子视图（SSH/SFTP） */
  activeSub: SubKind;
  /** 会话是否已建立（ssh_connect 成功） */
  connected: boolean;
}

/** 最近连接（后端快照记录） */
export interface RecentItem {
  id: string;
  name: string;
  type: string;
  host: string;
  port: number;
  username: string;
  remark?: string;
  /** 最近使用时间（Unix 秒） */
  lastUsed: number;
}

/** 连接详情浮层展示的数据 */
export interface ConnectionDetails {
  name: string;
  type: string;
  host: string;
  port: number;
  username: string;
  authType?: string;
  remark?: string;
  deleted?: boolean;
}

/** 连接类型的中文展示文案 */
export function connectionTypeLabel(type: string): string {
  if (type === "ssh") return "SSH";
  if (type === "sftp") return "SFTP";
  if (type === "ssh_sftp") return "SSH & SFTP";
  return type;
}

/** 认证方式的中文展示文案 */
export function connectionAuthLabel(authType?: string): string {
  if (authType === "password") return "账号密码";
  if (authType === "key") return "私钥";
  return "未知";
}

/** 新建/更新连接的输入参数 */
export interface AddConnInput {
  groupId: string;
  name: string;
  type: string;
  host: string;
  port: number;
  username: string;
  password: string;
  remark: string;
  authType: string;
  privateKeyPath: string;
  keyPassphrase: string;
}

/** 新建连接的默认值 */
export const EMPTY_CONN_INPUT: AddConnInput = {
  groupId: "",
  name: "",
  type: "ssh_sftp",
  host: "",
  port: 22,
  username: "",
  password: "",
  remark: "",
  authType: "password",
  privateKeyPath: "",
  keyPassphrase: "",
};

/** 按连接类型返回其支持的子视图列表 */
export function subTabsForType(type: string): SubKind[] {
  if (type === "sftp") return ["sftp"];
  if (type === "ssh") return ["ssh"];
  return ["ssh", "sftp"];
}

/** 更新任务状态（用于标题栏展示） */
export type UpdateStatus = "idle" | "downloading" | "downloaded" | "installing";

/** UpdateDialog 向父组件同步的更新状态 */
export interface UpdateStatusPayload {
  checking: boolean;
  status: UpdateStatus;
  /** 下载进度 0-100（总大小未知时为 null） */
  progress: number | null;
}
