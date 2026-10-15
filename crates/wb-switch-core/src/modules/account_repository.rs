//! Account index contains only allowlisted scalar metadata and opaque references.
//! Immutable encrypted blobs are prepared before the atomic index replacement.

use crate::modules::{
    account, config,
    credential_store::{self, CapabilityType, CredentialReference, CredentialStore},
};
use fs2::FileExt;
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

const PUBLIC_FIELDS: &[&str] = &[
    "id",
    "uid",
    "email",
    "nickname",
    "enterpriseName",
    "note",
    "displayField",
    "expiresAt",
    "refreshExpiresAt",
    "refreshedAt",
    "createdAt",
    "updatedAt",
    "needs_relogin",
    "is_enabled",
];

pub fn provider_of(account: &Value) -> String {
    account::get_str(account, "provider")
        .unwrap_or_else(|| "workbuddy".to_string())
        .to_ascii_lowercase()
}

pub fn scope_of(account: &Value) -> Value {
    json!({"provider": provider_of(account), "region": account::variant_of(account).as_str()})
}

pub fn same_scope(a: &Value, b: &Value) -> bool {
    scope_of(a) == scope_of(b)
}

fn credential_scope(account: &Value) -> Value {
    let mut scope = scope_of(account);
    scope["external_uid"] = json!(account::get_str(account, "uid"));
    scope
}

fn invalid_index() -> io::Error {
    io::Error::new(
        ErrorKind::InvalidData,
        "账号索引损坏或版本不兼容，已停止写入；请从备份恢复",
    )
}

pub struct AccountRepository {
    path: PathBuf,
}

impl AccountRepository {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
    fn root(&self) -> &Path {
        self.path.parent().unwrap_or_else(|| Path::new("."))
    }

    fn lock(&self) -> io::Result<std::fs::File> {
        std::fs::create_dir_all(self.root())?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.root().join("accounts.lock"))?;
        file.try_lock_exclusive().map_err(|_| {
            io::Error::new(ErrorKind::WouldBlock, "账号库正由另一任务更新，请稍后重试")
        })?;
        Ok(file)
    }

    pub fn update<T>(
        &self,
        mutate: impl FnOnce(&mut Vec<Value>) -> io::Result<T>,
    ) -> io::Result<T> {
        let _lock = self.lock()?;
        let mut accounts = self.load()?;
        let result = mutate(&mut accounts)?;
        self.save_locked(&accounts)?;
        Ok(result)
    }

    fn read_index(&self) -> io::Result<Vec<Value>> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(vec![]),
            Err(_) => return Err(invalid_index()),
        };
        let index: Vec<Value> = serde_json::from_str(&text).map_err(|_| invalid_index())?;
        if index
            .iter()
            .any(|v| !v.is_object() || v.get("storage_version").is_some_and(|v| v != 1))
        {
            return Err(invalid_index());
        }
        let mut ids = HashSet::new();
        for entry in &index {
            if entry.get("storage_version").is_some() {
                let id = account::get_str(entry, "id").ok_or_else(invalid_index)?;
                if !ids.insert(id)
                    || entry.as_object().unwrap().keys().any(|key| {
                        !PUBLIC_FIELDS.contains(&key.as_str())
                            && ![
                                "provider",
                                "region",
                                "variant",
                                "storage_version",
                                "credential_references",
                            ]
                            .contains(&key.as_str())
                    })
                {
                    return Err(invalid_index());
                }
            }
        }
        Ok(index)
    }

    pub fn load(&self) -> io::Result<Vec<Value>> {
        let store = CredentialStore::new(self.root());
        self.read_index()?
            .into_iter()
            .map(|mut entry| {
                if entry.get("storage_version").is_none() {
                    return Ok(entry);
                }
                let id = account::get_str(&entry, "id").ok_or_else(invalid_index)?;
                let scope = credential_scope(&entry);
                let references: Vec<CredentialReference> =
                    serde_json::from_value(entry["credential_references"].clone())
                        .map_err(|_| invalid_index())?;
                let mut credentials = Map::new();
                let mut locked = false;
                let mut purposes = HashSet::new();
                for reference in references {
                    if !purposes.insert(reference.capability_type.as_str()) {
                        return Err(invalid_index());
                    }
                    match store.load(&reference, &id, &scope) {
                        Ok(fields)
                            if reference.capability_type == CapabilityType::DesktopSwitch =>
                        {
                            for (key, value) in fields.as_object().ok_or_else(invalid_index)? {
                                // Ciphertext cannot override identity, display metadata or references.
                                if [
                                    "id",
                                    "uid",
                                    "provider",
                                    "region",
                                    "variant",
                                    "storage_version",
                                    "credential_references",
                                    "capability_credentials",
                                ]
                                .contains(&key.as_str())
                                {
                                    return Err(invalid_index());
                                }
                                entry[key] = value.clone();
                            }
                        }
                        Ok(fields) => {
                            credentials
                                .insert(reference.capability_type.as_str().to_string(), fields);
                        }
                        Err(_) => {
                            locked = true;
                        }
                    }
                }
                if !purposes.contains("DesktopSwitch") {
                    return Err(invalid_index());
                }
                entry["capability_credentials"] = Value::Object(credentials);
                if locked {
                    // Do not expose partially recovered credentials for a locked account.
                    for key in [
                        "access_token",
                        "refresh_token",
                        "auth_raw",
                        "profile_raw",
                        "capability_credentials",
                    ] {
                        entry.as_object_mut().unwrap().remove(key);
                    }
                    entry["needs_relogin"] = json!(true);
                    entry["credential_locked"] = json!(true);
                    entry["needs_relogin_reason"] = json!("凭证不可读取，请重新授权");
                }
                Ok(entry)
            })
            .collect()
    }

    pub fn save(&self, accounts: &[Value]) -> io::Result<()> {
        let _lock = self.lock()?;
        self.save_locked(accounts)
    }

    fn save_locked(&self, accounts: &[Value]) -> io::Result<()> {
        // Fail closed on a damaged index or unreadable existing credentials.
        let old_index = self.read_index()?;
        let mut ids = HashSet::new();
        for entry in accounts {
            let id = account::get_str(entry, "id").ok_or_else(invalid_index)?;
            if !entry.is_object() || !ids.insert(id) {
                return Err(invalid_index());
            }
        }
        std::fs::create_dir_all(self.root())?;
        if !old_index.is_empty() && old_index.iter().any(|v| v.get("storage_version").is_none()) {
            let mut raw = std::fs::read(&self.path)?;
            let result = credential_store::backup_legacy(self.root(), &raw);
            raw.fill(0);
            result?;
        }
        let store = CredentialStore::new(self.root());
        let mut index = vec![];
        for entry in accounts {
            let id = account::get_str(entry, "id").ok_or_else(invalid_index)?;
            let scope = credential_scope(entry);
            if entry["credential_locked"] == true {
                let old = old_index
                    .iter()
                    .find(|v| v["id"] == id && same_scope(v, entry))
                    .ok_or_else(invalid_index)?;
                if old.get("storage_version").is_none() {
                    return Err(invalid_index());
                }
                // Display-only updates preserve unreadable references; deletion is explicit.
                let mut preserved = old.clone();
                for key in ["note", "displayField", "is_enabled"] {
                    if let Some(value) = entry.get(key) {
                        preserved[key] = value.clone();
                    }
                }
                index.push(preserved);
                continue;
            }
            let mut metadata = Map::new();
            let mut secrets = Map::new();
            for (key, value) in entry.as_object().ok_or_else(invalid_index)? {
                if PUBLIC_FIELDS.contains(&key.as_str()) && !value.is_object() && !value.is_array()
                {
                    metadata.insert(key.clone(), value.clone());
                } else if ![
                    "provider",
                    "region",
                    "variant",
                    "storage_version",
                    "credential_references",
                    "credential_locked",
                    "capability_credentials",
                ]
                .contains(&key.as_str())
                {
                    secrets.insert(key.clone(), value.clone());
                }
            }
            metadata.insert("provider".into(), scope["provider"].clone());
            metadata.insert("region".into(), scope["region"].clone());
            metadata.insert("variant".into(), scope["region"].clone());
            metadata.insert("storage_version".into(), json!(1));
            metadata
                .entry("createdAt")
                .or_insert_with(|| json!(config::now_ms()));
            metadata.insert("updatedAt".into(), json!(config::now_ms()));
            let mut references = vec![store.save(
                &id,
                &scope,
                CapabilityType::DesktopSwitch,
                &Value::Object(secrets),
                entry["expiresAt"].as_i64(),
            )?];
            for capability in [CapabilityType::Checkin, CapabilityType::PointsRead] {
                if let Some(fields) = entry
                    .get("capability_credentials")
                    .and_then(|v| v.get(capability.as_str()))
                {
                    if !fields.is_object() {
                        return Err(invalid_index());
                    }
                    references.push(store.save(
                        &id,
                        &scope,
                        capability,
                        fields,
                        fields["expiresAt"].as_i64(),
                    )?);
                }
            }
            metadata.insert("credential_references".into(), json!(references));
            index.push(Value::Object(metadata));
        }
        let content = serde_json::to_string_pretty(&index).map_err(|_| invalid_index())?;
        credential_store::durable_write(&self.path, &content)
    }
}

/// Public capability summary is independent from whether a token is syntactically present.
/// Purpose-specific grants are reported only after their dedicated adapter verifies scope.
pub fn capabilities(account: &Value) -> Value {
    let status = if account["credential_locked"] == true {
        "CredentialUnavailable"
    } else if account["needs_relogin"] == true
        || account::secret_value(account, "access_token").is_none()
    {
        "NeedsAuthorization"
    } else if account["expiresAt"]
        .as_i64()
        .is_some_and(|v| v <= config::now_ms())
    {
        "Expired"
    } else {
        "AvailableForDesktop"
    };
    let points_grant = &account["capability_credentials"]["PointsRead"];
    let points_status = if points_grant["authorized"] == true
        && points_grant["verified_for"] == "www.codebuddy.cn/billing/meter"
        && status == "AvailableForDesktop"
    {
        "Verified"
    } else {
        "Unverified"
    };
    json!([
        {"capability_type": "DesktopSwitch", "authorization_status": status, "expires_at": account["expiresAt"], "last_verified_at": null},
        {"capability_type": "Checkin", "authorization_status": "Unverified", "expires_at": null, "last_verified_at": null},
        {"capability_type": "PointsRead", "authorization_status": points_status, "expires_at": account["expiresAt"], "last_verified_at": points_grant["verified_at_ms"]},
    ])
}

/// Legacy global API entrypoints remain closed. PointsRead uses a separate
/// per-account grant in credits.rs; presence of a token alone is not proof.
pub fn api_capability_error(capability: CapabilityType) -> Option<&'static str> {
    match capability {
        CapabilityType::DesktopSwitch => None,
        CapabilityType::Checkin => Some("签到授权边界尚未验证，真实接口已禁用"),
        CapabilityType::PointsRead => Some("积分查询授权边界尚未验证，真实接口已禁用"),
        CapabilityType::GrowthRead => Some("请使用限定的成长任务查询入口"),
        CapabilityType::GrowthClaim => Some("请使用限定的成长任务领取入口"),
        CapabilityType::GrowthTravel => Some("请使用限定的成长旅行入口"),
    }
}
