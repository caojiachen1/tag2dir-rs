// 数据模型定义
// 定义图片信息、操作日志等核心数据结构

use serde::{Deserialize, Serialize};

/// 图片信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// 唯一标识
    pub id: String,
    /// 文件完整路径
    pub path: String,
    /// 文件名
    pub filename: String,
    /// 检测到的人物标签列表
    pub persons: Vec<String>,
    /// 所有元数据关键字
    pub keywords: Vec<String>,
    /// 缩略图 base64 编码
    pub thumbnail: String,
    /// 用户选择的目标人物（用于移动分类）
    pub selected_person: Option<String>,
    /// 处理状态
    pub status: ImageStatus,
}

/// 图片处理状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageStatus {
    /// 已扫描，等待处理
    Scanned,
    /// 已选择人物，等待移动
    Ready,
    /// 正在移动
    Moving,
    /// 移动完成
    Moved,
    /// 处理出错
    Error(String),
}

/// 扫描进度事件 - 通过 Tauri event 推送到前端
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgressEvent {
    /// 当前已扫描数量
    pub scanned: usize,
    /// 扫描到的图片信息（增量）
    pub image: Option<ImageInfo>,
    /// 是否扫描完成（正常完成或取消都会为 true）
    pub done: bool,
    /// 是否被用户取消
    pub cancelled: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 移动进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveProgressEvent {
    /// 当前已移动数量
    pub moved_count: usize,
    /// 总数
    pub total: usize,
    /// 当前正在处理的文件名
    pub current_file: String,
    /// 是否完成
    pub done: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 移动操作记录（用于撤销）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRecord {
    /// 原始路径
    pub original_path: String,
    /// 移动后的路径
    pub new_path: String,
    /// 文件名
    pub filename: String,
}

/// 操作日志（用于撤销整次操作）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    /// 操作唯一标识
    pub id: String,
    /// 操作时间
    pub timestamp: String,
    /// 目标文件夹
    pub target_dir: String,
    /// 所有移动记录
    pub records: Vec<MoveRecord>,
}

/// 扫描统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanStats {
    /// 总图片数
    pub total_images: usize,
    /// 检测到的不同人物数（桶计数）
    pub person_count: usize,
    /// 人物名称列表
    pub person_names: Vec<String>,
}
