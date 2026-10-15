//! Same-machine shared attachment and immutable Buddy snapshot validation.
use super::session::SessionPaths;
use rusqlite::{types::Value as SqlValue, Connection, OpenFlags, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read, path::PathBuf};

const MAX_FILE: u64 = 64 * 1024 * 1024;
const MAX_TOTAL: u64 = 256 * 1024 * 1024;
const MAX_REFERENCES: usize = 128;
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn sha_id(id: &str) -> bool {
    id.len() == 64
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn local_file(path: &str) -> Result<(PathBuf, u64, String), String> {
    let path = PathBuf::from(path);
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("附件路径含上级目录，已停止写入".into());
    }
    if !path.is_absolute() {
        return Err("附件路径不是绝对本地路径，已停止写入".into());
    }
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        if !matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        {
            return Err("网络或设备附件路径暂不支持迁移".into());
        }
    }
    for parent in path.ancestors() {
        let meta = std::fs::symlink_metadata(parent)
            .map_err(|_| "附件或其目录缺失，已停止写入".to_string())?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err("附件路径含重解析链接，已停止写入".into());
            }
        }
        if meta.file_type().is_symlink() {
            return Err("附件路径含符号链接，已停止写入".into());
        }
    }
    let path = path
        .canonicalize()
        .map_err(|_| "附件路径无法确认".to_string())?;
    let mut file = std::fs::File::open(&path).map_err(|_| "附件无法读取".to_string())?;
    let before = file
        .metadata()
        .map_err(|_| "附件状态无法读取".to_string())?;
    if !before.is_file() || before.len() > MAX_FILE {
        return Err("附件类型或大小暂不支持，已停止写入".into());
    }
    let mut digest = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 65536];
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|_| "附件读取失败".to_string())?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > MAX_FILE {
            return Err("附件读取时变大，已停止写入".into());
        }
        digest.update(&buffer[..n]);
    }
    let after = file
        .metadata()
        .map_err(|_| "附件状态无法读取".to_string())?;
    if size != before.len()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
    {
        return Err("附件读取时发生变化，请重新检查".into());
    }
    Ok((path, size, format!("{:x}", digest.finalize())))
}

fn unescape_path(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

pub(crate) fn attachment_identity(
    paths: &SessionPaths,
    text: &str,
) -> Result<Option<String>, String> {
    let mut refs: BTreeMap<String, (u64, String)> = BTreeMap::new();
    let attached = regex::Regex::new(r"(?s)<attached_files>(.*?)</attached_files>").unwrap();
    let file_tag = regex::Regex::new(r#"<file\s+path="([^"]+)"\s*/?>"#).unwrap();
    let image_tag = regex::Regex::new(r"(?s)<image_local_path>(.*?)</image_local_path>").unwrap();
    fn add(
        refs: &mut BTreeMap<String, (u64, String)>,
        raw: &str,
    ) -> Result<(PathBuf, u64, String), String> {
        let (path, size, digest) = local_file(raw)?;
        refs.insert(path.to_string_lossy().to_string(), (size, digest.clone()));
        if refs.len() > MAX_REFERENCES || refs.values().map(|x| x.0).sum::<u64>() > MAX_TOTAL {
            return Err("会话附件数量或总大小超限".into());
        }
        Ok((path, size, digest))
    }
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let record: Value =
            serde_json::from_str(line).map_err(|_| "会话正文无法解析".to_string())?;
        let Some(content) = record.get("content").and_then(Value::as_array) else {
            continue;
        };
        for item in content {
            match item.get("type").and_then(Value::as_str) {
                Some("input_image") => {
                    let image = &item["image"];
                    if image["type"] != "image_blob_ref" {
                        return Err("尚未验证的图片附件格式，已停止写入".into());
                    }
                    let id = image["blob_id"]
                        .as_str()
                        .filter(|s| sha_id(s))
                        .ok_or("图片附件内容标识无效")?;
                    let raw = image["blob_path"]
                        .as_str()
                        .ok_or("图片附件缺少本地资源路径")?;
                    let (path, size, digest) = add(&mut refs, raw)?;
                    let parent = paths
                        .data_root
                        .join("blobs")
                        .join(&id[..2])
                        .canonicalize()
                        .map_err(|_| "图片附件库缺失".to_string())?;
                    if path.parent() != Some(parent.as_path())
                        || path.file_stem().and_then(|s| s.to_str()) != Some(id)
                        || digest != id
                        || image["size"].as_u64() != Some(size)
                        || !image["mime"]
                            .as_str()
                            .is_some_and(|s| s.starts_with("image/"))
                    {
                        return Err("图片附件的路径、大小或内容摘要不一致".into());
                    }
                }
                Some("input_file" | "image" | "image_url" | "file" | "document") => {
                    return Err("该附件格式尚未验证，已停止写入".into())
                }
                Some("input_text" | "text") => {
                    let Some(t) = item.get("text").and_then(Value::as_str) else {
                        continue;
                    };
                    for block in attached.captures_iter(t) {
                        for entry in file_tag.captures_iter(&block[1]) {
                            add(&mut refs, &unescape_path(&entry[1]))?;
                        }
                    }
                    for entry in image_tag.captures_iter(t) {
                        add(&mut refs, entry[1].trim())?;
                    }
                }
                _ => {}
            }
        }
    }
    if refs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(hash(
            &serde_json::to_vec(&refs).map_err(|_| "附件摘要生成失败".to_string())?,
        )))
    }
}

fn field<'a>(row: &'a [(String, SqlValue)], name: &str) -> Result<Option<&'a str>, String> {
    match row.iter().find(|x| x.0 == name).map(|x| &x.1) {
        None | Some(SqlValue::Null) => Ok(None),
        Some(SqlValue::Text(s)) if s.is_empty() => Ok(None),
        Some(SqlValue::Text(s)) => Ok(Some(s)),
        _ => Err("Buddy 引用字段类型无效".into()),
    }
}

pub(crate) fn buddy_reference(
    row: &[(String, SqlValue)],
) -> Result<Option<(String, String)>, String> {
    let (id, raw) = (
        field(row, "buddy_snapshot_id")?,
        field(row, "buddy_binding_json")?,
    );
    if id.is_none() && raw.is_none() {
        return Ok(None);
    }
    let (Some(id), Some(raw)) = (id, raw) else {
        return Err("Buddy 绑定与快照引用不完整".into());
    };
    if !id.strip_prefix("bs_").is_some_and(sha_id) || raw.len() > 131072 {
        return Err("Buddy 绑定标识或大小无效".into());
    }
    let binding: Value = serde_json::from_str(raw).map_err(|_| "Buddy 绑定无法解析".to_string())?;
    if binding["version"] != 1
        || binding["snapshotId"] != id
        || binding["resources"]["resourceSnapshotKey"] != id
    {
        return Err("Buddy 绑定与资源快照不一致".into());
    }
    for key in [
        "selection",
        "runtime",
        "presentation",
        "continuation",
        "resources",
    ] {
        if !binding[key].is_object() {
            return Err("Buddy 绑定结构未通过验证".into());
        }
    }
    if let Some(mode) = binding["runtime"].get("industryMode") {
        if !mode.is_object()
            || mode["resourceSnapshotKey"] != id
            || !mode["applicationId"]
                .as_str()
                .is_some_and(|s| !s.is_empty())
        {
            return Err("Buddy 运行模式引用不一致".into());
        }
    }
    Ok(Some((id.into(), raw.into())))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BuddyIdentity<'a> {
    application_id: &'a str,
    template_id: &'a str,
    schema_version: i64,
    template_version: i64,
}

pub(crate) fn validate_buddy_row(
    db: &Connection,
    row: &[(String, SqlValue)],
) -> Result<Option<String>, String> {
    let Some((id, binding)) = buddy_reference(row)? else {
        return Ok(None);
    };
    let snapshot=db.query_row("SELECT application_id,template_id,schema_version,template_version,snapshot_format_version,config_json,config_digest FROM buddy_snapshots WHERE snapshot_id=?1",[&id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?,r.get::<_,String>(5)?,r.get::<_,String>(6)?)))
        .optional().map_err(|_|"Buddy 快照无法读取".to_string())?.ok_or("Buddy 快照缺失，已停止写入")?;
    let (app, template, schema, version, format, config, digest) = snapshot;
    if format != 1
        || schema < 0
        || version < 0
        || app.is_empty()
        || template.is_empty()
        || config.len() > 2 * 1024 * 1024
    {
        return Err("Buddy 快照版本或大小尚未验证".into());
    }
    let parsed: Value =
        serde_json::from_str(&config).map_err(|_| "Buddy 配置快照无法解析".to_string())?;
    if !parsed.is_object() {
        return Err("Buddy 配置快照格式无效".into());
    }
    let identity = serde_json::to_string(&BuddyIdentity {
        application_id: &app,
        template_id: &template,
        schema_version: schema,
        template_version: version,
    })
    .map_err(|_| "Buddy 快照身份无法生成".to_string())?;
    let calculated = hash(format!("{{\"config\":{config},\"identity\":{identity}}}").as_bytes());
    if calculated != digest || id != format!("bs_{digest}") {
        return Err("Buddy 配置快照摘要不一致，已停止写入".into());
    }
    let b: Value = serde_json::from_str(&binding).unwrap();
    if b["runtime"]
        .get("industryMode")
        .is_some_and(|m| m["applicationId"] != app)
    {
        return Err("Buddy 运行模式与应用身份不一致".into());
    }
    Ok(Some(hash(format!("{digest}\n{binding}").as_bytes())))
}

pub(crate) fn buddy_identity(paths: &SessionPaths, sid: &str) -> Result<Option<String>, String> {
    let db = Connection::open_with_flags(paths.workbuddy_db(), OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| "会话数据库无法只读打开".to_string())?;
    let mut stmt = db
        .prepare("PRAGMA table_info(sessions)")
        .map_err(|_| "会话表结构无法检查".to_string())?;
    let columns = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .map_err(|_| "会话表结构无法检查".to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "会话表结构无法检查".to_string())?;
    if !columns
        .iter()
        .any(|c| c == "buddy_snapshot_id" || c == "buddy_binding_json")
    {
        return Ok(None);
    }
    let row = db
        .query_row(
            "SELECT buddy_snapshot_id,buddy_binding_json FROM sessions WHERE id=?1",
            [sid],
            |r| {
                Ok(vec![
                    ("buddy_snapshot_id".into(), r.get::<_, SqlValue>(0)?),
                    ("buddy_binding_json".into(), r.get::<_, SqlValue>(1)?),
                ])
            },
        )
        .optional()
        .map_err(|_| "Buddy 会话引用无法读取".to_string())?;
    match row {
        Some(row) => validate_buddy_row(&db, &row),
        None => Ok(None),
    }
}

pub(crate) fn require_same_buddy(
    source: &SessionPaths,
    source_id: &str,
    target: &SessionPaths,
    target_id: &str,
) -> Result<(), String> {
    if buddy_identity(source, source_id)? != buddy_identity(target, target_id)? {
        return Err("双方 Buddy 配置或绑定不同，已停止同步".into());
    }
    Ok(())
}
