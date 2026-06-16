//! Laden en bewaren van de applicatieconfiguratie.
//!
//! De configuratie staat in `C:\ArtezNotulist\config.toml`. Daar kunnen later
//! eenvoudig andere modellen, binaries of paden worden ingesteld zonder
//! hercompilatie.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Vaste hoofdmap zoals beschreven in de installatie-eisen.
pub const ROOT: &str = r"C:\ArtezNotulist";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Paths {
    pub root: String,
    pub inbox: String,
    pub output: String,
    pub logs: String,
    pub templates: String,
    pub models: String,
}

impl Default for Paths {
    fn default() -> Self {
        let root = PathBuf::from(ROOT);
        Self {
            inbox: root.join("inbox").to_string_lossy().into(),
            output: root.join("output").to_string_lossy().into(),
            logs: root.join("logs").to_string_lossy().into(),
            templates: root.join("templates").to_string_lossy().into(),
            models: root.join("models").to_string_lossy().into(),
            root: root.to_string_lossy().into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Binaries {
    /// Pad naar ffmpeg.exe (of "ffmpeg" als het in PATH staat).
    pub ffmpeg: String,
    /// Pad naar whisper-cli.exe (whisper.cpp).
    pub whisper: String,
    /// Pad naar pandoc.exe.
    pub pandoc: String,
    /// Pad naar wkhtmltopdf.exe (HTML → PDF).
    pub wkhtmltopdf: String,
}

impl Default for Binaries {
    fn default() -> Self {
        let root = PathBuf::from(ROOT);
        Self {
            ffmpeg: root.join(r"bin\ffmpeg.exe").to_string_lossy().into(),
            whisper: root
                .join(r"whisper\whisper-cli.exe")
                .to_string_lossy()
                .into(),
            pandoc: root.join(r"bin\pandoc.exe").to_string_lossy().into(),
            wkhtmltopdf: r"C:\Program Files\wkhtmltopdf\bin\wkhtmltopdf.exe".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WhisperConfig {
    /// Modelnaam (large-v3) en pad naar het ggml-bestand.
    pub model_file: String,
    /// Aantal threads.
    pub threads: u32,
    /// GPU gebruiken indien beschikbaar (whisper.cpp -ng = uitschakelen).
    pub use_gpu: bool,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        let root = PathBuf::from(ROOT);
        Self {
            model_file: root
                .join(r"models\ggml-large-v3.bin")
                .to_string_lossy()
                .into(),
            threads: 8,
            use_gpu: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OllamaConfig {
    /// Endpoint van de lokale Ollama-server.
    pub host: String,
    /// Standaardmodel.
    pub default_model: String,
    /// Alle modellen die in de UI gekozen kunnen worden.
    pub models: Vec<String>,
    /// Temperatuur voor het genereren (laag = feitelijk).
    pub temperature: f32,
    /// Contextvenster (num_ctx).
    pub num_ctx: u32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "http://127.0.0.1:11434".into(),
            default_model: "mistral".into(),
            models: vec![
                "mistral".into(),
                "llama3.1".into(),
                "qwen2.5".into(),
                "gemma2".into(),
            ],
            temperature: 0.2,
            num_ctx: 8192,
        }
    }
}

/// Kwaliteits- en stijlinstellingen die vanuit de UI worden aangepast.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub anonimiseren: bool,
    pub sprekerherkenning: bool,
    pub actiepunten_extractie: bool,
    pub vertrouwelijkheidslabel: String,
    pub huisstijl_template: String,
    pub model: String,
    pub taal: String,
}

impl Default for Settings {
    fn default() -> Self {
        let root = PathBuf::from(ROOT);
        Self {
            anonimiseren: false,
            sprekerherkenning: false,
            actiepunten_extractie: true,
            vertrouwelijkheidslabel: "Intern".into(),
            huisstijl_template: root
                .join(r"templates\artez_huisstijl.docx")
                .to_string_lossy()
                .into(),
            model: "mistral".into(),
            taal: "nl".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub paths: Paths,
    pub binaries: Binaries,
    pub whisper: WhisperConfig,
    pub ollama: OllamaConfig,
    pub settings: Settings,
}

impl Config {
    /// Pad naar het configuratiebestand.
    pub fn config_path() -> PathBuf {
        PathBuf::from(ROOT).join("config.toml")
    }

    /// Laad de configuratie; maak een standaardbestand aan als het ontbreekt.
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("config lezen mislukt: {}", path.display()))?;
            let cfg: Config =
                toml::from_str(&raw).with_context(|| "config.toml kon niet worden geparsed")?;
            Ok(cfg)
        } else {
            let cfg = Config::default();
            // Best-effort: probeer de standaardconfig weg te schrijven.
            let _ = cfg.save();
            Ok(cfg)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let toml = toml::to_string_pretty(self)?;
        std::fs::write(&path, toml)
            .with_context(|| format!("config schrijven mislukt: {}", path.display()))?;
        Ok(())
    }

    /// Zorg dat alle vereiste mappen bestaan.
    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [
            &self.paths.root,
            &self.paths.inbox,
            &self.paths.output,
            &self.paths.logs,
            &self.paths.templates,
            &self.paths.models,
        ] {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("map aanmaken mislukt: {dir}"))?;
        }
        Ok(())
    }

    /// Pad naar het prompt-template.
    pub fn prompt_template(&self) -> PathBuf {
        Path::new(&self.paths.templates).join("notulen_prompt.md")
    }

    /// Pad naar de PDF-stylesheet (huisstijl).
    pub fn pdf_stylesheet(&self) -> PathBuf {
        Path::new(&self.paths.templates).join("pdf_style.css")
    }
}
