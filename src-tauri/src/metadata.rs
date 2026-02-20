// 元数据读取模块
// 从图片文件中提取人物标签信息（EXIF/IPTC/XMP）

use std::fs;
use std::io::BufReader;
use std::path::Path;

/// 从图片文件中提取所有人物/关键字标签
/// 按优先级检查: XMP人物区域 > XMP dc:subject > IPTC关键字 > EXIF XPKeywords
pub fn extract_person_tags(path: &Path) -> (Vec<String>, Vec<String>) {
    let mut all_keywords: Vec<String> = Vec::new();
    let mut persons: Vec<String> = Vec::new();

    // 尝试读取 EXIF 数据
    if let Ok(exif_keywords) = read_exif_keywords(path) {
        all_keywords.extend(exif_keywords);
    }

    // 尝试读取 XMP 数据（嵌入在 JPEG/TIFF 等格式中）
    if let Ok((xmp_persons, xmp_keywords)) = read_xmp_data(path) {
        persons.extend(xmp_persons);
        all_keywords.extend(xmp_keywords);
    }

    // 尝试读取 IPTC 关键字
    if let Ok(iptc_keywords) = read_iptc_keywords(path) {
        all_keywords.extend(iptc_keywords);
    }

    // 去重
    all_keywords.sort();
    all_keywords.dedup();
    persons.sort();
    persons.dedup();

    // 如果没有从 XMP 人物区域中找到人物，将所有关键字都视为潜在人物标签
    if persons.is_empty() && !all_keywords.is_empty() {
        persons = all_keywords.clone();
    }

    (persons, all_keywords)
}

/// 读取 EXIF 中的 XPKeywords（Windows 风格的关键字标签）
fn read_exif_keywords(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut reader)?;

    let mut keywords = Vec::new();

    // XPKeywords (Tag 0x9C9E) 不在 kamadak-exif 预定义常量中，手动构造
    let xp_keywords_tag = exif::Tag(exif::Context::Tiff, 0x9C9E);
    if let Some(field) = exif.get_field(xp_keywords_tag, exif::In::PRIMARY) {
        if let exif::Value::Byte(ref bytes) = field.value {
            // XPKeywords 是 UTF-16LE 编码的字符串，用分号分隔
            let u16_chars: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
            let text = String::from_utf16_lossy(&u16_chars);
            let text = text.trim_end_matches('\0');
            for kw in text.split(';') {
                let kw = kw.trim();
                if !kw.is_empty() {
                    keywords.push(kw.to_string());
                }
            }
        }
    }

    // 读取 ImageDescription 作为补充信息
    if let Some(field) = exif.get_field(exif::Tag::ImageDescription, exif::In::PRIMARY) {
        let desc = field.display_value().to_string();
        let desc = desc.trim_matches('"').trim();
        if !desc.is_empty() && desc.len() < 100 {
            // 有些软件把人物名放在描述字段中
            // 只在描述较短时考虑（长描述通常不是人物名）
        }
    }

    Ok(keywords)
}

/// 从文件中提取 XMP 数据段并解析人物和关键字
fn read_xmp_data(path: &Path) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let mut persons = Vec::new();
    let mut keywords = Vec::new();

    // 查找 XMP 数据段 - 标记为 "<?xpacket" 或 "<x:xmpmeta"
    let xmp_xml = extract_xmp_from_bytes(&data);

    if let Some(xml) = xmp_xml {
        parse_xmp_xml(&xml, &mut persons, &mut keywords);
    }

    Ok((persons, keywords))
}

/// 从原始字节中提取 XMP XML 数据
fn extract_xmp_from_bytes(data: &[u8]) -> Option<String> {
    let data_str = String::from_utf8_lossy(data);

    // 查找 XMP 包开始标记
    let start_markers = ["<x:xmpmeta", "<?xpacket begin"];
    let end_markers = ["</x:xmpmeta>", "<?xpacket end"];

    for (start_marker, end_marker) in start_markers.iter().zip(end_markers.iter()) {
        if let Some(start) = data_str.find(start_marker) {
            if let Some(end_pos) = data_str[start..].find(end_marker) {
                let end = start + end_pos + end_marker.len();
                // 确保截取的是 xmpmeta 部分
                let xml_chunk = &data_str[start..end];
                // 如果以 <?xpacket 开头，需要找到实际的 xmpmeta
                if let Some(meta_start) = xml_chunk.find("<x:xmpmeta") {
                    if let Some(meta_end) = xml_chunk.find("</x:xmpmeta>") {
                        return Some(xml_chunk[meta_start..meta_end + "</x:xmpmeta>".len()].to_string());
                    }
                }
                return Some(xml_chunk.to_string());
            }
        }
    }

    None
}

/// 解析 XMP XML 并提取人物和关键字
fn parse_xmp_xml(xml: &str, persons: &mut Vec<String>, keywords: &mut Vec<String>) {
    let doc = match roxmltree::Document::parse(xml) {
        Ok(doc) => doc,
        Err(_) => return,
    };

    // 遍历所有节点
    for node in doc.descendants() {
        let tag_name = node.tag_name().name();

        // dc:subject - Dublin Core 主题/关键字
        // 通常包含在 <rdf:Bag><rdf:li> 结构中
        if tag_name == "subject" {
            for child in node.descendants() {
                if child.tag_name().name() == "li" {
                    if let Some(text) = child.text() {
                        let text = text.trim();
                        if !text.is_empty() {
                            keywords.push(text.to_string());
                        }
                    }
                }
            }
        }

        // mwg-rs:RegionList / MP:RegionInfo - 人物区域标记
        // Metadata Working Group 标准的人物区域
        if tag_name == "RegionList" || tag_name == "Regions" || tag_name == "RegionInfo" {
            extract_region_persons(node, persons);
        }

        // Lightroom/Bridge 人物标签: lr:hierarchicalSubject
        if tag_name == "hierarchicalSubject" {
            for child in node.descendants() {
                if child.tag_name().name() == "li" {
                    if let Some(text) = child.text() {
                        let text = text.trim();
                        // 层级标签格式: "People|人物名" 或 "人物|名字"
                        if text.contains('|') {
                            let parts: Vec<&str> = text.split('|').collect();
                            if parts.len() >= 2 {
                                let category = parts[0].to_lowercase();
                                if category.contains("people")
                                    || category.contains("person")
                                    || category.contains("人物")
                                    || category.contains("人")
                                {
                                    persons.push(parts[parts.len() - 1].trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // digiKam 人物标签: digiKam:TagsList
        if tag_name == "TagsList" {
            for child in node.descendants() {
                if child.tag_name().name() == "li" {
                    if let Some(text) = child.text() {
                        let text = text.trim();
                        if text.contains('/') {
                            let parts: Vec<&str> = text.split('/').collect();
                            if parts.len() >= 2 {
                                let category = parts[0].to_lowercase();
                                if category.contains("people")
                                    || category.contains("person")
                                    || category.contains("人物")
                                {
                                    persons.push(parts[parts.len() - 1].trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 从人物区域标记中提取人物名称
fn extract_region_persons(node: roxmltree::Node, persons: &mut Vec<String>) {
    for descendant in node.descendants() {
        let tag = descendant.tag_name().name();

        // MWG-RS 标准: <mwg-rs:Name>人物名</mwg-rs:Name>
        if tag == "Name" {
            if let Some(text) = descendant.text() {
                let text = text.trim();
                if !text.is_empty() {
                    persons.push(text.to_string());
                }
            }
        }

        // 也检查 PersonDisplayName 属性（Microsoft Photo 使用）
        if tag == "PersonDisplayName" {
            if let Some(text) = descendant.text() {
                let text = text.trim();
                if !text.is_empty() {
                    persons.push(text.to_string());
                }
            }
        }

        // 检查属性中的人物名
        for attr in descendant.attributes() {
            if attr.name().to_lowercase().contains("name")
                || attr.name().to_lowercase().contains("person")
            {
                let val = attr.value().trim();
                if !val.is_empty() && val != "true" && val != "false" {
                    persons.push(val.to_string());
                }
            }
        }
    }
}

/// 读取 IPTC 关键字（解析 JPEG 中的 IPTC-IIM 数据段）
fn read_iptc_keywords(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let mut keywords = Vec::new();

    // JPEG IPTC 数据位于 APP13 段 (0xFFED) 中
    // 查找 Photoshop 3.0 标记
    let photoshop_marker = b"Photoshop 3.0\x00";

    if let Some(ps_start) = find_subsequence(&data, photoshop_marker) {
        let iptc_data = &data[ps_start + photoshop_marker.len()..];
        // 查找 8BIM 段中的 IPTC 数据 (resource ID 0x0404)
        parse_iptc_from_photoshop(iptc_data, &mut keywords);
    }

    Ok(keywords)
}

/// 在字节切片中查找子序列
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// 从 Photoshop 资源块中解析 IPTC 关键字
fn parse_iptc_from_photoshop(data: &[u8], keywords: &mut Vec<String>) {
    let mut pos = 0;

    while pos + 12 <= data.len() {
        // 查找 8BIM 标记
        if &data[pos..pos + 4] != b"8BIM" {
            pos += 1;
            continue;
        }

        let resource_id = u16::from_be_bytes([data[pos + 4], data[pos + 5]]);
        // 跳过 pascal string (resource name)
        let name_len = data[pos + 6] as usize;
        let padded_name_len = if (name_len + 1) % 2 != 0 {
            name_len + 2
        } else {
            name_len + 1
        };

        let size_offset = pos + 6 + padded_name_len;
        if size_offset + 4 > data.len() {
            break;
        }

        let block_size = u32::from_be_bytes([
            data[size_offset],
            data[size_offset + 1],
            data[size_offset + 2],
            data[size_offset + 3],
        ]) as usize;

        let block_start = size_offset + 4;
        let block_end = block_start + block_size;

        if block_end > data.len() {
            break;
        }

        // IPTC-IIM 数据的资源 ID 是 0x0404
        if resource_id == 0x0404 {
            parse_iptc_records(&data[block_start..block_end], keywords);
        }

        // 移动到下一个资源块（对齐到偶数边界）
        pos = block_end;
        if pos % 2 != 0 {
            pos += 1;
        }
    }
}

/// 解析 IPTC-IIM 记录，提取关键字 (DataSet 2:25)
fn parse_iptc_records(data: &[u8], keywords: &mut Vec<String>) {
    let mut pos = 0;

    while pos + 5 <= data.len() {
        // IPTC 记录标记 0x1C
        if data[pos] != 0x1C {
            pos += 1;
            continue;
        }

        let record_number = data[pos + 1];
        let dataset_number = data[pos + 2];
        let field_len = u16::from_be_bytes([data[pos + 3], data[pos + 4]]) as usize;

        pos += 5;

        if pos + field_len > data.len() {
            break;
        }

        // Record 2, Dataset 25 = Keywords
        if record_number == 2 && dataset_number == 25 {
            if let Ok(keyword) = String::from_utf8(data[pos..pos + field_len].to_vec()) {
                let keyword = keyword.trim().to_string();
                if !keyword.is_empty() {
                    keywords.push(keyword);
                }
            } else {
                // 尝试 Latin1 解码
                let keyword: String = data[pos..pos + field_len]
                    .iter()
                    .map(|&b| b as char)
                    .collect();
                let keyword = keyword.trim().to_string();
                if !keyword.is_empty() {
                    keywords.push(keyword);
                }
            }
        }

        pos += field_len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_xmp_from_bytes() {
        let xml = r#"some prefix <x:xmpmeta xmlns:x="adobe:ns:meta/">
            <rdf:RDF>
                <rdf:Description>
                    <dc:subject>
                        <rdf:Bag>
                            <rdf:li>Alice</rdf:li>
                            <rdf:li>Bob</rdf:li>
                        </rdf:Bag>
                    </dc:subject>
                </rdf:Description>
            </rdf:RDF>
        </x:xmpmeta> some suffix"#;

        let result = extract_xmp_from_bytes(xml.as_bytes());
        assert!(result.is_some());
        let xmp = result.unwrap();
        assert!(xmp.contains("Alice"));
        assert!(xmp.contains("Bob"));
    }
}
