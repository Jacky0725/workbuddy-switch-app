//! All credentials are fabricated; DPAPI uses the current Windows user solely
//! on these temporary fixtures. No official auth file, database or network.
#![cfg(windows)]

use serde_json::{json, Value};
use std::path::PathBuf;
use wb_switch_core::modules::{
    account,
    account_repository::{capabilities, scope_of, AccountRepository},
    credential_store::{read_legacy_backup, CapabilityType, CredentialStore},
    export_import,
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("wbm-security-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> PathBuf {
        self.0.join("accounts.json")
    }
    fn repo(&self) -> AccountRepository {
        AccountRepository::new(&self.path())
    }
    fn read(&self) -> String {
        std::fs::read_to_string(self.path()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn record(id: &str, region: &str) -> Value {
    json!({"id": id, "uid": "same-uid", "variant": region, "nickname": "同名",
        "access_token": "FABRICATED-ACCESS-SECRET", "refresh_token": "FABRICATED-REFRESH-SECRET",
        "auth_raw": {"accessToken": "FABRICATED-NESTED-SECRET"},
        "profile_raw": {"phoneNumber": "000-fixture", "unknownToken": "FABRICATED-PROFILE-SECRET"},
        "future_secret": "FABRICATED-UNKNOWN-SECRET"})
}

#[test]
fn dpapi_roundtrip_and_reference_binding() {
    let f = Fixture::new();
    let store = CredentialStore::new(&f.0);
    let scope = scope_of(&record("a", "cn"));
    let reference = store
        .save(
            "a",
            &scope,
            CapabilityType::DesktopSwitch,
            &json!({"token": "fake-secret"}),
            None,
        )
        .unwrap();
    assert_eq!(
        store.load(&reference, "a", &scope).unwrap()["token"],
        "fake-secret"
    );
    assert!(store.load(&reference, "other", &scope).is_err());
    assert!(store
        .load(&reference, "a", &scope_of(&record("a", "ai")))
        .is_err());
    let mut switched = reference.clone();
    switched.capability_type = CapabilityType::Checkin;
    assert!(store.load(&switched, "a", &scope).is_err());
    switched.secure_storage_key = "../../accounts.json".into();
    assert!(store.load(&switched, "a", &scope).is_err());
}

#[test]
fn persisted_index_and_all_files_contain_no_plaintext_credentials() {
    let f = Fixture::new();
    f.repo().save(&[record("a", "cn")]).unwrap();
    let raw = f.read();
    assert!(!raw.contains("FABRICATED-"));
    assert!(!raw.contains("access_token"));
    assert!(!raw.contains("auth_raw"));
    let index: Value = serde_json::from_str(&raw).unwrap();
    assert!(index[0]["createdAt"].as_i64().is_some());
    assert!(index[0]["updatedAt"].as_i64().is_some());
    assert_eq!(
        index[0]["credential_references"][0]["capability_type"],
        "DesktopSwitch"
    );
    for item in std::fs::read_dir(f.0.join("credentials")).unwrap() {
        let text = std::fs::read_to_string(item.unwrap().path()).unwrap();
        assert!(!text.contains("FABRICATED-"));
    }
    assert_eq!(
        f.repo().load().unwrap()[0]["auth_raw"]["accessToken"],
        "FABRICATED-NESTED-SECRET"
    );
}

#[test]
fn legacy_migration_creates_recoverable_encrypted_backup_first() {
    let f = Fixture::new();
    let original = serde_json::to_vec(&vec![record("a", "cn")]).unwrap();
    std::fs::write(f.path(), &original).unwrap();
    let mut accounts = f.repo().load().unwrap();
    accounts[0]["note"] = json!("migrated");
    f.repo().save(&accounts).unwrap();
    let files: Vec<_> = std::fs::read_dir(f.0.join("backups"))
        .unwrap()
        .map(|v| v.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1);
    assert!(!std::fs::read_to_string(&files[0])
        .unwrap()
        .contains("FABRICATED-"));
    assert_eq!(read_legacy_backup(&files[0]).unwrap(), original);
    let restored = Fixture::new();
    std::fs::write(restored.path(), read_legacy_backup(&files[0]).unwrap()).unwrap();
    assert_eq!(
        restored.repo().load().unwrap()[0]["access_token"],
        "FABRICATED-ACCESS-SECRET"
    );
    assert!(!f.read().contains("FABRICATED-"));
}

#[test]
fn corrupted_index_and_unknown_version_are_never_overwritten() {
    let f = Fixture::new();
    for raw in ["{corrupted", r#"[{"id":"a","storage_version":99}]"#, "[42]"] {
        std::fs::write(f.path(), raw).unwrap();
        let error = f
            .repo()
            .save(&[record("new", "cn")])
            .unwrap_err()
            .to_string();
        assert!(!error.contains("FABRICATED-"));
        assert_eq!(f.read(), raw);
    }
}

#[test]
fn bad_ciphertext_shows_reauthorization_and_preserves_unreadable_reference() {
    let f = Fixture::new();
    f.repo().save(&[record("a", "cn")]).unwrap();
    let index: Value = serde_json::from_str(&f.read()).unwrap();
    let key = index[0]["credential_references"][0]["secure_storage_key"]
        .as_str()
        .unwrap();
    std::fs::write(
        f.0.join("credentials").join(format!("{key}.dpapi")),
        "corrupted",
    )
    .unwrap();
    let mut locked = f.repo().load().unwrap();
    assert_eq!(
        capabilities(&locked[0])[0]["authorization_status"],
        "CredentialUnavailable"
    );
    assert!(locked[0].get("access_token").is_none());
    locked[0]["note"] = json!("display-only edit");
    f.repo().save(&locked).unwrap();
    let after: Value = serde_json::from_str(&f.read()).unwrap();
    assert_eq!(
        after[0]["credential_references"],
        index[0]["credential_references"]
    );
    assert_eq!(after[0]["note"], "display-only edit");
    f.repo()
        .update(|accounts| {
            account::upsert_collected_account(accounts, record("new-local-id", "cn"));
            Ok(())
        })
        .unwrap();
    let repaired = f.repo().load().unwrap();
    assert_eq!(repaired[0]["id"], "a");
    assert_eq!(repaired[0]["access_token"], "FABRICATED-ACCESS-SECRET");
}

#[test]
fn changing_external_uid_cannot_reassign_an_existing_credential() {
    let f = Fixture::new();
    f.repo().save(&[record("a", "cn")]).unwrap();
    let mut index: Value = serde_json::from_str(&f.read()).unwrap();
    index[0]["uid"] = json!("different-external-identity");
    std::fs::write(f.path(), serde_json::to_string(&index).unwrap()).unwrap();
    let loaded = f.repo().load().unwrap();
    assert_eq!(loaded[0]["credential_locked"], true);
    assert!(loaded[0].get("access_token").is_none());
}

#[test]
fn index_cannot_inject_plaintext_tokens_into_secure_records() {
    let f = Fixture::new();
    f.repo().save(&[record("a", "cn")]).unwrap();
    let mut index: Value = serde_json::from_str(&f.read()).unwrap();
    index[0]["access_token"] = json!("untrusted-token-in-index");
    std::fs::write(f.path(), serde_json::to_string(&index).unwrap()).unwrap();
    assert!(f.repo().load().is_err());
    let before = f.read();
    assert!(f.repo().save(&[record("a", "cn")]).is_err());
    assert_eq!(f.read(), before);
}

#[test]
fn region_and_provider_never_merge_identical_uid_or_email() {
    let mut accounts = vec![record("cn", "cn")];
    account::upsert_collected_account(&mut accounts, record("ai", "ai"));
    let mut other = record("other", "cn");
    other["provider"] = json!("other-provider");
    account::upsert_collected_account(&mut accounts, other);
    assert_eq!(accounts.len(), 3);
    let mut email_accounts = vec![json!({"id":"cn","email":"same@example.invalid","variant":"cn"})];
    account::upsert_collected_account(
        &mut email_accounts,
        json!({"id":"ai","email":"same@example.invalid","variant":"ai"}),
    );
    assert_eq!(email_accounts.len(), 2);
}

#[test]
fn cross_region_import_regenerates_colliding_local_id_and_preserves_stable_same_scope_id() {
    let mut accounts = vec![record("local", "cn")];
    let other = record("local", "ai");
    let text = serde_json::to_string(&vec![other]).unwrap();
    let result = export_import::merge_import_records(&mut accounts, &text, &[0]).unwrap();
    assert_eq!(result.overwritten, 0);
    assert_eq!(accounts.len(), 2);
    assert_ne!(accounts[0]["id"], accounts[1]["id"]);
    let text = serde_json::to_string(&vec![record("foreign-id", "cn")]).unwrap();
    export_import::merge_import_records(&mut accounts, &text, &[0]).unwrap();
    assert_eq!(accounts[0]["id"], "local");
}

#[test]
fn capability_credentials_are_independent_and_not_implicitly_reused() {
    let f = Fixture::new();
    let mut entry = record("a", "cn");
    entry["capability_credentials"] = json!({
        "Checkin": {"access_token": "FABRICATED-CHECKIN-SECRET"},
        "PointsRead": {"access_token": "FABRICATED-POINTS-SECRET"},
    });
    f.repo().save(&[entry]).unwrap();
    let loaded = f.repo().load().unwrap();
    assert_eq!(loaded[0]["access_token"], "FABRICATED-ACCESS-SECRET");
    assert_eq!(
        loaded[0]["capability_credentials"]["Checkin"]["access_token"],
        "FABRICATED-CHECKIN-SECRET"
    );
    let meta = account::account_meta(&loaded[0]);
    assert_eq!(
        meta["capabilities"][1]["authorization_status"],
        "Unverified"
    );
    assert!(!meta.to_string().contains("FABRICATED-"));
    assert!(!f.read().contains("FABRICATED-"));
}

#[test]
fn public_export_redacts_unknown_fields_nested_tokens_and_remote_error_echo() {
    let f = Fixture::new();
    let mut entry = record("a", "cn");
    entry["needs_relogin"] = json!(true);
    entry["needs_relogin_reason"] = json!("gateway echoed FABRICATED-ACCESS-SECRET");
    let exported = export_import::select_export_records(&[entry.clone()], &["a".into()]).unwrap();
    assert!(!serde_json::to_string(&exported)
        .unwrap()
        .contains("FABRICATED-"));
    assert!(exported[0].get("credential_references").is_none());
    let path = export_import::write_records_to_file(&f.0, &[entry], "metadata.json").unwrap();
    assert!(!std::fs::read_to_string(path)
        .unwrap()
        .contains("FABRICATED-"));
}

#[test]
fn expired_credentials_require_authorization_and_api_capabilities_remain_unverified() {
    let mut entry = record("a", "cn");
    entry["expiresAt"] = json!(1);
    assert_eq!(capabilities(&entry)[0]["authorization_status"], "Expired");
    assert_eq!(
        capabilities(&entry)[1]["authorization_status"],
        "Unverified"
    );
    assert_eq!(
        capabilities(&entry)[2]["authorization_status"],
        "Unverified"
    );
}

#[test]
fn points_capability_requires_verified_host_and_available_desktop_credential() {
    let mut entry = record("a", "cn");
    entry["capability_credentials"]["PointsRead"] = json!({
        "authorized": true,
        "verified_for": "www.codebuddy.cn/billing/meter",
        "verified_at_ms": 123,
    });
    assert_eq!(capabilities(&entry)[2]["authorization_status"], "Verified");
    assert_eq!(capabilities(&entry)[2]["last_verified_at"], 123);
    entry["capability_credentials"]["PointsRead"]["verified_for"] = json!("example.invalid");
    assert_eq!(
        capabilities(&entry)[2]["authorization_status"],
        "Unverified"
    );
}

#[test]
fn failed_credential_write_preserves_original_account_index() {
    let f = Fixture::new();
    let original = serde_json::to_string(&vec![record("a", "cn")]).unwrap();
    std::fs::write(f.path(), &original).unwrap();
    std::fs::write(f.0.join("credentials"), "block-vault-directory").unwrap();
    assert!(f.repo().save(&[record("a", "cn")]).is_err());
    assert_eq!(f.read(), original);
}

#[test]
fn account_update_lock_prevents_concurrent_writes_and_releases_on_failure() {
    let f = Fixture::new();
    f.repo().save(&[record("a", "cn")]).unwrap();
    let before = f.read();
    f.repo()
        .update(|_| {
            assert_eq!(
                f.repo().save(&[]).unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
            Err::<(), _>(std::io::Error::other("fixture failure"))
        })
        .unwrap_err();
    assert_eq!(f.read(), before);
    f.repo().save(&[]).unwrap();
    assert!(f.repo().load().unwrap().is_empty());
}

#[test]
fn stale_refresh_is_rejected_after_new_credentials_are_saved() {
    let f = Fixture::new();
    f.repo().save(&[record("a", "cn")]).unwrap();
    let stale = f.repo().load().unwrap()[0].clone();
    f.repo()
        .update(|accounts| {
            accounts[0]["access_token"] = json!("newer-fabricated-token");
            Ok(())
        })
        .unwrap();
    let current = f.repo().load().unwrap();
    assert!(account::reject_stale_credential_update(&current, &stale).is_err());
    assert!(account::reject_stale_credential_update(&current, &current[0]).is_ok());
}

#[test]
fn invalid_expired_disabled_or_wrong_provider_credentials_are_rejected_before_switching() {
    for patch in [
        json!({"credential_locked":true}),
        json!({"needs_relogin":true}),
        json!({"expiresAt":1}),
        json!({"is_enabled":false}),
        json!({"provider":"other"}),
        json!({"access_token":""}),
    ] {
        let mut entry = record("a", "cn");
        entry
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(account::desktop_credential_error(&entry).is_some());
    }
    assert!(account::desktop_credential_error(&record("a", "cn")).is_none());
}

#[test]
fn enabled_state_is_validated_atomically_and_survives_reimport() {
    let mut accounts = vec![record("a", "cn")];
    let before = accounts.clone();
    assert!(account::update_account_display_in(
        &mut accounts,
        "a",
        &json!({"note":"partial", "is_enabled":"false"})
    )
    .is_err());
    assert_eq!(accounts, before);
    account::update_account_display_in(&mut accounts, "a", &json!({"is_enabled":false})).unwrap();
    account::upsert_collected_account(&mut accounts, record("new-id", "cn"));
    assert_eq!(accounts[0]["is_enabled"], false);
}

#[test]
fn crud_on_same_name_accounts_preserves_other_accounts_and_credentials() {
    let f = Fixture::new();
    let mut accounts = vec![];
    account::upsert_collected_account(&mut accounts, record("cn", "cn"));
    account::upsert_collected_account(&mut accounts, record("ai", "ai"));
    f.repo().save(&accounts).unwrap();
    f.repo()
        .update(|accounts| {
            account::update_account_display_in(accounts, "cn", &json!({"note":"first"})).unwrap();
            Ok(())
        })
        .unwrap();
    let loaded = f.repo().load().unwrap();
    assert_eq!(loaded[0]["note"], "first");
    assert!(loaded[1].get("note").is_none());
    f.repo()
        .update(|accounts| {
            accounts.retain(|v| v["id"] != "cn");
            Ok(())
        })
        .unwrap();
    assert_eq!(f.repo().load().unwrap()[0]["id"], "ai");
}

#[tokio::test]
async fn unverified_checkin_and_points_are_gated_before_refresh_or_request() {
    use wb_switch_core::modules::{checkin, credits};
    let mut expired = record("gated", "cn");
    expired["expiresAt"] = json!(1);
    assert_eq!(
        checkin::checkin_account(&expired).await["reason"],
        "authorization_unverified"
    );
    assert_eq!(
        credits::get_credit_expiry(&expired).await["reason"],
        "authorization_unverified"
    );
    assert_eq!(checkin::get_checkin_status(&expired).await["ok"], false);
    assert_eq!(checkin::perform_checkin(&expired).await["ok"], false);
}
