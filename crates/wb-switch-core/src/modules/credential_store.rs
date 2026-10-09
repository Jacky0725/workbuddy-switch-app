//! Current-user DPAPI vault. References never contain a caller-controlled path.
//! No plaintext or machine-scope fallback is permitted.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityType {
    DesktopSwitch,
    Checkin,
    PointsRead,
}

impl CapabilityType {
    pub const ALL: [Self; 3] = [Self::DesktopSwitch, Self::Checkin, Self::PointsRead];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DesktopSwitch => "DesktopSwitch",
            Self::Checkin => "Checkin",
            Self::PointsRead => "PointsRead",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialReference {
    pub account_id: String,
    pub capability_type: CapabilityType,
    pub secure_storage_key: String,
    pub expires_at: Option<i64>,
    pub credential_version: u32,
}

pub fn vault_error() -> io::Error {
    io::Error::new(
        ErrorKind::PermissionDenied,
        "凭证不可读取或校验失败，请重新授权；原始数据未覆盖",
    )
}

pub struct CredentialStore {
    root: PathBuf,
}

impl CredentialStore {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.join("credentials"),
        }
    }

    pub fn save(
        &self,
        account_id: &str,
        scope: &Value,
        capability: CapabilityType,
        fields: &Value,
        expires_at: Option<i64>,
    ) -> io::Result<CredentialReference> {
        let key = uuid::Uuid::new_v4().to_string();
        let context = context(account_id, scope, capability);
        let mut plain = serde_json::to_vec(&json!({
            "account_id": account_id, "scope": scope,
            "capability": capability, "fields": fields,
        }))
        .map_err(|_| vault_error())?;
        let protected = protect(&plain, &context);
        plain.fill(0);
        let protected = protected?;
        std::fs::create_dir_all(&self.root)?;
        write_ciphertext(&self.root.join(format!("{key}.dpapi")), &protected)?;
        Ok(CredentialReference {
            account_id: account_id.to_string(),
            capability_type: capability,
            secure_storage_key: key,
            expires_at,
            credential_version: 1,
        })
    }

    pub fn load(
        &self,
        reference: &CredentialReference,
        account_id: &str,
        scope: &Value,
    ) -> io::Result<Value> {
        let key =
            uuid::Uuid::parse_str(&reference.secure_storage_key).map_err(|_| vault_error())?;
        if reference.credential_version != 1 || reference.account_id != account_id {
            return Err(vault_error());
        }
        let path = self.root.join(format!("{key}.dpapi"));
        let encrypted = read_ciphertext(&path)?;
        let mut plain = unprotect(
            &encrypted,
            &context(account_id, scope, reference.capability_type),
        )?;
        let value: Result<Value, _> = serde_json::from_slice(&plain);
        plain.fill(0);
        let value = value.map_err(|_| vault_error())?;
        if value["account_id"] != account_id
            || value["scope"] != *scope
            || value["capability"] != json!(reference.capability_type)
            || !value["fields"].is_object()
        {
            return Err(vault_error());
        }
        Ok(value["fields"].clone())
    }
}

fn context(account_id: &str, scope: &Value, capability: CapabilityType) -> Vec<u8> {
    // Public entropy binds each blob to its account, region, provider and purpose.
    // This is a namespace, not an encryption key; DPAPI owns the key material.
    serde_json::to_vec(&json!([
        "WorkBuddyManager.credentials.v1",
        account_id,
        scope,
        capability
    ]))
    .unwrap()
}

fn write_ciphertext(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let text = serde_json::to_string(&json!({"version": 1, "ciphertext": STANDARD.encode(bytes)}))
        .map_err(|_| vault_error())?;
    durable_write(path, &text)
}

/// Flush ciphertext/index bytes before rename; temporary files also contain only
/// ciphertext or public metadata. A failed write does not replace the old file.
pub(crate) fn durable_write(path: &Path, text: &str) -> io::Result<()> {
    use std::io::Write;
    let temporary = path.with_file_name(format!("wbm-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn read_ciphertext(path: &Path) -> io::Result<Vec<u8>> {
    let text = std::fs::read_to_string(path).map_err(|_| vault_error())?;
    let value: Value = serde_json::from_str(&text).map_err(|_| vault_error())?;
    if value["version"] != 1 {
        return Err(vault_error());
    }
    STANDARD
        .decode(value["ciphertext"].as_str().ok_or_else(vault_error)?)
        .map_err(|_| vault_error())
}

/// Backup legacy bytes before migration; never create a new plaintext backup.
pub fn backup_legacy(root: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    let encrypted = protect(bytes, b"WorkBuddyManager.accounts-backup.v1")?;
    let dir = root.join("backups");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("accounts-legacy-{}.dpapi", uuid::Uuid::new_v4()));
    write_ciphertext(&path, &encrypted)?;
    // Check recoverability before publishing a migrated account index.
    let mut recovered = read_legacy_backup(&path)?;
    let equal = recovered == bytes;
    recovered.fill(0);
    if !equal {
        return Err(vault_error());
    }
    Ok(path)
}

pub fn read_legacy_backup(path: &Path) -> io::Result<Vec<u8>> {
    unprotect(
        &read_ciphertext(path)?,
        b"WorkBuddyManager.accounts-backup.v1",
    )
}

#[cfg(windows)]
fn dpapi(bytes: &[u8], entropy: &[u8], encrypt: bool) -> io::Result<Vec<u8>> {
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };
    if bytes.len() > u32::MAX as usize || entropy.len() > u32::MAX as usize {
        return Err(vault_error());
    }
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let extra = CRYPT_INTEGER_BLOB {
        cbData: entropy.len() as u32,
        pbData: entropy.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        let result = if encrypt {
            CryptProtectData(
                &input,
                None,
                Some(&extra),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                None,
                Some(&extra),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        // Return a fixed error: native errors must not echo secrets or ciphertext.
        if result.is_err() {
            return Err(vault_error());
        }
        if output.pbData.is_null() {
            return Err(vault_error());
        }
        let buffer = std::slice::from_raw_parts_mut(output.pbData, output.cbData as usize);
        let value = buffer.to_vec();
        buffer.fill(0);
        let _ = LocalFree(HLOCAL(output.pbData as _));
        Ok(value)
    }
}

#[cfg(windows)]
fn protect(bytes: &[u8], entropy: &[u8]) -> io::Result<Vec<u8>> {
    dpapi(bytes, entropy, true)
}
#[cfg(windows)]
fn unprotect(bytes: &[u8], entropy: &[u8]) -> io::Result<Vec<u8>> {
    dpapi(bytes, entropy, false)
}

#[cfg(not(windows))]
fn protect(_: &[u8], _: &[u8]) -> io::Result<Vec<u8>> {
    Err(vault_error())
}
#[cfg(not(windows))]
fn unprotect(_: &[u8], _: &[u8]) -> io::Result<Vec<u8>> {
    Err(vault_error())
}
