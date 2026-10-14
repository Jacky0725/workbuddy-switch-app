//! Copies referenced checkpoint backups without overwriting existing target files.
use super::{session::SessionPaths, session_backup};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_FILES: usize = 4096;
const MAX_BYTES: u64 = 256 * 1024 * 1024;
const MANIFEST: &str = "history-resources.json";

#[derive(Clone, Default)]
pub(crate) struct HistoryBundle(Vec<(String, Vec<u8>)>);

impl HistoryBundle {
    pub(crate) fn identity(&self) -> Option<String> {
        if self.0.is_empty() {
            return None;
        }
        let mut hash = Sha256::new();
        for (name, bytes) in &self.0 {
            hash.update((name.len() as u64).to_le_bytes());
            hash.update(name.as_bytes());
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(Sha256::digest(bytes));
        }
        Some(format!("{:x}", hash.finalize()))
    }
}

#[derive(Serialize, Deserialize)]
struct Entry {
    name: String,
    digest: String,
    size: u64,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn segment(name: &str) -> Result<(), String> {
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    if name.is_empty()
        || name.len() > 180
        || name == "."
        || name == ".."
        || name.ends_with('.')
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._@-".contains(&b))
        || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit())
    {
        return Err("文件历史资源标识不合法，已停止写入".into());
    }
    Ok(())
}

pub(crate) fn names(text: &str) -> Result<BTreeSet<String>, String> {
    let mut names = BTreeSet::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|_| "会话正文无法解析".to_string())?;
        if value["type"] != "file-history-snapshot" {
            continue;
        }
        let Some(map) = value
            .get("snapshot")
            .and_then(|v| v.get("trackedFileBackups"))
        else {
            continue;
        };
        let Some(map) = map.as_object() else {
            return Err("文件历史引用格式不支持，已停止写入".into());
        };
        for entry in map.values() {
            let Some(entry) = entry.as_object() else {
                return Err("文件历史引用格式不支持，已停止写入".into());
            };
            let Some(name) = entry.get("backupFileName") else {
                continue;
            };
            let Some(name) = name.as_str() else {
                return Err("文件历史备份名称无法确认，已停止写入".into());
            };
            segment(name)?;
            if !entry
                .get("version")
                .and_then(|v| v.as_u64())
                .is_some_and(|v| v > 0)
            {
                return Err("文件历史备份版本无法确认，已停止写入".into());
            }
            names.insert(name.to_string());
            if names.len() > MAX_FILES {
                return Err("文件历史资源过多，已停止写入".into());
            }
        }
    }
    Ok(names)
}

fn plain(path: &Path, directory: bool) -> Result<(), String> {
    let meta =
        std::fs::symlink_metadata(path).map_err(|_| "文件历史资源缺失或无法读取".to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("文件历史资源含重解析路径，已停止写入".into());
        }
    }
    if meta.file_type().is_symlink()
        || (directory && !meta.is_dir())
        || (!directory && !meta.is_file())
    {
        return Err("文件历史资源路径类型不支持，已停止写入".into());
    }
    Ok(())
}

fn present(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err("文件历史资源路径无法检查".into()),
    }
}

fn read_file(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    plain(path, false)?;
    let len = std::fs::metadata(path)
        .map_err(|_| "文件历史资源无法读取".to_string())?
        .len();
    if len > limit {
        return Err("文件历史资源过大，已停止写入".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| "文件历史资源无法读取".to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "文件历史资源无法读取".to_string())?;
    if bytes.len() as u64 != len {
        return Err("文件历史资源读取时发生变化，已停止写入".into());
    }
    Ok(bytes)
}

fn history_dir(paths: &SessionPaths, sid: &str, create: bool) -> Result<PathBuf, String> {
    segment(sid)?;
    let root = paths
        .data_root
        .canonicalize()
        .map_err(|_| "会话数据目录无法确认".to_string())?;
    let parent = root.join("file-history");
    if create && !present(&parent)? {
        std::fs::create_dir(&parent).map_err(|_| "文件历史目录创建失败".to_string())?;
        session_backup::sync_dir(&root).map_err(|_| "文件历史目录持久化失败".to_string())?;
    }
    plain(&parent, true)?;
    let dir = parent.join(sid);
    if create && !present(&dir)? {
        std::fs::create_dir(&dir).map_err(|_| "目标文件历史目录创建失败".to_string())?;
        session_backup::sync_dir(&parent).map_err(|_| "文件历史目录持久化失败".to_string())?;
    }
    plain(&dir, true)?;
    if dir
        .canonicalize()
        .map_err(|_| "文件历史目录无法确认".to_string())?
        .parent()
        != Some(parent.as_path())
    {
        return Err("文件历史资源不在预期目录内，已停止写入".into());
    }
    Ok(dir)
}

pub(crate) fn snapshot(
    paths: &SessionPaths,
    sid: &str,
    text: &str,
) -> Result<HistoryBundle, String> {
    let names = names(text)?;
    if names.is_empty() {
        return Ok(HistoryBundle::default());
    }
    let dir = history_dir(paths, sid, false)?;
    let mut bundle = HistoryBundle::default();
    let mut total = 0_u64;
    for name in names {
        let path = dir.join(&name);
        plain(&path, false)?;
        let len = std::fs::metadata(&path)
            .map_err(|_| "文件历史资源无法读取".to_string())?
            .len();
        total = total
            .checked_add(len)
            .filter(|v| *v <= MAX_BYTES)
            .ok_or("文件历史资源过大，已停止写入")?;
        let bytes = read_file(&path, len)?;
        if bytes.len() as u64 != len {
            return Err("文件历史资源读取时发生变化，已停止写入".into());
        }
        bundle.0.push((name, bytes));
    }
    Ok(bundle)
}

pub(crate) fn install(
    paths: &SessionPaths,
    sid: &str,
    bundle: &HistoryBundle,
) -> Result<(), String> {
    if bundle.0.is_empty() {
        return Ok(());
    }
    preflight_install(paths, sid, bundle)?;
    let dir = history_dir(paths, sid, true)?;
    // Check the complete batch before writing any files. Never replace a different backup.
    for (name, bytes) in &bundle.0 {
        segment(name)?;
        let path = dir.join(name);
        if present(&path)? {
            plain(&path, false)?;
            if read_file(&path, bytes.len() as u64)? != *bytes {
                return Err("目标文件历史备份同名但内容不同，已停止写入".into());
            }
        }
    }
    for (name, bytes) in &bundle.0 {
        let dest = dir.join(name);
        if !present(&dest)? {
            let temporary = dir.join(format!(".history-{}", uuid::Uuid::new_v4()));
            session_backup::durable_write(&temporary, bytes)
                .map_err(|_| "文件历史资源暂存失败".to_string())?;
            // Creating a hard link cannot overwrite a racing destination.
            let result = std::fs::hard_link(&temporary, &dest);
            let _ = std::fs::remove_file(&temporary);
            result.map_err(|_| "文件历史资源保存失败，保留操作供恢复".to_string())?;
            session_backup::sync_dir(&dir).map_err(|_| "文件历史资源持久化失败".to_string())?;
        }
        plain(&dest, false)?;
        if read_file(&dest, bytes.len() as u64)? != *bytes {
            return Err("文件历史资源写后校验失败".into());
        }
    }
    Ok(())
}

pub(crate) fn preflight_install(
    paths: &SessionPaths,
    sid: &str,
    bundle: &HistoryBundle,
) -> Result<(), String> {
    if bundle.0.is_empty() {
        return Ok(());
    }
    segment(sid)?;
    let root = paths
        .data_root
        .canonicalize()
        .map_err(|_| "会话数据目录无法确认".to_string())?;
    let parent = root.join("file-history");
    if !present(&parent)? {
        return Ok(());
    }
    plain(&parent, true)?;
    let dir = parent.join(sid);
    if !present(&dir)? {
        return Ok(());
    }
    plain(&dir, true)?;
    for (name, bytes) in &bundle.0 {
        segment(name)?;
        let dest = dir.join(name);
        if present(&dest)? {
            plain(&dest, false)?;
            if read_file(&dest, bytes.len() as u64)? != *bytes {
                return Err("目标文件历史备份同名但内容不同，已停止写入".into());
            }
        }
    }
    Ok(())
}

pub(crate) fn save(dir: &Path, bundle: &HistoryBundle) -> Result<(), String> {
    if bundle.0.is_empty() {
        return Ok(());
    }
    let files = dir.join("history-resources");
    std::fs::create_dir(&files).map_err(|_| "文件历史备份目录创建失败".to_string())?;
    let mut entries = Vec::new();
    for (name, bytes) in &bundle.0 {
        session_backup::durable_write(&files.join(name), bytes)
            .map_err(|_| "文件历史资源备份失败".to_string())?;
        entries.push(Entry {
            name: name.clone(),
            digest: digest(bytes),
            size: bytes.len() as u64,
        });
    }
    let bytes = serde_json::to_vec(&entries).map_err(|_| "文件历史备份清单创建失败".to_string())?;
    session_backup::durable_write(&dir.join(MANIFEST), &bytes)
        .map_err(|_| "文件历史备份清单保存失败".to_string())
}

pub(crate) fn load(dir: &Path, text: &str) -> Result<HistoryBundle, String> {
    let expected = names(text)?;
    if expected.is_empty() {
        return Ok(HistoryBundle::default());
    }
    plain(&dir.join(MANIFEST), false)?;
    let entries: Vec<Entry> = serde_json::from_slice(&read_file(&dir.join(MANIFEST), 1024 * 1024)?)
        .map_err(|_| "文件历史备份清单无法解析".to_string())?;
    if entries.len() != expected.len()
        || entries
            .iter()
            .map(|e| e.name.clone())
            .collect::<BTreeSet<_>>()
            != expected
    {
        return Err("文件历史备份清单与正文不一致".into());
    }
    let files = dir.join("history-resources");
    plain(&files, true)?;
    let mut bundle = HistoryBundle::default();
    let mut total = 0_u64;
    for entry in entries {
        segment(&entry.name)?;
        total = total
            .checked_add(entry.size)
            .filter(|v| *v <= MAX_BYTES)
            .ok_or("文件历史资源过大")?;
        let path = files.join(&entry.name);
        plain(&path, false)?;
        if std::fs::metadata(&path)
            .map_err(|_| "文件历史备份资源无法读取".to_string())?
            .len()
            != entry.size
        {
            return Err("文件历史备份资源大小不一致".into());
        }
        let bytes = read_file(&path, entry.size)?;
        if digest(&bytes) != entry.digest {
            return Err("文件历史备份资源摘要不一致".into());
        }
        bundle.0.push((entry.name, bytes));
    }
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn body(name: &str) -> String {
        serde_json::json!({"type":"file-history-snapshot","snapshot":{"trackedFileBackups":{"fiction.txt":{"backupFileName":name,"version":1}}}}).to_string()
    }
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("wbm-history-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn history_names_reject_traversal_reserved_and_unknown_shapes() {
        for name in [
            "../outside",
            "a/b",
            "a\\b",
            "a:stream",
            "CON",
            "LPT1.txt",
            "abc.",
            "",
        ] {
            assert!(names(&body(name)).is_err(), "{name}");
        }
        assert_eq!(names(&body("123abc@v1")).unwrap().len(), 1);
        assert!(
            names(r#"{"type":"file-history-snapshot","snapshot":{"trackedFileBackups":[1]}}"#)
                .is_err()
        );
        assert!(names(r#"{"type":"file-history-snapshot","snapshot":{"trackedFileBackups":{"a":{"backupFileName":"abc","version":0}}}}"#).is_err());
    }

    #[test]
    fn history_backup_rejects_tampering_and_directory_in_place_of_file() {
        let f = Fixture::new();
        let bytes = b"fictional bytes".to_vec();
        let bundle = HistoryBundle(vec![("abc@v1".into(), bytes.clone())]);
        save(&f.0, &bundle).unwrap();
        assert_eq!(
            load(&f.0, &body("abc@v1")).unwrap().identity(),
            bundle.identity()
        );
        let p = f.0.join("history-resources/abc@v1");
        std::fs::write(&p, b"tampered bytes!").unwrap();
        assert!(load(&f.0, &body("abc@v1")).is_err());
        std::fs::remove_file(&p).unwrap();
        std::fs::create_dir(&p).unwrap();
        assert!(load(&f.0, &body("abc@v1")).is_err());
    }

    #[test]
    fn history_read_is_bounded_before_loading_bytes() {
        let f = Fixture::new();
        let p = f.0.join("large");
        std::fs::write(&p, b"12345").unwrap();
        assert!(read_file(&p, 4).is_err());
        assert_eq!(read_file(&p, 5).unwrap(), b"12345");
    }
}
