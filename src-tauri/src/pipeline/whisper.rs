//! Stap 3: transcriptie met whisper.cpp (whisper-cli) en model large-v3.
//!
//! GPU-versnelling wordt automatisch gebruikt wanneer whisper.cpp met CUDA/
//! Vulkan is gebouwd; met `-ng` schakelen we GPU expliciet uit als de
//! gebruiker dat instelt.

use crate::config::{Config, Settings};
use crate::logging::Reporter;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn transcribe(
    reporter: &Reporter,
    cfg: &Config,
    settings: &Settings,
    audio_wav: &Path,
    transcript_out: &Path,
) -> Result<()> {
    let whisper = &cfg.binaries.whisper;
    let model = &cfg.whisper.model_file;

    if !Path::new(model).exists() {
        bail!(
            "Whisper-model niet gevonden: {model}. Download het large-v3-model (zie installer)."
        );
    }

    // whisper-cli schrijft <output-prefix>.txt; we geven het prefix zonder extensie.
    let prefix = transcript_out.with_extension("");
    let prefix_str = prefix.to_string_lossy().to_string();

    let mut cmd = Command::new(whisper);
    cmd.args(["-m", model])
        .args(["-f", &audio_wav.to_string_lossy()])
        .args(["-l", &settings.taal])
        .args(["-t", &cfg.whisper.threads.to_string()])
        .arg("-otxt") // tekstuitvoer
        .args(["-of", &prefix_str]); // output-prefix (zonder .txt)

    if settings.sprekerherkenning {
        // Tinydiarize-tokens (vereist een model dat dit ondersteunt; anders genegeerd).
        cmd.arg("-tdrz");
        reporter.info("Sprekerherkenning (diarisatie) ingeschakeld.");
    }

    if !cfg.whisper.use_gpu {
        cmd.arg("-ng"); // no-gpu
        reporter.info("GPU uitgeschakeld; transcriptie draait op CPU.");
    } else {
        reporter.info("GPU-versnelling gebruikt indien beschikbaar.");
    }

    reporter.info(format!("Whisper start: model={model}, taal={}", settings.taal));

    let output = cmd
        .output()
        .with_context(|| format!("whisper-cli kon niet worden gestart ({whisper})"))?;

    if !output.stderr.is_empty() {
        // whisper.cpp logt voortgang op stderr.
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines().filter(|l| !l.trim().is_empty()) {
            reporter.info(format!("whisper: {line}"));
        }
    }

    if !output.status.success() {
        bail!("whisper-cli eindigde met foutcode {:?}", output.status.code());
    }

    // whisper-cli schreef <prefix>.txt; whisper.cpp gebruikt dezelfde naam als ons doel.
    let produced = prefix.with_extension("txt");
    if produced != transcript_out && produced.exists() {
        std::fs::rename(&produced, transcript_out).ok();
    }
    if !transcript_out.exists() {
        bail!("Geen transcript geproduceerd: {}", transcript_out.display());
    }

    reporter.info("Transcriptie gereed.");
    Ok(())
}
