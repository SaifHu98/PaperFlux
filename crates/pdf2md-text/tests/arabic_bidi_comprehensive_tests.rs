use pdf2md_ast::geometry::WritingDirection;
use pdf2md_text::arabic::bidi_engine::{ArabicBidiEngine, BidiTokenizer};
use pdf2md_text::arabic::context::ArabicProcessingContext;
use pdf2md_text::arabic::pipeline::ArabicTextPipeline;

#[test]
fn test_bidi_arabic_only_text() {
    let text = "الذكاء الاصطناعي ومعالجة المستندات المعقدة بدقة فائقة.";
    let dir = ArabicBidiEngine::detect_paragraph_direction(text);
    assert_eq!(dir, WritingDirection::RightToLeft);

    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(text, &ctx);
    assert_eq!(processed, text);
}

#[test]
fn test_bidi_arabic_english_mixed_sentence() {
    // "تم نشر الإصدار PaperFlux 2.0 في عام 2026"
    let text = "تم نشر الإصدار PaperFlux 2.0 في عام 2026.";
    let dir = ArabicBidiEngine::detect_paragraph_direction(text);
    assert_eq!(dir, WritingDirection::RightToLeft);

    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(text, &ctx);

    assert!(processed.contains("PaperFlux"));
    assert!(processed.contains("2.0"));
    assert!(processed.contains("2026"));
    assert!(processed.contains("تم نشر الإصدار"));
}

#[test]
fn test_bidi_english_arabic_mixed_sentence() {
    let text = "The system supports اللغة العربية and Persian natively.";
    let dir = ArabicBidiEngine::detect_paragraph_direction(text);
    assert_eq!(dir, WritingDirection::LeftToRight);

    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(text, &ctx);

    assert!(processed.contains("The system supports"));
    assert!(processed.contains("اللغة العربية"));
    assert!(processed.contains("natively"));
}

#[test]
fn test_bidi_arabic_with_eastern_and_western_numbers() {
    let eastern = "بلغت نسبة النجاح ٩٨٫٥٪ لعدد ١٥٬٤٥٠ طالباً.";
    let western = "بلغت نسبة النجاح 98.5% لعدد 15,450 طالباً.";

    let ctx = ArabicProcessingContext::default();
    let (proc_eastern, _) = ArabicTextPipeline::process(eastern, &ctx);
    let (proc_western, _) = ArabicTextPipeline::process(western, &ctx);

    assert!(proc_eastern.contains("٩٨٫٥٪"));
    assert!(proc_eastern.contains("١٥٬٤٥٠"));

    assert!(proc_western.contains("98.5%"));
    assert!(proc_western.contains("15,450"));
}

#[test]
fn test_bidi_arabic_with_embedded_urls() {
    let text =
        "للمزيد من المعلومات يرجى زيارة الموقع https://github.com/ecouni/paperflux للمتابعة.";
    let tokens = BidiTokenizer::tokenize(text);
    let url_token = tokens
        .iter()
        .find(|t| t.kind == pdf2md_text::arabic::bidi_engine::BidiTokenKind::Url);

    assert!(url_token.is_some());
    assert_eq!(
        url_token.unwrap().text,
        "https://github.com/ecouni/paperflux"
    );

    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(text, &ctx);
    assert!(processed.contains("https://github.com/ecouni/paperflux"));
}

#[test]
fn test_bidi_arabic_with_embedded_emails() {
    let text = "للتواصل مع فريق التطوير راسلنا على dev@ecouni.org مباشرة.";
    let tokens = BidiTokenizer::tokenize(text);
    let email_token = tokens
        .iter()
        .find(|t| t.kind == pdf2md_text::arabic::bidi_engine::BidiTokenKind::Email);

    assert!(email_token.is_some());
    assert_eq!(email_token.unwrap().text, "dev@ecouni.org");

    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(text, &ctx);
    assert!(processed.contains("dev@ecouni.org"));
}

#[test]
fn test_bidi_arabic_with_code_fragments() {
    let text = "قم بتنفيذ الأمر `cargo install paperflux` لبدء الاستخدام.";
    let tokens = BidiTokenizer::tokenize(text);
    let code_token = tokens
        .iter()
        .find(|t| t.kind == pdf2md_text::arabic::bidi_engine::BidiTokenKind::CodeFragment);

    assert!(code_token.is_some());
    assert_eq!(code_token.unwrap().text, "`cargo install paperflux`");

    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(text, &ctx);
    assert!(processed.contains("`cargo install paperflux`"));
}

#[test]
fn test_bidi_arabic_with_mathematical_equations() {
    let text = "وفقاً للنسبية الخاصة فإن E = mc^2 حيث c هي سرعة الضوء.";
    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(text, &ctx);

    assert!(processed.contains("E = mc^2"));
    assert!(processed.contains("وفقاً للنسبية الخاصة"));
}

#[test]
fn test_bidi_arabic_footnotes_and_citations() {
    let text = "كما ورد في المرجع [1] انظر الصفحة 45.";
    let tokens = BidiTokenizer::tokenize(text);
    let cite_token = tokens
        .iter()
        .find(|t| t.kind == pdf2md_text::arabic::bidi_engine::BidiTokenKind::CitationOrReference);

    assert!(cite_token.is_some());
    assert_eq!(cite_token.unwrap().text, "[1]");

    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(text, &ctx);
    assert!(processed.contains("[1]"));
}

#[test]
fn test_bidi_mixed_arabic_persian_kurdish_urdu() {
    let mixed = "پروژه مشترك و ڕاپۆرتی نوێ مع دعم كامل للغة العربية و اردو زبان میں۔";
    let ctx = ArabicProcessingContext::default();
    let (processed, _) = ArabicTextPipeline::process(mixed, &ctx);

    assert!(processed.contains("پروژه"));
    assert!(processed.contains("ڕاپۆرتی"));
    assert!(processed.contains("العربية"));
    assert!(processed.contains("اردو"));
}
