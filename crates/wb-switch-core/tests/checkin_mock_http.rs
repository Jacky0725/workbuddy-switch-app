use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use wb_switch_core::modules::checkin_service::{
    CancellationToken, CheckinAccount, CheckinResultStatus as S, CheckinService,
    DisabledProductionAdapter, LoopbackCheckinAdapter,
};

const DAY: &str = "2026-10-09";
const NO: &str = r#"{"code":0,"data":{"today_checked_in":false}}"#;
const YES: &str = r#"{"code":0,"data":{"today_checked_in":true}}"#;
const SUCCESS: &str = r#"{"code":0,"data":{}}"#;
#[derive(Clone)]
struct Step {
    path: &'static str,
    status: u16,
    body: &'static str,
    delay: Duration,
    headers: &'static str,
}
fn step(path: &'static str, status: u16, body: &'static str) -> Step {
    Step {
        path,
        status,
        body,
        delay: Duration::ZERO,
        headers: "",
    }
}
type Calls = Arc<Mutex<Vec<(String, String, Instant)>>>;
struct Fixture {
    url: String,
    calls: Calls,
    remaining: Arc<Mutex<VecDeque<Step>>>,
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}
impl Fixture {
    fn new(steps: Vec<Step>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let calls = Arc::new(Mutex::new(vec![]));
        let remaining = Arc::new(Mutex::new(VecDeque::from(steps)));
        let stop = Arc::new(AtomicBool::new(false));
        let (c, r, s) = (calls.clone(), remaining.clone(), stop.clone());
        let join = thread::spawn(move || {
            let mut workers = vec![];
            while !s.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let (c, r) = (c.clone(), r.clone());
                        workers.push(thread::spawn(move || respond(stream, c, r)));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1))
                    }
                    Err(error) => panic!("fixture accept: {error}"),
                }
            }
            for worker in workers {
                worker.join().unwrap();
            }
        });
        Self {
            url,
            calls,
            remaining,
            stop,
            join: Some(join),
        }
    }
    fn service(&self) -> CheckinService<LoopbackCheckinAdapter> {
        self.with_timeout(Duration::from_secs(2))
    }
    fn with_timeout(&self, timeout: Duration) -> CheckinService<LoopbackCheckinAdapter> {
        CheckinService::new(
            LoopbackCheckinAdapter::new(&self.url, timeout).unwrap(),
            Duration::from_millis(1),
        )
    }
    fn count(&self, path: &str) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|v| v.0 == path)
            .count()
    }
    fn assert_drained(&self) {
        assert!(self.remaining.lock().unwrap().is_empty());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.join.take().unwrap().join().unwrap();
    }
}
fn respond(mut stream: TcpStream, calls: Calls, remaining: Arc<Mutex<VecDeque<Step>>>) {
    // Windows accepted sockets can inherit the listener's nonblocking mode.
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let mut bytes = vec![];
    loop {
        let mut part = [0u8; 1024];
        let read = stream.read(&mut part).unwrap_or(0);
        if read == 0 {
            return;
        }
        bytes.extend_from_slice(&part[..read]);
        if let Some(end) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
            let header = String::from_utf8_lossy(&bytes[..end]);
            let length: usize = header
                .lines()
                .find_map(|v| {
                    v.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|v| v.trim().parse().unwrap())
                })
                .unwrap_or(0);
            if bytes.len() >= end + 4 + length {
                break;
            }
        }
        assert!(bytes.len() < 16_384);
    }
    let text = String::from_utf8_lossy(&bytes);
    let path = text
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_string();
    let id = text
        .lines()
        .find_map(|v| {
            v.to_ascii_lowercase()
                .strip_prefix("x-fixture-account:")
                .map(|v| v.trim().to_string())
        })
        .unwrap();
    calls
        .lock()
        .unwrap()
        .push((path.clone(), id, Instant::now()));
    let planned = remaining
        .lock()
        .unwrap()
        .pop_front()
        .unwrap_or_else(|| step("unexpected", 500, "{}"));
    assert_eq!(path, format!("/{}", planned.path));
    thread::sleep(planned.delay);
    let response = format!("HTTP/1.1 {} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}", planned.status, planned.body.len(), planned.headers, planned.body);
    let _ = stream.write_all(response.as_bytes());
}
fn account(id: &str) -> CheckinAccount {
    CheckinAccount {
        id: id.into(),
        provider: "workbuddy".into(),
        region: "cn".into(),
        enabled: true,
    }
}

#[tokio::test]
async fn success_is_logged_once_and_repeated_trigger_does_not_submit_again() {
    let f = Fixture::new(vec![step("status", 200, NO), step("submit", 200, SUCCESS)]);
    let service = f.service();
    let cancel = CancellationToken::default();
    assert_eq!(
        service.checkin(&account("a"), DAY, &cancel).await.status,
        S::Success
    );
    assert_eq!(
        service.checkin(&account("a"), DAY, &cancel).await.status,
        S::AlreadyCompleted
    );
    assert_eq!(f.count("/submit"), 1);
    let history = service.history("a");
    assert_eq!(history.iter().filter(|v| v.status == S::Success).count(), 1);
    assert!(history[0].message.contains("另行查询"));
    assert!(history[0].finished_at_ms >= history[0].started_at_ms);
    f.assert_drained();
}

#[tokio::test]
async fn strict_status_and_error_matrix_never_submits_on_missing_or_changed_fields() {
    for (http, body, expected) in [
        (200, YES, S::AlreadyCompleted),
        (401, r#"{"message":"fake-secret-echo"}"#, S::AuthExpired),
        (403, "{}", S::RequiresUserAction),
        (428, "{}", S::RequiresUserAction),
        (429, "{}", S::RateLimited),
        (500, "{}", S::ServerError),
        (404, "{}", S::Unsupported),
        (
            200,
            r#"{"code":12153,"msg":"fake-secret-echo"}"#,
            S::AuthExpired,
        ),
        (200, r#"{"data":{"today_checked_in":false}}"#, S::Unknown),
        (200, r#"{"code":0,"data":{}}"#, S::Unknown),
        (
            200,
            r#"{"code":0,"data":{"today_checked_in":"false"}}"#,
            S::Unknown,
        ),
        (
            200,
            r#"{"code":0,"data":{"today_checked_in":false,"todayCheckedIn":true}}"#,
            S::Unknown,
        ),
        (
            200,
            r#"{"code":"0","data":{"today_checked_in":false}}"#,
            S::Unknown,
        ),
        (
            200,
            r#"{"code":0,"data":{"renamed_checkin_status":false}}"#,
            S::Unknown,
        ),
        (200, "<html>unexpected</html>", S::Unknown),
    ] {
        let f = Fixture::new(vec![step("status", http, body)]);
        let service = f.service();
        let result = service
            .checkin(&account("a"), DAY, &CancellationToken::default())
            .await;
        assert_eq!(result.status, expected);
        assert_eq!(f.count("/submit"), 0);
        assert!(!serde_json::to_string(&service.history("a"))
            .unwrap()
            .contains("fake-secret-echo"));
        f.assert_drained();
    }
}

#[tokio::test]
async fn submit_response_requires_explicit_business_code_and_data_object() {
    for (http, body, expected, uncertain) in [
        (200, SUCCESS, S::Success, false),
        (
            200,
            r#"{"code":0,"data":{"alreadyCompleted":true}}"#,
            S::AlreadyCompleted,
            false,
        ),
        (200, "{}", S::Unknown, true),
        (200, r#"{"code":0}"#, S::Unknown, true),
        (200, r#"{"code":0,"data":[]}"#, S::Unknown, true),
        (401, "{}", S::AuthExpired, false),
        (429, "{}", S::RateLimited, false),
        (500, "{}", S::ServerError, true),
        (
            400,
            r#"{"code":10001,"msg":"今天已签到","data":{}}"#,
            S::AlreadyCompleted,
            false,
        ),
        (
            401,
            r#"{"code":10001,"msg":"今天已签到","data":{}}"#,
            S::AuthExpired,
            false,
        ),
        (
            400,
            r#"{"code":10001,"msg":"already authorized","data":{}}"#,
            S::Unknown,
            true,
        ),
    ] {
        let f = Fixture::new(vec![step("status", 200, NO), step("submit", http, body)]);
        let result = f
            .service()
            .checkin(&account("a"), DAY, &CancellationToken::default())
            .await;
        assert_eq!(
            (result.status, result.may_have_completed),
            (expected, uncertain)
        );
        f.assert_drained();
    }
}

#[tokio::test]
async fn timeout_after_submit_is_resolved_by_status_without_a_second_submit() {
    let mut delayed = step("submit", 200, SUCCESS);
    delayed.delay = Duration::from_millis(350);
    let f = Fixture::new(vec![
        step("status", 200, NO),
        delayed,
        step("status", 200, YES),
    ]);
    let service = f.with_timeout(Duration::from_millis(100));
    let result = service
        .checkin(&account("a"), DAY, &CancellationToken::default())
        .await;
    assert_eq!(
        (result.status, result.may_have_completed),
        (S::AlreadyCompleted, false)
    );
    assert_eq!(f.count("/submit"), 1);
    f.assert_drained();
}

#[tokio::test]
async fn unresolved_timeout_preserves_uncertainty_and_subsequent_trigger_only_queries() {
    let mut delayed = step("submit", 200, SUCCESS);
    delayed.delay = Duration::from_millis(350);
    let f = Fixture::new(vec![
        step("status", 200, NO),
        delayed,
        step("status", 200, NO),
        step("status", 200, NO),
    ]);
    let service = f.with_timeout(Duration::from_millis(100));
    let cancel = CancellationToken::default();
    let first = service.checkin(&account("a"), DAY, &cancel).await;
    let second = service.checkin(&account("a"), DAY, &cancel).await;
    assert_eq!((first.status, first.may_have_completed), (S::Unknown, true));
    assert_eq!(
        (second.status, second.may_have_completed),
        (S::Unknown, true)
    );
    assert_eq!(f.count("/submit"), 1);
    f.assert_drained();
}

#[tokio::test]
async fn concurrent_duplicate_is_in_progress_and_uses_one_submission() {
    let mut delayed = step("status", 200, NO);
    delayed.delay = Duration::from_millis(80);
    let f = Fixture::new(vec![delayed, step("submit", 200, SUCCESS)]);
    let service = f.service();
    let (account, cancel) = (account("a"), CancellationToken::default());
    let (a, b) = tokio::join!(
        service.checkin(&account, DAY, &cancel),
        service.checkin(&account, DAY, &cancel)
    );
    assert_eq!(a.status, S::Success);
    assert_eq!(b.status, S::InProgress);
    assert_eq!(f.count("/submit"), 1);
    f.assert_drained();
}

#[tokio::test]
async fn three_accounts_partial_failure_and_rate_limit_do_not_abort_the_batch() {
    let f = Fixture::new(vec![
        step("status", 200, NO),
        step("submit", 200, SUCCESS),
        step("status", 401, "{}"),
        step("status", 429, "{}"),
    ]);
    let service = CheckinService::new(
        LoopbackCheckinAdapter::new(&f.url, Duration::from_secs(2)).unwrap(),
        Duration::from_millis(15),
    );
    let cancel = CancellationToken::default();
    let results = service
        .checkin_all(&[account("a"), account("b"), account("c")], DAY, &cancel)
        .await;
    assert_eq!(
        results.iter().map(|v| v.status).collect::<Vec<_>>(),
        vec![S::Success, S::AuthExpired, S::RateLimited]
    );
    assert_eq!(
        service.checkin(&account("b"), DAY, &cancel).await.status,
        S::AuthExpired
    );
    assert_eq!(
        service.checkin(&account("c"), DAY, &cancel).await.status,
        S::RateLimited
    );
    let calls = f.calls.lock().unwrap();
    assert_eq!(calls.len(), 4);
    assert!(calls
        .windows(2)
        .all(|v| v[1].2.duration_since(v[0].2) >= Duration::from_millis(10)));
    f.assert_drained();
}

#[tokio::test]
async fn cancellation_during_submit_stops_remaining_accounts_and_keeps_uncertainty() {
    let mut delayed = step("submit", 200, SUCCESS);
    delayed.delay = Duration::from_millis(200);
    let f = Fixture::new(vec![step("status", 200, NO), delayed]);
    let service = Arc::new(f.service());
    let cancel = CancellationToken::default();
    let (s, c) = (service.clone(), cancel.clone());
    let task =
        tokio::spawn(async move { s.checkin_all(&[account("a"), account("b")], DAY, &c).await });
    tokio::time::timeout(Duration::from_secs(2), async {
        while f.count("/submit") == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    cancel.cancel();
    let results = tokio::time::timeout(Duration::from_millis(100), task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(results[0].status, S::Cancelled);
    assert!(results[0].may_have_completed);
    assert_eq!(results[1].status, S::Cancelled);
    assert!(!results[1].may_have_completed);
    assert_eq!(f.calls.lock().unwrap().len(), 2);
    f.assert_drained();
}

#[tokio::test]
async fn disabled_accounts_regions_and_production_adapter_never_submit() {
    let f = Fixture::new(vec![]);
    let service = f.service();
    for (region, enabled) in [("ai", true), ("cn", false)] {
        let mut a = account("a");
        a.region = region.into();
        a.enabled = enabled;
        assert_eq!(
            service
                .checkin(&a, DAY, &CancellationToken::default())
                .await
                .status,
            S::Unsupported
        );
    }
    assert_eq!(
        service
            .checkin(&account("a"), "2026-1-2", &CancellationToken::default())
            .await
            .status,
        S::Unknown
    );
    assert!(f.calls.lock().unwrap().is_empty());
    let disabled = CheckinService::new(DisabledProductionAdapter, Duration::ZERO);
    assert_eq!(
        disabled
            .checkin(&account("a"), DAY, &CancellationToken::default())
            .await
            .status,
        S::Unsupported
    );
}

#[tokio::test]
async fn closed_loopback_port_is_network_error_and_never_a_success() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    drop(listener);
    let service = CheckinService::new(
        LoopbackCheckinAdapter::new(&url, Duration::from_millis(100)).unwrap(),
        Duration::ZERO,
    );
    assert_eq!(
        service
            .checkin(&account("a"), DAY, &CancellationToken::default())
            .await
            .status,
        S::NetworkError
    );
}

#[test]
fn mock_adapter_rejects_non_loopback_urls_and_url_credentials() {
    for url in [
        "https://www.codebuddy.cn/",
        "http://localhost:8080/",
        "http://0.0.0.0:8080/",
        "http://127.0.0.1/",
        "http://user:secret@127.0.0.1:8080/",
        "http://127.0.0.1:8080/other",
        "http://127.0.0.1:8080/?url=remote",
    ] {
        assert!(LoopbackCheckinAdapter::new(url, Duration::from_secs(1)).is_err());
    }
}

#[tokio::test]
async fn redirects_cannot_escape_to_another_server() {
    let mut redirect = step("status", 302, "{}");
    // Test redirect points to an unused port; redirect policy must prevent dialing it.
    redirect.headers = "Location: http://127.0.0.1:1/\r\n";
    let f = Fixture::new(vec![redirect]);
    let result = f
        .service()
        .checkin(&account("a"), DAY, &CancellationToken::default())
        .await;
    assert_eq!(result.status, S::Unknown);
    assert_eq!(f.calls.lock().unwrap().len(), 1);
    f.assert_drained();
}

#[tokio::test]
async fn persistent_scheduler_uses_the_http_engine_and_does_not_repeat_after_restart() {
    use chrono::TimeZone;
    use wb_switch_core::modules::daily_scheduler::{
        Clock, DailyScheduler, SchedulerConfig, TaskState,
    };
    struct FrozenClock;
    impl Clock for FrozenClock {
        fn now_ms(&self) -> i64 {
            chrono::Utc
                .with_ymd_and_hms(2026, 10, 9, 1, 15, 0)
                .unwrap()
                .timestamp_millis()
        }
    }
    struct StateDir(std::path::PathBuf);
    impl Drop for StateDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let dir =
        StateDir(std::env::temp_dir().join(format!("wbm-http-scheduler-{}", uuid::Uuid::new_v4())));
    let f = Fixture::new(vec![step("status", 200, NO), step("submit", 200, SUCCESS)]);
    let config = SchedulerConfig {
        enabled: true,
        ..SchedulerConfig::default()
    };
    let scheduler = DailyScheduler::new(&dir.0, config.clone(), FrozenClock, f.service()).unwrap();
    let result = scheduler
        .tick(&[account("a")], &CancellationToken::default())
        .await
        .unwrap();
    assert_eq!(result.executed[0].state, TaskState::Finished(S::Success));
    let restarted = DailyScheduler::new(&dir.0, config, FrozenClock, f.service()).unwrap();
    assert!(restarted
        .tick(&[account("a")], &CancellationToken::default())
        .await
        .unwrap()
        .executed
        .is_empty());
    assert_eq!(f.count("/submit"), 1);
    f.assert_drained();
}
