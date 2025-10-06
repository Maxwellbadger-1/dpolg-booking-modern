use rusqlite::{Connection, Result};
use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, Attachment, SinglePart, MultiPart},
    transport::smtp::authentication::Credentials,
    transport::smtp::client::{Tls, TlsParameters},
};
use crate::database::{EmailConfig, EmailTemplate, EmailLog, get_db_path};
use base64::{Engine as _, engine::general_purpose};
use std::path::PathBuf;

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

    // Prüfe ob bereits eine Konfiguration existiert
    let existing_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM email_config",
        [],
        |row| row.get(0)
    ).map_err(|e| format!("Fehler beim Prüfen der Konfiguration: {}", e))?;

    let use_tls_int = if use_tls { 1 } else { 0 };

    if existing_count > 0 {
        // Update existierende Konfiguration
        // Wenn Passwort leer ist, behalte das alte Passwort
        if smtp_password.is_empty() {
            // Update ohne Passwort zu ändern
            conn.execute(
                "UPDATE email_config SET
                    smtp_server = ?1,
                    smtp_port = ?2,
                    smtp_username = ?3,
                    from_email = ?4,
                    from_name = ?5,
                    use_tls = ?6,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = 1",
                [
                    &smtp_server,
                    &smtp_port.to_string(),
                    &smtp_username,
                    &from_email,
                    &from_name,
                    &use_tls_int.to_string(),
                ],
            ).map_err(|e| format!("Fehler beim Aktualisieren der Konfiguration: {}", e))?;
        } else {
            // Verschlüssele nur neues Passwort wenn eins eingegeben wurde
            let encrypted_password = encrypt_password(&smtp_password);
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
        }
    } else {
        // Neue Konfiguration erstellen - Passwort ist Pflicht
        if smtp_password.is_empty() {
            return Err("Passwort darf beim Erstellen einer neuen Konfiguration nicht leer sein".to_string());
        }

        let encrypted_password = encrypt_password(&smtp_password);
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
    eprintln!("🧪 [TEST EMAIL] Starte Test-Email...");
    eprintln!("📧 SMTP Server: {}:{}", smtp_server, smtp_port);
    eprintln!("👤 Username: {}", smtp_username);
    eprintln!("📨 From: {} <{}>", from_name, from_email);
    eprintln!("📬 To: {}", test_recipient);

    // Wenn kein Passwort übergeben wurde, lade gespeichertes Passwort
    let password_to_use = if smtp_password.is_empty() {
        eprintln!("⚠️  Kein Passwort übergeben, lade gespeichertes Passwort...");
        let config = get_email_config()
            .map_err(|e| format!("Keine Email-Konfiguration gefunden: {}", e))?;
        // Entschlüssele das gespeicherte Passwort
        decrypt_password(&config.smtp_password)
            .map_err(|e| format!("Fehler beim Entschlüsseln des Passworts: {}", e))?
    } else {
        smtp_password
    };

    // Erstelle Test-Email
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse().map_err(|e| {
            eprintln!("❌ Fehler: Ungültige Absender-Adresse: {}", e);
            format!("Ungültige Absender-Adresse: {}", e)
        })?)
        .to(test_recipient.parse().map_err(|e| {
            eprintln!("❌ Fehler: Ungültige Empfänger-Adresse: {}", e);
            format!("Ungültige Empfänger-Adresse: {}", e)
        })?)
        .subject("Test Email - DPolG Stiftung Buchungssystem")
        .header(ContentType::TEXT_PLAIN)
        .body("Dies ist eine Test-Email. Ihre SMTP-Konfiguration funktioniert!".to_string())
        .map_err(|e| {
            eprintln!("❌ Fehler beim Erstellen der Email: {}", e);
            format!("Fehler beim Erstellen der Email: {}", e)
        })?;

    eprintln!("✅ Test-Email erstellt");

    // Erstelle SMTP Transport mit dem (möglicherweise geladenen) Passwort
    let creds = Credentials::new(smtp_username.clone(), password_to_use);

    eprintln!("🔌 Verbinde zu SMTP-Server mit Port {}...", smtp_port);

    // Entscheide zwischen direktem SSL/TLS (Port 465) und STARTTLS (Port 587)
    let mailer = if smtp_port == 465 {
        eprintln!("🔒 Verwende Port 465 (direktes SSL/TLS)");
        // Port 465 = direktes SSL/TLS (Implicit TLS)
        let tls = TlsParameters::builder(smtp_server.clone())
            .build()
            .map_err(|e| {
                eprintln!("❌ TLS-Konfiguration fehlgeschlagen: {}", e);
                format!("TLS-Konfiguration fehlgeschlagen: {}", e)
            })?;

        SmtpTransport::relay(&smtp_server)
            .map_err(|e| {
                eprintln!("❌ SMTP-Server nicht erreichbar: {}", e);
                format!("SMTP-Server nicht erreichbar: {}", e)
            })?
            .port(smtp_port as u16)
            .tls(Tls::Wrapper(tls))
            .credentials(creds)
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build()
    } else {
        eprintln!("🔒 Verwende Port {} (STARTTLS)", smtp_port);
        // Port 587 = STARTTLS (Explicit TLS)
        SmtpTransport::starttls_relay(&smtp_server)
            .map_err(|e| {
                eprintln!("❌ SMTP-Server nicht erreichbar: {}", e);
                format!("SMTP-Server nicht erreichbar: {}", e)
            })?
            .port(smtp_port as u16)
            .credentials(creds)
            .timeout(Some(std::time::Duration::from_secs(30)))
            .build()
    };

    eprintln!("📤 Sende Test-Email...");

    // Sende Email
    mailer.send(&email)
        .map_err(|e| {
            eprintln!("❌ Fehler beim Senden der Test-Email: {}", e);
            format!("Fehler beim Senden der Test-Email: {}", e)
        })?;

    eprintln!("✅ Test-Email erfolgreich gesendet!");
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

/// Erstellt eine HashMap mit allen verfügbaren Platzhaltern für eine Buchung
fn create_all_placeholders(
    booking: &crate::database::BookingWithDetails,
    guest: &crate::database::Guest,
    room: &crate::database::Room,
) -> std::collections::HashMap<String, String> {
    use crate::database::{get_company_settings, get_booking_accompanying_guests, get_booking_services, get_payment_settings};

    let mut placeholders = std::collections::HashMap::new();

    // Gast-Daten (Basis)
    placeholders.insert("gast_vorname".to_string(), guest.vorname.clone());
    placeholders.insert("gast_nachname".to_string(), guest.nachname.clone());
    placeholders.insert("gast_email".to_string(), guest.email.clone());
    placeholders.insert("gast_telefon".to_string(), guest.telefon.clone());

    // Gast-Adresse (NEU)
    placeholders.insert("gast_strasse".to_string(), guest.strasse.clone().unwrap_or_default());
    placeholders.insert("gast_plz".to_string(), guest.plz.clone().unwrap_or_default());
    placeholders.insert("gast_ort".to_string(), guest.ort.clone().unwrap_or_default());
    placeholders.insert("gast_land".to_string(), "Deutschland".to_string()); // Default - kann später erweitert werden

    // Buchungs-Daten (Basis)
    placeholders.insert("reservierungsnummer".to_string(), booking.reservierungsnummer.clone());
    placeholders.insert("checkin_date".to_string(), booking.checkin_date.clone());
    placeholders.insert("checkout_date".to_string(), booking.checkout_date.clone());
    placeholders.insert("anzahl_gaeste".to_string(), booking.anzahl_gaeste.to_string());
    placeholders.insert("anzahl_naechte".to_string(), booking.anzahl_naechte.to_string());

    // Buchungs-Status (NEU)
    placeholders.insert("buchung_status".to_string(), booking.status.clone());
    placeholders.insert("bezahlt_status".to_string(), if booking.bezahlt { "Bezahlt".to_string() } else { "Offen".to_string() });
    placeholders.insert("erstellt_am".to_string(), booking.reservierungsnummer.clone()); // TODO: created_at fehlt in BookingWithDetails

    // Zimmer-Daten
    placeholders.insert("zimmer_name".to_string(), room.name.clone());
    placeholders.insert("zimmer_ort".to_string(), room.ort.clone());
    placeholders.insert("zimmer_typ".to_string(), room.gebaeude_typ.clone());
    if let Some(code) = &room.schluesselcode {
        placeholders.insert("schluesselcode".to_string(), code.clone());
    } else {
        placeholders.insert("schluesselcode".to_string(), "Wird beim Check-in bekannt gegeben".to_string());
    }

    // Mitreisende/Begleitpersonen (NEU)
    if let Ok(accompanying) = get_booking_accompanying_guests(booking.id) {
        placeholders.insert("anzahl_mitreisende".to_string(), accompanying.len().to_string());

        // Formatierte Liste
        let mut liste = String::new();
        for (i, person) in accompanying.iter().enumerate() {
            liste.push_str(&format!("{}. {} {}", i + 1, person.vorname, person.nachname));
            if let Some(geburtsdatum) = &person.geburtsdatum {
                liste.push_str(&format!(" (geb. {})", geburtsdatum));
            }
            liste.push('\n');
        }
        placeholders.insert("mitreisende_liste".to_string(), liste.trim().to_string());

        // Komma-separierte Namen
        let namen: Vec<String> = accompanying.iter()
            .map(|p| format!("{} {}", p.vorname, p.nachname))
            .collect();
        placeholders.insert("mitreisende_namen".to_string(), namen.join(", "));
    } else {
        placeholders.insert("anzahl_mitreisende".to_string(), "0".to_string());
        placeholders.insert("mitreisende_liste".to_string(), "Keine Begleitpersonen".to_string());
        placeholders.insert("mitreisende_namen".to_string(), "".to_string());
    }

    // Preis-Daten
    placeholders.insert("grundpreis".to_string(), format!("{:.2}", booking.grundpreis));
    placeholders.insert("services_preis".to_string(), format!("{:.2}", booking.services_preis));
    placeholders.insert("rabatt_preis".to_string(), format!("{:.2}", booking.rabatt_preis));
    placeholders.insert("gesamtpreis".to_string(), format!("{:.2}", booking.gesamtpreis));

    // Zahlungs-Informationen (NEU)
    if let Ok(payment_settings) = get_payment_settings() {
        let zahlungsziel_tage = payment_settings.payment_due_days;
        placeholders.insert("zahlungsziel_tage".to_string(), zahlungsziel_tage.to_string());

        // Berechne Zahlungsziel-Datum (created_at + payment_due_days)
        // TODO: created_at fehlt in BookingWithDetails, verwende vorläufig checkin_date
        placeholders.insert("zahlungsziel_datum".to_string(), booking.checkin_date.clone());
    } else {
        placeholders.insert("zahlungsziel_tage".to_string(), "14".to_string());
        placeholders.insert("zahlungsziel_datum".to_string(), "".to_string());
    }

    // Offener Betrag
    let offener_betrag = if booking.bezahlt { 0.0 } else { booking.gesamtpreis };
    placeholders.insert("offener_betrag".to_string(), format!("{:.2}", offener_betrag));

    // Services/Zusatzleistungen (NEU)
    if let Ok(services) = get_booking_services(booking.id) {
        // Einfache Liste
        let mut services_liste = String::new();
        for (i, service) in services.iter().enumerate() {
            services_liste.push_str(&format!("{}. {}\n", i + 1, service.service_name));
        }
        placeholders.insert("services_liste".to_string(), services_liste.trim().to_string());

        // Details mit Preisen
        let mut services_details = String::new();
        for service in &services {
            services_details.push_str(&format!("{}: {:.2} €\n", service.service_name, service.service_price));
        }
        placeholders.insert("services_details".to_string(), services_details.trim().to_string());
    } else {
        placeholders.insert("services_liste".to_string(), "Keine Zusatzleistungen".to_string());
        placeholders.insert("services_details".to_string(), "".to_string());
    }

    // Datums-Platzhalter
    placeholders.insert("heute".to_string(), crate::time_utils::format_today_de());
    placeholders.insert("jetzt".to_string(), crate::time_utils::format_now_de());

    // Company-Daten (wenn verfügbar)
    if let Ok(company) = get_company_settings() {
        placeholders.insert("firma_name".to_string(), company.company_name);
        placeholders.insert("firma_adresse".to_string(), company.street_address);
        placeholders.insert("firma_plz".to_string(), company.plz);
        placeholders.insert("firma_ort".to_string(), company.city);
        placeholders.insert("firma_telefon".to_string(), company.phone);
        placeholders.insert("firma_email".to_string(), company.email);
        placeholders.insert("firma_website".to_string(), company.website);
        placeholders.insert("firma_steuernummer".to_string(), company.tax_id);
    }

    // Bank-Daten aus payment_settings (NEU)
    if let Ok(payment_settings) = get_payment_settings() {
        placeholders.insert("firma_iban".to_string(), payment_settings.iban);
        placeholders.insert("firma_bic".to_string(), payment_settings.bic);
        placeholders.insert("firma_kontoinhaber".to_string(), payment_settings.account_holder);
    } else {
        placeholders.insert("firma_iban".to_string(), "".to_string());
        placeholders.insert("firma_bic".to_string(), "".to_string());
        placeholders.insert("firma_kontoinhaber".to_string(), "".to_string());
    }

    placeholders
}

/// Buchungsbestätigungs-Email senden
pub async fn send_confirmation_email(booking_id: i64) -> Result<String, String> {
    use crate::database::get_booking_with_details_by_id;

    // Lade Buchung mit allen Details
    let booking_details = get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let config = get_email_config()?;
    let template = get_template_by_name("confirmation")?;

    // Erstelle alle Platzhalter
    let placeholders = create_all_placeholders(&booking_details, &booking_details.guest, &booking_details.room);

    // Ersetze Platzhalter
    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    // Sende Email
    send_email_internal(&config, &booking_details.guest.email, &subject, &body, booking_id, booking_details.guest.id, "confirmation").await
}

/// Reminder-Email senden
pub async fn send_reminder_email(booking_id: i64) -> Result<String, String> {
    use crate::database::get_booking_with_details_by_id;

    let booking_details = get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let config = get_email_config()?;
    let template = get_template_by_name("reminder")?;

    let placeholders = create_all_placeholders(&booking_details, &booking_details.guest, &booking_details.room);
    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_internal(&config, &booking_details.guest.email, &subject, &body, booking_id, booking_details.guest.id, "reminder").await
}

/// Rechnungs-Email senden (ohne PDF-Anhang - Legacy)
pub async fn send_invoice_email(booking_id: i64) -> Result<String, String> {
    use crate::database::get_booking_with_details_by_id;

    let booking_details = get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let config = get_email_config()?;
    let template = get_template_by_name("invoice")?;

    let placeholders = create_all_placeholders(&booking_details, &booking_details.guest, &booking_details.room);
    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_internal(&config, &booking_details.guest.email, &subject, &body, booking_id, booking_details.guest.id, "invoice").await
}

/// Rechnungs-Email mit PDF-Anhang senden (NEU - automatisch)
pub async fn send_invoice_email_with_pdf(booking_id: i64, pdf_path: PathBuf) -> Result<String, String> {
    use crate::database::get_booking_with_details_by_id;

    let booking_details = get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let config = get_email_config()?;
    let template = get_template_by_name("invoice")?;

    let placeholders = create_all_placeholders(&booking_details, &booking_details.guest, &booking_details.room);
    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_with_attachment_internal(
        &config,
        &booking_details.guest.email,
        &subject,
        &body,
        &pdf_path,
        booking_id,
        booking_details.guest.id,
        "invoice"
    ).await
}

/// Zahlungserinnerungs-Email senden
pub async fn send_payment_reminder_email(booking_id: i64) -> Result<String, String> {
    use crate::database::get_booking_with_details_by_id;

    let booking_details = get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let config = get_email_config()?;
    let template = get_template_by_name("payment_reminder")?;

    let placeholders = create_all_placeholders(&booking_details, &booking_details.guest, &booking_details.room);
    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_internal(&config, &booking_details.guest.email, &subject, &body, booking_id, booking_details.guest.id, "payment_reminder").await
}

/// Stornierungsbestätigungs-Email senden
pub async fn send_cancellation_email(booking_id: i64) -> Result<String, String> {
    use crate::database::get_booking_with_details_by_id;

    let booking_details = get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let config = get_email_config()?;
    let template = get_template_by_name("cancellation")?;

    let placeholders = create_all_placeholders(&booking_details, &booking_details.guest, &booking_details.room);
    let subject = replace_placeholders(&template.subject, &placeholders);
    let body = replace_placeholders(&template.body, &placeholders);

    send_email_internal(&config, &booking_details.guest.email, &subject, &body, booking_id, booking_details.guest.id, "cancellation").await
}

/// Erstellt HTML-Email-Body mit Logo-Header
fn create_html_email_body(body_text: &str, logo_path: Option<&str>) -> String {
    let logo_html = if let Some(path) = logo_path {
        format!(r#"
            <div style="text-align: center; margin-bottom: 30px;">
                <img src="cid:company_logo" alt="Logo" style="max-width: 200px; height: auto;" />
            </div>
        "#)
    } else {
        String::new()
    };

    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            font-family: Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .container {{
            background-color: #ffffff;
            border: 1px solid #e0e0e0;
            border-radius: 8px;
            padding: 30px;
        }}
        .content {{
            white-space: pre-wrap;
        }}
    </style>
</head>
<body>
    <div class="container">
        {}
        <div class="content">{}</div>
    </div>
</body>
</html>
    "#, logo_html, body_text)
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
    use crate::database::get_company_settings;

    // Entschlüssele Passwort
    let password = decrypt_password(&config.smtp_password)?;

    // Lade Company Settings für Logo
    let company = get_company_settings().ok();
    let logo_path = company.as_ref().and_then(|c| c.logo_path.as_deref());

    // Erstelle Email
    let from_address = format!("{} <{}>", config.from_name, config.from_email);

    // Versuche Email mit Logo zu erstellen, falle zurück auf HTML ohne Logo oder Plain Text
    let email = if let Some(logo_file) = logo_path {
        // Versuche Logo-Datei zu lesen
        match std::fs::read(logo_file) {
            Ok(logo_data) => {
                // Email mit Logo (HTML + Inline-Image)
                let html_body = create_html_email_body(body, Some(logo_file));

                let content_type = if logo_file.ends_with(".png") {
                    "image/png"
                } else if logo_file.ends_with(".jpg") || logo_file.ends_with(".jpeg") {
                    "image/jpeg"
                } else if logo_file.ends_with(".gif") {
                    "image/gif"
                } else {
                    "image/png"
                };

                // Erstelle Inline-Logo-Attachment
                let logo_part = Attachment::new_inline(String::from("company_logo"))
                    .body(logo_data, content_type.parse().unwrap());

                // Erstelle HTML-Teil
                let html_part = SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_body);

                // Kombiniere HTML + Inline-Image
                let multipart = MultiPart::related()
                    .singlepart(html_part)
                    .singlepart(logo_part);

                Message::builder()
                    .from(from_address.parse().map_err(|e| format!("Ungültige Absender-Adresse: {}", e))?)
                    .to(recipient.parse().map_err(|e| format!("Ungültige Empfänger-Adresse: {}", e))?)
                    .subject(subject)
                    .multipart(multipart)
                    .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?
            }
            Err(e) => {
                // Logo konnte nicht gelesen werden - verwende HTML ohne Logo
                eprintln!("⚠️ Logo konnte nicht gelesen werden ({}), verwende HTML ohne Logo", e);
                let html_body = create_html_email_body(body, None);

                Message::builder()
                    .from(from_address.parse().map_err(|e| format!("Ungültige Absender-Adresse: {}", e))?)
                    .to(recipient.parse().map_err(|e| format!("Ungültige Empfänger-Adresse: {}", e))?)
                    .subject(subject)
                    .header(ContentType::TEXT_HTML)
                    .body(html_body)
                    .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?
            }
        }
    } else {
        // Kein Logo konfiguriert - verwende HTML ohne Logo
        let html_body = create_html_email_body(body, None);

        Message::builder()
            .from(from_address.parse().map_err(|e| format!("Ungültige Absender-Adresse: {}", e))?)
            .to(recipient.parse().map_err(|e| format!("Ungültige Empfänger-Adresse: {}", e))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body)
            .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?
    };

    // Erstelle SMTP Transport
    let creds = Credentials::new(config.smtp_username.clone(), password);

    // Entscheide zwischen direktem SSL/TLS (Port 465) und STARTTLS (Port 587)
    let mailer = if config.smtp_port == 465 {
        // Port 465 = direktes SSL/TLS (Implicit TLS) - für web.de, GMX, Gmail, Yahoo, Strato
        // Verwendet Tls::Wrapper für SSL-wrapped connection von Anfang an (wie Python's SMTP_SSL)
        let tls = TlsParameters::builder(config.smtp_server.clone())
            .build()
            .map_err(|e| format!("TLS-Konfiguration fehlgeschlagen: {}", e))?;

        SmtpTransport::relay(&config.smtp_server)
            .map_err(|e| format!("SMTP-Server nicht erreichbar: {}", e))?
            .tls(Tls::Wrapper(tls))
            .port(465)
            .credentials(creds)
            .build()
    } else {
        // Port 587 = STARTTLS (Explicit TLS) - für Office365, Apple Mail, IONOS, Telekom
        SmtpTransport::starttls_relay(&config.smtp_server)
            .map_err(|e| format!("SMTP-Server nicht erreichbar: {}", e))?
            .port(config.smtp_port as u16)
            .credentials(creds)
            .build()
    };

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

/// Interne Funktion zum Senden von Emails MIT Anhang (z.B. PDF)
async fn send_email_with_attachment_internal(
    config: &EmailConfig,
    recipient: &str,
    subject: &str,
    body: &str,
    attachment_path: &PathBuf,
    booking_id: i64,
    guest_id: i64,
    template_name: &str,
) -> Result<String, String> {
    use crate::database::get_company_settings;

    // Entschlüssele Passwort
    let password = decrypt_password(&config.smtp_password)?;

    // PDF-Datei lesen
    let pdf_content = std::fs::read(attachment_path)
        .map_err(|e| format!("Fehler beim Lesen der PDF-Datei: {}", e))?;

    // Dateiname extrahieren
    let filename = attachment_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("rechnung.pdf");

    // Erstelle Email mit Attachment
    let from_address = format!("{} <{}>", config.from_name, config.from_email);

    let attachment = Attachment::new(filename.to_string())
        .body(pdf_content, ContentType::parse("application/pdf").unwrap());

    // Lade Company Settings für Logo
    let company = get_company_settings().ok();
    let logo_path = company.as_ref().and_then(|c| c.logo_path.as_deref());

    // Versuche Email mit Logo zu erstellen, falle zurück auf HTML ohne Logo
    let email = if let Some(logo_file) = logo_path {
        // Versuche Logo-Datei zu lesen
        match std::fs::read(logo_file) {
            Ok(logo_data) => {
                // Email mit Logo (HTML + Inline-Image + PDF-Anhang)
                let html_body = create_html_email_body(body, Some(logo_file));

                let content_type = if logo_file.ends_with(".png") {
                    "image/png"
                } else if logo_file.ends_with(".jpg") || logo_file.ends_with(".jpeg") {
                    "image/jpeg"
                } else if logo_file.ends_with(".gif") {
                    "image/gif"
                } else {
                    "image/png"
                };

                // Erstelle Inline-Logo-Attachment
                let logo_part = Attachment::new_inline(String::from("company_logo"))
                    .body(logo_data, content_type.parse().unwrap());

                // Erstelle HTML-Teil
                let html_part = SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_body);

                // Kombiniere HTML + Inline-Image in related part
                let related = MultiPart::related()
                    .singlepart(html_part)
                    .singlepart(logo_part);

                // Kombiniere related part + PDF attachment in mixed part
                let multipart = MultiPart::mixed()
                    .multipart(related)
                    .singlepart(attachment);

                Message::builder()
                    .from(from_address.parse().map_err(|e| format!("Ungültige Absender-Adresse: {}", e))?)
                    .to(recipient.parse().map_err(|e| format!("Ungültige Empfänger-Adresse: {}", e))?)
                    .subject(subject)
                    .multipart(multipart)
                    .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?
            }
            Err(e) => {
                // Logo konnte nicht gelesen werden - verwende HTML ohne Logo
                eprintln!("⚠️ Logo konnte nicht gelesen werden ({}), verwende HTML ohne Logo", e);
                let html_body = create_html_email_body(body, None);

                let html_part = SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_body);

                let multipart = MultiPart::mixed()
                    .singlepart(html_part)
                    .singlepart(attachment);

                Message::builder()
                    .from(from_address.parse().map_err(|e| format!("Ungültige Absender-Adresse: {}", e))?)
                    .to(recipient.parse().map_err(|e| format!("Ungültige Empfänger-Adresse: {}", e))?)
                    .subject(subject)
                    .multipart(multipart)
                    .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?
            }
        }
    } else {
        // Kein Logo konfiguriert - verwende HTML ohne Logo
        let html_body = create_html_email_body(body, None);

        let html_part = SinglePart::builder()
            .header(ContentType::TEXT_HTML)
            .body(html_body);

        let multipart = MultiPart::mixed()
            .singlepart(html_part)
            .singlepart(attachment);

        Message::builder()
            .from(from_address.parse().map_err(|e| format!("Ungültige Absender-Adresse: {}", e))?)
            .to(recipient.parse().map_err(|e| format!("Ungültige Empfänger-Adresse: {}", e))?)
            .subject(subject)
            .multipart(multipart)
            .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?
    };

    // Erstelle SMTP Transport
    let creds = Credentials::new(config.smtp_username.clone(), password);

    // Entscheide zwischen direktem SSL/TLS (Port 465) und STARTTLS (Port 587)
    let mailer = if config.smtp_port == 465 {
        // Port 465 = direktes SSL/TLS (Implicit TLS) - für web.de, GMX, Gmail, Yahoo, Strato
        // Verwendet Tls::Wrapper für SSL-wrapped connection von Anfang an (wie Python's SMTP_SSL)
        let tls = TlsParameters::builder(config.smtp_server.clone())
            .build()
            .map_err(|e| format!("TLS-Konfiguration fehlgeschlagen: {}", e))?;

        SmtpTransport::relay(&config.smtp_server)
            .map_err(|e| format!("SMTP-Server nicht erreichbar: {}", e))?
            .tls(Tls::Wrapper(tls))
            .port(465)
            .credentials(creds)
            .build()
    } else {
        // Port 587 = STARTTLS (Explicit TLS) - für Office365, Apple Mail, IONOS, Telekom
        SmtpTransport::starttls_relay(&config.smtp_server)
            .map_err(|e| format!("SMTP-Server nicht erreichbar: {}", e))?
            .port(config.smtp_port as u16)
            .credentials(creds)
            .build()
    };

    // Sende Email und logge Ergebnis
    match mailer.send(&email) {
        Ok(_) => {
            log_email(booking_id, guest_id, template_name, recipient, subject, "gesendet", None)?;
            Ok(format!("Email mit PDF-Anhang erfolgreich an {} gesendet", recipient))
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

/// Alle Email-Logs abrufen (für Email-Verlauf)
pub fn get_all_email_logs() -> Result<Vec<EmailLog>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, booking_id, guest_id, template_name, recipient_email, subject, status, error_message, sent_at
         FROM email_logs
         ORDER BY sent_at DESC"
    ).map_err(|e| format!("Fehler beim Vorbereiten der Abfrage: {}", e))?;

    let logs = stmt.query_map([], |row| {
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

/// Löscht einen Email-Log Eintrag
pub fn delete_email_log(log_id: i64) -> Result<(), String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    conn.execute(
        "DELETE FROM email_logs WHERE id = ?1",
        [log_id],
    ).map_err(|e| format!("Fehler beim Löschen des Email-Logs: {}", e))?;

    println!("✅ Email-Log {} gelöscht", log_id);
    Ok(())
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
