//! One bounded daily batch, dispatched by the existing manager coordinator.
use super::{config, credential_store::durable_write, growth_automation};
use chrono::{FixedOffset, TimeZone, Timelike};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GrowthConfig {
    pub version: u32,
    pub enabled: bool,
    pub auto_claim: bool,
    pub account_ids: Vec<String>,
    pub daily_time: String,
    #[serde(default)]
    pub travel_claim: bool,
    #[serde(default)]
    pub travel_depart: bool,
    #[serde(default)]
    pub travel_location_id: Option<i64>,
}
impl Default for GrowthConfig {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: false,
            auto_claim: false,
            account_ids: vec![],
            daily_time: "09:30".into(),
            travel_claim: false,
            travel_depart: false,
            travel_location_id: None,
        }
    }
}
#[derive(Serialize)]
pub struct ScheduleSnapshot {
    pub config: GrowthConfig,
    pub chat_execution_enabled: bool,
    pub revision: String,
    pub running: bool,
    pub state: ScheduleState,
}
#[derive(Default, Serialize, Deserialize)]
pub struct ScheduleState {
    pub date: String,
    #[serde(default)]
    pub config_revision: String,
    pub last_started_ms: Option<i64>,
    pub finished_ms: Option<i64>,
    pub next_retry_ms: Option<i64>,
    pub error: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveRequest {
    pub config: GrowthConfig,
    pub expected_revision: String,
    pub chat_execution_enabled: Option<bool>,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatConfig {
    version: u32,
    enabled: bool,
}
fn chat_enabled(root: &Path) -> Result<bool, String> {
    let path = root.join("growth_chat_config.json");
    if !path.exists() {
        return Ok(false);
    }
    let c: ChatConfig = read(&path)?;
    if c.version != 1 {
        return Err("聊天执行配置版本不支持，已停止执行".into());
    }
    Ok(c.enabled)
}
fn schedule_revision(root: &Path, c: &GrowthConfig) -> Result<String, String> {
    Ok(format!("{}:{}", revision(c), chat_enabled(root)?))
}
static RUNNING: AtomicBool = AtomicBool::new(false);
fn read<T: serde::de::DeserializeOwned + Default>(path: &Path) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    if std::fs::metadata(path)
        .map_err(|_| "成长计划不可读取")?
        .len()
        > 128 * 1024
    {
        return Err("成长计划文件超限".into());
    }
    serde_json::from_slice(&std::fs::read(path).map_err(|_| "成长计划不可读取")?)
        .map_err(|_| "成长计划文件损坏，已停止执行".into())
}
fn validate(c: &GrowthConfig) -> Result<(), String> {
    if c.version != 1
        || minutes(&c.daily_time).is_none()
        || c.account_ids.len() > 256
        || c.account_ids
            .iter()
            .any(|id| id.is_empty() || id.len() > 128)
    {
        return Err("成长计划配置无效".into());
    }
    if c.travel_depart && c.travel_location_id.is_none_or(|id| id <= 0) {
        return Err("请选择旅行目的地".into());
    }
    if c.enabled && c.account_ids.is_empty() {
        return Err("请选择账号".into());
    }
    let unique: std::collections::BTreeSet<_> = c.account_ids.iter().collect();
    if unique.len() != c.account_ids.len() {
        return Err("账号选择重复".into());
    }
    Ok(())
}
fn minutes(s: &str) -> Option<u32> {
    if s.len() != 5 || s.as_bytes()[2] != b':' {
        return None;
    }
    let h: u32 = s.get(..2)?.parse().ok()?;
    let m: u32 = s.get(3..)?.parse().ok()?;
    (h < 24 && m < 60).then_some(h * 60 + m)
}
fn revision(c: &GrowthConfig) -> String {
    format!("{:x}", Sha256::digest(serde_json::to_vec(c).unwrap()))
}
fn load(root: &Path) -> Result<GrowthConfig, String> {
    let c = read(&root.join("growth_config.json"))?;
    validate(&c)?;
    Ok(c)
}
pub fn snapshot() -> Result<ScheduleSnapshot, String> {
    snapshot_at(&config::store_dir())
}
pub(crate) fn snapshot_at(root: &Path) -> Result<ScheduleSnapshot, String> {
    let c = load(root)?;
    Ok(ScheduleSnapshot {
        revision: schedule_revision(root, &c)?,
        chat_execution_enabled: chat_enabled(root)?,
        config: c,
        running: RUNNING.load(Ordering::Relaxed),
        state: read(&root.join("growth_schedule_state.json"))?,
    })
}
pub fn save(request: SaveRequest) -> Result<ScheduleSnapshot, String> {
    let root = config::store_dir();
    std::fs::create_dir_all(root.join("locks")).map_err(|_| "成长配置目录不可创建")?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("locks/growth_config.lock"))
        .map_err(|_| "成长配置锁不可读取")?;
    lock.try_lock_exclusive().map_err(|_| "成长配置正在保存")?;
    validate(&request.config)?;
    if schedule_revision(&root, &load(&root)?)? != request.expected_revision {
        return Err("配置已被其他窗口更新，请刷新后保存".into());
    }
    // Disable before changing selected accounts; enable only after the plan is
    // saved. Readers must never see a new plan with an old enabled chat switch
    // when this save was intended to cancel chat execution.
    if request.chat_execution_enabled == Some(false) {
        save_chat(&root, false)?;
    }
    durable_write(
        &root.join("growth_config.json"),
        &serde_json::to_string(&request.config).unwrap(),
    )
    .map_err(|_| "成长计划保存失败")?;
    if request.chat_execution_enabled == Some(true) {
        save_chat(&root, true)?;
    }
    snapshot_at(&root)
}
fn save_chat(root: &Path, enabled: bool) -> Result<(), String> {
    durable_write(
        &root.join("growth_chat_config.json"),
        &serde_json::to_string(&ChatConfig {
            version: 1,
            enabled,
        })
        .unwrap(),
    )
    .map_err(|_| "聊天执行开关保存失败，请刷新核对".into())
}
pub(crate) fn allows(id: &str) -> Result<bool, String> {
    let c = load(&config::store_dir())?;
    Ok(c.enabled && c.auto_claim && c.account_ids.iter().any(|v| v == id))
}
pub(crate) fn allows_chat(root: &Path, id: &str) -> Result<bool, String> {
    let c = load(root)?;
    Ok(c.enabled && chat_enabled(root)? && c.account_ids.iter().any(|v| v == id))
}
pub(crate) fn allows_travel(id: &str, action: &str, location: Option<i64>) -> Result<bool, String> {
    let c = load(&config::store_dir())?;
    Ok(c.enabled
        && c.account_ids.iter().any(|v| v == id)
        && match action {
            "claim" => c.travel_claim,
            "depart" => c.travel_depart && c.travel_location_id == location,
            _ => false,
        })
}
fn due(c: &GrowthConfig, s: &ScheduleState, now: i64, chat: bool) -> bool {
    if !c.enabled || (!c.auto_claim && !c.travel_claim && !c.travel_depart && !chat) {
        return false;
    }
    let Some(t) = FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .timestamp_millis_opt(now)
        .single()
    else {
        return false;
    };
    if t.hour() * 60 + t.minute() < minutes(&c.daily_time).unwrap_or(1440) {
        return false;
    }
    let date = t.format("%Y-%m-%d").to_string();
    s.date != date
        || s.config_revision != format!("{}:{chat}", revision(c))
        || s.next_retry_ms.is_some_and(|v| v <= now)
}
async fn execute_batch(
    root: &Path,
    c: GrowthConfig,
    chat: bool,
) -> Result<growth_automation::RunReport, String> {
    let mut chat_pending = 0;
    let mut chat_errors = Vec::new();
    let mut chat_halted = false;
    // Reconcile even when disabled, but never send a prompt with zero budget.
    for id in &c.account_ids {
        let pending = growth_automation::read_journal(root)?
            .operations
            .values()
            .any(|op| {
                op.account_id == *id
                    && op.task_code.starts_with("chat_5/")
                    && op.state == growth_automation::OperationState::AwaitingVerification
            });
        if !chat && !pending {
            continue;
        }
        let budget = if allows_chat(root, id)? { 5 } else { 0 };
        match super::growth_execution::run_locked(root, id, budget, true).await {
            Ok(r) => {
                chat_pending += usize::from(r.awaiting_verification);
                if r.halted {
                    chat_halted = true;
                    chat_errors.push("聊天执行已暂停，请查看操作记录".into());
                    break;
                }
            }
            Err(e) => chat_errors.push(e),
        }
    }
    let mut total = if c.auto_claim && !chat_halted {
        growth_automation::run_claims_locked(root, None, Some(&c.account_ids)).await?
    } else {
        growth_automation::RunReport::default()
    };
    total.halted |= chat_halted;
    total.awaiting_verification += chat_pending;
    total.errors.extend(chat_errors);
    if !total.halted && (c.travel_claim || c.travel_depart) {
        for a in super::account::try_load_accounts()
            .map_err(|_| "账号库不可读取")?
            .iter()
            .filter(|a| c.account_ids.iter().any(|id| a["id"] == *id))
        {
            match super::growth_travel::run_locked(
                root,
                a,
                c.travel_claim,
                if c.travel_depart {
                    c.travel_location_id
                } else {
                    None
                },
                true,
            )
            .await
            {
                Ok(r) => {
                    total.verified += r.verified;
                    total.waiting += r.waiting;
                    total.awaiting_verification += r.awaiting_verification;
                    total.errors.extend(r.errors);
                    if r.halted || r.awaiting_verification > 0 {
                        total.halted = true;
                        break;
                    }
                }
                Err(e) => total.errors.push(e),
            }
        }
    }
    Ok::<_, String>(total)
}
async fn run_due(root: &Path, now: i64) -> Result<(), String> {
    run_due_with(root, now, |c, chat| execute_batch(root, c, chat)).await
}
async fn run_due_with<F, Fut>(root: &Path, now: i64, execute: F) -> Result<(), String>
where
    F: FnOnce(GrowthConfig, bool) -> Fut,
    Fut: std::future::Future<Output = Result<growth_automation::RunReport, String>>,
{
    std::fs::create_dir_all(root.join("locks")).map_err(|_| "成长锁目录不可创建")?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("locks/growth.lock"))
        .map_err(|_| "成长锁不可读取")?;
    if lock.try_lock_exclusive().is_err() {
        return Ok(());
    }
    let c = load(root)?;
    let mut s: ScheduleState = read(&root.join("growth_schedule_state.json"))?;
    let chat = chat_enabled(root)?;
    if !due(&c, &s, now, chat) {
        return Ok(());
    }
    s.date = FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .timestamp_millis_opt(now)
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();
    s.last_started_ms = Some(now);
    s.finished_ms = None;
    s.next_retry_ms = Some(now + 15 * 60 * 1000);
    s.error = None;
    s.config_revision = schedule_revision(root, &c)?;
    let persist = |s: &ScheduleState| {
        durable_write(
            &root.join("growth_schedule_state.json"),
            &serde_json::to_string(s).unwrap(),
        )
        .map_err(|_| "成长运行状态保存失败".to_string())
    };
    persist(&s)?;
    let result = execute(c, chat).await;
    match result {
        Ok(r) => {
            s.finished_ms = Some(config::now_ms());
            if r.errors.is_empty() && r.awaiting_verification == 0 && r.waiting == 0 {
                s.next_retry_ms = None;
            } else if !r.errors.is_empty() || r.awaiting_verification > 0 {
                s.error = Some(format!(
                    "待核实 {} 项；查询异常 {} 项",
                    r.awaiting_verification,
                    r.errors.len()
                ));
            }
        }
        Err(e) => s.error = Some(e),
    }
    persist(&s)
}
/// Returns immediately; a finite worker cannot hold up the existing check-in heartbeat.
pub fn dispatch() {
    if RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    tokio::spawn(async {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                RUNNING.store(false, Ordering::Release);
            }
        }
        let _reset = Reset;
        let root = config::store_dir();
        let result: Result<bool, String> = (|| {
            let c = load(&root)?;
            let s = read(&root.join("growth_schedule_state.json"))?;
            Ok(due(&c, &s, config::now_ms(), chat_enabled(&root)?))
        })();
        if matches!(result, Ok(true)) {
            if run_due(&root, config::now_ms()).await.is_err() {
                eprintln!("[成长任务] 配置或运行记录不可读取，已暂停本批次");
            } else if let Ok(c) = load(&root) {
                if c.enabled {
                    // The batch has released the shared lock. Refresh the display cache
                    // from official state rather than leaving old completed rows visible.
                    for id in c.account_ids {
                        let _ = super::growth::refresh(Some(&id)).await;
                    }
                }
            }
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("growth-plan-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        root
    }
    #[tokio::test]
    async fn scheduled_worker_runs_once_and_shared_lock_prevents_second_batch() {
        let root = fixture();
        let c = GrowthConfig {
            enabled: true,
            account_ids: vec!["fictional".into()],
            ..Default::default()
        };
        std::fs::write(
            root.join("growth_config.json"),
            serde_json::to_vec(&c).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("growth_chat_config.json"),
            r#"{"version":1,"enabled":true}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("daily_scheduler_state.json"),
            "unchanged-signin-fixture",
        )
        .unwrap();
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-19T10:00:00+08:00")
            .unwrap()
            .timestamp_millis();
        let calls = std::sync::atomic::AtomicUsize::new(0);
        for time in [now - 3_600_000, now, now + 60_000] {
            run_due_with(&root, time, |_, chat| {
                assert!(chat);
                calls.fetch_add(1, Ordering::SeqCst);
                std::future::ready(Ok(growth_automation::RunReport::default()))
            })
            .await
            .unwrap();
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            std::fs::read_to_string(root.join("daily_scheduler_state.json")).unwrap(),
            "unchanged-signin-fixture"
        );
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join("locks/growth.lock"))
            .unwrap();
        lock.try_lock_exclusive().unwrap();
        run_due_with(&root, now + 86_400_000, |_, _| {
            panic!("second worker must not run");
            #[allow(unreachable_code)]
            std::future::ready(Ok(growth_automation::RunReport::default()))
        })
        .await
        .unwrap();
        drop(lock);
        std::fs::write(
            root.join("growth_chat_config.json"),
            r#"{"version":1,"enabled":false}"#,
        )
        .unwrap();
        run_due_with(&root, now + 86_400_000, |_, _| {
            panic!("disabled worker must not run");
            #[allow(unreachable_code)]
            std::future::ready(Ok(growth_automation::RunReport::default()))
        })
        .await
        .unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn upgrade_defaults_off_and_cancellation_rechecks_accounts_and_switch() {
        let root = fixture();
        let mut c = GrowthConfig {
            enabled: true,
            account_ids: vec!["a".into()],
            ..Default::default()
        };
        let original = serde_json::to_vec(&c).unwrap();
        std::fs::write(root.join("growth_config.json"), &original).unwrap();
        assert!(!snapshot_at(&root).unwrap().chat_execution_enabled);
        assert!(!allows_chat(&root, "a").unwrap());
        std::fs::write(
            root.join("growth_chat_config.json"),
            r#"{"version":1,"enabled":true}"#,
        )
        .unwrap();
        assert!(allows_chat(&root, "a").unwrap());
        assert!(!allows_chat(&root, "removed").unwrap());
        assert_eq!(
            std::fs::read(root.join("growth_config.json")).unwrap(),
            original
        );
        c.enabled = false;
        std::fs::write(
            root.join("growth_config.json"),
            serde_json::to_vec(&c).unwrap(),
        )
        .unwrap();
        assert!(!allows_chat(&root, "a").unwrap());
        std::fs::write(
            root.join("growth_chat_config.json"),
            r#"{"version":1,"enabled":false}"#,
        )
        .unwrap();
        assert!(!allows_chat(&root, "a").unwrap());
        std::fs::write(root.join("growth_chat_config.json"), "broken").unwrap();
        c.enabled = true;
        std::fs::write(
            root.join("growth_config.json"),
            serde_json::to_vec(&c).unwrap(),
        )
        .unwrap();
        assert!(allows_chat(&root, "a").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn chat_only_schedule_is_due_once_then_survives_restart_and_toggle() {
        let root = fixture();
        let c = GrowthConfig {
            enabled: true,
            account_ids: vec!["a".into()],
            ..Default::default()
        };
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-19T10:00:00+08:00")
            .unwrap()
            .timestamp_millis();
        assert!(!due(&c, &ScheduleState::default(), now, false));
        assert!(due(&c, &ScheduleState::default(), now, true));
        let s = ScheduleState {
            date: "2026-10-19".into(),
            config_revision: format!("{}:true", revision(&c)),
            ..Default::default()
        };
        std::fs::write(
            root.join("growth_schedule_state.json"),
            serde_json::to_vec(&s).unwrap(),
        )
        .unwrap();
        let restored: ScheduleState = read(&root.join("growth_schedule_state.json")).unwrap();
        assert!(!due(&c, &restored, now + 60_000, true));
        assert!(!due(&c, &restored, now - 60_000, true));
        assert!(due(&c, &restored, now + 86_400_000, true));
        assert!(!due(&c, &restored, now, false));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn defaults_do_not_submit_and_catch_up_only_today() {
        let mut c = GrowthConfig::default();
        let mut s = ScheduleState::default();
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-15T10:00:00+08:00")
            .unwrap()
            .timestamp_millis();
        assert!(!due(&c, &s, now, false));
        c.enabled = true;
        c.auto_claim = true;
        c.account_ids = vec!["a".into()];
        assert!(due(&c, &s, now, false));
        s.date = "2026-10-15".into();
        s.config_revision = format!("{}:false", revision(&c));
        assert!(!due(&c, &s, now, false));
        s.next_retry_ms = Some(now + 900_000);
        assert!(!due(&c, &s, now, false));
        assert!(due(&c, &s, now + 900_000, false));
        assert!(!due(&c, &ScheduleState::default(), now - 3_600_000, false));
    }
    #[test]
    fn invalid_times_and_unknown_versions_fail_closed() {
        for t in ["24:00", "09:60", "9:30", "💫:3"] {
            assert!(minutes(t).is_none());
        }
        let c = GrowthConfig {
            version: 2,
            ..Default::default()
        };
        assert!(validate(&c).is_err());
    }
}
