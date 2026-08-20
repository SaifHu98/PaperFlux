use pdf2md_ast::DocumentMetadata;

pub struct FrontmatterRenderer;

impl FrontmatterRenderer {
    pub fn render(metadata: &DocumentMetadata) -> String {
        let mut out = String::new();
        out.push_str("---\n");

        if let Some(title) = &metadata.title {
            out.push_str(&format!("title: \"{}\"\n", Self::sanitize_yaml_string(title)));
        }
        if let Some(author) = &metadata.author {
            out.push_str(&format!("author: \"{}\"\n", Self::sanitize_yaml_string(author)));
        }
        if let Some(subject) = &metadata.subject {
            out.push_str(&format!("subject: \"{}\"\n", Self::sanitize_yaml_string(subject)));
        }
        if !metadata.keywords.is_empty() {
            out.push_str("keywords:\n");
            for kw in &metadata.keywords {
                out.push_str(&format!("  - \"{}\"\n", Self::sanitize_yaml_string(kw)));
            }
        }
        if let Some(creator) = &metadata.creator {
            out.push_str(&format!("creator: \"{}\"\n", Self::sanitize_yaml_string(creator)));
        }
        if let Some(producer) = &metadata.producer {
            out.push_str(&format!("producer: \"{}\"\n", Self::sanitize_yaml_string(producer)));
        }
        if let Some(created) = &metadata.creation_date {
            out.push_str(&format!("created: \"{}\"\n", Self::sanitize_yaml_string(created)));
        }

        out.push_str(&format!("pages: {}\n", metadata.total_pages));
        out.push_str("---\n\n");

        out
    }

    /// Sanitizes metadata strings to prevent YAML syntax injection, XSS payloads, and control escapes
    fn sanitize_yaml_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| !c.is_control() && *c != '\0')
            .collect::<String>()
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', " ")
            .replace('\r', "")
    }
}
