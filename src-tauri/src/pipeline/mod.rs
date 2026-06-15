//! De volledige verwerkingspijplijn:
//! FFmpeg → Whisper → Ollama → Pandoc → WeasyPrint → actielijst.

pub mod actions;
pub mod documents;
pub mod ffmpeg;
pub mod ollama;
pub mod whisper;

use crate::config::{Config, Settings};
use crate::logging::Reporter;
use anyhow::{Context, Result};
use chrono::Local;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Resultaatbestanden die naar de frontend gaan.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineResult {
    pub output_dir: String,
    pub transcript: String,
    pub samenvatting: String,
    pub docx: String,
    pub pdf: String,
    pub actielijst: String,
}

/// Werkmap en bestandsnamen voor één opname.
pub struct Job {
    pub input: PathBuf,
    pub work_dir: PathBuf,
    pub audio_wav: PathBuf,
    pub transcript: PathBuf,
    pub samenvatting: PathBuf,
    pub docx: PathBuf,
    pub pdf: PathBuf,
    pub actielijst: PathBuf,
}

impl Job {
    /// Maak een deterministische werkmap op basis van bestandsnaam en datum.
    pub fn new(input: &Path, cfg: &Config) -> Result<Self> {
        let stem = input
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "opname".into());
        let stamp = Local::now().format("%Y%m%d_%H%M%S");
        let work_dir = Path::new(&cfg.paths.output).join(format!("{stem}_{stamp}"));
        std::fs::create_dir_all(&work_dir)
            .with_context(|| format!("werkmap aanmaken mislukt: {}", work_dir.display()))?;

        Ok(Self {
            input: input.to_path_buf(),
            audio_wav: work_dir.join("audio.wav"),
            transcript: work_dir.join("transcript.txt"),
            samenvatting: work_dir.join("samenvatting.md"),
            docx: work_dir.join("concept_notulen.docx"),
            pdf: work_dir.join("concept_notulen.pdf"),
            actielijst: work_dir.join("actielijst.csv"),
            work_dir,
        })
    }

    /// Hervind een bestaande werkmap (de meest recente voor deze opname),
    /// zodat "genereren" na "transcriberen" hetzelfde transcript hergebruikt.
    pub fn latest_for(input: &Path, cfg: &Config) -> Option<PathBuf> {
        let stem = input.file_stem()?.to_string_lossy().to_string();
        let mut candidates: Vec<PathBuf> = std::fs::read_dir(&cfg.paths.output)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_dir()
                    && p.file_name()
                        .map(|n| n.to_string_lossy().starts_with(&format!("{stem}_")))
                        .unwrap_or(false)
                    && p.join("transcript.txt").exists()
            })
            .collect();
        candidates.sort();
        candidates.pop()
    }
}

/// Stap 1–3: audio-extractie + transcriptie. Geeft het transcriptpad terug.
pub async fn transcribe(
    reporter: &Reporter,
    cfg: &Config,
    settings: &Settings,
    input: &Path,
) -> Result<PathBuf> {
    let job = Job::new(input, cfg)?;

    reporter.progress("extracting", 5.0, "Audio extraheren met FFmpeg…");
    ffmpeg::extract_audio(reporter, cfg, &job.input, &job.audio_wav)?;

    reporter.progress("transcribing", 30.0, "Transcriptie met Whisper large-v3…");
    whisper::transcribe(reporter, cfg, settings, &job.audio_wav, &job.transcript)?;

    reporter.progress("done", 100.0, "Transcriptie gereed.");
    Ok(job.transcript)
}

/// Stap 4–6: notulen genereren en documenten produceren.
pub async fn generate(
    reporter: &Reporter,
    cfg: &Config,
    settings: &Settings,
    input: &Path,
) -> Result<PipelineResult> {
    // Hervind de werkmap met het bestaande transcript.
    let work_dir = Job::latest_for(input, cfg)
        .context("Geen transcript gevonden — start eerst de transcriptie.")?;
    let job = Job {
        input: input.to_path_buf(),
        audio_wav: work_dir.join("audio.wav"),
        transcript: work_dir.join("transcript.txt"),
        samenvatting: work_dir.join("samenvatting.md"),
        docx: work_dir.join("concept_notulen.docx"),
        pdf: work_dir.join("concept_notulen.pdf"),
        actielijst: work_dir.join("actielijst.csv"),
        work_dir,
    };

    let mut transcript = std::fs::read_to_string(&job.transcript)
        .with_context(|| format!("transcript lezen mislukt: {}", job.transcript.display()))?;

    if settings.anonimiseren {
        reporter.info("Namen worden geanonimiseerd vóór verwerking.");
        transcript = anonymize(&transcript);
    }

    reporter.progress("generating", 45.0, "Conceptnotulen genereren met Ollama…");
    let samenvatting = ollama::generate_notulen(reporter, cfg, settings, &transcript).await?;
    std::fs::write(&job.samenvatting, &samenvatting)
        .with_context(|| "samenvatting.md schrijven mislukt")?;

    reporter.progress("documents", 80.0, "DOCX maken met Pandoc…");
    documents::to_docx(reporter, cfg, settings, &job.samenvatting, &job.docx)?;

    reporter.progress("documents", 90.0, "PDF maken met WeasyPrint…");
    documents::to_pdf(reporter, cfg, &job.samenvatting, &job.pdf)?;

    if settings.actiepunten_extractie {
        reporter.progress("documents", 95.0, "Actiepunten extraheren…");
        actions::extract(reporter, &samenvatting, &job.actielijst)?;
    }

    reporter.progress("done", 100.0, "Conceptnotulen gereed.");

    Ok(PipelineResult {
        output_dir: job.work_dir.to_string_lossy().into(),
        transcript: job.transcript.to_string_lossy().into(),
        samenvatting: job.samenvatting.to_string_lossy().into(),
        docx: job.docx.to_string_lossy().into(),
        pdf: job.pdf.to_string_lossy().into(),
        actielijst: if settings.actiepunten_extractie {
            job.actielijst.to_string_lossy().into()
        } else {
            "(actiepuntenextractie uitgeschakeld)".into()
        },
    })
}

/// Eenvoudige anonimisering: vervang aaneengesloten hoofdletterwoorden
/// (vermoedelijke namen) door [PERSOON N]. Bewust conservatief.
fn anonymize(text: &str) -> String {
    use regex::Regex;
    // Twee of meer woorden die met een hoofdletter beginnen (Voornaam Achternaam).
    let re = Regex::new(r"\b([A-ZÀ-Þ][a-zà-ÿ]+)(\s+[A-ZÀ-Þ][a-zà-ÿ]+)+\b").unwrap();
    let mut counter = 0usize;
    let mut mapping: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    re.replace_all(text, |caps: &regex::Captures| {
        let name = caps[0].to_string();
        mapping
            .entry(name)
            .or_insert_with(|| {
                counter += 1;
                format!("[PERSOON {counter}]")
            })
            .clone()
    })
    .into_owned()
}
