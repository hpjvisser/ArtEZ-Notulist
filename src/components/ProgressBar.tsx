import type { ProgressEvent } from "../types";

const STAGE_LABEL: Record<string, string> = {
  idle: "Gereed",
  extracting: "Audio extraheren (FFmpeg)",
  transcribing: "Transcriptie (Whisper large-v3)",
  generating: "Conceptnotulen genereren (Ollama)",
  documents: "Documenten maken (Pandoc / WeasyPrint)",
  done: "Klaar",
  error: "Fout",
};

export function ProgressBar({ progress }: { progress: ProgressEvent }) {
  const indeterminate = progress.percent < 0;
  const pct = Math.max(0, Math.min(100, progress.percent));

  return (
    <div className="progress">
      <div className="progress__header">
        <span className="progress__stage">
          {STAGE_LABEL[progress.stage] ?? progress.stage}
        </span>
        <span className="progress__pct">
          {indeterminate ? "" : `${Math.round(pct)}%`}
        </span>
      </div>
      <div className={`progress__track ${progress.stage === "error" ? "is-error" : ""}`}>
        <div
          className={`progress__fill ${indeterminate ? "is-indeterminate" : ""}`}
          style={indeterminate ? undefined : { width: `${pct}%` }}
        />
      </div>
      {progress.message && <p className="progress__msg">{progress.message}</p>}
    </div>
  );
}
