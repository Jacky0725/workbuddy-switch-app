import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import * as api from "@/lib/api";
import type { ManagerBackupInfo, ManagerRestorePreview } from "@/lib/types";
import { beijingTime } from "@/lib/manager-scheduler";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";

export default function BackupsPage() {
 const [items,setItems]=useState<ManagerBackupInfo[]>([]);
 const [error,setError]=useState<string|null>(null);
 const [busy,setBusy]=useState(false);
 const [preview,setPreview]=useState<ManagerRestorePreview|null>(null);
 const load=useCallback(async()=>{try {setItems(await api.listManagerBackups());setError(null);}catch(cause){setError(api.asError(cause));}},[]);
 useEffect(()=>{void load();},[load]);
 async function create() { if(busy)return;setBusy(true);try {await api.createManagerBackup();await load();toast.success("加密备份已创建并核实");}catch(cause){toast.error("备份未完成",{description:api.asError(cause)});}finally{setBusy(false);} }
 async function inspect(id:string) {setBusy(true);try{setPreview(await api.previewManagerRestore(id));}catch(cause){toast.error("无法预览恢复",{description:api.asError(cause)});}finally{setBusy(false);} }
 async function restore() {if(!preview||busy)return;setBusy(true);try {const report=await api.restoreManagerBackup(preview);setPreview(null);await load();toast.success(`已恢复 ${report.restored_accounts} 个账号`,{description:"恢复前的账号库已另存为加密备份，可从列表回滚。"});}catch(cause){toast.error("恢复未完成",{description:api.asError(cause)});}finally{setBusy(false);} }
 return <div className="mx-auto max-w-5xl space-y-6 p-6 md:p-8">
  <div className="flex flex-wrap items-center justify-between gap-3"><div><h1 className="text-2xl font-semibold">备份与恢复</h1><p className="mt-2 text-sm text-muted-foreground">Manager 账号库的加密备份 · 恢复前先预览并确认</p></div><div className="flex gap-2"><Button variant="outline" disabled={busy} onClick={()=>void load()}>刷新列表</Button><Button disabled={busy||api.isDemoMode()} onClick={()=>void create()}>创建加密备份</Button></div></div>
  <Alert><AlertTitle>备份范围与恢复条件</AlertTitle><AlertDescription>包含账号资料及加密凭证，仅供原 Windows 用户在可解密的环境中恢复。跨用户或跨电脑迁移请重新授权。恢复不会切换官方登录，不会修改会话数据库或每日计划；生产会话恢复仍等待版本兼容验证。</AlertDescription></Alert>
  {api.isDemoMode()&&<p className="text-sm text-muted-foreground">演示模式不会创建或恢复真实备份。</p>}
  {error&&<Alert variant="destructive"><AlertTitle>备份列表不可读取</AlertTitle><AlertDescription>{error}</AlertDescription></Alert>}
  <Card><CardContent className="divide-y">{items.length===0?<p className="py-8 text-sm text-muted-foreground">暂无账号加密备份。普通账号导出只包含资料，不能替代凭证备份。</p>:items.map(item=><div key={item.id} className="flex flex-wrap items-center gap-3 py-4"><div className="min-w-0 flex-1"><p>{beijingTime(item.created_at_ms)} · {item.account_count??"未知"} 个账号</p><p className="mt-1 break-all text-xs text-muted-foreground">{item.id}</p>{item.error&&<p className="mt-1 text-sm text-destructive">{item.error}</p>}</div><Button variant="outline" disabled={busy||!!item.error||api.isDemoMode()} onClick={()=>void inspect(item.id)}>预览恢复</Button></div>)}</CardContent></Card>
  <Dialog open={!!preview} onOpenChange={open=>{if(!open&&!busy)setPreview(null);}}><DialogContent><DialogHeader><DialogTitle>确认替换 Manager 账号库</DialogTitle><DialogDescription>将恢复 {preview?.account_count} 个账号。{preview?.scope}。恢复前会创建当前账号库的回滚备份；备份失败会取消恢复。</DialogDescription></DialogHeader><DialogFooter><Button variant="outline" disabled={busy} onClick={()=>setPreview(null)}>取消</Button><Button disabled={busy} onClick={()=>void restore()}>{busy?"正在恢复…":"确认恢复"}</Button></DialogFooter></DialogContent></Dialog>
 </div>;
}
