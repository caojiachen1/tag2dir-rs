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
    let result = tokio::task::spawn_blocking(move || -> Result<ScanStats, String> {
        use rayon::prelude::*;

        // 1. 扫描图片文件列表
        let files = scanner::scan_image_files(&source_dir, include_subdirs);
        let total = files.len();

        log::info!("找到 {} 个图片文件，开始并行处理元数据...", total);

        // 用于统计人物（并行安全容器）
        let person_buckets = dashmap::DashSet::new();
        let scanned_count = std::sync::atomic::AtomicUsize::new(0);

        // 限制并行线程数，避免 100% 占用导致电脑卡顿
        // 设置为逻辑核心数的一半，但至少 1 个线程，最多 6 个线程
        let num_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        let worker_threads = (num_cpus / 2).clamp(1, 6);
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(worker_threads)
            .build()
            .map_err(|e| format!("创建线程池失败: {}", e))?;

        pool.install(|| {
            files.into_par_iter().for_each(|path| {
                // 检查取消标志
                if cancel_flag.load(Ordering::Relaxed) {
                    return;
                }

                let result = scanner::process_single_image(&path);
                let current_count = scanned_count.fetch_add(1, Ordering::SeqCst) + 1;

                match result {
                    Ok(info) => {
                        for person in &info.persons {
                            person_buckets.insert(person.clone());
                        }
                        let done = current_count >= total;
                        let event = ScanProgressEvent {
                            scanned: current_count,
                            image: Some(info),
                            done,
                            cancelled: false,
                            error: None,
                        };
                        let _ = app_handle.emit("scan-progress", &event);
                    }
                    Err(e) => {
                        log::warn!("处理图片失败: {}", e);
                        let done = current_count >= total;
                        let event = ScanProgressEvent {
                            scanned: current_count,
                            image: None,
                            done,
                            cancelled: false,
                            error: Some(e),
                        };
                        let _ = app_handle.emit("scan-progress", &event);
                    }
                }
            });
        });

        // 检查是否是被取消的
        let final_count = scanned_count.load(Ordering::SeqCst);
        if cancel_flag.load(Ordering::Relaxed) {
            log::info!("扫描被用户取消，已处理 {} 张", final_count);
            let event = ScanProgressEvent {
                scanned: final_count,
                image: None,
                done: true,
                cancelled: true,
                error: None,
            };
            let _ = app_handle.emit("scan-progress", &event);
        } else if total == 0 {
            // 如果 files 为空，也发送完成事件
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

        Ok(ScanStats {
            total_images: final_count,
            person_count: person_names.len(),
            person_names,
        })
    })
    .await
    .map_err(|e| format!("扫描任务失败: {}", e))??;

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
