import { useCallback, useEffect, useMemo, useState } from "react";
import {
  generateNotulen,
  loadConfig,
  onLog,
  onProgress,
  openOutputDir,
  pickFile,
  saveSettings,
  startTranscription,
} from "./api";
import type {
  AppConfig,
  LogEvent,
  PipelineResult,
  ProgressEvent,
  Settings,
} from "./types";
import { ProgressBar } from "./components/ProgressBar";
import { LogPanel } from "./components/LogPanel";
import { SettingsPanel } from "./components/SettingsPanel";

const IDLE: ProgressEvent = { stage: "idle", percent: 0, message: "" };

export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [inputPath, setInputPath] = useState<string | null>(null);
  const [progress, setProgress] = useState<ProgressEvent>(IDLE);
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [busy, setBusy] = useState(false);
  const [transcribed, setTranscribed] = useState(false);
  const [result, setResult] = useState<PipelineResult | null>(null);

  // Config + event-listeners initialiseren.
  useEffect(() => {
    loadConfig()
      .then((cfg) => {
        setConfig(cfg);
        setSettings(cfg.settings);
      })
      .catch((e) => pushLog("error", `Config laden mislukt: ${e}`));

    const unsubProgress = onProgress(setProgress);
    const unsubLog = onLog((l) => setLogs((prev) => [...prev, l]));

    return () => {
      unsubProgress.then((fn) => fn());
      unsubLog.then((fn) => fn());
    };
  }, []);

  const pushLog = useCallback((level: LogEvent["level"], message: string) => {
    setLogs((prev) => [
      ...prev,
      { level, message, timestamp: new Date().toISOString() },
    ]);
  }, []);

  const handlePick = useCallback(async () => {
    try {
      const path = await pickFile();
      if (path) {
        setInputPath(path);
        setTranscribed(false);
        setResult(null);
        setProgress(IDLE);
        pushLog("info", `Bestand gekozen: ${path}`);
      }
    } catch (e) {
      pushLog("error", `Bestand kiezen mislukt: ${e}`);
    }
  }, [pushLog]);

  const handleTranscribe = useCallback(async () => {
    if (!inputPath || !settings) return;
    setBusy(true);
    try {
      const transcriptPath = await startTranscription(inputPath, settings);
      setTranscribed(true);
      pushLog("info", `Transcriptie gereed: ${transcriptPath}`);
    } catch (e) {
      setProgress({ stage: "error", percent: -1, message: String(e) });
      pushLog("error", `Transcriptie mislukt: ${e}`);
    } finally {
      setBusy(false);
    }
  }, [inputPath, settings, pushLog]);

  const handleGenerate = useCallback(async () => {
    if (!inputPath || !settings) return;
    setBusy(true);
    try {
      const res = await generateNotulen(inputPath, settings);
      setResult(res);
      pushLog("info", `Conceptnotulen gereed in ${res.outputDir}`);
    } catch (e) {
      setProgress({ stage: "error", percent: -1, message: String(e) });
      pushLog("error", `Genereren mislukt: ${e}`);
    } finally {
      setBusy(false);
    }
  }, [inputPath, settings, pushLog]);

  const handleSettingsChange = useCallback(
    (s: Settings) => {
      setSettings(s);
      saveSettings(s).catch((e) => pushLog("warn", `Instellingen opslaan mislukt: ${e}`));
    },
    [pushLog]
  );

  const fileName = useMemo(
    () => (inputPath ? inputPath.split(/[\\/]/).pop() : null),
    [inputPath]
  );

  if (!config || !settings) {
    return <div className="app app--loading">ArtEZ Notulist wordt geladen…</div>;
  }

  return (
    <div className="app">
      <header className="app__header">
        <h1>ArtEZ Notulist</h1>
        <p className="app__subtitle">
          Lokale conceptnotulen — College van Bestuur
        </p>
      </header>

      <section className="toolbar">
        <button className="btn" onClick={handlePick} disabled={busy}>
          Bestand kiezen
        </button>
        <button
          className="btn"
          onClick={handleTranscribe}
          disabled={busy || !inputPath}
        >
          Transcriptie starten
        </button>
        <button
          className="btn btn--primary"
          onClick={handleGenerate}
          disabled={busy || !inputPath || !transcribed}
          title={!transcribed ? "Start eerst de transcriptie" : ""}
        >
          Conceptnotulen genereren
        </button>
        <button
          className="btn"
          onClick={() => openOutputDir().catch((e) => pushLog("error", String(e)))}
        >
          Open outputmap
        </button>
      </section>

      <div className="filebar">
        {fileName ? (
          <span>
            Geselecteerd: <strong>{fileName}</strong>
          </span>
        ) : (
          <span className="filebar__hint">Nog geen bestand geselecteerd.</span>
        )}
      </div>

      <SettingsPanel
        config={config}
        settings={settings}
        disabled={busy}
        onChange={handleSettingsChange}
      />

      <ProgressBar progress={progress} />

      {result && (
        <section className="result">
          <h2>Resultaat</h2>
          <ul>
            <li>{result.transcript}</li>
            <li>{result.samenvatting}</li>
            <li>{result.docx}</li>
            <li>{result.pdf}</li>
            <li>{result.actielijst}</li>
          </ul>
        </section>
      )}

      <LogPanel logs={logs} />

      <footer className="app__footer">
        Volledig lokale verwerking · Whisper.cpp large-v3 · Ollama ·{" "}
        {settings.vertrouwelijkheidslabel}
      </footer>
    </div>
  );
}
