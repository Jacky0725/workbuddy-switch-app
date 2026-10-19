import { useEffect, useRef, useState } from "react";
import * as api from "@/lib/api";
import type { AccountMeta, GrowthSnapshot, GrowthOperation, GrowthSchedule, GrowthTravel, ManagerSchedulerSnapshot } from "@/lib/types";
import { beijingTime } from "@/lib/manager-scheduler";
import { growthTaskProgress, growthTaskStatus } from "@/lib/growth";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

export default function GrowthTasksPage() {
  const [data, setData] = useState<GrowthSnapshot | null>(null);
  const [accounts, setAccounts] = useState<AccountMeta[]>([]);
  const [scheduler, setScheduler] = useState<ManagerSchedulerSnapshot | null>(null);
  const [selected, setSelected] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [operations, setOperations] = useState<GrowthOperation[]>([]);
  const [plan, setPlan] = useState<GrowthSchedule | null>(null);
  const [travel, setTravel] = useState<GrowthTravel | null>(null);
  useEffect(() => setTravel(null), [selected]);
  const mounted = useRef(false);
  useEffect(() => {
    mounted.current = true;
    void Promise.all([api.getGrowthTasks(), api.getAccounts(), api.getManagerScheduler(), api.getGrowthOperations(), api.getGrowthSchedule()]).then(([growth, list, schedule, ops, p]) => {
      if (mounted.current) { setData(growth); setAccounts(list.accounts); setScheduler(schedule); setOperations(ops); setPlan(p); }
    }).catch(cause => { if (mounted.current) setError(api.asError(cause)); });
    return () => { mounted.current = false; };
  }, []);
  async function refresh() {
    if (busy) return;
    setBusy(true); setError(null);
    try { const next = await api.refreshGrowthTasks(selected || undefined); if (mounted.current) setData(next); }
    catch (cause) { if (mounted.current) setError(api.asError(cause)); }
    finally { if (mounted.current) setBusy(false); }
  }
  async function claim() {
    if (busy) return;
    setBusy(true); setError(null); setMessage(null);
    try {
      const result = await api.claimGrowthRewards(selected || undefined);
      if (mounted.current) {
        setMessage(`服务端确认领取 ${result.verified} 项，待核实 ${result.awaiting_verification} 项。${result.errors.join("；")}`);
      }
      const [next, ops] = await Promise.all([api.refreshGrowthTasks(selected || undefined), api.getGrowthOperations()]);
      if (mounted.current) { setData(next); setOperations(ops); }
    } catch (cause) { if (mounted.current) setError(api.asError(cause)); }
    finally { if (mounted.current) setBusy(false); }
  }
  async function savePlan() {
    if (!plan || busy) return;
    setBusy(true); setError(null);
    try {
      const saved = await api.saveGrowthSchedule(plan.config, plan.revision, plan.chat_execution_enabled);
      const next = await api.getGrowthTasks();
      if (mounted.current) { setPlan(saved); setData(next); setMessage("成长计划已保存。电脑唤醒且程序运行时，按北京时间处理当天任务。"); }
    } catch(cause) { if (mounted.current) setError(api.asError(cause)); }
    finally { if (mounted.current) setBusy(false); }
  }
  async function travelAction(action: "query" | "claim" | "depart") {
    if (!selected || busy) return;
    setBusy(true); setError(null); setMessage(null);
    try {
      if (action !== "query") {
        const result = await api.runGrowthTravel(selected, action === "claim", action === "depart" ? plan?.config.travel_location_id ?? undefined : undefined);
        if (mounted.current) setMessage(`旅行结果：已核实 ${result.verified} 项，待核实 ${result.awaiting_verification} 项，跳过 ${result.skipped} 项。`);
      }
      const [next, ops] = await Promise.all([api.refreshGrowthTravel(selected), api.getGrowthOperations()]);
      if (mounted.current) {setTravel(next);setOperations(ops);}
    } catch(cause) {if (mounted.current) setError(api.asError(cause));}
    finally {if (mounted.current) setBusy(false);}
  }
  return <div className="mx-auto max-w-5xl space-y-6 p-6 md:p-8">
    <div><h1 className="text-2xl font-semibold">成长任务</h1><p className="mt-2 text-sm text-muted-foreground">按账号查看官方任务进度 · 缓存不会自动触发任务</p></div>
    <Card><CardHeader><CardTitle>后台运行状态</CardTitle></CardHeader><CardContent className="space-y-2 text-sm">
      <p>签到计划：{scheduler ? !scheduler.config.enabled ? "已关闭" : !scheduler.production_enabled ? "已暂停 · 请查看自动签到页面" : scheduler.background_active ? "后台协调器运行中" : "已配置 · 尚未检测到后台心跳" : "状态未读取"}</p>
      <p>最近心跳：{beijingTime(scheduler?.last_heartbeat_ms ?? null)}</p>
      <p>{data?.execution_status ?? "成长任务执行状态尚未读取"}</p>
      <p className="text-muted-foreground">旅行遵循下方独立开关；CLI 轮换及通用保活循环未启动。签到心跳不代表成长动作已经执行。</p>
    </CardContent></Card>
    {plan && <Card><CardHeader><CardTitle>成长任务定时计划</CardTitle></CardHeader><CardContent className="space-y-3 text-sm">
      <label className="flex gap-2"><input type="checkbox" checked={plan.config.enabled} disabled={busy} onChange={e => setPlan({...plan, config:{...plan.config, enabled:e.target.checked}})} />开启成长计划</label>
      <label className="flex gap-2"><input type="checkbox" checked={plan.chat_execution_enabled} disabled={busy} onChange={e => setPlan({...plan, chat_execution_enabled:e.target.checked})} />自动完成聊天成长任务（可能消耗模型额度）</label>
      <label className="flex gap-2"><input type="checkbox" checked={plan.config.auto_claim} disabled={busy} onChange={e => setPlan({...plan, config:{...plan.config, auto_claim:e.target.checked}})} />自动领取已完成的普通任务奖励</label>
      <label className="flex gap-2"><input type="checkbox" checked={plan.config.travel_claim} disabled={busy} onChange={e => setPlan({...plan, config:{...plan.config, travel_claim:e.target.checked}})} />旅行到达后自动领取</label>
      <label className="flex gap-2"><input type="checkbox" checked={plan.config.travel_depart} disabled={busy} onChange={e => setPlan({...plan, config:{...plan.config, travel_depart:e.target.checked}})} />每日自动出发（先查询账号旅行状态并选择下方目的地）</label>
      <label className="flex items-center gap-2">北京时间 <input type="time" value={plan.config.daily_time} disabled={busy} onChange={e => setPlan({...plan,config:{...plan.config,daily_time:e.target.value}})} className="rounded border bg-background p-1" /></label>
      <div className="flex flex-wrap gap-4">{accounts.map(a => <label key={a.id} className="flex gap-2"><input type="checkbox" checked={plan.config.account_ids.includes(a.id)} disabled={busy} onChange={e => setPlan({...plan,config:{...plan.config,account_ids:e.target.checked ? [...plan.config.account_ids,a.id] : plan.config.account_ids.filter(id=>id!==a.id)}})} />{a.note || a.nickname || a.id}</label>)}</div>
      <p className="text-muted-foreground">聊天执行默认关闭。启用后仅补足五次对话任务的剩余进度，每账号每次最多 5 条虚构对话；结果待核实时暂停该账号。勾选自动领取后，达标再领奖。关闭开关会停止后续动作，已发送的请求仍会核实。技能和资料库任务不会自动执行。</p>
      <p>最近开始：{beijingTime(plan.state.last_started_ms)} · 完成：{beijingTime(plan.state.finished_ms)}</p>
      {plan.state.error && <p className="text-destructive">{plan.state.error}</p>}
      <Button disabled={busy} onClick={() => void savePlan()}>保存成长计划</Button>
    </CardContent></Card>}
    <div className="flex flex-wrap items-center gap-3"><label htmlFor="growth-account">查询账号</label>
      <select id="growth-account" className="rounded-md border bg-background p-2" value={selected} onChange={e => setSelected(e.target.value)} disabled={busy}>
        <option value="">全部账号</option>{accounts.map(a => <option key={a.id} value={a.id}>{a.note || a.nickname || a.id}</option>)}
      </select><Button onClick={() => void refresh()} disabled={busy}>刷新官方状态</Button>
      <Button onClick={() => void claim()} disabled={busy}>领取已完成奖励</Button>
      <span className="text-sm text-muted-foreground">{busy ? "处理中…" : "支持五次对话、技能、资料库任务；领取前自动查询"}</span>
    </div>
    <Card><CardHeader><CardTitle>旅行</CardTitle></CardHeader><CardContent className="space-y-3 text-sm">
      <p>选择单个账号后查询官方旅行状态。</p>
      <Button disabled={busy || !selected} onClick={() => void travelAction("query")}>查询旅行</Button>
      {travel && <>
        <p>官方状态：{travel.state.state === "arrived" ? "已到达" : travel.state.state === "traveling" ? "旅行中" : travel.state.daily_limit_reached ? "今日已结束" : "可以出发"}{travel.enabled ? "" : " · 活动未开放"}</p>
        {plan && <label className="flex gap-2">目的地 <select className="rounded border bg-background" disabled={busy} value={plan.config.travel_location_id ?? ""} onChange={e=>setPlan({...plan,config:{...plan.config,travel_location_id:e.target.value ? Number(e.target.value) : null}})}>
          <option value="">请选择</option>{travel.locations.map(l=><option key={l.id} value={l.id}>{l.name || l.id}</option>)}
        </select></label>}
        <div className="flex gap-3"><Button disabled={busy || travel.state.state !== "arrived"} onClick={() => void travelAction("claim")}>领取到达奖励</Button>
        <Button disabled={busy || !travel.enabled || travel.state.state !== "idle" || travel.state.daily_limit_reached || !plan?.config.travel_location_id} onClick={() => void travelAction("depart")}>出发一次</Button></div>
      </>}
    </CardContent></Card>
    {message && <p role="status" className="text-sm">{message}</p>}
    {error && <Alert variant="destructive"><AlertTitle>操作未完成</AlertTitle><AlertDescription>{error}</AlertDescription></Alert>}
    {operations.filter(op => op.state === "AwaitingVerification" && (!selected || op.account_id === selected)).map(op => <Alert key={op.operation_id}><AlertTitle>成长操作结果待核实</AlertTitle><AlertDescription>{accounts.find(a => a.id === op.account_id)?.note || op.account_id} · {op.task_code}：{op.error || "等待官方状态确认"}。系统不会重复提交。</AlertDescription></Alert>)}
    {data?.accounts.filter(row => !selected || row.account_id === selected).map(row => {
      const account = accounts.find(a => a.id === row.account_id);
      return <Card key={row.account_id}><CardHeader><CardTitle>{account?.note || account?.nickname || row.account_id}</CardTitle>
        <p className="text-sm text-muted-foreground">更新于 {beijingTime(row.observed_at_ms)}{row.stale ? " · 缓存待刷新" : " · 最近查询成功"}</p>
      </CardHeader><CardContent>
        {row.error && <p className="mb-3 text-sm text-destructive">{row.error}</p>}
        {!row.tasks.length ? <p className="text-sm text-muted-foreground">{row.observed_at_ms === null ? "尚无查询结果" : "官方未返回任务"}</p> : <div className="overflow-x-auto"><table className="w-full text-left text-sm">
          <thead><tr className="border-b"><th className="p-2">任务</th><th className="p-2">进度</th><th className="p-2">状态</th><th className="p-2">有效期至</th></tr></thead>
          <tbody>{row.tasks.map(t => <tr key={t.code} className="border-b"><td className="p-2">{t.title}</td><td className="p-2">{growthTaskProgress(t)}</td><td className="p-2">{growthTaskStatus(t)}</td><td className="p-2">{beijingTime(t.valid_end_ms)}</td></tr>)}</tbody>
        </table></div>}
      </CardContent></Card>;
    })}
  </div>;
}
