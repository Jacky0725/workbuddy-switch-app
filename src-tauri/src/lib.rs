// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod companion;
#[cfg(target_os = "macos")]
mod instance_lock;
#[cfg(desktop)]
mod tray;
mod update_service;
#[cfg(target_os = "windows")]
mod windows_autostart;

use tauri::Emitter;
use wb_switch_core::modules;

const SCREENSHOT_DEMO_ENV: &str = "WB_SWITCH_SCREENSHOT_DEMO";

pub(crate) fn is_screenshot_demo() -> bool {
    std::env::var(SCREENSHOT_DEMO_ENV).as_deref() == Ok("1")
}

/// 轮换推迟提示：桌面端先向前端推 `rotate-deferred`（应用内提示，窗口开着就能看到），
/// 再尽力投递系统通知（应用在托盘/后台时可见）。
///
/// 应用内提示不依赖系统通知权限：插件在开发态会把通知登记到「终端」名下，且投递失败
/// 无法观测（`show()` 恒返回 Ok），所以两者都发、以前者为准。
/// 其它形态由 core 的日志与 `notify` 返回字段承载，宿主不投递。
pub(crate) fn deliver_rotate_notify(app: &tauri::AppHandle, result: &serde_json::Value) {
    #[cfg(desktop)]
    {
        if let Some(notify) = result.get("notify") {
            let _ = app.emit("rotate-deferred", notify.clone());
            tray::notify_rotate_deferred(app, notify);
        }
    }
    #[cfg(not(desktop))]
    {
        let _ = (app, result);
    }
}

/// 广播「CodeBuddy CLI 认证状态可能已变」，让前端立即重读。
///
/// 保活刷新会先批量改写账号库里的 token、再异步同步回 `settings.json`；这个窗口里
/// 状态接口会短暂读到「账号库已换新、settings 未跟上」。刷新前后各广播一次，
/// 前端就能把旧判断及时收敛，而不是等下一次页面重挂载。
pub(crate) fn notify_codebuddy_cli_updated(app: &tauri::AppHandle) {
    #[cfg(desktop)]
    {
        let _ = app.emit("codebuddy-cli-updated", ());
    }
    #[cfg(not(desktop))]
    {
        let _ = app;
    }
}

/// 每日签到由统一服务轮询配置，授权未验证时仅记录后台心跳。
/// 未验证的上游后台网络与自动安装任务不在启动时运行。
fn spawn_background_loops(_app: tauri::AppHandle) {
    // No automatic travel, rotation, refresh, hook installation or updater requests.
    tauri::async_runtime::spawn(modules::manager_scheduler::background_loop());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // 单实例互斥必须最先注册：`Builder::build()` 按注册顺序 initialize_plugins，
    // 插件 setup 命中已有实例会直接 `std::process::exit(0)`，因此第二个进程在
    // 建主窗口 / 建托盘图标 / 起后台循环之前就已退出，不会产生账号侧副作用。
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            tray::on_second_instance(app, args);
        }));
    }

    builder = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    if wb_switch_core::modules::manager_security::OPTIONAL_COMPANION_ENABLED
        && !is_screenshot_demo()
    {
        builder = builder.plugin(agent_studio_desktop::init(companion::config()));
    }

    #[cfg(desktop)]
    {
        #[cfg(not(target_os = "windows"))]
        {
            builder = builder.plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec![tray::SILENT_STARTUP_ARG]),
            ));
        }
        builder = builder.on_window_event(tray::on_window_event);
    }

    let app = builder
        .setup(|app| {
            #[cfg(desktop)]
            {
                // 插件已在 initialize_plugins 阶段决定 notify-or-exit；此处只兜底
                // 插件漏掉的 macOS 竞态。必须在 tray::setup 之前：拿不到锁的第二
                // 实例不能先建出托盘图标。不得放到 run() 开头，否则会抢在插件
                // notify 之前拦下正常第二实例，丢掉「再点开 → 既有窗口弹出」。
                #[cfg(target_os = "macos")]
                instance_lock::acquire_or_exit(app.handle());
                tray::setup(app)?;
                // 主窗口由配置创建为不可见；在事件循环呈现前决定本次启动是否静默。
                // 仅系统自启（精确 `--hidden` 参数）进入静默托盘，普通启动立即显示主窗口。
                tray::setup_startup_visibility(
                    app.handle(),
                    tray::is_silent_startup(std::env::args()),
                );
            }
            // README 截图模式只渲染前端虚构数据，禁止读取账号后执行签到、轮换或保活。
            if !is_screenshot_demo() {
                spawn_background_loops(app.handle().clone());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_accounts,
            commands::get_codebuddy_cli_status,
            commands::install_codebuddy_cli_helper,
            commands::switch_codebuddy_cli_account,
            commands::get_codebuddy_cn_ide_status,
            commands::switch_codebuddy_cn_ide_account,
            commands::detect_codebuddy_cn_ide_account,
            commands::list_codebuddy_ide_sessions,
            commands::codebuddy_ide_session_links_preview,
            commands::get_vscode_ext_status,
            commands::switch_vscode_ext_account,
            commands::detect_vscode_ext_account,
            commands::list_vscode_sessions,
            commands::vscode_session_links_preview,
            commands::get_codebuddy_ide_status,
            commands::switch_codebuddy_ide_account,
            commands::list_codebuddy_intl_ide_sessions,
            commands::codebuddy_intl_ide_session_links_preview,
            commands::detect_codebuddy_ide_account,
            commands::get_jetbrains_status,
            commands::switch_jetbrains_account,
            commands::detect_jetbrains_account,
            commands::delete_account,
            commands::update_account_display,
            commands::oauth_start,
            commands::oauth_status,
            commands::import_local,
            commands::export_accounts,
            commands::export_accounts_to_path,
            commands::preview_import_accounts,
            commands::import_accounts,
            commands::switch_account,
            commands::list_sessions,
            commands::list_account_sessions,
            commands::copy_sessions,
            commands::copy_sessions_cross,
            commands::session_links_preview,
            commands::session_links_preview_cross,
            commands::session_sync_cross,
            commands::list_session_groups,
            commands::get_session_group,
            commands::preview_session_group_pair,
            commands::sync_session_group_pair,
            commands::sync_session_group_unify,
            commands::sync_session_group_safe_batch,
            commands::add_session_group_member,
            commands::copy_linked_sessions,
            commands::vscode_restart_precheck,
            commands::unlink_session_group_member,
            commands::delete_session_group,
            commands::open_permission_settings,
            commands::check_auth_permission,
            commands::reveal_app_in_finder,
            commands::get_checkin_status,
            commands::get_credit_expiry,
            commands::get_credit_statistics,
            commands::get_token_statistics,
            commands::get_rate_limits,
            commands::get_rate_limit_hook_status,
            commands::install_rate_limit_hook,
            commands::uninstall_rate_limit_hook,
            commands::get_rate_limit_config,
            commands::save_rate_limit_config,
            commands::checkin,
            commands::checkin_all,
            commands::get_auto_checkin_config,
            commands::get_manager_scheduler,
            commands::get_growth_tasks,
            commands::refresh_growth_tasks,
            commands::claim_growth_rewards,
            commands::get_growth_operations,
            commands::get_growth_schedule,
            commands::save_growth_schedule,
            commands::refresh_growth_travel,
            commands::run_growth_travel,
            commands::list_manager_backups,
            commands::create_manager_backup,
            commands::preview_manager_restore,
            commands::restore_manager_backup,
            commands::get_manager_logs,
            commands::save_manager_scheduler,
            commands::run_manager_checkin,
            commands::save_auto_checkin_config,
            commands::get_checkin_logs,
            commands::get_travel_status,
            commands::get_auto_travel_config,
            commands::save_auto_travel_config,
            commands::refresh_account_token,
            commands::get_auto_rotate_config,
            commands::save_auto_rotate_config,
            commands::rotate_status,
            commands::run_rotate,
            commands::get_rotate_logs,
            commands::get_github_config,
            commands::save_github_config,
            commands::check_update,
            commands::update_state,
            commands::update_download,
            commands::update_restart,
            commands::relaunch_app,
            commands::get_launch_at_login_enabled,
            commands::set_launch_at_login_enabled,
            commands::record_notification,
            commands::list_notifications,
            commands::clear_notifications,
            commands::log_error,
            commands::get_error_log_path,
            commands::reveal_error_log,
            companion::get_companion_enabled,
            companion::set_companion_enabled,
            companion::open_companion_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        #[cfg(desktop)]
        {
            // 点击 Dock / Finder 再次激活已运行实例：窗口已隐藏到托盘时显示主窗口。
            // `Reopen` 在主线程派发，可直接调用窗口路径。
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen {
                has_visible_windows: false,
                ..
            } = &event
            {
                tray::show_main_window_on_reopen(_app_handle);
            }
            tray::on_run_event(event);
        }
    });
}
