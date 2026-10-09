//! Durable Beijing-time coordinator. No OS task registration or GUI startup.
//! Reserve tasks before executing; interrupted/uncertain POSTs are verify-only.

use crate::modules::{
    checkin_service::{
        CancellationToken, CheckinAccount, CheckinAdapter, CheckinResult,
        CheckinResultStatus as Status, CheckinService,
    },
    config,
    credential_store::durable_write,
};
use chrono::{FixedOffset, TimeZone, Timelike};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    future::Future,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    pin::Pin,
};

pub trait Clock: Send + Sync {
    fn now_ms(&self) -> i64;
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        config::now_ms()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerConfig {
    pub version: u32,
    pub enabled: bool,
    pub timezone: String,
    pub daily_time: String,
    pub catch_up: bool,
    pub max_daily_attempts: u8,
    pub retry_delay_seconds: u32,
    pub account_enabled: BTreeMap<String, bool>,
}
impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: false,
            timezone: "Asia/Shanghai".into(),
            daily_time: "09:15".into(),
            catch_up: true,
            max_daily_attempts: 2,
            retry_delay_seconds: 300,
            account_enabled: BTreeMap::new(),
        }
    }
}
impl SchedulerConfig {
    pub fn validate(&self) -> io::Result<(u32, u32)> {
        let parts = self.daily_time.split(':').collect::<Vec<_>>();
        if self.version != 1
            || self.timezone != "Asia/Shanghai"
            || parts.len() != 2
            || self.daily_time.len() != 5
            || !parts
                .iter()
                .all(|v| v.len() == 2 && v.bytes().all(|c| c.is_ascii_digit()))
            || !(1..=3).contains(&self.max_daily_attempts)
            || !(30..=86_400).contains(&self.retry_delay_seconds)
        {
            return Err(invalid("调度配置无效"));
        }
        let hour = parts[0]
            .parse::<u32>()
            .map_err(|_| invalid("调度时间无效"))?;
        let minute = parts[1]
            .parse::<u32>()
            .map_err(|_| invalid("调度时间无效"))?;
        if hour > 23 || minute > 59 {
            return Err(invalid("调度时间无效"));
        }
        Ok((hour, minute))
    }
    pub fn load(root: &Path) -> io::Result<Self> {
        let path = root.join("daily_scheduler_config.json");
        let config = match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str::<Self>(&text)
                .map_err(|_| invalid("调度配置损坏或版本不兼容"))?,
            Err(e) if e.kind() == ErrorKind::NotFound => Self::default(),
            Err(e) => return Err(e),
        };
        config.validate()?;
        Ok(config)
    }
    pub fn save(&self, root: &Path) -> io::Result<()> {
        self.validate()?;
        std::fs::create_dir_all(root)?;
        let text = serde_json::to_string_pretty(self).map_err(|_| invalid("调度配置无法保存"))?;
        durable_write(&root.join("daily_scheduler_config.json"), &text)
    }
}

pub type ExecutionFuture<'a> = Pin<Box<dyn Future<Output = CheckinResult> + Send + 'a>>;
pub type VerificationFuture<'a> = Pin<Box<dyn Future<Output = Result<bool, Status>> + Send + 'a>>;
pub trait ScheduledExecutor: Send + Sync {
    fn execute<'a>(
        &'a self,
        account: &'a CheckinAccount,
        date: &'a str,
        cancel: &'a CancellationToken,
    ) -> ExecutionFuture<'a>;
    fn verify<'a>(
        &'a self,
        account: &'a CheckinAccount,
        cancel: &'a CancellationToken,
    ) -> VerificationFuture<'a>;
}
impl<A: CheckinAdapter> ScheduledExecutor for CheckinService<A> {
    fn execute<'a>(
        &'a self,
        account: &'a CheckinAccount,
        date: &'a str,
        cancel: &'a CancellationToken,
    ) -> ExecutionFuture<'a> {
        Box::pin(self.checkin(account, date, cancel))
    }
    fn verify<'a>(
        &'a self,
        account: &'a CheckinAccount,
        cancel: &'a CancellationToken,
    ) -> VerificationFuture<'a> {
        Box::pin(self.get_checkin_status(account, cancel))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Finished(Status),
    Missed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskEvent {
    pub started_at_ms: i64,
    pub finished_at_ms: i64,
    pub status: Status,
    pub may_have_completed: bool,
    pub verification_only: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DailyTask {
    pub account_id: String,
    pub provider: String,
    pub region: String,
    pub business_date: String,
    pub scheduled_at_ms: i64,
    pub state: TaskState,
    pub attempts: u8,
    pub verification_attempts: u8,
    pub last_started_at_ms: Option<i64>,
    pub uncertain: bool,
    pub events: Vec<TaskEvent>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerJournal {
    pub version: u32,
    pub latest_business_date: Option<String>,
    pub tasks: BTreeMap<String, DailyTask>,
}
impl Default for SchedulerJournal {
    fn default() -> Self {
        Self {
            version: 1,
            latest_business_date: None,
            tasks: BTreeMap::new(),
        }
    }
}
#[derive(Default, Debug, Serialize)]
pub struct TickResult {
    pub clock_moved_to_earlier_date: bool,
    pub executed: Vec<DailyTask>,
}

pub struct DailyScheduler<C: Clock, E: ScheduledExecutor> {
    root: PathBuf,
    pub config: SchedulerConfig,
    clock: C,
    executor: E,
}
impl<C: Clock, E: ScheduledExecutor> DailyScheduler<C, E> {
    pub fn new(root: &Path, config: SchedulerConfig, clock: C, executor: E) -> io::Result<Self> {
        config.validate()?;
        Ok(Self {
            root: root.to_path_buf(),
            config,
            clock,
            executor,
        })
    }
    fn journal_path(&self) -> PathBuf {
        self.root.join("daily_scheduler_state.json")
    }
    pub fn snapshot(&self) -> io::Result<SchedulerJournal> {
        let path = self.journal_path();
        if let Ok(meta) = std::fs::metadata(&path) {
            if meta.len() > 16_777_216 {
                return Err(invalid("调度状态文件过大，已停止执行"));
            }
        }
        let journal = match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str::<SchedulerJournal>(&text)
                .map_err(|_| invalid("调度状态损坏，已停止执行"))?,
            Err(e) if e.kind() == ErrorKind::NotFound => SchedulerJournal::default(),
            Err(e) => return Err(e),
        };
        if journal.version != 1 {
            return Err(invalid("调度状态版本不兼容"));
        }
        for (key, task) in &journal.tasks {
            let expected = task_key(
                &task.provider,
                &task.region,
                &task.account_id,
                &task.business_date,
            );
            if *key != expected
                || !valid_date(&task.business_date)
                || task.attempts > 3
                || task.verification_attempts > 1
            {
                return Err(invalid("调度任务身份或日期损坏"));
            }
        }
        if journal
            .latest_business_date
            .as_deref()
            .is_some_and(|v| !valid_date(v))
        {
            return Err(invalid("调度日期损坏"));
        }
        Ok(journal)
    }
    fn save(&self, journal: &SchedulerJournal) -> io::Result<()> {
        durable_write(
            &self.journal_path(),
            &serde_json::to_string_pretty(journal).map_err(|_| invalid("调度状态无法保存"))?,
        )
    }
    fn lock(&self) -> io::Result<std::fs::File> {
        std::fs::create_dir_all(&self.root)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.root.join("daily_scheduler.lock"))?;
        file.try_lock_exclusive()
            .map_err(|_| io::Error::new(ErrorKind::WouldBlock, "另一调度实例正在执行"))?;
        Ok(file)
    }
    pub async fn tick(
        &self,
        accounts: &[CheckinAccount],
        cancel: &CancellationToken,
    ) -> io::Result<TickResult> {
        let (hour, minute) = self.config.validate()?;
        if !self.config.enabled || cancel.is_cancelled() {
            return Ok(TickResult::default());
        }
        // Hold across requests. A crash releases the OS lock but leaves Pending on disk.
        let _lock = self.lock()?;
        let now = self.clock.now_ms();
        let (date, scheduled) = schedule(now, hour, minute)?;
        let mut journal = self.snapshot()?;
        if journal
            .latest_business_date
            .as_deref()
            .is_some_and(|v| v > date.as_str())
        {
            return Ok(TickResult {
                clock_moved_to_earlier_date: true,
                executed: vec![],
            });
        }
        journal.latest_business_date = Some(date.clone());
        self.save(&journal)?;
        if now < scheduled {
            return Ok(TickResult::default());
        }
        let mut result = TickResult::default();
        for account in accounts {
            let current = self.clock.now_ms();
            if cancel.is_cancelled()
                || current < scheduled
                || schedule(current, hour, minute)?.0 != date
            {
                break;
            }
            if !account.enabled
                || account.provider != "workbuddy"
                || account.region != "cn"
                || self.config.account_enabled.get(&account.id) == Some(&false)
            {
                continue;
            }
            let key = task_key(&account.provider, &account.region, &account.id, &date);
            let task = journal
                .tasks
                .entry(key.clone())
                .or_insert_with(|| DailyTask {
                    account_id: account.id.clone(),
                    provider: account.provider.clone(),
                    region: account.region.clone(),
                    business_date: date.clone(),
                    scheduled_at_ms: scheduled,
                    state: TaskState::Pending,
                    attempts: 0,
                    verification_attempts: 0,
                    last_started_at_ms: None,
                    uncertain: false,
                    events: vec![],
                });
            if matches!(
                task.state,
                TaskState::Missed
                    | TaskState::Finished(
                        Status::Success
                            | Status::AlreadyCompleted
                            | Status::AuthExpired
                            | Status::RequiresUserAction
                            | Status::RateLimited
                            | Status::Unsupported
                    )
            ) || matches!(
                task.state,
                TaskState::Finished(Status::Unknown | Status::Cancelled)
            ) && !task.uncertain
            {
                continue;
            }
            let verification_only =
                task.attempts > 0 && (task.state == TaskState::Pending || task.uncertain);
            if verification_only && task.verification_attempts >= 1
                || !verification_only && task.attempts >= self.config.max_daily_attempts
            {
                continue;
            }
            if task.attempts == 0 && !self.config.catch_up && current >= scheduled + 60_000 {
                task.state = TaskState::Missed;
                self.save(&journal)?;
                continue;
            }
            let started = self.clock.now_ms();
            if !verification_only
                && task.last_started_at_ms.is_some_and(|v| {
                    started.saturating_sub(v) < i64::from(self.config.retry_delay_seconds) * 1000
                })
            {
                continue;
            }
            if verification_only {
                task.verification_attempts += 1;
            } else {
                task.attempts += 1;
            }
            task.last_started_at_ms = Some(started);
            task.state = TaskState::Pending;
            task.uncertain = true;
            self.save(&journal)?;
            let (mut status, mut uncertain) = if verification_only {
                match self.executor.verify(account, cancel).await {
                    Ok(true) => (Status::AlreadyCompleted, false),
                    Ok(false) => (Status::Unknown, true),
                    Err(Status::AuthExpired) => (Status::AuthExpired, true),
                    Err(Status::Cancelled) => (Status::Cancelled, true),
                    Err(_) => (Status::Unknown, true),
                }
            } else {
                let outcome = self.executor.execute(account, &date, cancel).await;
                (outcome.status, outcome.may_have_completed)
            };
            let finished = self.clock.now_ms();
            if schedule(finished, hour, minute)?.0 != date
                && matches!(status, Status::Success | Status::AlreadyCompleted)
            {
                // Do not mark an old business date successful based on a response after midnight.
                status = Status::Unknown;
                uncertain = true;
            }
            let task = journal.tasks.get_mut(&key).unwrap();
            task.state = TaskState::Finished(status);
            task.uncertain = uncertain;
            task.events.push(TaskEvent {
                started_at_ms: started,
                finished_at_ms: finished,
                status,
                may_have_completed: uncertain,
                verification_only,
            });
            result.executed.push(task.clone());
            self.save(&journal)?;
        }
        Ok(result)
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}
fn valid_date(date: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .is_ok_and(|v| v.format("%Y-%m-%d").to_string() == date)
}
fn task_key(provider: &str, region: &str, id: &str, date: &str) -> String {
    serde_json::to_string(&[provider, region, id, date]).unwrap()
}
pub fn schedule(now_ms: i64, hour: u32, minute: u32) -> io::Result<(String, i64)> {
    let offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let local = offset
        .timestamp_millis_opt(now_ms)
        .single()
        .ok_or_else(|| invalid("系统时钟超出范围"))?;
    let planned = local
        .with_hour(hour)
        .and_then(|v| v.with_minute(minute))
        .and_then(|v| v.with_second(0))
        .and_then(|v| v.with_nanosecond(0))
        .ok_or_else(|| invalid("计划时间无效"))?;
    Ok((
        local.format("%Y-%m-%d").to_string(),
        planned.timestamp_millis(),
    ))
}
