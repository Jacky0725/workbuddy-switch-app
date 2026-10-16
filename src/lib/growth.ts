import type { GrowthTask } from "./types";

export function growthTaskStatus(task: GrowthTask, now = Date.now()): string {
  if (task.valid_end_ms !== null && task.valid_end_ms <= now) return "活动已结束";
  if (task.valid_start_ms !== null && task.valid_start_ms > now) return "尚未开始";
  if (task.accept_status === "claimed") return "已领取";
  if (task.locked === true) return "尚未解锁";
  if (task.accept_status === "completed") return "已完成 · 尚未领取";
  if (task.progress_complete) return "进度已达标 · 领取资格待核实";
  if (task.accept_status === "accepted") return "已接受 · 尚未完成";
  if (task.accept_status === "in_progress") return "进行中 · 尚未完成";
  if (task.accept_status === "not_accepted") return "尚未接受";
  return "任务状态待核实";
}
