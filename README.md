# ArtEZ Notulist

Een **volledig lokale** Windows-desktopapplicatie die audio- en video-opnames van
vergaderingen omzet in conceptnotulen volgens de redactionele stijl van het
**College van Bestuur (CvB) van ArtEZ**. Er gaat geen data naar de cloud: alle
transcriptie en tekstgeneratie draait op de eigen machine.

## Architectuur

| Laag | Technologie |
|------|-------------|
| Desktop-framework | **Tauri 2** |
| Frontend | **React + TypeScript** (Vite) |
| Backend | **Rust** |
| Transcriptie | **whisper.cpp**, model **large-v3**, GPU-versnelling indien beschikbaar |
| Taalmodel | **Ollama** (standaard `mistral`, instelbaar via `config.toml`) |
| Audio | **FFmpeg** |
| Documenten | **Pandoc** (DOCX) + **wkhtmltopdf** (PDF) |
| Installer | **Inno Setup** → `ArtezNotulistSetup.exe` |

## Workflow

1. Gebruiker kiest een audio-/videobestand (`mp3, wav, m4a, mp4, webm, mov`).
2. **FFmpeg** extraheert 16 kHz mono-audio.
3. **Whisper large-v3** maakt het transcript (`transcript.txt`).
4. **Ollama** genereert de conceptnotulen volgens het ArtEZ-stijltemplate.
5. **Pandoc** maakt `concept_notulen.docx` (met huisstijl-referentiedocument).
6. **wkhtmltopdf** maakt `concept_notulen.pdf`.
7. Actiepunten worden geëxtraheerd naar `actielijst.csv`.

### Outputbestanden (`C:\ArtezNotulist\output\<opname>_<datum>\`)

- `transcript.txt`
- `samenvatting.md`
- `concept_notulen.docx`
- `concept_notulen.pdf`
- `actielijst.csv`

## Mapstructuur

```
C:\ArtezNotulist
├── inbox
├── output
├── logs
├── templates      (notulen_prompt.md, pdf_style.css, artez_huisstijl.docx)
├── models         (ggml-large-v3.bin)
├── whisper        (whisper-cli.exe)
└── config.toml
```

## Projectstructuur

```
artez-notulist/
├── src/                       React + TypeScript frontend
│   ├── App.tsx                Hoofdscherm (knoppen, voortgang, log)
│   ├── api.ts                 Tauri-commando's & events
│   └── components/            ProgressBar, LogPanel, SettingsPanel
├── src-tauri/                 Rust-backend
│   ├── src/
│   │   ├── commands.rs        Tauri-commando's
│   │   ├── config.rs          config.toml laden/bewaren
│   │   ├── logging.rs         Voortgang + logevents
│   │   └── pipeline/          ffmpeg · whisper · ollama · documents · actions
│   ├── tauri.conf.json
│   └── Cargo.toml
├── templates/
│   ├── notulen_prompt.md      ArtEZ-stijlregels (promptengineering)
│   └── pdf_style.css          Huisstijl voor de PDF
├── config/config.toml         Standaardconfiguratie
├── installer/
│   ├── ArtezNotulist.iss      Inno Setup → ArtezNotulistSetup.exe
│   └── scripts/install_deps.ps1
└── docs/INSTALL.md
```

## Bouwen (op een Windows-ontwikkelmachine)

Vereist: Node 18+, Rust (stable, MSVC-toolchain), de Tauri-systeemafhankelijkheden
en (voor de installer) Inno Setup 6.

```powershell
# 1. Frontend-dependencies
npm install

# 2. Iconen genereren (eenmalig) uit de meegeleverde app-icon.png
npx @tauri-apps/cli icon .\app-icon.png

# 3. Ontwikkelen
npm run tauri:dev

# 4. Release bouwen (maakt src-tauri\target\release\artez-notulist.exe)
npm run tauri:build

# 5. Installer compileren
iscc installer\ArtezNotulist.iss
#    -> dist-installer\ArtezNotulistSetup.exe
```

De installer (`ArtezNotulistSetup.exe`) installeert daarna automatisch FFmpeg,
Ollama, Pandoc, wkhtmltopdf, whisper.cpp en het large-v3-model, en maakt de
mapstructuur aan. Zie [docs/INSTALL.md](docs/INSTALL.md) voor details en
handmatige fallbacks.

## Bouwen via GitHub Actions (zonder lokale Windows-machine)

`.github/workflows/build-windows.yml` bouwt alles op een `windows-latest`-runner:
het genereert de iconen, bouwt de Tauri-release, compileert de Inno
Setup-installer en uploadt **`artez-notulist.exe`** en
**`ArtezNotulistSetup.exe`** als build-artifacts.

```bash
# Eenmalig: maak er een git-repo van en push naar GitHub.
cd artez-notulist
git init && git add . && git commit -m "ArtEZ Notulist"
git branch -M main
git remote add origin <jouw-repo-url>
git push -u origin main
```

De build start automatisch bij elke push naar `main` en kan handmatig worden
gestart via *Actions → Build Windows → Run workflow*. Bij een versietag
(`git tag v1.0.0 && git push --tags`) wordt bovendien een GitHub Release met de
installer gepubliceerd. Download de installer daarna onder het tabblad
**Actions → (run) → Artifacts**.

## Kwaliteitsinstellingen (in de app)

- **Namen anonimiseren** — vervangt vermoedelijke namen door `[PERSOON N]`.
- **Sprekerherkenning** — diarisatie-hint voor whisper.cpp.
- **Automatische actiepuntenextractie** — vult `actielijst.csv`.
- **Vertrouwelijkheidslabel** — Openbaar / Intern / Vertrouwelijk / Strikt vertrouwelijk.
- **ArtEZ-huisstijl DOCX-template** — referentiedocument voor Pandoc.
- **Ollama-model** — keuze uit de modellen in `config.toml`.

## Stijlregels van de notulen

Vastgelegd in [`templates/notulen_prompt.md`](templates/notulen_prompt.md):
exacte agendanummering, kopblok (Datum/Tijd/Locatie/Aanwezig/Afwezig/Gasten),
besluiten als aparte sectie, acties inline als `(actie Achternaam)`, "het CvB"
voor gezamenlijke besluiten, geen verzonnen feiten, onzekerheden als
`[?: beschrijving]`, neutrale weergave van gevoelige onderwerpen, en
"Geen bijzonderheden" voor punten zonder bespreking.

## Status / let op

- De broncode is volledig; de Windows-**build** en de **native afhankelijkheden**
  (Whisper, Ollama) draaien op een Windows-machine. Dit project is geschreven op
  macOS en is daar niet gecompileerd.
- De installer is **self-contained**: alle binaries komen als portable bestanden
  onder `C:\ArtezNotulist` met absolute paden in `config.toml` (geen PATH/winget
  nodig). PDF gaat via **wkhtmltopdf** (één .exe), niet via WeasyPrint.
