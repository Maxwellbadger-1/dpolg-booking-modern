# Changelog

Alle wesentlichen Änderungen werden hier dokumentiert.

Format: [Keep a Changelog](https://keepachangelog.com/de/1.1.0/)
Versionierung: [Semantic Versioning](https://semver.org/lang/de/)

---

## [Unreleased]

### Hinzugefügt
- **Doppelbuchungs-Schutz** - Transaktionale Verfügbarkeitsprüfung mit Row-Level Locking
  - `SELECT ... FOR UPDATE` verhindert Race Conditions
  - SERIALIZABLE Isolation Level für atomare Operationen
  - User-freundliche Fehlermeldung bei Konflikt
- **PostgreSQL LISTEN/NOTIFY aktiviert** - Echtzeit-Updates statt 3-Sekunden-Polling
  - Sofortige Synchronisation (< 1 Sekunde)
  - Automatischer Reconnect bei Verbindungsabbruch
  - Polling bleibt als Fallback aktiv

### Behoben
- **E-Mail-Verlauf (Email History)** - Manuell versendete E-Mails erscheinen jetzt im Email-Protokoll
  - Bug 1: SQL-Fehler in `EmailLogRepository::create()` - ungültige `sent_at::text as sent_at` Syntax behoben
  - Bug 2: Manuelle Email-Commands (Bestätigung, Stornierung, Rechnung, Erinnerung) erstellten keine Log-Einträge
  - Bug 3: Status-Strings vereinheitlicht - `"sent"/"failed"` → `"gesendet"/"fehler"` für Filter-Kompatibilität
  - Alle 4 manuellen Email-Commands loggen jetzt erfolgreich/fehlgeschlagen mit korrekten deutschen Status-Strings
- **Preissystem komplett überarbeitet** - 5 kritische Bugs behoben:
  - Bug 1: Batch-Preise ignorierten DPolG-Rabatt
  - Bug 2: Invoice-Berechnung inkonsistent mit `rabatt_basis`
  - Bug 3: f32 → f64 Datentyp-Migration (Präzisionsverlust behoben)
  - Bug 6: Service `applies_to: total_price` Two-Pass Berechnung
  - Bug 7: Validierung für negative Werte und >100% Rabatte
- **Preiskonsistenz Sidebar/Invoice** - View-Mode zeigt jetzt gespeicherte Preise (wie Rechnung)
  - Verhindert Inkonsistenzen wenn Zimmerpreise nach Buchung geändert wurden
  - Edit-Mode berechnet weiterhin dynamisch mit aktuellen Preisen
- **Invoice Endreinigung** - Doppelzählung behoben wenn Endreinigung bereits als Service existiert
- **Statistiken** - Echte Auslastungsquote berechnet (vorher hardcoded 30%)
- **Dokumentation** - Feature-Status korrigiert

### Technisch
- Datenbank-Migration: `additional_services.service_price`, `additional_services.original_value`, `discounts.discount_value` von `real` auf `double precision`

### Geplant
- PDF-Rechnungen als E-Mail-Anhang

---

## [1.8.2] - 2025-01-29

### Geändert
- Version bump für Release

---

## [1.8.1] - 2025-01-28

### Geändert
- Automatische Tag-Builds in CI/CD deaktiviert
- Lokale Builds werden bevorzugt

---

## [1.8.0] - 2025-01-28

### Hinzugefügt
- **Gäste-Guthaben-System** - Vorauszahlungen verwalten und bei Buchungen verrechnen
- **Geplante E-Mails (Scheduled Emails)** - E-Mails zeitversetzt versenden
- **DevTools für Entwickler** - Integration Test Suite, DB-Statistiken
- Cleanup-Utilities für Datenbereinigung

### Behoben
- Doppelte Reinigungsaufgaben in PostgreSQL-Queries eliminiert
- Mobile App Sync-Probleme mit Turso behoben

---

## [1.7.0] - 2025-11

### Hinzugefügt
- **Multi-User Real-Time Updates** - Polling-basierte Synchronisation
- **PostgreSQL LISTEN/NOTIFY** - Real-time Benachrichtigungsmodul (vorbereitet)
- **Optimistic Locking** - Konfliktauflösung bei gleichzeitiger Bearbeitung
- Frontend-Integration für Multi-User Support
- useBookingSync Hook für automatische Updates

### Behoben
- INSERT OR REPLACE für Turso (keine doppelten Tasks)
- Race Conditions bei gleichzeitigen Buchungsänderungen

---

## [1.6.0] - 2025-11

### Hinzugefügt
- **PostgreSQL Migration komplett** - Von SQLite zu Cloud-Datenbank
- **Repository Pattern Architektur** - 20 Repositories für saubere Trennung
- pgBouncer Connection Pooling (Port 6432)
- deadpool-postgres für Connection Management

### Geändert
- Alle 90+ Frontend invoke() Calls auf _pg Commands migriert
- calculated_price Migration auf DB-Ebene
- priceInfo durch priceBreakdown Struktur ersetzt

### Entfernt
- 1150+ Zeilen doppelten Code eliminiert
- Alte SQLite-spezifische Queries

### Behoben
- Spaltennamen in Pricing Query (nebensaison_preis, hauptsaison_preis)
- setPriceInfo Referenzen entfernt
- date-fns Imports wiederhergestellt

---

## [1.5.0] - 2025-10

### Hinzugefügt
- **Begleitpersonen (Companions)** - Wiederverwendbare Begleitpersonen-Pool
- CompanionSelector Komponente
- guest_companions Tabelle

### Geändert
- Quick Wins implementiert: Date Formatting, Dialog Hook
- Duplicated Sync Logic refactored und vereinheitlicht

### Dokumentation
- Umfassende Architektur-Analyse (7 kritische Issues identifiziert)

---

## [1.4.0] - 2025-10

### Hinzugefügt
- **Service Templates** - Wiederverwendbare Zusatzleistungen
- **Discount Templates** - Wiederverwendbare Rabatt-Vorlagen
- TemplatesManagement Komponente
- Emoji-Auswahl für Templates

### Geändert
- Zusatzleistungen können jetzt als Templates gespeichert werden
- Rabatte können jetzt als Templates gespeichert werden

---

## [1.3.0] - 2025-10

### Hinzugefügt
- **E-Mail-Template-System** - Platzhalter für dynamische Inhalte
- **7 SMTP-Provider** - Gmail, Outlook, Yahoo, Postfix, Sendmail, DomainFactory, NetEasy
- EmailConfigTab mit Provider-Auswahl
- Test-E-Mail Funktionalität

### Geändert
- E-Mail-Konfiguration in eigene Tabelle ausgelagert
- Verbesserte Fehlerbehandlung bei SMTP-Fehlern

---

## [1.2.0] - 2025-09

### Hinzugefügt
- **Erinnerungen (Reminders)** - Manuelle Erinnerungen pro Buchung
- **Snooze-Funktion** - Erinnerungen verschieben
- RemindersView Komponente
- BookingReminders Sidebar-Integration
- Urgent Reminder Badge im Header

---

## [1.1.0] - 2025-09

### Hinzugefügt
- **Preisberechnung** - Automatische Berechnung mit Saison und DPolG-Rabatt
- **Zusatzleistungen** - Fixe und prozentuale Services
- **Rabatte** - Manuelle Rabatte pro Buchung
- usePriceCalculation Hook
- Preis-Breakdown Anzeige in Sidebar

### Geändert
- TapeChart zeigt jetzt Preise an
- Buchungsdetails mit vollständiger Preisaufschlüsselung

---

## [1.0.0] - 2025-09

### Hinzugefügt
- **Buchungsverwaltung**
  - CRUD-Operationen für Buchungen
  - TapeChart Timeline mit Drag & Drop
  - Statusverwaltung (5 Status)
  - Zahlungsstatus-Tracking
- **Gästeverwaltung**
  - CRUD für Gäste
  - 35+ Datenfelder
  - DPolG-Mitgliedschaft
  - Suchfunktion
- **Zimmerverwaltung**
  - 10 Zimmer mit Kapazitäten
  - Saisonpreise (Haupt-/Nebensaison)
  - Verfügbarkeitsprüfung
- **E-Mail-System**
  - SMTP-Versand
  - Bestätigungs-, Rechnungs-, Stornierungs-E-Mails
  - E-Mail-Protokoll
- **PDF-Rechnungen**
  - Generierung mit Firmenlogo
  - Buchungsbestätigungen
- **Einstellungen**
  - Firmendaten, Logo-Upload
  - Zahlungseinstellungen (IBAN, Fristen)
  - Preiseinstellungen (Saisonen)
- **Desktop-App**
  - Tauri 2 Framework
  - Windows (.msi) und macOS (.dmg)
  - Auto-Update Funktion

### Technisch
- React 18, TypeScript, TailwindCSS
- Rust Backend mit Tauri 2
- SQLite Datenbank (lokal)
- Vite Build Tool

---

## Legende

- **Hinzugefügt** - Neue Features
- **Geändert** - Änderungen an bestehenden Features
- **Behoben** - Bug Fixes
- **Entfernt** - Entfernte Features
- **Sicherheit** - Sicherheits-Updates
- **Dokumentation** - Nur Doku-Änderungen
- **Technisch** - Technische Details ohne User-Impact
