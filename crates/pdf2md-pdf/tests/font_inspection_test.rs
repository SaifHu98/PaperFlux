use pdf2md_pdf::PdfDocument;
use std::fs;
use std::path::Path;

#[test]
fn test_inspect_pdf_fonts() {
    let pdf_path = Path::new("C:/Users/saifx/Desktop/طب حياتي/ثالث بولونيا دور ثاني.pdf");
    if !pdf_path.exists() {
        return;
    }
    let pdf_bytes = fs::read(pdf_path).unwrap();
    let doc = PdfDocument::parse(&pdf_bytes, pdf2md_pdf::security::SecurityLimits::default()).unwrap();

    println!("Total pages: {}", doc.pages.len());
    for (i, page) in doc.pages.iter().take(2).enumerate() {
        println!("\n--- Page {} Text Spans (first 20) ---", i + 1);
        for s in page.text_spans.iter().take(20) {
            println!("Font: '{}' | Text: '{}' | Unicode Codepoints: {:X?}", s.font_name, s.text, s.text.chars().map(|c| c as u32).collect::<Vec<_>>());
        }
    }
}
