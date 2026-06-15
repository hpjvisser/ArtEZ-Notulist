//! Tauri-commando's die door de frontend worden aangeroepen.

use crate::config::{Config, Settings};
use crate::logging::Reporter;
use crate::pipeline::{self, PipelineResult};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Configuratie zoals de frontend die verwacht (camelCase).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    models: Vec<String>,
    default_model: String,
    settings: Settings,
}

fn reporter(app: &AppHandle) -> Reporter {
    let cfg = Config::load().unwrap_or_default();
    let _ = cfg.ensure_dirs();
    Reporter::new(app.clone(), cfg.paths.logs)
}

#[tauri::command]
pub fn load_config() -> Result<AppConfig, String> {
    let cfg = Config::load().map_err(|e| e.to_string())?;
    cfg.ensure_dirs().map_err(|e| e.to_string())?;
    Ok(AppConfig {
        models: cfg.ollama.models.clone(),
        default_model: cfg.ollama.default_model.clone(),
        settings: cfg.settings,
    })
}

#[tauri::command]
pub fn save_settings(settings: Settings) -> Result<(), String> {
    let mut cfg = Config::load().map_err(|e| e.to_string())?;
    cfg.settings = settings;
    cfg.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_transcription(
    app: AppHandle,
    input_path: String,
    settings: Settings,
) -> Result<String, String> {
    let cfg = Config::load().map_err(|e| e.to_string())?;
    cfg.ensure_dirs().map_err(|e| e.to_string())?;
    let rep = reporter(&app);
    let input = PathBuf::from(&input_path);

    match pipeline::transcribe(&rep, &cfg, &settings, &input).await {
        Ok(path) => Ok(path.to_string_lossy().into()),
        Err(e) => {
            rep.error(format!("Transcriptie mislukt: {e:#}"));
            Err(format!("{e:#}"))
        }
    }
}

#[tauri::command]
pub async fn generate_notulen(
    app: AppHandle,
    input_path: String,
    settings: Settings,
) -> Result<PipelineResult, String> {
    let cfg = Config::load().map_err(|e| e.to_string())?;
    cfg.ensure_dirs().map_err(|e| e.to_string())?;
    let rep = reporter(&app);
    let input = PathBuf::from(&input_path);

    match pipeline::generate(&rep, &cfg, &settings, &input).await {
        Ok(result) => Ok(result),
        Err(e) => {
            rep.error(format!("Genereren mislukt: {e:#}"));
            Err(format!("{e:#}"))
        }
    }
}

#[tauri::command]
pub fn open_output_dir(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let cfg = Config::load().map_err(|e| e.to_string())?;
    cfg.ensure_dirs().map_err(|e| e.to_string())?;
    app.opener()
        .open_path(cfg.paths.output.clone(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Zorg dat config + mappen bestaan bij het opstarten.
pub fn bootstrap(app: &AppHandle) {
    if let Ok(cfg) = Config::load() {
        let _ = cfg.ensure_dirs();
        // Eerste keer: kopieer gebundelde templates naar C:\ArtezNotulist\templates.
        if let Ok(resource_dir) = app.path().resource_dir() {
            let bundled = resource_dir.join("templates");
            if bundled.exists() {
                copy_missing(&bundled, std::path::Path::new(&cfg.paths.templates));
            }
        }
    }
}

/// Kopieer bestanden uit `src` naar `dst` als ze nog niet bestaan.
fn copy_missing(src: &std::path::Path, dst: &std::path::Path) {
    if let Ok(entries) = std::fs::read_dir(src) {
        let _ = std::fs::create_dir_all(dst);
        for entry in entries.flatten() {
            let from = entry.path();
            if let Some(name) = from.file_name() {
                let to = dst.join(name);
                if from.is_file() && !to.exists() {
                    let _ = std::fs::copy(&from, &to);
                }
            }
        }
    }
}
