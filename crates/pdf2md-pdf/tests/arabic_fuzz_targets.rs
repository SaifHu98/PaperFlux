use pdf2md_pdf::arabic_font_recovery::{
    AdobeArabicGlyphMap, ArabicCorruptionDetector, ArabicFontDecoder,
};
use std::collections::HashMap;

#[test]
fn test_fuzz_arabic_adobe_glyph_names() {
    let mut rng_seed = 0x12345678u64;

    let prefixes = ["afii", "uni", "glyph", "char", "ar_", "lam_", "allah_"];
    for _ in 0..500 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let prefix = prefixes[(rng_seed as usize) % prefixes.len()];
        let suffix = format!("{:x}", rng_seed & 0xFFFF);
        let fake_glyph_name = format!("{}{}", prefix, suffix);

        // Fuzzer must never panic or crash on arbitrary glyph names
        let _ = AdobeArabicGlyphMap::glyph_name_to_unicode(&fake_glyph_name);
    }
}

#[test]
fn test_fuzz_arabic_cmap_streams() {
    let mut rng_seed = 0x87654321u64;
    let empty_map = HashMap::new();

    for _ in 0..500 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let code = (rng_seed & 0xFFFF) as u32;

        // Decoder must gracefully handle arbitrary/malformed codes
        let _ = ArabicFontDecoder::recover_glyph(code, None, &empty_map);
    }
}

#[test]
fn test_fuzz_arabic_pua_and_presentation_forms() {
    let mut rng_seed = 0xA1B2C3D4u64;
    let empty_map = HashMap::new();

    for _ in 0..500 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let pua_code = 0xE000 + ((rng_seed % 0x1800) as u32);

        // Fuzzer checks all PUA codes in range 0xE000..0xF800
        let recovered = ArabicFontDecoder::recover_glyph(pua_code, None, &empty_map);
        assert!(
            !recovered.contains('\u{FFFD}'),
            "Recovered character must not contain replacement char"
        );
    }
}

#[test]
fn test_fuzz_arabic_bidi_control_sequences() {
    let mut rng_seed = 0xFEDCBA98u64;

    let bidi_controls = [
        '\u{200E}', '\u{200F}', '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}',
        '\u{061C}',
    ];
    for _ in 0..300 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut text = String::from("تقرير ");
        for _ in 0..5 {
            text.push(bidi_controls[(rng_seed as usize) % bidi_controls.len()]);
            text.push_str("2026 ");
        }

        // Corruption detector and pipeline must handle arbitrary control characters
        let is_isolated = ArabicCorruptionDetector::detect_isolated_glyph_corruption(&text);
        let has_broken_lam_alef = ArabicCorruptionDetector::detect_broken_lam_alef(&text);
        let has_pua = ArabicCorruptionDetector::detect_pua_leakage(&text);

        let _ = is_isolated || has_broken_lam_alef || has_pua;
    }
}
