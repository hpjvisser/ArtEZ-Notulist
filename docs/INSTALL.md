# Installatie & probleemoplossing

## 1. Eindgebruiker (kant-en-klare installer)

1. Voer **`ArtezNotulistSetup.exe`** uit (als administrator).
2. Laat de taak *"Afhankelijkheden automatisch installeren"* aangevinkt.
3. De installer (self-contained — geen `winget` of PATH nodig):
   - maakt `C:\ArtezNotulist\{inbox,output,logs,templates,models,whisper,bin}` aan;
   - zet **FFmpeg** en **Pandoc** als portable `.exe` in `C:\ArtezNotulist\bin`;
   - installeert **wkhtmltopdf** (PDF-engine, één self-contained programma);
   - downloadt **whisper.cpp** en het **large-v3-model** (~3 GB);
   - installeert **Ollama** en haalt het model **`mistral`** op;
   - schrijft `config.toml` met **absolute paden** naar al deze binaries.
4. Start **ArtEZ Notulist** via het menu.

> De eerste installatie kan lang duren door de modeldownload van ~3 GB.

> Mislukt er iets (bijv. de internetverbinding viel weg)? Gebruik de
> startmenu-snelkoppeling **"Onderdelen installeren of herstellen"** om het
> script opnieuw te draaien — het slaat alles over wat al binnen is.

## 2. Handmatige afhankelijkheden (fallback)

Draai eerst de startmenu-snelkoppeling **"Onderdelen installeren of herstellen"**
(of `installer\scripts\install_deps.ps1` als administrator). Lukt een onderdeel
dan nog niet, installeer het handmatig en zet het juiste pad in `config.toml`:

| Onderdeel | Download | `config.toml`-sleutel |
|-----------|----------|-----------------------|
| FFmpeg | https://www.gyan.dev/ffmpeg/builds/ (essentials zip) | `binaries.ffmpeg` |
| Pandoc | https://github.com/jgm/pandoc/releases (windows-x86_64.zip) | `binaries.pandoc` |
| wkhtmltopdf | https://wkhtmltopdf.org/downloads.html (win64 installer) | `binaries.wkhtmltopdf` |
| Ollama | https://ollama.com/download (daarna `ollama pull mistral`) | n.v.t. (HTTP 11434) |

### whisper.cpp + model

```powershell
# Binaries
# Download whisper-bin-x64.zip van:
#   https://github.com/ggerganov/whisper.cpp/releases
# en pak uit naar C:\ArtezNotulist\whisper  (whisper-cli.exe moet daar staan)

# Model large-v3
Invoke-WebRequest `
  'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin' `
  -OutFile 'C:\ArtezNotulist\models\ggml-large-v3.bin'
```

## 3. GPU-versnelling (Whisper)

`whisper-cli.exe` gebruikt automatisch de GPU als de binary met **CUDA**
(NVIDIA) of **Vulkan** is gebouwd. De generieke `whisper-bin-x64.zip` is een
CPU-build; voor GPU:

- Download/gebruik een CUDA- of Vulkan-build van whisper.cpp, óf
- Bouw zelf: `cmake -B build -DGGML_CUDA=1` (zie whisper.cpp-README).

Zet in `config.toml` onder `[whisper]` `use_gpu = true` (standaard). Met
`use_gpu = false` draait alles op de CPU.

## 4. PDF-export (wkhtmltopdf)

De PDF wordt gemaakt met **wkhtmltopdf** — één self-contained programma, geen
Python of GTK nodig. Standaard geïnstalleerd in
`C:\Program Files\wkhtmltopdf\bin\wkhtmltopdf.exe`. Faalt de PDF-stap, controleer
dan dat dit bestand bestaat en dat `binaries.wkhtmltopdf` in `config.toml` ernaar
verwijst. De DOCX-export (Pandoc) staat hier los van en werkt onafhankelijk.

## 5. Configuratie aanpassen

Alles staat in **`C:\ArtezNotulist\config.toml`** en wordt bij elke run opnieuw
gelezen:

- Ander Ollama-model: voeg toe aan `models = [...]`, draai `ollama pull <model>`.
- Andere binarypaden, threads, contextvenster, temperatuur, taal: idem.

## 6. Logbestanden

Per run wordt gelogd naar het venster én naar
`C:\ArtezNotulist\logs\notulist.log`.
