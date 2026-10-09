//! Shared host facade. No credential reads or network access while authorization is unverified.
use crate::modules::{
    checkin_service::{CheckinService, DisabledProductionAdapter},
    config,
    daily_scheduler::{self, DailyScheduler, DailyTask, SchedulerConfig, SystemClock},
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::atomic::{AtomicI64, Ordering},
    time::Duration,
};

pub const BLOCKED_REASON: &str = "签到授权与服务接口尚未验证，计划已暂停；不会发送真实请求";
static LAST_HEARTBEAT: AtomicI64 = AtomicI64::new(0);

#[derive(Debug, Serialize)]
pub struct SchedulerSnapshot {
    pub config: SchedulerConfig,
    pub revision: String,
    pub business_date: String,
    pub observed_at_ms: i64,
    pub planned_at_ms: i64,
    pub next_planned_at_ms: Option<i64>,
    pub production_enabled: bool,
    pub blocked_reason: &'static str,
    pub background_active: bool,
    pub last_heartbeat_ms: Option<i64>,
    pub tasks: Vec<DailyTask>,
    pub history_limit: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveSchedulerRequest {
    pub config: SchedulerConfig,
    pub expected_revision: String,
}

fn revision(config: &SchedulerConfig) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(config).expect("typed config"))
    )
}

pub fn snapshot_at(root: &Path, now: i64) -> Result<SchedulerSnapshot, String> {
    let config = SchedulerConfig::load(root)
        .map_err(|_| "调度配置损坏或无法读取，请保留原文件".to_string())?;
    let (hour, minute) = config.validate().map_err(|_| "调度配置无效".to_string())?;
    let scheduler = DailyScheduler::new(
        root,
        config.clone(),
        SystemClock,
        CheckinService::new(DisabledProductionAdapter, Duration::from_secs(1)),
    )
    .map_err(|_| "调度配置无效".to_string())?;
    let journal = scheduler
        .snapshot()
        .map_err(|_| "调度记录损坏或无法读取，请保留原文件".to_string())?;
    let (date, planned) = daily_scheduler::schedule(now, hour, minute)
        .map_err(|_| "系统时间无法用于调度".to_string())?;
    let next = if now < planned {
        planned
    } else if config.catch_up {
        now
    } else {
        planned.checked_add(86_400_000).ok_or("系统时间超出范围")?
    };
    let mut tasks = journal.tasks.into_values().collect::<Vec<_>>();
    tasks.sort_by(|a, b| {
        b.last_started_at_ms
            .unwrap_or(b.scheduled_at_ms)
            .cmp(&a.last_started_at_ms.unwrap_or(a.scheduled_at_ms))
            .then_with(|| a.account_id.cmp(&b.account_id))
    });
    tasks.truncate(500);
    let heartbeat = LAST_HEARTBEAT.load(Ordering::Relaxed);
    Ok(SchedulerSnapshot {
        revision: revision(&config),
        next_planned_at_ms: config.enabled.then_some(next),
        config,
        business_date: date,
        observed_at_ms: now,
        planned_at_ms: planned,
        production_enabled: false,
        blocked_reason: BLOCKED_REASON,
        background_active: heartbeat > 0 && (0..=45_000).contains(&now.saturating_sub(heartbeat)),
        last_heartbeat_ms: (heartbeat > 0).then_some(heartbeat),
        tasks,
        history_limit: 500,
    })
}

pub fn snapshot() -> Result<SchedulerSnapshot, String> {
    snapshot_at(&config::store_dir(), config::now_ms())
}

pub fn save_at(
    root: &Path,
    request: SaveSchedulerRequest,
    now: i64,
) -> Result<SchedulerSnapshot, String> {
    request
        .config
        .validate()
        .map_err(|_| "请输入有效时间和重试参数".to_string())?;
    if request.expected_revision.len() != 64
        || !request
            .expected_revision
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || request.config.account_enabled.len() > 10_000
        || request
            .config
            .account_enabled
            .keys()
            .any(|id| id.is_empty() || id.len() > 256)
    {
        return Err("调度配置参数无效".into());
    }
    std::fs::create_dir_all(root).map_err(|_| "调度配置目录不可写".to_string())?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("daily_scheduler.lock"))
        .map_err(|_| "调度锁无法建立".to_string())?;
    lock.try_lock_exclusive()
        .map_err(|_| "其他窗口或调度实例正在操作，请稍后重试".to_string())?;
    // Validate the existing config and journal before replacing anything.
    let mut before = snapshot_at(root, now)?;
    if before.revision != request.expected_revision {
        return Err("设置已被其他窗口修改，请重新加载后再保存".into());
    }
    request
        .config
        .save(root)
        .map_err(|_| "调度配置保存失败，原配置已保留".to_string())?;
    before.revision = revision(&request.config);
    before.config = request.config;
    let (hour, minute) = before
        .config
        .validate()
        .map_err(|_| "调度配置无效".to_string())?;
    before.planned_at_ms = daily_scheduler::schedule(now, hour, minute)
        .map_err(|_| "系统时间无效".to_string())?
        .1;
    before.next_planned_at_ms = before
        .config
        .enabled
        .then_some(if now < before.planned_at_ms {
            before.planned_at_ms
        } else if before.config.catch_up {
            now
        } else {
            before
                .planned_at_ms
                .checked_add(86_400_000)
                .ok_or("系统时间超出范围")?
        });
    Ok(before)
}

pub fn save(request: SaveSchedulerRequest) -> Result<SchedulerSnapshot, String> {
    save_at(&config::store_dir(), request, config::now_ms())
}

pub fn run_now() -> Result<(), String> {
    Err(BLOCKED_REASON.into())
}

/// Replaces the legacy Local/random-window loop in both hosts. The compatibility gate
/// precedes any account loading, task reservation or HTTP. Polling records no success.
pub async fn background_tick_at(root: &Path, now: i64) -> Result<(), String> {
    SchedulerConfig::load(root).map_err(|_| "后台调度配置不可读取".to_string())?;
    LAST_HEARTBEAT.store(now, Ordering::Relaxed);
    Ok(())
}

pub async fn background_loop() {
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        if background_tick_at(&config::store_dir(), config::now_ms())
            .await
            .is_err()
        {
            LAST_HEARTBEAT.store(0, Ordering::Relaxed);
            eprintln!("[自动签到] 调度配置不可读取，已暂停，请保留原文件");
        }
    }
}
