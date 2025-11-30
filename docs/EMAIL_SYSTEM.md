# Email-System Dokumentation

## ✅ Vollständig implementiert (21.11.2025)

### Übersicht
Das Email-System ist vollständig funktionsfähig und produktionsbereit. Es unterstützt alle gängigen Email-Provider in Deutschland und bietet ein flexibles Template-System.

## Funktionen

### 1. SMTP-Verbindung testen
**Command:** `test_email_connection_command`

Testet die SMTP-Verbindung mit den konfigurierten Einstellungen.

**Rückgabe:**
- ✅ Erfolg: "Verbindung erfolgreich! SMTP-Server ist erreichbar und Anmeldedaten sind korrekt."
- ❌ Fehler: Detaillierte Fehlermeldungen (Authentication, Timeout, TLS-Fehler, etc.)

### 2. Test-Email senden
**Command:** `send_test_email_command(recipientEmail: String)`

Sendet eine Test-Email an die angegebene Adresse.

**Parameter:**
- `recipientEmail` - Empfänger-Email-Adresse

### 3. Email-Versand mit Templates

Alle Email-Commands unterstützen:
- Custom-Templates aus der Datenbank (`email_templates` Tabelle)
- Fallback auf Default-Templates
- Platzhalter-Ersetzung

#### 3.1 Rechnung senden
**Command:** `send_invoice_email_command(bookingId: i64)`

Sendet Rechnung per Email an den Gast.

**Template-Name:** `rechnung`

#### 3.2 Stornierungsbestätigung
**Command:** `send_cancellation_email_command(bookingId: i64)`

Sendet Stornierungsbestätigung.

**Template-Name:** `stornierung`

#### 3.3 Erinnerung
**Command:** `send_reminder_email_command(bookingId: i64)`

Sendet Erinnerung vor Check-in.

**Template-Name:** `erinnerung`

#### 3.4 Buchungsbestätigung
**Command:** `send_confirmation_email_command(bookingId: i64)`

Sendet Buchungsbestätigung.

**Template-Name:** `bestaetigung`

## Template-System

### Verfügbare Platzhalter

| Platzhalter | Beschreibung | Beispiel |
|-------------|--------------|----------|
| `{{gast_name}}` | Voller Name | "Max Mustermann" |
| `{{gast_vorname}}` | Vorname | "Max" |
| `{{gast_nachname}}` | Nachname | "Mustermann" |
| `{{zimmer}}` | Zimmer-Name | "Zimmer 101" |
| `{{checkin}}` | Check-in Datum | "2025-12-01" |
| `{{checkout}}` | Check-out Datum | "2025-12-05" |
| `{{naechte}}` | Anzahl Nächte | "4" |
| `{{gesamtpreis}}` | Gesamtpreis | "1.234,56 €" |
| `{{buchungs_id}}` | Buchungsnummer | "173" |
| `{{status}}` | Buchungsstatus | "bestätigt" |

### Template erstellen

Templates werden in der `email_templates` Tabelle gespeichert:

```sql
INSERT INTO email_templates (template_name, subject, body, is_active)
VALUES (
  'bestaetigung',
  'Buchungsbestätigung #{{buchungs_id}}',
  'Sehr geehrte/r {{gast_name}},\n\nvielen Dank für Ihre Buchung...',
  TRUE
);
```

## Unterstützte Email-Provider

### 1. Gmail
- **SMTP:** smtp.gmail.com
- **Port:** 465 (SSL)
- **Hinweis:** App-Passwort erforderlich

### 2. Outlook / Microsoft 365
- **SMTP:** smtp-mail.outlook.com
- **Port:** 587 (STARTTLS)

### 3. GMX
- **SMTP:** mail.gmx.net
- **Port:** 465 (SSL)

### 4. WEB.DE
- **SMTP:** smtp.web.de
- **Port:** 465 (SSL)

### 5. 1&1 IONOS
- **SMTP:** smtp.ionos.de
- **Port:** 587 (STARTTLS)

### 6. Strato
- **SMTP:** smtp.strato.de
- **Port:** 465 (SSL)

### 7. Telekom / T-Online
- **SMTP:** securesmtp.t-online.de
- **Port:** 587 (STARTTLS)

## Konfiguration

### Email-Einstellungen
Die Konfiguration erfolgt über die UI in: **Einstellungen → E-Mail**

**Erforderliche Felder:**
- SMTP-Server
- SMTP-Port
- Benutzername
- Passwort
- Absender-Email
- Absender-Name

**Optional:**
- TLS verwenden (Standard: aktiviert)

### Automatische Port-Erkennung

Das System erkennt automatisch den richtigen Verbindungsmodus:
- **Port 465:** Implicit TLS (SMTPS)
- **Port 587:** STARTTLS
- **Andere Ports:** Ohne TLS (nicht empfohlen)

## Fehlerbehandlung

### Typische Fehlermeldungen

| Fehler | Ursache | Lösung |
|--------|---------|--------|
| "Authentifizierung fehlgeschlagen" | Falscher Benutzername/Passwort | Bei Gmail: App-Passwort verwenden |
| "Verbindungszeitüberschreitung" | Server nicht erreichbar | Server-Adresse und Port prüfen |
| "TLS/SSL-Fehler" | Zertifikatsproblem | Anderen Port versuchen (465 ↔ 587) |
| "Verbindung abgelehnt" | Port blockiert/falsch | Port-Einstellung prüfen |
| "Gast hat keine Email-Adresse" | Gast-Email fehlt | Email-Adresse im Gast-Profil eintragen |

## Technische Details

### Backend (Rust)
- **Crate:** `lettre` v0.11
- **Features:** `smtp-transport`, `tokio1-native-tls`
- **Datei:** [src-tauri/src/lib_pg.rs](../src-tauri/src/lib_pg.rs)

### Helper-Funktionen
- `create_smtp_transport()` - SMTP-Transport erstellen
- `send_email_helper()` - Email versenden
- `replace_template_placeholders()` - Platzhalter ersetzen

### Frontend (TypeScript)
- **Komponente:** [EmailConfigTab.tsx](../src/components/Settings/EmailConfigTab.tsx)
- **Provider:** [emailProviders.ts](../src/lib/emailProviders.ts)

## Testing

### Test-Email senden
1. Gehe zu **Einstellungen → E-Mail**
2. Konfiguriere SMTP-Einstellungen
3. Gib Test-Empfänger-Email ein
4. Klicke auf **"Test-Email senden"**
5. Prüfe Posteingang (auch Spam-Ordner!)

### Produktions-Test
1. Erstelle eine Test-Buchung
2. Sende Buchungsbestätigung über Buchungsdetails
3. Prüfe ob Email beim Gast ankommt

## Bekannte Limitierungen

1. **Gmail App-Passwort:** Google verlangt App-spezifische Passwörter statt des normalen Passworts
2. **Anhänge:** Noch nicht implementiert (für Rechnungs-PDF geplant)
3. **HTML-Emails:** Aktuell nur Plain-Text (HTML-Support geplant)
4. **Email-Logs:** Noch nicht in UI integriert

## Nächste Schritte

- [ ] PDF-Anhänge für Rechnungen
- [ ] HTML-Email-Templates
- [ ] Email-Log-Anzeige in UI
- [ ] Geplante Emails (Reminder-System)
- [ ] Massen-Email-Versand
