<#
.SYNOPSIS
    Self-contained bootstrap voor ArtEZ Notulist.

.DESCRIPTION
    Installeert alle afhankelijkheden als PORTABLE binaries onder
    C:\ArtezNotulist en schrijft config.toml met ABSOLUTE paden, zodat de app
    niet afhankelijk is van PATH of winget.

    Onderdelen: FFmpeg, Pandoc, wkhtmltopdf (PDF), whisper.cpp + large-v3-model,
    Ollama (+ standaardmodel mistral).

    Idempotent: alles wat al aanwezig is, wordt overgeslagen. Veilig om opnieuw
    te draaien ("Onderdelen herstellen").
#>

[CmdletBinding()]
param(
    [string]$Root = 'C:\ArtezNotulist'
)

$ErrorActionPreference = 'Continue'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

function Write-Step($m) { Write-Host "==> $m" -ForegroundColor Cyan }
function Write-Ok($m)   { Write-Host "    OK: $m" -ForegroundColor Green }
function Write-Warn2($m){ Write-Host "    LET OP: $m" -ForegroundColor Yellow }
function Write-Err2($m) { Write-Host "    FOUT: $m" -ForegroundColor Red }

function Get-File($url, $out) {
    for ($i = 1; $i -le 3; $i++) {
        try {
            Invoke-WebRequest $url -OutFile $out -UseBasicParsing -Headers @{ 'User-Agent' = 'ArtezNotulist' }
            return $true
        } catch {
            Write-Warn2 "Download poging $i mislukt ($url): $($_.Exception.Message)"
            Start-Sleep -Seconds 4
        }
    }
    return $false
}

function Get-LatestAsset($repo, $pattern) {
    try {
        $rel = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" `
            -Headers @{ 'User-Agent' = 'ArtezNotulist' }
        return ($rel.assets | Where-Object { $_.name -match $pattern } | Select-Object -First 1).browser_download_url
    } catch {
        return $null
    }
}

# ---------------------------------------------------------------------------
# 0. Mapstructuur
# ---------------------------------------------------------------------------
Write-Step "Mapstructuur aanmaken onder $Root"
foreach ($d in 'inbox','output','logs','templates','models','whisper','bin','tmp') {
    $p = Join-Path $Root $d
    if (-not (Test-Path $p)) { New-Item -ItemType Directory -Path $p -Force | Out-Null }
}
$tmp = Join-Path $Root 'tmp'
Write-Ok "Mappen gereed."

# ---------------------------------------------------------------------------
# 1. FFmpeg (portable)
# ---------------------------------------------------------------------------
$ffmpeg = Join-Path $Root 'bin\ffmpeg.exe'
if (Test-Path $ffmpeg) {
    Write-Ok "FFmpeg al aanwezig."
} else {
    Write-Step "FFmpeg downloaden…"
    $zip = Join-Path $tmp 'ffmpeg.zip'
    if (Get-File 'https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip' $zip) {
        $ex = Join-Path $tmp 'ffmpeg'
        Remove-Item $ex -Recurse -Force -ErrorAction SilentlyContinue
        Expand-Archive $zip -DestinationPath $ex -Force
        $src = Get-ChildItem $ex -Recurse -Filter 'ffmpeg.exe' | Select-Object -First 1
        if ($src) { Copy-Item $src.FullName $ffmpeg -Force; Write-Ok "FFmpeg geïnstalleerd." }
        else { Write-Err2 "ffmpeg.exe niet gevonden in download." }
    } else {
        Write-Err2 "FFmpeg-download mislukt."
    }
}

# ---------------------------------------------------------------------------
# 2. Pandoc (portable)
# ---------------------------------------------------------------------------
$pandoc = Join-Path $Root 'bin\pandoc.exe'
if (Test-Path $pandoc) {
    Write-Ok "Pandoc al aanwezig."
} else {
    Write-Step "Pandoc downloaden…"
    $url = Get-LatestAsset 'jgm/pandoc' 'windows-x86_64\.zip$'
    if (-not $url) { $url = 'https://github.com/jgm/pandoc/releases/download/3.1.11/pandoc-3.1.11-windows-x86_64.zip' }
    $zip = Join-Path $tmp 'pandoc.zip'
    if (Get-File $url $zip) {
        $ex = Join-Path $tmp 'pandoc'
        Remove-Item $ex -Recurse -Force -ErrorAction SilentlyContinue
        Expand-Archive $zip -DestinationPath $ex -Force
        $src = Get-ChildItem $ex -Recurse -Filter 'pandoc.exe' | Select-Object -First 1
        if ($src) { Copy-Item $src.FullName $pandoc -Force; Write-Ok "Pandoc geïnstalleerd." }
        else { Write-Err2 "pandoc.exe niet gevonden in download." }
    } else {
        Write-Err2 "Pandoc-download mislukt."
    }
}

# ---------------------------------------------------------------------------
# 3. wkhtmltopdf (PDF-engine, silent installer)
# ---------------------------------------------------------------------------
$wkCandidates = @(
    'C:\Program Files\wkhtmltopdf\bin\wkhtmltopdf.exe',
    (Join-Path $Root 'bin\wkhtmltopdf.exe')
)
$wkhtml = $wkCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($wkhtml) {
    Write-Ok "wkhtmltopdf al aanwezig."
} else {
    Write-Step "wkhtmltopdf installeren…"
    $exe = Join-Path $tmp 'wkhtmltox.exe'
    $url = 'https://github.com/wkhtmltopdf/packaging/releases/download/0.12.6-1/wkhtmltox-0.12.6-1.msvc2015-win64.exe'
    if (Get-File $url $exe) {
        Start-Process -FilePath $exe -ArgumentList '/S' -Wait
        $wkhtml = 'C:\Program Files\wkhtmltopdf\bin\wkhtmltopdf.exe'
        if (Test-Path $wkhtml) { Write-Ok "wkhtmltopdf geïnstalleerd." }
        else { Write-Err2 "wkhtmltopdf-installatie niet gevonden na setup." }
    } else {
        Write-Err2 "wkhtmltopdf-download mislukt."
    }
}
if (-not $wkhtml) { $wkhtml = 'C:\Program Files\wkhtmltopdf\bin\wkhtmltopdf.exe' }

# ---------------------------------------------------------------------------
# 4. whisper.cpp (portable) + large-v3-model
# ---------------------------------------------------------------------------
$whisperDir = Join-Path $Root 'whisper'
$whisperCli = Get-ChildItem $whisperDir -Recurse -Include 'whisper-cli.exe','main.exe' -ErrorAction SilentlyContinue |
              Select-Object -First 1
if ($whisperCli) {
    $whisperCli = $whisperCli.FullName
    Write-Ok "whisper.cpp al aanwezig ($whisperCli)."
} else {
    Write-Step "whisper.cpp downloaden…"
    $url = Get-LatestAsset 'ggml-org/whisper.cpp' '^whisper-bin-x64\.zip$'
    if (-not $url) { $url = Get-LatestAsset 'ggerganov/whisper.cpp' '^whisper-bin-x64\.zip$' }
    if (-not $url) { $url = 'https://github.com/ggml-org/whisper.cpp/releases/latest/download/whisper-bin-x64.zip' }
    $zip = Join-Path $tmp 'whisper.zip'
    if (Get-File $url $zip) {
        Expand-Archive $zip -DestinationPath $whisperDir -Force
        $cli = Get-ChildItem $whisperDir -Recurse -Include 'whisper-cli.exe','main.exe' | Select-Object -First 1
        if ($cli) { $whisperCli = $cli.FullName; Write-Ok "whisper.cpp geïnstalleerd ($whisperCli)." }
        else { Write-Err2 "whisper-cli.exe/main.exe niet gevonden in download." }
    } else {
        Write-Err2 "whisper.cpp-download mislukt. Download handmatig naar $whisperDir."
    }
}
if (-not $whisperCli) { $whisperCli = Join-Path $Root 'whisper\whisper-cli.exe' }

$model = Join-Path $Root 'models\ggml-large-v3.bin'
if ((Test-Path $model) -and ((Get-Item $model).Length -gt 1GB)) {
    Write-Ok "Model large-v3 al aanwezig."
} else {
    Write-Step "large-v3-model downloaden (~3 GB, dit duurt even)…"
    $url = 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin'
    if (Get-File $url $model) { Write-Ok "Model gedownload." }
    else { Write-Err2 "Modeldownload mislukt. Download ggml-large-v3.bin handmatig naar $Root\models." }
}

# ---------------------------------------------------------------------------
# 5. Ollama (installer) + standaardmodel
# ---------------------------------------------------------------------------
$ollama = Get-Command ollama -ErrorAction SilentlyContinue
if (-not $ollama) {
    $ollama = Get-Item (Join-Path $env:LOCALAPPDATA 'Programs\Ollama\ollama.exe') -ErrorAction SilentlyContinue
}
if ($ollama) {
    Write-Ok "Ollama al aanwezig."
} else {
    Write-Step "Ollama installeren…"
    $exe = Join-Path $tmp 'OllamaSetup.exe'
    if (Get-File 'https://ollama.com/download/OllamaSetup.exe' $exe) {
        Start-Process -FilePath $exe -ArgumentList '/SILENT' -Wait
        $ollama = Get-Item (Join-Path $env:LOCALAPPDATA 'Programs\Ollama\ollama.exe') -ErrorAction SilentlyContinue
        if ($ollama) { Write-Ok "Ollama geïnstalleerd." } else { Write-Warn2 "Ollama-installatie niet teruggevonden." }
    } else {
        Write-Err2 "Ollama-download mislukt."
    }
}
$ollamaExe = if ($ollama) { $ollama.Source } else { $null }
if (-not $ollamaExe -and $ollama) { $ollamaExe = $ollama.FullName }
if ($ollamaExe) {
    Write-Step "Standaardmodel 'mistral' ophalen (kan even duren)…"
    try {
        Start-Process -FilePath $ollamaExe -ArgumentList 'pull','mistral' -Wait -NoNewWindow
        Write-Ok "Model 'mistral' beschikbaar."
    } catch {
        Write-Warn2 "Kon 'mistral' nog niet ophalen; voer later 'ollama pull mistral' uit."
    }
}

# ---------------------------------------------------------------------------
# 6. Huisstijl-referentiedocument (Pandoc-default als basis)
# ---------------------------------------------------------------------------
$ref = Join-Path $Root 'templates\artez_huisstijl.docx'
if ((-not (Test-Path $ref)) -and (Test-Path $pandoc)) {
    Write-Step "Huisstijl-referentiedocument genereren…"
    try {
        & $pandoc -o $ref --print-default-data-file reference.docx
        if (Test-Path $ref) { Write-Ok "Referentiedocument aangemaakt." }
    } catch { Write-Warn2 "Kon geen referentiedocument genereren." }
}

# ---------------------------------------------------------------------------
# 7. config.toml met ABSOLUTE paden schrijven
# ---------------------------------------------------------------------------
Write-Step "config.toml schrijven met absolute paden…"
$cfg = Join-Path $Root 'config.toml'
$configContent = @"
# ArtEZ Notulist — automatisch gegenereerd door de installer.
# Absolute paden zodat de app niet van PATH afhankelijk is.
# Je mag dit bestand met de hand aanpassen; het wordt bij elke run gelezen.

[paths]
root      = '$Root'
inbox     = '$Root\inbox'
output    = '$Root\output'
logs      = '$Root\logs'
templates = '$Root\templates'
models    = '$Root\models'

[binaries]
ffmpeg      = '$ffmpeg'
whisper     = '$whisperCli'
pandoc      = '$pandoc'
wkhtmltopdf = '$wkhtml'

[whisper]
model_file = '$model'
threads    = 8
use_gpu    = true

[ollama]
host          = 'http://127.0.0.1:11434'
default_model = 'mistral'
models        = ['mistral', 'llama3.1', 'qwen2.5', 'gemma2']
temperature   = 0.2
num_ctx       = 8192

[settings]
anonimiseren            = false
sprekerherkenning       = false
actiepuntenExtractie    = true
vertrouwelijkheidslabel = 'Intern'
huisstijlTemplate       = '$ref'
model                   = 'mistral'
taal                    = 'nl'
"@
Set-Content -Path $cfg -Value $configContent -Encoding UTF8
Write-Ok "config.toml geschreven."

# ---------------------------------------------------------------------------
# 8. Opruimen + eindrapport
# ---------------------------------------------------------------------------
Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "================ ARTEZ NOTULIST — STATUS ================" -ForegroundColor Cyan
"FFmpeg       : " + (Test-Path $ffmpeg)
"Pandoc       : " + (Test-Path $pandoc)
"wkhtmltopdf  : " + (Test-Path $wkhtml)
"whisper-cli  : " + (Test-Path $whisperCli)
"model large-v3: " + ((Test-Path $model) -and ((Get-Item $model -ErrorAction SilentlyContinue).Length -gt 1GB))
"Ollama       : " + ([bool]$ollamaExe)
Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "Klaar. Start ArtEZ Notulist (opnieuw) om de wijzigingen te laden." -ForegroundColor Green
