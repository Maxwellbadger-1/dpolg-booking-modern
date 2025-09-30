# Migration Guide: Python BookingSystem → Modern Tauri React App

## Zweck dieses Dokuments
Diese Anleitung hilft dabei, alle Funktionen aus dem bestehenden Python-Buchungssystem in das neue moderne Tauri + React Projekt zu übertragen.

## Aktueller Stand des neuen Projekts

### ✅ Bereits implementiert:
- **Tape Chart Visualisierung** mit Drag & Drop (dnd-kit)
- **Monatsnavigation** mit Kalenderpicker
- **Grundlegende Datenbank-Struktur**:
  - Tabellen: `rooms`, `guests`, `bookings`
  - Rust Backend mit Tauri 2
  - SQLite Datenbank
- **Status-Farbcodierung** für Buchungen
- **Moderne UI** mit TailwindCSS 4

### 📋 Noch zu implementieren:
Alle weiteren Funktionen aus dem Python-System

---

## Analyse-Checkliste für das Python-Projekt

### 1. Datenbank-Schema vollständig erfassen

**Was zu suchen:**
- Datei: `BookingSystem/database/database.py`
- Funktion: `setup_database()`

**Informationen extrahieren:**
```python
# Für jede CREATE TABLE Anweisung notieren:
# - Tabellenname
# - Alle Spalten mit Datentypen
# - Primary Keys, Foreign Keys
# - Default Values
# - UNIQUE Constraints
```

**Beispiel-Output:**
```
Tabelle: additional_services
- id: INTEGER PRIMARY KEY AUTOINCREMENT
- booking_id: INTEGER FOREIGN KEY → bookings(id)
- service_name: TEXT NOT NULL
- service_price: REAL NOT NULL
- created_at: TEXT DEFAULT CURRENT_TIMESTAMP

Tabelle: accompanying_guests
- id: INTEGER PRIMARY KEY
- booking_id: INTEGER FOREIGN KEY
- ...
```

---

### 2. GUI-Fenster und Funktionen erfassen

**Was zu suchen:**
- Ordner: `BookingSystem/gui/`
- Alle `*_window.py` Dateien

**Für jedes Fenster dokumentieren:**

#### A) Fenstername und Zweck
```
Datei: add_booking_window.py
Zweck: Neue Buchung erstellen
```

#### B) Formularfelder
```
Felder:
- Guest Dropdown (Auswahl bestehender Gast oder "Neuer Gast")
- Check-in Datum (DatePicker)
- Check-out Datum (DatePicker)
- Zimmer Auswahl (Dropdown)
- Anzahl Gäste (NumberInput)
- Status (Dropdown: reserviert, bestätigt, eingecheckt, ausgecheckt, storniert)
- Bemerkungen (Textarea)
- Begleitpersonen (Liste mit Add/Remove)
- Zusätzliche Services (Liste mit Add/Remove)
```

#### C) Validierungslogik
```
Validierungen:
- Check-out muss nach Check-in sein
- Zimmer darf nicht doppelt gebucht sein für diesen Zeitraum
- Anzahl Gäste darf Zimmerkapazität nicht überschreiten
- Email-Format prüfen
- Pflichtfelder: Gast, Check-in, Check-out, Zimmer
```

#### D) Berechnungen
```
Berechnungen:
- Anzahl Nächte = Differenz zwischen Check-in und Check-out
- Grundpreis = Nächte × Zimmerpreis (Mitglied/Nicht-Mitglied)
- Services-Summe = Summe aller zusätzlichen Services
- Rabatte anwenden (wenn vorhanden)
- Gesamtpreis = Grundpreis + Services - Rabatte
```

---

### 3. Business Logic Handlers erfassen

**Was zu suchen:**
- Ordner: `BookingSystem/logic/`
- Dateien: `booking_handler.py`, `email_handler.py`, `config_handler.py`

**Für jeden Handler dokumentieren:**

#### booking_handler.py
```
Funktionen:
- validate_booking_dates(checkin, checkout, room_id, booking_id=None)
  → Prüft Verfügbarkeit und Datum-Logik

- calculate_booking_price(room_id, checkin, checkout, guest_id, services, discounts)
  → Berechnet Gesamtpreis

- check_room_availability(room_id, start_date, end_date, exclude_booking_id=None)
  → Gibt True/False zurück
```

#### email_handler.py
```
Funktionen:
- send_confirmation_email(booking_id)
  → Bestätigungsmail mit Buchungsdetails

- send_reminder_email(booking_id)
  → Erinnerungsmail vor Check-in

- get_email_template(template_type)
  → Lädt Template aus config.json

Platzhalter in Templates:
- {gast_vorname}
- {gast_nachname}
- {buchung_reservierungsnummer}
- {zimmer_name}
- {checkin_date}
- {checkout_date}
- {gesamtpreis}
```

#### config_handler.py
```
Funktionen:
- load_email_config()
  → Lädt SMTP-Einstellungen

- save_email_config(smtp_server, port, username, password, ...)
  → Speichert verschlüsselte Konfiguration

- encrypt_password(password)
- decrypt_password(encrypted_password)
```

---

### 4. Reports und Statistiken

**Was zu suchen:**
- Dateien mit "report", "statistics", "export" im Namen

**Dokumentieren:**
```
Reports:
1. Belegungsstatistik
   - Zeitraum wählbar
   - Auslastung pro Zimmer
   - Auslastung pro Monat
   - Export als PDF/Excel

2. Umsatzübersicht
   - Gesamtumsatz pro Zeitraum
   - Aufschlüsselung nach Zimmern
   - Services-Umsatz
   - Export-Funktion

3. Gäste-Liste
   - Alle Gäste mit Kontaktdaten
   - Anzahl Buchungen pro Gast
   - Filter: Mitglieder/Nicht-Mitglieder
   - Export als CSV/Excel
```

---

### 5. Such- und Filterfunktionen

**Was zu suchen:**
- Search-Felder in GUI-Dateien
- Filter-Dropdowns

**Dokumentieren:**
```
Suchfunktionen:
- Buchungsübersicht:
  - Suche nach Reservierungsnummer
  - Suche nach Gastname
  - Filter nach Status
  - Filter nach Zeitraum
  - Filter nach Zimmer

- Gästeverwaltung:
  - Suche nach Name
  - Suche nach Email
  - Filter nach Mitgliedschaft
```

---

### 6. Datenvalidierung und Constraints

**Was zu suchen:**
- Alle `if`-Bedingungen die Fehler werfen
- `messagebox.showerror()` Aufrufe

**Dokumentieren:**
```
Validierungen:
1. Email-Format: muss @ und . enthalten
2. Telefon: nur Zahlen, +, -, Leerzeichen, Klammern
3. PLZ: nur Zahlen, 4-5 Stellen
4. Datum: Check-out > Check-in
5. Zimmer: nicht doppelt buchen
6. Gäste: Anzahl ≤ Kapazität
7. Preis: muss > 0 sein
8. Reservierungsnummer: eindeutig
```

---

### 7. PDF-Generierung

**Was zu suchen:**
- Import von `reportlab`, `fpdf` oder ähnlichen Libraries

**Dokumentieren:**
```
PDF-Dokumente:
1. Buchungsbestätigung
   - Logo/Header
   - Gastdaten
   - Zimmerdaten
   - Check-in/Check-out
   - Preisaufschlüsselung
   - Zahlungsinformationen
   - Footer mit Kontaktdaten

2. Rechnung
   - Rechnungsnummer
   - Alle Details aus Buchungsbestätigung
   - Mehrwertsteuer-Berechnung
   - Zahlungskonditionen
```

---

### 8. Benutzerrechte und Rollen

**Was zu suchen:**
- Login-Fenster
- User-Tabelle in Datenbank
- Permissions-Checks

**Dokumentieren:**
```
Rollen (falls vorhanden):
- Admin: Alle Rechte
- Rezeption: Buchungen verwalten, Gäste verwalten
- Manager: Zusätzlich Reports, Konfiguration
- Readonly: Nur ansehen

Rechte:
- buchung_erstellen
- buchung_bearbeiten
- buchung_loeschen
- buchung_stornieren
- gaeste_verwalten
- zimmer_verwalten
- konfiguration_aendern
- reports_erstellen
```

---

## Template für die Übergabe an Claude Code

Kopiere dieses Template und fülle es mit den gefundenen Informationen:

```markdown
# Funktionsübersicht Python BookingSystem

## 1. Datenbank-Schema

### Tabelle: [Name]
\`\`\`sql
CREATE TABLE [name] (
  [spalte] [typ] [constraints],
  ...
)
\`\`\`

[Wiederhole für jede Tabelle]

## 2. GUI-Fenster

### [Fenstername]
**Zweck:** [Beschreibung]

**Felder:**
- [Feld1]: [Typ] - [Beschreibung]
- [Feld2]: [Typ] - [Beschreibung]

**Validierungen:**
- [Regel 1]
- [Regel 2]

**Berechnungen:**
- [Formel 1]
- [Formel 2]

[Wiederhole für jedes Fenster]

## 3. Business Logic

### [Handler-Name]
**Funktionen:**
- `[function_name](params)`: [Beschreibung, was sie tut]

[Wiederhole für jeden Handler]

## 4. Email-System

**Templates:**
- Bestätigungsmail: [Template-Text mit Platzhaltern]
- Erinnerungsmail: [Template-Text]

**SMTP-Konfiguration:**
- Server: [Wert]
- Port: [Wert]
- Verschlüsselung: [TLS/SSL]

## 5. Reports

### [Report-Name]
- Zweck: [Beschreibung]
- Datenquellen: [Welche Tabellen]
- Export-Formate: [PDF/Excel/CSV]
- Filter-Optionen: [Liste]

## 6. Spezielle Features

[Beschreibe alle besonderen Features wie z.B.:]
- Automatische Erinnerungsmails X Tage vor Check-in
- Rabatt-System für Stammgäste
- Saisonale Preisanpassungen
- Zimmerkategorien
- Blacklist für Gäste
- etc.
```

---

## Nächste Schritte

1. **Python-Projekt analysieren** mit dieser Checkliste
2. **Template ausfüllen** mit allen gefundenen Informationen
3. **Template an Claude Code übergeben** mit der Anweisung:
   ```
   "Implementiere alle Funktionen aus diesem Python-System
   in das moderne Tauri + React Projekt. Behalte die moderne
   UI bei und nutze React-Komponenten statt CustomTkinter."
   ```

---

## Prioritäten für die Implementierung

**Phase 1 (Kritisch):**
- Vollständige Buchungsverwaltung (CRUD)
- Gästeverwaltung (CRUD)
- Zimmerverwaltung (CRUD)
- Datenbankschema vervollständigen

**Phase 2 (Wichtig):**
- Email-System
- PDF-Generierung
- Such- und Filterfunktionen

**Phase 3 (Nice-to-have):**
- Reports und Statistiken
- Export-Funktionen
- Benutzerrechte-System

---

## Technologie-Mapping

| Python | React/Tauri Equivalent |
|--------|------------------------|
| CustomTkinter Window | React Component + Modal |
| CTkEntry | `<input>` mit Tailwind |
| CTkComboBox | `<select>` oder React-Select |
| CTkTextbox | `<textarea>` |
| Tkcalendar DateEntry | react-datepicker |
| messagebox.showinfo | Toast Notification |
| SQLite (Python) | SQLite (Rust/Tauri) |
| smtplib | Rust email crate |
| reportlab (PDF) | react-pdf oder Rust PDF lib |

---

## Kontakt / Fragen

Bei Unklarheiten während der Migration diese Punkte klären:
- Welche Datenbank-Felder sind wirklich Pflichtfelder?
- Welche Berechnungsformeln sind exakt anzuwenden?
- Welche Validierungen sind geschäftskritisch?
- Welche Features werden am häufigsten genutzt?