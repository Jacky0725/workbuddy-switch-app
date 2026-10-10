//! Current-user encrypted account backups. Official client/session files are never touched.
use crate::modules::{account_repository::AccountRepository, config, credential_store};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

const MAX_BYTES: u64 = 32 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    version: u32,
    created_at_ms: i64,
    accounts: Vec<Value>,
}
#[derive(Serialize)]
pub struct BackupInfo {
    pub id: String,
    pub created_at_ms: Option<i64>,
    pub account_count: Option<usize>,
    pub error: Option<String>,
}
#[derive(Serialize)]
pub struct RestorePreview {
    pub id: String,
    pub account_count: usize,
    pub current_revision: String,
    pub scope: &'static str,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreRequest {
    pub id: String,
    pub expected_revision: String,
    pub confirmed: bool,
}
#[derive(Serialize)]
pub struct RestoreReport {
    pub restored_accounts: usize,
    pub rollback_id: String,
}
fn error() -> String {
    "备份损坏、无法解密或版本不兼容，已停止操作并保留原文件".into()
}
fn valid_id(id: &str) -> Result<(), String> {
    let uuid = uuid::Uuid::parse_str(id).map_err(|_| "备份标识无效")?;
    if uuid.to_string() != id {
        return Err("备份标识无效".into());
    }
    Ok(())
}
fn path(root: &Path, id: &str) -> Result<PathBuf, String> {
    valid_id(id)?;
    Ok(root.join("backups").join(format!("{id}.wbm")))
}
fn directory_is_safe(directory: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(directory).map_err(|_| error())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(error());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(error());
        }
    }
    Ok(())
}
fn read_payload(root: &Path, id: &str) -> Result<Payload, String> {
    let path = path(root, id)?;
    directory_is_safe(root)?;
    directory_is_safe(&root.join("backups"))?;
    let meta = std::fs::symlink_metadata(&path).map_err(|_| error())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > MAX_BYTES {
        return Err(error());
    }
    let mut encrypted = std::fs::read(path).map_err(|_| error())?;
    let mut bytes = credential_store::open_backup(id, &encrypted).map_err(|_| error())?;
    encrypted.fill(0);
    if bytes.len() as u64 > MAX_BYTES {
        bytes.fill(0);
        return Err(error());
    }
    let payload = serde_json::from_slice::<Payload>(&bytes).map_err(|_| error());
    bytes.fill(0);
    let payload = payload?;
    if payload.version != 1
        || payload.accounts.len() > 10_000
        || payload
            .accounts
            .iter()
            .any(|account| !account.is_object() || account["credential_locked"] == true)
    {
        return Err(error());
    }
    Ok(payload)
}
fn lock(root: &Path) -> Result<std::fs::File, String> {
    std::fs::create_dir_all(root).map_err(|_| "备份目录不可写")?;
    directory_is_safe(root)?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("accounts.lock"))
        .map_err(|_| "账号库锁不可用")?;
    file.try_lock_exclusive()
        .map_err(|_| "账号库正由另一任务更新，请稍后重试")?;
    Ok(file)
}
pub fn revision(root: &Path) -> Result<String, String> {
    let bytes = match std::fs::read(root.join("accounts.json")) {
        Ok(value) => value,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => vec![],
        Err(_) => return Err("当前账号索引不可读".into()),
    };
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn write_payload(root: &Path, accounts: Vec<Value>) -> Result<BackupInfo, String> {
    if accounts.len() > 10_000 {
        return Err("账号数量超过备份限制".into());
    }
    if accounts
        .iter()
        .any(|account| account["credential_locked"] == true)
    {
        return Err("部分账号凭证不可读取，备份已取消，请重新授权".into());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let created = config::now_ms();
    let count = accounts.len();
    let mut bytes = serde_json::to_vec(&Payload {
        version: 1,
        created_at_ms: created,
        accounts,
    })
    .map_err(|_| error())?;
    if bytes.len() as u64 > MAX_BYTES {
        bytes.fill(0);
        return Err("账号备份超过大小限制".into());
    }
    let encrypted = credential_store::seal_backup(&id, &bytes).map_err(|_| error());
    bytes.fill(0);
    let encrypted = encrypted?;
    let directory = root.join("backups");
    crate::modules::manager_security::private_directory(&directory)
        .map_err(|_| "备份目录不可写或权限无法保护")?;
    let target = path(root, &id)?;
    let temporary = directory.join(format!("{id}.tmp"));
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(&encrypted)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, &target)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
        return Err("备份写入失败，本次操作已取消".into());
    }
    let checked = read_payload(root, &id)?;
    if checked.accounts.len() != count {
        return Err(error());
    }
    Ok(BackupInfo {
        id,
        created_at_ms: Some(created),
        account_count: Some(count),
        error: None,
    })
}
pub fn create_at(root: &Path) -> Result<BackupInfo, String> {
    let _lock = lock(root)?;
    let accounts = AccountRepository::new(&root.join("accounts.json"))
        .load()
        .map_err(|_| "账号库不可读，备份已取消")?;
    let result = write_payload(root, accounts);
    if result.is_ok() {
        let _ = crate::modules::notifications::record_at(
            root,
            "success",
            "账号加密备份已创建并核实",
            None,
        );
    }
    result
}
pub fn list_at(root: &Path) -> Result<Vec<BackupInfo>, String> {
    let directory = root.join("backups");
    if !directory.exists() {
        return Ok(vec![]);
    }
    directory_is_safe(root)?;
    directory_is_safe(&directory)?;
    let mut result = vec![];
    for entry in std::fs::read_dir(directory).map_err(|_| "备份目录不可读")? {
        let entry = entry.map_err(|_| "备份目录不可读")?;
        if entry.path().extension().and_then(|v| v.to_str()) != Some("wbm") {
            continue;
        }
        let Some(id) = entry
            .path()
            .file_stem()
            .and_then(|v| v.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if valid_id(&id).is_err() {
            continue;
        }
        let info = match read_payload(root, &id) {
            Ok(value) => BackupInfo {
                id,
                created_at_ms: Some(value.created_at_ms),
                account_count: Some(value.accounts.len()),
                error: None,
            },
            Err(reason) => BackupInfo {
                id,
                created_at_ms: None,
                account_count: None,
                error: Some(reason),
            },
        };
        result.push(info);
        if result.len() > 1_000 {
            return Err("备份数量过多，请先在备份目录归档".into());
        }
    }
    result.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
    Ok(result)
}
pub fn preview_at(root: &Path, id: &str) -> Result<RestorePreview, String> {
    let payload = read_payload(root, id)?;
    Ok(RestorePreview {
        id: id.into(),
        account_count: payload.accounts.len(),
        current_revision: revision(root)?,
        scope: "仅替换 Manager 账号库；不切换官方登录、不修改会话或计划",
    })
}
pub fn restore_at(root: &Path, request: RestoreRequest) -> Result<RestoreReport, String> {
    if !request.confirmed {
        return Err("恢复需要用户确认".into());
    }
    let payload = read_payload(root, &request.id)?;
    let _lock = lock(root)?;
    if revision(root)? != request.expected_revision {
        return Err("账号库已变化，请重新预览后确认恢复".into());
    }
    let current = AccountRepository::new(&root.join("accounts.json"))
        .load()
        .map_err(|_| "当前账号库损坏，自动恢复已停止；请保留原文件")?;
    let rollback = write_payload(root, current)?;
    // Prepare complete immutable vault and verify it before the single index commit.
    let staging = root.join(format!("restore-stage-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&staging).map_err(|_| "恢复准备目录不可写")?;
    let result = (|| -> Result<RestoreReport, String> {
        let repo = AccountRepository::new(&staging.join("accounts.json"));
        repo.save(&payload.accounts).map_err(|_| error())?;
        let loaded = repo.load().map_err(|_| error())?;
        if loaded.len() != payload.accounts.len()
            || loaded.iter().any(|a| a["credential_locked"] == true)
        {
            return Err(error());
        }
        crate::modules::manager_security::private_directory(&root.join("credentials"))
            .map_err(|_| "凭证目录不可写或权限无法保护")?;
        if staging.join("credentials").exists() {
            for entry in std::fs::read_dir(staging.join("credentials")).map_err(|_| error())? {
                let entry = entry.map_err(|_| error())?;
                let bytes = std::fs::read(entry.path()).map_err(|_| error())?;
                let target = root.join("credentials").join(entry.file_name());
                let mut file = std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(target)
                    .map_err(|_| "恢复凭证写入失败，账号索引未替换")?;
                file.write_all(&bytes)
                    .and_then(|_| file.sync_all())
                    .map_err(|_| "恢复凭证写入失败，账号索引未替换")?;
            }
        }
        let index = std::fs::read_to_string(staging.join("accounts.json")).map_err(|_| error())?;
        credential_store::durable_write(&root.join("accounts.json"), &index)
            .map_err(|_| "恢复提交失败，原账号索引已保留")?;
        Ok(RestoreReport {
            restored_accounts: payload.accounts.len(),
            rollback_id: rollback.id,
        })
    })();
    // Only this randomly named staging directory is removed, never any user-selected path.
    let _ = std::fs::remove_dir_all(staging);
    let _ = crate::modules::notifications::record_at(
        root,
        if result.is_ok() { "success" } else { "error" },
        if result.is_ok() {
            "Manager 账号库恢复完成"
        } else {
            "Manager 账号库恢复未完成"
        },
        result.as_ref().err().map(String::as_str),
    );
    result
}
