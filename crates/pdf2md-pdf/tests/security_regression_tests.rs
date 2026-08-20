use flate2::write::ZlibEncoder;
use flate2::Compression;
use pdf2md_ast::geometry::BoundingBox;
use pdf2md_images::ImageExtractor;
use pdf2md_pdf::document::PdfError;
use pdf2md_pdf::security::{CycleDetector, SecurityError, SecurityLimits};
use pdf2md_pdf::stream::StreamDecoder;
use pdf2md_pdf::PdfDocument;
use std::io::Write;

#[test]
fn test_security_decompression_bomb_rejected() {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    let zeros = vec![0u8; 5 * 1024 * 1024]; // 5MB zeros compressed to small bytes
    encoder.write_all(&zeros).unwrap();
    let compressed = encoder.finish().unwrap();

    let limits = SecurityLimits {
        max_decompressed_stream_bytes: 2 * 1024 * 1024, // 2MB limit
        ..Default::default()
    };

    let result = StreamDecoder::decode_flate(&compressed, &limits);
    assert!(result.is_err(), "Decompression bomb must fail");
    match result {
        Err(SecurityError::DecompressionBomb(ratio, max)) => {
            assert!(ratio > max);
        }
        Err(SecurityError::StreamSizeExceeded(sz, max)) => {
            assert!(sz > max);
        }
        other => panic!(
            "Expected DecompressionBomb or StreamSizeExceeded, got {:?}",
            other
        ),
    }
}

#[test]
fn test_security_cyclic_reference_loop_detected() {
    let mut detector = CycleDetector::new(10);

    // Object 1 -> Object 2 -> Object 3
    assert!(detector.enter_object(1).is_ok());
    assert!(detector.enter_object(2).is_ok());
    assert!(detector.enter_object(3).is_ok());

    // Object 3 -> Object 1 (Cycle)
    let cycle_res = detector.enter_object(1);
    assert!(
        cycle_res.is_err(),
        "Cyclic loop must be detected and rejected"
    );
    match cycle_res {
        Err(SecurityError::CyclicReference(id)) => assert_eq!(id, 1),
        other => panic!("Expected CyclicReference error, got {:?}", other),
    }
}

#[test]
fn test_security_max_nesting_depth_enforced() {
    let mut detector = CycleDetector::new(5);

    for i in 1..=5 {
        assert!(detector.enter_object(i).is_ok());
    }

    // 6th object exceeds max depth of 5
    let depth_res = detector.enter_object(6);
    assert!(
        depth_res.is_err(),
        "Exceeding nesting depth must be rejected"
    );
    match depth_res {
        Err(SecurityError::NestingDepthExceeded(d)) => assert_eq!(d, 5),
        other => panic!("Expected NestingDepthExceeded error, got {:?}", other),
    }
}

#[test]
fn test_security_path_traversal_image_sanitization() {
    let hostile_names = [
        "../../../../etc/passwd",
        "..\\..\\windows\\system32\\cmd.exe",
        "/absolute/root/escape.png",
        "nested/sub/dir/image.jpg",
        "null\0byte\0injection.png",
    ];

    for name in &hostile_names {
        let safe = ImageExtractor::sanitize_filename(name, "png");
        assert!(
            !safe.contains(".."),
            "Path traversal token '..' must be eliminated"
        );
        assert!(!safe.contains('/'), "Path separator '/' must be eliminated");
        assert!(
            !safe.contains('\\'),
            "Path separator '\\' must be eliminated"
        );
        assert!(!safe.contains('\0'), "Null bytes must be eliminated");
        assert!(safe.ends_with(".png"));
    }
}

#[test]
fn test_security_invalid_pdf_header_fails_safely() {
    let hostile_header = b"NOT A VALID PDF HEADER %%%$$$";
    let res = PdfDocument::parse(hostile_header, SecurityLimits::default());

    assert!(res.is_err(), "Corrupted PDF header must fail safely");
    match res {
        Err(PdfError::InvalidHeader) => {}
        other => panic!("Expected InvalidHeader error, got {:?}", other),
    }
}

#[test]
fn test_security_image_dimension_bomb_rejected() {
    let extractor = ImageExtractor::new(pdf2md_images::ImageExtractorConfig {
        enabled: true,
        ..Default::default()
    });

    let giant_image = pdf2md_pdf::elements::ImageObject {
        id: "giant_bomb".into(),
        bbox: BoundingBox::new(0.0, 0.0, 100.0, 100.0),
        width: 100_000, // 100,000 x 100,000 px dimension bomb
        height: 100_000,
        mime_type: "image/png".into(),
        data: vec![0x89, 0x50, 0x4E, 0x47],
    };

    let node = extractor.process_image(&giant_image);
    assert!(
        node.is_none(),
        "Giant image dimension bombs must be rejected without allocation"
    );
}
