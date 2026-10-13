//! Shared host facade. No credential reads or network access while authorization is unverified.
use crate::modules::{
    account,
    account_repository::AccountRepository,
    checkin_service::{
        CancellationToken, CheckinAccount, CheckinService, DisabledProductionAdapter,
        OfficialCheckinAdapter,
    },
    config,
    credential_store::CapabilityType,
    daily_scheduler::{self, DailyScheduler, DailyTask, SchedulerConfig, SystemClock, TaskState},
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

fn production_accounts() -> Result<(OfficialCheckinAdapter, Vec<CheckinAccount>), String> {
    let stored = account::try_load_accounts().map_err(|_| BLOCKED_REASON.to_string())?;
    let adapter = OfficialCheckinAdapter::from_accounts(&stored, Duration::from_secs(15))
        .map_err(str::to_string)?;
    let accounts = stored
        .iter()
        .filter(|item| item["region"] == "cn")
        .map(|item| CheckinAccount {
            id: item["id"].as_str().unwrap_or("").to_string(),
            provider: "workbuddy".into(),
            region: "cn".into(),
            enabled: item["is_enabled"] != false,
        })
        .collect::<Vec<_>>();
    if accounts.is_empty()
        || accounts
            .iter()
            .any(|item| item.id.is_empty() || item.enabled && !adapter.has_account(&item.id))
    {
        return Err(BLOCKED_REASON.into());
    }
    Ok((adapter, accounts))
}

/// Explicitly enroll one locally stored account for the verified fixed-host
/// check-in transport. A successful read-only status response is required
/// before adding the purpose-specific grant to the DPAPI-backed account store.
/// The scheduler remains disabled until its own acceptance is complete.
pub async fn verify_and_enroll_account(account_id: &str) -> Result<bool, String> {
    let accounts = account::try_load_accounts().map_err(|_| "账号库无法读取".to_string())?;
    let candidate = accounts
        .iter()
        .find(|item| item["id"] == account_id)
        .ok_or_else(|| "账号不存在".to_string())?;
    if candidate["region"] != "cn"
        || candidate["credential_locked"] == true
        || candidate["needs_relogin"] == true
    {
        return Err("该账号不能用于国内版签到验证".into());
    }
    let uid = candidate["uid"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "账号身份缺失".to_string())?
        .to_string();
    let token = candidate["access_token"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "账号签到凭证缺失".to_string())?;
    let token_digest = Sha256::digest(token.as_bytes());
    let mut probe = candidate.clone();
    probe["capability_credentials"]["Checkin"] = serde_json::json!({
        "authorized": true,
        "verified_for": "copilot.tencent.com/v2/billing/meter"
    });
    let adapter = OfficialCheckinAdapter::from_accounts(&[probe], Duration::from_secs(15))
        .map_err(str::to_string)?;
    let service = CheckinService::new(adapter, Duration::from_secs(1));
    let status = service
        .get_checkin_status(
            &CheckinAccount {
                id: account_id.to_string(),
                provider: "workbuddy".into(),
                region: "cn".into(),
                enabled: true,
            },
            &CancellationToken::default(),
        )
        .await
        .map_err(|result| format!("官方签到状态未通过验证：{result:?}"))?;
    let repository = AccountRepository::new(&config::accounts_file());
    repository
        .update(|items| {
            let current = items
                .iter_mut()
                .find(|item| item["id"] == account_id)
                .ok_or_else(|| std::io::Error::other("账号验证期间发生变化"))?;
            let current_token = current["access_token"].as_str().unwrap_or("");
            if current["uid"] != uid || Sha256::digest(current_token.as_bytes()) != token_digest {
                return Err(std::io::Error::other("账号凭证在验证期间发生变化，请重试"));
            }
            current["capability_credentials"]["Checkin"] = serde_json::json!({
                "authorized": true,
                "verified_for": "copilot.tencent.com/v2/billing/meter",
                "verified_at_ms": config::now_ms(),
                "capability_type": CapabilityType::Checkin.as_str(),
            });
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    Ok(status)
}

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
    let production = (root == config::store_dir()).then(production_accounts).and_then(Result::ok);
    let completed_today = production.as_ref().is_some_and(|(_, accounts)| {
        let eligible = accounts.iter().filter(|account| {
            account.enabled && config.account_enabled.get(&account.id) != Some(&false)
        }).collect::<Vec<_>>();
        !eligible.is_empty() && eligible.iter().all(|account| {
            journal.tasks.values().any(|task| {
                task.account_id == account.id
                    && task.business_date == date
                    && matches!(task.state, TaskState::Finished(
                        crate::modules::checkin_service::CheckinResultStatus::Success
                        | crate::modules::checkin_service::CheckinResultStatus::AlreadyCompleted
                    ))
            })
        })
    });
    let next = if now < planned {
        planned
    } else if config.catch_up && !completed_today {
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
    let production_enabled = production.is_some();
    Ok(SchedulerSnapshot {
        revision: revision(&config),
        next_planned_at_ms: config.enabled.then_some(next),
        config,
        business_date: date,
        observed_at_ms: now,
        planned_at_ms: planned,
        production_enabled,
        blocked_reason: if production_enabled {
            ""
        } else {
            BLOCKED_REASON
        },
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
    let before = snapshot_at(root, now)?;
    if before.revision != request.expected_revision {
        return Err("设置已被其他窗口修改，请重新加载后再保存".into());
    }
    request
        .config
        .save(root)
        .map_err(|_| "调度配置保存失败，原配置已保留".to_string())?;
    snapshot_at(root, now)
}

pub fn save(request: SaveSchedulerRequest) -> Result<SchedulerSnapshot, String> {
    save_at(&config::store_dir(), request, config::now_ms())
}

pub async fn run_now(account_id: Option<&str>) -> Result<(), String> {
    let root = config::store_dir();
    let config = SchedulerConfig::load(&root).map_err(|_| "调度配置不可读取".to_string())?;
    let (adapter, accounts) = production_accounts()?;
    if account_id.is_some_and(|target| !accounts.iter().any(|item| item.id == target)) {
        return Err("账号不存在或不支持签到".into());
    }
    let scheduler = DailyScheduler::new(
        &root,
        config,
        SystemClock,
        CheckinService::new(adapter, Duration::from_secs(1)),
    )
    .map_err(|_| "调度配置无效".to_string())?;
    scheduler
        .tick_manual(&accounts, &Default::default(), account_id)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

/// Replaces the legacy Local/random-window loop in both hosts. The compatibility gate
/// precedes any account loading, task reservation or HTTP. Polling records no success.
pub async fn background_tick_at(root: &Path, now: i64) -> Result<(), String> {
    let config = SchedulerConfig::load(root).map_err(|_| "后台调度配置不可读取".to_string())?;
    LAST_HEARTBEAT.store(now, Ordering::Relaxed);
    if root == config::store_dir() && config.enabled {
        let (adapter, accounts) = match production_accounts() {
            Ok(value) => value,
            Err(_) => return Ok(()),
        };
        let scheduler = DailyScheduler::new(
            root,
            config,
            SystemClock,
            CheckinService::new(adapter, Duration::from_secs(1)),
        )
        .map_err(|_| "调度配置无效".to_string())?;
        scheduler
            .tick(&accounts, &Default::default())
            .await
            .map_err(|error| error.to_string())?;
    }
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
