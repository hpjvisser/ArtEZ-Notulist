//! Stap 2: audio extraheren met FFmpeg naar 16 kHz mono WAV (wat whisper.cpp wil).

use crate::config::Config;
use crate::logging::Reporter;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn extract_audio(
    reporter: &Reporter,
    cfg: &Config,
    input: &Path,
    output_wav: &Path,
) -> Result<()> {
    let ffmpeg = &cfg.binaries.ffmpeg;
    reporter.info(format!("FFmpeg: {} → {}", input.display(), output_wav.display()));

    let status = Command::new(ffmpeg)
        .args(["-y", "-i"])
        .arg(input)
        .args([
            "-vn", // geen video
            "-ac", "1", // mono
            "-ar", "16000", // 16 kHz
            "-c:a", "pcm_s16le",
        ])
        .arg(output_wav)
        .status()
        .with_context(|| {
            format!(
                "FFmpeg kon niet worden gestart ({ffmpeg}). Is FFmpeg geïnstalleerd en in PATH?"
            )
        })?;

    if !status.success() {
        bail!("FFmpeg eindigde met foutcode {:?}", status.code());
    }
    if !output_wav.exists() {
        bail!("FFmpeg leverde geen audiobestand op: {}", output_wav.display());
    }
    reporter.info("Audio-extractie gereed.");
    Ok(())
}
