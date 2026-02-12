# Changelog

Alle wesentlichen Änderungen werden hier dokumentiert.

Format: [Keep a Changelog](https://keepachangelog.com/de/1.1.0/)
Versionierung: [Semantic Versioning](https://semver.org/lang/de/)

---

## [Unreleased]

### Hinzugefügt
- **Real-Time Sync Modernisierung - Phase 1 + 2** - Immediate UI Updates ohne Polling-Verzögerung
  - **Phase 1 (CRITICAL)**:
    - **Reminder Badge Live Update**: Zusätzlicher `reminder-completed` Event Listener in App.tsx
      - Badge aktualisiert sich sofort (< 100ms) bei Reminder-Operationen
      - Kein Warten mehr auf PostgreSQL NOTIFY (3s Polling)
      - Pattern: Dual-Listener (db-change + reminder-completed) für maximale Responsiveness
    - **DataContext Return Types**: `updateBookingPayment`, `updateBookingStatus`, `markInvoiceSent` geben jetzt `Promise<Booking>` zurück
      - Ermöglicht sofortiges Context-Update nach Backend-Call
      - BookingSidebar nutzt Return Value für immediate local update (kein `loadBookingDetails()` mehr nötig)
      - Konsistent mit TapeChart `syncBookingFromBackend()` Pattern
    - **Reduzierte Latenz**: Payment/Status-Änderungen sichtbar in < 100ms (vorher bis zu 3s)
  - **Phase 2 (IMPORTANT)**:
    - **BookingReminders Optimistic Updates**: Entfernt redundante `loadReminders()` Calls
      - `handleCreateReminder`: Optimistic Add zu State (< 10ms)
      - `handleUpdateReminder`: Optimistic Update in State (< 10ms)
      - `handleMarkCompleted`: Optimistic Status Change (< 10ms)
      - `handleMarkUncompleted`: Optimistic Status Change (< 10ms)
      - `handleDelete`: Optimistic Remove mit Rollback (< 10ms)
      - LISTEN/NOTIFY synchronisiert andere Tabs/Instanzen automatisch
      - Performance: 5x Full Reload eliminiert → ~300ms Latenz-Reduktion pro Operation
- **Live-Update für Reminder Badge** - Echtzeit-Aktualisierung der Anzahl beim Glockensymbol (Migration 018)
  - PostgreSQL NOTIFY Triggers für `reminders` Tabelle (INSERT/UPDATE/DELETE)
  - Badge aktualisiert sich sofort bei Reminder-Änderungen (< 1 Sekunde)
  - Entfernt: 5-Minuten-Polling (Performance-Verbesserung)
  - Pattern: Konsistent mit bookings, guests, rooms (Migration 007)
  - Funktioniert für: Auto-Reminders (Backend-Trigger), manuelles Erstellen/Löschen, Multi-Tab Sync
  - Backend: `listener.rs` LISTEN reminder_changes Channel
  - Frontend: `App.tsx` Tauri Event Listener statt Browser CustomEvent + Polling

### Behoben
- **TapeChart Drag & Drop Visual Jump** - Race Condition Fix
  - **Root Cause**: Race Condition zwischen manuellem `updateBooking()` Call und PostgreSQL LISTEN/NOTIFY
  - **Fix A**: Entfernt `updateBooking()` Call (Zeile 1104) - LISTEN/NOTIFY macht Context-Sync bereits
  - **Fix B**: Overlap-Check nutzt `bookings` (Context) statt `localBookings` - autoritative Server-Daten
  - Balken springt nicht mehr visuell zurück nach Save
  - `localBookings` bleibt stabil mit Backend-Daten
  - useEffect-Barriere schützt während Pending State
  - Nach Save: Context ist durch LISTEN/NOTIFY bereits aktualisiert → sauberer Sync
  - Betroffene Komponente: `src/components/TapeChart.tsx`
- **Auto-Reminders aktualisieren sich bei Buchungsänderungen** - Migration 017
  - **Root Cause**: COUNT-basierte Duplicate Prevention verhinderte Updates
  - **Fix**: ON CONFLICT DO UPDATE Pattern (nach Email-System Migration 005 Vorbild)
  - Check-in Änderung → `due_date` aktualisiert automatisch
  - Guest Änderung → `description` aktualisiert automatisch
  - Stornierung → Alle Auto-Reminders auto-completed mit "[Buchung storniert]" Suffix
  - Buchung löschen → Reminders CASCADE deleted (ON DELETE CASCADE)
  - **Performance**: Selective Trigger (~60% weniger Ausführungen)
  - Beispiel: Check-in 2026-03-01 → 2026-04-01 | Payment Reminder due_date 2026-02-22 → 2026-03-25 ✅
- **Preisaufschlüsselung bei Drag & Drop** - Critical Bug-Fix für veraltete Rabattbeträge
  - `discounts.calculated_amount` wird jetzt bei Neuberechnung aktualisiert
  - Behobene Inkonsistenz: Anzeige zeigte alte Rabattbeträge, Gesamtpreis basierte auf neuen
  - Backend: `recalculate_and_save_booking_prices` updated jetzt auch `discounts` Tabelle
  - Automatischer DPolG-Rabatt wird persistent in DB gespeichert/aktualisiert
  - Migration Script: `fix_calculated_amounts.py` für Reparatur von Bestandsdaten
  - Beispiel-Bug: Nach Drag & Drop zeigte Anzeige "-23.25 €" aber Gesamtpreis basierte auf "-15.75 €"

### Hinzugefügt
- **Automatische Preisberechnung bei Buchungsänderungen** - Backend-Triggered Price Recalculation
  - Preise werden automatisch neu berechnet, wenn Daten/Zimmer/Gast geändert werden
  - Backend-Funktion `recalculate_and_save_booking_prices` für konsistente Preisspeicherung
  - Automatische Integration in `update_booking_pg` und `update_booking_dates_and_room_pg`
  - Verhindert Inkonsistenzen zwischen View-Mode, Rechnungen und tatsächlichen Preisen
  - **Per-Night Hauptsaison-Berechnung** - Identisch zur Live-Berechnung bei Erstellung
  - **Endreinigung-Konsistenz** - Berücksichtigt Zimmer-Endreinigung automatisch
  - Berücksichtigt DPolG-Mitgliederrabatte, Saisonpreise, Services und Discounts
  - Frontend nutzt optimistic updates mit Backend-Response (keine extra Reloads)
- **Multi-User System (Vollständig)** - Enterprise-Grade Kollaborations-Features
  - **Advisory Locks System** - PostgreSQL-basierte Sperren für Booking-Bearbeitung
    - Automatisches Lock-Erwerb beim Öffnen von Buchungen
    - Heartbeat-System (30s) mit Auto-Unlock nach 5 Min Inaktivität
    - PostgreSQL NOTIFY für Echtzeit-Lock-Updates
  - **Presence System** - Sichtbarkeit wer gerade was bearbeitet
    - `LockBadge` Component zeigt "🔒 In Bearbeitung von User X"
    - `useLockManager` Hook für automatisches Lock-Management
    - Lock-Anzeige in Booking-Dialogen und Listen
  - **Conflict Resolution UI** - Visueller Konflikt-Dialog
    - Side-by-Side Diff-Viewer (Deine Änderungen vs. Aktuelle Daten)
    - 3 Auflösungs-Optionen: Überschreiben, Verwerfen, Field-by-Field Merge
    - Highlighting von geänderten Feldern
  - **Audit Log System** - Vollständige Change-History
    - Automatisches Logging aller Änderungen (INSERT/UPDATE/DELETE)
    - Triggers auf Bookings, Guests, Rooms
    - `AuditLogViewer` Timeline-Component mit JSON-Diff
    - Backend Commands: `get_audit_log_pg`, `get_booking_audit_log_pg`
  - **User Context** - Multi-User Identifikation
    - Persistent User-Namen via LocalStorage
    - Integration in Audit Trail (created_by, updated_by)
- **Audit Trail Anzeige** - Erstellungs- und Änderungshistorie in BookingSidebar
  - Zeigt Ersteller und letzten Bearbeiter mit Zeitstempel
  - Nur im View-Mode sichtbar
  - Dark-Mode-optimierte Farbgebung
- **Multi-Device Workflow** - Git-basierter Entwicklungs-Workflow
  - DEVICE_SYNC_WORKFLOW.md - Komplette Anleitung für Windows ↔ macOS Wechsel
  - MACOS_SETUP.md - Vollständige macOS Setup-Dokumentation
  - 90% schnellerer Device-Switch (1-2 Min statt 10+ Min)
- **Dev-Scripts** - Automatisierte Entwicklungs-Workflows
  - `dev-start.ps1` - Startet App im Dev-Modus (Port-Check, Auto-Kill)
  - `dev-stop.ps1` - Stoppt laufende Dev-Prozesse
  - `cleanup.ps1` - Optionale Bereinigung (~5-6 GB Einsparungen)
- **Doppelbuchungs-Schutz** - Transaktionale Verfügbarkeitsprüfung mit Row-Level Locking
  - `SELECT ... FOR UPDATE` verhindert Race Conditions
  - SERIALIZABLE Isolation Level für atomare Operationen
  - User-freundliche Fehlermeldung bei Konflikt
- **PostgreSQL LISTEN/NOTIFY aktiviert** - Echtzeit-Updates statt 3-Sekunden-Polling
  - Sofortige Synchronisation (< 1 Sekunde)
  - Automatischer Reconnect bei Verbindungsabbruch
  - Polling bleibt als Fallback aktiv
  - Neuer Channel: `lock_changes` für Advisory Locks

### Behoben
- **E-Mail-Verlauf Datums-Anzeige** - "Ungültiges Datum" komplett behoben
  - **Root Cause**: PostgreSQL gibt Timestamps mit ungültigem Timezone-Format zurück (`+00` statt `+00:00` oder `Z`)
  - JavaScript Date-Parser kann `2026-02-12T04:47:23.736642+00` nicht parsen (Invalid Date)
  - **Fix**: `+00` wird automatisch zu `Z` konvertiert (da +00 = UTC)
  - TypeScript Interface korrigiert: `sent_at: string | null` (Type Mismatch mit Rust `Option<String>` behoben)
  - Backend filtert nun Email-Logs ohne Sendedatum konsistent (`WHERE sent_at IS NOT NULL`)
  - Frontend `formatDateTime()` robuster gegen Whitespace, NULL-Werte und PostgreSQL Default-Dates ("0001-01-01")
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
- **Audit Trail Farben** - Light Mode Darstellung korrigiert in BookingSidebar
  - Text-Farben angepasst (weiß auf weiß → dunkel auf hell)
  - Border-Farben für hellen Hintergrund optimiert
- **Dokumentation** - Feature-Status korrigiert

### Entfernt
- **BookingDetails.tsx** - 1.281 Zeilen ungenutzter Legacy-Code entfernt
  - Funktionalität vollständig in BookingSidebar konsolidiert
  - Reduziert Wartungsaufwand und Verwirrung
- **Alte Dokumentation** - ~13.000 Zeilen veralteter Dokumentation gelöscht
  - ARCHITECTURE_ISSUES.md, ARCHITECTURE_PRICING.md
  - DEVTOOLS_COMPREHENSIVE_DESIGN.md
  - MULTI_USER_MIGRATION.md, MULTI_USER_MIGRATION_ENHANCED.md
  - POSTGRESQL_ARCHITECTURE.md, PREVENTION_SYSTEM.md
  - UPDATE_SYSTEM.md, Z_INDEX_STRUCTURE.md
  - REALTIME_MULTIUSER_IMPLEMENTATION.md
  - WINDOWS_HANDOVER.md, handover.md
  - Verschiedene alte Build- und Migration-Scripts
- **Legacy Backend-Module** - Ungenutzte Rust-Module entfernt
  - database_views.rs, db_pool.rs (zugunsten von database_pg)
  - payment_recipients.rs, pdf_generator.rs (alte Implementierungen)
  - putzplan_debug.rs, reminder_automation.rs, reminders.rs
  - supabase.rs, time_utils.rs, transaction_log.rs

### Technisch
- **Migration 011** - `active_locks` Tabelle für Presence System
  - Trigger: `notify_lock_change()` für PostgreSQL NOTIFY
  - Funktion: `cleanup_stale_locks()` für Auto-Cleanup
  - Indexes: booking_id, user_name, last_heartbeat
- **Migration 012** - Audit-Triggers auf existierende `audit_log` Tabelle
  - Trigger: `log_booking_changes()`, `log_guest_changes()`, `log_room_changes()`
  - Automatisches JSONB-Logging aller Änderungen
- **Backend Commands** - 8 neue Commands
  - Lock-System: `acquire_booking_lock_pg`, `release_booking_lock_pg`, `update_lock_heartbeat_pg`, `get_all_active_locks_pg`, `get_booking_lock_pg`, `release_all_user_locks_pg`
  - Audit-System: `get_audit_log_pg`, `get_booking_audit_log_pg`
- **Frontend Components** - 3 neue Components
  - `LockBadge` - Presence-Anzeige
  - `ConflictResolutionDialog` - Konflikt-Auflösung
  - `AuditLogViewer` - Change-History Timeline
- **Hooks** - Neuer Custom Hook `useLockManager` für Lock-Lifecycle
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
