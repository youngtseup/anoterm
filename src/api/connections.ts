/**
 * 分组 / 连接 / 最近连接 的 Tauri 命令封装。
 *
 * Tauri 返回值不做字段名转换（snake_case 原样返回），
 * 这里统一把后端 snake_case 映射为前端的 camelCase 结构。
 */

import { invoke } from "@tauri-apps/api/core";
import type { AddConnInput, Connection, Group, RecentItem, TreeOrder } from "../types";

/** 后端返回的连接原始结构（snake_case） */
interface RawConnection {
  id: string;
  name: string;
  type: string;
  host: string;
  port: number;
  username: string;
  password: string;
  remark: string;
  group_id: string;
  auth_type: string;
  private_key_path: string;
  key_passphrase: string;
}

/** 后端返回的分组原始结构 */
interface RawGroup {
  id: string;
  name: string;
  connections: RawConnection[];
}

/** 后端返回的最近连接原始结构 */
interface RawRecent {
  id: string;
  name: string;
  type: string;
  host: string;
  port: number;
  username: string;
  remark: string;
  last_used: number;
}

/** snake_case → camelCase：连接 */
function toConnection(raw: RawConnection): Connection {
  return {
    id: raw.id,
    name: raw.name,
    type: raw.type,
    host: raw.host,
    port: raw.port,
    username: raw.username,
    password: raw.password,
    remark: raw.remark,
    groupId: raw.group_id,
    authType: raw.auth_type,
    privateKeyPath: raw.private_key_path,
    keyPassphrase: raw.key_passphrase,
  };
}

/** snake_case → camelCase：分组（open 为前端展开状态，初始收起） */
function toGroup(raw: RawGroup): Group {
  return {
    id: raw.id,
    name: raw.name,
    open: false,
    connections: raw.connections.map(toConnection),
  };
}

/** camelCase → snake_case：新建/更新连接的输入 */
function toInput(input: AddConnInput) {
  return {
    group_id: input.groupId,
    type: input.type,
    name: input.name,
    host: input.host,
    port: input.port,
    username: input.username,
    password: input.password,
    remark: input.remark,
    auth_type: input.authType,
    private_key_path: input.privateKeyPath,
    key_passphrase: input.keyPassphrase,
  };
}

/** 列出全部分组（含连接） */
export async function listGroups(): Promise<Group[]> {
  return (await invoke<RawGroup[]>("list_groups")).map(toGroup);
}

/** 新建分组 */
export async function createGroup(name: string): Promise<Group> {
  return toGroup(await invoke<RawGroup>("create_group", { name }));
}

/** 重命名分组 */
export async function renameGroup(id: string, name: string): Promise<void> {
  await invoke("rename_group", { id, name });
}

/** 删除分组（后端级联删除其下连接） */
export async function deleteGroup(id: string): Promise<void> {
  await invoke("delete_group", { id });
}

/** 新建连接 */
export async function createConnection(input: AddConnInput): Promise<Connection> {
  return toConnection(await invoke<RawConnection>("create_connection", { input: toInput(input) }));
}

/** 更新连接 */
export async function updateConnection(id: string, input: AddConnInput): Promise<void> {
  await invoke("update_connection", { id, input: toInput(input) });
}

/** 删除连接 */
export async function deleteConnection(id: string): Promise<void> {
  await invoke("delete_connection", { id });
}

/** 保存拖拽后的树顺序 */
export async function saveTreeOrder(order: TreeOrder): Promise<void> {
  await invoke("reorder_tree", {
    groupIds: order.groupIds,
    connections: order.connections.map((c) => ({ id: c.id, group_id: c.groupId })),
  });
}

/** 列出最近连接（按使用时间倒序，最多 30 条） */
export async function listRecent(): Promise<RecentItem[]> {
  const raws = await invoke<RawRecent[]>("list_recent");
  return raws.map((r) => ({
    id: r.id,
    name: r.name,
    type: r.type,
    host: r.host,
    port: r.port,
    username: r.username,
    lastUsed: r.last_used,
  }));
}

/** 清空最近连接记录 */
export async function clearRecent(): Promise<void> {
  await invoke("clear_recent");
}
