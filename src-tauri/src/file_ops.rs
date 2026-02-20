// 文件操作模块
// 负责文件移动、撤销操作、操作日志管理

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{MoveRecord, OperationLog};

/// 将图片移动到目标文件夹中对应人物的子文件夹
/// 返回操作日志用于撤销
pub fn move_images(
    images: &[(String, String, String)], // (path, filename, selected_person)
    target_dir: &str,
) -> Result<OperationLog, String> {
    let target_path = Path::new(target_dir);

    // 确保目标文件夹存在
    if !target_path.exists() {
        fs::create_dir_all(target_path)
            .map_err(|e| format!("创建目标文件夹失败: {}", e))?;
    }

    let mut records = Vec::new();
    let mut person_dirs: HashMap<String, PathBuf> = HashMap::new();

    for (path, _filename, person) in images {
        let source = Path::new(path);
        if !source.exists() {
            log::warn!("源文件不存在，跳过: {}", path);
            continue;
        }

        // 获取或创建人物文件夹
        let person_dir = person_dirs
            .entry(person.clone())
            .or_insert_with(|| {
                let dir = target_path.join(person);
                if !dir.exists() {
                    let _ = fs::create_dir_all(&dir);
                }
                dir
            })
            .clone();

        // 处理文件名冲突
        let original_filename = source
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dest_path = resolve_filename_conflict(&person_dir, &original_filename);

        // 执行移动
        match fs::rename(source, &dest_path) {
            Ok(()) => {
                records.push(MoveRecord {
                    original_path: path.clone(),
                    new_path: dest_path.to_string_lossy().to_string(),
                    filename: original_filename,
                });
            }
            Err(e) => {
                // rename 跨卷失败时，用 copy + delete
                match fs::copy(source, &dest_path) {
                    Ok(_) => {
                        let _ = fs::remove_file(source);
                        records.push(MoveRecord {
                            original_path: path.clone(),
                            new_path: dest_path.to_string_lossy().to_string(),
                            filename: original_filename,
                        });
                    }
                    Err(e2) => {
                        log::error!("移动文件失败 {} -> {}: rename={}, copy={}", path, dest_path.display(), e, e2);
                    }
                }
            }
        }
    }

    let log = OperationLog {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        target_dir: target_dir.to_string(),
        records,
    };

    Ok(log)
}

/// 撤销移动操作：将文件移回原处
pub fn undo_move(operation_log: &OperationLog) -> Result<usize, String> {
    let mut restored = 0;

    for record in &operation_log.records {
        let new_path = Path::new(&record.new_path);
        let original_path = Path::new(&record.original_path);

        if !new_path.exists() {
            log::warn!("要恢复的文件不存在: {}", record.new_path);
            continue;
        }

        // 确保原始目录存在
        if let Some(parent) = original_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建原始目录失败: {}", e))?;
            }
        }

        // 移回原处
        match fs::rename(new_path, original_path) {
            Ok(()) => {
                restored += 1;
            }
            Err(_) => {
                // 跨卷时用 copy + delete
                match fs::copy(new_path, original_path) {
                    Ok(_) => {
                        let _ = fs::remove_file(new_path);
                        restored += 1;
                    }
                    Err(e) => {
                        log::error!("恢复文件失败: {} -> {}: {}", record.new_path, record.original_path, e);
                    }
                }
            }
        }
    }

    // 清理可能留下的空人物文件夹
    cleanup_empty_dirs(&operation_log.target_dir);

    Ok(restored)
}

/// 解决文件名冲突：如果目标已存在同名文件，添加数字后缀
fn resolve_filename_conflict(dir: &Path, filename: &str) -> PathBuf {
    let dest = dir.join(filename);
    if !dest.exists() {
        return dest;
    }

    let path = Path::new(filename);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let new_name = format!("{}_{}{}", stem, counter, ext);
        let new_path = dir.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

/// 清理空的子文件夹
fn cleanup_empty_dirs(target_dir: &str) {
    let target = Path::new(target_dir);
    if let Ok(entries) = fs::read_dir(target) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(mut dir) = fs::read_dir(&path) {
                    if dir.next().is_none() {
                        let _ = fs::remove_dir(&path);
                    }
                }
            }
        }
    }
}
