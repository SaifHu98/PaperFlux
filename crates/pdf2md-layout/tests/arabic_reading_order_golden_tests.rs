use pdf2md_ast::geometry::BoundingBox;
use pdf2md_layout::arabic_reading_order::ArabicReadingOrderEngine;
use pdf2md_pdf::elements::TextSpan;

fn make_span(text: &str, x: f32, y: f32, w: f32, h: f32, size: f32) -> TextSpan {
    TextSpan::new(
        text.to_string(),
        BoundingBox::new(x, y, w, h),
        "Amiri".to_string(),
        size,
        size >= 14.0,
        false,
        false,
    )
}

#[test]
fn test_golden_arabic_one_column_book() {
    let spans = vec![
        make_span("الفصل الأول: نشأة الحضارات", 150.0, 700.0, 300.0, 24.0, 18.0),
        make_span(
            "كانت البدايات الأولى للحضارة ترتكز على ضفاف الأنهار.",
            60.0,
            660.0,
            480.0,
            14.0,
            12.0,
        ),
        make_span(
            "وقد ساهمت الزراعة في بناء أولى التجمعات البشرية المستقرة.",
            60.0,
            640.0,
            480.0,
            14.0,
            12.0,
        ),
        make_span(
            "[1] انظر كتاب تاريخ الحضارات القديمة، ص 24.",
            60.0,
            90.0,
            350.0,
            10.0,
            9.0,
        ),
    ];

    let (ordered, conf, warnings) =
        ArabicReadingOrderEngine::sequence_arabic_page(&spans, 600.0, 800.0);

    assert_eq!(ordered.len(), 4);
    assert_eq!(ordered[0].text, "الفصل الأول: نشأة الحضارات");
    assert_eq!(
        ordered[1].text,
        "كانت البدايات الأولى للحضارة ترتكز على ضفاف الأنهار."
    );
    assert_eq!(
        ordered[2].text,
        "وقد ساهمت الزراعة في بناء أولى التجمعات البشرية المستقرة."
    );
    assert_eq!(
        ordered[3].text,
        "[1] انظر كتاب تاريخ الحضارات القديمة، ص 24."
    );
    assert!(conf >= 0.95);
    assert!(warnings.is_empty());
}

#[test]
fn test_golden_arabic_two_column_academic_paper() {
    let spans = vec![
        // Spanning Title & Abstract
        make_span(
            "أثر الذكاء الاصطناعي في تحليل البيانات الضخمة",
            100.0,
            720.0,
            400.0,
            26.0,
            18.0,
        ),
        make_span(
            "ملخص: تهدف هذه الورقة البحثية إلى دراسة النماذج اللغوية...",
            100.0,
            670.0,
            400.0,
            14.0,
            11.0,
        ),
        // Right Column 1 (X: 320 -> 540)
        make_span(
            "1. المقدمة: شهدت السنوات الأخيرة تطوراً هائلاً.",
            320.0,
            600.0,
            220.0,
            14.0,
            11.0,
        ),
        make_span(
            "وقد أدى هذا التطور إلى تحسين كفاءة المعالجة.",
            320.0,
            580.0,
            220.0,
            14.0,
            11.0,
        ),
        // Left Column 2 (X: 60 -> 280)
        make_span(
            "2. منهجية البحث: اعتمدنا على تجارب معيارية.",
            60.0,
            600.0,
            220.0,
            14.0,
            11.0,
        ),
        make_span(
            "وشملت العينة أكثر من عشرة آلاف وثيقة مصنفة.",
            60.0,
            580.0,
            220.0,
            14.0,
            11.0,
        ),
    ];

    let (ordered, conf, _) = ArabicReadingOrderEngine::sequence_arabic_page(&spans, 600.0, 800.0);

    assert_eq!(ordered.len(), 6);
    // 1. Spanning title first
    assert_eq!(
        ordered[0].text,
        "أثر الذكاء الاصطناعي في تحليل البيانات الضخمة"
    );
    // 2. Abstract second
    assert_eq!(
        ordered[1].text,
        "ملخص: تهدف هذه الورقة البحثية إلى دراسة النماذج اللغوية..."
    );
    // 3. Right Column 1 MUST precede Left Column 2 in Arabic
    assert_eq!(
        ordered[2].text,
        "1. المقدمة: شهدت السنوات الأخيرة تطوراً هائلاً."
    );
    assert_eq!(
        ordered[3].text,
        "وقد أدى هذا التطور إلى تحسين كفاءة المعالجة."
    );
    assert_eq!(
        ordered[4].text,
        "2. منهجية البحث: اعتمدنا على تجارب معيارية."
    );
    assert_eq!(
        ordered[5].text,
        "وشملت العينة أكثر من عشرة آلاف وثيقة مصنفة."
    );
    assert!(conf >= 0.95);
}

#[test]
fn test_golden_arabic_three_column_newspaper() {
    let spans = vec![
        make_span(
            "عناوين الأخبار: إطلاق محرك PaperFlux الذكي",
            50.0,
            740.0,
            500.0,
            28.0,
            20.0,
        ),
        // Right Column
        make_span(
            "العمود الأيمن: تفاصيل المؤتمر الصحفي.",
            410.0,
            660.0,
            140.0,
            12.0,
            10.0,
        ),
        // Center Column
        make_span(
            "العمود الأوسط: تصريحات رئيس الفريق التقني.",
            230.0,
            660.0,
            140.0,
            12.0,
            10.0,
        ),
        // Left Column
        make_span(
            "العمود الأيسر: الخطط المستقبلية للتوسع.",
            50.0,
            660.0,
            140.0,
            12.0,
            10.0,
        ),
    ];

    let (ordered, _, _) = ArabicReadingOrderEngine::sequence_arabic_page(&spans, 600.0, 800.0);

    assert_eq!(ordered.len(), 4);
    assert_eq!(ordered[0].text, "عناوين الأخبار: إطلاق محرك PaperFlux الذكي");
    assert_eq!(ordered[1].text, "العمود الأيمن: تفاصيل المؤتمر الصحفي.");
    assert_eq!(ordered[2].text, "العمود الأوسط: تصريحات رئيس الفريق التقني.");
    assert_eq!(ordered[3].text, "العمود الأيسر: الخطط المستقبلية للتوسع.");
}

#[test]
fn test_golden_arabic_university_thesis() {
    let spans = vec![
        make_span(
            "أطروحة دكتوراه في علوم الحاسوب",
            150.0,
            740.0,
            300.0,
            20.0,
            16.0,
        ),
        make_span(
            "الباب الأول: مراجعة الأدبيات السابقة",
            150.0,
            700.0,
            300.0,
            18.0,
            14.0,
        ),
        make_span(
            "تناولت الدراسات السابقة معالجة اللغات الطبيعية من زوايا متعددة.",
            60.0,
            650.0,
            480.0,
            14.0,
            12.0,
        ),
        make_span(
            "1. انظر دراسة جامعة الملك سعود (2025).",
            60.0,
            80.0,
            300.0,
            10.0,
            9.0,
        ),
    ];

    let (ordered, conf, _) = ArabicReadingOrderEngine::sequence_arabic_page(&spans, 600.0, 800.0);

    assert_eq!(ordered[0].text, "أطروحة دكتوراه في علوم الحاسوب");
    assert_eq!(ordered[1].text, "الباب الأول: مراجعة الأدبيات السابقة");
    assert_eq!(ordered[3].text, "1. انظر دراسة جامعة الملك سعود (2025).");
    assert!(conf >= 0.95);
}

#[test]
fn test_golden_arabic_government_report_and_decree() {
    let spans = vec![
        make_span(
            "المملكة العربية السعودية - الجريدة الرسمية",
            150.0,
            750.0,
            300.0,
            18.0,
            14.0,
        ),
        make_span(
            "مرسوم ملكي رقم (م/15) بتاريخ 1445 هـ",
            150.0,
            710.0,
            300.0,
            16.0,
            13.0,
        ),
        make_span(
            "المادة الأولى: تسري أحكام هذا النظام على كافة المنشآت.",
            60.0,
            660.0,
            480.0,
            14.0,
            12.0,
        ),
        make_span(
            "المادة الثانية: ينشر هذا القرار في الجريدة الرسمية ويعمل به من تاريخه.",
            60.0,
            630.0,
            480.0,
            14.0,
            12.0,
        ),
    ];

    let (ordered, conf, _) = ArabicReadingOrderEngine::sequence_arabic_page(&spans, 600.0, 800.0);

    assert_eq!(ordered.len(), 4);
    assert_eq!(
        ordered[2].text,
        "المادة الأولى: تسري أحكام هذا النظام على كافة المنشآت."
    );
    assert!(conf >= 0.95);
}

#[test]
fn test_golden_arabic_form_survey() {
    let spans = vec![
        make_span(
            "استمارة تسجيل البيانات الشخصية",
            150.0,
            730.0,
            300.0,
            20.0,
            15.0,
        ),
        make_span(
            "الاسم الكامل: ____________________",
            60.0,
            680.0,
            480.0,
            14.0,
            11.0,
        ),
        make_span(
            "الرقم الوطني: ____________________",
            60.0,
            650.0,
            480.0,
            14.0,
            11.0,
        ),
        make_span(
            "البريد الإلكتروني: ________________",
            60.0,
            620.0,
            480.0,
            14.0,
            11.0,
        ),
    ];

    let (ordered, _, _) = ArabicReadingOrderEngine::sequence_arabic_page(&spans, 600.0, 800.0);
    assert_eq!(ordered.len(), 4);
    assert_eq!(ordered[0].text, "استمارة تسجيل البيانات الشخصية");
}

#[test]
fn test_golden_arabic_magazine_with_sidebar() {
    let spans = vec![
        make_span("مجلة التقنية والابتكار", 150.0, 740.0, 300.0, 22.0, 16.0),
        // Main body on the right (X: 200 -> 540)
        make_span(
            "المقال الرئيسي: كيف تغير النماذج التوليدية مستقبل البرمجة.",
            200.0,
            680.0,
            340.0,
            14.0,
            12.0,
        ),
        make_span(
            "لقد أصبحت أدوات الذكاء الاصطناعي شريكاً أساسياً للمطورين.",
            200.0,
            650.0,
            340.0,
            14.0,
            12.0,
        ),
        // Sidebar on the left (X: 50 -> 170)
        make_span(
            "إضاءة: حقائق سريعة عن الحوسبة السحابية.",
            50.0,
            680.0,
            120.0,
            12.0,
            9.0,
        ),
    ];

    let (ordered, _, _) = ArabicReadingOrderEngine::sequence_arabic_page(&spans, 600.0, 800.0);
    assert_eq!(ordered.len(), 4);
    assert_eq!(ordered[0].text, "مجلة التقنية والابتكار");
    assert_eq!(
        ordered[1].text,
        "المقال الرئيسي: كيف تغير النماذج التوليدية مستقبل البرمجة."
    );
}

#[test]
fn test_golden_arabic_legal_contract() {
    let spans = vec![
        make_span(
            "عقد تقديم خدمات هندسية واستشارية",
            150.0,
            730.0,
            300.0,
            20.0,
            15.0,
        ),
        make_span(
            "البند الأول: موضوع العقد ونطاق العمل",
            60.0,
            680.0,
            480.0,
            16.0,
            13.0,
        ),
        make_span(
            "يلتزم الطرف الثاني بتقديم كافة الاستشارات الفنية المتفق عليها.",
            60.0,
            650.0,
            480.0,
            14.0,
            11.0,
        ),
        make_span(
            "البند الثاني: المقابل المالي وشروط الدفع",
            60.0,
            610.0,
            480.0,
            16.0,
            13.0,
        ),
    ];

    let (ordered, conf, _) = ArabicReadingOrderEngine::sequence_arabic_page(&spans, 600.0, 800.0);
    assert_eq!(ordered.len(), 4);
    assert_eq!(ordered[0].text, "عقد تقديم خدمات هندسية واستشارية");
    assert_eq!(ordered[1].text, "البند الأول: موضوع العقد ونطاق العمل");
    assert!(conf >= 0.95);
}
