import { useCallback, useEffect, useState } from "react";
import * as api from "@/lib/api";
import { beijingTime, taskLabel } from "@/lib/manager-scheduler";
import type { ManagerLogs } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
export default function OperationLogsPage() {
 const [data,setData]=useState<ManagerLogs|null>(null); const [error,setError]=useState<string|null>(null);
 const load=useCallback(async()=>{try {setData(await api.getManagerLogs());setError(null);}catch(cause){setError(api.asError(cause));}},[]);
 useEffect(()=>{void load();},[load]);
 return <div className="mx-auto max-w-5xl space-y-6 p-6 md:p-8"><div className="flex items-center justify-between gap-3"><div><h1 className="text-2xl font-semibold">操作日志</h1><p className="mt-2 text-sm text-muted-foreground">本机执行记录与应用提示 · 时间为北京时间</p></div><Button variant="outline" onClick={()=>void load()}>刷新日志</Button></div>
 {error&&<Alert variant="destructive"><AlertTitle>日志无法读取</AlertTitle><AlertDescription>{error}</AlertDescription></Alert>}
 <Card><CardHeader><CardTitle>签到执行记录</CardTitle></CardHeader><CardContent className="divide-y">{!data?.tasks.length?<p className="py-5 text-muted-foreground">暂无执行记录；计划暂停不代表签到成功。</p>:data.tasks.map(task=><div key={`${task.provider}:${task.region}:${task.account_id}:${task.business_date}`} className="py-3"><p>{task.business_date} · {task.account_id} · {taskLabel(task)}</p><p className="mt-1 text-xs text-muted-foreground">开始 {beijingTime(task.last_started_at_ms)} · 尝试 {task.attempts} 次 / 核实 {task.verification_attempts} 次</p></div>)}</CardContent></Card>
 <Card><CardHeader><CardTitle>应用提示</CardTitle></CardHeader><CardContent className="divide-y">{!data?.notifications.length?<p className="py-5 text-muted-foreground">暂无应用提示。会话操作的详细备份记录在对应会话流程中查看。</p>:data.notifications.map((item,index)=><div key={`${item.at}:${index}`} className="py-3"><p>{item.title}</p><p className="mt-1 break-words text-sm text-muted-foreground">{item.description}</p><p className="mt-1 text-xs text-muted-foreground">{beijingTime(item.at)} · {item.level}</p></div>)}</CardContent></Card>
 </div>;
}
