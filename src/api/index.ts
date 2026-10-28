/**
 * 后端命令封装统一入口。
 *
 * 按领域拆分：连接/分组（connections.ts）、SFTP 文件操作（sftp.ts）。
 * 组件统一从 `../api` 导入，保持路径不变。
 */

import { toast } from "vue-sonner";

export * from "./connections";
export * from "./sftp";
export * from "./local";

/** 统一错误提示：兼容 Tauri 返回的字符串错误与 Error 对象 */
export function notifyError(err: unknown): void {
  const msg = typeof err === "string" ? err : ((err as Error)?.message ?? "操作失败");
  toast.error(msg);
}
