import type { AppConfig, Settings } from "../types";

interface Props {
  config: AppConfig;
  settings: Settings;
  disabled: boolean;
  onChange: (s: Settings) => void;
}

const LABELS = ["Openbaar", "Intern", "Vertrouwelijk", "Strikt vertrouwelijk"] as const;

export function SettingsPanel({ config, settings, disabled, onChange }: Props) {
  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    onChange({ ...settings, [key]: value });

  return (
    <details className="settings">
      <summary>Instellingen &amp; kwaliteit</summary>

      <div className="settings__grid">
        <label className="settings__row">
          <input
            type="checkbox"
            disabled={disabled}
            checked={settings.anonimiseren}
            onChange={(e) => set("anonimiseren", e.target.checked)}
          />
          <span>Namen anonimiseren</span>
        </label>

        <label className="settings__row">
          <input
            type="checkbox"
            disabled={disabled}
            checked={settings.sprekerherkenning}
            onChange={(e) => set("sprekerherkenning", e.target.checked)}
          />
          <span>Sprekerherkenning (diarisatie)</span>
        </label>

        <label className="settings__row">
          <input
            type="checkbox"
            disabled={disabled}
            checked={settings.actiepuntenExtractie}
            onChange={(e) => set("actiepuntenExtractie", e.target.checked)}
          />
          <span>Automatische actiepuntenextractie</span>
        </label>

        <label className="settings__row settings__row--wide">
          <span>Vertrouwelijkheidslabel</span>
          <select
            disabled={disabled}
            value={settings.vertrouwelijkheidslabel}
            onChange={(e) =>
              set(
                "vertrouwelijkheidslabel",
                e.target.value as Settings["vertrouwelijkheidslabel"]
              )
            }
          >
            {LABELS.map((l) => (
              <option key={l} value={l}>
                {l}
              </option>
            ))}
          </select>
        </label>

        <label className="settings__row settings__row--wide">
          <span>Ollama-model</span>
          <select
            disabled={disabled}
            value={settings.model}
            onChange={(e) => set("model", e.target.value)}
          >
            {config.models.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
        </label>

        <label className="settings__row settings__row--wide">
          <span>Taal opname</span>
          <input
            type="text"
            disabled={disabled}
            value={settings.taal}
            maxLength={5}
            onChange={(e) => set("taal", e.target.value)}
          />
        </label>

        <label className="settings__row settings__row--wide">
          <span>ArtEZ-huisstijl DOCX-template</span>
          <input
            type="text"
            disabled={disabled}
            value={settings.huisstijlTemplate}
            onChange={(e) => set("huisstijlTemplate", e.target.value)}
          />
        </label>
      </div>
    </details>
  );
}
