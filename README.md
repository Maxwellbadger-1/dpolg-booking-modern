# Stiftung der DPolG Buchungssystem

Modernes Buchungsverwaltungssystem mit TapeChart (Timeline-Visualisierung).

## Tech-Stack

| Bereich | Technologie |
|---------|-------------|
| Frontend | React 19, TypeScript, TailwindCSS 4, Vite 7 |
| Backend | Tauri 2, Rust |
| Datenbank | PostgreSQL 16.11 (Oracle Cloud) |

## Quick Start

```bash
# 1. Repository klonen
git clone <repo-url>
cd dpolg-booking-modern

# 2. Umgebung konfigurieren
cp .env.example .env
# .env mit Datenbank-Credentials ausfüllen

# 3. Dependencies installieren
npm install

# 4. Entwicklungsserver starten
npm run tauri:dev
```

## 🖥️ Multi-Geräte-Entwicklung

Du arbeitest auf mehreren Geräten (MacBook, Windows PC)?

**Setup-Anleitungen:**
- **Windows:** [docs/WINDOWS_BUILD_GUIDE.md](docs/WINDOWS_BUILD_GUIDE.md)
- **macOS:** [docs/MACOS_SETUP.md](docs/MACOS_SETUP.md)
- **Gerätewechsel-Workflow:** [docs/DEVICE_SYNC_WORKFLOW.md](docs/DEVICE_SYNC_WORKFLOW.md)

**Wichtig:** Nur Source Code (~7 MB) wird via Git synchronisiert. Build-Artefakte (`node_modules`, `src-tauri/target`) werden lokal generiert!

**Quick Reference:**
```bash
# Vor Gerätewechsel:
git push origin main

# Nach Gerätewechsel:
git pull origin main
npm run tauri:dev
```

## Dokumentation

| Dokument | Beschreibung |
|----------|--------------|
| [CHANGELOG.md](CHANGELOG.md) | Versionshistorie |
| [docs/architecture.md](docs/architecture.md) | Systemarchitektur |
| [docs/Project_Status.md](docs/Project_Status.md) | Aktueller Projektstatus |
| [docs/Project_Spec.md](docs/Project_Spec.md) | Projektspezifikation |
| [docs/DATABASE.md](docs/DATABASE.md) | PostgreSQL-Setup |
| [docs/RELEASE.md](docs/RELEASE.md) | Release-Prozess |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Entwickler-Guide |
| [docs/DEVICE_SYNC_WORKFLOW.md](docs/DEVICE_SYNC_WORKFLOW.md) | Multi-Geräte-Workflow |
| [docs/WINDOWS_BUILD_GUIDE.md](docs/WINDOWS_BUILD_GUIDE.md) | Windows-Setup |
| [docs/MACOS_SETUP.md](docs/MACOS_SETUP.md) | macOS-Setup |

## Features

- TapeChart mit Drag & Drop
- Buchungsverwaltung (CRUD)
- Gästeverwaltung
- E-Mail-System mit Templates
- PDF-Rechnungen
- Multi-User (PostgreSQL)

## Build

```bash
npm run tauri build
```

## Lizenz

Proprietär - Stiftung der DPolG
