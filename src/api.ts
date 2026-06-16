// Dunne wrapper rond de Tauri-commando's en events.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AppConfig,
  LogEvent,
  PipelineResult,
  ProgressEvent,
  Settings,
} from "./types";

const AUDIO_EXTENSIONS = ["mp3", "wav", "m4a", "mp4", "webm", "mov"];

export async function pickFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: "Audio / Video",
        extensions: AUDIO_EXTENSIONS,
      },
    ],
  });
  return typeof selected === "string" ? selected : null;
}

export async function loadConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("load_config");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { settings });
}

/** Stap 2 + 3: FFmpeg-extractie en Whisper-transcriptie. */
export async function startTranscription(
  inputPath: string,
  settings: Settings
): Promise<string> {
  return invoke<string>("start_transcription", { inputPath, settings });
}

/** Stap 4 t/m 6: Ollama-notulen + Pandoc/wkhtmltopdf-documenten. */
export async function generateNotulen(
  inputPath: string,
  settings: Settings
): Promise<PipelineResult> {
  return invoke<PipelineResult>("generate_notulen", { inputPath, settings });
}

export async function openOutputDir(): Promise<void> {
  return invoke("open_output_dir");
}

export function onProgress(cb: (e: ProgressEvent) => void): Promise<UnlistenFn> {
  return listen<ProgressEvent>("pipeline://progress", (event) => cb(event.payload));
}

export function onLog(cb: (e: LogEvent) => void): Promise<UnlistenFn> {
  return listen<LogEvent>("pipeline://log", (event) => cb(event.payload));
}
