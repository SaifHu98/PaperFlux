use std::fs;
use std::path::Path;

struct PageDef {
    content: String,
}

struct DocDef {
    filename: &'static str,
    pages: Vec<PageDef>,
    gold_md: &'static str,
}

fn build_pdf(pages: &[PageDef]) -> Vec<u8> {
    let num_pages = pages.len();
    let mut page_objs = Vec::new();
    let mut kids_refs = Vec::new();
    let mut contents_objs = Vec::new();

    for (i, p) in pages.iter().enumerate() {
        let page_obj_id = 3 + i * 2;
        let contents_obj_id = 4 + i * 2;
        kids_refs.push(format!("{} 0 R", page_obj_id));

        let stream_len = p.content.len();
        let contents_obj = format!(
            "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            contents_obj_id, stream_len, p.content
        );
        contents_objs.push(contents_obj);

        let page_obj = format!(
            "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents {} 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> /F2 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >> >> >> >>\nendobj\n",
            page_obj_id, contents_obj_id
        );
        page_objs.push(page_obj);
    }

    let mut pdf = String::new();
    pdf.push_str("%PDF-1.4\n");
    pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    pdf.push_str(&format!(
        "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
        kids_refs.join(" "),
        num_pages
    ));

    for i in 0..num_pages {
        pdf.push_str(&page_objs[i]);
        pdf.push_str(&contents_objs[i]);
    }

    pdf.push_str("xref\n0 1\n0000000000 65535 f \n");
    pdf.push_str("trailer\n<< /Size ");
    pdf.push_str(&(3 + num_pages * 2).to_string());
    pdf.push_str(" /Root 1 0 R >>\nstartxref\n500\n%%EOF\n");

    pdf.into_bytes()
}

fn main() {
    let out_dir = Path::new("tests/fixtures");
    fs::create_dir_all(out_dir).unwrap();

    let fixtures = vec![
        DocDef {
            filename: "academic_bilingual_paper",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(Machine Learning Approaches for Arabic NLP) Tj\n/F1 11 Tf\n0 -25 Td\n(Authors: Dr. Tariq Al-Mansoor, Prof. Elena Rostova) Tj\n0 -20 Td\n/F2 13 Tf\n(Abstract) Tj\n/F1 10 Tf\n0 -18 Td\n(This paper investigates transformer architectures for morphological disambiguation.) Tj\n0 -15 Td\n(\\xd9\\x8a\\xd8\\xaa\\xd9\\x86\\xd8\\xa7\\xd9\\x88\\xd9\\x84 \\xd9\\x87\\xd8\\xb0\\xd8\\xa7 \\xd8\\xa7\\xd9\\x84\\xd8\\xa8\\xd8\\xad\\xd8\\xab \\xd9\\x86\\xd9\\x85\\xd8\\xa7\\xd8\\xb0\\xd8\\xac \\xd8\\xa7\\xd9\\x84\\xd8\\xaa\\xd8\\xb9\\xd9\\x84\\xd9\\x85 \\xd8\\xa7\\xd9\\x84\\xd8\\xb9\\xd9\\x85\\xd9\\x8a\\xd9\\x82 \\xd9\\x84\\xd9\\x85\\xd8\\xb9\\xd8\\xa7\\xd9\\x84\\xd8\\xac\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd9\\x84\\xd8\\xba\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xb9\\xd8\\xb1\\xd8\\xa8\\xd9\\x8a\\xd8\\xa9) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 14 Tf\n72 710 Td\n(1. Methodology and Model Architecture) Tj\n/F1 10 Tf\n0 -22 Td\n(We utilized bidirectional cross-attention with 24 layers and 16 attention heads.) Tj\n0 -18 Td\n(Training was conducted on a multi-genre Arabic dataset comprising 50M tokens.) Tj\n0 -18 Td\n(Table 1: Hyperparameter Configuration) Tj\n0 -18 Td\n(Parameter | Layer Count | Learning Rate | Batch Size) Tj\n0 -15 Td\n(Value | 24 | 1e-4 | 128) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 14 Tf\n72 710 Td\n(2. Results and Conclusion) Tj\n/F1 10 Tf\n0 -22 Td\n(The proposed model achieved 94.8% F1-score across benchmark datasets.) Tj\n0 -18 Td\n(\\xd8\\xa3\\xd8\\xb8\\xd9\\x87\\xd8\\xb1\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd9\\x86\\xd8\\xaa\\xd8\\xa7\\xd8\\xa6\\xd8\\xac \\xd8\\xaa\\xd9\\x81\\xd9\\x88\\xd9\\x82\\xd8\\xa7 \\xd9\\x88\\xd8\\xa7\\xd8\\xb6\\xd8\\xad\\xd8\\xa7 \\xd9\\x81\\xd9\\x8a \\xd8\\xaf\\xd9\\x82\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xaa\\xd8\\xb5\\xd9\\x86\\xd9\\x8a\\xd9\\x81) Tj\n0 -25 Td\n/F2 12 Tf\n(References) Tj\n/F1 9 Tf\n0 -18 Td\n([1] Vaswani et al., Attention Is All You Need, NeurIPS 2017.) Tj\n0 -15 Td\n([2] Al-Badr et al., Arabic Semantic Benchmarks, ACL 2024.) Tj\nET\n".into(),
                },
            ],
            gold_md: "# Machine Learning Approaches for Arabic NLP\n\nAuthors: Dr. Tariq Al-Mansoor, Prof. Elena Rostova\n\n## Abstract\n\nThis paper investigates transformer architectures for morphological disambiguation.\n\nيتناول هذا البحث نماذج التعلم العميق لمعالجة اللغة العربية\n\n<!-- pagebreak: page 2 -->\n\n## 1. Methodology and Model Architecture\n\nWe utilized bidirectional cross-attention with 24 layers and 16 attention heads.\n\nTraining was conducted on a multi-genre Arabic dataset comprising 50M tokens.\n\nTable 1: Hyperparameter Configuration\n\n| Parameter | Layer Count | Learning Rate | Batch Size |\n|---|---|---|---|\n| Value | 24 | 1e-4 | 128 |\n\n<!-- pagebreak: page 3 -->\n\n## 2. Results and Conclusion\n\nThe proposed model achieved 94.8% F1-score across benchmark datasets.\n\nأظهرت النتائج تفوقا واضحا في دقة التصنيف\n\n### References\n\n[1] Vaswani et al., Attention Is All You Need, NeurIPS 2017.\n\n[2] Al-Badr et al., Arabic Semantic Benchmarks, ACL 2024.\n",
        },
        DocDef {
            filename: "financial_annual_report_multipage_tables",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(Alpha Holding Group - Annual Financial Report 2026) Tj\n/F1 11 Tf\n0 -25 Td\n(Consolidated Balance Sheet and Operating Revenue) Tj\n0 -20 Td\n(Currency: USD in Thousands) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 12 Tf\n72 710 Td\n(Consolidated Statement of Operations - Part 1) Tj\n/F1 10 Tf\n0 -20 Td\n(Category | Q1 | Q2 | Q3 | Q4 | Total) Tj\n0 -16 Td\n(Cloud Software Revenue | 12,400 | 14,200 | 15,800 | 18,100 | 60,500) Tj\n0 -16 Td\n(Hardware Systems | 8,200 | 8,500 | 9,100 | 9,800 | 35,600) Tj\n0 -16 Td\n(Consulting & Professional Services | 4,100 | 4,300 | 4,600 | 5,000 | 18,000) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 12 Tf\n72 710 Td\n(Consolidated Statement of Operations - Part 2) Tj\n/F1 10 Tf\n0 -20 Td\n(Category | Q1 | Q2 | Q3 | Q4 | Total) Tj\n0 -16 Td\n(Research & Development Expenses | 3,200 | 3,400 | 3,700 | 4,100 | 14,400) Tj\n0 -16 Td\n(Sales & Marketing | 2,800 | 3,000 | 3,200 | 3,500 | 12,500) Tj\n0 -16 Td\n(Net Operating Income | 18,700 | 20,600 | 22,600 | 25,300 | 87,200) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 14 Tf\n72 710 Td\n(Audit Committee Statement & Signatures) Tj\n/F1 10 Tf\n0 -25 Td\n(The financial statements have been audited in accordance with International Financial Reporting Standards.) Tj\n0 -20 Td\n(Chief Financial Officer: Marcus Vance, CPA) Tj\n0 -15 Td\n(External Auditor: Global Assurance LLP) Tj\nET\n".into(),
                },
            ],
            gold_md: "# Alpha Holding Group - Annual Financial Report 2026\n\nConsolidated Balance Sheet and Operating Revenue\n\nCurrency: USD in Thousands\n\n<!-- pagebreak: page 2 -->\n\n## Consolidated Statement of Operations - Part 1\n\n| Category | Q1 | Q2 | Q3 | Q4 | Total |\n|---|---|---|---|---|---|\n| Cloud Software Revenue | 12,400 | 14,200 | 15,800 | 18,100 | 60,500 |\n| Hardware Systems | 8,200 | 8,500 | 9,100 | 9,800 | 35,600 |\n| Consulting & Professional Services | 4,100 | 4,300 | 4,600 | 5,000 | 18,000 |\n\n<!-- pagebreak: page 3 -->\n\n## Consolidated Statement of Operations - Part 2\n\n| Category | Q1 | Q2 | Q3 | Q4 | Total |\n|---|---|---|---|---|---|\n| Research & Development Expenses | 3,200 | 3,400 | 3,700 | 4,100 | 14,400 |\n| Sales & Marketing | 2,800 | 3,000 | 3,200 | 3,500 | 12,500 |\n| Net Operating Income | 18,700 | 20,600 | 22,600 | 25,300 | 87,200 |\n\n<!-- pagebreak: page 4 -->\n\n## Audit Committee Statement & Signatures\n\nThe financial statements have been audited in accordance with International Financial Reporting Standards.\n\nChief Financial Officer: Marcus Vance, CPA\n\nExternal Auditor: Global Assurance LLP\n",
        },
        DocDef {
            filename: "government_decree_complex_rtl",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(\\xd8\\xa7\\xd9\\x84\\xd8\\xac\\xd8\\xb1\\xd9\\x8a\\xd8\\xaf\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xb1\\xd8\\xb3\\xd9\\x85\\xd9\\x8a\\xd8\\xa9 - \\xd9\\x82\\xd8\\xb1\\xd8\\xa7\\xd8\\xb1 \\xd8\\xb1\\xd9\\x82\\xd9\\x85 452 \\xd9\\x84\\xd8\\xb3\\xd9\\x86\\xd8\\xa9 2026) Tj\n/F1 11 Tf\n0 -25 Td\n(\\xd8\\xa8\\xd8\\xb4\\xd8\\xa3\\xd9\\x86 \\xd8\\xaa\\xd9\\x86\\xd8\\xb8\\xd9\\x8a\\xd9\\x85 \\xd8\\xae\\xd8\\xaf\\xd9\\x85\\xd8\\xa7\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd8\\xad\\xd9\\x88\\xd8\\xb3\\xd8\\xa8\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xb3\\xd8\\xad\\xd8\\xa7\\xd8\\xa8\\xd9\\x8a\\xd8\\xa9 \\xd9\\x88\\xd8\\xa7\\xd9\\x84\\xd8\\xa8\\xd9\\x8a\\xd8\\xa7\\xd9\\x86\\xd8\\xa7\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd8\\xad\\xd9\\x83\\xd9\\x88\\xd9\\x85\\xd9\\x8a\\xd8\\xa9) Tj\n0 -20 Td\n/F2 12 Tf\n(\\xd8\\xa7\\xd9\\x84\\xd9\\x85\\xd8\\xa7\\xd8\\xaf\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xa3\\xd9\\x88\\xd9\\x84\\xd9\\x89: \\xd8\\xa7\\xd9\\x84\\xd8\\xaa\\xd8\\xb9\\xd8\\xb1\\xd9\\x8a\\xd9\\x81\\xd8\\xa7\\xd8\\xaa) Tj\n/F1 10 Tf\n0 -18 Td\n(\\xd9\\x8a\\xd9\\x82\\xd8\\xb5\\xd8\\xaf \\xd8\\xa8\\xd8\\xa7\\xd9\\x84\\xd8\\xa8\\xd9\\x8a\\xd8\\xa7\\xd9\\x86\\xd8\\xa7\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd8\\xad\\xd9\\x83\\xd9\\x88\\xd9\\x85\\xd9\\x8a\\xd8\\xa9 \\xd8\\xac\\xd9\\x85\\xd9\\x8a\\xd8\\xb9 \\xd8\\xa7\\xd9\\x84\\xd9\\x85\\xd8\\xb9\\xd9\\x84\\xd9\\x88\\xd9\\x85\\xd8\\xa7\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd9\\x85\\xd8\\xad\\xd9\\x81\\xd9\\x88\\xd8\\xb8\\xd8\\xa9 \\xd8\\xa5\\xd9\\x84\\xd9\\x83\\xd8\\xaa\\xd8\\xb1\\xd9\\x88\\xd9\\x86\\xd9\\x8a\\xd8\\xa7.) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 14 Tf\n72 710 Td\n(\\xd8\\xa7\\xd9\\x84\\xd9\\x85\\xd8\\xa7\\xd8\\xaf\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xab\\xd8\\xa7\\xd9\\x86\\xd9\\x8a\\xd8\\xa9: \\xd9\\x85\\xd8\\xb9\\xd8\\xa7\\xd9\\x8a\\xd9\\x8a\\xd8\\xb1 \\xd8\\xa7\\xd9\\x84\\xd8\\xa3\\xd9\\x85\\xd9\\x86 \\xd8\\xa7\\xd9\\x84\\xd8\\xb3\\xd9\\x8a\\xd8\\xa8\\xd8\\xb1\\xd8\\xa7\\xd9\\x86\\xd9\\x8a) Tj\n/F1 10 Tf\n0 -22 Td\n(\\xd8\\xaa\\xd9\\x84\\xd8\\xaa\\xd8\\xb2\\xd9\\x85 \\xd9\\x83\\xd8\\xa7\\xd9\\x81\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xac\\xd9\\x87\\xd8\\xa7\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd8\\xad\\xd9\\x83\\xd9\\x88\\xd9\\x85\\xd9\\x8a\\xd8\\xa9 \\xd8\\xa8\\xd8\\xaa\\xd8\\xb7\\xd8\\xa8\\xd9\\x8a\\xd9\\x82 \\xd8\\xa7\\xd9\\x84\\xd8\\xaa\\xd8\\xb4\\xd9\\x81\\xd9\\x8a\\xd8\\xb1 \\xd8\\xa7\\xd9\\x84\\xd8\\xb4\\xd8\\xa7\\xd9\\x85\\xd9\\x84 \\xd9\\x84\\xd9\\x84\\xd8\\xa8\\xd9\\x8a\\xd8\\xa7\\xd9\\x86\\xd8\\xa7\\xd8\\xaa.) Tj\n0 -18 Td\n(\\xd9\\x8a\\xd8\\xad\\xd8\\xb8\\xd8\\xb1 \\xd9\\x86\\xd9\\x82\\xd9\\x84 \\xd8\\xa7\\xd9\\x84\\xd8\\xa8\\xd9\\x8a\\xd8\\xa7\\xd9\\x86\\xd8\\xa7\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd8\\xb3\\xd8\\xb1\\xd9\\x8a\\xd8\\xa9 \\xd8\\xae\\xd8\\xa7\\xd8\\xb1\\xd8\\xac \\xd8\\xa7\\xd9\\x84\\xd8\\xad\\xd8\\xaf\\xd9\\x88\\xd8\\xaf \\xd8\\xa7\\xd9\\x84\\xd9\\x88\\xd8\\xb7\\xd9\\x86\\xd9\\x8a\\xd8\\xa9.) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 14 Tf\n72 710 Td\n(\\xd8\\xa7\\xd9\\x84\\xd9\\x85\\xd8\\xa7\\xd8\\xaf\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xab\\xd8\\xa7\\xd9\\x84\\xd8\\xab\\xd8\\xa9: \\xd8\\xa7\\xd9\\x84\\xd9\\x86\\xd9\\x81\\xd8\\xa7\\xd8\\xb0 \\xd9\\x88\\xd8\\xa7\\xd9\\x84\\xd9\\x86\\xd8\\xb4\\xd8\\xb1) Tj\n/F1 10 Tf\n0 -22 Td\n(\\xd9\\x8a\\xd8\\xb9\\xd9\\x85\\xd9\\x84 \\xd8\\xa8\\xd9\\x87\\xd8\\xb0\\xd8\\xa7 \\xd8\\xa7\\xd9\\x84\\xd9\\x82\\xd8\\xb1\\xd8\\xa7\\xd8\\xb1 \\xd8\\xa7\\xd8\\xb9\\xd8\\xaa\\xd8\\xa8\\xd8\\xa7\\xd8\\xb1\\xd8\\xa7 \\xd9\\x85\\xd9\\x86 \\xd8\\xaa\\xd8\\xa7\\xd8\\xb1\\xd9\\x8a\\xd8\\xae \\xd9\\x86\\xd8\\xb4\\xd8\\xb1\\xd9\\x87 \\xd9\\x81\\xd9\\x8a \\xd8\\xa7\\xd9\\x84\\xd8\\xac\\xd8\\xb1\\xd9\\x8a\\xd8\\xaf\\xd8\\xa9 \\xd8\\xa7\\xd9\\x84\\xd8\\xb1\\xd8\\xb3\\xd9\\x85\\xd9\\x8a\\xd8\\xa9.) Tj\n0 -30 Td\n/F2 11 Tf\n(\\xd8\\xaa\\xd9\\x88\\xd9\\x82\\xd9\\x8a\\xd8\\xb9: \\xd8\\xb1\\xd8\\xa6\\xd9\\x8a\\xd8\\xb3 \\xd9\\x85\\xd8\\xac\\xd9\\x84\\xd8\\xb3 \\xd8\\xa7\\xd9\\x84\\xd9\\x88\\xd8\\xb2\\xd8\\xb1\\xd8\\xa7\\xd8\\xa1) Tj\nET\n".into(),
                },
            ],
            gold_md: "# الجريدة الرسمية - قرار رقم 452 لسنة 2026\n\nبشأن تنظيم خدمات الحوسبة السحابية والبيانات الحكومية\n\n## المادة الأولى: التعريفات\n\nيقصد بالبيانات الحكومية جميع المعلومات المحفوظة إلكترونيا.\n\n<!-- pagebreak: page 2 -->\n\n## المادة الثانية: معايير الأمن السيبراني\n\nتلتزم كافة الجهات الحكومية بتطبيق التشفير الشامل للبيانات.\n\nيحظر نقل البيانات السرية خارج الحدود الوطنية.\n\n<!-- pagebreak: page 3 -->\n\n## المادة الثالثة: النفاذ والنشر\n\nيعمل بهذا القرار اعتبارا من تاريخ نشره في الجريدة الرسمية.\n\nتوقيع: رئيس مجلس الوزراء\n",
        },
        DocDef {
            filename: "scanned_historical_manuscript",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 16 Tf\n72 710 Td\n(Manuscript Archive - Codex Arabica #108) Tj\n/F1 11 Tf\n0 -25 Td\n(Folio 12a: Treatise on Astronomy and Optics) Tj\n0 -20 Td\n(\\xd9\\x85\\xd8\\xae\\xd8\\xb7\\xd9\\x88\\xd8\\xb7 \\xd9\\x81\\xd9\\x8a \\xd8\\xb9\\xd9\\x84\\xd9\\x85 \\xd8\\xa7\\xd9\\x84\\xd9\\x81\\xd9\\x84\\xd9\\x83 \\xd9\\x88\\xd8\\xa7\\xd9\\x84\\xd8\\xa8\\xd8\\xb5\\xd8\\xb1\\xd9\\x8a\\xd8\\xa7\\xd8\\xaa) Tj\n0 -18 Td\n(Preserved in the National Heritage Manuscript Collection) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 14 Tf\n72 710 Td\n(Folio 12b: Planetary Motions and Geometric Projections) Tj\n/F1 10 Tf\n0 -22 Td\n(\\xd8\\xa7\\xd9\\x84\\xd8\\xa8\\xd8\\xa7\\xd8\\xa8 \\xd8\\xa7\\xd9\\x84\\xd8\\xab\\xd8\\xa7\\xd9\\x84\\xd8\\xab \\xd9\\x81\\xd9\\x8a \\xd8\\xad\\xd8\\xb1\\xd9\\x83\\xd8\\xa7\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd9\\x83\\xd9\\x88\\xd8\\xa7\\xd9\\x83\\xd8\\xa8 \\xd9\\x88\\xd8\\xa7\\xd9\\x84\\xd8\\xa3\\xd9\\x81\\xd9\\x84\\xd8\\xa7\\xd9\\x83) Tj\n0 -18 Td\n(Transcript: In the name of God, chapter on calculating celestial ascendants.) Tj\nET\n".into(),
                },
            ],
            gold_md: "# Manuscript Archive - Codex Arabica #108\n\nFolio 12a: Treatise on Astronomy and Optics\n\nمخطوط في علم الفلك والبصريات\n\nPreserved in the National Heritage Manuscript Collection\n\n<!-- pagebreak: page 2 -->\n\n## Folio 12b: Planetary Motions and Geometric Projections\n\nالباب الثالث في حركات الكواكب والأفلاك\n\nTranscript: In the name of God, chapter on calculating celestial ascendants.\n",
        },
        DocDef {
            filename: "legal_commercial_contract",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(COMMERCIAL LEASE AGREEMENT) Tj\n/F1 11 Tf\n0 -25 Td\n(Between Lessor: Horizon Properties Ltd. and Lessee: Nexus Systems FZ-LLC) Tj\n0 -20 Td\n/F2 12 Tf\n(Clause 1: Premises and Term) Tj\n/F1 10 Tf\n0 -18 Td\n(The leased premises shall comprise Suite 504, Dubai Internet City.) Tj\n0 -15 Td\n(\\xd8\\xb9\\xd9\\x82\\xd8\\xaf \\xd8\\xa5\\xd9\\x8a\\xd8\\xac\\xd8\\xa7\\xd8\\xb1 \\xd8\\xaa\\xd8\\xac\\xd8\\xa7\\xd8\\xb1\\xd9\\x8a \\xd9\\x85\\xd9\\x84\\xd8\\xb2\\xd9\\x85 \\xd9\\x84\\xd9\\x84\\xd8\\xb7\\xd8\\xb1\\xd9\\x81\\xd9\\x8a\\xd9\\x86) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 14 Tf\n72 710 Td\n(Clause 2: Rent and Security Deposit) Tj\n/F1 10 Tf\n0 -22 Td\n(The annual rent of AED 240,000 shall be paid in four equal quarterly installments.) Tj\n0 -18 Td\n(Security deposit of AED 20,000 is refundable upon termination.) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 14 Tf\n72 710 Td\n(Clause 3: Governing Law and Dispute Resolution) Tj\n/F1 10 Tf\n0 -22 Td\n(This agreement is governed by the laws of DIFC Arbitration Center.) Tj\n0 -25 Td\n/F2 11 Tf\n(Signatures: Lessor [______________]  Lessee [______________]) Tj\nET\n".into(),
                },
            ],
            gold_md: "# COMMERCIAL LEASE AGREEMENT\n\nBetween Lessor: Horizon Properties Ltd. and Lessee: Nexus Systems FZ-LLC\n\n## Clause 1: Premises and Term\n\nThe leased premises shall comprise Suite 504, Dubai Internet City.\n\nعقد إيجار تجاري ملزم للطرفين\n\n<!-- pagebreak: page 2 -->\n\n## Clause 2: Rent and Security Deposit\n\nThe annual rent of AED 240,000 shall be paid in four equal quarterly installments.\n\nSecurity deposit of AED 20,000 is refundable upon termination.\n\n<!-- pagebreak: page 3 -->\n\n## Clause 3: Governing Law and Dispute Resolution\n\nThis agreement is governed by the laws of DIFC Arbitration Center.\n\nSignatures: Lessor [______________]  Lessee [______________]\n",
        },
        DocDef {
            filename: "medical_clinical_report",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(Al-Amal Specialized Hospital - Diagnostic Pathology Report) Tj\n/F1 11 Tf\n0 -25 Td\n(Patient: Fatima Al-Zahra | MRN: 902144 | Date: 2026-08-15) Tj\n0 -20 Td\n/F2 13 Tf\n(Clinical Indication: Complete Metabolic Panel) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 13 Tf\n72 710 Td\n(Laboratory Test Results) Tj\n/F1 10 Tf\n0 -20 Td\n(Test Name | Result | Reference Range | Units | Status) Tj\n0 -16 Td\n(Fasting Blood Glucose | 92 | 70-99 | mg/dL | Normal) Tj\n0 -16 Td\n(HbA1c | 5.4 | < 5.7 | % | Normal) Tj\n0 -16 Td\n(Serum Creatinine | 0.8 | 0.6-1.1 | mg/dL | Normal) Tj\n0 -16 Td\n(Total Cholesterol | 178 | < 200 | mg/dL | Desirable) Tj\n0 -25 Td\n/F2 11 Tf\n(Consultant Pathologist: Dr. Samir Qasim, MD, FRCPath) Tj\nET\n".into(),
                },
            ],
            gold_md: "# Al-Amal Specialized Hospital - Diagnostic Pathology Report\n\nPatient: Fatima Al-Zahra | MRN: 902144 | Date: 2026-08-15\n\n## Clinical Indication: Complete Metabolic Panel\n\n<!-- pagebreak: page 2 -->\n\n## Laboratory Test Results\n\n| Test Name | Result | Reference Range | Units | Status |\n|---|---|---|---|---|\n| Fasting Blood Glucose | 92 | 70-99 | mg/dL | Normal |\n| HbA1c | 5.4 | < 5.7 | % | Normal |\n| Serum Creatinine | 0.8 | 0.6-1.1 | mg/dL | Normal |\n| Total Cholesterol | 178 | < 200 | mg/dL | Desirable |\n\nConsultant Pathologist: Dr. Samir Qasim, MD, FRCPath\n",
        },
        DocDef {
            filename: "university_syllabus_grading",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(Faculty of Engineering - Course Syllabus CS402) Tj\n/F1 11 Tf\n0 -25 Td\n(Course Title: Distributed Systems and Cloud Computing) Tj\n0 -20 Td\n(Semester: Fall 2026 | Credits: 3.0) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 13 Tf\n72 710 Td\n(Assessment Matrix & Grade Distribution) Tj\n/F1 10 Tf\n0 -20 Td\n(Assessment Component | Weight | Due Week) Tj\n0 -16 Td\n(Midterm Examination | 25% | Week 7) Tj\n0 -16 Td\n(Programming Assignments (4x) | 30% | Weeks 3, 6, 9, 12) Tj\n0 -16 Td\n(Term Project & Presentation | 15% | Week 14) Tj\n0 -16 Td\n(Final Comprehensive Exam | 30% | Week 16) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 13 Tf\n72 710 Td\n(Academic Integrity & Course Policies) Tj\n/F1 10 Tf\n0 -20 Td\n(All submitted code will be screened via automated plagiarism detection systems.) Tj\n0 -18 Td\n(Office Hours: Mondays & Wednesdays 14:00 - 16:00, Engineering Hall B-210) Tj\nET\n".into(),
                },
            ],
            gold_md: "# Faculty of Engineering - Course Syllabus CS402\n\nCourse Title: Distributed Systems and Cloud Computing\n\nSemester: Fall 2026 | Credits: 3.0\n\n<!-- pagebreak: page 2 -->\n\n## Assessment Matrix & Grade Distribution\n\n| Assessment Component | Weight | Due Week |\n|---|---|---|\n| Midterm Examination | 25% | Week 7 |\n| Programming Assignments (4x) | 30% | Weeks 3, 6, 9, 12 |\n| Term Project & Presentation | 15% | Week 14 |\n| Final Comprehensive Exam | 30% | Week 16 |\n\n<!-- pagebreak: page 3 -->\n\n## Academic Integrity & Course Policies\n\nAll submitted code will be screened via automated plagiarism detection systems.\n\nOffice Hours: Mondays & Wednesdays 14:00 - 16:00, Engineering Hall B-210\n",
        },
        DocDef {
            filename: "technical_specification_manual",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(PaperFlux Architecture Specification v2.0) Tj\n/F1 11 Tf\n0 -25 Td\n(High-Performance Arabic PDF to Markdown Engine) Tj\n0 -20 Td\n(Module Overview: pdf2md-core, pdf2md-layout, pdf2md-table) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 13 Tf\n72 710 Td\n(Engine Performance SLAs) Tj\n/F1 10 Tf\n0 -20 Td\n(Metric | SLA Target | Unit) Tj\n0 -16 Td\n(Single-Page Latency | < 5.0 | ms) Tj\n0 -16 Td\n(Memory per Worker | < 64 | MB) Tj\n0 -16 Td\n(Parallel Throughput | > 200 | pages/sec) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 13 Tf\n72 710 Td\n(CLI Deployment Instructions) Tj\n/F1 10 Tf\n0 -20 Td\n(Command: cargo build --release -p pdf2md-cli) Tj\n0 -16 Td\n(Binary Location: target/release/pdf2md) Tj\nET\n".into(),
                },
            ],
            gold_md: "# PaperFlux Architecture Specification v2.0\n\nHigh-Performance Arabic PDF to Markdown Engine\n\nModule Overview: pdf2md-core, pdf2md-layout, pdf2md-table\n\n<!-- pagebreak: page 2 -->\n\n## Engine Performance SLAs\n\n| Metric | SLA Target | Unit |\n|---|---|---|\n| Single-Page Latency | < 5.0 | ms |\n| Memory per Worker | < 64 | MB |\n| Parallel Throughput | > 200 | pages/sec |\n\n<!-- pagebreak: page 3 -->\n\n## CLI Deployment Instructions\n\nCommand: cargo build --release -p pdf2md-cli\n\nBinary Location: target/release/pdf2md\n",
        },
        DocDef {
            filename: "news_magazine_three_column",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(Al-Thaqafa Monthly: Arabic Cultural Periodical) Tj\n/F1 11 Tf\n0 -25 Td\n(Issue #88 - Modern Arabic Typography and Digital Preservation) Tj\n0 -20 Td\n(\\xd8\\xa2\\xd9\\x81\\xd8\\xa7\\xd9\\x82 \\xd8\\xa7\\xd9\\x84\\xd8\\xae\\xd8\\xb7 \\xd8\\xa7\\xd9\\x84\\xd8\\xb9\\xd8\\xb1\\xd8\\xa8\\xd9\\x8a \\xd9\\x81\\xd9\\x8a \\xd8\\xa7\\xd9\\x84\\xd8\\xb9\\xd8\\xb5\\xd8\\xb1 \\xd8\\xa7\\xd9\\x84\\xd8\\xb1\\xd9\\x82\\xd9\\x85\\xd9\\x8a) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 13 Tf\n72 710 Td\n(Digital Font Heritage and Calligraphic Renaissance) Tj\n/F1 10 Tf\n0 -20 Td\n(Arabic type design has evolved rapidly with OpenType shaping engines.) Tj\n0 -16 Td\n(Preserving calligraphic subtleties remains a primary design objective.) Tj\nET\n".into(),
                },
            ],
            gold_md: "# Al-Thaqafa Monthly: Arabic Cultural Periodical\n\nIssue #88 - Modern Arabic Typography and Digital Preservation\n\nآفاق الخط العربي في العصر الرقمي\n\n<!-- pagebreak: page 2 -->\n\n## Digital Font Heritage and Calligraphic Renaissance\n\nArabic type design has evolved rapidly with OpenType shaping engines.\n\nPreserving calligraphic subtleties remains a primary design objective.\n",
        },
        DocDef {
            filename: "banking_account_statement",
            pages: vec![
                PageDef {
                    content: "BT\n/F2 18 Tf\n72 710 Td\n(National Commercial Bank - Account Statement) Tj\n/F1 11 Tf\n0 -25 Td\n(Account: 108-449201-01 | Currency: SAR | Period: 01/08/2026 - 31/08/2026) Tj\n0 -20 Td\n(\\xd9\\x83\\xd8\\xb4\\xd9\\x81 \\xd8\\xad\\xd8\\xb3\\xd8\\xa7\\xd8\\xa8 \\xd8\\xa7\\xd9\\x84\\xd9\\x85\\xd8\\xb9\\xd8\\xa7\\xd9\\x85\\xd9\\x84\\xd8\\xa7\\xd8\\xaa \\xd8\\xa7\\xd9\\x84\\xd9\\x85\\xd8\\xb5\\xd8\\xb1\\xd9\\x81\\xd9\\x8a\\xd8\\xa9) Tj\nET\n".into(),
                },
                PageDef {
                    content: "BT\n/F2 13 Tf\n72 710 Td\n(Transaction Activity Ledger) Tj\n/F1 10 Tf\n0 -20 Td\n(Date | Description | Debit (SAR) | Credit (SAR) | Balance (SAR)) Tj\n0 -16 Td\n(2026-08-01 | Opening Balance | - | - | 45,200.00) Tj\n0 -16 Td\n(2026-08-05 | Client Wire Transfer | - | 18,500.00 | 63,700.00) Tj\n0 -16 Td\n(2026-08-12 | Cloud Server Hosting | 2,400.00 | - | 61,300.00) Tj\n0 -16 Td\n(2026-08-28 | Monthly Office Lease | 8,000.00 | - | 53,300.00) Tj\nET\n".into(),
                },
            ],
            gold_md: "# National Commercial Bank - Account Statement\n\nAccount: 108-449201-01 | Currency: SAR | Period: 01/08/2026 - 31/08/2026\n\nكشف حساب المعاملات المصرفية\n\n<!-- pagebreak: page 2 -->\n\n## Transaction Activity Ledger\n\n| Date | Description | Debit (SAR) | Credit (SAR) | Balance (SAR) |\n|---|---|---|---|---|\n| 2026-08-01 | Opening Balance | - | - | 45,200.00 |\n| 2026-08-05 | Client Wire Transfer | - | 18,500.00 | 63,700.00 |\n| 2026-08-12 | Cloud Server Hosting | 2,400.00 | - | 61,300.00 |\n| 2026-08-28 | Monthly Office Lease | 8,000.00 | - | 53,300.00 |\n",
        },
    ];

    for f in fixtures {
        let pdf_data = build_pdf(&f.pages);
        let pdf_path = out_dir.join(format!("{}.pdf", f.filename));
        let gold_path = out_dir.join(format!("{}.md.gold", f.filename));

        fs::write(&pdf_path, &pdf_data).unwrap();
        fs::write(&gold_path, f.gold_md).unwrap();
        println!("Generated fixture: {:?} and {:?}", pdf_path, gold_path);
    }
}
