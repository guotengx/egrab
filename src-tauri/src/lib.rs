// EGrab - 跨平台电商数据抓取客户端
// 后端核心库入口

pub mod cdp;
pub mod commands;
pub mod config;
pub mod downloader;
pub mod models;
pub mod parser;
pub mod resize;
pub mod scraper;
pub mod storage;

use tauri::Manager;
use tauri::WindowEvent;

/// 日志文件超过这个大小时，启动时轮转为 egrab.log.1。
const LOG_ROTATE_BYTES: u64 = 5 * 1024 * 1024;

/// 初始化诊断日志：同时输出到 stderr 和 `<app_data>/logs/egrab.log`。
///
/// Windows 安装版没有控制台，导航超时这类失败不会生成 raw.json，
/// 日志文件是排查时唯一的现场记录。日志文件打不开时退回只输出 stderr。
fn init_tracing() {
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    let level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    match open_log_file() {
        Some(file) => {
            let writer = std::io::stderr.and(std::sync::Mutex::new(file));
            tracing_subscriber::fmt()
                .with_max_level(level)
                .with_ansi(false)
                .with_writer(writer)
                .init();
        }
        None => {
            tracing_subscriber::fmt().with_max_level(level).init();
        }
    }
}

/// 以追加模式打开 `<app_data>/logs/egrab.log`；超过 LOG_ROTATE_BYTES 时先轮转。
fn open_log_file() -> Option<std::fs::File> {
    let dir = parser::rules::app_data_dir().join("logs");
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("egrab.log");
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > LOG_ROTATE_BYTES {
            let rotated = dir.join("egrab.log.1");
            let _ = std::fs::remove_file(&rotated);
            let _ = std::fs::rename(&path, &rotated);
        }
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 释放/升级外置抓取规则包到磁盘。
            // 规则文件可由用户直接编辑，平台改版时无需重新编译或重装。
            let rules_status = parser::rules::ensure_rules_on_disk();
            tracing::info!(status = %rules_status, "Rule pack ready");

            // Initialize and inject managed state: AppConfigManager
            let config_manager = config::AppConfigManager::new();
            let storage_root = config_manager.get_config().storage_root;
            app.manage(config_manager);

            // Initialize and inject managed state: CdpManager
            let cdp_manager = cdp::CdpManager::new(app.handle().clone());
            app.manage(cdp_manager);

            // Initialize and inject managed state: StorageEngine
            let storage_engine = storage::StorageEngine::new(storage_root);
            // Initialize the database and create directories.
            // Using expect here is acceptable: if storage init fails, the app
            // cannot function and should report the error immediately.
            storage_engine.init().expect("Failed to initialize StorageEngine");
            app.manage(tokio::sync::Mutex::new(storage_engine));

            #[cfg(debug_assertions)]
            {
                // The unwrap is safe here: the "main" window is guaranteed to exist
                // when running in debug mode with the standard Tauri config.
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
                // App is exiting: kill any browser that EGrab launched.
                if let Some(cdp) = window.app_handle().try_state::<cdp::CdpManager>() {
                    cdp.shutdown();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::cdp_commands::cdp_connect,
            commands::cdp_commands::cdp_auto_connect,
            commands::cdp_commands::cdp_disconnect,
            commands::cdp_commands::cdp_status,
            commands::cdp_commands::cdp_list_tabs,
            commands::cdp_commands::cdp_navigate,
            commands::config_commands::get_config,
            commands::config_commands::set_config,
            commands::scrape_commands::start_scrape,
            commands::scrape_commands::cancel_scrape,
            commands::task_commands::get_task_history,
            commands::task_commands::get_task_detail,
            commands::task_commands::delete_task,
            commands::task_commands::open_folder,
            commands::task_commands::get_cover_image,
            commands::resize_commands::resize_images,
            commands::rules_commands::get_rules_info,
            commands::rules_commands::reload_rules,
            commands::rules_commands::open_rules_folder,
            commands::rules_commands::dump_page_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
