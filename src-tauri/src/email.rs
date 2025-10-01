use rusqlite::{Connection, Result};
use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use crate::database::{EmailConfig, EmailTemplate, EmailLog, get_db_path};
use base64::{Engine as _, engine::general_purpose};

// ============================================================================
// EMAIL CONFIGURATION FUNCTIONS
// ============================================================================

/// Verschlüsselt ein Passwort (Base64 - für Production sollte AES-256 verwendet werden)
pub fn encrypt_password(password: &str) -> String {
    general_purpose::STANDARD.encode(password.as_bytes())
}

/// Entschlüsselt ein Passwort
pub fn decrypt_password(encrypted: &str) -> Result<String, String> {
    general_purpose::STANDARD
        .decode(encrypted.as_bytes())
        .map_err(|e| format!("Fehler beim Entschlüsseln: {}", e))
        .and_then(|bytes| {
            String::from_utf8(bytes)
                .map_err(|e| format!("Fehler beim Konvertieren: {}", e))
        })
}

/// Speichert oder aktualisiert die Email-Konfiguration
pub fn save_email_config(
    smtp_server: String,
    smtp_port: i32,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
    use_tls: bool,
) -> Result<EmailConfig, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("Fehler beim Aktivieren von Foreign Keys: {}", e))?;

    // Verschlüssele Passwort
    let encrypted_password = encrypt_password(&smtp_password);

    // Prüfe ob bereits eine Konfiguration existiert
    let existing_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM email_config",
        [],
        |row| row.get(0)
    ).map_err(|e| format!("Fehler beim Prüfen der Konfiguration: {}", e))?;

    let use_tls_int = if use_tls { 1 } else { 0 };

    if existing_count > 0 {
        // Update existierende Konfiguration
        conn.execute(
            "UPDATE email_config SET
                smtp_server = ?1,
                smtp_port = ?2,
                smtp_username = ?3,
                smtp_password = ?4,
                from_email = ?5,
                from_name = ?6,
                use_tls = ?7,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = 1",
            [
                &smtp_server,
                &smtp_port.to_string(),
                &smtp_username,
                &encrypted_password,
                &from_email,
                &from_name,
                &use_tls_int.to_string(),
            ],
        ).map_err(|e| format!("Fehler beim Aktualisieren der Konfiguration: {}", e))?;
    } else {
        // Neue Konfiguration erstellen
        conn.execute(
            "INSERT INTO email_config (smtp_server, smtp_port, smtp_username, smtp_password, from_email, from_name, use_tls)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            [
                &smtp_server,
                &smtp_port.to_string(),
                &smtp_username,
                &encrypted_password,
                &from_email,
                &from_name,
                &use_tls_int.to_string(),
            ],
        ).map_err(|e| format!("Fehler beim Erstellen der Konfiguration: {}", e))?;
    }

    // Lade und return die aktualisierte Konfiguration
    get_email_config()
}

/// Lädt die aktuelle Email-Konfiguration
pub fn get_email_config() -> Result<EmailConfig, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, smtp_server, smtp_port, smtp_username, smtp_password,
                from_email, from_name, use_tls, created_at, updated_at
         FROM email_config
         LIMIT 1"
    ).map_err(|e| format!("Fehler beim Vorbereiten der Abfrage: {}", e))?;

    let config = stmt.query_row([], |row| {
        Ok(EmailConfig {
            id: row.get(0)?,
            smtp_server: row.get(1)?,
            smtp_port: row.get(2)?,
            smtp_username: row.get(3)?,
            smtp_password: row.get(4)?,
            from_email: row.get(5)?,
            from_name: row.get(6)?,
            use_tls: row.get::<_, i32>(7)? != 0,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }).map_err(|e| format!("Keine Email-Konfiguration gefunden: {}", e))?;

    Ok(config)
}

/// Test-Email senden um Konfiguration zu testen
pub async fn test_email_connection(
    smtp_server: String,
    smtp_port: i32,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
    test_recipient: String,
) -> Result<String, String> {
    // Erstelle Test-Email
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse().map_err(|e| format!("Ungültige Absender-Adresse: {}", e))?)
        .to(test_recipient.parse().map_err(|e| format!("Ungültige Empfänger-Adresse: {}", e))?)
        .subject("Test Email - DPolG Stiftung Buchungssystem")
        .header(ContentType::TEXT_PLAIN)
        .body("Dies ist eine Test-Email. Ihre SMTP-Konfiguration funktioniert!".to_string())
        .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?;

    // Erstelle SMTP Transport
    let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());

    let mailer = SmtpTransport::relay(&smtp_server)
        .map_err(|e| format!("SMTP-Server nicht erreichbar: {}", e))?
        .port(smtp_port as u16)
        .credentials(creds)
        .build();

    // Sende Email
    mailer.send(&email)
        .map_err(|e| format!("Fehler beim Senden der Test-Email: {}", e))?;

    Ok("Test-Email erfolgreich gesendet!".to_string())
}

// ============================================================================
// EMAIL TEMPLATE FUNCTIONS
// ============================================================================

/// Alle Email-Templates abrufen
pub fn get_all_templates() -> Result<Vec<EmailTemplate>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, template_name, subject, body, description, created_at, updated_at
         FROM email_templates
         ORDER BY template_name"
    ).map_err(|e| format!("Fehler beim Vorbereiten der Abfrage: {}", e))?;

    let templates = stmt.query_map([], |row| {
        Ok(EmailTemplate {
            id: row.get(0)?,
            template_name: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            description: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| format!("Fehler beim Abrufen der Templates: {}", e))?;

    let mut result = Vec::new();
    for template in templates {
        result.push(template.map_err(|e| format!("Fehler beim Verarbeiten: {}", e))?);
    }

    Ok(result)
}

/// Ein spezifisches Template abrufen
pub fn get_template_by_name(template_name: &str) -> Result<EmailTemplate, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, template_name, subject, body, description, created_at, updated_at
         FROM email_templates
         WHERE template_name = ?1"
    ).map_err(|e| format!("Fehler beim Vorbereiten der Abfrage: {}", e))?;

    let template = stmt.query_row([template_name], |row| {
        Ok(EmailTemplate {
            id: row.get(0)?,
            template_name: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            description: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| format!("Template '{}' nicht gefunden: {}", template_name, e))?;

    Ok(template)
}

/// Template aktualisieren
pub fn update_template(
    id: i64,
    subject: String,
    body: String,
    description: Option<String>,
) -> Result<EmailTemplate, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    conn.execute(
        "UPDATE email_templates
         SET subject = ?1, body = ?2, description = ?3, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?4",
        [&subject, &body, &description.unwrap_or_default(), &id.to_string()],
    ).map_err(|e| format!("Fehler beim Aktualisieren des Templates: {}", e))?;

    // Lade aktualisiertes Template
    let mut stmt = conn.prepare(
        "SELECT id, template_name, subject, body, description, created_at, updated_at
         FROM email_templates
         WHERE id = ?1"
    ).map_err(|e| format!("Fehler beim Abrufen: {}", e))?;

    let template = stmt.query_row([id], |row| {
        Ok(EmailTemplate {
            id: row.get(0)?,
            template_name: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            description: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| format!("Template nicht gefunden: {}", e))?;

    Ok(template)
}

// ============================================================================
// EMAIL SENDING FUNCTIONS
// ============================================================================

/// Platzhalter in Template ersetzen
fn replace_placeholders(text: &str, placeholders: &std::collections::HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in placeholders {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

/// Buchungsbestätigungs-Email senden
pub async fn send_confirmation_email(booking_id: i64) -> Result<String, String> {
    use crate::database::{get_booking_by_id, get_guest_by_id, get_room_by_id};

    // Lade Daten
    let booking = get_booking_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;
    let guest = get_guest_by_id(booking.guest_id)
        .map_err(|e| format!("Fehler beim Laden des Gastes: {}", e))?;
    let room = get_room_by_id(booking.room_id)
        .map_err(|e| format!("Fehler beim Laden des Zimmers: {}", e))?;
    let config = get_email_config()?;
    let template = get_template_by_name("confirmation")?;

    // Erstelle Platzhalter
    let mut placeholders = std::collections::HashMap::new();
    placeholders.insert("gast_vorname".to_string(), guest.vorname.clone());
    placeholders.insert("gast_nachname".to_string(), guest.nachname.clone());
    placeholders.insert("reservierungsnummer".to_string(), booking.reservierungsnummer.clone());
    placeholders.insert("zimmer_name".to_string(), room.name.clone());
    placeholders.insert("checkin_date".to_string(), booking.checkin_date.clone());
    placeholders.insert("checkout_date".to_string(), booking.checkout_date.clone());
    placeholders.insert("anzahl_gaeste".to_string(), booking.anzahl_gaeste.to_string());
    placeholders.insert("anzahl_naechte".to_string(), booking.anzahl_naechte.to_string());
    placeholders.insert("gesamtpreis".to_string(), format!("{:.2}", booking.gesamtpreis));

    // Ersetze Platzhalter
    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    // Sende Email
    send_email_internal(&config, &guest.email, &subject, &body, booking_id, guest.id, "confirmation").await
}

/// Reminder-Email senden
pub async fn send_reminder_email(booking_id: i64) -> Result<String, String> {
    use crate::database::{get_booking_by_id, get_guest_by_id, get_room_by_id};

    let booking = get_booking_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;
    let guest = get_guest_by_id(booking.guest_id)
        .map_err(|e| format!("Fehler beim Laden des Gastes: {}", e))?;
    let room = get_room_by_id(booking.room_id)
        .map_err(|e| format!("Fehler beim Laden des Zimmers: {}", e))?;
    let config = get_email_config()?;
    let template = get_template_by_name("reminder")?;

    let mut placeholders = std::collections::HashMap::new();
    placeholders.insert("gast_vorname".to_string(), guest.vorname.clone());
    placeholders.insert("gast_nachname".to_string(), guest.nachname.clone());
    placeholders.insert("reservierungsnummer".to_string(), booking.reservierungsnummer.clone());
    placeholders.insert("zimmer_name".to_string(), room.name.clone());
    placeholders.insert("checkin_date".to_string(), booking.checkin_date.clone());
    placeholders.insert("checkout_date".to_string(), booking.checkout_date.clone());

    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_internal(&config, &guest.email, &subject, &body, booking_id, guest.id, "reminder").await
}

/// Rechnungs-Email senden
pub async fn send_invoice_email(booking_id: i64) -> Result<String, String> {
    use crate::database::{get_booking_by_id, get_guest_by_id, get_room_by_id};

    let booking = get_booking_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;
    let guest = get_guest_by_id(booking.guest_id)
        .map_err(|e| format!("Fehler beim Laden des Gastes: {}", e))?;
    let room = get_room_by_id(booking.room_id)
        .map_err(|e| format!("Fehler beim Laden des Zimmers: {}", e))?;
    let config = get_email_config()?;
    let template = get_template_by_name("invoice")?;

    let mut placeholders = std::collections::HashMap::new();
    placeholders.insert("gast_vorname".to_string(), guest.vorname.clone());
    placeholders.insert("gast_nachname".to_string(), guest.nachname.clone());
    placeholders.insert("reservierungsnummer".to_string(), booking.reservierungsnummer.clone());
    placeholders.insert("zimmer_name".to_string(), room.name.clone());
    placeholders.insert("checkin_date".to_string(), booking.checkin_date.clone());
    placeholders.insert("checkout_date".to_string(), booking.checkout_date.clone());
    placeholders.insert("anzahl_naechte".to_string(), booking.anzahl_naechte.to_string());
    placeholders.insert("grundpreis".to_string(), format!("{:.2}", booking.grundpreis));
    placeholders.insert("services_preis".to_string(), format!("{:.2}", booking.services_preis));
    placeholders.insert("rabatt_preis".to_string(), format!("{:.2}", booking.rabatt_preis));
    placeholders.insert("gesamtpreis".to_string(), format!("{:.2}", booking.gesamtpreis));

    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_internal(&config, &guest.email, &subject, &body, booking_id, guest.id, "invoice").await
}

/// Zahlungserinnerungs-Email senden
pub async fn send_payment_reminder_email(booking_id: i64) -> Result<String, String> {
    use crate::database::{get_booking_by_id, get_guest_by_id, get_room_by_id};

    let booking = get_booking_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;
    let guest = get_guest_by_id(booking.guest_id)
        .map_err(|e| format!("Fehler beim Laden des Gastes: {}", e))?;
    let room = get_room_by_id(booking.room_id)
        .map_err(|e| format!("Fehler beim Laden des Zimmers: {}", e))?;
    let config = get_email_config()?;
    let template = get_template_by_name("payment_reminder")?;

    let mut placeholders = std::collections::HashMap::new();
    placeholders.insert("gast_vorname".to_string(), guest.vorname.clone());
    placeholders.insert("gast_nachname".to_string(), guest.nachname.clone());
    placeholders.insert("reservierungsnummer".to_string(), booking.reservierungsnummer.clone());
    placeholders.insert("zimmer_name".to_string(), room.name.clone());
    placeholders.insert("checkin_date".to_string(), booking.checkin_date.clone());
    placeholders.insert("checkout_date".to_string(), booking.checkout_date.clone());
    placeholders.insert("gesamtpreis".to_string(), format!("{:.2}", booking.gesamtpreis));

    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_internal(&config, &guest.email, &subject, &body, booking_id, guest.id, "payment_reminder").await
}

/// Stornierungsbestätigungs-Email senden
pub async fn send_cancellation_email(booking_id: i64) -> Result<String, String> {
    use crate::database::{get_booking_by_id, get_guest_by_id, get_room_by_id};
    use chrono::Local;

    let booking = get_booking_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;
    let guest = get_guest_by_id(booking.guest_id)
        .map_err(|e| format!("Fehler beim Laden des Gastes: {}", e))?;
    let room = get_room_by_id(booking.room_id)
        .map_err(|e| format!("Fehler beim Laden des Zimmers: {}", e))?;
    let config = get_email_config()?;
    let template = get_template_by_name("cancellation")?;

    let mut placeholders = std::collections::HashMap::new();
    placeholders.insert("gast_vorname".to_string(), guest.vorname.clone());
    placeholders.insert("gast_nachname".to_string(), guest.nachname.clone());
    placeholders.insert("reservierungsnummer".to_string(), booking.reservierungsnummer.clone());
    placeholders.insert("zimmer_name".to_string(), room.name.clone());
    placeholders.insert("checkin_date".to_string(), booking.checkin_date.clone());
    placeholders.insert("checkout_date".to_string(), booking.checkout_date.clone());
    placeholders.insert("heute".to_string(), Local::now().format("%d.%m.%Y").to_string());

    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_internal(&config, &guest.email, &subject, &body, booking_id, guest.id, "cancellation").await
}

/// Interne Funktion zum tatsächlichen Senden der Email
async fn send_email_internal(
    config: &EmailConfig,
    recipient: &str,
    subject: &str,
    body: &str,
    booking_id: i64,
    guest_id: i64,
    template_name: &str,
) -> Result<String, String> {
    // Entschlüssele Passwort
    let password = decrypt_password(&config.smtp_password)?;

    // Erstelle Email
    let from_address = format!("{} <{}>", config.from_name, config.from_email);
    let email = Message::builder()
        .from(from_address.parse().map_err(|e| format!("Ungültige Absender-Adresse: {}", e))?)
        .to(recipient.parse().map_err(|e| format!("Ungültige Empfänger-Adresse: {}", e))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?;

    // Erstelle SMTP Transport
    let creds = Credentials::new(config.smtp_username.clone(), password);

    let mailer = SmtpTransport::relay(&config.smtp_server)
        .map_err(|e| format!("SMTP-Server nicht erreichbar: {}", e))?
        .port(config.smtp_port as u16)
        .credentials(creds)
        .build();

    // Sende Email und logge Ergebnis
    match mailer.send(&email) {
        Ok(_) => {
            log_email(booking_id, guest_id, template_name, recipient, subject, "gesendet", None)?;
            Ok(format!("Email erfolgreich an {} gesendet", recipient))
        }
        Err(e) => {
            let error_msg = format!("Fehler beim Senden: {}", e);
            log_email(booking_id, guest_id, template_name, recipient, subject, "fehler", Some(&error_msg))?;
            Err(error_msg)
        }
    }
}

/// Logge Email-Versand in Datenbank
fn log_email(
    booking_id: i64,
    guest_id: i64,
    template_name: &str,
    recipient_email: &str,
    subject: &str,
    status: &str,
    error_message: Option<&str>,
) -> Result<(), String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    conn.execute(
        "INSERT INTO email_logs (booking_id, guest_id, template_name, recipient_email, subject, status, error_message)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            &booking_id.to_string(),
            &guest_id.to_string(),
            template_name,
            recipient_email,
            subject,
            status,
            &error_message.unwrap_or(""),
        ],
    ).map_err(|e| format!("Fehler beim Loggen: {}", e))?;

    Ok(())
}

/// Email-Logs für eine Buchung abrufen
pub fn get_email_logs_for_booking(booking_id: i64) -> Result<Vec<EmailLog>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, booking_id, guest_id, template_name, recipient_email, subject, status, error_message, sent_at
         FROM email_logs
         WHERE booking_id = ?1
         ORDER BY sent_at DESC"
    ).map_err(|e| format!("Fehler beim Vorbereiten der Abfrage: {}", e))?;

    let logs = stmt.query_map([booking_id], |row| {
        Ok(EmailLog {
            id: row.get(0)?,
            booking_id: row.get(1)?,
            guest_id: row.get(2)?,
            template_name: row.get(3)?,
            recipient_email: row.get(4)?,
            subject: row.get(5)?,
            status: row.get(6)?,
            error_message: row.get(7)?,
            sent_at: row.get(8)?,
        })
    }).map_err(|e| format!("Fehler beim Abrufen der Logs: {}", e))?;

    let mut result = Vec::new();
    for log in logs {
        result.push(log.map_err(|e| format!("Fehler beim Verarbeiten: {}", e))?);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_password() {
        let original = "test_password_123";
        let encrypted = encrypt_password(original);
        let decrypted = decrypt_password(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_replace_placeholders() {
        let mut placeholders = std::collections::HashMap::new();
        placeholders.insert("name".to_string(), "Max".to_string());
        placeholders.insert("zimmer".to_string(), "101".to_string());

        let text = "Hallo {name}, Zimmer {zimmer}";
        let result = replace_placeholders(text, &placeholders);
        assert_eq!(result, "Hallo Max, Zimmer 101");
    }
}
