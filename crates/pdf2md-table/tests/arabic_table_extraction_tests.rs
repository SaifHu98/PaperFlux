use pdf2md_ast::geometry::WritingDirection;
use pdf2md_ast::{InlineNode, TableCell, TableRow};
use pdf2md_table::arabic_table::ArabicTableExtractor;

fn make_cell(text: &str) -> TableCell {
    TableCell::new(vec![InlineNode::Text(text.to_string())])
}

fn make_merged_cell(text: &str, colspan: usize, rowspan: usize) -> TableCell {
    let mut cell = TableCell::new(vec![InlineNode::Text(text.to_string())]);
    cell.colspan = colspan;
    cell.rowspan = rowspan;
    cell
}

#[test]
fn test_rtl_table_direction_and_gfm_formatting() {
    let headers = vec![TableRow {
        cells: vec![
            make_cell("الرقم"),
            make_cell("الاسم الكامل"),
            make_cell("المعدل"),
        ],
        is_header: true,
    }];

    let rows = vec![
        TableRow {
            cells: vec![
                make_cell("١"),
                make_cell("أحمد بن علي"),
                make_cell("٩٨٫٥٪"),
            ],
            is_header: false,
        },
        TableRow {
            cells: vec![
                make_cell("٢"),
                make_cell("سارة المنصوري"),
                make_cell("٩٥٫٠٪"),
            ],
            is_header: false,
        },
    ];

    let (gfm, diag) = ArabicTableExtractor::format_arabic_table(&headers, &rows, Some("بيانات الطلاب"));

    assert_eq!(diag.table_direction, WritingDirection::RightToLeft);
    assert!(diag.column_confidence >= 0.95);
    assert!(gfm.contains("| الرقم | الاسم الكامل | المعدل |"));
    assert!(gfm.contains("| ---: | ---: | ---: |"));
    assert!(gfm.contains("| ١ | أحمد بن علي | ٩٨٫٥٪ |"));
}

#[test]
fn test_mixed_arabic_english_cells() {
    let headers = vec![TableRow {
        cells: vec![
            make_cell("المعرف"),
            make_cell("المنتج"),
            make_cell("السعر"),
        ],
        is_header: true,
    }];

    let rows = vec![TableRow {
        cells: vec![
            make_cell("PF-01"),
            make_cell("PaperFlux Engine 2.0"),
            make_cell("١٥٠ ر.س (150 SAR)"),
        ],
        is_header: false,
    }];

    let (gfm, _) = ArabicTableExtractor::format_arabic_table(&headers, &rows, None);
    assert!(gfm.contains("PF-01"));
    assert!(gfm.contains("PaperFlux Engine 2.0"));
    assert!(gfm.contains("١٥٠ ر.س (150 SAR)"));
}

#[test]
fn test_complex_merged_cells_html_fallback() {
    let headers = vec![
        TableRow {
            cells: vec![
                make_merged_cell("التقرير السنوي", 3, 1),
            ],
            is_header: true,
        },
        TableRow {
            cells: vec![
                make_cell("البند"),
                make_cell("الميزانية"),
                make_cell("المصروفات"),
            ],
            is_header: true,
        },
    ];

    let rows = vec![TableRow {
        cells: vec![
            make_cell("التطوير"),
            make_cell("٥٠٠٬٠٠٠"),
            make_cell("٤٥٠٬٠٠٠"),
        ],
        is_header: false,
    }];

    let (html, diag) = ArabicTableExtractor::format_arabic_table(&headers, &rows, Some("الميزانية التقديرية"));

    assert_eq!(diag.table_direction, WritingDirection::RightToLeft);
    assert!(html.contains("<table dir=\"rtl\">"));
    assert!(html.contains("<caption>الميزانية التقديرية</caption>"));
    assert!(html.contains("colspan=\"3\""));
    assert!(html.contains("<th colspan=\"3\">التقرير السنوي</th>"));
}

#[test]
fn test_multiline_arabic_cells() {
    let headers = vec![TableRow {
        cells: vec![
            make_cell("الرمز"),
            make_cell("الوصف التفصيلي للمشروع"),
        ],
        is_header: true,
    }];

    let rows = vec![TableRow {
        cells: vec![
            make_cell("A1"),
            make_cell("مشروع بناء نظام ذكي متكامل\nلمعالجة الوثائق والمستندات"),
        ],
        is_header: false,
    }];

    let (gfm, _) = ArabicTableExtractor::format_arabic_table(&headers, &rows, None);
    // Escaped without breaking into broken markdown rows
    assert!(gfm.contains("مشروع بناء نظام ذكي متكامل لمعالجة الوثائق والمستندات"));
}
