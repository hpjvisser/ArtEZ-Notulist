//! Automatische actiepuntenextractie → actielijst.csv.
//!
//! Twee bronnen worden gecombineerd:
//!   1. Inline-acties in de tekst, genoteerd als `(actie Achternaam)`.
//!   2. Een eventuele expliciete sectie "## Acties" met opsommings- of
//!      tabelregels.

use crate::logging::Reporter;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
struct Action {
    nr: usize,
    actiehouder: String,
    omschrijving: String,
    bron: String,
}

pub fn extract(reporter: &Reporter, markdown: &str, csv_out: &Path) -> Result<()> {
    let mut actions: Vec<Action> = Vec::new();

    // 1. Inline-acties: (actie Achternaam) — pak de zin eromheen als omschrijving.
    let inline = Regex::new(r"\(actie\s+([^)]+)\)").unwrap();
    let mut current_agenda = String::new();
    let agenda_re = Regex::new(r"^\s*#{1,6}\s*(\d+[.\d]*)").unwrap();

    for line in markdown.lines() {
        if let Some(cap) = agenda_re.captures(line) {
            current_agenda = cap[1].trim().to_string();
        }
        for cap in inline.captures_iter(line) {
            let houder = cap[1].trim().to_string();
            let omschrijving = clean_sentence(line, &cap[0]);
            actions.push(Action {
                nr: actions.len() + 1,
                actiehouder: houder,
                omschrijving,
                bron: if current_agenda.is_empty() {
                    "—".into()
                } else {
                    format!("Agendapunt {current_agenda}")
                },
            });
        }
    }

    // 2. Expliciete "## Acties"-sectie met opsommingstekens.
    if let Some(section) = extract_section(markdown, "Acties") {
        let bullet = Regex::new(r"^\s*[-*]\s+(.*)$").unwrap();
        for line in section.lines() {
            if let Some(cap) = bullet.captures(line) {
                let text = cap[1].trim();
                if text.is_empty() {
                    continue;
                }
                // Vermijd dubbele inline-acties.
                if inline.is_match(text) {
                    continue;
                }
                let (houder, omschrijving) = split_houder(text);
                actions.push(Action {
                    nr: actions.len() + 1,
                    actiehouder: houder,
                    omschrijving,
                    bron: "Sectie Acties".into(),
                });
            }
        }
    }

    // Schrijf CSV (UTF-8, scheidingsteken komma).
    let mut wtr = csv::WriterBuilder::new().from_path(csv_out).with_context(|| {
        format!("actielijst.csv kon niet worden gemaakt: {}", csv_out.display())
    })?;
    wtr.write_record(["Nr", "Actiehouder", "Actie", "Bron"])?;
    for a in &actions {
        wtr.write_record([
            a.nr.to_string(),
            a.actiehouder.clone(),
            a.omschrijving.clone(),
            a.bron.clone(),
        ])?;
    }
    wtr.flush()?;

    reporter.info(format!("{} actiepunt(en) geëxtraheerd.", actions.len()));
    Ok(())
}

/// Verwijder de `(actie …)`-markering en markdown-ruis uit een zin.
fn clean_sentence(line: &str, marker: &str) -> String {
    let s = line.replace(marker, "");
    let s = s.trim_start_matches(['-', '*', '#', ' ']);
    s.trim().to_string()
}

/// Splits "Achternaam: doet iets" of "Achternaam – doet iets" in houder + tekst.
fn split_houder(text: &str) -> (String, String) {
    for sep in [": ", " – ", " - ", " — "] {
        if let Some((h, rest)) = text.split_once(sep) {
            if h.split_whitespace().count() <= 3 {
                return (h.trim().to_string(), rest.trim().to_string());
            }
        }
    }
    ("—".into(), text.to_string())
}

/// Haal de tekst onder een markdown-kop (## <naam>) tot de volgende kop.
fn extract_section(markdown: &str, name: &str) -> Option<String> {
    let mut out = String::new();
    let mut in_section = false;
    let head = Regex::new(r"^\s*#{1,6}\s+(.*)$").unwrap();
    for line in markdown.lines() {
        if let Some(cap) = head.captures(line) {
            if in_section {
                break; // volgende kop → einde sectie
            }
            if cap[1].to_lowercase().contains(&name.to_lowercase()) {
                in_section = true;
                continue;
            }
        }
        if in_section {
            out.push_str(line);
            out.push('\n');
        }
    }
    if in_section {
        Some(out)
    } else {
        None
    }
}
