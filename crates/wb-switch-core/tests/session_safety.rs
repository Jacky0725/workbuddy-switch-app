use rusqlite::Connection;
use serde_json::json;
use std::path::PathBuf;
use wb_switch_core::modules::{
    session::{self, LinkNamespace, SessionPaths},
    session_backup,
    session_groups::{self, SessionClient},
    session_safety,
    variant::WbVariant,
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("wbm-session-safety-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        Self(root)
    }
    fn paths(&self) -> SessionPaths {
        SessionPaths {
            store_root: self.0.join("manager"),
            data_root: self.0.join("data"),
            auth_file: self.0.join("fictional-auth.json"),
            link_namespace: LinkNamespace::WorkBuddy,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn database() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE sessions(id TEXT PRIMARY KEY, user_id TEXT NOT NULL, title TEXT, cwd TEXT, created_at INTEGER, updated_at INTEGER, deleted_at INTEGER);").unwrap();
    conn
}

#[test]
fn all_production_copy_sync_and_group_writes_are_denied_before_resolving_paths() {
    let source = json!({"id":"fiction-a", "uid":"fiction-uid-a", "variant":"cn"});
    let target = json!({"id":"fiction-b", "uid":"fiction-uid-b", "variant":"cn"});
    for result in [
        session::copy_sessions_for_switch(&target, &["fiction-session".into()]),
        session::copy_sessions_cross(&source, &target, &["fiction-session".into()]),
        session::sync_sessions_for_switch(&target, &[]),
        session::sync_sessions_cross(&source, &target, &[]),
        session_groups::sync_pair(
            SessionClient::Workbuddy,
            None,
            "fiction-group",
            "a",
            "b",
            "fiction-token",
            "fastForward",
            true,
        ),
        session_groups::sync_unify_batch(
            SessionClient::Workbuddy,
            None,
            "fiction-group",
            "a",
            &[],
            true,
        ),
        session_groups::sync_safe_batch(SessionClient::Workbuddy, None, "fiction-group", true),
        session_groups::add_member(
            SessionClient::Workbuddy,
            None,
            "fiction-group",
            "a",
            "fiction-b",
            true,
        ),
    ] {
        assert_eq!(result.unwrap_err(), session_safety::UNVERIFIED_CLIENT);
    }
}

#[test]
fn switch_preflight_with_no_pending_operation_creates_no_files() {
    let fixture = Fixture::new();
    let paths = fixture.paths();
    session_safety::preflight_switch(&paths, WbVariant::Cn, false).unwrap();
    assert!(!paths.store_root.exists());
    assert!(!paths.data_root.exists());
    assert_eq!(
        session_safety::preflight_switch(&paths, WbVariant::Cn, true).unwrap_err(),
        session_safety::UNVERIFIED_CLIENT
    );
    assert!(!paths.store_root.exists());
}

#[test]
fn unreadable_operation_is_preserved_and_prevents_switch_recovery() {
    let fixture = Fixture::new();
    let paths = fixture.paths();
    std::fs::create_dir_all(paths.operations_dir()).unwrap();
    let record = paths.operations_dir().join("fiction-op.json");
    std::fs::write(&record, b"{broken").unwrap();
    let before = std::fs::read(&record).unwrap();
    let error = session_safety::preflight_switch(&paths, WbVariant::Cn, false).unwrap_err();
    assert!(error.contains("保留现场"));
    assert_eq!(std::fs::read(&record).unwrap(), before);
    assert!(!paths.data_root.exists());
}

#[test]
fn schema_changes_and_missing_identity_constraints_are_rejected() {
    let conn = database();
    session_safety::validate_session_schema(&conn).unwrap();
    conn.execute_batch("ALTER TABLE sessions RENAME COLUMN updated_at TO changed_updated_at")
        .unwrap();
    assert!(session_safety::validate_session_schema(&conn).is_err());
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE sessions(id TEXT, user_id TEXT, title TEXT, cwd TEXT, created_at INTEGER, updated_at INTEGER, deleted_at INTEGER)").unwrap();
    assert!(session_safety::validate_session_schema(&conn).is_err());
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE sessions(id TEXT PRIMARY KEY, user_id TEXT, title TEXT, cwd TEXT, created_at INTEGER, updated_at TEXT, deleted_at INTEGER)").unwrap();
    assert!(session_safety::validate_session_schema(&conn).is_err());
}

#[test]
fn preexisting_foreign_key_violation_is_detected_without_repairing_data() {
    let conn = database();
    conn.execute_batch("PRAGMA foreign_keys=OFF; CREATE TABLE child(owner TEXT REFERENCES sessions(id)); INSERT INTO child VALUES('missing-parent');").unwrap();
    assert!(session_backup::verify_database_integrity(&conn)
        .unwrap_err()
        .contains("外键"));
    assert!(session_backup::prepare_write_connection(&conn).is_err());
    let count: i64 = conn
        .query_row("SELECT count(*) FROM child", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn deferred_foreign_key_violation_rolls_back_whole_transaction() {
    let mut conn = database();
    conn.execute_batch(
        "CREATE TABLE child(owner TEXT REFERENCES sessions(id) DEFERRABLE INITIALLY DEFERRED);",
    )
    .unwrap();
    session_backup::prepare_write_connection(&conn).unwrap();
    {
        let tx = conn.transaction().unwrap();
        tx.execute(
            "INSERT INTO sessions(id,user_id) VALUES('fiction-session','fiction-uid')",
            [],
        )
        .unwrap();
        tx.execute("INSERT INTO child VALUES('missing-parent')", [])
            .unwrap();
        assert!(session_backup::verify_database_integrity(&tx).is_err());
    }
    let count: i64 = conn
        .query_row("SELECT count(*) FROM sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
    session_backup::verify_database_integrity(&conn).unwrap();
}

#[test]
fn target_preflight_rejects_missing_readonly_and_occupied_database_without_writes() {
    let fixture = Fixture::new();
    let paths = fixture.paths();
    assert!(session_safety::preflight_target_database(&paths).is_err());
    assert!(!paths.data_root.exists());
    std::fs::create_dir_all(&paths.data_root).unwrap();
    let conn = Connection::open(paths.workbuddy_db()).unwrap();
    conn.execute_batch("CREATE TABLE sessions(id TEXT PRIMARY KEY, user_id TEXT NOT NULL, title TEXT, cwd TEXT, created_at INTEGER, updated_at INTEGER, deleted_at INTEGER)").unwrap();
    session_safety::preflight_target_database(&paths).unwrap();
    conn.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert!(session_safety::preflight_target_database(&paths)
        .unwrap_err()
        .contains("占用"));
    conn.execute_batch("ROLLBACK").unwrap();
    drop(conn);
    let original = std::fs::metadata(paths.workbuddy_db())
        .unwrap()
        .permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    std::fs::set_permissions(paths.workbuddy_db(), readonly).unwrap();
    let result = session_safety::preflight_target_database(&paths);
    std::fs::set_permissions(paths.workbuddy_db(), original).unwrap();
    assert!(result.unwrap_err().contains("不可写"));
    let conn = Connection::open(paths.workbuddy_db()).unwrap();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn body_path_rejects_outside_directory_and_parent_traversal() {
    let fixture = Fixture::new();
    let paths = fixture.paths();
    let workspace = paths.projects_dir().join("fiction-workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    session_safety::validate_body_path(&paths, &workspace.join("fiction-session.jsonl")).unwrap();
    assert!(session_safety::validate_body_path(&paths, &fixture.0.join("outside.jsonl")).is_err());
    assert!(session_safety::validate_body_path(
        &paths,
        &workspace.join("../fiction-session.jsonl")
    )
    .is_err());
}
