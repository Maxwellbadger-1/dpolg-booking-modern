# Projekt-Spezifikation

**Projekt:** Stiftung der DPolG Buchungssystem
**Version:** 1.8.4
**Stand:** 2025-02-04
**Typ:** Desktop-Anwendung (Windows, macOS)

---

## Zweck

Buchungsverwaltungssystem für das Gästehaus der Stiftung der Deutschen Polizeigewerkschaft (DPolG). Verwaltet Zimmerbuchungen, Gästedaten, Preisberechnung, Rechnungserstellung und E-Mail-Kommunikation.

---

## Benutzer

| Rolle | Aufgaben | Zugriff |
|-------|----------|---------|
| Rezeption | Buchungen verwalten, Gäste anlegen, Rechnungen erstellen, E-Mails senden | Vollzugriff |
| Verwaltung | Einstellungen, Preise, E-Mail-Konfiguration, Berichte | Vollzugriff |
| Reinigung | Mobile App für Reinigungsaufgaben | Nur Reinigungsplan |

---

## Funktionale Anforderungen

### FR-001: Buchungsverwaltung

| ID | Anforderung | Priorität | Status |
|----|-------------|-----------|--------|
| FR-001.1 | Buchungen erstellen mit Gast, Zimmer, Zeitraum | Hoch | Fertig |
| FR-001.2 | Buchungen bearbeiten und löschen | Hoch | Fertig |
| FR-001.3 | Visuelle Timeline (TapeChart) mit Drag & Drop | Hoch | Fertig |
| FR-001.4 | Buchungsstatus verwalten (5 Status) | Hoch | Fertig |
| FR-001.5 | Zahlungsstatus tracken | Hoch | Fertig |
| FR-001.6 | Begleitpersonen pro Buchung hinzufügen | Mittel | Fertig |
| FR-001.7 | Zusatzleistungen hinzufügen | Mittel | Fertig |
| FR-001.8 | Rabatte anwenden | Mittel | Fertig |
| FR-001.9 | Automatische Preisberechnung | Hoch | Fertig |
| FR-001.10 | Multi-User gleichzeitige Bearbeitung | Hoch | Fertig |

**Buchungsstatus:**
- `anfrage` - Anfrage eingegangen
- `bestaetigt` - Buchung bestätigt
- `eingecheckt` - Gast ist angereist
- `ausgecheckt` - Gast ist abgereist
- `storniert` - Buchung wurde storniert

### FR-002: Gästeverwaltung

| ID | Anforderung | Priorität | Status |
|----|-------------|-----------|--------|
| FR-002.1 | Gäste anlegen mit Kontaktdaten | Hoch | Fertig |
| FR-002.2 | DPolG-Mitgliedschaft tracken | Hoch | Fertig |
| FR-002.3 | 35+ Datenfelder (Name, Adresse, Kontakt, etc.) | Mittel | Fertig |
| FR-002.4 | Gäste suchen und filtern | Mittel | Fertig |
| FR-002.5 | Guthaben-System für Vorauszahlungen | Niedrig | Fertig |
| FR-002.6 | Wiederverwendbare Begleitpersonen | Niedrig | Fertig |

### FR-003: Zimmerverwaltung

| ID | Anforderung | Priorität | Status |
|----|-------------|-----------|--------|
| FR-003.1 | 10 Zimmer mit unterschiedlichen Kapazitäten | Hoch | Fertig |
| FR-003.2 | Preise nach Saison (Haupt-/Nebensaison) | Hoch | Fertig |
| FR-003.3 | Automatische Verfügbarkeitsprüfung | Hoch | Fertig |
| FR-003.4 | Zimmer-Details (Beschreibung, Ausstattung) | Niedrig | Fertig |

### FR-004: Preisberechnung

| ID | Anforderung | Priorität | Status |
|----|-------------|-----------|--------|
| FR-004.1 | Saisonbasierte Preise (konfigurierbar) | Hoch | Fertig |
| FR-004.2 | DPolG-Mitglieder-Rabatt automatisch | Hoch | Fertig |
| FR-004.3 | Begleitpersonen-Aufschlag | Mittel | Fertig |
| FR-004.4 | Zusatzleistungen (fix & prozentual) | Mittel | Fertig |
| FR-004.5 | Manuelle Rabatte | Mittel | Fertig |
| FR-004.6 | Guthaben-Verrechnung | Niedrig | Fertig |
| FR-004.7 | Konsistenz Sidebar/Invoice | Hoch | Fertig |

**Preiskonsistenz-Prinzip:**
- **View-Mode:** Zeigt gespeicherte Preise (historisch korrekt, wie Rechnung)
- **Edit-Mode:** Berechnet dynamisch mit aktuellen Zimmerpreisen
- **Invoice:** Verwendet immer gespeicherte Preise aus Buchung

### FR-005: Kommunikation

| ID | Anforderung | Priorität | Status |
|----|-------------|-----------|--------|
| FR-005.1 | E-Mail-Versand (Bestätigung, Rechnung, etc.) | Hoch | Fertig |
| FR-005.2 | 7 SMTP-Provider unterstützt | Mittel | Fertig |
| FR-005.3 | Template-System mit Platzhaltern | Hoch | Fertig |
| FR-005.4 | Geplante E-Mails (Scheduled) | Mittel | Fertig |
| FR-005.5 | E-Mail-Protokoll | Niedrig | Fertig |
| FR-005.6 | PDF-Rechnung als Anhang | Niedrig | STUB |

### FR-006: Dokumente

| ID | Anforderung | Priorität | Status |
|----|-------------|-----------|--------|
| FR-006.1 | PDF-Rechnungen mit Firmenlogo | Hoch | Fertig |
| FR-006.2 | Buchungsbestätigungen | Mittel | Fertig |
| FR-006.3 | Reinigungsplan PDF | Niedrig | Fertig |

### FR-007: Erinnerungen

| ID | Anforderung | Priorität | Status |
|----|-------------|-----------|--------|
| FR-007.1 | Manuelle Erinnerungen pro Buchung | Mittel | Fertig |
| FR-007.2 | Fälligkeitsdatum | Mittel | Fertig |
| FR-007.3 | Snooze-Funktion | Niedrig | Fertig |
| FR-007.4 | Automatische Erinnerungen | Niedrig | STUB |

### FR-008: Berichte

| ID | Anforderung | Priorität | Status |
|----|-------------|-----------|--------|
| FR-008.1 | Belegungsübersicht | Mittel | Fertig |
| FR-008.2 | Umsatzstatistiken | Mittel | Fertig |
| FR-008.3 | E-Mail-Historie | Niedrig | Fertig |

---

## Nicht-Funktionale Anforderungen

### NFR-001: Plattformen

| Plattform | Status | Installer |
|-----------|--------|-----------|
| Windows 10/11 | Unterstützt | .msi |
| macOS 12+ | Unterstützt | .dmg |
| Linux | Nicht unterstützt | - |

### NFR-002: Lokalisierung

| Aspekt | Anforderung | Status |
|--------|-------------|--------|
| Sprache | Komplett Deutsch | Fertig |
| Datumsformat | DD.MM.YYYY | Fertig |
| Währung | EUR (1.234,56 €) | Fertig |
| Zeitzone | Europe/Berlin | Fertig |

### NFR-003: Performance

| Aspekt | Anforderung | Status |
|--------|-------------|--------|
| App-Start | < 3 Sekunden | Fertig |
| Buchungsoperationen | < 500ms | Fertig |
| Gleichzeitige Benutzer | 100+ | Fertig |
| TapeChart Rendering | < 100ms | Fertig |
| Real-Time Sync | < 1 Sekunde | Fertig |
| Doppelbuchungs-Schutz | Transaktional | Fertig |

### NFR-004: Sicherheit

| Aspekt | Implementierung | Status |
|--------|-----------------|--------|
| DB-Verbindung | TLS/SSL verschlüsselt | Fertig |
| Credentials | Nur in .env, nie in Git | Fertig |
| SQL-Injection | Prepared Statements | Fertig |
| Input-Validierung | Rust Backend | Fertig |

### NFR-005: Wartung

| Aspekt | Implementierung | Status |
|--------|-----------------|--------|
| Auto-Update | Tauri Updater | Fertig |
| Fehler-Logging | Console + Datei | Teilweise |
| Backup | In-App SQL-Dumps | Fertig |

---

## UI/UX Spezifikation

### Farbschema

| Element | Tailwind-Klasse | Hex |
|---------|-----------------|-----|
| Background Primary | `bg-slate-900` | #0f172a |
| Background Secondary | `bg-slate-800` | #1e293b |
| Primary Action | `bg-blue-600` | #2563eb |
| Success | `bg-emerald-500` | #10b981 |
| Error | `bg-red-500` | #ef4444 |
| Warning | `bg-amber-500` | #f59e0b |
| Text Primary | `text-white` | #ffffff |
| Text Secondary | `text-slate-400` | #94a3b8 |

### Formate

| Element | Format | Beispiel |
|---------|--------|----------|
| Datum | DD.MM.YYYY | 30.01.2025 |
| Uhrzeit | HH:mm | 14:30 |
| Währung | #.###,## € | 1.234,56 € |
| Prozent | ##,# % | 15,0 % |

### z-index Hierarchie

| Element | z-index |
|---------|---------|
| Base Content | 0 |
| Sticky Headers | 10 |
| Dropdowns | 100 |
| Modals | 200 |
| Tooltips | 300 |
| Toast Notifications | 400 |

---

## Use Cases

### UC-001: Buchung erstellen

**Akteur:** Rezeption
**Vorbedingung:** Gast existiert, Zimmer verfügbar
**Ablauf:**
1. User klickt "Neue Buchung" oder zieht im TapeChart
2. System öffnet BookingSidebar
3. User wählt Gast (Autocomplete-Suche)
4. User wählt Zimmer (Dropdown)
5. User wählt Check-in und Check-out Datum
6. System berechnet Preis automatisch (Saison, DPolG)
7. Optional: User fügt Begleitpersonen hinzu
8. Optional: User fügt Zusatzleistungen hinzu
9. Optional: User fügt Rabatte hinzu
10. User klickt "Speichern"
11. System erstellt Buchung
12. Optional: System sendet Bestätigungs-E-Mail

**Nachbedingung:** Buchung in DB, TapeChart aktualisiert

### UC-002: Rechnung versenden

**Akteur:** Rezeption
**Vorbedingung:** Buchung existiert
**Ablauf:**
1. User öffnet Buchungsdetails
2. User klickt "Rechnung"
3. System generiert PDF mit allen Positionen
4. User klickt "Per E-Mail senden"
5. System öffnet E-Mail-Dialog mit Vorschau
6. User bestätigt
7. System sendet E-Mail mit Rechnung

**Nachbedingung:** E-Mail gesendet, Status "Rechnung versendet"

### UC-003: TapeChart Drag & Drop

**Akteur:** Rezeption
**Vorbedingung:** Buchung existiert
**Ablauf:**
1. User zieht Buchungsblock im TapeChart
2. System zeigt Vorschau der neuen Position
3. User lässt los
4. System prüft Verfügbarkeit
5. Wenn verfügbar: Bestätigungsdialog
6. User bestätigt
7. System aktualisiert Buchung

**Nachbedingung:** Buchung hat neues Datum/Zimmer

---

## Datenanforderungen

### Buchung (bookings)

| Feld | Typ | Pflicht | Validierung |
|------|-----|---------|-------------|
| id | BIGSERIAL | Auto | PK |
| room_id | BIGINT | Ja | FK rooms |
| guest_id | BIGINT | Ja | FK guests |
| checkin_date | DATE | Ja | >= heute (bei Neuanlage) |
| checkout_date | DATE | Ja | > checkin_date |
| status | VARCHAR | Ja | ENUM: anfrage, bestaetigt, eingecheckt, ausgecheckt, storniert |
| payment_status | VARCHAR | Ja | ENUM: offen, angezahlt, bezahlt |
| calculated_price | DECIMAL | Nein | >= 0 |
| notes | TEXT | Nein | - |

### Gast (guests)

| Feld | Typ | Pflicht | Validierung |
|------|-----|---------|-------------|
| id | BIGSERIAL | Auto | PK |
| first_name | VARCHAR | Ja | 1-100 Zeichen |
| last_name | VARCHAR | Ja | 1-100 Zeichen |
| email | VARCHAR | Nein | E-Mail-Format |
| phone | VARCHAR | Nein | - |
| is_dpolg_member | BOOLEAN | Ja | Default: false |
| dpolg_member_id | VARCHAR | Nein | - |
| address_* | VARCHAR | Nein | - |

### Zimmer (rooms)

| Feld | Typ | Pflicht | Validierung |
|------|-----|---------|-------------|
| id | BIGSERIAL | Auto | PK |
| name | VARCHAR | Ja | 1-50 Zeichen |
| capacity | INTEGER | Ja | 1-10 |
| nebensaison_preis | DECIMAL | Ja | >= 0 |
| hauptsaison_preis | DECIMAL | Ja | >= 0 |

---

## Technische Spezifikation

### Tech-Stack

| Komponente | Technologie | Version |
|------------|-------------|---------|
| Frontend | React | 19 |
| Language | TypeScript | 5.x |
| Styling | TailwindCSS | 4 |
| Build Tool | Vite | 7 |
| Backend | Tauri | 2 |
| Backend Language | Rust | 1.70+ |
| Database | PostgreSQL | 16.11 |
| Connection Pool | pgBouncer | 1.25 |
| PDF | headless_chrome | - |
| E-Mail | lettre | 0.11 |
| Icons | Lucide React | - |
| Datum | date-fns | - |

### Metriken

| Metrik | Wert |
|--------|------|
| React Components | 60 |
| Tauri Commands | 174 |
| Database Tables | 32 |
| Repositories | 20 |

### Deployment

| Aspekt | Lösung |
|--------|--------|
| Distribution | GitHub Releases |
| Auto-Update | Tauri Updater |
| Windows | .msi Installer |
| macOS | .dmg |

---

## Grenzen / Nicht enthalten

| Feature | Grund |
|---------|-------|
| Online-Buchung für Gäste | Kein Self-Service Portal geplant |
| Zahlungsabwicklung | Nur Tracking, keine Integration |
| Buchhaltungs-Integration | Export geplant, keine Echtzeit-Sync |
| Smartphone-App | Desktop + Mobile-Web für Reinigung |
| Multi-Sprache | Nur Deutsch (Zielgruppe) |

---

## Abhängigkeiten

### Externe Dienste

| Dienst | Zweck | Kritikalität |
|--------|-------|--------------|
| Oracle Cloud | PostgreSQL Hosting | Hoch |
| E-Mail-Provider | SMTP für E-Mail-Versand | Mittel |
| Vercel | Mobile App Hosting | Niedrig |
| Turso | Mobile App DB | Niedrig |

### Build-Anforderungen

| Tool | Version | Zweck |
|------|---------|-------|
| Node.js | 18+ | Frontend Build |
| Rust | 1.70+ | Backend Build |
| Tauri CLI | 2.x | App Packaging |
| Chrome/Chromium | - | PDF Generation |
