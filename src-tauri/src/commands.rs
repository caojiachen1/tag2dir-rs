// Tauri 命令模块
// 暴露给前端调用的所有命令，处理扫描、移动、撤销等操作

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager};

use crate::file_ops;
use crate::models::*;
use crate::scanner;

/// 全局应用状态
pub struct AppState {
    /// 最近一次操作日志（用于撤销）
    pub last_operation: Mutex<Option<OperationLog>>,
    /// 是否正在扫描
    pub scanning: Mutex<bool>,
    /// 取消扫描标志位（原子操作，跨线程安全，无需 Mutex）
    pub cancel_scan: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            last_operation: Mutex::new(None),
            scanning: Mutex::new(false),
            cancel_scan: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// 扫描图片命令
/// 异步递归扫描指定文件夹，通过事件流式推送结果到前端
#[tauri::command]
pub async fn scan_images(
    app: AppHandle,
    source_dir: String,
    include_subdirs: bool,
) -> Result<ScanStats, String> {
    // 检查是否已在扫描
    let state = app.state::<AppState>();
    {
        let mut scanning = state.scanning.lock();
        if *scanning {
            return Err("已有扫描任务在进行中".to_string());
        }
        *scanning = true;
    }
    // 重置取消标志
    state.cancel_scan.store(false, Ordering::Relaxed);
    let cancel_flag = state.cancel_scan.clone();

    // 在后台线程中执行扫描
    let app_handle = app.clone();
    let result = tokio::task::spawn_blocking(move || {
        // 1. 扫描图片文件列表
        let files = scanner::scan_image_files(&source_dir, include_subdirs);
        let total = files.len();

        log::info!("找到 {} 个图片文件，开始处理元数据...", total);

        // 用于统计人物（桶算法）
        let mut person_buckets: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut scanned_count = 0usize;

        // 2. 逐个处理图片（流式推送到前端），每批并行处理后逐个推送
        let batch_size = 4;
        'outer: for batch in files.chunks(batch_size) {
            // 检查取消标志 —— 每批开始前检查一次
            if cancel_flag.load(Ordering::Relaxed) {
                log::info!("扫描被用户取消，已处理 {} 张", scanned_count);
                let event = ScanProgressEvent {
                    scanned: scanned_count,
                    image: None,
                    done: true,
                    cancelled: true,
                    error: None,
                };
                let _ = app_handle.emit("scan-progress", &event);
                break 'outer;
            }

            let results: Vec<Result<ImageInfo, String>> = batch
                .iter()
                .map(|path| scanner::process_single_image(path))
                .collect();

            for result in results {
                scanned_count += 1;
                match result {
                    Ok(info) => {
                        for person in &info.persons {
                            person_buckets.insert(person.clone());
                        }
                        let done = scanned_count >= total;
                        let event = ScanProgressEvent {
                            scanned: scanned_count,
                            image: Some(info),
                            done,
                            cancelled: false,
                            error: None,
                        };
                        let _ = app_handle.emit("scan-progress", &event);
                    }
                    Err(e) => {
                        log::warn!("处理图片失败: {}", e);
                        let done = scanned_count >= total;
                        let event = ScanProgressEvent {
                            scanned: scanned_count,
                            image: None,
                            done,
                            cancelled: false,
                            error: Some(e),
                        };
                        let _ = app_handle.emit("scan-progress", &event);
                    }
                }
            }
        }

        // 如果 files 为空，也发送完成事件
        if total == 0 {
            let event = ScanProgressEvent {
                scanned: 0,
                image: None,
                done: true,
                cancelled: false,
                error: None,
            };
            let _ = app_handle.emit("scan-progress", &event);
        }

        let mut person_names: Vec<String> = person_buckets.into_iter().collect();
        person_names.sort();

        ScanStats {
            total_images: scanned_count,
            person_count: person_names.len(),
            person_names,
        }
    })
    .await
    .map_err(|e| format!("扫描任务失败: {}", e))?;

    // 重置扫描状态
    let state = app.state::<AppState>();
    *state.scanning.lock() = false;

    Ok(result)
}

/// 取消正在进行的扫描
#[tauri::command]
pub async fn cancel_scan(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    state.cancel_scan.store(true, Ordering::Relaxed);
    log::info!("收到取消扫描请求");
    Ok(())
}

/// 执行移动命令
/// 将选中的图片移动到目标文件夹的人物子文件夹中
#[tauri::command]
pub async fn move_images(
    app: AppHandle,
    images: Vec<MoveImageRequest>,
    target_dir: String,
) -> Result<MoveResult, String> {
    let app_handle = app.clone();

    let result = tokio::task::spawn_blocking(move || {
        let total = images.len();

        // 转换为内部格式
        let move_items: Vec<(String, String, String)> = images
            .iter()
            .map(|img| (img.path.clone(), img.filename.clone(), img.person.clone()))
            .collect();

        // 推送初始进度
        let _ = app_handle.emit(
            "move-progress",
            &MoveProgressEvent {
                moved_count: 0,
                total,
                current_file: String::new(),
                done: false,
                error: None,
            },
        );

        // 执行批量移动
        let operation_log = file_ops::move_images(&move_items, &target_dir)?;
        let moved = operation_log.records.len();

        // 推送完成事件
        let _ = app_handle.emit(
            "move-progress",
            &MoveProgressEvent {
                moved_count: moved,
                total,
                current_file: String::new(),
                done: true,
                error: None,
            },
        );

        Ok::<(OperationLog, usize), String>((operation_log, moved))
    })
    .await
    .map_err(|e| format!("移动任务失败: {}", e))??;

    let (operation_log, moved) = result;

    // 保存操作日志用于撤销
    let state = app.state::<AppState>();
    *state.last_operation.lock() = Some(operation_log);

    Ok(MoveResult {
        moved_count: moved,
        has_undo: true,
    })
}

/// 撤销上次移动操作
#[tauri::command]
pub async fn undo_move(app: AppHandle) -> Result<UndoResult, String> {
    let state = app.state::<AppState>();
    let operation_log = {
        let mut lock = state.last_operation.lock();
        lock.take()
    };

    match operation_log {
        Some(log) => {
            let restored = tokio::task::spawn_blocking(move || file_ops::undo_move(&log))
                .await
                .map_err(|e| format!("撤销任务失败: {}", e))??;

            Ok(UndoResult {
                restored_count: restored,
                success: true,
            })
        }
        None => Err("没有可撤销的操作".to_string()),
    }
}

// === 请求/响应数据结构 ===

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MoveImageRequest {
    pub path: String,
    pub filename: String,
    pub person: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MoveResult {
    pub moved_count: usize,
    pub has_undo: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UndoResult {
    pub restored_count: usize,
    pub success: bool,
}
