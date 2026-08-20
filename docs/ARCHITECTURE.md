# pdf2md Architecture & Pipeline Specification

## 1. Pipeline Stages

The `pdf2md` conversion pipeline processes documents in a series of decoupled, deterministic stages:

```
[PDF Binary Source]
       │
       ▼
[Stage 1: Validation & Security Boundary]
   • Verify magic header `%PDF-1.x`
   • Check file size against `max_file_size_bytes`
   • Check total pages against `max_pages`
       │
       ▼
[Stage 2: Stream & Font Decompression]
   • Object table / cross-reference traversal
   • FlateDecode / ASCIIHexDecode with decompression-bomb guards (100x max ratio)
   • Parse ToUnicode CMaps and glyph mapping
       │
       ▼
[Stage 3: Per-Page Capability Assessment]
   • Count extractable character glyphs
   • Classify page: Usable Digital Text vs. Scanned / Image-Only
   • Trigger OCR provider selectively if page lacks text but contains images
       │
       ▼
[Stage 4: Typography & Script Normalization]
   • Ligature unfolding (e.g. `\u{FB01}` fi -> `fi`)
   • Arabic Presentation Forms normalized to base Unicode (U+0600..U+06FF)
   • BiDi / RTL logical order reordering
   • CJK ideograph boundary detection to avoid inserting unwanted spaces
   • Hyphenation merge across line-breaks
       │
       ▼
[Stage 5: Table Extraction]
   • Lattice vector line detector (horizontal/vertical lines + intersections)
   • Borderless whitespace projection profile detector
   • Cell bounding box and colspan/rowspan resolution
       │
       ▼
[Stage 6: Layout Segmentation & Reading Order]
   • Strips repeated headers, footers, and isolated page numbers
   • Computes dominant body font size
   • Recursive XY-Cut++: slices page along vertical and horizontal whitespace gutters
   • Clusters lines and reconstructs paragraphs, headings (H1-H6), and nested lists
       │
       ▼
[Stage 7: Document AST Builder]
   • Assembles typed AST nodes (Heading, Paragraph, Table, List, Image, CodeBlock)
   • Attaches confidence metrics and bounding boxes
       │
       ▼
[Stage 8: Markdown Serializer]
   • Renders CommonMark / GitHub Flavored Markdown (GFM) / Extended
   • Generates YAML frontmatter metadata
   • Formats GFM tables with HTML fallback for complex merged cells
   • Emits machine-readable diagnostics JSON
```

---

## 2. Document AST Specification

```rust
pub struct Document {
    pub metadata: DocumentMetadata,
    pub sections: Vec<Section>,
    pub diagnostics: ConversionDiagnostics,
}
```

Nodes include:
- `Heading`: Level 1-6, inlines, confidence score.
- `Paragraph`: Inlines, confidence score.
- `CodeBlock`: Language tag, code text.
- `BlockQuote`: Children nodes.
- `List`: Ordered / Unordered, starting number, nested items.
- `Table`: Headers, data rows, caption, border flag, confidence score.
- `Image`: Source path, alt text, title, dimensions, mime-type.
- `Footnote`: Unique ID, inlines.
- `Caption`: Target type (Figure/Table/Equation), inlines.
- `Formula`: LaTeX string, inline / block flag.
- `PageBreak`: Page number marker.

---

## 3. Pluggable OCR Interface

```rust
pub trait OCRProvider: Send + Sync {
    fn detect_language(&self, image_data: &[u8]) -> Result<String, OcrError>;
    fn recognize(&self, image_data: &[u8], lang: Option<&str>) -> Result<OcrResult, OcrError>;
    fn detect_orientation(&self, image_data: &[u8]) -> Result<i32, OcrError>;
    fn confidence(&self) -> f32;
    fn available_languages(&self) -> Vec<String>;
}
```
