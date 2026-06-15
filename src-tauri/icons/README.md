# Iconen

Tauri verwacht hier `32x32.png`, `128x128.png`, `128x128@2x.png` en `icon.ico`
(zie `tauri.conf.json`). Genereer ze in één keer uit de meegeleverde
1024×1024 PNG (`app-icon.png` in de projectroot):

```bash
npx @tauri-apps/cli icon ../../app-icon.png
```

`app-icon.svg` is het vectorbronbestand; `app-icon.png` is de kant-en-klare
rasterversie die de bovenstaande opdracht (en de CI-workflow) gebruiken.

Dit vult deze map met alle vereiste formaten (inclusief `icon.ico` voor Windows).
Zolang dit niet is gebeurd, faalt `npm run tauri:build`.
