<#
.SYNOPSIS
    Bootstrapt alle afhankelijkheden voor ArtEZ Notulist.

.DESCRIPTION
    Wordt door de Inno Setup-installer aangeroepen na het kopiëren van de app.
    Installeert (indien afwezig): FFmpeg, Ollama, Pandoc, WeasyPrint.
    Downloadt whisper.cpp en het large-v3-model. Maakt de mapstructuur aan,
    genereert het huisstijl-referentiedocument en haalt het standaard
    Ollama-model op.

    Idempotent: alles dat al aanwezig is, wordt overgeslagen.
#>

[CmdletBinding()]
param(
    [string]$Root = 'C:\ArtezNotulist'
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Write-Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "    $msg" -ForegroundColor Green }
function Write-Warn2($msg) { Write-Host "    LET OP: $msg" -ForegroundColor Yellow }

function Test-Command($name) {
    return [bool](Get-Command $name -ErrorAction SilentlyContinue)
}

function Invoke-Winget($id, $name) {
    if (-not (Test-Command 'winget')) {
        Write-Warn2 "winget niet beschikbaar; sla automatische installatie van $name over."
        return $false
    }
    Write-Step "Installeer $name via winget ($id)…"
    winget install --id $id --silent --accept-source-agreements --accept-package-agreements -e | Out-Null
    return $true
}

# ---------------------------------------------------------------------------
# 1. Mapstructuur
# ---------------------------------------------------------------------------
Write-Step "Mapstructuur aanmaken onder $Root"
foreach ($d in 'inbox','output','logs','templates','models','whisper','bin') {
    $p = Join-Path $Root $d
    if (-not (Test-Path $p)) { New-Item -ItemType Directory -Path $p -Force | Out-Null }
}
Write-Ok "Mappen gereed."

# ---------------------------------------------------------------------------
# 2. FFmpeg
# ---------------------------------------------------------------------------
if (Test-Command 'ffmpeg') {
    Write-Ok "FFmpeg al aanwezig."
} else {
    if (-not (Invoke-Winget 'Gyan.FFmpeg' 'FFmpeg')) {
        Write-Step "FFmpeg handmatig downloaden…"
        $zip = Join-Path $env:TEMP 'ffmpeg.zip'
        Invoke-WebRequest 'https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip' -OutFile $zip
        $dest = Join-Path $Root 'bin\ffmpeg'
        Expand-Archive -Path $zip -DestinationPath $dest -Force
        $exe = Get-ChildItem -Path $dest -Recurse -Filter 'ffmpeg.exe' | Select-Object -First 1
        if ($exe) {
            Copy-Item $exe.FullName (Join-Path $Root 'bin\ffmpeg.exe') -Force
            Write-Ok "FFmpeg geïnstalleerd in $Root\bin."
        }
    }
}

# ---------------------------------------------------------------------------
# 3. Ollama
# ---------------------------------------------------------------------------
if (Test-Command 'ollama') {
    Write-Ok "Ollama al aanwezig."
} else {
    if (-not (Invoke-Winget 'Ollama.Ollama' 'Ollama')) {
        Write-Step "Ollama-installer downloaden…"
        $exe = Join-Path $env:TEMP 'OllamaSetup.exe'
        Invoke-WebRequest 'https://ollama.com/download/OllamaSetup.exe' -OutFile $exe
        Start-Process -FilePath $exe -ArgumentList '/SILENT' -Wait
        Write-Ok "Ollama geïnstalleerd."
    }
}

# ---------------------------------------------------------------------------
# 4. Pandoc
# ---------------------------------------------------------------------------
if (Test-Command 'pandoc') {
    Write-Ok "Pandoc al aanwezig."
} else {
    Invoke-Winget 'JohnMacFarlane.Pandoc' 'Pandoc' | Out-Null
}

# ---------------------------------------------------------------------------
# 5. Python + WeasyPrint
# ---------------------------------------------------------------------------
if (-not (Test-Command 'python')) {
    Invoke-Winget 'Python.Python.3.12' 'Python 3.12' | Out-Null
    $env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' +
                [System.Environment]::GetEnvironmentVariable('Path','User')
}
if (Test-Command 'weasyprint') {
    Write-Ok "WeasyPrint al aanwezig."
} elseif (Test-Command 'python') {
    Write-Step "WeasyPrint via pip installeren…"
    python -m pip install --upgrade pip | Out-Null
    python -m pip install weasyprint | Out-Null
    Write-Ok "WeasyPrint geïnstalleerd."
    Write-Warn2 "WeasyPrint vereist de Pango/GTK-runtime. Installeer die zo nodig via MSYS2 of gtk-for-windows; zie docs\INSTALL.md."
} else {
    Write-Warn2 "Python ontbreekt; WeasyPrint kon niet worden geïnstalleerd."
}

# ---------------------------------------------------------------------------
# 6. whisper.cpp (Windows-binaries)
# ---------------------------------------------------------------------------
$whisperExe = Join-Path $Root 'whisper\whisper-cli.exe'
if (Test-Path $whisperExe) {
    Write-Ok "whisper.cpp al aanwezig."
} else {
    Write-Step "whisper.cpp downloaden…"
    $zip = Join-Path $env:TEMP 'whisper-bin.zip'
    # Voorgecompileerde Windows-binaries van whisper.cpp (x64).
    $url = 'https://github.com/ggerganov/whisper.cpp/releases/latest/download/whisper-bin-x64.zip'
    try {
        Invoke-WebRequest $url -OutFile $zip
        Expand-Archive -Path $zip -DestinationPath (Join-Path $Root 'whisper') -Force
        # Sommige releases noemen de binary 'main.exe' of 'whisper-cli.exe'.
        $cli = Get-ChildItem -Path (Join-Path $Root 'whisper') -Recurse -Include 'whisper-cli.exe','main.exe' |
               Select-Object -First 1
        if ($cli -and $cli.Name -ne 'whisper-cli.exe') {
            Copy-Item $cli.FullName $whisperExe -Force
        }
        Write-Ok "whisper.cpp geïnstalleerd."
    } catch {
        Write-Warn2 "Automatische whisper.cpp-download mislukt: $($_.Exception.Message)"
        Write-Warn2 "Download handmatig van https://github.com/ggerganov/whisper.cpp/releases naar $Root\whisper."
    }
}

# ---------------------------------------------------------------------------
# 7. large-v3-model
# ---------------------------------------------------------------------------
$model = Join-Path $Root 'models\ggml-large-v3.bin'
if (Test-Path $model) {
    Write-Ok "Model large-v3 al aanwezig."
} else {
    Write-Step "large-v3-model downloaden (~3 GB)…"
    $url = 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin'
    try {
        Invoke-WebRequest $url -OutFile $model
        Write-Ok "Model gedownload."
    } catch {
        Write-Warn2 "Modeldownload mislukt: $($_.Exception.Message)"
        Write-Warn2 "Download ggml-large-v3.bin handmatig naar $Root\models."
    }
}

# ---------------------------------------------------------------------------
# 8. Huisstijl-referentiedocument (Pandoc-default als basis)
# ---------------------------------------------------------------------------
$ref = Join-Path $Root 'templates\artez_huisstijl.docx'
if ((Test-Path $ref) -eq $false -and (Test-Command 'pandoc')) {
    Write-Step "Huisstijl-referentiedocument genereren…"
    try {
        pandoc -o $ref --print-default-data-file reference.docx 2>$null
        if (-not (Test-Path $ref)) {
            # Oudere pandoc: schrijf via stdout-redirect.
            cmd /c "pandoc --print-default-data-file reference.docx > `"$ref`""
        }
        Write-Ok "Referentiedocument aangemaakt (vervang dit door de ArtEZ-huisstijl)."
    } catch {
        Write-Warn2 "Kon geen referentiedocument genereren."
    }
}

# ---------------------------------------------------------------------------
# 9. Standaard Ollama-model ophalen
# ---------------------------------------------------------------------------
if (Test-Command 'ollama') {
    Write-Step "Standaardmodel 'mistral' ophalen via Ollama…"
    try {
        Start-Process -FilePath 'ollama' -ArgumentList 'pull','mistral' -Wait -NoNewWindow
        Write-Ok "Model 'mistral' beschikbaar."
    } catch {
        Write-Warn2 "Kon 'mistral' niet ophalen; voer later 'ollama pull mistral' uit."
    }
}

Write-Step "Klaar. ArtEZ Notulist is gereed voor gebruik."
