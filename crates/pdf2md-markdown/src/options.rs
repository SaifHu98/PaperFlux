use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MarkdownDialect {
    CommonMark,
    #[default]
    GitHubFlavored,
    Extended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PageBreakStyle {
    None,
    ThematicBreak, // ---
    #[default]
    HtmlComment, // <!-- pagebreak -->
    CustomMarker,  // [[Page X]]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderOptions {
    pub dialect: MarkdownDialect,
    pub emit_frontmatter: bool,
    pub page_breaks: PageBreakStyle,
    pub allow_html_tables_for_spans: bool,
    pub max_column_width: usize,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            dialect: MarkdownDialect::GitHubFlavored,
            emit_frontmatter: true,
            page_breaks: PageBreakStyle::HtmlComment,
            allow_html_tables_for_spans: true,
            max_column_width: 80,
        }
    }
}
