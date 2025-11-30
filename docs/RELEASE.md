# Release-Prozess

## Quick Release (EIN Befehl!)

```bash
./quick-release.sh 1.7.5
```

**Was passiert automatisch:**
1. Version Bump (package.json, Cargo.toml, tauri.conf.json)
2. Git Commit + Tag
3. Push zu GitHub
4. Lokaler Build mit Signierung (~5-10 Min)
5. GitHub Release erstellen
6. Upload: .exe + .exe.sig + latest.json

---

## Voraussetzungen

**Lokale Dateien (in .gitignore):**
- `.github-token` - GitHub Personal Access Token
- `src-tauri/dpolg-signing.key` - Signing Key

---

## Vor dem Release

```bash
# 1. Änderungen committen
git add .
git commit -m "fix: Beschreibung"

# 2. Release
./quick-release.sh 1.7.5
```

---

## Nach dem Release testen

1. Alte App-Version öffnen
2. Update-Dialog erscheint
3. "Ja" klicken → Download + Installation
4. App neustartet

---

## NIEMALS

- GitHub Actions verwenden (headless_chrome Problem)
- Manuell Dateien hochladen
- Signing Key in Git committen
- latest.json vergessen

---

## Zeitaufwand

| Schritt | Dauer |
|---------|-------|
| Commit | 1-2 min |
| Release Script | 7-12 min |
| **Gesamt** | ~10 min |
