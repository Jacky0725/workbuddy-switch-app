//! Fixed-origin, read-only growth queries. No claim or activity-event submissions.
use super::{
    account, config,
    credential_store::{durable_write, CapabilityType, CredentialReference, CredentialStore},
    variant::WbVariant,
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path, time::Duration};

const TASKS_URL: &str = "https://copilot.tencent.com/v2/activity/growth/tasks";
const MAX_RESPONSE: usize = 2 * 1024 * 1024;
const MAX_CACHE: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrowthTask {
    pub code: String,
    pub title: String,
    pub accept_status: String,
    #[serde(default)]
    pub task_type: String,
    #[serde(default)]
    pub validity_known: bool,
    pub locked: Option<bool>,
    pub current: Option<u64>,
    pub target: Option<u64>,
    pub reward_credit: Option<u64>,
    pub reward_energy: Option<u64>,
    pub valid_start_ms: Option<i64>,
    pub valid_end_ms: Option<i64>,
    /// Progress is advisory until a dedicated claim adapter is verified.
    pub progress_complete: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrowthAccount {
    pub account_id: String,
    pub observed_at_ms: Option<i64>,
    pub stale: bool,
    pub error: Option<String>,
    pub tasks: Vec<GrowthTask>,
    identity_digest: String,
}
#[derive(Default, Serialize, Deserialize)]
struct Cache {
    version: u32,
    accounts: BTreeMap<String, GrowthAccount>,
    /// New purposes stay outside the legacy account index so alpha.5 can still read it.
    #[serde(default)]
    read_grants: BTreeMap<String, CredentialReference>,
}
#[derive(Serialize)]
pub struct GrowthSnapshot {
    pub accounts: Vec<GrowthAccount>,
    pub automatic_execution_enabled: bool,
    pub execution_status: String,
}

fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
pub(crate) fn identity(a: &Value) -> String {
    digest(
        &json!([
            a["uid"],
            a["domain"],
            a["provider"],
            super::account_repository::scope_of(a),
            a["enterpriseId"],
            a["enterprise_id"]
        ])
        .to_string(),
    )
}
pub(crate) fn grant_scope(a: &Value) -> Value {
    let mut scope = super::account_repository::scope_of(a);
    scope["external_uid"] = json!(account::get_str(a, "uid"));
    scope
}
fn grant_is_current(root: &Path, a: &Value, reference: Option<&CredentialReference>) -> bool {
    let Some(reference) = reference.filter(|r| r.capability_type == CapabilityType::GrowthRead)
    else {
        return false;
    };
    let Some(id) = account::get_str(a, "id") else {
        return false;
    };
    CredentialStore::new(root)
        .load(reference, &id, &grant_scope(a))
        .is_ok_and(|g| {
            g["authorized"] == true
                && g["verified_for"] == TASKS_URL
                && g["credential_digest"] == digest(a["access_token"].as_str().unwrap_or(""))
        })
}
fn cache_at(root: &Path) -> Result<Cache, String> {
    let path = root.join("growth_cache.json");
    if !path.exists() {
        return Ok(Cache {
            version: 1,
            ..Default::default()
        });
    }
    if std::fs::metadata(&path)
        .map_err(|_| "成长任务缓存不可读取")?
        .len()
        > MAX_CACHE
    {
        return Err("成长任务缓存超限，请保留原文件".into());
    }
    let cache: Cache =
        serde_json::from_slice(&std::fs::read(path).map_err(|_| "成长任务缓存不可读取")?)
            .map_err(|_| "成长任务缓存损坏，请保留原文件")?;
    if cache.version != 1 {
        return Err("成长任务缓存版本尚不支持".into());
    }
    Ok(cache)
}
pub(crate) fn query_error(a: &Value) -> Option<&'static str> {
    if account::variant_of(a) != WbVariant::Cn
        || super::account_repository::provider_of(a) != "workbuddy"
    {
        return Some("该区域或账号提供方不支持成长任务查询");
    }
    if a["is_enabled"] == false {
        return Some("账号已停用");
    }
    if a["credential_locked"] == true
        || a["needs_relogin"] == true
        || account::get_str(a, "access_token").is_none()
    {
        return Some("凭证不可用，请重新登录");
    }
    if a["expiresAt"]
        .as_i64()
        .is_some_and(|v| v <= config::now_ms())
    {
        return Some("凭证已过期，请刷新账号");
    }
    if account::get_str(a, "uid").is_none() {
        return Some("账号身份缺失");
    }
    None
}
pub(crate) async fn prepare_account(a: &Value) -> Value {
    if account::variant_of(a) == WbVariant::Cn
        && super::account_repository::provider_of(a) == "workbuddy"
        && a["is_enabled"] != false
        && a["credential_locked"] != true
        && a["needs_relogin"] != true
    {
        super::refresh::ensure_fresh_token(a.clone(), &json!({"lazy_refresh_hours":1})).await
    } else {
        a.clone()
    }
}
pub fn snapshot() -> Result<GrowthSnapshot, String> {
    snapshot_at(
        &config::store_dir(),
        &account::try_load_accounts().map_err(|_| "账号库不可读取")?,
    )
}
fn snapshot_at(root: &Path, accounts: &[Value]) -> Result<GrowthSnapshot, String> {
    let cache = cache_at(root)?;
    let now = config::now_ms();
    let mut rows = Vec::new();
    for a in accounts {
        let Some(id) = account::get_str(a, "id") else {
            continue;
        };
        let mut row = cache
            .accounts
            .get(&id)
            .filter(|v| v.identity_digest == identity(a))
            .cloned()
            .unwrap_or(GrowthAccount {
                account_id: id,
                observed_at_ms: None,
                stale: true,
                error: None,
                tasks: vec![],
                identity_digest: identity(a),
            });
        row.stale = row.stale
            || row
                .observed_at_ms
                .is_none_or(|t| now.saturating_sub(t) > 15 * 60 * 1000 || t > now);
        if !grant_is_current(root, a, cache.read_grants.get(&row.account_id)) {
            row.stale = true;
        }
        if let Some(error) = query_error(a) {
            row.error = Some(error.into());
            row.stale = true;
        }
        rows.push(row);
    }
    let schedule = super::growth_schedule::snapshot_at(root);
    let enabled = schedule.as_ref().is_ok_and(|s| s.config.enabled);
    Ok(GrowthSnapshot {
        accounts: rows,
        automatic_execution_enabled: enabled,
        execution_status: if schedule.is_err() {
            "成长计划不可读取，自动执行已暂停"
        } else if enabled {
            "成长计划已启用；普通领奖与旅行分别遵循所选开关"
        } else {
            "成长计划未启用；可手动查询、领取已完成奖励及处理旅行"
        }
        .into(),
    })
}
fn timestamp(value: &Value) -> Option<i64> {
    if let Some(n) = value.as_i64().filter(|n| *n > 0) {
        return if n < 1_000_000_000_000 {
            n.checked_mul(1000)
        } else {
            Some(n)
        };
    }
    let s = value.as_str()?;
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|v| v.timestamp_millis())
}
pub(crate) fn parse_tasks(data: &Value) -> Result<Vec<GrowthTask>, String> {
    let items = data
        .get("tasks")
        .and_then(Value::as_array)
        .ok_or("成长任务响应结构未识别")?;
    if items.len() > 256 {
        return Err("成长任务数量超限".into());
    }
    let mut codes = std::collections::HashSet::new();
    items
        .iter()
        .map(|t| {
            let code = t["task_code"]
                .as_str()
                .filter(|s| {
                    !s.is_empty()
                        && s.len() <= 128
                        && s.bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                })
                .ok_or("成长任务标识不合法")?
                .to_string();
            if !codes.insert(code.clone()) {
                return Err("成长任务标识重复，已停止更新".into());
            }
            let progress = t.get("progress").filter(|v| v.is_object()).unwrap_or(t);
            let current = progress["current"].as_u64();
            let target = progress["target"].as_u64();
            let status = t["accept_status"].as_str().unwrap_or("unknown");
            Ok(GrowthTask {
                code,
                title: t["title"]
                    .as_str()
                    .unwrap_or("未命名任务")
                    .chars()
                    .take(256)
                    .collect(),
                accept_status: status.chars().take(64).collect(),
                task_type: t["task_type"]
                    .as_str()
                    .unwrap_or("unknown")
                    .chars()
                    .take(32)
                    .collect(),
                validity_known: ["valid_start", "valid_end"].iter().all(|key| {
                    t.get(*key).is_some_and(|v| {
                        v.is_null() || v.as_str() == Some("") || timestamp(v).is_some()
                    })
                }),
                locked: t["locked"].as_bool(),
                current,
                target,
                reward_credit: t["reward_credit"].as_u64(),
                reward_energy: t["reward_energy"].as_u64(),
                valid_start_ms: timestamp(&t["valid_start"]),
                valid_end_ms: timestamp(&t["valid_end"]),
                progress_complete: matches!((current,target),(Some(c),Some(t)) if t>0 && c>=t),
            })
        })
        .collect()
}
pub(crate) async fn query(a: &Value) -> Result<Vec<GrowthTask>, String> {
    if let Some(error) = query_error(a) {
        return Err(error.into());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "成长查询网络初始化失败")?;
    let mut request = client
        .get(TASKS_URL)
        .header("Origin", "https://www.workbuddy.cn")
        .header("Referer", "https://www.workbuddy.cn/profile/growth-center")
        .header("X-Client-Platform", "web");
    for (key, value) in account::build_auth_headers(a) {
        request = request.header(key, value)
    }
    let mut response = request
        .send()
        .await
        .map_err(|_| "成长查询网络失败，请稍后重试")?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(match status {
            401 => "成长查询凭证失效",
            403 => "成长查询被拒绝，需在官方客户端检查账号资格",
            429 => "成长查询请求受限，请稍后重试",
            _ => "成长查询服务暂不可用",
        }
        .into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "成长查询响应读取失败")? {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE {
            return Err("成长查询响应超限".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let body: Value = serde_json::from_slice(&bytes).map_err(|_| "成长查询响应不是有效 JSON")?;
    if body["code"].as_i64() != Some(0) {
        return Err("成长查询业务状态未通过，请在官方客户端检查".into());
    }
    parse_tasks(&body["data"])
}
/// Explicit refresh queries official state and may renew expiring credentials through
/// the established refresh path. Cached snapshots never issue HTTP.
// Erase the large service future at the host boundary; Tauri's generated IPC
// handlers otherwise instantiate its entire nested state machine again.
pub fn refresh(
    account_id: Option<&str>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<GrowthSnapshot, String>> + Send + '_>>
{
    Box::pin(refresh_inner(account_id))
}
async fn refresh_inner(account_id: Option<&str>) -> Result<GrowthSnapshot, String> {
    let root = config::store_dir();
    std::fs::create_dir_all(root.join("locks")).map_err(|_| "查询锁目录不可创建")?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("locks/growth.lock"))
        .map_err(|_| "成长查询锁不可读取")?;
    lock.try_lock_exclusive()
        .map_err(|_| "成长任务正在处理，请稍后查看")?;
    let accounts = account::try_load_accounts().map_err(|_| "账号库不可读取")?;
    if account_id.is_some_and(|id| !accounts.iter().any(|a| a["id"] == id)) {
        return Err("账号不存在".into());
    }
    let mut cache = cache_at(&root)?;
    for a in &accounts {
        let Some(id) = account::get_str(a, "id") else {
            continue;
        };
        if account_id.is_some_and(|selected| selected != id) {
            continue;
        }
        let prepared = prepare_account(a).await;
        let a = &prepared;
        let outcome = query(a).await;
        let mut row = cache
            .accounts
            .get(&id)
            .filter(|v| v.identity_digest == identity(a))
            .cloned()
            .unwrap_or(GrowthAccount {
                account_id: id.clone(),
                identity_digest: identity(a),
                tasks: vec![],
                error: None,
                stale: true,
                observed_at_ms: None,
            });
        match outcome {
            Ok(tasks) => {
                let fingerprint = digest(a["access_token"].as_str().unwrap_or(""));
                let current = account::find_account(&id).ok_or("查询期间账号已变化，请重试")?;
                if identity(&current) != identity(a)
                    || digest(current["access_token"].as_str().unwrap_or("")) != fingerprint
                {
                    return Err("成长查询期间账号发生变化，未保存结果".into());
                }
                let grant=CredentialStore::new(&root).save(&id,&grant_scope(a),CapabilityType::GrowthRead,&json!({"authorized":true,"verified_for":TASKS_URL,"verified_at_ms":config::now_ms(),"credential_digest":fingerprint}),a["expiresAt"].as_i64()).map_err(|_|"成长查询验证记录保存失败")?;
                cache.read_grants.insert(id.clone(), grant);
                row.tasks = tasks;
                row.observed_at_ms = Some(config::now_ms());
                row.stale = false;
                row.error = None;
            }
            Err(error) => {
                row.error = Some(error);
                row.stale = true;
            }
        }
        cache.accounts.insert(id, row);
    }
    cache.accounts.retain(|id, row| {
        accounts
            .iter()
            .any(|a| a["id"] == *id && identity(a) == row.identity_digest)
    });
    let text = serde_json::to_string(&cache).map_err(|_| "成长查询缓存不可保存")?;
    if text.len() as u64 > MAX_CACHE {
        return Err("成长任务缓存超限，未覆盖原文件".into());
    }
    durable_write(&root.join("growth_cache.json"), &text).map_err(|_| "成长查询缓存保存失败")?;
    snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_progress_and_unknown_state_are_not_completed() {
        let tasks=parse_tasks(&json!({"tasks":[{"task_code":"chat_5","accept_status":"new_state","title":"fictional"}]})).unwrap();
        assert_eq!(tasks[0].current, None);
        assert!(!tasks[0].progress_complete);
        assert_eq!(tasks[0].accept_status, "new_state");
    }
    #[test]
    fn claimed_and_completed_are_distinct() {
        let tasks=parse_tasks(&json!({"tasks":[{"task_code":"test","accept_status":"accepted","locked":false,"progress":{"current":1,"target":1}}]})).unwrap();
        assert!(tasks[0].progress_complete);
        assert_ne!(tasks[0].accept_status, "claimed");
    }
    #[test]
    fn malformed_or_duplicate_tasks_rejected() {
        for v in [
            json!({}),
            json!({"tasks":[{"task_code":"../bad"}]}),
            json!({"tasks":[{"task_code":"same"},{"task_code":"same"}]}),
        ] {
            assert!(parse_tasks(&v).is_err());
        }
    }
    #[test]
    fn cache_does_not_follow_reused_account_id() {
        let root = std::env::temp_dir().join(format!("growth-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let old = json!({"id":"a","uid":"old","variant":"cn"});
        let row = GrowthAccount {
            account_id: "a".into(),
            identity_digest: identity(&old),
            observed_at_ms: Some(config::now_ms()),
            stale: false,
            error: None,
            tasks: vec![parse_tasks(&json!({"tasks":[{"task_code":"old"}]}))
                .unwrap()
                .remove(0)],
        };
        let cache = Cache {
            version: 1,
            accounts: BTreeMap::from([("a".into(), row)]),
            read_grants: BTreeMap::new(),
        };
        std::fs::write(
            root.join("growth_cache.json"),
            serde_json::to_vec(&cache).unwrap(),
        )
        .unwrap();
        let snapshot = snapshot_at(&root, &[json!({"id":"a","uid":"new","variant":"cn"})]).unwrap();
        assert!(snapshot.accounts[0].tasks.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn canonical_region_changes_invalidate_growth_identity() {
        let cn = json!({"id":"a","uid":"fictional","variant":"cn"});
        let global = json!({"id":"a","uid":"fictional","variant":"ai"});
        assert_ne!(identity(&cn), identity(&global));
    }
}
