import type { AccountMeta, ManagerDailyTask, ManagerSchedulerSnapshot } from "./types";

export function managerDemoSnapshot(accounts: AccountMeta[], now = Date.now()): ManagerSchedulerSnapshot {
  const date = new Intl.DateTimeFormat("sv-SE", { timeZone: "Asia/Shanghai" }).format(new Date(now));
  const planned = Date.parse(`${date}T09:15:00+08:00`);
  const statuses = ["Success", "Unknown", "AuthExpired"] as const;
  const tasks: ManagerDailyTask[] = accounts.filter(a => (a.variant ?? "cn") === "cn").slice(0, 3).map((account, index) => ({
    account_id: account.id, provider: "workbuddy", region: "cn", business_date: date, scheduled_at_ms: planned,
    state: { Finished: statuses[index] }, attempts: 1, verification_attempts: index === 1 ? 1 : 0,
    last_started_at_ms: planned, uncertain: index === 1,
    events: [{ started_at_ms: planned, finished_at_ms: planned + 1200, status: statuses[index], may_have_completed: index === 1, verification_only: false }],
  }));
  return {
    config: { version: 1, enabled: true, timezone: "Asia/Shanghai", daily_time: "09:15", catch_up: true, max_daily_attempts: 2, retry_delay_seconds: 300, account_enabled: {} },
    revision: "demo-readonly", business_date: date, observed_at_ms: now, planned_at_ms: planned,
    next_planned_at_ms: planned + 86400000, production_enabled: false,
    blocked_reason: "签到授权与服务接口尚未验证，计划已暂停；不会发送真实请求",
    background_active: false, last_heartbeat_ms: null, tasks, history_limit: 500,
  };
}
