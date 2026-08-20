export interface ConversionOptions {
  dialect?: 'commonmark' | 'gfm' | 'extended';
  detectTables?: boolean;
  extractImages?: boolean;
}

export interface Diagnostics {
  total_pages: number;
  text_pages: number;
  ocr_pages: number;
  tables_detected: number;
  images_extracted: number;
  overall_confidence: number;
}

export interface ConversionResponse {
  markdown: string;
  diagnostics: string;
}

export class PDFMarkdown {
  constructor(dialect?: string, detect_tables?: boolean);
  convert(pdf_bytes: Uint8Array): ConversionResponse;
}

export default function init(): Promise<void>;
