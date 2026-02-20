// tag2dir - 图片人物分类工具
// 主入口模块，注册所有 Tauri 命令和插件

mod commands;
mod file_ops;
mod metadata;
mod models;
mod scanner;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_images,
            commands::cancel_scan,
            commands::move_images,
            commands::undo_move,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
