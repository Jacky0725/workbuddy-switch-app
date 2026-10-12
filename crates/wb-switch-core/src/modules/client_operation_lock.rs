//! Serializes operations that change or depend on the active WorkBuddy identity.
//! The lease spans process shutdown, auth-file changes and future UI check-in.

use fs2::FileExt;
use std::{fs::File, fs::OpenOptions, path::Path};

pub struct ClientOperationLease {
    _file: File,
}

impl ClientOperationLease {
    pub fn acquire(root: &Path) -> Result<Self, String> {
        let directory = root.join("locks");
        std::fs::create_dir_all(&directory)
            .map_err(|_| "无法创建 WorkBuddy 操作锁目录，已取消操作".to_string())?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("client_operation.lock"))
            .map_err(|_| "无法建立 WorkBuddy 操作锁，已取消操作".to_string())?;
        file.try_lock_exclusive()
            .map_err(|_| "另一项账号切换或签到任务正在操作 WorkBuddy，请稍后重试".to_string())?;
        Ok(Self { _file: file })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_operation_is_rejected_until_first_releases_lock() {
        let root = std::env::temp_dir().join(format!(
            "workbuddy-client-operation-test-{}",
            uuid::Uuid::new_v4()
        ));
        let first = ClientOperationLease::acquire(&root).unwrap();
        assert!(ClientOperationLease::acquire(&root).is_err());
        drop(first);
        assert!(ClientOperationLease::acquire(&root).is_ok());
        std::fs::remove_dir_all(root).unwrap();
    }
}
