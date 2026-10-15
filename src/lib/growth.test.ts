import { expect, it } from "vitest";
import { growthTaskStatus } from "./growth";
import type { GrowthTask } from "./types";
const task: GrowthTask = {code:"chat_5",title:"测试任务",accept_status:"accepted",locked:false,current:5,target:5,reward_credit:null,reward_energy:null,valid_start_ms:null,valid_end_ms:null,progress_complete:true};
it("进度达标或完成都不等于已经领取",()=>{
  expect(growthTaskStatus(task)).toContain("待核实");
  expect(growthTaskStatus({...task,accept_status:"completed"})).toContain("尚未领取");
  expect(growthTaskStatus({...task,accept_status:"claimed"})).toBe("已领取");
});
it("过期和锁定优先于可领取的视觉状态",()=>{
  expect(growthTaskStatus({...task,accept_status:"completed",valid_end_ms:10},11)).toBe("活动已结束");
  expect(growthTaskStatus({...task,accept_status:"completed",locked:true})).toBe("尚未解锁");
  expect(growthTaskStatus({...task,progress_complete:false,accept_status:"unknown"})).toBe("任务状态待核实");
});
