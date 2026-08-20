import os
import sys

# Ensure UTF-8 output on Windows
if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    import paperflux
    print("[PASS] Successfully imported paperflux Python extension module!")
except ImportError as e:
    print(f"[FAIL] Failed to import paperflux: {e}")
    sys.exit(1)

# Synthetic minimal valid PDF
minimal_pdf = b"""%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> >>
endobj
4 0 obj
<< /Length 75 >>
stream
BT
/F1 14 Tf
72 700 Td
(PaperFlux Python FFI Integration) Tj
0 -20 Td
(Arabic: \xd8\xaa\xd9\x82\xd8\xb1\xd9\x8a\xd8\xb1 \xd8\xb9\xd9\x84\xd9\x85\xd9\x8a) Tj
ET
endstream
endobj
xref
0 5
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
0000000115 00000 n 
0000000300 00000 n 
trailer
<< /Size 5 /Root 1 0 R >>
startxref
450
%%EOF
"""

print("\n--- Testing convert_bytes() ---")
result = paperflux.convert_bytes(minimal_pdf, dialect="gfm", detect_tables=True)

print(f"  Result repr: {repr(result)}")
print(f"  Total pages: {result.total_pages}")
print(f"  Confidence: {result.confidence:.2f}")
print(f"  Tables detected: {result.tables_detected}")
print(f"  Markdown snippet:\n{result.markdown}")

assert result.total_pages == 1, "Expected total_pages == 1"
assert "PaperFlux Python FFI" in result.markdown, "Expected markdown to contain header"
assert result.confidence >= 0.85, "Expected high confidence"

diag_json = result.diagnostics_json()
assert len(diag_json) > 10, "Expected diagnostics JSON"
print(f"  [PASS] Diagnostics JSON parsed ({len(diag_json)} bytes)")

# Test saving to temporary file and using convert()
temp_pdf_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "test_temp.pdf")
try:
    with open(temp_pdf_path, "wb") as f:
        f.write(minimal_pdf)

    print("\n--- Testing convert(path) ---")
    file_result = paperflux.convert(temp_pdf_path, dialect="gfm")
    assert file_result.total_pages == 1
    assert "PaperFlux Python FFI" in file_result.markdown
    print("  [PASS] convert(path) successfully executed!")
finally:
    if os.path.exists(temp_pdf_path):
        os.remove(temp_pdf_path)

print("\n=== ALL PYTHON FFI BINDINGS TESTS PASSED! ===\n")
