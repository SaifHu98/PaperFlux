use crate::profile::ExecutionProfile;
use pdf2md_markdown::{MarkdownDialect, PageBreakStyle, RenderOptions};
use pdf2md_ocr::{OCRProvider, OcrMode};
use pdf2md_pdf::security::SecurityLimits;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub profile: ExecutionProfile,
    pub dialect: MarkdownDialect,
    pub emit_frontmatter: bool,
    pub page_breaks: PageBreakStyle,
    pub extract_images: bool,
    pub images_dir: Option<PathBuf>,
    pub detect_tables: bool,
    pub paragraph_gap_threshold: Option<f32>,
    pub ocr_mode: OcrMode,
    pub ocr_provider: Option<Arc<dyn OCRProvider>>,
    pub ocr_dpi: Option<u32>,
    pub auto_calligraphy_dpi_boost: bool,
    pub calligraphic_dpi_escalation: bool,
    pub security_limits: SecurityLimits,
    pub deterministic: bool,
    pub enable_caching: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: ExecutionProfile::Balanced,
            dialect: MarkdownDialect::GitHubFlavored,
            emit_frontmatter: true,
            page_breaks: PageBreakStyle::HtmlComment,
            extract_images: false,
            images_dir: None,
            detect_tables: true,
            paragraph_gap_threshold: None,
            ocr_mode: OcrMode::Auto,
            ocr_provider: None,
            ocr_dpi: None,
            auto_calligraphy_dpi_boost: true,
            calligraphic_dpi_escalation: true,
            security_limits: SecurityLimits::default(),
            deterministic: true,
            enable_caching: true,
        }
    }
}

pub struct ConfigBuilder {
    config: Config,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn profile(mut self, profile: ExecutionProfile) -> Self {
        self.config.profile = profile;
        self.config.enable_caching = profile.enable_caching();
        self
    }

    pub fn dialect(mut self, dialect: MarkdownDialect) -> Self {
        self.config.dialect = dialect;
        self
    }

    pub fn emit_frontmatter(mut self, emit: bool) -> Self {
        self.config.emit_frontmatter = emit;
        self
    }

    pub fn page_breaks(mut self, style: PageBreakStyle) -> Self {
        self.config.page_breaks = style;
        self
    }

    pub fn extract_images(mut self, extract: bool) -> Self {
        self.config.extract_images = extract;
        self
    }

    pub fn images_dir(mut self, dir: PathBuf) -> Self {
        self.config.images_dir = Some(dir);
        self
    }

    pub fn detect_tables(mut self, detect: bool) -> Self {
        self.config.detect_tables = detect;
        self
    }

    pub fn paragraph_gap_threshold(mut self, threshold: f32) -> Self {
        self.config.paragraph_gap_threshold = Some(threshold);
        self
    }

    pub fn ocr_mode(mut self, mode: OcrMode) -> Self {
        self.config.ocr_mode = mode;
        self
    }

    pub fn ocr_provider(mut self, provider: Arc<dyn OCRProvider>) -> Self {
        self.config.ocr_provider = Some(provider);
        self
    }

    pub fn ocr_dpi(mut self, dpi: u32) -> Self {
        self.config.ocr_dpi = Some(dpi);
        self
    }

    pub fn auto_calligraphy_dpi_boost(mut self, boost: bool) -> Self {
        self.config.auto_calligraphy_dpi_boost = boost;
        self.config.calligraphic_dpi_escalation = boost;
        self
    }

    pub fn calligraphic_dpi_escalation(mut self, enabled: bool) -> Self {
        self.config.calligraphic_dpi_escalation = enabled;
        self.config.auto_calligraphy_dpi_boost = enabled;
        self
    }

    pub fn memory_limit_mb(mut self, mb: usize) -> Self {
        self.config.security_limits.max_decompressed_stream_bytes = mb * 1024 * 1024;
        self
    }

    pub fn max_pages(mut self, max: usize) -> Self {
        self.config.security_limits.max_pages = max;
        self
    }

    pub fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.security_limits.timeout_seconds = timeout;
        self
    }

    pub fn deterministic(mut self, deterministic: bool) -> Self {
        self.config.deterministic = deterministic;
        self
    }

    pub fn enable_caching(mut self, enable: bool) -> Self {
        self.config.enable_caching = enable;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    pub fn to_render_options(&self) -> RenderOptions {
        RenderOptions {
            dialect: self.dialect,
            emit_frontmatter: self.emit_frontmatter,
            page_breaks: self.page_breaks,
            allow_html_tables_for_spans: true,
            max_column_width: 80,
        }
    }
}
