//! Small durable coordinator for verified ordinary growth rewards.
//! No event simulation, task execution, redemption, lottery or travel submissions.
use super::{
    account, config,
    credential_store::{durable_write, CapabilityType, CredentialReference, CredentialStore},
    growth::{self, GrowthTask},
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, future::Future, path::Path, pin::Pin, time::Duration};

pub const CLAIM_CODES: &[&str] = &["chat_5", "skill_1"];
type ListFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<GrowthTask>, String>> + Send + 'a>>;
type SubmitFuture<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;
trait ClaimApi: Sync {
    fn list(&self) -> ListFuture<'_>;
    fn submit<'a>(&'a self, code: &'a str) -> SubmitFuture<'a>;
}
struct Official<'a>(&'a Value, bool);
impl ClaimApi for Official<'_> {
    fn list(&self) -> ListFuture<'_> {
        Box::pin(growth::query(self.0))
    }
    fn submit<'a>(&'a self, code: &'a str) -> SubmitFuture<'a> {
        Box::pin(async move {
            if !CLAIM_CODES.contains(&code) {
                return Err("该奖励尚未接入".into());
            }
            if self.1 && !super::growth_schedule::allows(self.0["id"].as_str().unwrap_or(""))? {
                return Err("自动领取已取消，已停止提交".into());
            }
            let current = account::try_load_accounts()
                .map_err(|_| "账号库不可读取")?
                .into_iter()
                .find(|a| a["id"] == self.0["id"])
                .ok_or("账号已移除，已停止提交")?;
            if growth::identity(&current) != growth::identity(self.0)
                || current["access_token"] != self.0["access_token"]
                || current["is_enabled"] == false
                || current["needs_relogin"] == true
                || current["credential_locked"] == true
            {
                return Err("账号状态已变化，已停止提交".into());
            }
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|_| "领取连接初始化失败")?;
            let mut request = client
                .post(format!(
                    "https://www.workbuddy.cn/activity/growth/tasks/{code}/claim"
                ))
                .header("Origin", "https://www.workbuddy.cn")
                .header("Referer", "https://www.workbuddy.cn/profile/growth-center")
                .header("X-Client-Platform", "web");
            for (key, value) in account::build_auth_headers(self.0) {
                request = request.header(key, value)
            }
            let mut response = request
                .send()
                .await
                .map_err(|_| "领取请求结果不明，等待服务端核实")?;
            let status = response.status().as_u16();
            if status != 200 {
                return Err(format!("领取返回 HTTP {status}，已停止自动提交"));
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| "领取响应中断，等待核实")?
            {
                if bytes.len() + chunk.len() > 128 * 1024 {
                    return Err("领取响应超限，等待核实".into());
                }
                bytes.extend_from_slice(&chunk);
            }
            let body: Value =
                serde_json::from_slice(&bytes).map_err(|_| "领取响应未识别，等待核实")?;
            if body["code"].as_i64() != Some(0) {
                return Err("领取业务结果未通过，已停止自动提交".into());
            }
            Ok(())
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationState {
    AwaitingVerification,
    Verified,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityOperation {
    pub operation_id: String,
    pub account_id: String,
    pub task_code: String,
    pub state: OperationState,
    pub submitted_at_ms: i64,
    pub verified_at_ms: Option<i64>,
    /// Travel writes retain the successful business acknowledgement before readback.
    #[serde(default)]
    pub submission_acknowledged: Option<bool>,
    pub error: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Journal {
    version: u32,
    pub(crate) operations: BTreeMap<String, ActivityOperation>,
    #[serde(default)]
    claim_grants: BTreeMap<String, CredentialReference>,
}
impl Default for Journal {
    fn default() -> Self {
        Self {
            version: 1,
            operations: BTreeMap::new(),
            claim_grants: BTreeMap::new(),
        }
    }
}
#[derive(Default, Serialize)]
pub struct RunReport {
    pub verified: usize,
    pub already_claimed: usize,
    pub awaiting_verification: usize,
    pub skipped: usize,
    pub waiting: usize,
    pub halted: bool,
    pub errors: Vec<String>,
}
pub(crate) fn read_journal(root: &Path) -> Result<Journal, String> {
    let path = root.join("activity_operations.json");
    if !path.exists() {
        return Ok(Journal::default());
    }
    if std::fs::metadata(&path)
        .map_err(|_| "活动记录不可读取")?
        .len()
        > 8 * 1024 * 1024
    {
        return Err("活动记录超限，请保留原文件".into());
    }
    let journal: Journal =
        serde_json::from_slice(&std::fs::read(path).map_err(|_| "活动记录不可读取")?)
            .map_err(|_| "活动记录损坏，已停止提交")?;
    if journal.version != 1 {
        return Err("活动记录版本不支持，已停止提交".into());
    }
    Ok(journal)
}
pub(crate) fn save_journal(root: &Path, journal: &Journal) -> Result<(), String> {
    let text = serde_json::to_string(journal).map_err(|_| "活动记录不可序列化")?;
    if text.len() > 8 * 1024 * 1024 {
        return Err("活动记录超限，已停止提交".into());
    }
    durable_write(&root.join("activity_operations.json"), &text)
        .map_err(|_| "活动记录无法持久保存，已停止提交".into())
}
pub(crate) fn operation_key(a: &Value, t: &GrowthTask) -> String {
    format!(
        "{:x}",
        Sha256::digest(
            json!([
                growth::identity(a),
                t.code,
                t.task_type,
                t.valid_start_ms,
                t.valid_end_ms,
                "claim"
            ])
            .to_string()
            .as_bytes()
        )
    )
}
fn eligible(t: &GrowthTask, now: i64) -> bool {
    CLAIM_CODES.contains(&t.code.as_str())
        && t.task_type == "single"
        && t.validity_known
        && t.accept_status == "completed"
        && t.locked == Some(false)
        && t.progress_complete
        && t.valid_start_ms.is_none_or(|v| v <= now)
        && t.valid_end_ms.is_none_or(|v| v > now)
}

/// Each POST is preceded by a durable uncertain marker. Its mere failure never authorizes replay.
async fn run_for_account(
    root: &Path,
    a: &Value,
    api: &impl ClaimApi,
    journal: &mut Journal,
    report: &mut RunReport,
    allow_submit: bool,
) -> Result<(), String> {
    let id = account::get_str(a, "id").ok_or("账号标识缺失")?;
    let tasks = api.list().await?;
    for task in tasks {
        let key = operation_key(a, &task);
        if task.accept_status == "claimed" {
            if let Some(op) = journal.operations.get_mut(&key) {
                op.state = OperationState::Verified;
                op.verified_at_ms = Some(config::now_ms());
                op.error = None;
                save_journal(root, journal)?;
            }
            report.already_claimed += 1;
            continue;
        }
        if let Some(op) = journal.operations.get(&key) {
            if op.state == OperationState::AwaitingVerification {
                report.awaiting_verification += 1;
            } else {
                report.already_claimed += 1;
            }
            continue;
        }
        if !allow_submit || !eligible(&task, config::now_ms()) {
            report.skipped += 1;
            continue;
        }
        if journal.operations.len() >= 4096 {
            return Err("活动操作数量超限，需检查记录后继续".into());
        }
        journal.operations.insert(
            key.clone(),
            ActivityOperation {
                operation_id: uuid::Uuid::new_v4().to_string(),
                account_id: id.clone(),
                task_code: task.code.clone(),
                state: OperationState::AwaitingVerification,
                submitted_at_ms: config::now_ms(),
                verified_at_ms: None,
                submission_acknowledged: None,
                error: None,
            },
        );
        save_journal(root, journal)?;
        let submitted = api.submit(&task.code).await;
        let confirmed = api.list().await;
        let verified = confirmed.as_ref().is_ok_and(|rows| {
            rows.iter().any(|r| {
                r.code == task.code && operation_key(a, r) == key && r.accept_status == "claimed"
            })
        });
        let op = journal.operations.get_mut(&key).unwrap();
        if verified {
            op.state = OperationState::Verified;
            op.verified_at_ms = Some(config::now_ms());
            op.error = None;
            report.verified += 1;
        } else {
            op.error = Some(
                submitted
                    .err()
                    .or_else(|| confirmed.err())
                    .unwrap_or_else(|| "服务端尚未确认已领取，禁止重复提交".into()),
            );
            report.awaiting_verification += 1;
        }
        save_journal(root, journal)?;
        // No next POST after an unconfirmed result: preserve the account's remaining tasks.
        if !verified {
            report.halted = true;
            break;
        }
    }
    Ok(())
}

pub fn operations() -> Result<Vec<ActivityOperation>, String> {
    Ok(read_journal(&config::store_dir())?
        .operations
        .into_values()
        .collect())
}
pub fn run_claims(
    account_id: Option<&str>,
) -> Pin<Box<dyn Future<Output = Result<RunReport, String>> + Send + '_>> {
    Box::pin(run_claims_inner(account_id))
}
async fn run_claims_inner(account_id: Option<&str>) -> Result<RunReport, String> {
    let root = config::store_dir();
    std::fs::create_dir_all(root.join("locks")).map_err(|_| "活动锁目录不可创建")?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("locks/growth.lock"))
        .map_err(|_| "活动锁不可读取")?;
    lock.try_lock_exclusive()
        .map_err(|_| "成长任务正在处理，请稍后查看")?;
    run_claims_locked(&root, account_id, None).await
}
pub(crate) async fn run_claims_locked(
    root: &Path,
    account_id: Option<&str>,
    automatic_ids: Option<&[String]>,
) -> Result<RunReport, String> {
    let accounts = account::try_load_accounts().map_err(|_| "账号库不可读取")?;
    if account_id.is_some_and(|id| !accounts.iter().any(|a| a["id"] == id)) {
        return Err("账号不存在".into());
    }
    let mut journal = read_journal(root)?;
    let mut report = RunReport::default();
    if automatic_ids.is_some()
        && journal
            .operations
            .values()
            .any(|op| op.state == OperationState::AwaitingVerification)
    {
        // Read every selected account before permitting any further POST; rotation is
        // not a retry strategy for a rejected or unknown write.
        for a in accounts
            .iter()
            .filter(|a| automatic_ids.is_none_or(|ids| ids.iter().any(|id| a["id"] == *id)))
        {
            if journal.operations.values().any(|op| {
                op.account_id == a["id"].as_str().unwrap_or("")
                    && op.task_code.starts_with("travel_")
                    && op.state == OperationState::AwaitingVerification
            }) {
                if let Err(e) = super::growth_travel::run_locked(root, a, false, None, true).await {
                    report.errors.push(e);
                }
                journal = read_journal(root)?;
            }
            if let Err(e) = run_for_account(
                root,
                a,
                &Official(a, true),
                &mut journal,
                &mut report,
                false,
            )
            .await
            {
                report.errors.push(e);
            }
        }
        if journal
            .operations
            .values()
            .any(|op| op.state == OperationState::AwaitingVerification)
        {
            report.halted = true;
            report
                .errors
                .push("仍有待核实活动操作，自动提交已暂停".into());
            return Ok(report);
        }
    }
    for a in accounts
        .iter()
        .filter(|a| account_id.is_none_or(|id| a["id"] == id))
        .filter(|a| automatic_ids.is_none_or(|ids| ids.iter().any(|id| a["id"] == *id)))
    {
        let prepared = growth::prepare_account(a).await;
        let a = &prepared;
        let before = report.verified;
        if automatic_ids.is_some()
            && !super::growth_schedule::allows(a["id"].as_str().unwrap_or(""))?
        {
            break;
        }
        if let Err(error) = run_for_account(
            root,
            a,
            &Official(a, automatic_ids.is_some()),
            &mut journal,
            &mut report,
            true,
        )
        .await
        {
            report.errors.push(error);
            continue;
        }
        if report.verified > before {
            let id = account::get_str(a, "id").ok_or("账号标识缺失")?;
            let grant=CredentialStore::new(root).save(&id,&growth::grant_scope(a),CapabilityType::GrowthClaim,&json!({"verified_for":"https://www.workbuddy.cn/activity/growth/tasks/*/claim","verified_at_ms":config::now_ms(),"authorized":true,"credential_digest":format!("{:x}",Sha256::digest(a["access_token"].as_str().unwrap_or("").as_bytes()))}),a["expiresAt"].as_i64()).map_err(|_|"领奖用途验证记录保存失败")?;
            journal.claim_grants.insert(id, grant);
            save_journal(root, &journal)?;
        }
        if report.halted {
            break;
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    struct Fake {
        tasks: Mutex<Vec<GrowthTask>>,
        posts: std::sync::atomic::AtomicUsize,
        ack: bool,
        confirm: bool,
    }
    impl ClaimApi for Fake {
        fn list(&self) -> ListFuture<'_> {
            Box::pin(async { Ok(self.tasks.lock().unwrap().clone()) })
        }
        fn submit<'a>(&'a self, code: &'a str) -> SubmitFuture<'a> {
            Box::pin(async move {
                self.posts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if self.confirm {
                    self.tasks
                        .lock()
                        .unwrap()
                        .iter_mut()
                        .find(|t| t.code == code)
                        .unwrap()
                        .accept_status = "claimed".into();
                }
                if self.ack {
                    Ok(())
                } else {
                    Err("simulated timeout".into())
                }
            })
        }
    }
    fn fixture() -> (std::path::PathBuf, Value, GrowthTask) {
        let root = std::env::temp_dir().join(format!("growth-op-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let task=growth::parse_tasks(&json!({"tasks":[{"task_code":"chat_5","task_type":"single","accept_status":"completed","locked":false,"progress":{"current":5,"target":5},"valid_start":null,"valid_end":null}]})).unwrap().remove(0);
        (
            root,
            json!({"id":"a","uid":"fictional","variant":"cn"}),
            task,
        )
    }
    #[tokio::test]
    async fn timeout_after_success_is_verified_and_never_replayed() {
        let (root, a, t) = fixture();
        let api = Fake {
            tasks: Mutex::new(vec![t]),
            posts: 0.into(),
            ack: false,
            confirm: true,
        };
        let mut journal = Journal::default();
        let mut r = RunReport::default();
        run_for_account(&root, &a, &api, &mut journal, &mut r, true)
            .await
            .unwrap();
        assert_eq!(r.verified, 1);
        let mut restored = read_journal(&root).unwrap();
        run_for_account(&root, &a, &api, &mut restored, &mut r, true)
            .await
            .unwrap();
        assert_eq!(api.posts.load(std::sync::atomic::Ordering::SeqCst), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[tokio::test]
    async fn uncertain_submission_survives_restart_without_replay() {
        let (root, a, t) = fixture();
        let api = Fake {
            tasks: Mutex::new(vec![t]),
            posts: 0.into(),
            ack: true,
            confirm: false,
        };
        let mut journal = Journal::default();
        let mut r = RunReport::default();
        run_for_account(&root, &a, &api, &mut journal, &mut r, true)
            .await
            .unwrap();
        assert_eq!(r.awaiting_verification, 1);
        assert!(r.halted);
        let mut restored = read_journal(&root).unwrap();
        run_for_account(&root, &a, &api, &mut restored, &mut r, true)
            .await
            .unwrap();
        assert_eq!(api.posts.load(std::sync::atomic::Ordering::SeqCst), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unsupported_or_incomplete_tasks_cannot_be_claimed() {
        let (root, _, mut t) = fixture();
        assert!(eligible(&t, config::now_ms()));
        t.accept_status = "accepted".into();
        assert!(!eligible(&t, config::now_ms()));
        t.accept_status = "completed".into();
        t.validity_known = false;
        assert!(!eligible(&t, config::now_ms()));
        t.validity_known = true;
        t.code = "unknown".into();
        assert!(!eligible(&t, config::now_ms()));
        std::fs::remove_dir_all(root).unwrap();
    }
}
