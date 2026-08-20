use pdf2md_pdf::font::FontMap;
use pdf2md_pdf::security::SecurityLimits;
use pdf2md_pdf::stream::StreamDecoder;
use pdf2md_pdf::PdfDocument;

#[test]
fn test_fuzz_pdf_parser_random_payloads() {
    let limits = SecurityLimits::default();

    let mut state: u64 = 0xDEADBEEFCAFE;
    let mut rng_bytes = |len: usize| -> Vec<u8> {
        let mut buf = Vec::with_capacity(len);
        for _ in 0..len {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            buf.push((state >> 33) as u8);
        }
        buf
    };

    for i in 0..100 {
        let payload = rng_bytes(128 + i * 32);
        let _ = PdfDocument::parse(&payload, limits.clone());
    }
}

#[test]
fn test_fuzz_stream_decompression() {
    let limits = SecurityLimits::default();
    let corrupted_data = [
        vec![0x78, 0x9C, 0xFF, 0xFF, 0x00, 0x01],
        vec![0x00, 0x00, 0x00, 0x00],
        vec![0xFF; 512],
        b"48656C6C6GINVALIDHEX>".to_vec(),
    ];

    for data in &corrupted_data {
        let _ = StreamDecoder::decode_flate(data, &limits);
        let _ = StreamDecoder::decode_ascii_hex(data);
    }
}

#[test]
fn test_fuzz_cmap_parser() {
    let corrupted_cmaps = [
        "/CIDInit /ProcSet findresource begin 12 dict begin begincmap endcmap",
        "beginbfrange <0001> <FFFF> <0020> endbfrange",
        "beginbfchar <01> <0041> endbfchar",
        "",
        "INVALID TOKENS %%%$$$ 999999999999999999999999",
    ];

    for cmap in &corrupted_cmaps {
        let mut font_map = FontMap::new("F1".into(), "Helvetica".into());
        font_map.parse_to_unicode_cmap(cmap);
    }
}
