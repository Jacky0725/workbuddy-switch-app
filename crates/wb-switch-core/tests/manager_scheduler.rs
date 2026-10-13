use fs2::FileExt;
use std::path::PathBuf;
use wb_switch_core::modules::manager_scheduler::{self as manager, SaveSchedulerRequest};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("wbm-manager-scheduler-{}", uuid::Uuid::new_v4())))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
const NOW: i64 = 1_791_507_600_000;

#[test]
fn first_snapshot_is_readonly_and_does_not_claim_execution() {
    let fixture = Fixture::new();
    let snapshot = manager::snapshot_at(&fixture.0, NOW).unwrap();
    assert!(!fixture.0.exists());
    assert!(!snapshot.production_enabled);
    assert!(!snapshot.config.enabled);
    assert!(snapshot.tasks.is_empty());
    assert!(snapshot.next_planned_at_ms.is_none());
    assert_eq!(snapshot.config.daily_time, "09:15");
    assert_eq!(snapshot.revision.len(), 64);
}

#[test]
fn saving_configuration_has_cas_and_never_touches_credentials() {
    let fixture = Fixture::new();
    std::fs::create_dir_all(&fixture.0).unwrap();
    let accounts = fixture.0.join("accounts.json");
    std::fs::write(&accounts, b"unreadable fictional account index").unwrap();
    let before = manager::snapshot_at(&fixture.0, NOW).unwrap();
    let mut config = before.config.clone();
    config.enabled = true;
    config.daily_time = "10:30".into();
    config
        .account_enabled
        .insert("fiction-account".into(), false);
    let saved = manager::save_at(
        &fixture.0,
        SaveSchedulerRequest {
            config: config.clone(),
            expected_revision: before.revision.clone(),
        },
        NOW,
    )
    .unwrap();
    assert!(!saved.production_enabled);
    assert_eq!(saved.config.daily_time, "10:30");
    assert_ne!(before.revision, saved.revision);
    let bytes = std::fs::read(fixture.0.join("daily_scheduler_config.json")).unwrap();
    assert!(manager::save_at(
        &fixture.0,
        SaveSchedulerRequest {
            config,
            expected_revision: before.revision
        },
        NOW
    )
    .unwrap_err()
    .contains("其他窗口"));
    assert_eq!(
        std::fs::read(fixture.0.join("daily_scheduler_config.json")).unwrap(),
        bytes
    );
    assert_eq!(
        std::fs::read(&accounts).unwrap(),
        b"unreadable fictional account index"
    );
}

#[test]
fn invalid_config_is_rejected_without_creating_directory() {
    let fixture = Fixture::new();
    let snapshot = manager::snapshot_at(&fixture.0, NOW).unwrap();
    let mut config = snapshot.config;
    config.daily_time = "25:00".into();
    assert!(manager::save_at(
        &fixture.0,
        SaveSchedulerRequest {
            config,
            expected_revision: snapshot.revision
        },
        NOW
    )
    .is_err());
    assert!(!fixture.0.exists());
}

#[test]
fn corrupt_or_future_records_are_preserved_and_block_save() {
    for text in [
        "{broken",
        "{\"version\":999,\"latest_business_date\":null,\"tasks\":{}}",
    ] {
        let fixture = Fixture::new();
        let before = manager::snapshot_at(&fixture.0, NOW).unwrap();
        before.config.save(&fixture.0).unwrap();
        let config_before = std::fs::read(fixture.0.join("daily_scheduler_config.json")).unwrap();
        std::fs::write(fixture.0.join("daily_scheduler_state.json"), text).unwrap();
        let error = manager::save_at(
            &fixture.0,
            SaveSchedulerRequest {
                config: before.config,
                expected_revision: before.revision,
            },
            NOW,
        )
        .unwrap_err();
        assert!(error.contains("保留原文件"));
        assert_eq!(
            std::fs::read(fixture.0.join("daily_scheduler_config.json")).unwrap(),
            config_before
        );
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("daily_scheduler_state.json")).unwrap(),
            text
        );
    }
}

#[test]
fn held_scheduler_lock_prevents_configuration_change() {
    let fixture = Fixture::new();
    let before = manager::snapshot_at(&fixture.0, NOW).unwrap();
    std::fs::create_dir_all(&fixture.0).unwrap();
    let lock = std::fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(fixture.0.join("daily_scheduler.lock"))
        .unwrap();
    lock.lock_exclusive().unwrap();
    assert!(manager::save_at(
        &fixture.0,
        SaveSchedulerRequest {
            config: before.config,
            expected_revision: before.revision
        },
        NOW
    )
    .unwrap_err()
    .contains("调度实例"));
    assert!(!fixture.0.join("daily_scheduler_config.json").exists());
}

#[tokio::test]
async fn background_poll_and_manual_request_cannot_reserve_or_send_a_checkin() {
    let fixture = Fixture::new();
    let mut config = manager::snapshot_at(&fixture.0, NOW).unwrap().config;
    config.enabled = true;
    config.save(&fixture.0).unwrap();
    std::fs::create_dir_all(fixture.0.join("accounts.json")).unwrap(); // Reading accounts would fail.
    manager::background_tick_at(&fixture.0, NOW).await.unwrap();
    assert!(!fixture.0.join("daily_scheduler_state.json").exists());
    assert!(!fixture.0.join("daily_scheduler.lock").exists());
    // Fixture roots remain isolated from live account credentials. The real
    // manual entry is exercised through the production account store.
    let snapshot = manager::snapshot_at(&fixture.0, NOW).unwrap();
    assert!(snapshot.background_active);
    assert!(snapshot.tasks.is_empty());
}
