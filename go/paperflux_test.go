package paperflux

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func findTestFixturesDir() string {
	candidates := []string{
		filepath.Join("..", "tests", "fixtures"),
		filepath.Join("tests", "fixtures"),
		filepath.Join("..", "..", "tests", "fixtures"),
	}

	for _, c := range candidates {
		if abs, err := filepath.Abs(c); err == nil {
			if info, err := os.Stat(abs); err == nil && info.IsDir() {
				return abs
			}
		}
	}
	return filepath.Join("..", "tests", "fixtures")
}

func TestConvertFileAcademicPaper(t *testing.T) {
	fixturesDir := findTestFixturesDir()
	pdfPath := filepath.Join(fixturesDir, "academic_bilingual_paper.pdf")

	if _, err := os.Stat(pdfPath); os.IsNotExist(err) {
		t.Skipf("Fixture file not found: %s", pdfPath)
	}

	cfg := DefaultConfig()
	result, err := ConvertFile(pdfPath, cfg)
	if err != nil {
		t.Fatalf("ConvertFile failed: %v", err)
	}

	if result.TotalPages < 3 {
		t.Errorf("Expected at least 3 pages, got %d", result.TotalPages)
	}

	if result.Confidence < 0.85 {
		t.Errorf("Expected confidence >= 0.85, got %f", result.Confidence)
	}

	if !strings.Contains(result.Markdown, "Machine Learning Approaches") {
		t.Errorf("Expected markdown to contain title, got:\n%s", result.Markdown)
	}
}

func TestConvertMemoryBytesFinancialReport(t *testing.T) {
	fixturesDir := findTestFixturesDir()
	pdfPath := filepath.Join(fixturesDir, "financial_annual_report_multipage_tables.pdf")

	if _, err := os.Stat(pdfPath); os.IsNotExist(err) {
		t.Skipf("Fixture file not found: %s", pdfPath)
	}

	pdfBytes, err := os.ReadFile(pdfPath)
	if err != nil {
		t.Fatalf("Failed to read fixture: %v", err)
	}

	cfg := DefaultConfig()
	cfg.DetectTables = true

	result, err := Convert(pdfBytes, cfg)
	if err != nil {
		t.Fatalf("Convert failed: %v", err)
	}

	if result.TotalPages < 4 {
		t.Errorf("Expected at least 4 pages, got %d", result.TotalPages)
	}

	if !strings.Contains(result.Markdown, "Alpha Holding Group") {
		t.Errorf("Expected title in output, got:\n%s", result.Markdown)
	}

	if !strings.Contains(result.Markdown, "|") {
		t.Errorf("Expected table markdown in output")
	}
}

func TestConvertStream(t *testing.T) {
	fixturesDir := findTestFixturesDir()
	pdfPath := filepath.Join(fixturesDir, "banking_account_statement.pdf")

	if _, err := os.Stat(pdfPath); os.IsNotExist(err) {
		t.Skipf("Fixture file not found: %s", pdfPath)
	}

	pdfBytes, err := os.ReadFile(pdfPath)
	if err != nil {
		t.Fatalf("Failed to read fixture: %v", err)
	}

	reader := bytes.NewReader(pdfBytes)
	cfg := DefaultConfig()

	result, err := ConvertStream(reader, cfg)
	if err != nil {
		t.Fatalf("ConvertStream failed: %v", err)
	}

	if result.TotalPages < 2 {
		t.Errorf("Expected at least 2 pages, got %d", result.TotalPages)
	}

	if !strings.Contains(result.Markdown, "National Commercial Bank") {
		t.Errorf("Expected header in output, got:\n%s", result.Markdown)
	}
}
