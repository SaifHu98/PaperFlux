use crate::elements::{ImageObject, RawPage};
use crate::font::FontMap;
use crate::parser::ContentStreamParser;
use crate::security::{SecurityError, SecurityLimits};
use crate::stream::decode_stream;
use pdf2md_ast::geometry::BoundingBox;
use pdf2md_ast::DocumentMetadata;
use std::collections::HashMap;
use thiserror::Error;

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

        let first_line = bytes
            .split(|&b| b == b'\n' || b == b'\r')
            .next()
            .unwrap_or(b"%PDF-1.4");
        let version = Some(String::from_utf8_lossy(first_line).trim().to_string());

        let mut metadata = DocumentMetadata {
            pdf_version: version,
            total_pages: 0,
            ..Default::default()
        };

        // Extract metadata from /Info dictionary if present
        extract_metadata(bytes, &mut metadata);

        // Extract and process pages using pure byte-level parsing
        let pages = extract_pages(bytes, &limits)?;
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

pub fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

pub fn rfind_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).rposition(|w| w == needle)
}

fn extract_metadata(bytes: &[u8], meta: &mut DocumentMetadata) {
    if let Some(info_pos) = find_subslice(bytes, b"/Title") {
        if let Some(val) = extract_pdf_string_value(&bytes[info_pos..]) {
            meta.title = Some(val);
        }
    }
    if let Some(author_pos) = find_subslice(bytes, b"/Author") {
        if let Some(val) = extract_pdf_string_value(&bytes[author_pos..]) {
            meta.author = Some(val);
        }
    }
    if let Some(subject_pos) = find_subslice(bytes, b"/Subject") {
        if let Some(val) = extract_pdf_string_value(&bytes[subject_pos..]) {
            meta.subject = Some(val);
        }
    }
    if let Some(creator_pos) = find_subslice(bytes, b"/Creator") {
        if let Some(val) = extract_pdf_string_value(&bytes[creator_pos..]) {
            meta.creator = Some(val);
        }
    }
    if let Some(producer_pos) = find_subslice(bytes, b"/Producer") {
        if let Some(val) = extract_pdf_string_value(&bytes[producer_pos..]) {
            meta.producer = Some(val);
        }
    }
}

fn extract_pdf_string_value(slice: &[u8]) -> Option<String> {
    let start = slice.iter().position(|&b| b == b'(')?;
    let end = slice[start..].iter().position(|&b| b == b')')? + start;
    if end > start {
        Some(
            String::from_utf8_lossy(&slice[start + 1..end])
                .trim()
                .to_string(),
        )
    } else {
        None
    }
}

fn extract_pages(raw_bytes: &[u8], limits: &SecurityLimits) -> Result<Vec<RawPage>, PdfError> {
    let mut raw_pages = Vec::new();
    let mut page_num = 1;

    let page_pattern = b"/Type /Page";
    let mut search_idx = 0;

    while let Some(pos) = find_subslice(&raw_bytes[search_idx..], page_pattern) {
        let abs_pos = search_idx + pos;
        let next_byte = raw_bytes.get(abs_pos + page_pattern.len());

        // Skip /Type /Pages
        if next_byte == Some(&b's') || next_byte == Some(&b'S') {
            search_idx = abs_pos + page_pattern.len() + 1;
            continue;
        }

        // Find enclosing object `n m obj ... endobj`
        let obj_start =
            rfind_subslice(&raw_bytes[..abs_pos], b"obj").unwrap_or(abs_pos.saturating_sub(100));
        let obj_end = find_subslice(&raw_bytes[abs_pos..], b"endobj")
            .map(|p| abs_pos + p)
            .unwrap_or(raw_bytes.len());

        let page_dict = &raw_bytes[obj_start..obj_end];

        // Parse MediaBox [x y w h]
        let (width, height) = parse_mediabox(page_dict).unwrap_or((612.0, 792.0));
        let mut raw_page = RawPage::new(page_num, width, height);

        // Find fonts referenced in /Resources /Font
        let mut fonts: HashMap<String, FontMap> = HashMap::new();
        extract_fonts_for_page(raw_bytes, page_dict, &mut fonts);

        // Find and decode /Contents stream
        if let Ok(Some(contents_stream)) = extract_contents_stream(raw_bytes, page_dict, limits) {
            let mut parser = ContentStreamParser::new(&fonts, height);
            parser.parse_content_stream(&contents_stream, &mut raw_page);
        }

        // Extract any embedded images referenced in page resources (e.g. scanned full-page images)
        extract_images_for_page(raw_bytes, page_dict, &mut raw_page, limits);

        raw_page.assess_capabilities();
        raw_pages.push(raw_page);

        page_num += 1;
        search_idx = obj_end;
        if page_num > limits.max_pages {
            break;
        }
    }

    // Fallback if no /Type /Page found
    if raw_pages.is_empty() {
        let mut fallback_page = RawPage::new(1, 612.0, 792.0);
        let fonts = HashMap::new();
        let mut parser = ContentStreamParser::new(&fonts, 792.0);

        let mut stream_search = 0;
        while let Some(start_stream) = find_subslice(&raw_bytes[stream_search..], b"stream") {
            let mut stream_body_start = stream_search + start_stream + 6;
            if raw_bytes.get(stream_body_start) == Some(&b'\r')
                && raw_bytes.get(stream_body_start + 1) == Some(&b'\n')
            {
                stream_body_start += 2;
            } else if raw_bytes.get(stream_body_start) == Some(&b'\n') {
                stream_body_start += 1;
            }

            if let Some(end_stream) = find_subslice(&raw_bytes[stream_body_start..], b"endstream") {
                let stream_body_end = stream_body_start + end_stream;
                if stream_body_end <= raw_bytes.len() {
                    let raw_slice = &raw_bytes[stream_body_start..stream_body_end];
                    if let Ok(decompressed) = decode_stream(raw_slice, Some("FlateDecode"), limits)
                    {
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

fn parse_mediabox(dict: &[u8]) -> Option<(f32, f32)> {
    let mb_pos = find_subslice(dict, b"/MediaBox")?;
    let start = dict[mb_pos..].iter().position(|&b| b == b'[')? + mb_pos + 1;
    let end = dict[start..].iter().position(|&b| b == b']')? + start;
    let str_slice = String::from_utf8_lossy(&dict[start..end]);
    let parts: Vec<f32> = str_slice
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

fn extract_fonts_for_page(
    raw_bytes: &[u8],
    _page_dict: &[u8],
    fonts: &mut HashMap<String, FontMap>,
) {
    let mut search = 0;
    let base_font_pat = b"/BaseFont";
    while let Some(pos) = find_subslice(&raw_bytes[search..], base_font_pat) {
        let abs_pos = search + pos + base_font_pat.len();
        let slice = &raw_bytes[abs_pos..abs_pos + 200.min(raw_bytes.len() - abs_pos)];
        let str_val = String::from_utf8_lossy(slice);
        let token = str_val.split_whitespace().next().unwrap_or("");
        let base_font_name = token.trim_start_matches('/').to_string();

        let font_id = format!("F{}", fonts.len() + 1);
        let mut font_map = FontMap::new(font_id.clone(), base_font_name);

        // Check for ToUnicode CMap
        if let Some(cmap_pos) = find_subslice(&raw_bytes[abs_pos..], b"beginbfchar") {
            let cmap_start = abs_pos + cmap_pos;
            let cmap_end = find_subslice(&raw_bytes[cmap_start..], b"endcmap")
                .map(|p| cmap_start + p)
                .unwrap_or(raw_bytes.len().min(cmap_start + 4096));
            font_map
                .parse_to_unicode_cmap(&String::from_utf8_lossy(&raw_bytes[cmap_start..cmap_end]));
        }

        fonts.insert(font_id, font_map);
        search = abs_pos + token.len().max(1);
        if fonts.len() > 64 {
            break;
        }
    }
}

fn extract_contents_stream(
    raw_bytes: &[u8],
    page_dict: &[u8],
    limits: &SecurityLimits,
) -> Result<Option<Vec<u8>>, PdfError> {
    // 1. Direct stream in page dictionary
    if let Some(stream_pos) = find_subslice(page_dict, b"stream") {
        let mut start_pos = stream_pos + 6;
        if page_dict.get(start_pos) == Some(&b'\r') && page_dict.get(start_pos + 1) == Some(&b'\n')
        {
            start_pos += 2;
        } else if page_dict.get(start_pos) == Some(&b'\n') {
            start_pos += 1;
        }

        if let Some(end_pos) = find_subslice(&page_dict[start_pos..], b"endstream") {
            let raw_slice = &page_dict[start_pos..start_pos + end_pos];
            let is_flate = find_subslice(page_dict, b"/FlateDecode").is_some();
            let filter = if is_flate { Some("FlateDecode") } else { None };
            let decoded = decode_stream(raw_slice, filter, limits)?;
            return Ok(Some(decoded));
        }
    }

    // 2. Indirect stream reference `/Contents 12 0 R`
    if let Some(contents_idx) = find_subslice(page_dict, b"/Contents") {
        let token_slice =
            &page_dict[contents_idx + 9..contents_idx + 40.min(page_dict.len() - contents_idx)];
        let token_str = String::from_utf8_lossy(token_slice);
        let parts: Vec<&str> = token_str.split_whitespace().take(3).collect();
        if parts.len() >= 2 && parts[1] == "0" {
            let obj_pattern = format!("{} 0 obj", parts[0]);
            if let Some(obj_pos) = find_subslice(raw_bytes, obj_pattern.as_bytes()) {
                let obj_slice = &raw_bytes[obj_pos..];
                if let Some(stream_pos) = find_subslice(obj_slice, b"stream") {
                    let mut stream_start = stream_pos + 6;
                    if obj_slice.get(stream_start) == Some(&b'\r')
                        && obj_slice.get(stream_start + 1) == Some(&b'\n')
                    {
                        stream_start += 2;
                    } else if obj_slice.get(stream_start) == Some(&b'\n') {
                        stream_start += 1;
                    }

                    if let Some(stream_end_rel) =
                        find_subslice(&obj_slice[stream_start..], b"endstream")
                    {
                        let stream_end = stream_start + stream_end_rel;
                        let raw_slice = &obj_slice[stream_start..stream_end];
                        let is_flate =
                            find_subslice(&obj_slice[..stream_start], b"/FlateDecode").is_some();
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

fn extract_images_for_page(
    raw_bytes: &[u8],
    page_dict: &[u8],
    raw_page: &mut RawPage,
    _limits: &SecurityLimits,
) {
    // Find /Resources in page dictionary
    let mut xobject_refs = Vec::new();
    let dict_str = String::from_utf8_lossy(page_dict);

    if let Some(res_pos) = find_subslice(page_dict, b"/Resources") {
        let res_slice = &page_dict[res_pos + 10..res_pos + 40.min(page_dict.len() - res_pos)];
        let res_str = String::from_utf8_lossy(res_slice);
        let parts: Vec<&str> = res_str.split_whitespace().take(3).collect();
        if parts.len() >= 2 && parts[1] == "0" {
            // Indirect /Resources N 0 R
            let obj_pattern = format!("{} 0 obj", parts[0]);
            if let Some(res_obj_pos) = find_subslice(raw_bytes, obj_pattern.as_bytes()) {
                let res_end = find_subslice(&raw_bytes[res_obj_pos..], b"endobj")
                    .map(|p| res_obj_pos + p)
                    .unwrap_or(raw_bytes.len().min(res_obj_pos + 2048));
                let res_obj_bytes = &raw_bytes[res_obj_pos..res_end];
                extract_xobject_ids_from_dict(res_obj_bytes, &mut xobject_refs);
            }
        } else {
            // Direct /Resources dictionary
            extract_xobject_ids_from_dict(page_dict, &mut xobject_refs);
        }
    } else if dict_str.contains("/XObject") {
        extract_xobject_ids_from_dict(page_dict, &mut xobject_refs);
    }

    for (img_id, obj_num) in xobject_refs {
        let obj_pattern = format!("{} 0 obj", obj_num);
        if let Some(obj_pos) = find_subslice(raw_bytes, obj_pattern.as_bytes()) {
            let obj_end = find_subslice(&raw_bytes[obj_pos..], b"endobj")
                .map(|p| obj_pos + p)
                .unwrap_or(raw_bytes.len());
            let obj_bytes = &raw_bytes[obj_pos..obj_end];

            if find_subslice(obj_bytes, b"/Subtype /Image").is_some()
                || find_subslice(obj_bytes, b"/Subtype/Image").is_some()
            {
                let width =
                    parse_dict_number(obj_bytes, b"/Width").unwrap_or(raw_page.width as usize);
                let height =
                    parse_dict_number(obj_bytes, b"/Height").unwrap_or(raw_page.height as usize);
                let is_dct = find_subslice(obj_bytes, b"/DCTDecode").is_some();
                let mime_type = if is_dct { "image/jpeg" } else { "image/png" }.to_string();

                if let Some(s_pos) = find_subslice(obj_bytes, b"stream") {
                    let mut s_start = s_pos + 6;
                    if obj_bytes.get(s_start) == Some(&b'\r')
                        && obj_bytes.get(s_start + 1) == Some(&b'\n')
                    {
                        s_start += 2;
                    } else if obj_bytes.get(s_start) == Some(&b'\n') {
                        s_start += 1;
                    }

                    if let Some(s_end_rel) = find_subslice(&obj_bytes[s_start..], b"endstream") {
                        let mut img_data = &obj_bytes[s_start..s_start + s_end_rel];
                        while img_data.ends_with(b"\n")
                            || img_data.ends_with(b"\r")
                            || img_data.ends_with(b" ")
                        {
                            img_data = &img_data[..img_data.len() - 1];
                        }

                        let image_object = ImageObject {
                            id: format!(
                                "page_{}_{}",
                                raw_page.page_number,
                                img_id.trim_start_matches('/')
                            ),
                            bbox: BoundingBox::new(0.0, 0.0, raw_page.width, raw_page.height),
                            width,
                            height,
                            mime_type,
                            data: img_data.to_vec(),
                        };
                        raw_page.images.push(image_object);
                    }
                }
            }
        }
    }
}

fn extract_xobject_ids_from_dict(dict: &[u8], refs: &mut Vec<(String, usize)>) {
    let clean = String::from_utf8_lossy(dict)
        .replace(['<', '>'], " ")
        .replace('/', " /");
    let mut tokens = clean.split_whitespace();
    let mut in_xobject = false;

    while let Some(tok) = tokens.next() {
        if tok == "/XObject" {
            in_xobject = true;
            continue;
        }
        if in_xobject && tok.starts_with('/') {
            let name = tok.to_string();
            if let Some(id_str) = tokens.next() {
                if let Ok(id) = id_str.parse::<usize>() {
                    if let Some(gen_str) = tokens.next() {
                        if gen_str == "0" {
                            if let Some(r_str) = tokens.next() {
                                if r_str == "R" {
                                    refs.push((name, id));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn parse_dict_number(dict: &[u8], key: &[u8]) -> Option<usize> {
    let pos = find_subslice(dict, key)?;
    let slice = &dict[pos + key.len()..pos + key.len() + 20.min(dict.len() - pos - key.len())];
    let s = String::from_utf8_lossy(slice);
    s.split_whitespace().next()?.parse::<usize>().ok()
}
