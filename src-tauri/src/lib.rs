// EGrab - 跨平台电商数据抓取客户端
// 后端核心库入口

pub mod cdp;
pub mod commands;
pub mod config;
pub mod downloader;
pub mod models;
pub mod parser;
pub mod scraper;
pub mod storage;

use tauri::Manager;
use tauri::WindowEvent;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber for diagnostic logging.
    // In dev mode, use a verbose subscriber that prints to stderr.
    // In release mode, log only warnings and errors to keep noise low.
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }
    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
