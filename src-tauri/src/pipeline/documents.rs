//! Stap 5 & 6: documenten genereren.
//!
//! - DOCX: Pandoc met het ArtEZ-huisstijl referentiebestand.
//! - PDF: Pandoc (Markdown → zelfstandige HTML met ingebedde huisstijl-CSS)
//!   gevolgd door wkhtmltopdf (HTML → PDF).

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

/// Markdown → HTML (Pandoc, CSS ingebed) → PDF (wkhtmltopdf).
pub fn to_pdf(reporter: &Reporter, cfg: &Config, markdown: &Path, pdf_out: &Path) -> Result<()> {
    let pandoc = &cfg.binaries.pandoc;
    let wkhtmltopdf = &cfg.binaries.wkhtmltopdf;

    // 1. Markdown → zelfstandige HTML met ingebedde huisstijl-CSS.
    let html_path = pdf_out.with_extension("html");
    let mut pcmd = Command::new(pandoc);
    pcmd.arg(markdown)
        .args(["-f", "markdown", "-t", "html5", "--standalone"])
        .args(["--metadata", "title=Conceptnotulen"]);

    let css = cfg.pdf_stylesheet();
    if css.exists() {
        // --embed-resources zorgt dat de CSS in de HTML wordt opgenomen,
        // zodat wkhtmltopdf geen losse bestanden hoeft te vinden.
        pcmd.args(["--embed-resources", "--css", &css.to_string_lossy()]);
    } else {
        reporter.warn(format!("PDF-stylesheet niet gevonden: {}", css.display()));
    }
    pcmd.arg("-o").arg(&html_path);

    let status = pcmd
        .status()
        .with_context(|| format!("Pandoc kon niet worden gestart ({pandoc})"))?;
    if !status.success() {
        bail!("Pandoc (HTML) eindigde met foutcode {:?}", status.code());
    }

    // 2. HTML → PDF met wkhtmltopdf (marges + concept-voettekst via CLI).
    let status = Command::new(wkhtmltopdf)
        .args(["--enable-local-file-access", "--quiet"])
        .args(["--margin-top", "25mm", "--margin-bottom", "20mm"])
        .args(["--margin-left", "22mm", "--margin-right", "22mm"])
        .args(["--footer-left", "Concept — niet vastgesteld"])
        .args(["--footer-right", "Pagina [page] / [topage]"])
        .args(["--footer-font-size", "8", "--footer-spacing", "5"])
        .arg(&html_path)
        .arg(pdf_out)
        .status()
        .with_context(|| {
            format!("wkhtmltopdf kon niet worden gestart ({wkhtmltopdf})")
        })?;
    if !status.success() {
        bail!("wkhtmltopdf eindigde met foutcode {:?}", status.code());
    }

    // Tussentijds HTML-bestand opruimen.
    std::fs::remove_file(&html_path).ok();

    reporter.info(format!("PDF gemaakt: {}", pdf_out.display()));
    Ok(())
}
