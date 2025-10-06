use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: i64,
    pub name: String,
    pub gebaeude_typ: String,
    pub capacity: i32,
    pub price_member: f64,
    pub price_non_member: f64,
    pub nebensaison_preis: f64,
    pub hauptsaison_preis: f64,
    pub endreinigung: f64,
    pub ort: String,
    pub schluesselcode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Guest {
    pub id: i64,
    pub vorname: String,
    pub nachname: String,
    pub email: String,
    pub telefon: String,
    pub dpolg_mitglied: bool,
    pub strasse: Option<String>,
    pub plz: Option<String>,
    pub ort: Option<String>,
    pub mitgliedsnummer: Option<String>,
    pub notizen: Option<String>,
    pub beruf: Option<String>,
    pub bundesland: Option<String>,
    pub dienststelle: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Booking {
    pub id: i64,
    pub room_id: i64,
    pub guest_id: i64,
    pub reservierungsnummer: String,
    pub checkin_date: String,
    pub checkout_date: String,
    pub anzahl_gaeste: i32,
    pub status: String,
    pub gesamtpreis: f64,
    pub bemerkungen: Option<String>,
    pub created_at: String,
    pub anzahl_begleitpersonen: i32,
    pub grundpreis: f64,
    pub services_preis: f64,
    pub rabatt_preis: f64,
    pub anzahl_naechte: i32,
    pub updated_at: Option<String>,
    // Phase 6.4: Zahlungsstatus
    pub bezahlt: bool,
    pub bezahlt_am: Option<String>,
    pub zahlungsmethode: Option<String>,
    pub mahnung_gesendet_am: Option<String>,
}

// Neue Tabelle: Begleitpersonen für eine Buchung
#[derive(Debug, Serialize, Deserialize)]
pub struct AccompanyingGuest {
    pub id: i64,
    pub booking_id: i64,
    pub vorname: String,
    pub nachname: String,
    pub geburtsdatum: Option<String>, // Format: YYYY-MM-DD
}

// Neue Tabelle: Zusätzliche Services für eine Buchung
#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalService {
    pub id: i64,
    pub booking_id: i64,
    pub service_name: String,
    pub service_price: f64,
    pub created_at: String,
}

// Neue Tabelle: Rabatte für eine Buchung
#[derive(Debug, Serialize, Deserialize)]
pub struct Discount {
    pub id: i64,
    pub booking_id: i64,
    pub discount_name: String,
    pub discount_type: String, // 'percent' oder 'fixed'
    pub discount_value: f64,
    pub created_at: String,
}

// Service-Template (vordefinierte Zusatzleistung)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceTemplate {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// Rabatt-Template (vordefinierter Rabatt)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscountTemplate {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub discount_type: String, // 'percent' oder 'fixed'
    pub discount_value: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

// Phase 6: Email-System Structs
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailConfig {
    pub id: i64,
    pub smtp_server: String,
    pub smtp_port: i32,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
    pub created_at: String,
    pub updated_at: String,
}

// Phase 7: Company & Payment Settings Structs
#[derive(Debug, Serialize, Deserialize)]
pub struct CompanySettings {
    pub id: i64,
    pub company_name: String,
    pub street_address: String,
    pub plz: String,
    pub city: String,
    pub country: String,
    pub phone: String,
    pub fax: Option<String>,
    pub email: String,
    pub website: String,
    pub tax_id: String,
    pub ceo_name: String,
    pub registry_court: String,
    pub logo_path: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentSettings {
    pub id: i64,
    pub bank_name: String,
    pub iban: String,
    pub bic: String,
    pub account_holder: String,
    pub mwst_rate: f64,
    pub dpolg_rabatt: f64,
    pub payment_due_days: i32,
    pub reminder_after_days: i32,
    pub payment_text: Option<String>,
    pub updated_at: String,
}

// Notification Settings (Email Scheduler Configuration)
#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub id: i64,
    pub checkin_reminders_enabled: bool,
    pub payment_reminders_enabled: bool,
    pub payment_reminder_after_days: i64,
    pub payment_reminder_repeat_days: i64,
    pub scheduler_interval_hours: i64,
    pub updated_at: String,
}

// Pricing Settings (Saisonzeiten & Mitgliederrabatt)
#[derive(Debug, Serialize, Deserialize)]
pub struct PricingSettings {
    pub id: i64,
    pub hauptsaison_aktiv: bool,
    pub hauptsaison_start: String,  // Format: MM-DD (z.B. "06-01")
    pub hauptsaison_ende: String,   // Format: MM-DD (z.B. "08-31")
    pub mitglieder_rabatt_aktiv: bool,
    pub mitglieder_rabatt_prozent: f64,  // z.B. 15.0 für 15%
    pub rabatt_basis: String,  // 'zimmerpreis' oder 'gesamtpreis'
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: i64,
    pub template_name: String,
    pub subject: String,
    pub body: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailLog {
    pub id: i64,
    pub booking_id: Option<i64>,
    pub guest_id: i64,
    pub template_name: String,
    pub recipient_email: String,
    pub subject: String,
    pub status: String,
    pub error_message: Option<String>,
    pub sent_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookingWithDetails {
    pub id: i64,
    pub room_id: i64,
    pub guest_id: i64,
    pub reservierungsnummer: String,
    pub checkin_date: String,
    pub checkout_date: String,
    pub anzahl_gaeste: i32,
    pub anzahl_begleitpersonen: i32,
    pub status: String,
    pub grundpreis: f64,
    pub services_preis: f64,
    pub rabatt_preis: f64,
    pub gesamtpreis: f64,
    pub anzahl_naechte: i32,
    pub bemerkungen: Option<String>,
    pub bezahlt: bool,
    pub bezahlt_am: Option<String>,
    pub zahlungsmethode: Option<String>,
    pub mahnung_gesendet_am: Option<String>,
    pub room: Room,
    pub guest: Guest,
}

pub fn get_db_path() -> PathBuf {
    // Use DB in src-tauri directory
    PathBuf::from("booking_system.db")
}

pub fn init_database() -> Result<()> {
    let conn = Connection::open(get_db_path())?;

    // WICHTIG: Foreign Keys in SQLite sind standardmäßig deaktiviert!
    // Muss für JEDE Connection aktiviert werden
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create rooms table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS rooms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            gebaeude_typ TEXT NOT NULL,
            capacity INTEGER NOT NULL,
            price_member REAL NOT NULL,
            price_non_member REAL NOT NULL,
            nebensaison_preis REAL NOT NULL DEFAULT 0.0,
            hauptsaison_preis REAL NOT NULL DEFAULT 0.0,
            endreinigung REAL NOT NULL DEFAULT 0.0,
            ort TEXT NOT NULL,
            schluesselcode TEXT
        )",
        [],
    )?;

    // Create guests table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS guests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            vorname TEXT NOT NULL,
            nachname TEXT NOT NULL,
            email TEXT NOT NULL,
            telefon TEXT NOT NULL,
            dpolg_mitglied BOOLEAN NOT NULL,
            strasse TEXT,
            plz TEXT,
            ort TEXT
        )",
        [],
    )?;

    // Create bookings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bookings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            room_id INTEGER NOT NULL,
            guest_id INTEGER NOT NULL,
            reservierungsnummer TEXT NOT NULL UNIQUE,
            checkin_date TEXT NOT NULL,
            checkout_date TEXT NOT NULL,
            anzahl_gaeste INTEGER NOT NULL,
            status TEXT NOT NULL,
            gesamtpreis REAL NOT NULL,
            bemerkungen TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (room_id) REFERENCES rooms (id),
            FOREIGN KEY (guest_id) REFERENCES guests (id)
        )",
        [],
    )?;

    // Phase 1.1: Erweitere guests Tabelle mit neuen Feldern
    // Prüfe zuerst, ob die Spalten bereits existieren
    add_column_if_not_exists(&conn, "guests", "mitgliedsnummer", "TEXT")?;
    add_column_if_not_exists(&conn, "guests", "notizen", "TEXT")?;
    add_column_if_not_exists(&conn, "guests", "beruf", "TEXT")?;
    add_column_if_not_exists(&conn, "guests", "bundesland", "TEXT")?;
    add_column_if_not_exists(&conn, "guests", "dienststelle", "TEXT")?;
    add_column_if_not_exists(&conn, "guests", "created_at", "TEXT DEFAULT CURRENT_TIMESTAMP")?;

    // Phase 1.1: Erweitere bookings Tabelle mit neuen Feldern für Preiskalkulation
    add_column_if_not_exists(&conn, "bookings", "anzahl_begleitpersonen", "INTEGER DEFAULT 0")?;
    add_column_if_not_exists(&conn, "bookings", "grundpreis", "REAL DEFAULT 0")?;
    add_column_if_not_exists(&conn, "bookings", "services_preis", "REAL DEFAULT 0")?;
    add_column_if_not_exists(&conn, "bookings", "rabatt_preis", "REAL DEFAULT 0")?;
    add_column_if_not_exists(&conn, "bookings", "anzahl_naechte", "INTEGER DEFAULT 0")?;
    add_column_if_not_exists(&conn, "bookings", "updated_at", "TEXT DEFAULT CURRENT_TIMESTAMP")?;

    // Phase 6.4: Zahlungsstatus-Tracking
    add_column_if_not_exists(&conn, "bookings", "bezahlt", "INTEGER DEFAULT 0")?;
    add_column_if_not_exists(&conn, "bookings", "bezahlt_am", "TEXT")?;
    add_column_if_not_exists(&conn, "bookings", "zahlungsmethode", "TEXT")?;
    add_column_if_not_exists(&conn, "bookings", "mahnung_gesendet_am", "TEXT")?;

    // Phase 1.6: Erweiterte Preisstruktur (Nebensaison/Hauptsaison + Endreinigung)
    add_column_if_not_exists(&conn, "rooms", "nebensaison_preis", "REAL DEFAULT 0.0")?;
    add_column_if_not_exists(&conn, "rooms", "hauptsaison_preis", "REAL DEFAULT 0.0")?;
    add_column_if_not_exists(&conn, "rooms", "endreinigung", "REAL DEFAULT 0.0")?;

    // Phase 1.6: DPolG-Rabatt in payment_settings
    add_column_if_not_exists(&conn, "payment_settings", "dpolg_rabatt", "REAL DEFAULT 15.0")?;

    // Phase 1.1: Neue Tabelle für Begleitpersonen
    // CASCADE DELETE: Wenn Buchung gelöscht wird, werden auch die Begleitpersonen gelöscht
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accompanying_guests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            booking_id INTEGER NOT NULL,
            vorname TEXT NOT NULL,
            nachname TEXT NOT NULL,
            geburtsdatum TEXT,
            FOREIGN KEY (booking_id) REFERENCES bookings (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Phase 1.1: Neue Tabelle für zusätzliche Services
    conn.execute(
        "CREATE TABLE IF NOT EXISTS additional_services (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            booking_id INTEGER NOT NULL,
            service_name TEXT NOT NULL,
            service_price REAL NOT NULL CHECK(service_price >= 0),
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (booking_id) REFERENCES bookings (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Phase 1.1: Neue Tabelle für Rabatte
    conn.execute(
        "CREATE TABLE IF NOT EXISTS discounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            booking_id INTEGER NOT NULL,
            discount_name TEXT NOT NULL,
            discount_type TEXT NOT NULL CHECK(discount_type IN ('percent', 'fixed')),
            discount_value REAL NOT NULL CHECK(discount_value >= 0),
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (booking_id) REFERENCES bookings (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Phase 1.1: Erstelle Indexes für Performance-Optimierung
    // Basierend auf Best Practices für Buchungssysteme:
    // - Datumsbereich-Abfragen sind am häufigsten
    // - Foreign Key Lookups werden oft verwendet
    // - Email-Suche für Gäste
    create_indexes(&conn)?;

    // Insert sample data if tables are empty
    // DEAKTIVIERT: Wir haben echte Produktionsdaten
    // insert_sample_data(&conn)?;

    Ok(())
}

// Hilfsfunktion: Fügt eine Spalte hinzu, falls sie noch nicht existiert
fn add_column_if_not_exists(
    conn: &Connection,
    table: &str,
    column: &str,
    column_type: &str,
) -> Result<()> {
    // Prüfe mit PRAGMA table_info, ob die Spalte existiert
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !columns.contains(&column.to_string()) {
        conn.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, column_type),
            [],
        )?;
    }

    Ok(())
}

// Erstellt alle notwendigen Indexes für Performance
fn create_indexes(conn: &Connection) -> Result<()> {
    // Index für Datumsbereich-Abfragen (wichtigste Query in Buchungssystemen)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_bookings_dates ON bookings(checkin_date, checkout_date)",
        [],
    )?;

    // Indexes für Foreign Key Lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_bookings_room ON bookings(room_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_bookings_guest ON bookings(guest_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_accompanying_booking ON accompanying_guests(booking_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_services_booking ON additional_services(booking_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_discounts_booking ON discounts(booking_id)",
        [],
    )?;

    // Index für Email-Suche bei Gästen
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_guests_email ON guests(email)",
        [],
    )?;

    // Phase 6: Email-System Tabellen
    // Tabelle für Email-Konfiguration (SMTP Settings)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_config (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            smtp_server TEXT NOT NULL,
            smtp_port INTEGER NOT NULL,
            smtp_username TEXT NOT NULL,
            smtp_password TEXT NOT NULL,
            from_email TEXT NOT NULL,
            from_name TEXT NOT NULL,
            use_tls INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Tabelle für Email-Templates
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_name TEXT NOT NULL UNIQUE,
            subject TEXT NOT NULL,
            body TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Tabelle für Email-Logs (Versandhistorie)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            booking_id INTEGER,
            guest_id INTEGER NOT NULL,
            template_name TEXT NOT NULL,
            recipient_email TEXT NOT NULL,
            subject TEXT NOT NULL,
            status TEXT NOT NULL,
            error_message TEXT,
            sent_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (booking_id) REFERENCES bookings (id) ON DELETE SET NULL,
            FOREIGN KEY (guest_id) REFERENCES guests (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Index für Email-Logs
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_logs_booking ON email_logs(booking_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_email_logs_guest ON email_logs(guest_id)",
        [],
    )?;

    // Standard-Email-Templates einfügen (falls noch nicht vorhanden)
    insert_default_email_templates(&conn)?;

    // Phase 7: Company & Payment Settings Tabellen
    // Tabelle für Firmeneinstellungen (Single-Row Tabelle mit id=1)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS company_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            company_name TEXT NOT NULL,
            street_address TEXT NOT NULL,
            plz TEXT NOT NULL,
            city TEXT NOT NULL,
            country TEXT NOT NULL DEFAULT 'Deutschland',
            phone TEXT NOT NULL,
            fax TEXT,
            email TEXT NOT NULL,
            website TEXT NOT NULL,
            tax_id TEXT NOT NULL,
            ceo_name TEXT NOT NULL,
            registry_court TEXT NOT NULL,
            logo_path TEXT,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Tabelle für Zahlungseinstellungen (Single-Row Tabelle mit id=1)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS payment_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            bank_name TEXT NOT NULL,
            iban TEXT NOT NULL,
            bic TEXT NOT NULL,
            account_holder TEXT NOT NULL,
            mwst_rate REAL NOT NULL CHECK(mwst_rate >= 0 AND mwst_rate <= 100),
            payment_due_days INTEGER NOT NULL DEFAULT 14,
            reminder_after_days INTEGER NOT NULL DEFAULT 14,
            payment_text TEXT,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Notification Settings Table (Email Scheduler Configuration)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notification_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            checkin_reminders_enabled BOOLEAN NOT NULL DEFAULT 1,
            payment_reminders_enabled BOOLEAN NOT NULL DEFAULT 1,
            payment_reminder_after_days INTEGER NOT NULL DEFAULT 14,
            payment_reminder_repeat_days INTEGER NOT NULL DEFAULT 14,
            scheduler_interval_hours INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Standard Notification Settings einfügen (falls noch nicht vorhanden)
    conn.execute(
        "INSERT OR IGNORE INTO notification_settings (id, checkin_reminders_enabled, payment_reminders_enabled, payment_reminder_after_days, payment_reminder_repeat_days, scheduler_interval_hours)
         VALUES (1, 1, 1, 14, 14, 1)",
        [],
    )?;

    // Pricing Settings Table (Saisonzeiten & Mitgliederrabatt)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pricing_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            hauptsaison_aktiv BOOLEAN NOT NULL DEFAULT 1,
            hauptsaison_start TEXT NOT NULL DEFAULT '06-01',
            hauptsaison_ende TEXT NOT NULL DEFAULT '08-31',
            mitglieder_rabatt_aktiv BOOLEAN NOT NULL DEFAULT 1,
            mitglieder_rabatt_prozent REAL NOT NULL DEFAULT 15.0,
            rabatt_basis TEXT NOT NULL DEFAULT 'zimmerpreis' CHECK(rabatt_basis IN ('zimmerpreis', 'gesamtpreis')),
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Standard Pricing Settings einfügen (falls noch nicht vorhanden)
    conn.execute(
        "INSERT OR IGNORE INTO pricing_settings (id, hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende, mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis)
         VALUES (1, 1, '06-01', '08-31', 1, 15.0, 'zimmerpreis')",
        [],
    )?;

    // Standard-Settings einfügen (falls noch nicht vorhanden)
    insert_default_settings(&conn)?;

    // Tabelle für Service-Templates (vordefinierte Zusatzleistungen)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS service_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            price REAL NOT NULL CHECK(price >= 0),
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Tabelle für Rabatt-Templates (vordefinierte Rabatte)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS discount_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            discount_type TEXT NOT NULL CHECK(discount_type IN ('percent', 'fixed')),
            discount_value REAL NOT NULL CHECK(discount_value >= 0),
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Indexes für Templates
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_service_templates_active ON service_templates(is_active)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_discount_templates_active ON discount_templates(is_active)",
        [],
    )?;

    Ok(())
}

// Hilfsfunktion zum Einfügen der Standard-Email-Templates
fn insert_default_email_templates(conn: &Connection) -> Result<()> {
    // Prüfe ob bereits Templates existieren
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM email_templates",
        [],
        |row| row.get(0)
    )?;

    if count == 0 {
        // Buchungsbestätigung Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "confirmation",
                "Buchungsbestätigung - Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\n\
                vielen Dank für Ihre Buchung bei {firma_name}!\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name} ({zimmer_typ})\n\
                Ort: {zimmer_ort}\n\
                Check-in: {checkin_date}\n\
                Check-out: {checkout_date}\n\
                Anzahl Gäste: {anzahl_gaeste}\n\
                Anzahl Nächte: {anzahl_naechte}\n\
                Gesamtpreis: {gesamtpreis} €\n\n\
                Bei Fragen erreichen Sie uns unter:\n\
                Telefon: {firma_telefon}\n\
                Email: {firma_email}\n\n\
                Wir freuen uns auf Ihren Besuch!\n\n\
                Mit freundlichen Grüßen\n\
                {firma_name}",
                "Standard-Template für Buchungsbestätigungen"
            ],
        )?;

        // Check-in Reminder Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "reminder",
                "Erinnerung - Check-in morgen für Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\n\
                wir möchten Sie daran erinnern, dass Ihr Check-in morgen ansteht.\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Ort: {zimmer_ort}\n\
                Check-in: {checkin_date}\n\
                Check-out: {checkout_date}\n\
                Schlüsselcode: {schluesselcode}\n\n\
                Bei Fragen erreichen Sie uns unter:\n\
                Telefon: {firma_telefon}\n\
                Email: {firma_email}\n\n\
                Wir freuen uns auf Ihren Besuch!\n\n\
                Mit freundlichen Grüßen\n\
                {firma_name}",
                "Reminder-Email einen Tag vor Check-in"
            ],
        )?;

        // Rechnung Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "invoice",
                "Rechnung - Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\n\
                anbei erhalten Sie die Rechnung für Ihren Aufenthalt.\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Zeitraum: {checkin_date} - {checkout_date}\n\
                Anzahl Nächte: {anzahl_naechte}\n\n\
                Grundpreis: {grundpreis} €\n\
                Zusatzleistungen: {services_preis} €\n\
                Rabatt: -{rabatt_preis} €\n\
                Gesamtpreis: {gesamtpreis} €\n\n\
                Bitte überweisen Sie den Betrag an:\n\
                {firma_bank}\n\
                IBAN: {firma_iban}\n\
                BIC: {firma_bic}\n\n\
                Vielen Dank für Ihren Aufenthalt!\n\n\
                Mit freundlichen Grüßen\n\
                {firma_name}\n\
                {firma_adresse}\n\
                {firma_plz} {firma_ort}\n\
                Steuernummer: {firma_steuernummer}",
                "Rechnungs-Email nach Check-out"
            ],
        )?;

        // Zahlungserinnerung Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "payment_reminder",
                "Zahlungserinnerung - Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\n\
                wir möchten Sie freundlich daran erinnern, dass die Zahlung für Ihre Buchung noch aussteht.\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Zeitraum: {checkin_date} - {checkout_date}\n\
                Gesamtbetrag: {gesamtpreis} €\n\n\
                Bitte überweisen Sie den Betrag innerhalb der nächsten 7 Tage an:\n\
                {firma_bank}\n\
                IBAN: {firma_iban}\n\
                BIC: {firma_bic}\n\n\
                Bei Fragen stehen wir Ihnen gerne zur Verfügung:\n\
                Telefon: {firma_telefon}\n\
                Email: {firma_email}\n\n\
                Mit freundlichen Grüßen\n\
                {firma_name}",
                "Zahlungserinnerung nach 14 Tagen"
            ],
        )?;

        // Stornierungsbestätigung Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "cancellation",
                "Stornierungsbestätigung - Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\n\
                wir bestätigen die Stornierung Ihrer Buchung.\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Ursprünglicher Zeitraum: {checkin_date} - {checkout_date}\n\
                Storniert am: {heute}\n\n\
                Falls Sie erneut buchen möchten, kontaktieren Sie uns gerne:\n\
                Telefon: {firma_telefon}\n\
                Email: {firma_email}\n\n\
                Mit freundlichen Grüßen\n\
                {firma_name}",
                "Bestätigung bei Stornierung"
            ],
        )?;
    }

    Ok(())
}

// Hilfsfunktion zum Einfügen der Standard-Settings (Company + Payment)
fn insert_default_settings(conn: &Connection) -> Result<()> {
    // Prüfe ob bereits Company Settings existieren
    let company_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM company_settings",
        [],
        |row| row.get(0)
    )?;

    if company_count == 0 {
        // Standard Company Settings einfügen (aus pdf_generator.rs)
        conn.execute(
            "INSERT INTO company_settings (
                id, company_name, street_address, plz, city, country,
                phone, fax, email, website, tax_id, ceo_name, registry_court
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            [
                "1", // id fixiert auf 1
                "Stiftung der Deutschen Polizeigewerkschaft",
                "Wackersberger Str. 12",
                "83661",
                "Lenggries",
                "Deutschland",
                "+49 8042 9725-20",
                "+49 8042 9725-22",
                "info@dpolg-stiftung.de",
                "www.dpolg-stiftung.de",
                "141/239/71040",
                "Herr Reinhold Merl",
                "München"
            ],
        )?;
        println!("✅ Default Company Settings eingefügt");
    }

    // Prüfe ob bereits Payment Settings existieren
    let payment_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM payment_settings",
        [],
        |row| row.get(0)
    )?;

    if payment_count == 0 {
        // Standard Payment Settings einfügen
        conn.execute(
            "INSERT INTO payment_settings (
                id, bank_name, iban, bic, account_holder,
                mwst_rate, dpolg_rabatt, payment_due_days, reminder_after_days
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                1, // id fixiert auf 1
                "Sparda Bank München",
                "DE70 7009 0500 0001 9999 90",
                "GENODEF1S04",
                "Stiftung der Deutschen Polizeigewerkschaft",
                7.0,  // 7% MwSt
                15.0, // 15% DPolG-Rabatt
                14,   // Zahlungsziel 14 Tage
                14    // Mahnung nach 14 Tagen
            ],
        )?;
        println!("✅ Default Payment Settings eingefügt (mit 15% DPolG-Rabatt)");
    }

    Ok(())
}

fn insert_sample_data(conn: &Connection) -> Result<()> {
    // Check if data already exists
    let room_count: i64 = conn.query_row("SELECT COUNT(*) FROM rooms", [], |row| row.get(0))?;

    if room_count == 0 {
        // Insert sample rooms
        let rooms = vec![
            ("Zimmer 101", "Hauptgebäude", 2, 45.0, 65.0, "Berlin"),
            ("Zimmer 102", "Hauptgebäude", 2, 45.0, 65.0, "Berlin"),
            ("Zimmer 103", "Hauptgebäude", 1, 35.0, 55.0, "Berlin"),
            ("Zimmer 201", "Nebengebäude", 3, 55.0, 75.0, "München"),
            ("Zimmer 202", "Nebengebäude", 2, 45.0, 65.0, "München"),
            ("Suite 301", "Hauptgebäude", 4, 85.0, 110.0, "Berlin"),
        ];

        for (name, typ, capacity, price_m, price_nm, ort) in rooms {
            conn.execute(
                "INSERT INTO rooms (name, gebaeude_typ, capacity, price_member, price_non_member, ort, schluesselcode)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                [name, typ, &capacity.to_string(), &price_m.to_string(), &price_nm.to_string(), ort, "1234"],
            )?;
        }

        // Insert sample guests
        let guests = vec![
            ("Max", "Mustermann", "max@example.com", "+49 123 456789", true),
            ("Anna", "Schmidt", "anna@example.com", "+49 987 654321", true),
            ("Peter", "Müller", "peter@example.com", "+49 555 123456", false),
            ("Lisa", "Weber", "lisa@example.com", "+49 444 567890", true),
        ];

        for (vorname, nachname, email, telefon, mitglied) in guests {
            conn.execute(
                "INSERT INTO guests (vorname, nachname, email, telefon, dpolg_mitglied)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![vorname, nachname, email, telefon, mitglied as i32],
            )?;
        }

        // Insert sample bookings
        let bookings = vec![
            (1, 1, "RES-2025-001", "2025-10-01", "2025-10-05", 2, "bestaetigt", 180.0),
            (2, 2, "RES-2025-002", "2025-10-03", "2025-10-07", 2, "bestaetigt", 180.0),
            (3, 3, "RES-2025-003", "2025-10-10", "2025-10-12", 1, "reserviert", 70.0),
            (4, 4, "RES-2025-004", "2025-10-15", "2025-10-20", 2, "bestaetigt", 225.0),
        ];

        for (room_id, guest_id, res_nr, checkin, checkout, gaeste, status, preis) in bookings {
            conn.execute(
                "INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date, anzahl_gaeste, status, gesamtpreis)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                [&room_id.to_string(), &guest_id.to_string(), res_nr, checkin, checkout, &gaeste.to_string(), status, &preis.to_string()],
            )?;
        }
    }

    Ok(())
}

/// Migration: Lösche alle Zimmer und erstelle Zimmer aus Preisliste 2025
pub fn migrate_to_price_list_2025() -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    println!("🔄 Migration zu Preisliste 2025 gestartet...");

    // Lösche alle existierenden Zimmer (nur wenn keine Buchungen existieren!)
    let booking_count: i64 = conn.query_row("SELECT COUNT(*) FROM bookings", [], |row| row.get(0))?;

    if booking_count > 0 {
        return Err(rusqlite::Error::InvalidQuery);  // Verhindere Datenverlust
    }

    conn.execute("DELETE FROM rooms", [])?;
    println!("  ✅ Alte Zimmer gelöscht");

    // Zimmer aus Preisliste 2025 (basierend auf Image)
    // Format: (id, name, typ, personen, price_m, price_nm, nebensaison, hauptsaison, endreinigung, ort)

    // Fall (Zimmer 1-10)
    let fall_rooms = vec![
        (1, "1 - Apartment (BG)", "App", 2, 51.00, 60.00, 51.00, 59.50, 40.00, "Fall"),
        (2, "2 - Einzelzimmer", "EZ", 1, 25.50, 30.00, 25.50, 34.00, 20.00, "Fall"),
        (3, "3 - Einzelzimmer", "EZ", 1, 29.75, 35.00, 29.75, 38.25, 25.00, "Fall"),
        (4, "4 - Doppelzimmer", "DZ", 2, 42.50, 50.00, 42.50, 51.00, 35.00, "Fall"),
        (5, "5 - Apartment", "App", 2, 51.00, 60.00, 51.00, 59.50, 40.00, "Fall"),
        (6, "6 - Apartment", "App", 2, 51.00, 60.00, 51.00, 59.50, 40.00, "Fall"),
        (7, "7 - Apartment", "App", 2, 51.00, 60.00, 51.00, 59.50, 40.00, "Fall"),
        (8, "8 - Bungalow", "BU", 4, 85.00, 100.00, 85.00, 93.50, 60.00, "Fall"),
        (9, "9 - Bungalow", "BU", 6, 93.50, 110.00, 93.50, 106.25, 65.00, "Fall"),
        (10, "10 - Bungalow (BG)", "BU", 6, 102.00, 120.00, 102.00, 114.75, 65.00, "Fall"),
    ];

    // Lenggries (Zimmer 11-15)
    let lenggries_rooms = vec![
        (11, "11 - Ferienwohnung", "FeWo", 5, 80.75, 95.00, 80.75, 89.25, 60.00, "Lenggries"),
        (12, "12 - Ferienwohnung", "FeWo", 5, 89.25, 105.00, 89.25, 102.00, 60.00, "Lenggries"),
        (13, "13 - Ferienwohnung", "FeWo", 4, 59.50, 70.00, 59.50, 76.50, 55.00, "Lenggries"),
        (14, "14 - Ferienwohnung", "FeWo", 4, 59.50, 70.00, 59.50, 76.50, 55.00, "Lenggries"),
        (15, "15 - Ferienwohnung", "FeWo", 4, 85.00, 100.00, 85.00, 93.50, 60.00, "Lenggries"),
    ];

    // Brauneckblick (Zimmer 16)
    let brauneckblick_rooms = vec![
        (16, "16 - Ferienwohnung", "FeWo", 4, 85.00, 100.00, 85.00, 93.50, 60.00, "Brauneckblick"),
    ];

    // Alle Zimmer zusammenführen
    let all_rooms = [fall_rooms, lenggries_rooms, brauneckblick_rooms].concat();

    for (id, name, typ, personen, price_m, price_nm, nebensaison, hauptsaison, endreinigung, ort) in all_rooms {
        conn.execute(
            "INSERT INTO rooms (id, name, gebaeude_typ, capacity, price_member, price_non_member, nebensaison_preis, hauptsaison_preis, endreinigung, ort)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![id, name, typ, personen, price_m, price_nm, nebensaison, hauptsaison, endreinigung, ort],
        )?;
    }

    println!("  ✅ 16 neue Zimmer aus Preisliste 2025 eingefügt");
    println!("✅ Migration abgeschlossen!");

    Ok(())
}

pub fn get_rooms() -> Result<Vec<Room>> {
    let conn = Connection::open(get_db_path())?;

    // WICHTIG: Foreign Keys aktivieren für diese Connection
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare("SELECT id, name, gebaeude_typ, capacity, price_member, price_non_member, nebensaison_preis, hauptsaison_preis, endreinigung, ort, schluesselcode FROM rooms")?;

    let rooms = stmt.query_map([], |row| {
        Ok(Room {
            id: row.get(0)?,
            name: row.get(1)?,
            gebaeude_typ: row.get(2)?,
            capacity: row.get(3)?,
            price_member: row.get(4)?,
            price_non_member: row.get(5)?,
            nebensaison_preis: row.get(6)?,
            hauptsaison_preis: row.get(7)?,
            endreinigung: row.get(8)?,
            ort: row.get(9)?,
            schluesselcode: row.get(10)?,
        })
    })?;

    rooms.collect()
}

pub fn get_bookings_with_details() -> Result<Vec<BookingWithDetails>> {
    let conn = Connection::open(get_db_path())?;

    // WICHTIG: Foreign Keys aktivieren für diese Connection
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT
            b.id, b.room_id, b.guest_id, b.reservierungsnummer,
            b.checkin_date, b.checkout_date, b.anzahl_gaeste, b.anzahl_begleitpersonen,
            b.status, b.grundpreis, b.services_preis, b.rabatt_preis, b.gesamtpreis, b.anzahl_naechte, b.bemerkungen,
            b.bezahlt, b.bezahlt_am, b.zahlungsmethode, b.mahnung_gesendet_am,
            r.id, r.name, r.gebaeude_typ, r.capacity, r.price_member, r.price_non_member, r.nebensaison_preis, r.hauptsaison_preis, r.endreinigung, r.ort, r.schluesselcode,
            g.id, g.vorname, g.nachname, g.email, g.telefon, g.dpolg_mitglied,
            g.strasse, g.plz, g.ort, g.mitgliedsnummer, g.notizen, g.beruf, g.bundesland, g.dienststelle, g.created_at
         FROM bookings b
         JOIN rooms r ON b.room_id = r.id
         JOIN guests g ON b.guest_id = g.id
         ORDER BY b.id DESC"
    )?;

    let bookings = stmt.query_map([], |row| {
        Ok(BookingWithDetails {
            id: row.get(0)?,
            room_id: row.get(1)?,
            guest_id: row.get(2)?,
            reservierungsnummer: row.get(3)?,
            checkin_date: row.get(4)?,
            checkout_date: row.get(5)?,
            anzahl_gaeste: row.get(6)?,
            anzahl_begleitpersonen: row.get(7)?,
            status: row.get(8)?,
            grundpreis: row.get(9)?,
            services_preis: row.get(10)?,
            rabatt_preis: row.get(11)?,
            gesamtpreis: row.get(12)?,
            anzahl_naechte: row.get(13)?,
            bemerkungen: row.get(14)?,
            bezahlt: row.get::<_, i32>(15)? != 0,
            bezahlt_am: row.get(16)?,
            zahlungsmethode: row.get(17)?,
            mahnung_gesendet_am: row.get(18)?,
            room: Room {
                id: row.get(19)?,
                name: row.get(20)?,
                gebaeude_typ: row.get(21)?,
                capacity: row.get(22)?,
                price_member: row.get(23)?,
                price_non_member: row.get(24)?,
                nebensaison_preis: row.get(25)?,
                hauptsaison_preis: row.get(26)?,
                endreinigung: row.get(27)?,
                ort: row.get(28)?,
                schluesselcode: row.get(29)?,
            },
            guest: Guest {
                id: row.get(30)?,
                vorname: row.get(31)?,
                nachname: row.get(32)?,
                email: row.get(33)?,
                telefon: row.get(34)?,
                dpolg_mitglied: row.get(35)?,
                strasse: row.get(36)?,
                plz: row.get(37)?,
                ort: row.get(38)?,
                mitgliedsnummer: row.get(39)?,
                notizen: row.get(40)?,
                beruf: row.get(41)?,
                bundesland: row.get(42)?,
                dienststelle: row.get(43)?,
                created_at: row.get(44)?,
            },
        })
    })?;

    bookings.collect()
}

// ============================================================================
// GUEST MANAGEMENT - Phase 1.2
// ============================================================================

/// Erstellt einen neuen Gast in der Datenbank
pub fn create_guest(
    vorname: String,
    nachname: String,
    email: String,
    telefon: String,
    dpolg_mitglied: bool,
    strasse: Option<String>,
    plz: Option<String>,
    ort: Option<String>,
    mitgliedsnummer: Option<String>,
    notizen: Option<String>,
    beruf: Option<String>,
    bundesland: Option<String>,
    dienststelle: Option<String>,
) -> Result<Guest> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO guests (vorname, nachname, email, telefon, dpolg_mitglied, strasse, plz, ort, mitgliedsnummer, notizen, beruf, bundesland, dienststelle)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        rusqlite::params![
            vorname,
            nachname,
            email,
            telefon,
            dpolg_mitglied as i32,
            strasse,
            plz,
            ort,
            mitgliedsnummer,
            notizen,
            beruf,
            bundesland,
            dienststelle
        ],
    )?;

    let id = conn.last_insert_rowid();
    let guest = get_guest_by_id(id)?;

    // Transaction Log: CREATE
    let guest_json = serde_json::to_string(&guest).unwrap_or_default();
    let user_action = format!("Gast {} {} erstellt", guest.vorname, guest.nachname);
    if let Err(e) = crate::transaction_log::log_create("guests", id, &guest_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(guest)
}

/// Aktualisiert einen existierenden Gast
pub fn update_guest(
    id: i64,
    vorname: String,
    nachname: String,
    email: String,
    telefon: String,
    dpolg_mitglied: bool,
    strasse: Option<String>,
    plz: Option<String>,
    ort: Option<String>,
    mitgliedsnummer: Option<String>,
    notizen: Option<String>,
    beruf: Option<String>,
    bundesland: Option<String>,
    dienststelle: Option<String>,
) -> Result<Guest> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log
    let old_guest = get_guest_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_guest).unwrap_or_default();

    let rows_affected = conn.execute(
        "UPDATE guests SET
         vorname = ?1, nachname = ?2, email = ?3, telefon = ?4, dpolg_mitglied = ?5,
         strasse = ?6, plz = ?7, ort = ?8, mitgliedsnummer = ?9, notizen = ?10,
         beruf = ?11, bundesland = ?12, dienststelle = ?13
         WHERE id = ?14",
        rusqlite::params![
            vorname,
            nachname,
            email,
            telefon,
            dpolg_mitglied as i32,
            strasse,
            plz,
            ort,
            mitgliedsnummer,
            notizen,
            beruf,
            bundesland,
            dienststelle,
            id
        ],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let updated_guest = get_guest_by_id(id)?;

    // Transaction Log: UPDATE
    let new_data_json = serde_json::to_string(&updated_guest).unwrap_or_default();
    let user_action = format!("Gast {} {} aktualisiert", updated_guest.vorname, updated_guest.nachname);
    if let Err(e) = crate::transaction_log::log_update("guests", id, &old_data_json, &new_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(updated_guest)
}

/// Löscht einen Gast (schlägt fehl, wenn der Gast Buchungen hat)
pub fn delete_guest(id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log BEFORE deleting
    let old_guest = get_guest_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_guest).unwrap_or_default();
    let guest_name = format!("{} {}", old_guest.vorname, old_guest.nachname);

    let rows_affected = conn.execute("DELETE FROM guests WHERE id = ?1", rusqlite::params![id])?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    // Transaction Log: DELETE
    let user_action = format!("Gast {} gelöscht", guest_name);
    if let Err(e) = crate::transaction_log::log_delete("guests", id, &old_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(())
}

/// Gibt einen Gast anhand der ID zurück
pub fn get_guest_by_id(id: i64) -> Result<Guest> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, vorname, nachname, email, telefon, dpolg_mitglied, strasse, plz, ort, mitgliedsnummer, notizen, beruf, bundesland, dienststelle, created_at
         FROM guests WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Guest {
                id: row.get(0)?,
                vorname: row.get(1)?,
                nachname: row.get(2)?,
                email: row.get(3)?,
                telefon: row.get(4)?,
                dpolg_mitglied: row.get(5)?,
                strasse: row.get(6)?,
                plz: row.get(7)?,
                ort: row.get(8)?,
                mitgliedsnummer: row.get(9)?,
                notizen: row.get(10)?,
                beruf: row.get(11)?,
                bundesland: row.get(12)?,
                dienststelle: row.get(13)?,
                created_at: row.get(14)?,
            })
        },
    )
}

/// Sucht Gäste nach Name oder Email
pub fn search_guests(query: String) -> Result<Vec<Guest>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, vorname, nachname, email, telefon, dpolg_mitglied, strasse, plz, ort, mitgliedsnummer, notizen, beruf, bundesland, dienststelle, created_at
         FROM guests
         WHERE vorname LIKE ?1 OR nachname LIKE ?1 OR email LIKE ?1
         ORDER BY nachname, vorname",
    )?;

    let guests = stmt.query_map([&pattern], |row| {
        Ok(Guest {
            id: row.get(0)?,
            vorname: row.get(1)?,
            nachname: row.get(2)?,
            email: row.get(3)?,
            telefon: row.get(4)?,
            dpolg_mitglied: row.get(5)?,
            strasse: row.get(6)?,
            plz: row.get(7)?,
            ort: row.get(8)?,
            mitgliedsnummer: row.get(9)?,
            notizen: row.get(10)?,
            beruf: row.get(11)?,
            bundesland: row.get(12)?,
            dienststelle: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;

    guests.collect()
}

/// Gibt alle Gäste zurück
pub fn get_all_guests() -> Result<Vec<Guest>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT id, vorname, nachname, email, telefon, dpolg_mitglied, strasse, plz, ort, mitgliedsnummer, notizen, beruf, bundesland, dienststelle, created_at
         FROM guests
         ORDER BY nachname, vorname",
    )?;

    let guests = stmt.query_map([], |row| {
        Ok(Guest {
            id: row.get(0)?,
            vorname: row.get(1)?,
            nachname: row.get(2)?,
            email: row.get(3)?,
            telefon: row.get(4)?,
            dpolg_mitglied: row.get(5)?,
            strasse: row.get(6)?,
            plz: row.get(7)?,
            ort: row.get(8)?,
            mitgliedsnummer: row.get(9)?,
            notizen: row.get(10)?,
            beruf: row.get(11)?,
            bundesland: row.get(12)?,
            dienststelle: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;

    guests.collect()
}

// ============================================================================
// ROOM MANAGEMENT - Phase 1.2
// ============================================================================

/// Erstellt einen neuen Raum
pub fn create_room(
    name: String,
    gebaeude_typ: String,
    capacity: i32,
    price_member: f64,
    price_non_member: f64,
    nebensaison_preis: f64,
    hauptsaison_preis: f64,
    endreinigung: f64,
    ort: String,
    schluesselcode: Option<String>,
) -> Result<Room> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO rooms (name, gebaeude_typ, capacity, price_member, price_non_member, nebensaison_preis, hauptsaison_preis, endreinigung, ort, schluesselcode)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            name,
            gebaeude_typ,
            capacity,
            price_member,
            price_non_member,
            nebensaison_preis,
            hauptsaison_preis,
            endreinigung,
            ort,
            schluesselcode
        ],
    )?;

    let id = conn.last_insert_rowid();
    let room = get_room_by_id(id)?;

    // Transaction Log: CREATE
    let room_json = serde_json::to_string(&room).unwrap_or_default();
    let user_action = format!("Zimmer {} erstellt", room.name);
    if let Err(e) = crate::transaction_log::log_create("rooms", id, &room_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(room)
}

/// Aktualisiert einen existierenden Raum
pub fn update_room(
    id: i64,
    name: String,
    gebaeude_typ: String,
    capacity: i32,
    price_member: f64,
    price_non_member: f64,
    nebensaison_preis: f64,
    hauptsaison_preis: f64,
    endreinigung: f64,
    ort: String,
    schluesselcode: Option<String>,
) -> Result<Room> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log
    let old_room = get_room_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_room).unwrap_or_default();

    let rows_affected = conn.execute(
        "UPDATE rooms SET
         name = ?1, gebaeude_typ = ?2, capacity = ?3, price_member = ?4,
         price_non_member = ?5, nebensaison_preis = ?6, hauptsaison_preis = ?7,
         endreinigung = ?8, ort = ?9, schluesselcode = ?10
         WHERE id = ?11",
        rusqlite::params![
            name,
            gebaeude_typ,
            capacity,
            price_member,
            price_non_member,
            nebensaison_preis,
            hauptsaison_preis,
            endreinigung,
            ort,
            schluesselcode,
            id
        ],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let updated_room = get_room_by_id(id)?;

    // Transaction Log: UPDATE
    let new_data_json = serde_json::to_string(&updated_room).unwrap_or_default();
    let user_action = format!("Zimmer {} aktualisiert", updated_room.name);
    if let Err(e) = crate::transaction_log::log_update("rooms", id, &old_data_json, &new_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(updated_room)
}

/// Löscht einen Raum (schlägt fehl, wenn der Raum Buchungen hat)
pub fn delete_room(id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log BEFORE deleting
    let old_room = get_room_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_room).unwrap_or_default();
    let room_name = old_room.name.clone();

    let rows_affected = conn.execute("DELETE FROM rooms WHERE id = ?1", rusqlite::params![id])?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    // Transaction Log: DELETE
    let user_action = format!("Zimmer {} gelöscht", room_name);
    if let Err(e) = crate::transaction_log::log_delete("rooms", id, &old_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(())
}

/// Gibt einen Raum anhand der ID zurück
pub fn get_room_by_id(id: i64) -> Result<Room> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, name, gebaeude_typ, capacity, price_member, price_non_member, nebensaison_preis, hauptsaison_preis, endreinigung, ort, schluesselcode
         FROM rooms WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Room {
                id: row.get(0)?,
                name: row.get(1)?,
                gebaeude_typ: row.get(2)?,
                capacity: row.get(3)?,
                price_member: row.get(4)?,
                price_non_member: row.get(5)?,
                nebensaison_preis: row.get(6)?,
                hauptsaison_preis: row.get(7)?,
                endreinigung: row.get(8)?,
                ort: row.get(9)?,
                schluesselcode: row.get(10)?,
            })
        },
    )
}

// ============================================================================
// BOOKING MANAGEMENT - Phase 1.2
// ============================================================================

/// Erstellt eine neue Buchung
pub fn create_booking(
    room_id: i64,
    guest_id: i64,
    reservierungsnummer: String,
    checkin_date: String,
    checkout_date: String,
    anzahl_gaeste: i32,
    status: String,
    gesamtpreis: f64,
    bemerkungen: Option<String>,
    anzahl_begleitpersonen: i32,
    grundpreis: f64,
    services_preis: f64,
    rabatt_preis: f64,
    anzahl_naechte: i32,
) -> Result<Booking> {
    println!("🔍 create_booking() aufgerufen - Transaction Logging aktiviert");
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Erst einfügen mit temporärer Reservierungsnummer
    conn.execute(
        "INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
         anzahl_gaeste, status, gesamtpreis, bemerkungen, anzahl_begleitpersonen,
         grundpreis, services_preis, rabatt_preis, anzahl_naechte)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            room_id,
            guest_id,
            "TEMP", // Temporäre Nummer
            checkin_date,
            checkout_date,
            anzahl_gaeste,
            status,
            gesamtpreis,
            bemerkungen,
            anzahl_begleitpersonen,
            grundpreis,
            services_preis,
            rabatt_preis,
            anzahl_naechte
        ],
    )?;

    let id = conn.last_insert_rowid();

    // Generiere Reservierungsnummer basierend auf ID und Jahr
    let final_reservierungsnummer = crate::validation::generate_reservation_number_with_id(id);

    // Update mit finaler Reservierungsnummer
    conn.execute(
        "UPDATE bookings SET reservierungsnummer = ?1 WHERE id = ?2",
        rusqlite::params![final_reservierungsnummer, id],
    )?;

    let booking = get_booking_by_id(id)?;

    // Transaction Log: CREATE
    let booking_json = serde_json::to_string(&booking).unwrap_or_default();
    let user_action = format!("Buchung {} erstellt", booking.reservierungsnummer);
    if let Err(e) = crate::transaction_log::log_create("bookings", id, &booking_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(booking)
}

/// Aktualisiert eine existierende Buchung
pub fn update_booking(
    id: i64,
    room_id: i64,
    guest_id: i64,
    checkin_date: String,
    checkout_date: String,
    anzahl_gaeste: i32,
    status: String,
    gesamtpreis: f64,
    bemerkungen: Option<String>,
    anzahl_begleitpersonen: i32,
    grundpreis: f64,
    services_preis: f64,
    rabatt_preis: f64,
    anzahl_naechte: i32,
) -> Result<Booking> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log
    let old_booking = get_booking_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_booking).unwrap_or_default();

    let rows_affected = conn.execute(
        "UPDATE bookings SET
         room_id = ?1, guest_id = ?2, checkin_date = ?3, checkout_date = ?4,
         anzahl_gaeste = ?5, status = ?6, gesamtpreis = ?7, bemerkungen = ?8,
         anzahl_begleitpersonen = ?9, grundpreis = ?10, services_preis = ?11,
         rabatt_preis = ?12, anzahl_naechte = ?13, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?14",
        rusqlite::params![
            room_id,
            guest_id,
            checkin_date,
            checkout_date,
            anzahl_gaeste,
            status,
            gesamtpreis,
            bemerkungen,
            anzahl_begleitpersonen,
            grundpreis,
            services_preis,
            rabatt_preis,
            anzahl_naechte,
            id
        ],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let updated_booking = get_booking_by_id(id)?;

    // Transaction Log: UPDATE
    let new_data_json = serde_json::to_string(&updated_booking).unwrap_or_default();
    let user_action = format!("Buchung {} aktualisiert", updated_booking.reservierungsnummer);
    if let Err(e) = crate::transaction_log::log_update("bookings", id, &old_data_json, &new_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(updated_booking)
}

/// Löscht eine Buchung (CASCADE löscht auch Services, Gäste, Rabatte)
pub fn delete_booking(id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log BEFORE deleting
    let old_booking = get_booking_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_booking).unwrap_or_default();
    let reservierungsnummer = old_booking.reservierungsnummer.clone();

    let rows_affected = conn.execute("DELETE FROM bookings WHERE id = ?1", rusqlite::params![id])?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    // Transaction Log: DELETE
    let user_action = format!("Buchung {} gelöscht", reservierungsnummer);
    if let Err(e) = crate::transaction_log::log_delete("bookings", id, &old_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(())
}

/// Aktualisiert nur Zimmer und Daten einer Buchung (für Drag & Drop im TapeChart)
pub fn update_booking_dates_and_room(
    id: i64,
    room_id: i64,
    checkin_date: String,
    checkout_date: String,
) -> Result<Booking> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log
    let old_booking = get_booking_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_booking).unwrap_or_default();

    let rows_affected = conn.execute(
        "UPDATE bookings SET
         room_id = ?1, checkin_date = ?2, checkout_date = ?3, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?4",
        rusqlite::params![room_id, checkin_date, checkout_date, id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let updated_booking = get_booking_by_id(id)?;

    // Transaction Log: UPDATE
    let new_data_json = serde_json::to_string(&updated_booking).unwrap_or_default();
    let user_action = format!("Buchung {} verschoben (Drag & Drop)", updated_booking.reservierungsnummer);
    if let Err(e) = crate::transaction_log::log_update("bookings", id, &old_data_json, &new_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(updated_booking)
}

/// Storniert eine Buchung (setzt Status auf 'storniert')
pub fn cancel_booking(id: i64) -> Result<Booking> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute(
        "UPDATE bookings SET status = 'storniert', updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_booking_by_id(id)
}

/// Markiert eine Buchung als bezahlt
pub fn mark_booking_as_paid(id: i64, zahlungsmethode: String) -> Result<Booking> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let bezahlt_am = crate::time_utils::format_today_db();

    let rows_affected = conn.execute(
        "UPDATE bookings SET bezahlt = 1, bezahlt_am = ?1, zahlungsmethode = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        rusqlite::params![bezahlt_am, zahlungsmethode, id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_booking_by_id(id)
}

/// Ändert den Status einer Buchung
pub fn update_booking_status(id: i64, new_status: String) -> Result<Booking> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log
    let old_booking = get_booking_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_booking).unwrap_or_default();

    // Validiere Status
    let valid_statuses = ["reserviert", "bestaetigt", "eingecheckt", "ausgecheckt", "storniert"];
    if !valid_statuses.contains(&new_status.as_str()) {
        return Err(rusqlite::Error::InvalidParameterName(format!("Ungültiger Status: {}", new_status)));
    }

    let rows_affected = conn.execute(
        "UPDATE bookings SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        rusqlite::params![new_status, id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let updated_booking = get_booking_by_id(id)?;

    // Transaction Log: UPDATE
    let new_data_json = serde_json::to_string(&updated_booking).unwrap_or_default();
    let user_action = format!("Buchung {} Status geändert: {} → {}", updated_booking.reservierungsnummer, old_booking.status, new_status);
    if let Err(e) = crate::transaction_log::log_update("bookings", id, &old_data_json, &new_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(updated_booking)
}

/// Ändert den Bezahlt-Status einer Buchung
pub fn update_booking_payment(id: i64, bezahlt: bool, zahlungsmethode: Option<String>) -> Result<Booking> {
    println!("🔍 [update_booking_payment] Called:");
    println!("   id: {}", id);
    println!("   bezahlt: {}", bezahlt);
    println!("   zahlungsmethode: {:?}", zahlungsmethode);

    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log
    let old_booking = get_booking_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_booking).unwrap_or_default();

    let zahlungsmethode_string = zahlungsmethode.clone().unwrap_or_else(|| "Überweisung".to_string());

    if bezahlt {
        // Bezahlt markieren mit Zahlungsmethode
        let bezahlt_am = crate::time_utils::format_today_db();
        let rows_affected = conn.execute(
            "UPDATE bookings SET bezahlt = 1, bezahlt_am = ?1, zahlungsmethode = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            rusqlite::params![bezahlt_am, &zahlungsmethode_string, id],
        )?;

        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
    } else {
        // Zurück auf offen setzen
        let rows_affected = conn.execute(
            "UPDATE bookings SET bezahlt = 0, bezahlt_am = NULL, zahlungsmethode = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            rusqlite::params![id],
        )?;

        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
    }

    let updated_booking = get_booking_by_id(id)?;

    // Transaction Log: UPDATE
    let new_data_json = serde_json::to_string(&updated_booking).unwrap_or_default();
    let user_action = if bezahlt {
        format!("Buchung {} als bezahlt markiert ({})", updated_booking.reservierungsnummer, zahlungsmethode_string)
    } else {
        format!("Buchung {} Zahlungsstatus zurückgesetzt", updated_booking.reservierungsnummer)
    };
    if let Err(e) = crate::transaction_log::log_update("bookings", id, &old_data_json, &new_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(updated_booking)
}

/// Gibt eine Buchung anhand der ID zurück
pub fn get_booking_by_id(id: i64) -> Result<Booking> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
         anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at, anzahl_begleitpersonen,
         grundpreis, services_preis, rabatt_preis, anzahl_naechte, updated_at,
         bezahlt, bezahlt_am, zahlungsmethode, mahnung_gesendet_am
         FROM bookings WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Booking {
                id: row.get(0)?,
                room_id: row.get(1)?,
                guest_id: row.get(2)?,
                reservierungsnummer: row.get(3)?,
                checkin_date: row.get(4)?,
                checkout_date: row.get(5)?,
                anzahl_gaeste: row.get(6)?,
                status: row.get(7)?,
                gesamtpreis: row.get(8)?,
                bemerkungen: row.get(9)?,
                created_at: row.get(10)?,
                anzahl_begleitpersonen: row.get(11)?,
                grundpreis: row.get(12)?,
                services_preis: row.get(13)?,
                rabatt_preis: row.get(14)?,
                anzahl_naechte: row.get(15)?,
                updated_at: row.get(16)?,
                bezahlt: row.get::<_, i32>(17)? != 0,
                bezahlt_am: row.get(18)?,
                zahlungsmethode: row.get(19)?,
                mahnung_gesendet_am: row.get(20)?,
            })
        },
    )
}

/// Gibt eine Buchung mit allen Details (Room + Guest) anhand der ID zurück
pub fn get_booking_with_details_by_id(id: i64) -> Result<BookingWithDetails> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT
            b.id, b.room_id, b.guest_id, b.reservierungsnummer,
            b.checkin_date, b.checkout_date, b.anzahl_gaeste, b.anzahl_begleitpersonen,
            b.status, b.grundpreis, b.services_preis, b.rabatt_preis, b.gesamtpreis, b.anzahl_naechte, b.bemerkungen,
            b.bezahlt, b.bezahlt_am, b.zahlungsmethode, b.mahnung_gesendet_am,
            r.id, r.name, r.gebaeude_typ, r.capacity, r.price_member, r.price_non_member,
            r.nebensaison_preis, r.hauptsaison_preis, r.endreinigung, r.ort, r.schluesselcode,
            g.id, g.vorname, g.nachname, g.email, g.telefon, g.dpolg_mitglied,
            g.strasse, g.plz, g.ort, g.mitgliedsnummer, g.notizen, g.beruf, g.bundesland, g.dienststelle, g.created_at
         FROM bookings b
         JOIN rooms r ON b.room_id = r.id
         JOIN guests g ON b.guest_id = g.id
         WHERE b.id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(BookingWithDetails {
                id: row.get(0)?,
                room_id: row.get(1)?,
                guest_id: row.get(2)?,
                reservierungsnummer: row.get(3)?,
                checkin_date: row.get(4)?,
                checkout_date: row.get(5)?,
                anzahl_gaeste: row.get(6)?,
                anzahl_begleitpersonen: row.get(7)?,
                status: row.get(8)?,
                grundpreis: row.get(9)?,
                services_preis: row.get(10)?,
                rabatt_preis: row.get(11)?,
                gesamtpreis: row.get(12)?,
                anzahl_naechte: row.get(13)?,
                bemerkungen: row.get(14)?,
                bezahlt: row.get::<_, i32>(15)? != 0,
                bezahlt_am: row.get(16)?,
                zahlungsmethode: row.get(17)?,
                mahnung_gesendet_am: row.get(18)?,
                room: Room {
                    id: row.get(19)?,
                    name: row.get(20)?,
                    gebaeude_typ: row.get(21)?,
                    capacity: row.get(22)?,
                    price_member: row.get(23)?,
                    price_non_member: row.get(24)?,
                    nebensaison_preis: row.get(25)?,
                    hauptsaison_preis: row.get(26)?,
                    endreinigung: row.get(27)?,
                    ort: row.get(28)?,
                    schluesselcode: row.get(29)?,
                },
                guest: Guest {
                    id: row.get(30)?,
                    vorname: row.get(31)?,
                    nachname: row.get(32)?,
                    email: row.get(33)?,
                    telefon: row.get(34)?,
                    dpolg_mitglied: row.get(35)?,
                    strasse: row.get(36)?,
                    plz: row.get(37)?,
                    ort: row.get(38)?,
                    mitgliedsnummer: row.get(39)?,
                    notizen: row.get(40)?,
                    beruf: row.get(41)?,
                    bundesland: row.get(42)?,
                    dienststelle: row.get(43)?,
                    created_at: row.get(44)?,
                },
            })
        },
    )
}

// ============================================================================
// ADDITIONAL SERVICES - Phase 1.2
// ============================================================================

/// Fügt einen Service zu einer Buchung hinzu
pub fn add_service_to_booking(
    booking_id: i64,
    service_name: String,
    service_price: f64,
) -> Result<AdditionalService> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO additional_services (booking_id, service_name, service_price)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![booking_id, service_name, service_price],
    )?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, booking_id, service_name, service_price, created_at
         FROM additional_services WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(AdditionalService {
                id: row.get(0)?,
                booking_id: row.get(1)?,
                service_name: row.get(2)?,
                service_price: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
}

/// Löscht einen Service
pub fn delete_service(service_id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute(
        "DELETE FROM additional_services WHERE id = ?1",
        rusqlite::params![service_id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
}

/// Gibt alle Services für eine Buchung zurück
pub fn get_booking_services(booking_id: i64) -> Result<Vec<AdditionalService>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT id, booking_id, service_name, service_price, created_at
         FROM additional_services
         WHERE booking_id = ?1
         ORDER BY created_at",
    )?;

    let services = stmt.query_map(rusqlite::params![booking_id], |row| {
        Ok(AdditionalService {
            id: row.get(0)?,
            booking_id: row.get(1)?,
            service_name: row.get(2)?,
            service_price: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    services.collect()
}

// ============================================================================
// ACCOMPANYING GUESTS - Phase 1.2
// ============================================================================

/// Fügt eine Begleitperson zu einer Buchung hinzu
pub fn add_accompanying_guest(
    booking_id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
) -> Result<AccompanyingGuest> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO accompanying_guests (booking_id, vorname, nachname, geburtsdatum)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![booking_id, vorname, nachname, geburtsdatum],
    )?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, booking_id, vorname, nachname, geburtsdatum
         FROM accompanying_guests WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(AccompanyingGuest {
                id: row.get(0)?,
                booking_id: row.get(1)?,
                vorname: row.get(2)?,
                nachname: row.get(3)?,
                geburtsdatum: row.get(4)?,
            })
        },
    )
}

/// Löscht eine Begleitperson
pub fn delete_accompanying_guest(guest_id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute(
        "DELETE FROM accompanying_guests WHERE id = ?1",
        rusqlite::params![guest_id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
}

/// Gibt alle Begleitpersonen für eine Buchung zurück
pub fn get_booking_accompanying_guests(booking_id: i64) -> Result<Vec<AccompanyingGuest>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT id, booking_id, vorname, nachname, geburtsdatum
         FROM accompanying_guests
         WHERE booking_id = ?1
         ORDER BY nachname, vorname",
    )?;

    let guests = stmt.query_map(rusqlite::params![booking_id], |row| {
        Ok(AccompanyingGuest {
            id: row.get(0)?,
            booking_id: row.get(1)?,
            vorname: row.get(2)?,
            nachname: row.get(3)?,
            geburtsdatum: row.get(4)?,
        })
    })?;

    guests.collect()
}

// ============================================================================
// DISCOUNTS - Phase 1.2
// ============================================================================

/// Fügt einen Rabatt zu einer Buchung hinzu
pub fn add_discount_to_booking(
    booking_id: i64,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
) -> Result<Discount> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO discounts (booking_id, discount_name, discount_type, discount_value)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![booking_id, discount_name, discount_type, discount_value],
    )?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, booking_id, discount_name, discount_type, discount_value, created_at
         FROM discounts WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Discount {
                id: row.get(0)?,
                booking_id: row.get(1)?,
                discount_name: row.get(2)?,
                discount_type: row.get(3)?,
                discount_value: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    )
}

/// Löscht einen Rabatt
pub fn delete_discount(discount_id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute(
        "DELETE FROM discounts WHERE id = ?1",
        rusqlite::params![discount_id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
}

/// Gibt alle Rabatte für eine Buchung zurück
pub fn get_booking_discounts(booking_id: i64) -> Result<Vec<Discount>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT id, booking_id, discount_name, discount_type, discount_value, created_at
         FROM discounts
         WHERE booking_id = ?1
         ORDER BY created_at",
    )?;

    let discounts = stmt.query_map(rusqlite::params![booking_id], |row| {
        Ok(Discount {
            id: row.get(0)?,
            booking_id: row.get(1)?,
            discount_name: row.get(2)?,
            discount_type: row.get(3)?,
            discount_value: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    discounts.collect()
}

// ============================================================================
// REPORTS & STATISTICS
// ============================================================================

/// Statistiken für Reports
#[derive(Debug, serde::Serialize)]
pub struct ReportStats {
    pub total_bookings: i32,
    pub active_bookings: i32,
    pub total_revenue: f64,
    pub total_nights: i32,
    pub average_price_per_night: f64,
    pub occupancy_rate: f64,
}

/// Belegung pro Zimmer
#[derive(Debug, serde::Serialize)]
pub struct RoomOccupancy {
    pub room_id: i64,
    pub room_name: String,
    pub total_bookings: i32,
    pub total_nights: i32,
    pub total_revenue: f64,
    pub occupancy_rate: f64,
}

/// Gibt Gesamt-Statistiken für einen Zeitraum zurück
pub fn get_report_stats(start_date: &str, end_date: &str) -> Result<ReportStats> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Total bookings (nicht storniert)
    let total_bookings: i32 = conn.query_row(
        "SELECT COUNT(*) FROM bookings
         WHERE status != 'storniert'
         AND checkin_date >= ?1 AND checkout_date <= ?2",
        rusqlite::params![start_date, end_date],
        |row| row.get(0),
    )?;

    // Active bookings (reserviert, bestätigt, eingecheckt)
    let active_bookings: i32 = conn.query_row(
        "SELECT COUNT(*) FROM bookings
         WHERE status IN ('reserviert', 'bestaetigt', 'eingecheckt')
         AND checkin_date >= ?1 AND checkout_date <= ?2",
        rusqlite::params![start_date, end_date],
        |row| row.get(0),
    )?;

    // Total revenue (nicht storniert)
    let total_revenue: f64 = conn.query_row(
        "SELECT COALESCE(SUM(gesamtpreis), 0.0) FROM bookings
         WHERE status != 'storniert'
         AND checkin_date >= ?1 AND checkout_date <= ?2",
        rusqlite::params![start_date, end_date],
        |row| row.get(0),
    )?;

    // Total nights
    let total_nights: i32 = conn.query_row(
        "SELECT COALESCE(SUM(anzahl_naechte), 0) FROM bookings
         WHERE status != 'storniert'
         AND checkin_date >= ?1 AND checkout_date <= ?2",
        rusqlite::params![start_date, end_date],
        |row| row.get(0),
    )?;

    // Average price per night
    let average_price_per_night = if total_nights > 0 {
        total_revenue / total_nights as f64
    } else {
        0.0
    };

    // Occupancy rate (simplified: booked nights / available nights)
    // Available nights = number of rooms × days in period
    let room_count: i32 = conn.query_row("SELECT COUNT(*) FROM rooms", [], |row| row.get(0))?;

    // Calculate days in period
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    let days_in_period = (end - start).num_days() as i32 + 1;

    let available_nights = room_count * days_in_period;
    let occupancy_rate = if available_nights > 0 {
        (total_nights as f64 / available_nights as f64) * 100.0
    } else {
        0.0
    };

    Ok(ReportStats {
        total_bookings,
        active_bookings,
        total_revenue,
        total_nights,
        average_price_per_night,
        occupancy_rate,
    })
}

/// Gibt Belegungsstatistiken pro Zimmer zurück
pub fn get_room_occupancy(start_date: &str, end_date: &str) -> Result<Vec<RoomOccupancy>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT
            r.id,
            r.name,
            COUNT(b.id) as total_bookings,
            COALESCE(SUM(b.anzahl_naechte), 0) as total_nights,
            COALESCE(SUM(b.gesamtpreis), 0.0) as total_revenue
         FROM rooms r
         LEFT JOIN bookings b ON r.id = b.room_id
            AND b.status != 'storniert'
            AND b.checkin_date >= ?1
            AND b.checkout_date <= ?2
         GROUP BY r.id, r.name
         ORDER BY total_revenue DESC",
    )?;

    // Calculate days in period for occupancy rate
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    let days_in_period = (end - start).num_days() as i32 + 1;

    let occupancies = stmt.query_map(rusqlite::params![start_date, end_date], |row| {
        let total_nights: i32 = row.get(3)?;
        let occupancy_rate = if days_in_period > 0 {
            (total_nights as f64 / days_in_period as f64) * 100.0
        } else {
            0.0
        };

        Ok(RoomOccupancy {
            room_id: row.get(0)?,
            room_name: row.get(1)?,
            total_bookings: row.get(2)?,
            total_nights,
            total_revenue: row.get(4)?,
            occupancy_rate,
        })
    })?;

    occupancies.collect()
}
/// Aktualisiert Buchungs-Status basierend auf Check-in/Check-out Daten
///
/// Logik:
/// - heute >= checkout_date → "ausgecheckt"
/// - heute >= checkin_date AND heute < checkout_date → "eingecheckt"
/// - Status "storniert" wird nicht geändert
pub fn update_booking_statuses_by_date() -> Result<usize> {
    let conn = Connection::open(get_db_path())?;

    // Aktuelles Datum in YYYY-MM-DD Format
    let heute = crate::time_utils::format_today_db();

    println!("📅 [database::update_booking_statuses_by_date] Heute: {}", heute);

    // 1. Ausgecheckt: heute >= checkout_date
    let ausgecheckt_count = conn.execute(
        "UPDATE bookings
         SET status = 'ausgecheckt'
         WHERE status IN ('bestaetigt', 'eingecheckt')
         AND checkout_date <= ?1",
        rusqlite::params![heute],
    )?;

    println!("✅ {} Buchungen auf 'ausgecheckt' gesetzt", ausgecheckt_count);

    // 2. Eingecheckt: heute >= checkin AND heute < checkout
    let eingecheckt_count = conn.execute(
        "UPDATE bookings
         SET status = 'eingecheckt'
         WHERE status = 'bestaetigt'
         AND checkin_date <= ?1
         AND checkout_date > ?1",
        rusqlite::params![heute],
    )?;

    println!("✅ {} Buchungen auf 'eingecheckt' gesetzt", eingecheckt_count);

    let total_changed = ausgecheckt_count + eingecheckt_count;
    println!("📊 Gesamt {} Status-Änderungen", total_changed);

    Ok(total_changed)
}

// ============================================================================
// COMPANY & PAYMENT SETTINGS - Phase 7
// ============================================================================

/// Gibt die Firmeneinstellungen zurück (id ist immer 1)
pub fn get_company_settings() -> Result<CompanySettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, company_name, street_address, plz, city, country,
         phone, fax, email, website, tax_id, ceo_name, registry_court, logo_path, updated_at
         FROM company_settings WHERE id = 1",
        [],
        |row| {
            Ok(CompanySettings {
                id: row.get(0)?,
                company_name: row.get(1)?,
                street_address: row.get(2)?,
                plz: row.get(3)?,
                city: row.get(4)?,
                country: row.get(5)?,
                phone: row.get(6)?,
                fax: row.get(7)?,
                email: row.get(8)?,
                website: row.get(9)?,
                tax_id: row.get(10)?,
                ceo_name: row.get(11)?,
                registry_court: row.get(12)?,
                logo_path: row.get(13)?,
                updated_at: row.get(14)?,
            })
        },
    )
}

/// Speichert/Aktualisiert die Firmeneinstellungen (id ist immer 1)
pub fn save_company_settings(settings: CompanySettings) -> Result<CompanySettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // UPSERT Pattern: INSERT OR REPLACE
    conn.execute(
        "INSERT OR REPLACE INTO company_settings (
            id, company_name, street_address, plz, city, country,
            phone, fax, email, website, tax_id, ceo_name, registry_court, logo_path,
            updated_at
        ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, CURRENT_TIMESTAMP)",
        rusqlite::params![
            settings.company_name,
            settings.street_address,
            settings.plz,
            settings.city,
            settings.country,
            settings.phone,
            settings.fax,
            settings.email,
            settings.website,
            settings.tax_id,
            settings.ceo_name,
            settings.registry_court,
            settings.logo_path,
        ],
    )?;

    // Aktualisierte Daten zurückgeben
    get_company_settings()
}

/// Gibt die Zahlungseinstellungen zurück (id ist immer 1)
pub fn get_payment_settings() -> Result<PaymentSettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, bank_name, iban, bic, account_holder,
         mwst_rate, dpolg_rabatt, payment_due_days, reminder_after_days, payment_text, updated_at
         FROM payment_settings WHERE id = 1",
        [],
        |row| {
            Ok(PaymentSettings {
                id: row.get(0)?,
                bank_name: row.get(1)?,
                iban: row.get(2)?,
                bic: row.get(3)?,
                account_holder: row.get(4)?,
                mwst_rate: row.get(5)?,
                dpolg_rabatt: row.get(6)?,
                payment_due_days: row.get(7)?,
                reminder_after_days: row.get(8)?,
                payment_text: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    )
}

// ============================================================================
// NOTIFICATION SETTINGS
// ============================================================================

pub fn get_notification_settings() -> Result<NotificationSettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, checkin_reminders_enabled, payment_reminders_enabled,
         payment_reminder_after_days, payment_reminder_repeat_days,
         scheduler_interval_hours, updated_at
         FROM notification_settings WHERE id = 1",
        [],
        |row| {
            Ok(NotificationSettings {
                id: row.get(0)?,
                checkin_reminders_enabled: row.get(1)?,
                payment_reminders_enabled: row.get(2)?,
                payment_reminder_after_days: row.get(3)?,
                payment_reminder_repeat_days: row.get(4)?,
                scheduler_interval_hours: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
}

pub fn save_notification_settings(settings: NotificationSettings) -> Result<NotificationSettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "UPDATE notification_settings
         SET checkin_reminders_enabled = ?1,
             payment_reminders_enabled = ?2,
             payment_reminder_after_days = ?3,
             payment_reminder_repeat_days = ?4,
             scheduler_interval_hours = ?5,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = 1",
        rusqlite::params![
            settings.checkin_reminders_enabled,
            settings.payment_reminders_enabled,
            settings.payment_reminder_after_days,
            settings.payment_reminder_repeat_days,
            settings.scheduler_interval_hours,
        ],
    )?;

    // Return updated settings
    get_notification_settings()
}

// ============================================================================
// PRICING SETTINGS
// ============================================================================

/// Gibt die Preiseinstellungen zurück (id ist immer 1)
pub fn get_pricing_settings() -> Result<PricingSettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende,
         mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis, updated_at
         FROM pricing_settings WHERE id = 1",
        [],
        |row| {
            Ok(PricingSettings {
                id: row.get(0)?,
                hauptsaison_aktiv: row.get(1)?,
                hauptsaison_start: row.get(2)?,
                hauptsaison_ende: row.get(3)?,
                mitglieder_rabatt_aktiv: row.get(4)?,
                mitglieder_rabatt_prozent: row.get(5)?,
                rabatt_basis: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
}

/// Speichert/Aktualisiert die Preiseinstellungen (id ist immer 1)
pub fn save_pricing_settings(settings: PricingSettings) -> Result<PricingSettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // UPSERT Pattern: INSERT OR REPLACE
    conn.execute(
        "INSERT OR REPLACE INTO pricing_settings (
            id, hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende,
            mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis,
            updated_at
        ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
        rusqlite::params![
            settings.hauptsaison_aktiv,
            settings.hauptsaison_start,
            settings.hauptsaison_ende,
            settings.mitglieder_rabatt_aktiv,
            settings.mitglieder_rabatt_prozent,
            settings.rabatt_basis,
        ],
    )?;

    // Aktualisierte Daten zurückgeben
    get_pricing_settings()
}

/// Speichert/Aktualisiert die Zahlungseinstellungen (id ist immer 1)
pub fn save_payment_settings(settings: PaymentSettings) -> Result<PaymentSettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // UPSERT Pattern: INSERT OR REPLACE
    conn.execute(
        "INSERT OR REPLACE INTO payment_settings (
            id, bank_name, iban, bic, account_holder,
            mwst_rate, dpolg_rabatt, payment_due_days, reminder_after_days, payment_text,
            updated_at
        ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP)",
        rusqlite::params![
            settings.bank_name,
            settings.iban,
            settings.bic,
            settings.account_holder,
            settings.mwst_rate,
            settings.dpolg_rabatt,
            settings.payment_due_days,
            settings.reminder_after_days,
            settings.payment_text,
        ],
    )?;

    // Aktualisierte Daten zurückgeben
    get_payment_settings()
}

// ============================================================================
// SERVICE TEMPLATES CRUD OPERATIONS
// ============================================================================

pub fn create_service_template(
    name: String,
    description: Option<String>,
    price: f64,
) -> Result<ServiceTemplate, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("PRAGMA Fehler: {}", e))?;

    // Prüfe ob Name bereits existiert
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM service_templates WHERE name = ?1",
            [&name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Fehler beim Prüfen: {}", e))?;

    if exists {
        return Err(format!("Service '{}' existiert bereits", name));
    }

    conn.execute(
        "INSERT INTO service_templates (name, description, price) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, description, price],
    )
    .map_err(|e| format!("Fehler beim Erstellen: {}", e))?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, name, description, price, is_active, created_at, updated_at
         FROM service_templates WHERE id = ?1",
        [id],
        |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("Fehler beim Laden: {}", e))
}

pub fn get_all_service_templates() -> Result<Vec<ServiceTemplate>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, price, is_active, created_at, updated_at
             FROM service_templates ORDER BY name",
        )
        .map_err(|e| format!("SQL Fehler: {}", e))?;

    let templates = stmt
        .query_map([], |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Query Fehler: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Mapping Fehler: {}", e))?;

    Ok(templates)
}

pub fn get_active_service_templates() -> Result<Vec<ServiceTemplate>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, price, is_active, created_at, updated_at
             FROM service_templates WHERE is_active = 1 ORDER BY name",
        )
        .map_err(|e| format!("SQL Fehler: {}", e))?;

    let templates = stmt
        .query_map([], |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Query Fehler: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Mapping Fehler: {}", e))?;

    Ok(templates)
}

pub fn update_service_template(
    id: i64,
    name: String,
    description: Option<String>,
    price: f64,
    is_active: bool,
) -> Result<ServiceTemplate, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("PRAGMA Fehler: {}", e))?;

    // Prüfe ob anderes Template mit diesem Namen existiert
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM service_templates WHERE name = ?1 AND id != ?2",
            rusqlite::params![&name, id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Fehler beim Prüfen: {}", e))?;

    if exists {
        return Err(format!("Service '{}' existiert bereits", name));
    }

    conn.execute(
        "UPDATE service_templates
         SET name = ?1, description = ?2, price = ?3, is_active = ?4, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?5",
        rusqlite::params![name, description, price, if is_active { 1 } else { 0 }, id],
    )
    .map_err(|e| format!("Fehler beim Aktualisieren: {}", e))?;

    conn.query_row(
        "SELECT id, name, description, price, is_active, created_at, updated_at
         FROM service_templates WHERE id = ?1",
        [id],
        |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("Fehler beim Laden: {}", e))
}

pub fn delete_service_template(id: i64) -> Result<(), String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("PRAGMA Fehler: {}", e))?;

    conn.execute("DELETE FROM service_templates WHERE id = ?1", [id])
        .map_err(|e| format!("Fehler beim Löschen: {}", e))?;

    Ok(())
}

// ============================================================================
// DISCOUNT TEMPLATES CRUD OPERATIONS
// ============================================================================

pub fn create_discount_template(
    name: String,
    description: Option<String>,
    discount_type: String,
    discount_value: f64,
) -> Result<DiscountTemplate, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("PRAGMA Fehler: {}", e))?;

    // Validiere discount_type
    if discount_type != "percent" && discount_type != "fixed" {
        return Err("Rabatt-Typ muss 'percent' oder 'fixed' sein".to_string());
    }

    // Prüfe ob Name bereits existiert
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM discount_templates WHERE name = ?1",
            [&name],
            |row| row.get(0),
        )
        .map_err(|e| format!("Fehler beim Prüfen: {}", e))?;

    if exists {
        return Err(format!("Rabatt '{}' existiert bereits", name));
    }

    conn.execute(
        "INSERT INTO discount_templates (name, description, discount_type, discount_value)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, description, discount_type, discount_value],
    )
    .map_err(|e| format!("Fehler beim Erstellen: {}", e))?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, name, description, discount_type, discount_value, is_active, created_at, updated_at
         FROM discount_templates WHERE id = ?1",
        [id],
        |row| {
            Ok(DiscountTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                discount_type: row.get(3)?,
                discount_value: row.get(4)?,
                is_active: row.get::<_, i64>(5)? == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| format!("Fehler beim Laden: {}", e))
}

pub fn get_all_discount_templates() -> Result<Vec<DiscountTemplate>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, discount_type, discount_value, is_active, created_at, updated_at
             FROM discount_templates ORDER BY name",
        )
        .map_err(|e| format!("SQL Fehler: {}", e))?;

    let templates = stmt
        .query_map([], |row| {
            Ok(DiscountTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                discount_type: row.get(3)?,
                discount_value: row.get(4)?,
                is_active: row.get::<_, i64>(5)? == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query Fehler: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Mapping Fehler: {}", e))?;

    Ok(templates)
}

pub fn get_active_discount_templates() -> Result<Vec<DiscountTemplate>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, discount_type, discount_value, is_active, created_at, updated_at
             FROM discount_templates WHERE is_active = 1 ORDER BY name",
        )
        .map_err(|e| format!("SQL Fehler: {}", e))?;

    let templates = stmt
        .query_map([], |row| {
            Ok(DiscountTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                discount_type: row.get(3)?,
                discount_value: row.get(4)?,
                is_active: row.get::<_, i64>(5)? == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query Fehler: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Mapping Fehler: {}", e))?;

    Ok(templates)
}

pub fn update_discount_template(
    id: i64,
    name: String,
    description: Option<String>,
    discount_type: String,
    discount_value: f64,
    is_active: bool,
) -> Result<DiscountTemplate, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("PRAGMA Fehler: {}", e))?;

    // Validiere discount_type
    if discount_type != "percent" && discount_type != "fixed" {
        return Err("Rabatt-Typ muss 'percent' oder 'fixed' sein".to_string());
    }

    // Prüfe ob anderes Template mit diesem Namen existiert
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM discount_templates WHERE name = ?1 AND id != ?2",
            rusqlite::params![&name, id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Fehler beim Prüfen: {}", e))?;

    if exists {
        return Err(format!("Rabatt '{}' existiert bereits", name));
    }

    conn.execute(
        "UPDATE discount_templates
         SET name = ?1, description = ?2, discount_type = ?3, discount_value = ?4,
             is_active = ?5, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?6",
        rusqlite::params![
            name,
            description,
            discount_type,
            discount_value,
            if is_active { 1 } else { 0 },
            id
        ],
    )
    .map_err(|e| format!("Fehler beim Aktualisieren: {}", e))?;

    conn.query_row(
        "SELECT id, name, description, discount_type, discount_value, is_active, created_at, updated_at
         FROM discount_templates WHERE id = ?1",
        [id],
        |row| {
            Ok(DiscountTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                discount_type: row.get(3)?,
                discount_value: row.get(4)?,
                is_active: row.get::<_, i64>(5)? == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| format!("Fehler beim Laden: {}", e))
}

pub fn delete_discount_template(id: i64) -> Result<(), String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("PRAGMA Fehler: {}", e))?;

    conn.execute("DELETE FROM discount_templates WHERE id = ?1", [id])
        .map_err(|e| format!("Fehler beim Löschen: {}", e))?;

    Ok(())
}

// ===============================================
// EMAIL TEMPLATE MANAGEMENT FUNCTIONS
// ===============================================

/// Lädt alle Email-Templates aus der Datenbank
pub fn get_all_templates() -> Result<Vec<EmailTemplate>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, template_name, subject, body, description, created_at, updated_at
         FROM email_templates
         ORDER BY template_name"
    ).map_err(|e| format!("SQL Fehler: {}", e))?;

    let templates: Vec<EmailTemplate> = stmt.query_map([], |row| {
        Ok(EmailTemplate {
            id: row.get(0)?,
            template_name: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            description: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })
    .map_err(|e| format!("Query Fehler: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    println!("🔍 DEBUG get_all_templates: Found {} templates", templates.len());
    for t in &templates {
        println!("   - {} ({})", t.template_name, t.id);
    }

    Ok(templates)
}

/// Aktualisiert ein Email-Template
pub fn update_template(
    id: i64,
    subject: String,
    body: String,
    description: Option<String>,
) -> Result<EmailTemplate, String> {
    use rusqlite::params;

    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Aktualisiere Template mit neuem updated_at Timestamp
    let updated_at = crate::time_utils::now_utc_plus_2()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    conn.execute(
        "UPDATE email_templates
         SET subject = ?1, body = ?2, description = ?3, updated_at = ?4
         WHERE id = ?5",
        params![subject, body, description, updated_at, id],
    )
    .map_err(|e| format!("Fehler beim Aktualisieren: {}", e))?;

    // Lade und gebe das aktualisierte Template zurück
    conn.query_row(
        "SELECT id, template_name, subject, body, description, created_at, updated_at
         FROM email_templates WHERE id = ?1",
        [id],
        |row| {
            Ok(EmailTemplate {
                id: row.get(0)?,
                template_name: row.get(1)?,
                subject: row.get(2)?,
                body: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("Fehler beim Laden: {}", e))
}
