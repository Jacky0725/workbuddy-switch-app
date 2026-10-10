#![cfg(windows)]
use serde_json::json;
use std::path::PathBuf;
use wb_switch_core::modules::{
    account_repository::AccountRepository,
    manager_backup::{self as backup, RestoreRequest},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("wbm-backup-test-{}", uuid::Uuid::new_v4())))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn accounts() -> Vec<serde_json::Value> {
    vec![
        json!({"id":"fiction", "uid":"fiction-uid", "nickname":"虚构账号", "variant":"cn", "access_token":"fiction-secret", "refresh_token":"fiction-refresh"}),
    ]
}
#[test]
fn backup_is_encrypted_restores_credentials_and_keeps_rollback() {
    let fixture = Fixture::new();
    let repo = AccountRepository::new(&fixture.0.join("accounts.json"));
    repo.save(&accounts()).unwrap();
    let saved = backup::create_at(&fixture.0).unwrap();
    let bytes = std::fs::read(fixture.0.join("backups").join(format!("{}.wbm", saved.id))).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("fiction-secret"));
    repo.save(&[]).unwrap();
    let preview = backup::preview_at(&fixture.0, &saved.id).unwrap();
    let restored = backup::restore_at(
        &fixture.0,
        RestoreRequest {
            id: saved.id,
            expected_revision: preview.current_revision,
            confirmed: true,
        },
    )
    .unwrap();
    assert_eq!(restored.restored_accounts, 1);
    assert_eq!(repo.load().unwrap()[0]["access_token"], "fiction-secret");
    assert_eq!(
        backup::preview_at(&fixture.0, &restored.rollback_id)
            .unwrap()
            .account_count,
        0
    );
    assert!(!fixture.0.join("daily_scheduler_config.json").exists());
    let notifications = std::fs::read_to_string(fixture.0.join("notifications.json")).unwrap();
    assert!(notifications.contains("Manager 账号库恢复完成"));
    assert!(!notifications.contains("fiction-secret"));
}
#[test]
fn invalid_identifier_tampering_missing_confirmation_and_stale_preview_preserve_index() {
    let fixture = Fixture::new();
    let repo = AccountRepository::new(&fixture.0.join("accounts.json"));
    repo.save(&accounts()).unwrap();
    let saved = backup::create_at(&fixture.0).unwrap();
    let preview = backup::preview_at(&fixture.0, &saved.id).unwrap();
    let index = std::fs::read(fixture.0.join("accounts.json")).unwrap();
    assert!(backup::preview_at(&fixture.0, "../accounts").is_err());
    for (confirmed, revision) in [
        (false, preview.current_revision.clone()),
        (true, "stale".into()),
    ] {
        assert!(backup::restore_at(
            &fixture.0,
            RestoreRequest {
                id: saved.id.clone(),
                expected_revision: revision,
                confirmed
            }
        )
        .is_err());
        assert_eq!(
            std::fs::read(fixture.0.join("accounts.json")).unwrap(),
            index
        );
    }
    std::fs::write(
        fixture.0.join("backups").join(format!("{}.wbm", saved.id)),
        b"corrupt",
    )
    .unwrap();
    assert!(backup::preview_at(&fixture.0, &saved.id).is_err());
    assert!(backup::list_at(&fixture.0).unwrap()[0].error.is_some());
    assert_eq!(
        std::fs::read(fixture.0.join("accounts.json")).unwrap(),
        index
    );
}
#[test]
fn lock_and_failed_backup_prevent_restore_before_index_replacement() {
    use fs2::FileExt;
    let fixture = Fixture::new();
    let repo = AccountRepository::new(&fixture.0.join("accounts.json"));
    repo.save(&accounts()).unwrap();
    let saved = backup::create_at(&fixture.0).unwrap();
    let preview = backup::preview_at(&fixture.0, &saved.id).unwrap();
    let index = std::fs::read(fixture.0.join("accounts.json")).unwrap();
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(fixture.0.join("accounts.lock"))
        .unwrap();
    lock.lock_exclusive().unwrap();
    assert!(backup::restore_at(
        &fixture.0,
        RestoreRequest {
            id: saved.id.clone(),
            expected_revision: preview.current_revision.clone(),
            confirmed: true
        }
    )
    .is_err());
    drop(lock);
    // Replace the backup directory with a fixture file to block creation.
    let old = fixture.0.join("backups");
    let held = fixture.0.join("held-backups");
    std::fs::rename(&old, &held).unwrap();
    std::fs::write(&old, b"blocked").unwrap();
    assert!(backup::create_at(&fixture.0).is_err());
    assert_eq!(
        std::fs::read(fixture.0.join("accounts.json")).unwrap(),
        index
    );
}
#[test]
fn failed_atomic_restore_commit_preserves_existing_accounts_and_rollback() {
    let fixture = Fixture::new();
    let repo = AccountRepository::new(&fixture.0.join("accounts.json"));
    repo.save(&accounts()).unwrap();
    let saved = backup::create_at(&fixture.0).unwrap();
    repo.save(&[]).unwrap();
    let preview = backup::preview_at(&fixture.0, &saved.id).unwrap();
    let path = fixture.0.join("accounts.json");
    let original = std::fs::read(&path).unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&path, permissions.clone()).unwrap();
    let result = backup::restore_at(
        &fixture.0,
        RestoreRequest {
            id: saved.id,
            expected_revision: preview.current_revision,
            confirmed: true,
        },
    );
    permissions.set_readonly(false);
    std::fs::set_permissions(&path, permissions).unwrap();
    assert!(result.is_err());
    assert_eq!(std::fs::read(path).unwrap(), original);
    assert!(repo.load().unwrap().is_empty());
    assert_eq!(backup::list_at(&fixture.0).unwrap().len(), 2);
}
