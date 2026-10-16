//! One finite ACP worker, using the verified official client's own Node runtime.
use super::{account, credential_store::durable_write};
#[cfg(target_os = "windows")]
use super::{process, variant::WbVariant};
use serde::Deserialize;
use serde_json::{json, Value};
#[cfg(target_os = "windows")]
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[cfg(target_os = "windows")]
const BUNDLE_HASH: &str = "1b1a01bc5f7b1121931b594770167ec6738b73b90b9d7d934d9aa9cab79db433";
const BRIDGE: &str = include_str!("growth_acp.cjs");
#[derive(Deserialize)]
struct Reply {
    #[serde(default)]
    submitted: bool,
    #[serde(default)]
    finished: bool,
    #[serde(default)]
    reply_matches: bool,
    #[serde(default)]
    initialized: bool,
    error: Option<String>,
}
pub(crate) fn compatible() -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    #[cfg(not(target_os = "windows"))]
    return Err("真实成长任务目前仅验证 Windows WorkBuddy 5.8.2".into());
    #[cfg(target_os = "windows")]
    {
        let exe = process::windows_workbuddy_exe_path(WbVariant::Cn).ok_or("未找到官方客户端")?;
        let cli = exe
            .parent()
            .ok_or("官方安装路径无效")?
            .join("resources/app.asar.unpacked/cli");
        let bundle = cli.join("dist/codebuddy-lite-wb.mjs");
        let product_file = cli.join("product.json");
        if std::fs::metadata(&product_file)
            .map_err(|_| "官方产品配置不存在")?
            .len()
            > 2 * 1024 * 1024
        {
            return Err("官方产品配置超限".into());
        }
        let product: Value = serde_json::from_slice(
            &std::fs::read(product_file).map_err(|_| "官方产品配置不可读取")?,
        )
        .map_err(|_| "官方产品配置无法识别")?;
        if product["endpoint"] != "https://www.workbuddy.cn"
            || product["authentication"]["attributes"]["platform"] != "workbuddy"
        {
            return Err("此官方运行时的服务端配置尚未验证".into());
        }
        if std::fs::metadata(&bundle)
            .map_err(|_| "官方运行时不存在")?
            .len()
            > 32 * 1024 * 1024
        {
            return Err("官方运行时文件超限".into());
        }
        let digest = format!(
            "{:x}",
            Sha256::digest(std::fs::read(&bundle).map_err(|_| "官方运行时不可读取")?)
        );
        if digest != BUNDLE_HASH {
            return Err("此官方客户端运行时尚未验证，禁止自动发送对话".into());
        }
        Ok((exe, cli.join("bin/codebuddy")))
    }
}
pub(crate) async fn chat(root: &Path, a: &Value, preflight: bool) -> Result<(), String> {
    let root = root.to_path_buf();
    let a = a.clone();
    tokio::task::spawn_blocking(move || run(&root, &a, preflight))
        .await
        .map_err(|_| "官方运行时工作进程中断，等待核实".to_string())?
}
fn run(root: &Path, a: &Value, preflight: bool) -> Result<(), String> {
    let _lease = super::client_operation_lock::ClientOperationLease::acquire(root)?;
    let (exe, cli) = compatible()?;
    let id = a["id"].as_str().ok_or("账号标识缺失")?;
    let current = account::find_account(id).ok_or("账号已移除")?;
    if super::growth::identity(&current) != super::growth::identity(a)
        || current["access_token"] != a["access_token"]
    {
        return Err("账号已变化，已停止调用".into());
    }
    if let Some(error) = super::growth::query_error(&current) {
        return Err(error.into());
    }
    let directory = root.join("growth-runtime");
    std::fs::create_dir_all(&directory).map_err(|_| "运行时目录不可创建")?;
    let bridge = directory.join("acp-worker.cjs");
    // Runs under the same growth lock as the operation journal.
    durable_write(&bridge, BRIDGE).map_err(|_| "运行时脚本不可保存")?;
    let home = directory.join(super::growth::identity(a));
    let mut command = Command::new(&exe);
    command
        .arg(&bridge)
        .env("ELECTRON_RUN_AS_NODE", "1")
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_PATH")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|_| "官方运行时不可启动")?;
    let mut stdin = child.stdin.take().ok_or("运行时输入不可用")?;
    let input = json!({"executable":exe,"cli":cli,"home":home,"token":a["access_token"],"uid":a["uid"],"preflight":preflight});
    if writeln!(stdin, "{input}").is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err("运行时输入失败，等待核实".into());
    }
    let start = Instant::now();
    while child.try_wait().map_err(|_| "运行时状态未知")?.is_none() {
        if start.elapsed() > Duration::from_secs(75) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("运行时超时，等待核实，禁止重发".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    drop(stdin);
    let output = child.wait_with_output().map_err(|_| "运行时结果未知")?;
    if !output.status.success() || output.stdout.len() > 4096 {
        return Err("运行时异常结束，等待核实".into());
    }
    let result: Reply =
        serde_json::from_slice(&output.stdout).map_err(|_| "运行时返回无法识别，等待核实")?;
    if result.error.is_some() {
        return Err("官方运行时未确认完成，等待任务进度核实".into());
    }
    if preflight && result.initialized && !result.submitted
        || !preflight && result.submitted && result.finished && result.reply_matches
    {
        Ok(())
    } else {
        Err("官方运行时未确认预期回复，等待任务进度核实".into())
    }
}
