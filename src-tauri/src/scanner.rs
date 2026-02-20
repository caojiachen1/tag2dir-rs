// 图片扫描模块
// 遍历文件夹、过滤图片文件、读取元数据、生成缩略图

use std::path::{Path, PathBuf};

use base64::Engine;
use image::GenericImageView;
use walkdir::WalkDir;

use crate::metadata;
use crate::models::{ImageInfo, ImageStatus};

/// 支持的图片扩展名
const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "tiff", "tif", "bmp", "gif", "heic", "heif", "avif",
];

/// 缩略图最大尺寸（像素，长边）
const THUMBNAIL_MAX_SIZE: u32 = 300;

/// 扫描指定文件夹中的图片文件
/// 返回所有图片文件路径列表
pub fn scan_image_files(source_dir: &str, include_subdirs: bool) -> Vec<PathBuf> {
    let walker = WalkDir::new(source_dir);
    let walker = if include_subdirs {
        walker
    } else {
        walker.max_depth(1)
    };

    walker
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if !entry.file_type().is_file() {
                return false;
            }
            if let Some(ext) = entry.path().extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                IMAGE_EXTENSIONS.contains(&ext_lower.as_str())
            } else {
                false
            }
        })
        .map(|entry| entry.into_path())
        .collect()
}

/// 处理单张图片：读取元数据 + 生成缩略图
/// 返回 ImageInfo 或错误信息
pub fn process_single_image(path: &Path) -> Result<ImageInfo, String> {
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let path_str = path.to_string_lossy().to_string();

    // 读取人物标签和关键字
    let (persons, keywords) = metadata::extract_person_tags(path);

    // 生成缩略图
    let thumbnail = generate_thumbnail(path).unwrap_or_default();

    // 只要有人物标签，就默认选择第一个（多人物时也选第一个，用户可在前端修改）
    let selected_person = if !persons.is_empty() {
        Some(persons[0].clone())
    } else {
        None
    };

    Ok(ImageInfo {
        id: uuid::Uuid::new_v4().to_string(),
        path: path_str,
        filename,
        persons,
        keywords,
        thumbnail,
        selected_person,
        status: ImageStatus::Scanned,
    })
}

/// 生成图片缩略图，返回 base64 编码的 JPEG 数据
fn generate_thumbnail(path: &Path) -> Result<String, String> {
    let img = image::open(path).map_err(|e| format!("无法打开图片: {}", e))?;

    let (w, h) = img.dimensions();

    // 计算缩放比例，保持宽高比
    let scale = if w > h {
        THUMBNAIL_MAX_SIZE as f64 / w as f64
    } else {
        THUMBNAIL_MAX_SIZE as f64 / h as f64
    };

    // 如果图片本身比缩略图小，不放大
    let (new_w, new_h) = if scale < 1.0 {
        ((w as f64 * scale) as u32, (h as f64 * scale) as u32)
    } else {
        (w, h)
    };

    // 使用 Lanczos3 滤波器获得较好的缩放质量
    let thumbnail = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);

    // 编码为 JPEG
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    thumbnail
        .write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| format!("缩略图编码失败: {}", e))?;

    // 转为 base64
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(format!("data:image/jpeg;base64,{}", b64))
}
