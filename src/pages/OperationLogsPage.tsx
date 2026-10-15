import { useCallback, useEffect, useState } from "react";
import * as api from "@/lib/api";
import { beijingTime, taskLabel } from "@/lib/manager-scheduler";
import type { GrowthOperation, ManagerLogs } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
export default function OperationLogsPage() {
 const [data,setData]=useState<ManagerLogs|null>(null); const [error,setError]=useState<string|null>(null);
 const [growth,setGrowth]=useState<GrowthOperation[]>([]); const [filter,setFilter]=useState("all");
 const [growthError,setGrowthError]=useState<string|null>(null);
 const load=useCallback(async()=>{try {setData(await api.getManagerLogs());setError(null);}catch(cause){setError(api.asError(cause));}try{setGrowth(await api.getGrowthOperations());setGrowthError(null);}catch(cause){setGrowthError(api.asError(cause));}},[]);
 useEffect(()=>{void load();},[load]);
 return <div className="mx-auto max-w-5xl space-y-6 p-6 md:p-8"><div className="flex items-center justify-between gap-3"><div><h1 className="text-2xl font-semibold">操作日志</h1><p className="mt-2 text-sm text-muted-foreground">本机执行记录与应用提示 · 时间为北京时间</p></div><Button variant="outline" onClick={()=>void load()}>刷新日志</Button></div>
 {error&&<Alert variant="destructive"><AlertTitle>日志无法读取</AlertTitle><AlertDescription>{error}</AlertDescription></Alert>}
 <Card><CardHeader><CardTitle>成长与旅行记录</CardTitle><label className="text-sm">筛选 <select className="rounded border bg-background p-1" value={filter} onChange={e=>setFilter(e.target.value)}><option value="all">全部</option><option value="claim">普通领奖</option><option value="travel">旅行</option><option value="pending">待核实</option></select></label></CardHeader><CardContent className="divide-y">
 {growthError&&<p className="py-3 text-destructive">成长记录未读取：{growthError}</p>}
 {!growth.length?<p className="py-5 text-muted-foreground">暂无成长活动提交记录。</p>:growth.filter(op=>filter==="all"||filter==="pending"&&op.state==="AwaitingVerification"||filter==="travel"&&op.task_code.startsWith("travel_")||filter==="claim"&&!op.task_code.startsWith("travel_")).sort((a,b)=>b.submitted_at_ms-a.submitted_at_ms).slice(0,100).map(op=><div key={op.operation_id} className="py-3"><p>{op.account_id} · {op.task_code} · {op.state==="Verified"?"服务端已核实":"结果待核实，不重复提交"}</p><p className="mt-1 text-xs text-muted-foreground">提交 {beijingTime(op.submitted_at_ms)} · 核实 {beijingTime(op.verified_at_ms)}</p>{op.error&&<p className="mt-1 text-sm text-destructive">{op.error}</p>}</div>)}
 </CardContent></Card>
 <Card><CardHeader><CardTitle>签到执行记录</CardTitle></CardHeader><CardContent className="divide-y">{!data?.tasks.length?<p className="py-5 text-muted-foreground">暂无执行记录；计划暂停不代表签到成功。</p>:data.tasks.map(task=><div key={`${task.provider}:${task.region}:${task.account_id}:${task.business_date}`} className="py-3"><p>{task.business_date} · {task.account_id} · {taskLabel(task)}</p><p className="mt-1 text-xs text-muted-foreground">开始 {beijingTime(task.last_started_at_ms)} · 尝试 {task.attempts} 次 / 核实 {task.verification_attempts} 次</p></div>)}</CardContent></Card>
 <Card><CardHeader><CardTitle>应用提示</CardTitle></CardHeader><CardContent className="divide-y">{!data?.notifications.length?<p className="py-5 text-muted-foreground">暂无应用提示。会话操作的详细备份记录在对应会话流程中查看。</p>:data.notifications.map((item,index)=><div key={`${item.at}:${index}`} className="py-3"><p>{item.title}</p><p className="mt-1 break-words text-sm text-muted-foreground">{item.description}</p><p className="mt-1 text-xs text-muted-foreground">{beijingTime(item.at)} · {item.level}</p></div>)}</CardContent></Card>
 </div>;
}
