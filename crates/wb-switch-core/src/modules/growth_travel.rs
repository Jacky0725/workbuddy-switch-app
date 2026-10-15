//! Fixed travel endpoints, sharing the ordinary reward lock and durable operation journal.
use super::{
    account, config,
    credential_store::{CapabilityType, CredentialStore},
    growth,
    growth_automation::{self, ActivityOperation, OperationState, RunReport},
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};
const PREFIX: &str = "/activity/growth/buddy/travel";
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelState {
    pub account_id: String,
    pub state: String,
    pub daily_limit_reached: bool,
    pub record_id: Option<i64>,
    pub location_id: Option<i64>,
    pub depart_at: Option<i64>,
    pub arrive_at: Option<i64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    pub id: i64,
    pub name: String,
}
#[derive(Serialize)]
pub struct TravelSnapshot {
    pub state: TravelState,
    pub enabled: bool,
    pub locations: Vec<Location>,
}
fn preflight(a: &Value) -> Result<(), String> {
    let current = account::try_load_accounts()
        .map_err(|_| "账号库不可读取")?
        .into_iter()
        .find(|c| c["id"] == a["id"])
        .ok_or("账号已移除")?;
    if growth::identity(&current) != growth::identity(a)
        || current["access_token"] != a["access_token"]
        || current["is_enabled"] == false
        || current["needs_relogin"] == true
        || current["credential_locked"] == true
    {
        return Err("账号已变化或不可用，已停止旅行操作".into());
    }
    if let Some(e) = growth::query_error(&current) {
        return Err(e.into());
    }
    Ok(())
}
async fn request(a: &Value, path: &str, body: Option<Value>) -> Result<Value, String> {
    if !["status", "config", "depart", "claim"].contains(&path) {
        return Err("旅行路径未接入".into());
    }
    preflight(a)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "旅行连接初始化失败")?;
    let url = format!("https://copilot.tencent.com{PREFIX}/{path}");
    let mut r = if let Some(body) = body {
        client.post(url).json(&body)
    } else {
        client.get(url)
    };
    for (k, v) in account::build_auth_headers(a) {
        r = r.header(k, v);
    }
    let mut response = r.send().await.map_err(|_| "旅行请求结果未知，等待核实")?;
    if response.status() != 200 {
        return Err(format!(
            "旅行返回 HTTP {}，停止提交",
            response.status().as_u16()
        ));
    }
    let mut bytes = vec![];
    while let Some(c) = response
        .chunk()
        .await
        .map_err(|_| "旅行响应中断，等待核实")?
    {
        if bytes.len() + c.len() > 256 * 1024 {
            return Err("旅行响应超限".into());
        }
        bytes.extend_from_slice(&c);
    }
    let data: Value = serde_json::from_slice(&bytes).map_err(|_| "旅行响应无效")?;
    if data["code"] != 0 {
        return Err("旅行业务状态未通过，停止提交".into());
    }
    Ok(data["data"].clone())
}
fn parse_state(id: &str, v: &Value) -> Result<TravelState, String> {
    let state =
        super::travel::parse_travel_state(v["state"].as_str()).ok_or("旅行状态未知，停止提交")?;
    let limit = v["daily_limit_reached"]
        .as_bool()
        .ok_or("旅行次数状态缺失")?;
    let record = v["record_id"].as_i64().filter(|n| *n > 0);
    if state != "idle" && record.is_none() {
        return Err("旅行记录缺失，停止提交".into());
    }
    Ok(TravelState {
        account_id: id.into(),
        state: state.into(),
        daily_limit_reached: limit,
        record_id: record,
        location_id: v["location"]["id"].as_i64(),
        depart_at: v["depart_at"].as_i64(),
        arrive_at: v["arrive_at"].as_i64(),
    })
}
async fn status(a: &Value) -> Result<TravelState, String> {
    parse_state(
        a["id"].as_str().ok_or("账号标识缺失")?,
        &request(a, "status", None).await?,
    )
}
async fn locations(a: &Value) -> Result<(bool, Vec<Location>), String> {
    let v = request(a, "config", None).await?;
    // Verified official v1 config has no enabled flag: success + server time +
    // an offered location list constitutes availability. Explicit false wins.
    let enabled = match v.get("enabled") {
        Some(value) => value.as_bool().ok_or("旅行活动状态无效")?,
        None if v["server_now"].as_i64().is_some_and(|n| n > 0) => true,
        None => return Err("旅行活动状态缺失".into()),
    };
    let rows = v["locations"]
        .as_array()
        .filter(|r| r.len() <= 64)
        .ok_or("旅行目的地列表无效")?;
    let mut result = vec![];
    for row in rows {
        let id = row["id"]
            .as_i64()
            .filter(|id| *id > 0)
            .ok_or("旅行目的地无效")?;
        let name = row["name"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(128)
            .collect();
        result.push(Location { id, name });
    }
    Ok((enabled, result))
}
pub fn query(
    account_id: &str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<TravelSnapshot, String>> + Send + '_>>
{
    Box::pin(query_inner(account_id))
}
async fn query_inner(account_id: &str) -> Result<TravelSnapshot, String> {
    let a = account::find_account(account_id).ok_or("账号不存在")?;
    let a = growth::prepare_account(&a).await;
    let state = status(&a).await?;
    let (enabled, locations) = locations(&a).await?;
    Ok(TravelSnapshot {
        state,
        enabled,
        locations,
    })
}
fn key(a: &Value, action: &str, period: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(
            json!([growth::identity(a), "travel", action, period])
                .to_string()
                .as_bytes()
        )
    )
}
fn verified(op: &ActivityOperation, s: &TravelState) -> bool {
    if let Some(record) = op
        .task_code
        .strip_prefix("travel_claim:")
        .and_then(|v| v.parse::<i64>().ok())
    {
        return s.state == "idle"
            && s.daily_limit_reached
            && s.record_id.is_none_or(|id| id == record)
            && business_date(op.submitted_at_ms) == business_date(config::now_ms());
    }
    let location = op
        .task_code
        .strip_prefix("travel_depart:")
        .and_then(|v| v.parse::<i64>().ok());
    let depart_ms = s.depart_at.and_then(|v| {
        if v < 1_000_000_000_000 {
            v.checked_mul(1000)
        } else {
            Some(v)
        }
    });
    matches!(s.state.as_str(), "traveling" | "arrived")
        && s.record_id.is_some()
        && s.location_id == location
        && depart_ms.is_some_and(|v| v >= op.submitted_at_ms - 30_000)
}
fn business_date(ms: i64) -> String {
    use chrono::TimeZone;
    chrono::FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .timestamp_millis_opt(ms)
        .single()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
fn belongs(k: &str, a: &Value, op: &ActivityOperation) -> bool {
    if let Some(record) = op.task_code.strip_prefix("travel_claim:") {
        return k == key(a, "claim", record);
    }
    op.task_code.starts_with("travel_depart:")
        && k == key(a, "depart", &business_date(op.submitted_at_ms))
}
pub(crate) async fn run_locked(
    root: &Path,
    a: &Value,
    claim: bool,
    depart: Option<i64>,
    automatic: bool,
) -> Result<RunReport, String> {
    let prepared = growth::prepare_account(a).await;
    let a = &prepared;
    let mut report = RunReport::default();
    let mut journal = growth_automation::read_journal(root)?;
    let id = a["id"].as_str().ok_or("账号标识缺失")?;
    let s = status(a).await?;
    // Resolve previous unknown outcomes before any new submission.
    let mut changed = false;
    for (_, op) in journal.operations.iter_mut().filter(|(k, o)| {
        o.account_id == id
            && belongs(k, a, o)
            && o.task_code.starts_with("travel_")
            && o.state == OperationState::AwaitingVerification
    }) {
        if verified(op, &s) {
            op.state = OperationState::Verified;
            op.verified_at_ms = Some(config::now_ms());
            op.error = None;
            changed = true;
            report.verified += 1;
        } else {
            report.awaiting_verification += 1;
        }
    }
    if changed {
        growth_automation::save_journal(root, &journal)?;
    }
    if report.awaiting_verification > 0 {
        return Ok(report);
    }
    let (action, period, code, body) = if claim && s.state == "arrived" {
        let record = s.record_id.ok_or("旅行记录缺失")?;
        (
            "claim",
            record.to_string(),
            format!("travel_claim:{record}"),
            json!({"record_id":record}),
        )
    } else if s.state == "idle" && !s.daily_limit_reached && depart.is_some() {
        let location = depart.unwrap();
        let (enabled, choices) = locations(a).await?;
        if !enabled || !choices.iter().any(|l| l.id == location) {
            report.skipped += 1;
            return Ok(report);
        }
        let date = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
            .format("%Y-%m-%d")
            .to_string();
        (
            "depart",
            date,
            format!("travel_depart:{location}"),
            json!({"location_id":location}),
        )
    } else {
        report.skipped += 1;
        if claim && s.state == "traveling" {
            report.waiting += 1;
        }
        return Ok(report);
    };
    let k = key(a, action, &period);
    if journal.operations.contains_key(&k) {
        report.already_claimed += 1;
        return Ok(report);
    }
    if journal.operations.len() >= 4096 {
        return Err("活动操作记录超限".into());
    }
    if automatic && !super::growth_schedule::allows_travel(id, action, depart)? {
        return Ok(report);
    }
    journal.operations.insert(
        k.clone(),
        ActivityOperation {
            operation_id: uuid::Uuid::new_v4().to_string(),
            account_id: id.into(),
            task_code: code,
            state: OperationState::AwaitingVerification,
            submitted_at_ms: config::now_ms(),
            verified_at_ms: None,
            error: None,
        },
    );
    growth_automation::save_journal(root, &journal)?;
    let submitted = request(a, action, Some(body)).await;
    let after = status(a).await;
    let op = journal.operations.get_mut(&k).unwrap();
    if after.as_ref().is_ok_and(|s| verified(op, s)) {
        op.state = OperationState::Verified;
        op.verified_at_ms = Some(config::now_ms());
        report.verified += 1;
    } else {
        op.error = Some(
            submitted
                .err()
                .or_else(|| after.err())
                .unwrap_or_else(|| "旅行结果尚未核实，禁止重复提交".into()),
        );
        report.awaiting_verification += 1;
    }
    growth_automation::save_journal(root, &journal)?;
    if report.verified > 0 {
        let grant=CredentialStore::new(root).save(id,&growth::grant_scope(a),CapabilityType::GrowthTravel,&json!({"authorized":true,"verified_for":format!("https://copilot.tencent.com{PREFIX}/{action}"),"verified_at_ms":config::now_ms()}),a["expiresAt"].as_i64()).map_err(|_|"旅行用途验证记录保存失败")?;
        super::credential_store::durable_write(
            &root.join(format!(
                "growth-travel-grant-{}.json",
                key(a, "grant", action)
            )),
            &serde_json::to_string(&grant).unwrap(),
        )
        .map_err(|_| "旅行用途记录保存失败")?;
    }
    Ok(report)
}
pub fn run(
    account_id: &str,
    claim: bool,
    depart: Option<i64>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<RunReport, String>> + Send + '_>> {
    Box::pin(run_inner(account_id, claim, depart))
}
async fn run_inner(
    account_id: &str,
    claim: bool,
    depart: Option<i64>,
) -> Result<RunReport, String> {
    let root = config::store_dir();
    std::fs::create_dir_all(root.join("locks")).map_err(|_| "活动锁目录不可创建")?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("locks/growth.lock"))
        .map_err(|_| "活动锁不可读取")?;
    lock.try_lock_exclusive().map_err(|_| "成长任务正在处理")?;
    let a = account::find_account(account_id).ok_or("账号不存在")?;
    run_locked(&root, &a, claim, depart, false).await
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_and_incomplete_states_do_not_authorize_writes() {
        for v in [
            json!({}),
            json!({"state":"new","daily_limit_reached":false}),
            json!({"state":"arrived","daily_limit_reached":true}),
        ] {
            assert!(parse_state("a", &v).is_err());
        }
    }
    #[test]
    fn departure_requires_matching_location_and_new_server_timestamp() {
        let op = ActivityOperation {
            operation_id: "o".into(),
            account_id: "a".into(),
            task_code: "travel_depart:2".into(),
            state: OperationState::AwaitingVerification,
            submitted_at_ms: 1_800_000_000_000,
            verified_at_ms: None,
            error: None,
        };
        let mut s=parse_state("a",&json!({"state":"traveling","daily_limit_reached":true,"record_id":123,"location":{"id":2},"depart_at":1_800_000_000})).unwrap();
        assert!(verified(&op, &s));
        s.location_id = Some(1);
        assert!(!verified(&op, &s));
        s.location_id = Some(2);
        s.depart_at = Some(1_700_000_000);
        assert!(!verified(&op, &s));
    }
    #[test]
    fn pending_operation_does_not_follow_reused_account_id() {
        let old = json!({"id":"a","uid":"old"});
        let new = json!({"id":"a","uid":"new"});
        let op = ActivityOperation {
            operation_id: "o".into(),
            account_id: "a".into(),
            task_code: "travel_claim:123".into(),
            state: OperationState::AwaitingVerification,
            submitted_at_ms: config::now_ms(),
            verified_at_ms: None,
            error: None,
        };
        let k = key(&old, "claim", "123");
        assert!(belongs(&k, &old, &op));
        assert!(!belongs(&k, &new, &op));
        let different = parse_state(
            "a",
            &json!({"state":"idle","daily_limit_reached":true,"record_id":999}),
        )
        .unwrap();
        assert!(!verified(&op, &different));
    }
}
