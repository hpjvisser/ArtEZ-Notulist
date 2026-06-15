; ============================================================================
;  ArtEZ Notulist — Inno Setup configuratie
;  Compileer met Inno Setup 6 (iscc.exe ArtezNotulist.iss) -> ArtezNotulistSetup.exe
;
;  Verwacht dat de Tauri-release eerst is gebouwd:
;     npm install
;     npm run tauri:build
;  zodat src-tauri\target\release\artez-notulist.exe bestaat.
; ============================================================================

#define AppName        "ArtEZ Notulist"
#define AppVersion      "1.0.0"
#define AppPublisher    "ArtEZ University of the Arts"
#define AppExeName      "artez-notulist.exe"
#define DataRoot        "C:\ArtezNotulist"

; Paden zijn relatief t.o.v. deze .iss (map: installer\). Pas zo nodig aan.
#define SourceRoot      ".."
#define ReleaseDir      SourceRoot + "\src-tauri\target\release"

[Setup]
AppId={{8F3A1C92-4E7B-4D2A-9C1E-ARTEZNOTULIST}}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\ArtEZ Notulist
DefaultGroupName=ArtEZ Notulist
DisableProgramGroupPage=yes
OutputDir={#SourceRoot}\dist-installer
OutputBaseFilename=ArtezNotulistSetup
Compression=lzma2/max
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
WizardStyle=modern
SetupLogging=yes
UninstallDisplayName={#AppName}

[Languages]
Name: "dutch";   MessagesFile: "compiler:Languages\Dutch.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "installdeps"; Description: "Afhankelijkheden automatisch installeren (FFmpeg, Ollama, Pandoc, WeasyPrint, Whisper + large-v3-model)"; GroupDescription: "Afhankelijkheden:"; Flags: checkedonce

[Files]
; --- De applicatie zelf (Tauri-release) ---
Source: "{#ReleaseDir}\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; WebView2 wordt door Tauri's NSIS normaliter meegeleverd; bij losse exe is
; Microsoft Edge WebView2 Runtime vereist (meestal al aanwezig op Windows 10/11).

; --- Templates en standaardconfiguratie naar de datamap ---
Source: "{#SourceRoot}\templates\*"; DestDir: "{#DataRoot}\templates"; Flags: onlyifdoesntexist recursesubdirs createallsubdirs
Source: "{#SourceRoot}\config\config.toml"; DestDir: "{#DataRoot}"; DestName: "config.toml"; Flags: onlyifdoesntexist

; --- Installatiescript ---
Source: "{#SourceRoot}\installer\scripts\install_deps.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion

[Dirs]
Name: "{#DataRoot}";           Permissions: users-modify
Name: "{#DataRoot}\inbox";     Permissions: users-modify
Name: "{#DataRoot}\output";    Permissions: users-modify
Name: "{#DataRoot}\logs";      Permissions: users-modify
Name: "{#DataRoot}\templates"; Permissions: users-modify
Name: "{#DataRoot}\models";    Permissions: users-modify
Name: "{#DataRoot}\whisper";   Permissions: users-modify

[Icons]
Name: "{group}\ArtEZ Notulist";      Filename: "{app}\{#AppExeName}"
Name: "{group}\Outputmap";           Filename: "{#DataRoot}\output"
Name: "{group}\{cm:UninstallProgram,ArtEZ Notulist}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\ArtEZ Notulist"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
; Afhankelijkheden installeren (kan lang duren door de modeldownload van ~3 GB).
Filename: "powershell.exe"; \
  Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\scripts\install_deps.ps1"" -Root ""{#DataRoot}"""; \
  StatusMsg: "Afhankelijkheden installeren (dit kan even duren)…"; \
  Flags: runhidden waituntilterminated; \
  Tasks: installdeps

; Applicatie starten na installatie.
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,ArtEZ Notulist}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Laat de datamap (notulen!) staan; verwijder alleen scripts.
Type: filesandordirs; Name: "{app}\scripts"
