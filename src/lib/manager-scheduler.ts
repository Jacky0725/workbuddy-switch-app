import type { AccountMeta, ManagerDailyTask, ManagerSchedulerConfig, ManagerSchedulerSnapshot } from "./types";
import { accountVariant } from "./variant";

const labels: Record<string, string> = {
  Success: "签到成功", AlreadyCompleted: "已完成", AuthExpired: "需要重新授权",
  RequiresUserAction: "需要手动处理", RateLimited: "请求受限，今日停止重试", NetworkError: "网络连接失败",
  ServerError: "服务暂不可用", Unsupported: "当前能力未开放", Unknown: "结果未确认",
  Cancelled: "已取消", InProgress: "正在执行",
};
export function taskLabel(task?: ManagerDailyTask): string {
  if (!task) return "暂无执行记录";
  if (task.state === "Pending") return "执行中断，等待核实";
  if (task.state === "Missed") return "已错过计划，补执行已关闭";
  const label = labels[task.state.Finished] ?? "结果未确认";
  return task.uncertain ? `${label} · 需核实，禁止重复提交` : label;
}
export function currentTask(snapshot: ManagerSchedulerSnapshot, account: AccountMeta): ManagerDailyTask | undefined {
  return snapshot.tasks.find(task => task.account_id === account.id && task.region === accountVariant(account)
    && task.provider === (account.provider ?? "workbuddy") && task.business_date === snapshot.business_date);
}
export function accountPlanLabel(snapshot: ManagerSchedulerSnapshot, account: AccountMeta): string {
  if (accountVariant(account) !== "cn" || (account.provider ?? "workbuddy") !== "workbuddy") return "该区域或提供方不支持签到";
  if (account.is_enabled === false) return "账号已停用";
  if (snapshot.config.account_enabled[account.id] === false) return "未参加计划";
  if (!snapshot.config.enabled) return "每日计划未开启";
  return `${snapshot.production_enabled ? "北京时间" : "计划暂停 · 北京时间"} ${snapshot.config.daily_time}`;
}
export function nextPlanLabel(snapshot: ManagerSchedulerSnapshot): string {
  if (!snapshot.config.enabled) return "每日计划未开启";
  if (!snapshot.production_enabled) return "执行暂停，尚未安排实际签到";
  return beijingTime(snapshot.next_planned_at_ms);
}
export function canRetryTask(snapshot: ManagerSchedulerSnapshot, task: ManagerDailyTask): boolean {
  if (!snapshot.production_enabled || !snapshot.config.enabled || task.uncertain
    || task.business_date !== snapshot.business_date || task.provider !== "workbuddy" || task.region !== "cn"
    || snapshot.config.account_enabled[task.account_id] === false || task.attempts >= snapshot.config.max_daily_attempts
    || typeof task.state === "string") return false;
  if (task.state.Finished !== "NetworkError" && task.state.Finished !== "ServerError") return false;
  return task.last_started_at_ms === null || snapshot.observed_at_ms - task.last_started_at_ms >= snapshot.config.retry_delay_seconds * 1000;
}
export function configIssue(config: ManagerSchedulerConfig): string | null {
  if (!/^(?:[01]\d|2[0-3]):[0-5]\d$/.test(config.daily_time)) return "请选择有效的北京时间";
  if (!Number.isInteger(config.max_daily_attempts) || config.max_daily_attempts < 1 || config.max_daily_attempts > 3) return "每日尝试次数应为 1–3";
  if (!Number.isInteger(config.retry_delay_seconds) || config.retry_delay_seconds < 30 || config.retry_delay_seconds > 86400) return "重试间隔应为 30–86400 秒";
  return null;
}
export function beijingTime(timestamp: number | null): string {
  if (timestamp === null || !Number.isFinite(timestamp)) return "—";
  return new Intl.DateTimeFormat("zh-CN", { timeZone: "Asia/Shanghai", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hour12: false }).format(new Date(timestamp));
}
