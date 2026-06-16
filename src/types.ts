// Gedeelde types tussen frontend en (de JSON-payloads van) de Rust-backend.

export type Stage =
  | "idle"
  | "extracting"   // FFmpeg
  | "transcribing" // Whisper
  | "generating"   // Ollama
  | "documents"    // Pandoc / wkhtmltopdf
  | "done"
  | "error";

export interface ProgressEvent {
  stage: Stage;
  /** 0..100, of -1 als de voortgang onbepaald is. */
  percent: number;
  message: string;
}

export interface LogEvent {
  level: "info" | "warn" | "error";
  message: string;
  timestamp: string; // ISO-8601
}

export interface PipelineResult {
  outputDir: string;
  transcript: string;
  samenvatting: string;
  docx: string;
  pdf: string;
  actielijst: string;
}

export interface Settings {
  /** Namen vervangen door rolaanduidingen / initialen. */
  anonimiseren: boolean;
  /** Whisper sprekerdiarisatie-hint (tsdiarize). */
  sprekerherkenning: boolean;
  /** Automatisch actiepunten extraheren naar actielijst.csv. */
  actiepuntenExtractie: boolean;
  /** Vertrouwelijkheidslabel dat in de kop van de notulen komt. */
  vertrouwelijkheidslabel: "Openbaar" | "Intern" | "Vertrouwelijk" | "Strikt vertrouwelijk";
  /** Pad naar het ArtEZ-huisstijl DOCX-referentiebestand. */
  huisstijlTemplate: string;
  /** Ollama-model dat gebruikt wordt voor het genereren. */
  model: string;
  /** Taal van de opname (ISO-639-1), bv. "nl". */
  taal: string;
}

export interface AppConfig {
  models: string[];
  defaultModel: string;
  settings: Settings;
}
