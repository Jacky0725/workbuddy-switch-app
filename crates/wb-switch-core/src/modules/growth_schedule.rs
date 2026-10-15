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
    if c.enabled
        && ((!c.auto_claim && !c.travel_claim && !c.travel_depart) || c.account_ids.is_empty())
    {
        return Err("请选择账号及自动动作".into());
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
        revision: revision(&c),
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
    if revision(&load(&root)?) != request.expected_revision {
        return Err("配置已被其他窗口更新，请刷新后保存".into());
    }
    durable_write(
        &root.join("growth_config.json"),
        &serde_json::to_string(&request.config).unwrap(),
    )
    .map_err(|_| "成长计划保存失败")?;
    snapshot_at(&root)
}
pub(crate) fn allows(id: &str) -> Result<bool, String> {
    let c = load(&config::store_dir())?;
    Ok(c.enabled && c.auto_claim && c.account_ids.iter().any(|v| v == id))
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
fn due(c: &GrowthConfig, s: &ScheduleState, now: i64) -> bool {
    if !c.enabled || (!c.auto_claim && !c.travel_claim && !c.travel_depart) {
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
    s.date != date || s.config_revision != revision(c) || s.next_retry_ms.is_some_and(|v| v <= now)
}
async fn run_due(root: &Path, now: i64) -> Result<(), String> {
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
    if !due(&c, &s, now) {
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
    s.config_revision = revision(&c);
    let persist = |s: &ScheduleState| {
        durable_write(
            &root.join("growth_schedule_state.json"),
            &serde_json::to_string(s).unwrap(),
        )
        .map_err(|_| "成长运行状态保存失败".to_string())
    };
    persist(&s)?;
    let result = async {
        let mut total = if c.auto_claim {
            growth_automation::run_claims_locked(root, None, Some(&c.account_ids)).await?
        } else {
            growth_automation::RunReport::default()
        };
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
    .await;
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
            Ok(due(&c, &s, config::now_ms()))
        })();
        if matches!(result, Ok(true)) {
            if run_due(&root, config::now_ms()).await.is_err() {
                eprintln!("[成长任务] 配置或运行记录不可读取，已暂停本批次");
            } else if let Ok(c) = load(&root) {
                if c.enabled && c.auto_claim {
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
    #[test]
    fn defaults_do_not_submit_and_catch_up_only_today() {
        let mut c = GrowthConfig::default();
        let mut s = ScheduleState::default();
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-15T10:00:00+08:00")
            .unwrap()
            .timestamp_millis();
        assert!(!due(&c, &s, now));
        c.enabled = true;
        c.auto_claim = true;
        c.account_ids = vec!["a".into()];
        assert!(due(&c, &s, now));
        s.date = "2026-10-15".into();
        s.config_revision = revision(&c);
        assert!(!due(&c, &s, now));
        s.next_retry_ms = Some(now + 900_000);
        assert!(!due(&c, &s, now));
        assert!(due(&c, &s, now + 900_000));
        assert!(!due(&c, &ScheduleState::default(), now - 3_600_000));
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
