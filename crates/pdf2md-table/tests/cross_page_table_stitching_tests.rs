use pdf2md_ast::{Document, DocumentMetadata, InlineNode, Node, Section, TableCell, TableRow};
use pdf2md_table::arabic_table::ArabicTableExtractor;
use pdf2md_table::stitching::CrossPageTableStitcher;

fn make_cell(text: &str) -> TableCell {
    TableCell::new(vec![InlineNode::Text(text.to_string())])
}

#[test]
fn test_cross_page_arabic_table_stitching_with_repeated_header() {
    let p1_headers = vec![TableRow {
        cells: vec![
            make_cell("الرقم"),
            make_cell("الاسم الكامل"),
            make_cell("القسم"),
        ],
        is_header: true,
    }];
    let p1_rows = vec![
        TableRow {
            cells: vec![
                make_cell("١"),
                make_cell("أحمد بن علي"),
                make_cell("تطوير البرمجيات"),
            ],
            is_header: false,
        },
        TableRow {
            cells: vec![
                make_cell("٢"),
                make_cell("سارة المنصوري"),
                make_cell("الذكاء الاصطناعي"),
            ],
            is_header: false,
        },
    ];

    let p2_headers = vec![TableRow {
        cells: vec![
            make_cell("الرقم"),
            make_cell("الاسم الكامل"),
            make_cell("القسم"),
        ],
        is_header: true,
    }];
    let p2_rows = vec![
        TableRow {
            cells: vec![
                make_cell("٣"),
                make_cell("عمر الفاروق"),
                make_cell("الأمن السيبراني"),
            ],
            is_header: false,
        },
        TableRow {
            cells: vec![
                make_cell("٤"),
                make_cell("مريم العبيدي"),
                make_cell("هندسة النظم"),
            ],
            is_header: false,
        },
    ];

    let stitched = ArabicTableExtractor::stitch_arabic_tables(
        &p1_headers,
        &p1_rows,
        &p2_headers,
        &p2_rows,
    );

    assert!(stitched.is_some(), "Arabic table stitching failed");
    let (headers, rows, diag) = stitched.unwrap();

    // 1 header row preserved
    assert_eq!(headers.len(), 1);
    // 4 total data rows stitched in order
    assert_eq!(rows.len(), 4);

    // Row order verified
    assert_eq!(rows[0].cells[0].text_content(), "١");
    assert_eq!(rows[1].cells[0].text_content(), "٢");
    assert_eq!(rows[2].cells[0].text_content(), "٣");
    assert_eq!(rows[3].cells[0].text_content(), "٤");

    // RTL direction verified
    assert_eq!(diag.table_direction, pdf2md_ast::geometry::WritingDirection::RightToLeft);

    // Rendered GFM table
    let (gfm, _) = ArabicTableExtractor::format_arabic_table(&headers, &rows, Some("سجل الموظفين"));
    assert!(gfm.contains("| الرقم | الاسم الكامل | القسم |"));
    assert!(gfm.contains("| ١ | أحمد بن علي | تطوير البرمجيات |"));
    assert!(gfm.contains("| ٤ | مريم العبيدي | هندسة النظم |"));
}

#[test]
fn test_cross_page_ltr_table_stitching_with_repeated_header() {
    let t1 = Node::Table {
        headers: vec![TableRow {
            cells: vec![make_cell("ID"), make_cell("Product"), make_cell("Price")],
            is_header: true,
        }],
        rows: vec![
            TableRow {
                cells: vec![make_cell("101"), make_cell("PaperFlux Pro"), make_cell("$499")],
                is_header: false,
            },
            TableRow {
                cells: vec![make_cell("102"), make_cell("Cloud Worker"), make_cell("$199")],
                is_header: false,
            },
        ],
        caption: Some("Pricing Catalog".to_string()),
        confidence: 0.96,
        has_borders: true,
        bbox: None,
    };

    let t2 = Node::Table {
        headers: vec![TableRow {
            cells: vec![make_cell("ID"), make_cell("Product"), make_cell("Price")],
            is_header: true,
        }],
        rows: vec![TableRow {
            cells: vec![make_cell("103"), make_cell("Enterprise Addon"), make_cell("$999")],
            is_header: false,
        }],
        caption: Some("Pricing Catalog (Continued)".to_string()),
        confidence: 0.94,
        has_borders: true,
        bbox: None,
    };

    let stitcher = CrossPageTableStitcher::new();
    assert!(stitcher.can_stitch(&t1, &t2));

    let stitched_node = stitcher.stitch_two_tables(&t1, &t2);
    assert!(stitched_node.is_some());

    if let Some(Node::Table { headers, rows, caption, confidence, .. }) = stitched_node {
        assert_eq!(headers.len(), 1);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[2].cells[1].text_content(), "Enterprise Addon");
        assert_eq!(caption.unwrap(), "Pricing Catalog");
        assert!((confidence - 0.95).abs() < 0.01);
    } else {
        panic!("Expected Table node");
    }
}

#[test]
fn test_cross_page_headerless_continuation() {
    let t1 = Node::Table {
        headers: vec![TableRow {
            cells: vec![make_cell("Col1"), make_cell("Col2"), make_cell("Col3")],
            is_header: true,
        }],
        rows: vec![TableRow {
            cells: vec![make_cell("A"), make_cell("B"), make_cell("C")],
            is_header: false,
        }],
        caption: None,
        confidence: 0.95,
        has_borders: true,
        bbox: None,
    };

    // Page 2 has NO header, only continuation data rows with matching 3 columns
    let t2 = Node::Table {
        headers: Vec::new(),
        rows: vec![
            TableRow {
                cells: vec![make_cell("D"), make_cell("E"), make_cell("F")],
                is_header: false,
            },
            TableRow {
                cells: vec![make_cell("G"), make_cell("H"), make_cell("I")],
                is_header: false,
            },
        ],
        caption: None,
        confidence: 0.95,
        has_borders: true,
        bbox: None,
    };

    let stitcher = CrossPageTableStitcher::new();
    assert!(stitcher.can_stitch(&t1, &t2));

    let stitched_node = stitcher.stitch_two_tables(&t1, &t2).unwrap();
    if let Node::Table { headers, rows, .. } = stitched_node {
        assert_eq!(headers.len(), 1);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].cells[0].text_content(), "A");
        assert_eq!(rows[1].cells[0].text_content(), "D");
        assert_eq!(rows[2].cells[0].text_content(), "G");
    } else {
        panic!("Expected Table node");
    }
}

#[test]
fn test_cross_page_rejection_on_incompatible_columns() {
    let t1 = Node::Table {
        headers: vec![TableRow {
            cells: vec![make_cell("Col1"), make_cell("Col2")],
            is_header: true,
        }],
        rows: vec![],
        caption: None,
        confidence: 0.95,
        has_borders: true,
        bbox: None,
    };

    let t2 = Node::Table {
        headers: vec![TableRow {
            cells: vec![make_cell("Col1"), make_cell("Col2"), make_cell("Col3"), make_cell("Col4")],
            is_header: true,
        }],
        rows: vec![],
        caption: None,
        confidence: 0.95,
        has_borders: true,
        bbox: None,
    };

    let stitcher = CrossPageTableStitcher::new();
    assert!(!stitcher.can_stitch(&t1, &t2));
    assert!(stitcher.stitch_two_tables(&t1, &t2).is_none());
}

#[test]
fn test_cross_page_document_section_stitching() {
    let mut doc = Document::new(DocumentMetadata::default());

    let mut sec1 = Section::new(1);
    sec1.elements.push(Node::Paragraph {
        inlines: vec![InlineNode::Text("Introduction paragraph on Page 1".to_string())],
        confidence: 0.98,
        bbox: None,
    });
    sec1.elements.push(Node::Table {
        headers: vec![TableRow {
            cells: vec![make_cell("Year"), make_cell("Revenue")],
            is_header: true,
        }],
        rows: vec![
            TableRow {
                cells: vec![make_cell("2024"), make_cell("$1.2M")],
                is_header: false,
            },
            TableRow {
                cells: vec![make_cell("2025"), make_cell("$2.5M")],
                is_header: false,
            },
        ],
        caption: Some("Annual Financial Growth".to_string()),
        confidence: 0.98,
        has_borders: true,
        bbox: None,
    });

    let mut sec2 = Section::new(2);
    sec2.elements.push(Node::Table {
        headers: vec![TableRow {
            cells: vec![make_cell("Year"), make_cell("Revenue")],
            is_header: true,
        }],
        rows: vec![
            TableRow {
                cells: vec![make_cell("2026"), make_cell("$4.8M")],
                is_header: false,
            },
        ],
        caption: Some("Annual Financial Growth (Cont.)".to_string()),
        confidence: 0.98,
        has_borders: true,
        bbox: None,
    });
    sec2.elements.push(Node::Paragraph {
        inlines: vec![InlineNode::Text("Conclusion paragraph on Page 2".to_string())],
        confidence: 0.98,
        bbox: None,
    });

    doc.sections.push(sec1);
    doc.sections.push(sec2);

    let stitcher = CrossPageTableStitcher::new();
    stitcher.stitch_document(&mut doc);

    // Section 1 should have paragraph + stitched table (3 rows)
    assert_eq!(doc.sections[0].elements.len(), 2);
    if let Node::Table { rows, headers, .. } = &doc.sections[0].elements[1] {
        assert_eq!(headers.len(), 1);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[2].cells[0].text_content(), "2026");
    } else {
        panic!("Expected Table node in Section 1");
    }

    // Section 2 should have its table removed and only contain conclusion paragraph
    assert_eq!(doc.sections[1].elements.len(), 1);
    assert!(matches!(doc.sections[1].elements[0], Node::Paragraph { .. }));
}
