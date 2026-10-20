//! Bounded, explicitly requested chat-task execution. Not enabled by the scheduler.
//! A real official ACP prompt is required; activity events are never synthesized.
use super::{
    account, config,
    growth::{self, GrowthTask},
    growth_automation::{self, ActivityOperation, Journal, OperationState},
};
use fs2::FileExt;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{future::Future, path::Path, pin::Pin};

type Tasks<'a> = Pin<Box<dyn Future<Output = Result<Vec<GrowthTask>, String>> + Send + 'a>>;
type Action<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;
trait Api: Sync {
    fn list(&self) -> Tasks<'_>;
    fn preflight(&self) -> Action<'_>;
    fn accept(&self) -> Action<'_>;
    fn chat(&self) -> Action<'_>;
}
struct Official<'a> {
    account: &'a Value,
    root: &'a Path,
    automatic: bool,
}
impl Official<'_> {
    fn current(&self) -> Result<Value, String> {
        if self.automatic
            && !super::growth_schedule::allows_chat(
                self.root,
                self.account["id"].as_str().unwrap_or(""),
            )?
        {
            return Err("自动聊天已关闭或账号已移除，停止后续动作".into());
        }
        let current = account::find_account(self.account["id"].as_str().ok_or("账号标识缺失")?)
            .ok_or("账号已移除")?;
        if growth::identity(&current) != growth::identity(self.account)
            || current["access_token"] != self.account["access_token"]
        {
            return Err("账号已变化，已停止执行".into());
        }
        if let Some(error) = growth::query_error(&current) {
            return Err(error.into());
        }
        Ok(current)
    }
}
impl Api for Official<'_> {
    fn list(&self) -> Tasks<'_> {
        Box::pin(growth::query(self.account))
    }
    fn preflight(&self) -> Action<'_> {
        Box::pin(async move {
            self.current()?;
            super::growth_runtime::chat(self.root, self.account, true).await
        })
    }
    fn chat(&self) -> Action<'_> {
        Box::pin(async move {
            self.current()?;
            super::growth_runtime::chat(self.root, self.account, false).await
        })
    }
    fn accept(&self) -> Action<'_> {
        Box::pin(async move {
            self.current()?;
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .map_err(|_| "接受任务连接不可创建")?;
            let mut request = client
                .post("https://www.workbuddy.cn/activity/growth/tasks/accept")
                .header("Origin", "https://www.workbuddy.cn")
                .header("Referer", "https://www.workbuddy.cn/profile/growth-center")
                .header("X-Client-Platform", "web")
                .json(&serde_json::json!({"task_codes":["chat_5"]}));
            for (k, v) in account::build_auth_headers(self.account) {
                request = request.header(k, v);
            }
            let mut response = request.send().await.map_err(|_| "接受结果未知，禁止重发")?;
            if response.status() != 200 {
                return Err("接受任务被拒绝，已停止提交".into());
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| "接受响应中断，等待核实")?
            {
                if bytes.len() + chunk.len() > 128 * 1024 {
                    return Err("接受响应超限，等待核实".into());
                }
                bytes.extend(chunk);
            }
            let body: Value = serde_json::from_slice(&bytes).map_err(|_| "接受响应无法识别")?;
            if body["code"] != 0 {
                return Err("接受任务业务状态未通过，已停止提交".into());
            }
            Ok(())
        })
    }
}
#[derive(Default, Serialize)]
pub struct ExecutionReport {
    pub accepted: bool,
    pub actions_verified: usize,
    pub task_claimed: bool,
    pub awaiting_verification: bool,
    pub budget_exhausted: bool,
    pub skipped: bool,
    pub halted: bool,
}
pub(crate) fn blocks_other_accounts(op: &ActivityOperation) -> bool {
    !(matches!(
        op.task_code.as_str(),
        "chat_5/execute/0"
            | "chat_5/execute/1"
            | "chat_5/execute/2"
            | "chat_5/execute/3"
            | "chat_5/execute/4"
    ) && op.submission_acknowledged == Some(true)
        && op.error.as_deref() == Some("等待官方进度核实，禁止重发"))
}
fn key(a: &Value, t: &GrowthTask, kind: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(format!("{}:{kind}", growth_automation::operation_key(a, t)))
    )
}
fn current(t: &GrowthTask) -> bool {
    t.code == "chat_5"
        && t.task_type == "single"
        && t.validity_known
        && t.locked == Some(false)
        && t.valid_start_ms.is_none_or(|v| v <= config::now_ms())
        && t.valid_end_ms.is_none_or(|v| v > config::now_ms())
        && (t.accept_status == "not_accepted" || t.target == Some(5))
}
async fn task(api: &impl Api) -> Result<GrowthTask, String> {
    api.list()
        .await?
        .into_iter()
        .find(|t| t.code == "chat_5")
        .ok_or("官方未返回聊天任务".into())
}
fn mark(
    root: &Path,
    j: &mut Journal,
    a: &Value,
    t: &GrowthTask,
    kind: &str,
) -> Result<String, String> {
    if j.operations.len() >= 4096 {
        return Err("操作记录超限，已停止提交".into());
    }
    let k = key(a, t, kind);
    j.operations.insert(
        k.clone(),
        ActivityOperation {
            operation_id: uuid::Uuid::new_v4().to_string(),
            account_id: a["id"].as_str().ok_or("账号标识缺失")?.into(),
            task_code: format!("chat_5/{kind}"),
            state: OperationState::AwaitingVerification,
            submitted_at_ms: config::now_ms(),
            verified_at_ms: None,
            submission_acknowledged: None,
            error: None,
        },
    );
    growth_automation::save_journal(root, j)?;
    Ok(k)
}
fn verify(root: &Path, j: &mut Journal, k: &str) -> Result<(), String> {
    let op = j.operations.get_mut(k).ok_or("操作记录缺失")?;
    op.state = OperationState::Verified;
    op.verified_at_ms = Some(config::now_ms());
    op.error = None;
    growth_automation::save_journal(root, j)
}
fn acknowledge(
    root: &Path,
    j: &mut Journal,
    k: &str,
    result: Result<(), String>,
) -> Result<(), String> {
    let op = j.operations.get_mut(k).ok_or("操作记录缺失")?;
    op.submission_acknowledged = Some(result.is_ok());
    op.error = Some(
        result
            .err()
            .unwrap_or_else(|| "等待官方进度核实，禁止重发".into()),
    );
    growth_automation::save_journal(root, j)
}
async fn drive(
    root: &Path,
    a: &Value,
    api: &impl Api,
    budget: u8,
) -> Result<ExecutionReport, String> {
    let mut report = ExecutionReport::default();
    let mut t = task(api).await?;
    let mut j = growth_automation::read_journal(root)?;
    // Reconcile persisted submissions before allowing any further side effect.
    let accept_key = key(a, &t, "accept");
    if let Some(op) = j.operations.get(&accept_key) {
        if op.state == OperationState::AwaitingVerification {
            if matches!(
                t.accept_status.as_str(),
                "accepted" | "in_progress" | "completed" | "claimed"
            ) {
                verify(root, &mut j, &accept_key)?;
            } else {
                report.awaiting_verification = true;
                return Ok(report);
            }
        }
    }
    for baseline in 0..5 {
        let k = key(a, &t, &format!("execute/{baseline}"));
        if j.operations
            .get(&k)
            .is_some_and(|op| op.state == OperationState::AwaitingVerification)
        {
            if t.current.is_some_and(|p| p > baseline) {
                verify(root, &mut j, &k)?;
            } else {
                report.awaiting_verification = true;
                return Ok(report);
            }
        }
    }
    if t.accept_status == "claimed" {
        let claim_key = growth_automation::operation_key(a, &t);
        if j.operations
            .get(&claim_key)
            .is_some_and(|op| op.state == OperationState::AwaitingVerification)
        {
            verify(root, &mut j, &claim_key)?;
        }
    }
    if j.operations.values().any(|op| {
        if op.state != OperationState::AwaitingVerification {
            return false;
        }
        // A completed official prompt with delayed progress pauses its own
        // account. Rejection/unknown runtime results and other writes still
        // block the batch; changing accounts must never retry those actions.
        op.account_id == a["id"].as_str().unwrap_or("") || blocks_other_accounts(op)
    }) {
        report.awaiting_verification = true;
        return Ok(report);
    }
    if t.accept_status == "claimed" {
        report.task_claimed = true;
        return Ok(report);
    }
    let prior_failure = std::iter::once("accept".to_string())
        .chain((0..5).map(|n| format!("execute/{n}")))
        .any(|kind| {
            j.operations
                .get(&key(a, &t, &kind))
                .is_some_and(|op| op.submission_acknowledged == Some(false))
        });
    if prior_failure {
        report.halted = true;
        return Ok(report);
    }
    if t.accept_status == "completed" {
        return Ok(report);
    }
    if !current(&t) {
        report.skipped = true;
        return Ok(report);
    }
    if budget == 0 {
        return Ok(report);
    }
    api.preflight().await?;
    if t.accept_status == "not_accepted" {
        if j.operations.contains_key(&accept_key) {
            report.awaiting_verification = true;
            return Ok(report);
        }
        let k = mark(root, &mut j, a, &t, "accept")?;
        let accepted = api.accept().await;
        let rejected = accepted.is_err();
        acknowledge(root, &mut j, &k, accepted)?;
        t = task(api).await?;
        if !matches!(
            t.accept_status.as_str(),
            "accepted" | "in_progress" | "completed" | "claimed"
        ) {
            report.awaiting_verification = true;
            return Ok(report);
        }
        verify(root, &mut j, &k)?;
        report.accepted = true;
        if rejected {
            report.halted = true;
            return Ok(report);
        }
    }
    for _ in 0..budget {
        // Always use current remaining progress; delayed counts never trigger another chat.
        t = task(api).await?;
        if matches!(t.accept_status.as_str(), "completed" | "claimed") {
            return Ok(report);
        }
        if !current(&t) || !matches!(t.accept_status.as_str(), "accepted" | "in_progress") {
            report.skipped = true;
            return Ok(report);
        }
        let baseline = t
            .current
            .filter(|p| *p < 5)
            .ok_or("官方进度无法可靠核实，已停止发送")?;
        let kind = format!("execute/{baseline}");
        let k = key(a, &t, &kind);
        if j.operations.contains_key(&k) {
            report.awaiting_verification = true;
            return Ok(report);
        }
        mark(root, &mut j, a, &t, &kind)?;
        let sent = api.chat().await;
        let rejected = sent.is_err();
        acknowledge(root, &mut j, &k, sent)?;
        t = task(api).await?;
        if t.current.is_none_or(|p| p <= baseline) {
            report.awaiting_verification = true;
            return Ok(report);
        }
        verify(root, &mut j, &k)?;
        report.actions_verified += 1;
        if rejected {
            report.halted = true;
            return Ok(report);
        }
    }
    report.budget_exhausted = t.current.is_some_and(|p| p < 5);
    Ok(report)
}
/// Explicit finite execution entry point. Zero only reconciles through official GETs.
pub fn run(
    account_id: &str,
    max_actions: u8,
) -> Pin<Box<dyn Future<Output = Result<ExecutionReport, String>> + Send + '_>> {
    Box::pin(async move {
        if max_actions > 5 {
            return Err("单次执行上限必须为 0 至 5 条；0 仅核实".into());
        }
        let root = config::store_dir();
        std::fs::create_dir_all(root.join("locks")).map_err(|_| "成长锁目录不可创建")?;
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("locks/growth.lock"))
            .map_err(|_| "成长锁不可读取")?;
        lock.try_lock_exclusive().map_err(|_| "成长操作正在运行")?;
        run_locked(&root, account_id, max_actions, false).await
    })
}
/// Caller holds the shared growth lock. Automatic calls recheck configuration
/// before each side effect; zero budget is always reconciliation only.
pub(crate) fn run_locked<'a>(
    root: &'a Path,
    account_id: &'a str,
    max_actions: u8,
    automatic: bool,
) -> Pin<Box<dyn Future<Output = Result<ExecutionReport, String>> + Send + 'a>> {
    Box::pin(async move {
        if max_actions > 5 {
            return Err("单次执行上限不得超过 5 条".into());
        }
        let a = account::find_account(account_id).ok_or("账号不存在")?;
        let a = growth::prepare_account(&a).await;
        if let Some(error) = growth::query_error(&a) {
            return Err(error.into());
        }
        let api = Official {
            account: &a,
            root,
            automatic,
        };
        let mut result = drive(root, &a, &api, max_actions).await?;
        if !result.awaiting_verification && !result.skipped && !result.halted {
            let t = task(&api).await?;
            if max_actions > 0
                && t.accept_status == "completed"
                && (!automatic
                    || (super::growth_schedule::allows_chat(root, account_id)?
                        && super::growth_schedule::allows(account_id)?))
            {
                let ids = vec![account_id.to_string()];
                let claim = growth_automation::run_claims_locked(
                    root,
                    Some(account_id),
                    if automatic { Some(&ids) } else { None },
                )
                .await?;
                result.awaiting_verification |= claim.awaiting_verification > 0;
                result.halted |= claim.halted || !claim.errors.is_empty();
            }
            result.task_claimed = task(&api).await?.accept_status == "claimed";
        }
        Ok(result)
    })
}

/// Zero-prompt compatibility check; does not accept or complete an activity task.
pub async fn preflight(account_id: &str) -> Result<(), String> {
    let root = config::store_dir();
    std::fs::create_dir_all(root.join("locks")).map_err(|_| "成长锁目录不可创建")?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("locks/growth.lock"))
        .map_err(|_| "成长锁不可读取")?;
    lock.try_lock_exclusive().map_err(|_| "成长操作正在运行")?;
    let a = account::find_account(account_id).ok_or("账号不存在")?;
    let a = growth::prepare_account(&a).await;
    if let Some(error) = growth::query_error(&a) {
        return Err(error.into());
    }
    super::growth_runtime::chat(&root, &a, true).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };
    #[tokio::test]
    async fn disabled_automatic_chat_stops_before_account_or_runtime_access() {
        let root = std::env::temp_dir().join(format!("growth-cancel-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let account = json!({"id":"fictional-missing-account"});
        let api = Official {
            account: &account,
            root: &root,
            automatic: true,
        };
        assert!(api.chat().await.unwrap_err().contains("自动聊天已关闭"));
        assert!(api.accept().await.unwrap_err().contains("自动聊天已关闭"));
        assert!(!root.join("growth-runtime").exists());
        assert!(!root.join("activity_operations.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    struct Fake {
        t: Mutex<GrowthTask>,
        calls: AtomicUsize,
        increment: bool,
        fail: bool,
        accept_fails: bool,
        accepts: AtomicUsize,
    }
    impl Api for Fake {
        fn list(&self) -> Tasks<'_> {
            Box::pin(async { Ok(vec![self.t.lock().unwrap().clone()]) })
        }
        fn preflight(&self) -> Action<'_> {
            Box::pin(async { Ok(()) })
        }
        fn accept(&self) -> Action<'_> {
            Box::pin(async {
                self.accepts.fetch_add(1, Ordering::SeqCst);
                if self.accept_fails {
                    return Err("accept result unknown".into());
                }
                let mut t = self.t.lock().unwrap();
                t.accept_status = "accepted".into();
                t.current = Some(0);
                t.target = Some(5);
                Ok(())
            })
        }
        fn chat(&self) -> Action<'_> {
            Box::pin(async {
                self.calls.fetch_add(1, Ordering::SeqCst);
                if self.increment {
                    let mut t = self.t.lock().unwrap();
                    t.current = Some(t.current.unwrap() + 1);
                    t.accept_status = if t.current == Some(5) {
                        "completed"
                    } else {
                        "in_progress"
                    }
                    .into();
                }
                if self.fail {
                    Err("unknown result".into())
                } else {
                    Ok(())
                }
            })
        }
    }
    fn setup() -> (std::path::PathBuf, Value, Fake) {
        let root = std::env::temp_dir().join(format!("growth-execution-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let t=super::super::growth::parse_tasks(&json!({"tasks":[{"task_code":"chat_5","task_type":"single","accept_status":"not_accepted","locked":false,"progress":null,"valid_start":null,"valid_end":null}]})).unwrap().remove(0);
        (
            root,
            json!({"id":"test","uid":"fictional"}),
            Fake {
                t: Mutex::new(t),
                calls: 0.into(),
                increment: true,
                fail: false,
                accept_fails: false,
                accepts: 0.into(),
            },
        )
    }
    #[tokio::test]
    async fn budget_one_accepts_and_executes_only_one_action() {
        let (root, a, api) = setup();
        let r = drive(&root, &a, &api, 1).await.unwrap();
        assert!(r.accepted);
        assert_eq!(r.actions_verified, 1);
        assert!(r.budget_exhausted);
        assert_eq!(api.calls.load(Ordering::SeqCst), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn other_accounts_acknowledged_chat_does_not_block_but_unknown_result_does() {
        let (root, a, api) = setup();
        let other = json!({"id":"other","uid":"other-fictional"});
        let t = api.t.lock().unwrap().clone();
        let mut journal = growth_automation::read_journal(&root).unwrap();
        let k = mark(&root, &mut journal, &other, &t, "execute/3").unwrap();
        acknowledge(&root, &mut journal, &k, Ok(())).unwrap();
        let r = drive(&root, &a, &api, 1).await.unwrap();
        assert_eq!(r.actions_verified, 1);
        let mut journal = growth_automation::read_journal(&root).unwrap();
        assert_eq!(
            journal.operations[&k].state,
            OperationState::AwaitingVerification
        );
        acknowledge(&root, &mut journal, &k, Err("unknown result".into())).unwrap();
        let r = drive(&root, &a, &api, 1).await.unwrap();
        assert!(r.awaiting_verification);
        assert_eq!(api.calls.load(Ordering::SeqCst), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn five_actions_stop_on_target_and_repeat_does_not_send() {
        let (root, a, api) = setup();
        let r = drive(&root, &a, &api, 5).await.unwrap();
        assert_eq!(r.actions_verified, 5);
        let r = drive(&root, &a, &api, 5).await.unwrap();
        assert_eq!(r.actions_verified, 0);
        assert_eq!(api.calls.load(Ordering::SeqCst), 5);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn delayed_progress_and_restart_never_replay_unknown_chat() {
        let (root, a, mut api) = setup();
        api.increment = false;
        api.fail = true;
        assert!(
            drive(&root, &a, &api, 5)
                .await
                .unwrap()
                .awaiting_verification
        );
        assert!(
            drive(&root, &a, &api, 5)
                .await
                .unwrap()
                .awaiting_verification
        );
        assert_eq!(api.calls.load(Ordering::SeqCst), 1);
        api.t.lock().unwrap().current = Some(1);
        let r = drive(&root, &a, &api, 0).await.unwrap();
        assert!(!r.awaiting_verification);
        assert_eq!(api.calls.load(Ordering::SeqCst), 1);
        let j = growth_automation::read_journal(&root).unwrap();
        assert!(!j
            .operations
            .values()
            .any(|op| op.state == OperationState::AwaitingVerification));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn lost_reply_with_official_progress_is_verified_without_extra_chat() {
        let (root, a, mut api) = setup();
        api.fail = true;
        let r = drive(&root, &a, &api, 1).await.unwrap();
        assert_eq!(r.actions_verified, 1);
        assert!(!r.awaiting_verification);
        assert!(r.halted);
        assert_eq!(api.calls.load(Ordering::SeqCst), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn unknown_accept_is_not_submitted_again_after_restart() {
        let (root, a, mut api) = setup();
        api.accept_fails = true;
        assert!(
            drive(&root, &a, &api, 5)
                .await
                .unwrap()
                .awaiting_verification
        );
        assert!(
            drive(&root, &a, &api, 5)
                .await
                .unwrap()
                .awaiting_verification
        );
        assert_eq!(api.accepts.load(Ordering::SeqCst), 1);
        assert_eq!(api.calls.load(Ordering::SeqCst), 0);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn unsupported_or_expired_tasks_never_accept_or_chat() {
        let (root, a, api) = setup();
        api.t.lock().unwrap().valid_end_ms = Some(1);
        assert!(drive(&root, &a, &api, 5).await.unwrap().skipped);
        assert_eq!(api.accepts.load(Ordering::SeqCst), 0);
        assert_eq!(api.calls.load(Ordering::SeqCst), 0);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn zero_budget_never_accepts_or_sends_even_for_an_eligible_task() {
        let (root, a, api) = setup();
        let result = drive(&root, &a, &api, 0).await.unwrap();
        assert!(!result.accepted);
        assert_eq!(api.accepts.load(Ordering::SeqCst), 0);
        assert_eq!(api.calls.load(Ordering::SeqCst), 0);
        assert!(growth_automation::read_journal(&root)
            .unwrap()
            .operations
            .is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn lost_claim_receipt_is_reconciled_from_claimed_without_any_new_action() {
        let (root, a, api) = setup();
        {
            let mut t = api.t.lock().unwrap();
            t.accept_status = "claimed".into();
            t.current = Some(5);
            t.target = Some(5);
        }
        let t = api.t.lock().unwrap().clone();
        let mut j = growth_automation::read_journal(&root).unwrap();
        let k = growth_automation::operation_key(&a, &t);
        j.operations.insert(
            k.clone(),
            ActivityOperation {
                operation_id: "fictional-claim".into(),
                account_id: "test".into(),
                task_code: "chat_5".into(),
                state: OperationState::AwaitingVerification,
                submitted_at_ms: 1,
                verified_at_ms: None,
                submission_acknowledged: None,
                error: Some("unknown result".into()),
            },
        );
        growth_automation::save_journal(&root, &j).unwrap();
        let r = drive(&root, &a, &api, 0).await.unwrap();
        assert!(r.task_claimed);
        assert!(!r.awaiting_verification);
        assert_eq!(
            growth_automation::read_journal(&root).unwrap().operations[&k].state,
            OperationState::Verified
        );
        assert_eq!(api.calls.load(Ordering::SeqCst), 0);
        assert_eq!(api.accepts.load(Ordering::SeqCst), 0);
        std::fs::remove_dir_all(root).unwrap();
    }
}
