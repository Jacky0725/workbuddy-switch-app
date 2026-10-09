import { describe, expect, it } from "vitest";
import { accountPlanLabel, beijingTime, canRetryTask, configIssue, currentTask, nextPlanLabel, taskLabel } from "./manager-scheduler";
import { managerDemoSnapshot } from "./manager-scheduler-demo";
import type { AccountMeta } from "./types";

const account: AccountMeta = {
  id: "fiction", uid: null, email: null, nickname: "虚构账号", enterpriseName: null,
  expiresAt: null, refreshExpiresAt: null, refreshedAt: null, createdAt: null,
  needsRelogin: false, needsReloginReason: null, variant: "cn",
};
const snapshot = () => managerDemoSnapshot([account], Date.parse("2026-10-09T01:00:00Z"));

describe("每日计划显示", () => {
  it("没有记录不冒充签到成功", () => {
    expect(taskLabel()).toBe("暂无执行记录");
    const task = snapshot().tasks[0];
    expect(taskLabel({ ...task, state: "Pending" })).toContain("等待核实");
    expect(taskLabel({ ...task, state: "Missed" })).toContain("已错过计划");
  });
  it("不确定结果禁止重复提交", () => {
    const task = snapshot().tasks[0];
    expect(taskLabel({ ...task, state: { Finished: "Unknown" }, uncertain: true })).toContain("禁止重复提交");
    expect(taskLabel({ ...task, state: { Finished: "AuthExpired" } })).toBe("需要重新授权");
  });
  it("仅使用同一天、同账号、同提供方、同区域的记录", () => {
    const value = snapshot();
    expect(currentTask(value, account)).toBe(value.tasks[0]);
    for (const patch of [{ business_date: "2026-10-08" }, { region: "ai" as const }, { provider: "other" }, { account_id: "other" }]) {
      expect(currentTask({ ...value, tasks: [{ ...value.tasks[0], ...patch }] }, account)).toBeUndefined();
    }
  });
  it("生产能力关闭时显示暂停，即使计划已经开启", () => {
    expect(accountPlanLabel(snapshot(), account)).toBe("计划暂停 · 北京时间 09:15");
    expect(nextPlanLabel(snapshot())).toBe("执行暂停，尚未安排实际签到");
  });
  it("只有当日可重试失败在间隔与次数预算内允许重试", () => {
    const value = { ...snapshot(), production_enabled: true };
    const task = { ...value.tasks[0], state: { Finished: "NetworkError" as const }, last_started_at_ms: value.observed_at_ms - 300_000 };
    expect(canRetryTask(value, task)).toBe(true);
    for (const patch of [{ uncertain: true }, { attempts: 2 }, { business_date: "2026-10-08" }, { last_started_at_ms: value.observed_at_ms - 299_000 }, { state: { Finished: "Success" as const } }, { state: { Finished: "RateLimited" as const } }, { state: "Pending" as const }]) {
      expect(canRetryTask(value, { ...task, ...patch })).toBe(false);
    }
    expect(canRetryTask({ ...value, production_enabled: false }, task)).toBe(false);
  });
  it("区域、停用账号、排除和总开关分别生效", () => {
    const value = snapshot();
    expect(accountPlanLabel(value, { ...account, variant: "ai" })).toContain("不支持");
    expect(accountPlanLabel(value, { ...account, provider: "other" })).toContain("不支持");
    expect(accountPlanLabel(value, { ...account, is_enabled: false })).toBe("账号已停用");
    expect(accountPlanLabel({ ...value, config: { ...value.config, account_enabled: { fiction: false } } }, account)).toBe("未参加计划");
    expect(accountPlanLabel({ ...value, config: { ...value.config, enabled: false } }, account)).toBe("每日计划未开启");
  });
  it("拒绝无效时间、非整数次数和越界重试间隔", () => {
    const config = snapshot().config;
    expect(configIssue(config)).toBeNull();
    for (const daily_time of ["9:15", "24:00", "09:60", ""]) expect(configIssue({ ...config, daily_time })).not.toBeNull();
    for (const max_daily_attempts of [0, 4, 1.5, NaN]) expect(configIssue({ ...config, max_daily_attempts })).not.toBeNull();
    for (const retry_delay_seconds of [29, 86401, 30.5]) expect(configIssue({ ...config, retry_delay_seconds })).not.toBeNull();
  });
  it("时间以北京时间显示，空时间不冒充实际执行", () => {
    expect(beijingTime(Date.parse("2026-10-08T16:15:00Z"))).toContain("10/09");
    expect(beijingTime(Date.parse("2026-10-08T16:15:00Z"))).toContain("00:15");
    expect(beijingTime(null)).toBe("—");
    expect(beijingTime(NaN)).toBe("—");
  });
  it("演示记录为虚构数据且不能启用生产执行", () => {
    const value = managerDemoSnapshot([account, { ...account, id: "uncertain" }, { ...account, id: "expired" }]);
    expect(value.production_enabled).toBe(false);
    expect(value.background_active).toBe(false);
    expect(value.revision).toBe("demo-readonly");
    expect(value.tasks[1].uncertain).toBe(true);
  });
});
