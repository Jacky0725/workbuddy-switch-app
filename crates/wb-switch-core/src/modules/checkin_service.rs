//! Strict, replaceable check-in engine. The only HTTP implementation here is
//! loopback-only for fixtures. Production authorization remains closed.

use crate::modules::config::now_ms;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tokio::sync::Notify;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckinResultStatus {
    Success,
    AlreadyCompleted,
    AuthExpired,
    RequiresUserAction,
    RateLimited,
    NetworkError,
    ServerError,
    Unsupported,
    Unknown,
    Cancelled,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinResult {
    pub account_id: String,
    pub business_date: String,
    pub status: CheckinResultStatus,
    pub started_at_ms: i64,
    pub finished_at_ms: i64,
    pub may_have_completed: bool,
    pub message: String,
}

#[derive(Clone)]
pub struct CheckinAccount {
    pub id: String,
    pub provider: String,
    pub region: String,
    pub enabled: bool,
}

#[derive(Clone, Default)]
pub struct CancellationToken {
    flag: Arc<AtomicBool>,
    signal: Arc<Notify>,
}
impl CancellationToken {
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
        self.signal.notify_waiters();
    }
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
    async fn cancelled(&self) {
        let notified = self.signal.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        if !self.is_cancelled() {
            notified.await;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckinOperation {
    Status,
    Submit,
}

pub struct AdapterResponse {
    pub http_status: u16,
    pub body: Value,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdapterError {
    Timeout,
    Network,
    Disabled,
    Cancelled,
}
pub type AdapterFuture<'a> =
    Pin<Box<dyn Future<Output = Result<AdapterResponse, AdapterError>> + Send + 'a>>;
pub trait CheckinAdapter: Send + Sync {
    fn request<'a>(&'a self, account_id: &'a str, operation: CheckinOperation)
        -> AdapterFuture<'a>;
}

pub struct DisabledProductionAdapter;
impl CheckinAdapter for DisabledProductionAdapter {
    fn request<'a>(&'a self, _: &'a str, _: CheckinOperation) -> AdapterFuture<'a> {
        Box::pin(async { Err(AdapterError::Disabled) })
    }
}

/// Development transport: literal loopback IP, explicit port, no proxy/redirect,
/// no token, no caller-selected operation path. Never exposed through host IPC.
pub struct LoopbackCheckinAdapter {
    base: reqwest::Url,
    client: reqwest::Client,
}
impl LoopbackCheckinAdapter {
    pub fn new(base: &str, timeout: Duration) -> Result<Self, &'static str> {
        let url = reqwest::Url::parse(base).map_err(|_| "Mock 地址无效")?;
        if url.scheme() != "http"
            || !matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "::1"))
            || url.port().is_none()
            || url.path() != "/"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || timeout.is_zero()
            || timeout > Duration::from_secs(30)
        {
            return Err("Mock 适配器只允许带端口的本地回环地址");
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout)
            .build()
            .map_err(|_| "Mock 客户端初始化失败")?;
        Ok(Self { base: url, client })
    }
}
impl CheckinAdapter for LoopbackCheckinAdapter {
    fn request<'a>(
        &'a self,
        account_id: &'a str,
        operation: CheckinOperation,
    ) -> AdapterFuture<'a> {
        Box::pin(async move {
            let path = match operation {
                CheckinOperation::Status => "status",
                CheckinOperation::Submit => "submit",
            };
            let url = self.base.join(path).map_err(|_| AdapterError::Network)?;
            let request = match operation {
                CheckinOperation::Status => self.client.get(url),
                CheckinOperation::Submit => self.client.post(url).json(&serde_json::json!({})),
            };
            let mut response = request
                .header("X-Fixture-Account", account_id)
                .send()
                .await
                .map_err(adapter_error)?;
            let status = response.status().as_u16();
            let mut bytes = vec![];
            while let Some(chunk) = response.chunk().await.map_err(adapter_error)? {
                if bytes.len() + chunk.len() > 65_536 {
                    return Ok(AdapterResponse {
                        http_status: status,
                        body: Value::Null,
                    });
                }
                bytes.extend_from_slice(&chunk);
            }
            Ok(AdapterResponse {
                http_status: status,
                body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
            })
        })
    }
}
fn adapter_error(error: reqwest::Error) -> AdapterError {
    if error.is_timeout() {
        AdapterError::Timeout
    } else {
        AdapterError::Network
    }
}

#[derive(Default)]
struct Attempt {
    running: bool,
    submissions: u8,
    terminal: Option<CheckinResultStatus>,
    uncertain: bool,
}
type Attempts = Arc<Mutex<HashMap<(String, String), Attempt>>>;
struct RunLease {
    attempts: Attempts,
    key: (String, String),
    finished: bool,
    submitting: bool,
}
impl Drop for RunLease {
    fn drop(&mut self) {
        if let Some(slot) = self.attempts.lock().unwrap().get_mut(&self.key) {
            slot.running = false;
            if !self.finished && self.submitting {
                slot.uncertain = true;
                slot.terminal = Some(CheckinResultStatus::Unknown);
            }
        }
    }
}

pub struct CheckinService<A: CheckinAdapter> {
    adapter: A,
    attempts: Attempts,
    history: Mutex<Vec<CheckinResult>>,
    request_gate: tokio::sync::Mutex<Instant>,
    minimum_interval: Duration,
}
impl<A: CheckinAdapter> CheckinService<A> {
    pub fn new(adapter: A, minimum_interval: Duration) -> Self {
        Self {
            adapter,
            attempts: Arc::new(Mutex::new(HashMap::new())),
            history: Mutex::new(vec![]),
            request_gate: tokio::sync::Mutex::new(Instant::now()),
            minimum_interval,
        }
    }
    pub fn history(&self, account_id: &str) -> Vec<CheckinResult> {
        self.history
            .lock()
            .unwrap()
            .iter()
            .filter(|v| v.account_id == account_id)
            .cloned()
            .collect()
    }

    pub async fn get_checkin_status(
        &self,
        account: &CheckinAccount,
        cancel: &CancellationToken,
    ) -> Result<bool, CheckinResultStatus> {
        if !account.enabled || account.provider != "workbuddy" || account.region != "cn" {
            return Err(CheckinResultStatus::Unsupported);
        }
        let response = self
            .request(&account.id, CheckinOperation::Status, cancel)
            .await
            .map_err(failure_status)?;
        parse_status(&response)
    }
    fn result(
        &self,
        account: &CheckinAccount,
        date: &str,
        started: i64,
        status: CheckinResultStatus,
        uncertain: bool,
    ) -> CheckinResult {
        let result = CheckinResult {
            account_id: account.id.clone(),
            business_date: date.into(),
            status,
            started_at_ms: started,
            finished_at_ms: now_ms(),
            may_have_completed: uncertain,
            message: message(status).into(),
        };
        let mut history = self.history.lock().unwrap();
        if history.len() >= 1024 {
            history.remove(0);
        }
        history.push(result.clone());
        result
    }
    async fn request(
        &self,
        id: &str,
        operation: CheckinOperation,
        cancel: &CancellationToken,
    ) -> Result<AdapterResponse, AdapterError> {
        let mut next = tokio::select! { _ = cancel.cancelled() => return Err(AdapterError::Cancelled), guard = self.request_gate.lock() => guard };
        let delay = next.saturating_duration_since(Instant::now());
        tokio::select! { _ = cancel.cancelled() => return Err(AdapterError::Cancelled), _ = tokio::time::sleep(delay) => {} }
        *next = Instant::now() + self.minimum_interval;
        let value = tokio::select! { _ = cancel.cancelled() => Err(AdapterError::Cancelled), value = self.adapter.request(id, operation) => value };
        drop(next);
        value
    }
    pub async fn checkin(
        &self,
        account: &CheckinAccount,
        date: &str,
        cancel: &CancellationToken,
    ) -> CheckinResult {
        let started = now_ms();
        if cancel.is_cancelled() {
            return self.result(
                account,
                date,
                started,
                CheckinResultStatus::Cancelled,
                false,
            );
        }
        if !chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .is_ok_and(|v| v.format("%Y-%m-%d").to_string() == date)
        {
            return self.result(account, date, started, CheckinResultStatus::Unknown, false);
        }
        if !account.enabled || account.region != "cn" || account.provider != "workbuddy" {
            return self.result(
                account,
                date,
                started,
                CheckinResultStatus::Unsupported,
                false,
            );
        }
        let key = (account.id.clone(), date.to_string());
        let (prior_uncertain, submissions) = {
            let mut attempts = self.attempts.lock().unwrap();
            let slot = attempts.entry(key.clone()).or_default();
            if slot.running {
                return self.result(
                    account,
                    date,
                    started,
                    CheckinResultStatus::InProgress,
                    slot.uncertain,
                );
            }
            match slot.terminal {
                Some(CheckinResultStatus::Success | CheckinResultStatus::AlreadyCompleted) => {
                    return self.result(
                        account,
                        date,
                        started,
                        CheckinResultStatus::AlreadyCompleted,
                        false,
                    )
                }
                Some(
                    status @ (CheckinResultStatus::AuthExpired
                    | CheckinResultStatus::RequiresUserAction
                    | CheckinResultStatus::RateLimited),
                ) => return self.result(account, date, started, status, slot.uncertain),
                _ => {}
            }
            slot.running = true;
            (slot.uncertain, slot.submissions)
        };
        let mut lease = RunLease {
            attempts: self.attempts.clone(),
            key: key.clone(),
            finished: false,
            submitting: false,
        };
        let (status, uncertain) = match self
            .request(&account.id, CheckinOperation::Status, cancel)
            .await
        {
            Err(error) => (failure_status(error), prior_uncertain),
            Ok(response) => match parse_status(&response) {
                Err(status) => (status, prior_uncertain),
                Ok(true) => (CheckinResultStatus::AlreadyCompleted, false),
                Ok(false) if prior_uncertain || submissions >= 2 => {
                    (CheckinResultStatus::Unknown, prior_uncertain)
                }
                Ok(false) if cancel.is_cancelled() => (CheckinResultStatus::Cancelled, false),
                Ok(false) => {
                    lease.submitting = true;
                    self.attempts
                        .lock()
                        .unwrap()
                        .get_mut(&key)
                        .unwrap()
                        .submissions += 1;
                    match self
                        .request(&account.id, CheckinOperation::Submit, cancel)
                        .await
                    {
                        Ok(response) => {
                            let status = parse_submit(&response);
                            (
                                status,
                                matches!(
                                    status,
                                    CheckinResultStatus::Unknown | CheckinResultStatus::ServerError
                                ),
                            )
                        }
                        Err(AdapterError::Timeout | AdapterError::Network) => {
                            // A timed-out POST is never retried inline. Verify exactly once.
                            match self
                                .request(&account.id, CheckinOperation::Status, cancel)
                                .await
                            {
                                Ok(response) if parse_status(&response) == Ok(true) => {
                                    (CheckinResultStatus::AlreadyCompleted, false)
                                }
                                _ => (CheckinResultStatus::Unknown, true),
                            }
                        }
                        Err(error) => (failure_status(error), error == AdapterError::Cancelled),
                    }
                }
            },
        };
        {
            let mut attempts = self.attempts.lock().unwrap();
            let slot = attempts.get_mut(&key).unwrap();
            slot.terminal = Some(status);
            slot.uncertain = uncertain;
        }
        lease.finished = true;
        self.result(account, date, started, status, uncertain)
    }
    /// Sequential execution bounds concurrency to one; errors do not abort other accounts.
    pub async fn checkin_all(
        &self,
        accounts: &[CheckinAccount],
        date: &str,
        cancel: &CancellationToken,
    ) -> Vec<CheckinResult> {
        let mut results = Vec::with_capacity(accounts.len());
        for account in accounts {
            results.push(self.checkin(account, date, cancel).await);
        }
        results
    }
}

fn failure_status(error: AdapterError) -> CheckinResultStatus {
    match error {
        AdapterError::Timeout | AdapterError::Network => CheckinResultStatus::NetworkError,
        AdapterError::Disabled => CheckinResultStatus::Unsupported,
        AdapterError::Cancelled => CheckinResultStatus::Cancelled,
    }
}
fn response_data(response: &AdapterResponse) -> Result<&Value, CheckinResultStatus> {
    use CheckinResultStatus::*;
    match response.http_status {
        401 => return Err(AuthExpired),
        403 | 428 => return Err(RequiresUserAction),
        429 => return Err(RateLimited),
        500..=599 => return Err(ServerError),
        404 => return Err(Unsupported),
        200..=299 => {}
        _ => return Err(Unknown),
    }
    let code = response
        .body
        .get("code")
        .and_then(Value::as_i64)
        .ok_or(Unknown)?;
    match code {
        401 | 12153 => return Err(AuthExpired),
        403 | 428 => return Err(RequiresUserAction),
        429 => return Err(RateLimited),
        500..=599 => return Err(ServerError),
        0 | 200 => {}
        _ => return Err(Unknown),
    }
    let data = response
        .body
        .get("data")
        .filter(|v| v.is_object())
        .ok_or(Unknown)?;
    Ok(data)
}
fn parse_status(response: &AdapterResponse) -> Result<bool, CheckinResultStatus> {
    let data = response_data(response)?;
    let snake = data.get("today_checked_in");
    let camel = data.get("todayCheckedIn");
    match (snake, camel) {
        (Some(a), Some(b)) if a != b => Err(CheckinResultStatus::Unknown),
        (Some(a), _) | (_, Some(a)) => a.as_bool().ok_or(CheckinResultStatus::Unknown),
        _ => Err(CheckinResultStatus::Unknown),
    }
}
fn parse_submit(response: &AdapterResponse) -> CheckinResultStatus {
    // Helper's tested repeat response: HTTP 400 + code 10001 + explicit
    // check-in wording. Do not treat arbitrary "already"/auth errors as success.
    let repeat_message = response
        .body
        .get("message")
        .or_else(|| response.body.get("msg"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if matches!(response.http_status, 200..=299 | 400)
        && response.body.get("code").and_then(Value::as_i64) == Some(10001)
        && response.body.get("data").is_some_and(Value::is_object)
        && matches!(
            repeat_message.as_str(),
            "今天已签到" | "今日已签到" | "已签到" | "already checked in"
        )
    {
        return CheckinResultStatus::AlreadyCompleted;
    }
    match response_data(response) {
        Ok(data) if data.get("alreadyCompleted").and_then(Value::as_bool) == Some(true) => {
            CheckinResultStatus::AlreadyCompleted
        }
        Ok(_) => CheckinResultStatus::Success,
        Err(status) => status,
    }
}
fn message(status: CheckinResultStatus) -> &'static str {
    use CheckinResultStatus::*;
    match status {
        Success => "签到已确认，积分到账需另行查询",
        AlreadyCompleted => "当日已完成",
        AuthExpired => "授权已失效，请重新登录",
        RequiresUserAction => "需要用户在官方客户端处理",
        RateLimited => "服务端限速，已停止当日自动尝试",
        NetworkError => "网络请求失败",
        ServerError => "服务端异常",
        Unsupported => "该能力未开放或未启用",
        Unknown => "结果不确定，请核实后处理",
        Cancelled => "任务已取消",
        InProgress => "同账号任务正在执行",
    }
}
