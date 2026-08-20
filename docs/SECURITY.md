# Security Engineering & Threat Model (`pdf2md`)

This document outlines the security architecture, threat model, and hardening controls implemented in `pdf2md` to guarantee safe processing of untrusted, potentially hostile PDF documents.

---

## 1. Threat Model (STRIDE Analysis)

| Threat Category | Potential Attack Vectors | Implemented Defense Mechanism |
| :--- | :--- | :--- |
| **Denial of Service (DoS)** | Decompression/Zip bombs, nested Flate streams | `StreamDecoder` enforces a maximum stream size (default 32MB) and maximum expansion ratio (100x), terminating streaming decompression immediately if exceeded. |
| **Denial of Service (DoS)** | Cyclic indirect object reference loops (`1 0 R -> 2 0 R -> 1 0 R`) | `CycleDetector` maintains a visited object set and caps maximum object nesting depth to 64. |
| **Denial of Service (DoS)** | Image dimension bombs ($100,000 \times 100,000$ px) | `ImageExtractor` inspects header dimensions before allocation; drops any image exceeding $10,000 \times 10,000$ px. |
| **Denial of Service (DoS)** | Unbounded processing duration | Strict timeout watchdog in Rust pipeline and non-blocking `stream_select` loop in PHP CLI wrapper. |
| **Elevation of Privilege** | PDF JavaScript execution (`/JS`, `/JavaScript`, `/Launch`) | The parser ignores and strips all executable action dictionaries. PDF scripts are never evaluated. |
| **Information Disclosure** | Path traversal attacks via image extraction (`../../../../etc/passwd`) | `ImageExtractor::sanitize_filename` strips directory separators (`/`, `\`), null bytes (`\0`), and path traversal tokens (`..`), falling back to deterministic SHA hashes. |
| **Information Disclosure** | External entity/resource inclusion (`/F`, `/ImportData`) | External references are disabled by default (`allow_external_references: false`). Local file access is strictly sandboxed. |
| **Command Injection** | CLI subprocess execution from PHP/Laravel | `ProcessRunner` uses array vectors `['arg1', 'arg2']` with `proc_open` without invoking shell interpreters (`sh -c` / `cmd.exe`). |
| **Injection Attacks** | YAML Frontmatter injection & XSS payloads in PDF metadata | `FrontmatterRenderer` sanitizes quotes, colons, newlines, and control characters in metadata fields. |

---

## 2. Resource Limits Reference

All resource boundaries are configurable via `SecurityLimits` (Rust) or `Config` (PHP):

```rust
use pdf2md_pdf::security::SecurityLimits;

let limits = SecurityLimits {
    max_decompressed_stream_bytes: 32 * 1024 * 1024, // 32 MB per stream
    max_decompression_ratio: 100.0,                  // Max 100x expansion ratio
    max_pages: 5000,                                 // Max 5,000 pages per document
    max_objects_count: 100_000,                      // Max 100,000 indirect objects
    max_object_depth: 64,                            // Max 64 recursive nesting depth
    max_image_width: 10_000,                         // Max 10,000 px raster width
    max_image_height: 10_000,                        // Max 10,000 px raster height
    timeout_seconds: 60,                             // 60-second execution budget
    allow_javascript: false,                         // Never execute JavaScript
    allow_external_references: false,                // Never fetch external URLs
};
```

---

## 3. Deployment Hardening Guidelines for Public SaaS

1. **Non-Root Execution**: Run worker processes and background daemons under an unprivileged user (e.g. `www-data` or `nobody`).
2. **Container Isolation**: Deploy with read-only root filesystems and an ephemeral `tmpfs` mounted at `/tmp`.
3. **Memory Capping**: In Docker or Kubernetes, set hard memory limits (e.g. `mem_limit: 512m`) and CPU quotas (`cpus: 2.0`).
4. **Temporary Directory Cleaning**: Ensure automated temporary file scrubbers run periodically to clean orphaned diagnostics files.
