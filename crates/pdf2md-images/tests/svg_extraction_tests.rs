use pdf2md_ast::geometry::{BoundingBox, Color, Point, Rect};
use pdf2md_images::svg_serializer::SvgSerializer;
use pdf2md_pdf::elements::{PathCommand, PathSegment, TextSpan, VectorGraphic};

#[test]
fn test_svg_serializer_bezier_curves_and_arabic_text() {
    let path1 = PathSegment {
        rect: Some(Rect::new(10.0, 10.0, 100.0, 50.0)),
        points: vec![Point::new(10.0, 10.0), Point::new(110.0, 60.0)],
        commands: vec![PathCommand::Rectangle(Rect::new(10.0, 10.0, 100.0, 50.0))],
        is_stroke: true,
        is_fill: true,
        stroke_width: 2.0,
        color: Color::rgb(0.0, 0.0, 1.0),
        fill_color: Some(Color::rgb(0.9, 0.9, 1.0)),
    };

    let path2 = PathSegment {
        rect: None,
        points: vec![Point::new(10.0, 60.0), Point::new(80.0, 120.0)],
        commands: vec![
            PathCommand::MoveTo(Point::new(10.0, 60.0)),
            PathCommand::CurveTo {
                p1: Point::new(30.0, 40.0),
                p2: Point::new(60.0, 140.0),
                p3: Point::new(80.0, 120.0),
            },
            PathCommand::LineTo(Point::new(110.0, 60.0)),
            PathCommand::ClosePath,
        ],
        is_stroke: true,
        is_fill: false,
        stroke_width: 1.5,
        color: Color::rgb(1.0, 0.0, 0.0),
        fill_color: None,
    };

    let text1 = TextSpan::new(
        "مخطط بياني & تحليل البيانات <2026>".to_string(),
        BoundingBox::new(20.0, 20.0, 80.0, 14.0),
        "Helvetica".to_string(),
        12.0,
        true,
        false,
        false,
    );

    let graphic = VectorGraphic {
        id: "schematic_1".to_string(),
        bbox: BoundingBox::new(10.0, 10.0, 100.0, 110.0),
        paths: vec![path1, path2],
        texts: vec![text1],
    };

    let svg_xml = SvgSerializer::serialize_vector_graphic(&graphic);

    assert!(svg_xml.starts_with("<svg xmlns="));
    assert!(svg_xml.contains(r#"viewBox="0 0 100.00 110.00""#));
    assert!(svg_xml.contains("<rect "));
    assert!(svg_xml.contains("<path d="));
    assert!(svg_xml.contains("C 20.00 30.00, 50.00 130.00, 70.00 110.00"));
    assert!(svg_xml.contains(r#"direction="rtl""#));
    assert!(svg_xml.contains(r#"font-weight="bold""#));
    assert!(svg_xml.contains("&amp;"));
    assert!(svg_xml.contains("&lt;2026&gt;"));
    assert!(svg_xml.ends_with("</svg>\n"));
}

#[test]
fn test_vector_object_clustering() {
    let paths = vec![
        PathSegment {
            rect: Some(Rect::new(50.0, 50.0, 200.0, 150.0)),
            points: vec![Point::new(50.0, 50.0), Point::new(250.0, 200.0)],
            commands: vec![],
            is_stroke: true,
            is_fill: false,
            stroke_width: 1.0,
            color: Color::BLACK,
            fill_color: None,
        },
        PathSegment {
            rect: None,
            points: vec![Point::new(60.0, 100.0), Point::new(180.0, 160.0)],
            commands: vec![
                PathCommand::MoveTo(Point::new(60.0, 100.0)),
                PathCommand::LineTo(Point::new(180.0, 160.0)),
            ],
            is_stroke: true,
            is_fill: false,
            stroke_width: 1.5,
            color: Color::BLACK,
            fill_color: None,
        },
    ];

    let texts = vec![TextSpan::new(
        "Quarterly Trend".to_string(),
        BoundingBox::new(80.0, 60.0, 100.0, 12.0),
        "Helvetica".to_string(),
        11.0,
        false,
        false,
        false,
    )];

    let clusters = SvgSerializer::cluster_vector_objects(&paths, &texts);
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].paths.len(), 2);
    assert_eq!(clusters[0].texts.len(), 1);
    assert_eq!(clusters[0].texts[0].text, "Quarterly Trend");
}
