//! Stap 5 & 6: documenten genereren.
//!
//! - DOCX: Pandoc met het ArtEZ-huisstijl referentiebestand.
//! - PDF: Pandoc (Markdown → HTML) gevolgd door WeasyPrint (HTML → PDF) met
//!   de huisstijl-stylesheet.

use crate::config::{Config, Settings};
use crate::logging::Reporter;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Markdown → DOCX via Pandoc, met huisstijl-referentiedocument indien aanwezig.
pub fn to_docx(
    reporter: &Reporter,
    cfg: &Config,
    settings: &Settings,
    markdown: &Path,
    docx_out: &Path,
) -> Result<()> {
    let pandoc = &cfg.binaries.pandoc;

    let mut cmd = Command::new(pandoc);
    cmd.arg(markdown)
        .args(["-f", "markdown", "-t", "docx"])
        .arg("-o")
        .arg(docx_out);

    let reference = Path::new(&settings.huisstijl_template);
    if reference.exists() {
        cmd.arg(format!("--reference-doc={}", reference.display()));
        reporter.info(format!("Huisstijl-template: {}", reference.display()));
    } else {
        reporter.warn(format!(
            "Huisstijl-template niet gevonden ({}); standaard Pandoc-opmaak gebruikt.",
            reference.display()
        ));
    }

    let status = cmd
        .status()
        .with_context(|| format!("Pandoc kon niet worden gestart ({pandoc})"))?;
    if !status.success() {
        bail!("Pandoc (DOCX) eindigde met foutcode {:?}", status.code());
    }
    reporter.info(format!("DOCX gemaakt: {}", docx_out.display()));
    Ok(())
}

/// Markdown → HTML (Pandoc) → PDF (WeasyPrint) met huisstijl-CSS.
pub fn to_pdf(reporter: &Reporter, cfg: &Config, markdown: &Path, pdf_out: &Path) -> Result<()> {
    let pandoc = &cfg.binaries.pandoc;
    let weasyprint = &cfg.binaries.weasyprint;

    // 1. Markdown → standalone HTML.
    let html_path = pdf_out.with_extension("html");
    let status = Command::new(pandoc)
        .arg(markdown)
        .args(["-f", "markdown", "-t", "html5", "--standalone", "--metadata", "title=Conceptnotulen"])
        .arg("-o")
        .arg(&html_path)
        .status()
        .with_context(|| format!("Pandoc kon niet worden gestart ({pandoc})"))?;
    if !status.success() {
        bail!("Pandoc (HTML) eindigde met foutcode {:?}", status.code());
    }

    // 2. HTML → PDF met WeasyPrint en huisstijl-stylesheet.
    let mut cmd = Command::new(weasyprint);
    let css = cfg.pdf_stylesheet();
    if css.exists() {
        cmd.args(["-s", &css.to_string_lossy()]);
    } else {
        reporter.warn(format!("PDF-stylesheet niet gevonden: {}", css.display()));
    }
    cmd.arg(&html_path).arg(pdf_out);

    let status = cmd
        .status()
        .with_context(|| format!("WeasyPrint kon niet worden gestart ({weasyprint})"))?;
    if !status.success() {
        bail!("WeasyPrint eindigde met foutcode {:?}", status.code());
    }

    // Tussentijds HTML-bestand opruimen.
    std::fs::remove_file(&html_path).ok();

    reporter.info(format!("PDF gemaakt: {}", pdf_out.display()));
    Ok(())
}
