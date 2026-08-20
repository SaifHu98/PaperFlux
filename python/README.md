# PaperFlux Python SDK

Official Python bindings for **PaperFlux** — the high-performance universal PDF intelligence engine with first-class Arabic support, powered by Rust & PyO3.

## Installation

```bash
pip install paperflux
```

## Quickstart

```python
import paperflux

# Convert from file
result = paperflux.convert("document.pdf", dialect="gfm", detect_tables=True)
print(result.markdown)
print(f"Confidence: {result.confidence:.2f}")

# Convert from memory bytes
with open("paper.pdf", "rb") as f:
    pdf_bytes = f.read()

result = paperflux.convert_bytes(pdf_bytes, profile="fast")
print(f"Processed {result.total_pages} pages in high-throughput mode.")
```
