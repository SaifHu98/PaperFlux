package paperflux

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

// Config defines the configuration options for PDF to Markdown conversion.
type Config struct {
	Dialect         string // "gfm", "commonmark", "extended" (default: "gfm")
	DetectTables    bool   // Enable/disable multi-page table detection (default: true)
	ExtractImages   bool   // Extract embedded raster images
	ExtractVectors  bool   // Extract vector charts and schematic diagrams to SVG
	ImagesDir       string // Directory where extracted assets will be saved
	OcrMode         string // "auto", "always", "never" (default: "auto")
	OcrLang         string // OCR language codes (default: "ara+eng")
	MemoryLimitMB   int    // Maximum memory limit in MB (default: 256)
	TimeoutSec      int    // Processing timeout in seconds (default: 60)
	BinaryPath      string // Custom path to pdf2md CLI binary (optional)
}

// DefaultConfig returns the recommended default configuration.
func DefaultConfig() Config {
	return Config{
		Dialect:        "gfm",
		DetectTables:   true,
		ExtractImages:  false,
		ExtractVectors: false,
		OcrMode:        "auto",
		OcrLang:        "ara+eng",
		MemoryLimitMB:  256,
		TimeoutSec:     60,
	}
}

// DiagnosticsOutput represents the telemetry and diagnostics returned by the engine.
type DiagnosticsOutput struct {
	TotalPages        int      `json:"total_pages"`
	OverallConfidence float64  `json:"overall_confidence"`
	DurationMs        float64  `json:"duration_ms"`
	Warnings          []string `json:"warnings"`
}

// ConversionResult contains the generated Markdown and conversion metrics.
type ConversionResult struct {
	Markdown    string
	TotalPages  int
	Confidence  float64
	DurationMs  float64
	Warnings    []string
	RawOutput   string
}

// findBinary locates the PaperFlux CLI binary across workspace and system paths.
func findBinary(customPath string) (string, error) {
	if customPath != "" {
		if _, err := os.Stat(customPath); err == nil {
			return customPath, nil
		}
		return "", fmt.Errorf("specified PaperFlux binary not found: %s", customPath)
	}

	binName := "pdf2md"
	if runtime.GOOS == "windows" {
		binName = "pdf2md.exe"
	}

	// Check PATH
	if p, err := exec.LookPath(binName); err == nil {
		return p, nil
	}

	// Search relative workspace target directories
	candidates := []string{
		binName,
		filepath.Join("target", "release", binName),
		filepath.Join("target", "debug", binName),
		filepath.Join("..", "target", "release", binName),
		filepath.Join("..", "target", "debug", binName),
		filepath.Join("..", "..", "target", "release", binName),
		filepath.Join("..", "..", "target", "debug", binName),
	}

	for _, cand := range candidates {
		if abs, err := filepath.Abs(cand); err == nil {
			if _, err := os.Stat(abs); err == nil {
				return abs, nil
			}
		}
	}

	return "", errors.New("pdf2md executable not found. Ensure target/release/pdf2md is built or install it in PATH")
}

// Convert converts raw in-memory PDF bytes into Markdown.
func Convert(pdfBytes []byte, cfg Config) (*ConversionResult, error) {
	if len(pdfBytes) == 0 {
		return nil, errors.New("input PDF byte slice is empty")
	}

	binPath, err := findBinary(cfg.BinaryPath)
	if err != nil {
		return nil, err
	}

	tmpDiag, err := os.CreateTemp("", "pdf2md_diag_*.json")
	if err != nil {
		return nil, fmt.Errorf("failed to create temp diagnostics file: %w", err)
	}
	tmpDiagPath := tmpDiag.Name()
	tmpDiag.Close()
	defer os.Remove(tmpDiagPath)

	args := []string{"-"} // Read from stdin

	if cfg.Dialect != "" {
		args = append(args, "--dialect", cfg.Dialect)
	} else {
		args = append(args, "--dialect", "gfm")
	}

	if !cfg.DetectTables {
		args = append(args, "--no-tables")
	}

	if cfg.ExtractImages {
		args = append(args, "--extract-images")
	}

	if cfg.ExtractVectors {
		args = append(args, "--extract-vectors")
	}

	if cfg.ImagesDir != "" {
		args = append(args, "--images-dir", cfg.ImagesDir)
	}

	if cfg.OcrMode != "" {
		args = append(args, "--ocr", cfg.OcrMode)
	}

	if cfg.OcrLang != "" {
		args = append(args, "--ocr-lang", cfg.OcrLang)
	}

	if cfg.MemoryLimitMB > 0 {
		args = append(args, "--memory-limit-mb", fmt.Sprintf("%d", cfg.MemoryLimitMB))
	}

	if cfg.TimeoutSec > 0 {
		args = append(args, "--timeout", fmt.Sprintf("%d", cfg.TimeoutSec))
	}

	args = append(args, "--diagnostics-json", tmpDiagPath)

	startTime := time.Now()
	cmd := exec.Command(binPath, args...)

	var stdout, stderr bytes.Buffer
	cmd.Stdin = bytes.NewReader(pdfBytes)
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	if err := cmd.Run(); err != nil {
		errMsg := strings.TrimSpace(stderr.String())
		if errMsg == "" {
			errMsg = err.Error()
		}
		return nil, fmt.Errorf("conversion failed: %s", errMsg)
	}

	elapsed := time.Since(startTime)
	markdown := stdout.String()

	var diag DiagnosticsOutput
	if diagData, err := os.ReadFile(tmpDiagPath); err == nil && len(diagData) > 0 {
		_ = json.Unmarshal(diagData, &diag)
	}

	totalPages := diag.TotalPages
	if totalPages == 0 {
		totalPages = 1
	}

	confidence := diag.OverallConfidence
	if confidence == 0.0 {
		confidence = 0.95
	}

	durationMs := float64(elapsed.Microseconds()) / 1000.0
	if diag.DurationMs > 0 {
		durationMs = diag.DurationMs
	}

	return &ConversionResult{
		Markdown:   markdown,
		TotalPages: totalPages,
		Confidence: confidence,
		DurationMs: durationMs,
		Warnings:   diag.Warnings,
		RawOutput:  markdown,
	}, nil
}

// ConvertFile converts a PDF document file on disk into Markdown.
func ConvertFile(filePath string, cfg Config) (*ConversionResult, error) {
	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read PDF file '%s': %w", filePath, err)
	}
	return Convert(data, cfg)
}

// ConvertStream reads PDF data from an io.Reader and returns the conversion result.
func ConvertStream(reader io.Reader, cfg Config) (*ConversionResult, error) {
	data, err := io.ReadAll(reader)
	if err != nil {
		return nil, fmt.Errorf("failed to read stream: %w", err)
	}
	return Convert(data, cfg)
}
