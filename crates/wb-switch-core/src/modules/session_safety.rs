//! Production compatibility remains unverified. Fixture checks do not grant write permission.
use crate::modules::{session::SessionPaths, session_link, variant::WbVariant};
use rusqlite::Connection;
use std::path::Path;

pub const UNVERIFIED_CLIENT: &str =
    "尚未验证 WorkBuddy 客户端版本与数据库兼容性，已禁用会话复制、同步和恢复写入";

/// No official version/schema pair has been verified for this product yet.
/// Never enable this from a frontend flag, environment variable, or the presence of a table.
pub fn require_verified_client() -> Result<(), String> {
    Err(UNVERIFIED_CLIENT.into())
}

/// Installed 5.7.7 distinguishes local NULL from cloud -1 deletion sentinels.
/// Reject unsupported bindings rather than cloning or silently stripping them.
/// Older fictional schemas without transport remain testable; this never grants
/// a production version/schema permission (require_verified_client stays closed).
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

/// Inspect manager-owned operation records before closing the client or backing up auth.
/// Plain account switching can proceed only when it needs no session recovery/write.
pub fn preflight_switch(
    paths: &SessionPaths,
    variant: WbVariant,
    requests_session_writes: bool,
) -> Result<(), String> {
    if requests_session_writes {
        return require_verified_client();
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
/// later, which is why the production gate remains closed.
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
