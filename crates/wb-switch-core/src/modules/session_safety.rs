//! Exact local-client profile for a limited CN candidate; other session kinds stay closed.
use crate::modules::{process, session::SessionPaths, session_link, variant::WbVariant};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub const UNVERIFIED_CLIENT: &str =
    "尚未验证 WorkBuddy 客户端版本与数据库兼容性，已禁用会话复制、同步和恢复写入";

/// Broad session migration has no verified profile. Never enable it from a
/// frontend flag, environment variable, or the presence of a table.
pub fn require_verified_client() -> Result<(), String> {
    Err(UNVERIFIED_CLIENT.into())
}

// This is a deliberately narrow, local-only compatibility profile. Any client
// update or schema change closes the write path until it is checked again.
const VERIFIED_CN_EXE_SHA256: &str =
    "b2877bdde11794652c93dc81625ef3ae6befa77a321463039e101956e470de22";
const VERIFIED_CN_SESSION_SCHEMA_SHA256: &str =
    "58f169b7f8ae860d7dd7c31424ae2e43d3a2980b79df5ddff40bc5d2f9a3b5ae";
const VERIFIED_CN_EDGE_SCHEMA_SHA256: &str =
    "5abcbbd5319851cd637d7e08002f546dd4041d08edc00b33d4a3dc99d9f060fd";

/// Matches only the observed WorkBuddy CN 5.8.1 local-session candidate.
/// This structural check is not a completed live write/read acceptance test.
/// Row and body checks still reject unsupported references.
pub fn require_verified_local_client(variant: WbVariant) -> Result<(), String> {
    if variant != WbVariant::Cn {
        return Err(UNVERIFIED_CLIENT.into());
    }
    #[cfg(not(windows))]
    {
        return Err(UNVERIFIED_CLIENT.into());
    }
    #[cfg(windows)]
    {
        let exe = process::windows_workbuddy_exe_path(variant)
            .ok_or_else(|| UNVERIFIED_CLIENT.to_string())?;
        if file_digest(&exe)? != VERIFIED_CN_EXE_SHA256 {
            return Err("WorkBuddy 客户端版本与已验证的 5.8.1 不同，已停止会话写入".into());
        }
        let paths = SessionPaths::for_variant(variant);
        let main = open_readonly(&paths.workbuddy_db())?;
        validate_session_schema(&main)?;
        if pragma_i64(&main, "schema_version")? != 58
            || schema_digest(&main)? != VERIFIED_CN_SESSION_SCHEMA_SHA256
        {
            return Err("WorkBuddy 会话库结构已变化，已停止会话写入".into());
        }
        let edge = open_readonly(&paths.edge_sync_db(variant))?;
        if pragma_i64(&edge, "user_version")? != 1
            || schema_digest(&edge)? != VERIFIED_CN_EDGE_SCHEMA_SHA256
        {
            return Err("WorkBuddy 映射库结构已变化，已停止会话写入".into());
        }
        Ok(())
    }
}

fn file_digest(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|_| UNVERIFIED_CLIENT.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let len = file
            .read(&mut buffer)
            .map_err(|_| UNVERIFIED_CLIENT.to_string())?;
        if len == 0 {
            break;
        }
        digest.update(&buffer[..len]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn open_readonly(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| UNVERIFIED_CLIENT.to_string())?;
    conn.execute_batch("PRAGMA query_only=ON;")
        .map_err(|_| UNVERIFIED_CLIENT.to_string())?;
    Ok(conn)
}

fn pragma_i64(conn: &Connection, name: &str) -> Result<i64, String> {
    match name {
        "schema_version" => conn.query_row("PRAGMA schema_version", [], |row| row.get(0)),
        "user_version" => conn.query_row("PRAGMA user_version", [], |row| row.get(0)),
        _ => return Err(UNVERIFIED_CLIENT.into()),
    }
    .map_err(|_| UNVERIFIED_CLIENT.to_string())
}

fn schema_digest(conn: &Connection) -> Result<String, String> {
    let mut stmt = conn
        .prepare("SELECT type,name,tbl_name,COALESCE(sql,'') FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name")
        .map_err(|_| UNVERIFIED_CLIENT.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok([
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ])
        })
        .map_err(|_| UNVERIFIED_CLIENT.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| UNVERIFIED_CLIENT.to_string())?;
    let bytes = serde_json::to_vec(&rows).map_err(|_| UNVERIFIED_CLIENT.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

/// Installed 5.7.7 distinguishes local NULL from cloud -1 deletion sentinels.
/// Reject unsupported bindings rather than cloning or silently stripping them.
/// Older fictional schemas without transport remain testable; this never grants
/// broad production permission (require_verified_client stays closed).
pub(crate) fn validate_local_transfer_row(
    row: &[(String, rusqlite::types::Value)],
) -> Result<(), String> {
    validate_local_transfer_kind(row)?;
    let deleted = row
        .iter()
        .find(|(key, _)| key == "deleted_at")
        .map(|(_, value)| value);
    if !matches!(deleted, Some(rusqlite::types::Value::Null)) {
        return Err("已删除或删除状态无法确认的会话不支持迁移写入".into());
    }
    Ok(())
}

/// Recovery may roll back a deleted local row's body. It must still never mutate
/// cloud/Buddy/group-bound bodies using the legacy local recovery algorithm.
pub(crate) fn validate_local_transfer_kind(
    row: &[(String, rusqlite::types::Value)],
) -> Result<(), String> {
    use rusqlite::types::Value;
    let field = |name: &str| {
        row.iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    };
    if let Some(transport) = field("transport") {
        if !matches!(transport, Value::Text(value) if value == "local") {
            return Err("云会话或未知传输类型暂不支持复制、同步或恢复写入".into());
        }
    }
    for name in [
        "buddy_snapshot_id",
        "buddy_binding_json",
        "group_id",
        "group_title",
    ] {
        if let Some(value) = field(name) {
            if !matches!(value, Value::Null)
                && !matches!(value, Value::Text(text) if text.is_empty())
            {
                return Err("会话含 Buddy 快照或官方分组绑定，尚未验证完整迁移，已停止写入".into());
            }
        }
    }
    Ok(())
}

/// The current copy/sync algorithm moves one JSONL file, not files referenced
/// by that JSONL. Refuse records with known external resources before writing.
pub(crate) fn validate_self_contained_body(text: &str) -> Result<(), String> {
    for line in text.lines() {
        let record: serde_json::Value =
            serde_json::from_str(line).map_err(|_| "会话正文无法解析，已停止写入".to_string())?;
        if record.get("type").and_then(serde_json::Value::as_str) == Some("file-history-snapshot")
            && record
                .get("snapshot")
                .and_then(|snapshot| snapshot.get("trackedFileBackups"))
                .is_some_and(nonempty_reference)
        {
            return Err("会话含文件历史备份引用，当前无法完整迁移，已停止写入".into());
        }
        if has_attachment_reference(&record) {
            return Err("会话含附件或外部文件引用，当前无法完整迁移，已停止写入".into());
        }
    }
    Ok(())
}

fn nonempty_reference(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::Array(items) => !items.is_empty(),
        serde_json::Value::Object(items) => !items.is_empty(),
        serde_json::Value::String(text) => !text.is_empty(),
        _ => true,
    }
}

fn has_attachment_reference(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(items) => items.iter().any(has_attachment_reference),
        serde_json::Value::Object(items) => items.iter().any(|(key, nested)| {
            matches!(
                key.as_str(),
                "attachments" | "attachment" | "fileReferences" | "file_refs" | "assetRefs"
            ) && nonempty_reference(nested)
                || has_attachment_reference(nested)
        }),
        _ => false,
    }
}

/// Inspect manager-owned operation records before closing the client or backing up auth.
/// Plain account switching can proceed only when it needs no session recovery/write.
pub fn preflight_switch(
    paths: &SessionPaths,
    variant: WbVariant,
    requests_session_writes: bool,
) -> Result<(), String> {
    if requests_session_writes {
        return require_verified_local_client(variant);
    }
    let scan = session_link::scan_operations(paths);
    if !scan.complete
        || !scan.problems.is_empty()
        || scan
            .operations
            .iter()
            .any(|op| op.variant == variant && op.phase.is_unfinished())
    {
        return Err(format!(
            "{UNVERIFIED_CLIENT}；存在未完成或无法读取的操作记录，请保留现场"
        ));
    }
    Ok(())
}

/// Minimum structural contract for the retained algorithm and its fictional fixtures.
/// This is not an official schema allowlist; production still requires the gate above.
pub fn validate_session_schema(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_xinfo(sessions)")
        .map_err(|_| "无法读取会话数据库结构".to_string())?;
    let columns = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .map_err(|_| "无法读取会话数据库结构".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "无法读取会话数据库结构".to_string())?;
    for (name, expected) in [
        ("id", "TEXT"),
        ("user_id", "TEXT"),
        ("title", "TEXT"),
        ("cwd", "TEXT"),
        ("created_at", "INTEGER"),
        ("updated_at", "INTEGER"),
        ("deleted_at", "INTEGER"),
    ] {
        if !columns.iter().any(|(col, ty, pk, hidden)| {
            col == name
                && ty.eq_ignore_ascii_case(expected)
                && *hidden == 0
                && (name != "id" || *pk == 1)
        }) {
            return Err("会话数据库结构不兼容，已停止写入".into());
        }
    }
    Ok(())
}

/// Probe an existing target without creating a database or changing rows. SQLite's
/// immediate transaction detects another writer; it cannot prevent a client starting
/// later, which is why the exact client, row, body and process checks all matter.
pub fn preflight_target_database(paths: &SessionPaths) -> Result<(), String> {
    let db = paths.workbuddy_db();
    let root = paths
        .data_root
        .canonicalize()
        .map_err(|_| "会话数据路径无法验证".to_string())?;
    let actual = db
        .canonicalize()
        .map_err(|_| "会话数据库不存在或无法访问".to_string())?;
    if actual.parent() != Some(root.as_path()) || !actual.is_file() {
        return Err("会话数据库路径不在已验证的数据目录内".into());
    }
    if std::fs::metadata(&actual)
        .map_err(|_| "会话数据库无法访问".to_string())?
        .permissions()
        .readonly()
    {
        return Err("会话数据库不可写，已停止操作".into());
    }
    let mut conn =
        Connection::open_with_flags(&actual, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(|_| "会话数据库不可写或被占用".to_string())?;
    conn.busy_timeout(std::time::Duration::ZERO)
        .map_err(|_| "无法检查数据库占用".to_string())?;
    crate::modules::session_backup::prepare_write_connection(&conn)?;
    let tx = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|_| "会话数据库正在被占用，已停止操作".to_string())?;
    tx.rollback()
        .map_err(|_| "数据库占用检查无法安全结束".to_string())
}

pub fn validate_body_path(paths: &SessionPaths, path: &Path) -> Result<(), String> {
    if path
        .components()
        .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err("会话正文路径不合法".into());
    }
    let root = paths
        .data_root
        .canonicalize()
        .map_err(|_| "会话数据路径无法验证".to_string())?;
    let projects = paths
        .projects_dir()
        .canonicalize()
        .map_err(|_| "会话正文目录无法验证".to_string())?;
    let parent = path
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .ok_or_else(|| "会话正文路径无法验证".to_string())?;
    if !projects.starts_with(&root)
        || !parent.starts_with(&projects)
        || std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
    {
        return Err("会话正文路径越出数据目录或包含链接，已停止写入".into());
    }
    Ok(())
}

#[cfg(test)]
mod compatibility_profile_tests {
    use super::*;

    #[test]
    fn schema_fingerprint_changes_only_with_structure() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE sessions (id TEXT PRIMARY KEY); INSERT INTO sessions VALUES ('a');",
        )
        .unwrap();
        let before = schema_digest(&conn).unwrap();
        conn.execute("INSERT INTO sessions VALUES ('b')", [])
            .unwrap();
        assert_eq!(schema_digest(&conn).unwrap(), before);
        conn.execute_batch("ALTER TABLE sessions ADD COLUMN transport TEXT;")
            .unwrap();
        assert_ne!(schema_digest(&conn).unwrap(), before);
    }

    #[test]
    fn external_resource_references_are_rejected_before_write() {
        assert!(validate_self_contained_body(r#"{"type":"message","content":"test"}"#).is_ok());
        assert!(validate_self_contained_body(
            r#"{"type":"file-history-snapshot","snapshot":{"trackedFileBackups":{"a":"b"}}}"#
        )
        .is_err());
        assert!(validate_self_contained_body(
            r#"{"type":"message","attachments":[{"id":"asset"}]}"#
        )
        .is_err());
    }
}
