import { useCallback, useEffect, useRef, useState } from "react";
import { CalendarClock, CircleCheck, RefreshCw, Save } from "lucide-react";
import { toast } from "sonner";
import * as api from "@/lib/api";
import { displayName } from "@/lib/account-display";
import { accountPlanLabel, beijingTime, canRetryTask, configIssue, currentTask, nextPlanLabel, taskLabel } from "@/lib/manager-scheduler";
import { accountVariant } from "@/lib/variant";
import type { AccountMeta, ManagerSchedulerConfig, ManagerSchedulerSnapshot } from "@/lib/types";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

export default function AutoCheckinPage() {
  const [snapshot, setSnapshot] = useState<ManagerSchedulerSnapshot | null>(null);
  const [accounts, setAccounts] = useState<AccountMeta[]>([]);
  const [draft, setDraft] = useState<ManagerSchedulerConfig | null>(null);
  const [baseRevision, setBaseRevision] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [dirty, setDirty] = useState(false);
  const dirtyRef = useRef(false);
  const busyRef = useRef(false);
  const generation = useRef(0);
  const mounted = useRef(true);
  const load = useCallback(async (discard = false) => {
    if (busyRef.current) return;
    const request = ++generation.current;
    try {
      const [next, result] = await Promise.all([api.getManagerScheduler(), api.getAccounts()]);
      if (!mounted.current || request !== generation.current) return;
      setSnapshot(next); setAccounts(result.accounts);
      if (discard || !dirtyRef.current) {
        setDraft(next.config); setBaseRevision(next.revision); setDirty(false); dirtyRef.current = false; setError(null);
      }
    } catch (cause) {
      if (mounted.current && request === generation.current) setError(api.asError(cause));
    }
  }, []);
  useEffect(() => {
    mounted.current = true;
    void load();
    const timer = window.setInterval(() => void load(), 30_000);
    return () => { mounted.current = false; ++generation.current; window.clearInterval(timer); };
  }, [load]);
  function edit(next: ManagerSchedulerConfig) { setDraft(next); setDirty(true); dirtyRef.current = true; }
  async function save() {
    if (!draft || busyRef.current) return;
    const issue = configIssue(draft);
    if (issue) { toast.error(issue); return; }
    busyRef.current = true; setSaving(true); ++generation.current;
    try {
      const next = await api.saveManagerScheduler(draft, baseRevision);
      if (!mounted.current) return;
      setSnapshot(next); setDraft(next.config); setBaseRevision(next.revision);
      dirtyRef.current = false; setDirty(false); setError(null);
      toast.success("计划设置已保存", { description: next.production_enabled ? undefined : "计划仍暂停，尚不会发送真实签到请求" });
    } catch (cause) {
      if (mounted.current) { setError(api.asError(cause)); toast.error("设置未保存", { description: api.asError(cause) }); }
    } finally { busyRef.current = false; if (mounted.current) setSaving(false); }
  }
  async function run(accountId?: string) {
    try { await api.runManagerCheckin(accountId); await load(); }
    catch (cause) { toast.error("无法执行签到", { description: api.asError(cause) }); }
  }
  const issue = draft ? configIssue(draft) : null;
  const readonly = saving || api.isDemoMode();
  return <div className="mx-auto max-w-6xl space-y-6 p-6 md:p-8">
    <div className="flex flex-wrap items-center justify-between gap-4">
      <div><h1 className="flex items-center gap-2 text-2xl font-semibold"><CalendarClock className="size-6" />自动签到</h1><p className="mt-1 text-sm text-muted-foreground">统一使用北京时间 · 关闭应用、睡眠或关机时不会执行</p></div>
      <Button variant="outline" disabled={saving} onClick={() => void load(dirty)}><RefreshCw />{dirty ? "放弃修改并重新加载" : "刷新状态"}</Button>
    </div>
    {api.isDemoMode() && <Alert><AlertTitle>演示数据</AlertTitle><AlertDescription>本页展示虚构账号与记录，设置为只读，不会连接真实账号。</AlertDescription></Alert>}
    {error && <Alert variant="destructive"><AlertTitle>当前数据无法确认</AlertTitle><AlertDescription>{error}。请重新加载后再保存。</AlertDescription></Alert>}
    {snapshot && !snapshot.production_enabled && <Alert><AlertTitle>签到计划暂停</AlertTitle><AlertDescription>{snapshot.blocked_reason}。可以保存计划设置，待能力验证完成后再启用执行。</AlertDescription></Alert>}
    {!draft || !snapshot ? <Card><CardContent className="py-10 text-muted-foreground" role="status">{error ? "加载失败，请重试" : "正在读取计划与账号…"}</CardContent></Card> : <>
      <Card><CardHeader><div className="flex flex-wrap items-center justify-between gap-3"><div><CardTitle>每日计划</CardTitle><CardDescription className="mt-1">业务日期 {snapshot.business_date} · {snapshot.background_active ? "后台已接入" : "未检测到后台运行"}</CardDescription></div><Badge variant="secondary">{draft.enabled ? "已配置" : "未开启"}{!snapshot.production_enabled ? " · 执行暂停" : ""}</Badge></div></CardHeader>
        <CardContent className="space-y-5">
          <div className="flex items-center justify-between gap-4"><Label htmlFor="daily-enabled">启用每日计划</Label><Switch id="daily-enabled" checked={draft.enabled} disabled={readonly || !!error} onCheckedChange={enabled => edit({ ...draft, enabled })} /></div>
          <div className="grid gap-4 sm:grid-cols-3"><div className="space-y-2"><Label htmlFor="daily-time">每天执行时间（北京时间）</Label><Input id="daily-time" type="time" value={draft.daily_time} disabled={readonly || !!error} onChange={e => edit({ ...draft, daily_time: e.target.value })} /></div>
            <div className="space-y-2"><Label htmlFor="daily-attempts">每日最多尝试次数</Label><Input id="daily-attempts" type="number" min={1} max={3} value={draft.max_daily_attempts} disabled={readonly || !!error} onChange={e => edit({ ...draft, max_daily_attempts: Number(e.target.value) })} /></div>
            <div className="space-y-2"><Label htmlFor="daily-retry">重试间隔（秒）</Label><Input id="daily-retry" type="number" min={30} max={86400} value={draft.retry_delay_seconds} disabled={readonly || !!error} onChange={e => edit({ ...draft, retry_delay_seconds: Number(e.target.value) })} /></div></div>
          <div className="flex items-center justify-between gap-4"><div><Label htmlFor="daily-catchup">允许当日补执行</Label><p className="mt-1 text-xs text-muted-foreground">重新打开应用或唤醒后处理当天错过的计划；不补签昨天</p></div><Switch id="daily-catchup" checked={draft.catch_up} disabled={readonly || !!error} onCheckedChange={catch_up => edit({ ...draft, catch_up })} /></div>
          <p className="text-sm text-muted-foreground">下次预计执行：{nextPlanLabel(snapshot)}</p>
          <p className="text-xs text-muted-foreground">成功后当天不再重复提交；提交结果未确认时仅核实状态。Windows 登录自启动尚未接入。</p>
          {issue && <p role="alert" className="text-sm text-destructive">{issue}</p>}
          <div className="flex flex-wrap items-center gap-3"><Button disabled={!dirty || readonly || !!error || !!issue} onClick={() => void save()}><Save />{saving ? "正在保存…" : "保存计划"}</Button><span className="text-xs text-muted-foreground">{dirty ? "有未保存修改" : "设置与已保存配置一致"}</span><Button variant="outline" disabled={!snapshot.production_enabled || saving || !!error} title={snapshot.blocked_reason} onClick={() => void run()}><CircleCheck />全部立即签到</Button></div>
        </CardContent></Card>
      <Card><CardHeader><CardTitle>逐账号计划</CardTitle><CardDescription>只有国内版支持签到。账号停用与计划开关分别生效。</CardDescription></CardHeader><CardContent>
        {accounts.length === 0 ? <p className="py-5 text-sm text-muted-foreground">暂无账号，请先在账号管理添加账号。</p> : <div className="divide-y">{accounts.map(account => {
          const supported = accountVariant(account) === "cn" && (account.provider ?? "workbuddy") === "workbuddy";
          const last = currentTask(snapshot, account);
          return <div key={account.id} className="flex flex-wrap items-center gap-3 py-4"><div className="min-w-0 flex-1"><div className="flex items-center gap-2"><span className="truncate font-medium">{displayName(account)}</span><Badge variant="outline">{accountVariant(account) === "cn" ? "国内版" : "国际版"}</Badge></div><p className="mt-1 text-sm text-muted-foreground">{taskLabel(last)}</p><p className="mt-1 text-xs text-muted-foreground">{accountPlanLabel({ ...snapshot, config: draft }, account)}</p></div>
            <Button size="sm" variant="outline" disabled={!snapshot.production_enabled || !supported || account.is_enabled === false || saving || !!error} title={snapshot.blocked_reason} onClick={() => void run(account.id)}>立即签到</Button>
            <Switch aria-label={`${displayName(account)}参加每日计划`} checked={supported && draft.account_enabled[account.id] !== false} disabled={readonly || !!error || !supported || account.is_enabled === false} onCheckedChange={allowed => edit({ ...draft, account_enabled: { ...draft.account_enabled, [account.id]: allowed } })} />
          </div>;
        })}</div>}
      </CardContent></Card>
      <Card><CardHeader><CardTitle>执行记录</CardTitle><CardDescription>最近 {snapshot.history_limit} 个账号日任务。时间为北京时间；没有记录不代表已经签到。</CardDescription></CardHeader><CardContent>
        {snapshot.tasks.length === 0 ? <p className="py-5 text-sm text-muted-foreground">暂无执行记录。计划暂停期间不会产生成功记录。</p> : <div className="overflow-x-auto"><Table><TableHeader><TableRow><TableHead>日期 / 账号</TableHead><TableHead>结果</TableHead><TableHead>实际开始</TableHead><TableHead>尝试 / 核实</TableHead><TableHead>操作</TableHead></TableRow></TableHeader><TableBody>{snapshot.tasks.map(task => <TableRow key={`${task.provider}:${task.region}:${task.account_id}:${task.business_date}`}><TableCell>{task.business_date}<br /><span className="text-muted-foreground">{accounts.find(a => a.id === task.account_id)?.nickname ?? task.account_id}</span></TableCell><TableCell>{taskLabel(task)}</TableCell><TableCell>{beijingTime(task.last_started_at_ms)}</TableCell><TableCell>{task.attempts} / {task.verification_attempts}</TableCell><TableCell><Button size="sm" variant="ghost" disabled={!canRetryTask(snapshot, task) || accounts.find(a => a.id === task.account_id)?.is_enabled === false || saving || !!error} title={task.uncertain ? "结果未确认，不能重复提交" : snapshot.blocked_reason} onClick={() => void run(task.account_id)}>失败重试</Button></TableCell></TableRow>)}</TableBody></Table></div>}
      </CardContent></Card>
    </>}
  </div>;
}
