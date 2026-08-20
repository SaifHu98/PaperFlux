export type MarkdownDialect = 'commonmark' | 'gfm' | 'extended';

export interface DocumentMetadata {
  title?: string;
  author?: string;
  subject?: string;
  keywords?: string[];
  creator?: string;
  producer?: string;
  creation_date?: string;
  mod_date?: string;
  total_pages: number;
  pdf_version?: string;
  is_encrypted: boolean;
}

export interface ConfidenceScores {
  text_confidence: number;
  reading_order_confidence: number;
  heading_confidence: number;
  table_confidence: number;
  ocr_confidence: number;
  language_confidence: number;
  layout_confidence: number;
}

export interface PageDiagnostics {
  page_number: number;
  is_scanned: boolean;
  ocr_applied: boolean;
  glyph_count: number;
  image_count: number;
  table_count: number;
  detected_language?: string;
  confidence: number;
  reading_order_score: number;
}

export interface ConversionWarning {
  code: string;
  message: string;
  page?: number;
  category: string;
}

export interface ProcessingStatistics {
  parse_time_ms: number;
  layout_time_ms: number;
  render_time_ms: number;
  total_time_ms: number;
  memory_peak_bytes: number;
}

export interface Diagnostics {
  total_pages: number;
  text_pages: number;
  ocr_pages: number;
  tables_detected: number;
  images_extracted: number;
  overall_confidence: number;
  confidence_breakdown: ConfidenceScores;
  pages: PageDiagnostics[];
  warnings: ConversionWarning[];
  stats: ProcessingStatistics;
}

export interface ConversionResult {
  markdown: string;
  metadata: DocumentMetadata;
  diagnostics: Diagnostics;
  statistics: ProcessingStatistics;
  confidence: number;
  warnings: ConversionWarning[];
  processedLocally: boolean;
}

export interface ProgressEvent {
  currentPage: number;
  totalPages: number;
  percent: number;
  stage: 'parsing' | 'extracting' | 'layout' | 'rendering' | 'completed';
}

export interface FallbackConfig {
  endpoint: string;
  authToken?: string;
  csrfToken?: string;
  timeoutMs?: number;
  chunkSizeMB?: number;
  headers?: Record<string, string>;
}

export interface ConversionOptions {
  dialect?: MarkdownDialect;
  detectTables?: boolean;
  extractImages?: boolean;
  maxBrowserPages?: number;
  maxBrowserFileSizeMB?: number;
  allowServerFallback?: boolean;
  fallbackConfig?: FallbackConfig;
  signal?: AbortSignal;
  onProgress?: (event: ProgressEvent) => void;
  onPageStart?: (page: number, total: number) => void;
  onPageComplete?: (page: number, total: number, pageMarkdown: string) => void;
  onWarning?: (warning: ConversionWarning) => void;
  onComplete?: (result: ConversionResult) => void;
}

export interface DocumentCapabilities {
  pageCount: number;
  scannedPages: number;
  digitalPages: number;
  totalImages: number;
  requiresOcr: boolean;
  isEncrypted: boolean;
}
