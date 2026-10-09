use chrono::{FixedOffset, TimeZone};
use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::{
        atomic::{AtomicI64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use wb_switch_core::modules::{
    checkin_service::{CancellationToken, CheckinAccount, CheckinResult, CheckinResultStatus as S},
    daily_scheduler::{
        schedule, Clock, DailyScheduler, DailyTask, ExecutionFuture, ScheduledExecutor,
        SchedulerConfig, SchedulerJournal, TaskState, VerificationFuture,
    },
};

#[derive(Clone)]
struct TestClock(Arc<AtomicI64>);
impl Clock for TestClock {
    fn now_ms(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}
impl TestClock {
    fn at(day: u32, hour: u32, minute: u32) -> Self {
        Self(Arc::new(AtomicI64::new(at(day, hour, minute))))
    }
    fn set(&self, day: u32, hour: u32, minute: u32) {
        self.0.store(at(day, hour, minute), Ordering::SeqCst);
    }
}
fn at(day: u32, hour: u32, minute: u32) -> i64 {
    FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .with_ymd_and_hms(2026, 10, day, hour, minute, 0)
        .unwrap()
        .timestamp_millis()
}
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("wbm-scheduler-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn state(&self) -> PathBuf {
        self.0.join("daily_scheduler_state.json")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone)]
struct Executor {
    results: Arc<Mutex<VecDeque<(S, bool)>>>,
    verifications: Arc<Mutex<VecDeque<Result<bool, S>>>>,
    calls: Arc<AtomicUsize>,
    verify_calls: Arc<AtomicUsize>,
    observe: Option<PathBuf>,
    clock_after: Option<TestClock>,
    entered: Arc<tokio::sync::Notify>,
    block: Option<Arc<tokio::sync::Notify>>,
    fail_publish: bool,
    cancel_after: bool,
}
impl Executor {
    fn new(results: Vec<(S, bool)>) -> Self {
        Self {
            results: Arc::new(Mutex::new(results.into())),
            verifications: Arc::new(Mutex::new(VecDeque::new())),
            calls: Arc::new(AtomicUsize::new(0)),
            verify_calls: Arc::new(AtomicUsize::new(0)),
            observe: None,
            clock_after: None,
            entered: Arc::new(tokio::sync::Notify::new()),
            block: None,
            fail_publish: false,
            cancel_after: false,
        }
    }
    fn count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}
impl ScheduledExecutor for Executor {
    fn execute<'a>(
        &'a self,
        account: &'a CheckinAccount,
        date: &'a str,
        cancel: &'a CancellationToken,
    ) -> ExecutionFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(path) = &self.observe {
                let journal: SchedulerJournal =
                    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
                let task = journal
                    .tasks
                    .values()
                    .find(|v| v.account_id == account.id && v.business_date == date)
                    .unwrap();
                assert_eq!(task.state, TaskState::Pending);
                assert!(task.uncertain);
                assert!(task.attempts > 0);
            }
            self.entered.notify_one();
            if let Some(block) = &self.block {
                block.notified().await;
            }
            if let Some(clock) = &self.clock_after {
                clock.set(10, 0, 0);
            }
            if self.fail_publish {
                let path = self.observe.as_ref().unwrap();
                let mut permissions = std::fs::metadata(path).unwrap().permissions();
                permissions.set_readonly(true);
                std::fs::set_permissions(path, permissions).unwrap();
            }
            if self.cancel_after {
                cancel.cancel();
            }
            let (status, may_have_completed) = self
                .results
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or((S::Success, false));
            CheckinResult {
                account_id: account.id.clone(),
                business_date: date.into(),
                status,
                started_at_ms: 0,
                finished_at_ms: 0,
                may_have_completed,
                message: "fixture".into(),
            }
        })
    }
    fn verify<'a>(
        &'a self,
        _: &'a CheckinAccount,
        _: &'a CancellationToken,
    ) -> VerificationFuture<'a> {
        Box::pin(async move {
            self.verify_calls.fetch_add(1, Ordering::SeqCst);
            self.verifications
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or(Ok(true))
        })
    }
}
fn account(id: &str) -> CheckinAccount {
    CheckinAccount {
        id: id.into(),
        provider: "workbuddy".into(),
        region: "cn".into(),
        enabled: true,
    }
}
fn config() -> SchedulerConfig {
    SchedulerConfig {
        enabled: true,
        ..SchedulerConfig::default()
    }
}
fn scheduler(
    f: &Fixture,
    c: &TestClock,
    e: &Executor,
    cfg: SchedulerConfig,
) -> DailyScheduler<TestClock, Executor> {
    DailyScheduler::new(&f.0, cfg, c.clone(), e.clone()).unwrap()
}
fn interrupted(f: &Fixture) {
    let task = DailyTask {
        account_id: "a".into(),
        provider: "workbuddy".into(),
        region: "cn".into(),
        business_date: "2026-10-09".into(),
        scheduled_at_ms: at(9, 9, 15),
        state: TaskState::Pending,
        attempts: 1,
        verification_attempts: 0,
        last_started_at_ms: Some(at(9, 9, 15)),
        uncertain: true,
        events: vec![],
    };
    let key = serde_json::to_string(&["workbuddy", "cn", "a", "2026-10-09"]).unwrap();
    let journal = SchedulerJournal {
        version: 1,
        latest_business_date: Some("2026-10-09".into()),
        tasks: BTreeMap::from([(key, task)]),
    };
    std::fs::write(f.state(), serde_json::to_string(&journal).unwrap()).unwrap();
}

#[test]
fn beijing_schedule_does_not_depend_on_host_local_timezone() {
    let utc = chrono::Utc
        .with_ymd_and_hms(2026, 10, 9, 1, 15, 0)
        .unwrap()
        .timestamp_millis();
    assert_eq!(schedule(utc, 9, 15).unwrap(), ("2026-10-09".into(), utc));
    assert_eq!(schedule(at(9, 0, 0), 9, 15).unwrap().0, "2026-10-09");
}

#[test]
fn configuration_defaults_validation_and_roundtrip() {
    let f = Fixture::new();
    let defaults = SchedulerConfig::load(&f.0).unwrap();
    assert_eq!(defaults.daily_time, "09:15");
    assert_eq!(defaults.timezone, "Asia/Shanghai");
    assert!(defaults.catch_up);
    assert!(!defaults.enabled);
    let mut cfg = config();
    cfg.account_enabled.insert("a".into(), false);
    cfg.daily_time = "22:10".into();
    cfg.save(&f.0).unwrap();
    assert_eq!(
        SchedulerConfig::load(&f.0).unwrap().account_enabled["a"],
        false
    );
    for bad in ["24:00", "09:60", "9:15", "09:1", "-1:00", "foo"] {
        cfg.daily_time = bad.into();
        assert!(cfg.save(&f.0).is_err());
    }
    cfg.daily_time = "09:15".into();
    cfg.timezone = "Local".into();
    assert!(cfg.validate().is_err());
}

#[tokio::test]
async fn before_due_at_due_after_due_and_backward_time_only_execute_once() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 14);
    let e = Executor::new(vec![]);
    let s = scheduler(&f, &c, &e, config());
    let cancel = CancellationToken::default();
    assert!(s
        .tick(&[account("a")], &cancel)
        .await
        .unwrap()
        .executed
        .is_empty());
    c.set(9, 9, 15);
    assert_eq!(
        s.tick(&[account("a")], &cancel)
            .await
            .unwrap()
            .executed
            .len(),
        1
    );
    c.set(9, 10, 0);
    s.tick(&[account("a")], &cancel).await.unwrap();
    c.set(9, 9, 14);
    s.tick(&[account("a")], &cancel).await.unwrap();
    c.set(9, 9, 15);
    s.tick(&[account("a")], &cancel).await.unwrap();
    assert_eq!(e.count(), 1);
    let snapshot = s.snapshot().unwrap();
    let task = snapshot.tasks.values().next().unwrap();
    assert_eq!(task.events.len(), 1);
    assert_eq!(task.events[0].started_at_ms, at(9, 9, 15));
}

#[tokio::test]
async fn restart_preserves_daily_completion_and_new_day_is_independent() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![]);
    let cancel = CancellationToken::default();
    scheduler(&f, &c, &e, config())
        .tick(&[account("a")], &cancel)
        .await
        .unwrap();
    scheduler(&f, &c, &e, config())
        .tick(&[account("a")], &cancel)
        .await
        .unwrap();
    assert_eq!(e.count(), 1);
    c.set(10, 0, 0);
    scheduler(&f, &c, &e, config())
        .tick(&[account("a")], &cancel)
        .await
        .unwrap();
    assert_eq!(e.count(), 1);
    c.set(10, 9, 15);
    let s = scheduler(&f, &c, &e, config());
    s.tick(&[account("a")], &cancel).await.unwrap();
    assert_eq!(e.count(), 2);
    assert_eq!(s.snapshot().unwrap().tasks.len(), 2);
}

#[tokio::test]
async fn catchup_enabled_runs_once_and_disabled_marks_missed() {
    for enabled in [true, false] {
        let f = Fixture::new();
        let c = TestClock::at(9, 11, 0);
        let e = Executor::new(vec![]);
        let mut cfg = config();
        cfg.catch_up = enabled;
        let s = scheduler(&f, &c, &e, cfg);
        let cancel = CancellationToken::default();
        s.tick(&[account("a")], &cancel).await.unwrap();
        s.tick(&[account("a")], &cancel).await.unwrap();
        assert_eq!(e.count(), usize::from(enabled));
        if !enabled {
            assert_eq!(
                s.snapshot().unwrap().tasks.values().next().unwrap().state,
                TaskState::Missed
            );
        }
    }
}

#[tokio::test]
async fn custom_time_and_per_account_switches_are_respected() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![]);
    let mut cfg = config();
    cfg.daily_time = "22:10".into();
    cfg.account_enabled.insert("a".into(), false);
    let s = scheduler(&f, &c, &e, cfg);
    let cancel = CancellationToken::default();
    let mut disabled = account("c");
    disabled.enabled = false;
    let mut intl = account("d");
    intl.region = "ai".into();
    let accounts = [account("a"), account("b"), disabled, intl];
    s.tick(&accounts, &cancel).await.unwrap();
    assert_eq!(e.count(), 0);
    c.set(9, 22, 10);
    let result = s.tick(&accounts, &cancel).await.unwrap();
    assert_eq!(e.count(), 1);
    assert_eq!(result.executed[0].account_id, "b");
}

#[tokio::test]
async fn network_failure_recovers_after_delay_and_does_not_loop_indefinitely() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![(S::NetworkError, false), (S::Success, false)]);
    let s = scheduler(&f, &c, &e, config());
    let cancel = CancellationToken::default();
    s.tick(&[account("a")], &cancel).await.unwrap();
    c.set(9, 9, 16);
    s.tick(&[account("a")], &cancel).await.unwrap();
    assert_eq!(e.count(), 1);
    c.set(9, 9, 20);
    s.tick(&[account("a")], &cancel).await.unwrap();
    c.set(9, 10, 0);
    s.tick(&[account("a")], &cancel).await.unwrap();
    assert_eq!(e.count(), 2);
    let task = s.snapshot().unwrap().tasks.into_values().next().unwrap();
    assert_eq!(
        task.events.iter().map(|v| v.status).collect::<Vec<_>>(),
        vec![S::NetworkError, S::Success]
    );
}

#[tokio::test]
async fn daily_retry_budget_survives_restart() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![(S::NetworkError, false), (S::ServerError, false)]);
    let cancel = CancellationToken::default();
    scheduler(&f, &c, &e, config())
        .tick(&[account("a")], &cancel)
        .await
        .unwrap();
    c.set(9, 9, 20);
    scheduler(&f, &c, &e, config())
        .tick(&[account("a")], &cancel)
        .await
        .unwrap();
    c.set(9, 10, 0);
    scheduler(&f, &c, &e, config())
        .tick(&[account("a")], &cancel)
        .await
        .unwrap();
    assert_eq!(e.count(), 2);
}

#[tokio::test]
async fn interrupted_post_is_only_verified_even_when_execution_budget_is_exhausted() {
    let f = Fixture::new();
    interrupted(&f);
    let c = TestClock::at(9, 9, 30);
    let e = Executor::new(vec![]);
    let mut cfg = config();
    cfg.max_daily_attempts = 1;
    let s = scheduler(&f, &c, &e, cfg);
    let result = s
        .tick(&[account("a")], &CancellationToken::default())
        .await
        .unwrap();
    assert_eq!(e.count(), 0);
    assert_eq!(e.verify_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        result.executed[0].state,
        TaskState::Finished(S::AlreadyCompleted)
    );
    assert!(result.executed[0].events[0].verification_only);
}

#[tokio::test]
async fn unconfirmed_post_remains_unknown_and_is_not_resubmitted() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![(S::Unknown, true)]);
    e.verifications.lock().unwrap().push_back(Ok(false));
    let cancel = CancellationToken::default();
    scheduler(&f, &c, &e, config())
        .tick(&[account("a")], &cancel)
        .await
        .unwrap();
    c.set(9, 10, 0);
    let s = scheduler(&f, &c, &e, config());
    s.tick(&[account("a")], &cancel).await.unwrap();
    s.tick(&[account("a")], &cancel).await.unwrap();
    assert_eq!(e.count(), 1);
    assert_eq!(e.verify_calls.load(Ordering::SeqCst), 1);
    assert!(
        s.snapshot()
            .unwrap()
            .tasks
            .values()
            .next()
            .unwrap()
            .uncertain
    );
}

#[tokio::test]
async fn forward_and_backward_date_changes_do_not_run_prior_days() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![]);
    let s = scheduler(&f, &c, &e, config());
    let cancel = CancellationToken::default();
    s.tick(&[account("a")], &cancel).await.unwrap();
    c.set(11, 12, 0);
    s.tick(&[account("a")], &cancel).await.unwrap();
    let before = std::fs::read(f.state()).unwrap();
    c.set(10, 12, 0);
    assert!(
        s.tick(&[account("a")], &cancel)
            .await
            .unwrap()
            .clock_moved_to_earlier_date
    );
    assert_eq!(std::fs::read(f.state()).unwrap(), before);
    assert_eq!(e.count(), 2);
    assert!(!s
        .snapshot()
        .unwrap()
        .tasks
        .values()
        .any(|v| v.business_date == "2026-10-10"));
}

#[tokio::test]
async fn crossing_midnight_during_execution_does_not_report_wrong_date_success() {
    let f = Fixture::new();
    let c = TestClock::at(9, 23, 59);
    let mut e = Executor::new(vec![]);
    e.clock_after = Some(c.clone());
    let s = scheduler(&f, &c, &e, config());
    let result = s
        .tick(&[account("a"), account("b")], &CancellationToken::default())
        .await
        .unwrap();
    assert_eq!(e.count(), 1);
    assert_eq!(result.executed[0].state, TaskState::Finished(S::Unknown));
    assert!(result.executed[0].uncertain);
    assert_eq!(result.executed[0].business_date, "2026-10-09");
}

#[tokio::test]
async fn three_accounts_have_independent_terminal_statuses() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![
        (S::Success, false),
        (S::AuthExpired, false),
        (S::RateLimited, false),
    ]);
    let s = scheduler(&f, &c, &e, config());
    let cancel = CancellationToken::default();
    let accounts = [account("a"), account("b"), account("c")];
    let result = s.tick(&accounts, &cancel).await.unwrap();
    assert_eq!(
        result
            .executed
            .iter()
            .map(|v| v.state.clone())
            .collect::<Vec<_>>(),
        vec![
            TaskState::Finished(S::Success),
            TaskState::Finished(S::AuthExpired),
            TaskState::Finished(S::RateLimited)
        ]
    );
    c.set(9, 12, 0);
    s.tick(&accounts, &cancel).await.unwrap();
    assert_eq!(e.count(), 3);
}

#[tokio::test]
async fn corrupted_or_future_journal_is_preserved_without_requests() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![]);
    let s = scheduler(&f, &c, &e, config());
    for text in [
        "broken",
        r#"{"version":99,"latest_business_date":null,"tasks":{}}"#,
    ] {
        std::fs::write(f.state(), text).unwrap();
        assert!(s
            .tick(&[account("a")], &CancellationToken::default())
            .await
            .is_err());
        assert_eq!(std::fs::read_to_string(f.state()).unwrap(), text);
    }
    assert_eq!(e.count(), 0);
}

#[tokio::test]
async fn disabled_or_cancelled_scheduler_has_no_tasks_and_no_side_effects() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let e = Executor::new(vec![]);
    let s = scheduler(&f, &c, &e, SchedulerConfig::default());
    s.tick(&[account("a")], &CancellationToken::default())
        .await
        .unwrap();
    assert!(!f.state().exists());
    let cancel = CancellationToken::default();
    cancel.cancel();
    scheduler(&f, &c, &e, config())
        .tick(&[account("a")], &cancel)
        .await
        .unwrap();
    assert!(!f.state().exists());
    assert_eq!(e.count(), 0);
}

#[tokio::test]
async fn pending_is_flushed_before_execution_and_cancellation_stops_the_batch() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let mut e = Executor::new(vec![(S::Cancelled, true)]);
    e.observe = Some(f.state());
    e.cancel_after = true;
    let s = scheduler(&f, &c, &e, config());
    let result = s
        .tick(&[account("a"), account("b")], &CancellationToken::default())
        .await
        .unwrap();
    assert_eq!(e.count(), 1);
    assert!(result.executed[0].uncertain);
}

#[tokio::test]
async fn two_instances_cannot_execute_concurrently() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let mut e = Executor::new(vec![]);
    let release = Arc::new(tokio::sync::Notify::new());
    e.block = Some(release.clone());
    let first = Arc::new(scheduler(&f, &c, &e, config()));
    let s = first.clone();
    let task = tokio::spawn(async move {
        s.tick(&[account("a")], &CancellationToken::default())
            .await
            .unwrap()
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), e.entered.notified())
        .await
        .unwrap();
    let second = scheduler(&f, &c, &e, config());
    assert_eq!(
        second
            .tick(&[account("a")], &CancellationToken::default())
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::WouldBlock
    );
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let script="$f=[IO.File]::Open($env:WBM_LOCK_FIXTURE,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::ReadWrite); try {$f.Lock(0,1); $f.Unlock(0,1); exit 2} catch [IO.IOException] {exit 0} finally {$f.Dispose()}";
        let output = std::process::Command::new("powershell.exe")
            .creation_flags(0x08000000)
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .env("WBM_LOCK_FIXTURE", f.0.join("daily_scheduler.lock"))
            .output()
            .unwrap();
        assert!(output.status.success(), "child lock probe failed");
    }
    release.notify_one();
    task.await.unwrap();
    assert_eq!(e.count(), 1);
}

#[cfg(windows)]
#[tokio::test]
async fn publish_failure_after_execution_leaves_pending_for_verify_only_recovery() {
    let f = Fixture::new();
    let c = TestClock::at(9, 9, 15);
    let mut e = Executor::new(vec![]);
    e.observe = Some(f.state());
    e.fail_publish = true;
    let s = scheduler(&f, &c, &e, config());
    assert!(s
        .tick(&[account("a")], &CancellationToken::default())
        .await
        .is_err());
    let mut permissions = std::fs::metadata(f.state()).unwrap().permissions();
    permissions.set_readonly(false);
    std::fs::set_permissions(f.state(), permissions).unwrap();
    assert_eq!(
        s.snapshot().unwrap().tasks.values().next().unwrap().state,
        TaskState::Pending
    );
    let recovered = Executor::new(vec![]);
    let s = scheduler(&f, &c, &recovered, config());
    s.tick(&[account("a")], &CancellationToken::default())
        .await
        .unwrap();
    assert_eq!(recovered.count(), 0);
    assert_eq!(recovered.verify_calls.load(Ordering::SeqCst), 1);
}
