//! Voortgangs- en logmeldingen naar de frontend sturen én naar een logbestand.

use chrono::Local;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct ProgressEvent {
    pub stage: String,
    pub percent: f32,
    pub message: String,
}

#[derive(Clone, Serialize)]
pub struct LogEvent {
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

/// Hulpstructuur die events uitzendt en wegschrijft naar logs/notulist.log.
#[derive(Clone)]
pub struct Reporter {
    app: AppHandle,
    log_dir: String,
}

impl Reporter {
    pub fn new(app: AppHandle, log_dir: impl Into<String>) -> Self {
        Self {
            app,
            log_dir: log_dir.into(),
        }
    }

    pub fn progress(&self, stage: &str, percent: f32, message: impl Into<String>) {
        let msg = message.into();
        let _ = self.app.emit(
            "pipeline://progress",
            ProgressEvent {
                stage: stage.to_string(),
                percent,
                message: msg.clone(),
            },
        );
        self.log("info", &format!("[{stage}] {msg}"));
    }

    pub fn info(&self, message: impl Into<String>) {
        self.log("info", &message.into());
    }

    pub fn warn(&self, message: impl Into<String>) {
        self.log("warn", &message.into());
    }

    pub fn error(&self, message: impl Into<String>) {
        self.log("error", &message.into());
    }

    fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().to_rfc3339();
        let _ = self.app.emit(
            "pipeline://log",
            LogEvent {
                level: level.to_string(),
                message: message.to_string(),
                timestamp: timestamp.clone(),
            },
        );
        self.write_to_file(level, message, &timestamp);
    }

    fn write_to_file(&self, level: &str, message: &str, timestamp: &str) {
        let path = Path::new(&self.log_dir).join("notulist.log");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{timestamp} {level:>5}  {message}");
        }
    }
}
