//! Stap 4: conceptnotulen genereren met een lokaal Ollama-model.

use crate::config::{Config, Settings};
use crate::logging::Reporter;
use anyhow::{bail, Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: String,
    system: String,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Serialize)]
struct GenerateOptions {
    temperature: f32,
    num_ctx: u32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

/// Lees het prompt-template en vul de variabelen in.
fn build_prompt(cfg: &Config, settings: &Settings, transcript: &str) -> Result<(String, String)> {
    let template_path = cfg.prompt_template();
    let template = std::fs::read_to_string(&template_path).with_context(|| {
        format!(
            "Prompt-template niet gevonden: {}. Controleer de templates-map.",
            template_path.display()
        )
    })?;

    let datum = Local::now().format("%d-%m-%Y").to_string();

    // Splits het template in een systeem- en gebruikersdeel op de marker.
    let (system_part, user_part) = match template.split_once("<!-- /SYSTEM -->") {
        Some((s, u)) => (s.to_string(), u.to_string()),
        None => (template.clone(), String::new()),
    };

    let fill = |s: &str| {
        s.replace("{{DATUM}}", &datum)
            .replace("{{VERTROUWELIJKHEID}}", &settings.vertrouwelijkheidslabel)
            .replace("{{TAAL}}", &settings.taal)
            .replace("{{TRANSCRIPT}}", transcript)
    };

    let system = fill(&system_part);
    let user = if user_part.trim().is_empty() {
        format!("Transcript van de vergadering:\n\n{transcript}")
    } else {
        fill(&user_part)
    };

    Ok((system, user))
}

pub async fn generate_notulen(
    reporter: &Reporter,
    cfg: &Config,
    settings: &Settings,
    transcript: &str,
) -> Result<String> {
    let model = if settings.model.trim().is_empty() {
        cfg.ollama.default_model.clone()
    } else {
        settings.model.clone()
    };

    let (system, prompt) = build_prompt(cfg, settings, transcript)?;
    reporter.info(format!("Ollama-model: {model} ({} tekens transcript)", transcript.len()));

    let req = GenerateRequest {
        model: &model,
        prompt,
        system,
        stream: false,
        options: GenerateOptions {
            temperature: cfg.ollama.temperature,
            num_ctx: cfg.ollama.num_ctx,
        },
    };

    let url = format!("{}/api/generate", cfg.ollama.host.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1800))
        .build()?;

    let resp = client.post(&url).json(&req).send().await.with_context(|| {
        format!("Kon Ollama niet bereiken op {url}. Draait `ollama serve`?")
    })?;

    if !resp.status().is_success() {
        let code = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("Ollama gaf status {code}: {body}");
    }

    let parsed: GenerateResponse = resp.json().await.context("Ollama-antwoord parsen mislukt")?;
    let text = parsed.response.trim().to_string();

    if text.is_empty() {
        bail!("Ollama gaf een leeg antwoord terug.");
    }

    reporter.info("Conceptnotulen ontvangen van Ollama.");
    Ok(prepend_frontmatter(settings, &text))
}

/// Zorg dat een vertrouwelijkheidsregel bovenaan staat (voor de documenten).
fn prepend_frontmatter(settings: &Settings, body: &str) -> String {
    let datum = Local::now().format("%d-%m-%Y").to_string();
    if body.contains("Vertrouwelijkheid:") {
        return body.to_string();
    }
    format!(
        "> **Vertrouwelijkheid:** {label} · Gegenereerd op {datum} · Concept\n\n{body}",
        label = settings.vertrouwelijkheidslabel,
    )
}
