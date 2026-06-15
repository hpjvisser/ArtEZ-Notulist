# ArtEZ-huisstijl DOCX-template

Pandoc gebruikt een **referentiedocument** (`artez_huisstijl.docx`) om de DOCX-
opmaak te bepalen: lettertypen, kleuren, kopstijlen, kop-/voettekst en logo.

## Aanmaken

De installer genereert automatisch een basis-referentiedocument:

```powershell
pandoc -o artez_huisstijl.docx --print-default-data-file reference.docx
```

## Aanpassen naar de ArtEZ-huisstijl

1. Open `C:\ArtezNotulist\templates\artez_huisstijl.docx` in Word.
2. Pas de **stijlen** aan (niet de inhoud — die wordt overschreven):
   - *Title* / *Heading 1*: ArtEZ-blauw (#0A2540), schreefloos.
   - *Heading 2/3*: idem, kleiner.
   - *Normal*: huisstijl-broodtekstlettertype.
3. Voeg in de **kop-/voettekst** het ArtEZ-logo en
   "Concept — niet vastgesteld" toe.
4. Sla op als `.docx` (zelfde naam).

Het pad is instelbaar in de app (instelling *ArtEZ-huisstijl DOCX-template*) en
in `config.toml` onder `[settings] huisstijlTemplate`.
