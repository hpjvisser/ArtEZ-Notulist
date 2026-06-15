# Installatie & probleemoplossing

## 1. Eindgebruiker (kant-en-klare installer)

1. Voer **`ArtezNotulistSetup.exe`** uit (als administrator).
2. Laat de taak *"Afhankelijkheden automatisch installeren"* aangevinkt.
3. De installer:
   - maakt `C:\ArtezNotulist\{inbox,output,logs,templates,models,whisper}` aan;
   - installeert **FFmpeg, Ollama, Pandoc, WeasyPrint** (via `winget`, met
     download-fallback);
   - downloadt **whisper.cpp** en het **large-v3-model** (~3 GB);
   - genereert een huisstijl-referentiedocument;
   - haalt het Ollama-model **`mistral`** op.
4. Start **ArtEZ Notulist** via het menu.

> De eerste installatie kan lang duren door de modeldownload van ~3 GB.

## 2. Handmatige afhankelijkheden (fallback)

Als automatische installatie faalt (bijv. geen `winget`):

```powershell
winget install Gyan.FFmpeg
winget install Ollama.Ollama
winget install JohnMacFarlane.Pandoc
winget install Python.Python.3.12
python -m pip install weasyprint
ollama pull mistral
```

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

## 4. WeasyPrint op Windows (Pango/GTK)

WeasyPrint heeft de **Pango/GTK-runtime** nodig. Als PDF-export faalt met een
melding over `libgobject`/`pango`:

1. Installeer de **GTK3-runtime** (bijv. via MSYS2: `pacman -S mingw-w64-x86_64-pango`),
   of het *gtk-for-windows* runtime-installatiepakket.
2. Zorg dat de GTK-`bin`-map in `PATH` staat.
3. Herstart de applicatie.

Alternatief: vervang in `config.toml` de PDF-stap door een andere engine
(bijv. `wkhtmltopdf`) — pas dan `pipeline/documents.rs` aan.

## 5. Configuratie aanpassen

Alles staat in **`C:\ArtezNotulist\config.toml`** en wordt bij elke run opnieuw
gelezen:

- Ander Ollama-model: voeg toe aan `models = [...]`, draai `ollama pull <model>`.
- Andere binarypaden, threads, contextvenster, temperatuur, taal: idem.

## 6. Logbestanden

Per run wordt gelogd naar het venster én naar
`C:\ArtezNotulist\logs\notulist.log`.
