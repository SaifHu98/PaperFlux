use std::collections::HashMap;
use thiserror::Error;
use pdf2md_ast::DocumentMetadata;
use crate::elements::RawPage;
use crate::font::FontMap;
use crate::parser::ContentStreamParser;
use crate::security::{SecurityError, SecurityLimits};
use crate::stream::decode_stream;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("Invalid PDF header or format")]
    InvalidHeader,

    #[error("Missing or corrupt xref table")]
    CorruptXref,

    #[error("PDF is password encrypted")]
    EncryptedDocument,

    #[error("Security limit violated: {0}")]
    Security(#[from] SecurityError),

    #[error("PDF parsing error: {0}")]
    ParseError(String),
}

#[derive(Debug)]
pub struct PdfDocument {
    pub metadata: DocumentMetadata,
    pub pages: Vec<RawPage>,
    pub limits: SecurityLimits,
}

impl PdfDocument {
    pub fn parse(bytes: &[u8], limits: SecurityLimits) -> Result<Self, PdfError> {
        if bytes.len() > limits.max_file_size_bytes {
            return Err(PdfError::Security(SecurityError::FileSizeExceeded(
                bytes.len(),
                limits.max_file_size_bytes,
            )));
        }

        if !bytes.starts_with(b"%PDF-") {
            return Err(PdfError::InvalidHeader);
        }

        let pdf_str = String::from_utf8_lossy(bytes);
        let version = pdf_str.lines().next().map(|l| l.trim().to_string());

        let mut metadata = DocumentMetadata {
            pdf_version: version,
            total_pages: 0,
            ..Default::default()
        };

        // Extract metadata from /Info dictionary if present
        extract_metadata(&pdf_str, &mut metadata);

        // Extract and process pages
        let pages = extract_pages(bytes, &pdf_str, &limits)?;
        metadata.total_pages = pages.len();

        if metadata.total_pages > limits.max_pages {
            return Err(PdfError::Security(SecurityError::PageCountExceeded(
                metadata.total_pages,
                limits.max_pages,
            )));
        }

        Ok(Self {
            metadata,
            pages,
            limits,
        })
    }
}

fn extract_metadata(pdf_str: &str, meta: &mut DocumentMetadata) {
    if let Some(info_pos) = pdf_str.find("/Title") {
        if let Some(val) = extract_pdf_string_value(&pdf_str[info_pos..]) {
            meta.title = Some(val);
        }
    }
    if let Some(author_pos) = pdf_str.find("/Author") {
        if let Some(val) = extract_pdf_string_value(&pdf_str[author_pos..]) {
            meta.author = Some(val);
        }
    }
    if let Some(subject_pos) = pdf_str.find("/Subject") {
        if let Some(val) = extract_pdf_string_value(&pdf_str[subject_pos..]) {
            meta.subject = Some(val);
        }
    }
    if let Some(creator_pos) = pdf_str.find("/Creator") {
        if let Some(val) = extract_pdf_string_value(&pdf_str[creator_pos..]) {
            meta.creator = Some(val);
        }
    }
    if let Some(producer_pos) = pdf_str.find("/Producer") {
        if let Some(val) = extract_pdf_string_value(&pdf_str[producer_pos..]) {
            meta.producer = Some(val);
        }
    }
}

fn extract_pdf_string_value(slice: &str) -> Option<String> {
    let start = slice.find('(')?;
    let end = slice.find(')')?;
    if end > start {
        Some(slice[start + 1..end].trim().to_string())
    } else {
        None
    }
}

fn extract_pages(raw_bytes: &[u8], pdf_str: &str, limits: &SecurityLimits) -> Result<Vec<RawPage>, PdfError> {
    let mut raw_pages = Vec::new();
    let mut page_num = 1;

    // Scan for /Type /Page objects (ensuring not /Type /Pages)
    let mut search_idx = 0;
    while let Some(page_pos) = pdf_str[search_idx..].find("/Type /Page") {
        let abs_pos = search_idx + page_pos;
        let next_char = pdf_str.as_bytes().get(abs_pos + 11);
        
        // If it is /Type /Pages, skip
        if next_char == Some(&b's') || next_char == Some(&b'S') {
            search_idx = abs_pos + 12;
            continue;
        }
        
        // Find enclosing object `n m obj ... endobj`
        let obj_start = pdf_str[..abs_pos].rfind("obj").unwrap_or(abs_pos.saturating_sub(100));
        let obj_end = pdf_str[abs_pos..].find("endobj").map(|p| abs_pos + p).unwrap_or(pdf_str.len());

        let page_dict = &pdf_str[obj_start..obj_end];

        // Parse MediaBox [x y w h]
        let (width, height) = parse_mediabox(page_dict).unwrap_or((612.0, 792.0)); // Default letter: 612x792 pt
        let mut raw_page = RawPage::new(page_num, width, height);

        // Find fonts referenced in /Resources /Font
        let mut fonts: HashMap<String, FontMap> = HashMap::new();
        extract_fonts_for_page(pdf_str, page_dict, &mut fonts);

        // Find and decode /Contents stream
        if let Some(contents_stream) = extract_contents_stream(raw_bytes, pdf_str, page_dict, limits)? {
            let mut parser = ContentStreamParser::new(&fonts, height);
            parser.parse_content_stream(&contents_stream, &mut raw_page);
        }

        raw_page.assess_capabilities();
        raw_pages.push(raw_page);

        page_num += 1;
        search_idx = obj_end;
        if page_num > limits.max_pages {
            break;
        }
    }

    // If no /Type /Page objects found, fallback: parse all stream objects directly
    if raw_pages.is_empty() {
        let mut fallback_page = RawPage::new(1, 612.0, 792.0);
        let fonts = HashMap::new();
        let mut parser = ContentStreamParser::new(&fonts, 792.0);

        let mut stream_search = 0;
        while let Some(start_stream) = pdf_str[stream_search..].find("stream") {
            let stream_body_start = stream_search + start_stream + 6;
            let stream_body_start = if pdf_str.as_bytes().get(stream_body_start) == Some(&b'\r') && pdf_str.as_bytes().get(stream_body_start + 1) == Some(&b'\n') {
                stream_body_start + 2
            } else if pdf_str.as_bytes().get(stream_body_start) == Some(&b'\n') {
                stream_body_start + 1
            } else {
                stream_body_start
            };

            if let Some(end_stream) = pdf_str[stream_body_start..].find("endstream") {
                let stream_body_end = stream_body_start + end_stream;
                if stream_body_end <= raw_bytes.len() {
                    let raw_slice = &raw_bytes[stream_body_start..stream_body_end];
                    if let Ok(decompressed) = decode_stream(raw_slice, Some("FlateDecode"), limits) {
                        parser.parse_content_stream(&decompressed, &mut fallback_page);
                    } else {
                        parser.parse_content_stream(raw_slice, &mut fallback_page);
                    }
                }
                stream_search = stream_body_end + 9;
            } else {
                break;
            }
        }
        fallback_page.assess_capabilities();
        raw_pages.push(fallback_page);
    }

    Ok(raw_pages)
}

fn parse_mediabox(dict: &str) -> Option<(f32, f32)> {
    let mb_pos = dict.find("/MediaBox")?;
    let start = dict[mb_pos..].find('[')? + mb_pos + 1;
    let end = dict[start..].find(']')? + start;
    let parts: Vec<f32> = dict[start..end]
        .split_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();

    if parts.len() >= 4 {
        let width = (parts[2] - parts[0]).abs();
        let height = (parts[3] - parts[1]).abs();
        Some((width, height))
    } else {
        None
    }
}

fn extract_fonts_for_page(pdf_str: &str, _page_dict: &str, fonts: &mut HashMap<String, FontMap>) {
    // Scan PDF for /BaseFont declarations
    let mut search = 0;
    while let Some(pos) = pdf_str[search..].find("/BaseFont") {
        let abs_pos = search + pos + 9;
        let token = pdf_str[abs_pos..].split_whitespace().next().unwrap_or("");
        let base_font_name = token.trim_start_matches('/').to_string();

        let font_id = format!("F{}", fonts.len() + 1);
        let mut font_map = FontMap::new(font_id.clone(), base_font_name);

        // Check for ToUnicode CMap
        if let Some(cmap_pos) = pdf_str[abs_pos..].find("beginbfchar") {
            let cmap_start = abs_pos + cmap_pos;
            let cmap_end = pdf_str[cmap_start..].find("endcmap").map(|p| cmap_start + p).unwrap_or(pdf_str.len().min(cmap_start + 4096));
            font_map.parse_to_unicode_cmap(&pdf_str[cmap_start..cmap_end]);
        }

        fonts.insert(font_id, font_map);
        search = abs_pos + token.len();
        if fonts.len() > 64 {
            break;
        }
    }
}

fn extract_contents_stream(
    raw_bytes: &[u8],
    pdf_str: &str,
    page_dict: &str,
    limits: &SecurityLimits,
) -> Result<Option<Vec<u8>>, PdfError> {
    if let Some(stream_pos) = page_dict.find("stream") {
        let start_pos = stream_pos + 6;
        let start_pos = if page_dict.as_bytes().get(start_pos) == Some(&b'\r') && page_dict.as_bytes().get(start_pos + 1) == Some(&b'\n') {
            start_pos + 2
        } else if page_dict.as_bytes().get(start_pos) == Some(&b'\n') {
            start_pos + 1
        } else {
            start_pos
        };

        if let Some(end_pos) = page_dict[start_pos..].find("endstream") {
            let raw_slice = &page_dict.as_bytes()[start_pos..start_pos + end_pos];
            let is_flate = page_dict.contains("/FlateDecode");
            let filter = if is_flate { Some("FlateDecode") } else { None };
            let decoded = decode_stream(raw_slice, filter, limits)?;
            return Ok(Some(decoded));
        }
    }

    // Check if /Contents points to an indirect object like `/Contents 12 0 R`
    if let Some(contents_idx) = page_dict.find("/Contents") {
        let token = &page_dict[contents_idx + 9..];
        let parts: Vec<&str> = token.split_whitespace().take(3).collect();
        if parts.len() >= 2 && parts[1] == "0" {
            let obj_pattern = format!("{} 0 obj", parts[0]);
            if let Some(obj_pos) = pdf_str.find(&obj_pattern) {
                let obj_slice = &pdf_str[obj_pos..];
                if let Some(stream_pos) = obj_slice.find("stream") {
                    let mut stream_start = obj_pos + stream_pos + 6;
                    if pdf_str.as_bytes().get(stream_start) == Some(&b'\r') && pdf_str.as_bytes().get(stream_start + 1) == Some(&b'\n') {
                        stream_start += 2;
                    } else if pdf_str.as_bytes().get(stream_start) == Some(&b'\n') {
                        stream_start += 1;
                    }

                    if let Some(stream_end_rel) = pdf_str[stream_start..].find("endstream") {
                        let stream_end = stream_start + stream_end_rel;
                        let raw_slice = &raw_bytes[stream_start..stream_end];
                        let is_flate = obj_slice.contains("/FlateDecode");
                        let filter = if is_flate { Some("FlateDecode") } else { None };
                        let decoded = decode_stream(raw_slice, filter, limits)?;
                        return Ok(Some(decoded));
                    }
                }
            }
        }
    }

    Ok(None)
}
