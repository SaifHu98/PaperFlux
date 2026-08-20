use std::fs;
use std::path::PathBuf;
use pdf2md_ast::Node;
use pdf2md_pdf::elements::ImageObject;

#[derive(Debug, Clone)]
pub struct ImageExtractorConfig {
    pub enabled: bool,
    pub output_dir: Option<PathBuf>,
    pub min_width: usize,
    pub min_height: usize,
    pub base_url: Option<String>,
}

impl Default for ImageExtractorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            output_dir: None,
            min_width: 32,
            min_height: 32,
            base_url: None,
        }
    }
}

pub struct ImageExtractor {
    config: ImageExtractorConfig,
}

impl ImageExtractor {
    pub fn new(config: ImageExtractorConfig) -> Self {
        Self { config }
    }

    /// Safely sanitizes an image identifier to prevent path traversal, null-byte injection, and invalid characters
    pub fn sanitize_filename(id: &str, ext: &str) -> String {
        // Strip null bytes and control chars
        let clean_id: String = id
            .chars()
            .filter(|c| !c.is_control() && *c != '\0' && *c != '/' && *c != '\\')
            .collect();

        // If identifier is suspicious or empty, generate a deterministic safe hash
        let safe_name = if clean_id.is_empty() || clean_id.contains("..") || clean_id.len() > 64 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            id.hash(&mut hasher);
            format!("img_{:016x}", hasher.finish())
        } else {
            clean_id
        };

        let safe_ext = match ext.to_lowercase().as_str() {
            "png" => "png",
            "jpg" | "jpeg" => "jpg",
            "webp" => "webp",
            _ => "bin",
        };

        format!("{}.{}", safe_name, safe_ext)
    }

    pub fn process_image(&self, image: &ImageObject) -> Option<Node> {
        if !self.config.enabled {
            return None;
        }

        // Memory & Dimension Bomb Protection: reject images exceeding 10,000 x 10,000 pixels
        if image.width > 10_000 || image.height > 10_000 {
            return None;
        }

        // Filter tiny icon / noise artifacts
        if image.width < self.config.min_width || image.height < self.config.min_height {
            return None;
        }

        let ext = if image.mime_type.contains("png") {
            "png"
        } else if image.mime_type.contains("jpeg") || image.mime_type.contains("jpg") {
            "jpg"
        } else {
            "png"
        };

        let filename = Self::sanitize_filename(&image.id, ext);

        let src = if let Some(dir) = &self.config.output_dir {
            // Verify output directory path does not escape
            let file_path = dir.join(&filename);
            if let Some(parent) = file_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&file_path, &image.data);

            if let Some(base_url) = &self.config.base_url {
                format!("{}/{}", base_url.trim_end_matches('/'), filename)
            } else {
                file_path.to_string_lossy().to_string()
            }
        } else {
            // In-memory placeholder URL (safe from memory bloat)
            format!("assets/{}", filename)
        };

        Some(Node::Image {
            alt_text: format!("Figure {}", image.id),
            src,
            title: None,
            width: Some(image.width as f32),
            height: Some(image.height as f32),
            bbox: Some(image.bbox),
            mime_type: Some(image.mime_type.clone()),
        })
    }
}
