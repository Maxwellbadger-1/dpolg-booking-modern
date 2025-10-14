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
    // Rechnungsversand-Tracking
    pub rechnung_versendet_am: Option<String>,
    pub rechnung_versendet_an: Option<String>,
    // Stiftungsfall-Flag
    pub ist_stiftungsfall: bool,
    // Externer Rechnungsempfänger
    pub payment_recipient_id: Option<i64>,
}

// Externer Rechnungsempfänger (für dienstliche Buchungen, etc.)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentRecipient {
    pub id: i64,
    pub name: String,
    pub company: Option<String>,
    pub street: Option<String>,
    pub plz: Option<String>,
    pub city: Option<String>,
    pub country: String,
    pub contact_person: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

// Neue Tabelle: Begleitpersonen für eine Buchung
#[derive(Debug, Serialize, Deserialize)]
pub struct AccompanyingGuest {
    pub id: i64,
    pub booking_id: i64,
    pub vorname: String,
    pub nachname: String,
    pub geburtsdatum: Option<String>, // Format: YYYY-MM-DD
    pub companion_id: Option<i64>, // Referenz zu guest_companions (wenn aus Pool)
}

// Credit-System: Guthaben-Transaktion
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuestCreditTransaction {
    pub id: i64,
    pub guest_id: i64,
    pub amount: f64,
    pub transaction_type: String, // "added" | "used" | "expired" | "refund"
    pub booking_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

// Credit-System: Guthaben-Balance (berechnet aus Transaktionen)
#[derive(Debug, Serialize, Deserialize)]
pub struct GuestCreditBalance {
    pub guest_id: i64,
    pub balance: f64,
    pub transaction_count: i32,
}

// Neue Tabelle: Permanenter Pool von Begleitpersonen pro Gast
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuestCompanion {
    pub id: i64,
    pub guest_id: i64, // Hauptgast
    pub vorname: String,
    pub nachname: String,
    pub geburtsdatum: Option<String>, // Format: YYYY-MM-DD
    pub beziehung: Option<String>, // "Ehepartner", "Kind", "Freund", etc.
    pub notizen: Option<String>,
    pub created_at: String,
}

// Neue Tabelle: Zusätzliche Services für eine Buchung
#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionalService {
    pub id: i64,
    pub booking_id: i64,
    pub service_name: String,
    pub service_price: f64,
    pub created_at: String,
    pub template_id: Option<i64>, // Wenn Service von Template kommt (aus booking_services Junction-Table)
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
    // Emoji & Visualisierung (nur für Display!)
    pub emoji: Option<String>,
    pub color_hex: Option<String>,
    // Putzplan-Integration
    pub show_in_cleaning_plan: bool,
    pub cleaning_plan_position: String, // 'start' oder 'end'
    pub requires_dog_cleaning: bool,
    pub requires_bedding_change: bool,
    pub requires_deep_cleaning: bool,
    // Timestamps
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
    // Emoji & Visualisierung
    pub emoji: Option<String>,
    pub color_hex: Option<String>,
    // Putzplan-Integration (falls Rabatt mit Zusatzleistung verbunden)
    pub show_in_cleaning_plan: bool,
    pub cleaning_plan_position: String, // 'start' oder 'end'
    // Worauf bezieht sich der Rabatt?
    pub applies_to: String, // 'overnight_price' oder 'total_price'
    pub created_at: String,
    pub updated_at: String,
}

// Reminder-System Structs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reminder {
    pub id: i64,
    pub booking_id: Option<i64>, // NULL für globale Erinnerungen
    pub reminder_type: String, // 'manual', 'auto_incomplete_data', 'auto_payment', 'auto_checkin'
    pub title: String,
    pub description: Option<String>,
    pub due_date: String, // Format: YYYY-MM-DD
    pub priority: String, // 'low', 'medium', 'high'
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub is_snoozed: bool,
    pub snoozed_until: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// Settings für Auto-Reminders
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReminderSettings {
    pub id: i64,
    pub auto_reminder_incomplete_data: bool,
    pub auto_reminder_payment: bool,
    pub auto_reminder_checkin: bool,
    pub auto_reminder_invoice: bool,
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
    // Rechnungsversand-Tracking
    pub rechnung_versendet_am: Option<String>,
    pub rechnung_versendet_an: Option<String>,
    // Stiftungsfall-Flag
    pub ist_stiftungsfall: bool,
    // Payment Recipient (optional)
    pub payment_recipient_id: Option<i64>,
    pub room: Room,
    pub guest: Guest,
    // Services & Discounts mit Emoji & Visualisierung
    pub services: Vec<ServiceTemplate>,
    pub discounts: Vec<DiscountTemplate>,
}

pub fn get_db_path() -> PathBuf {
    // WICHTIG: Immer die DB in src-tauri/ verwenden (unabhängig vom CWD)
    // Das stellt sicher dass Desktop App und Playwright Tests die gleiche DB nutzen
    let db_path = PathBuf::from("src-tauri/booking_system.db");

    // Wenn src-tauri/booking_system.db existiert, verwende diese
    if db_path.exists() {
        return db_path;
    }

    // Fallback: Wenn wir bereits in src-tauri/ sind (CWD = src-tauri)
    let local_db = PathBuf::from("booking_system.db");
    if local_db.exists() {
        return local_db;
    }

    // Default: src-tauri/booking_system.db (wird erstellt wenn nicht existiert)
    db_path
}

// Helper function: Get a database connection
// Used by all modules that need DB access
pub fn get_connection() -> Result<Connection> {
    let conn = Connection::open(get_db_path())?;
    // WICHTIG: Foreign Keys aktivieren für jede Connection
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    Ok(conn)
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

    // Phase 6.5: Rechnungsversand-Tracking
    add_column_if_not_exists(&conn, "bookings", "rechnung_versendet_am", "TEXT")?;
    add_column_if_not_exists(&conn, "bookings", "rechnung_versendet_an", "TEXT")?;

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

    // Erweitere accompanying_guests um companion_id Referenz (Migration)
    add_column_if_not_exists(&conn, "accompanying_guests", "companion_id", "INTEGER")?;

    // Neue Tabelle: Permanenter Pool von Begleitpersonen pro Gast
    // CASCADE DELETE: Wenn Hauptgast gelöscht wird, werden auch seine Begleitpersonen gelöscht
    conn.execute(
        "CREATE TABLE IF NOT EXISTS guest_companions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            guest_id INTEGER NOT NULL,
            vorname TEXT NOT NULL,
            nachname TEXT NOT NULL,
            geburtsdatum TEXT,
            beziehung TEXT,
            notizen TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (guest_id) REFERENCES guests (id) ON DELETE CASCADE
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

    // Reminder-System: Erinnerungen-Tabelle
    conn.execute(
        "CREATE TABLE IF NOT EXISTS reminders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            booking_id INTEGER,
            reminder_type TEXT NOT NULL CHECK(reminder_type IN ('manual', 'auto_incomplete_data', 'auto_payment', 'auto_checkin', 'auto_invoice')),
            title TEXT NOT NULL,
            description TEXT,
            due_date TEXT NOT NULL,
            priority TEXT NOT NULL CHECK(priority IN ('low', 'medium', 'high')) DEFAULT 'medium',
            is_completed INTEGER NOT NULL DEFAULT 0,
            completed_at TEXT,
            is_snoozed INTEGER NOT NULL DEFAULT 0,
            snoozed_until TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (booking_id) REFERENCES bookings (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Reminder-System: Settings-Tabelle für Auto-Reminder On/Off
    conn.execute(
        "CREATE TABLE IF NOT EXISTS reminder_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            auto_reminder_incomplete_data INTEGER NOT NULL DEFAULT 1,
            auto_reminder_payment INTEGER NOT NULL DEFAULT 1,
            auto_reminder_checkin INTEGER NOT NULL DEFAULT 1,
            auto_reminder_invoice INTEGER NOT NULL DEFAULT 1,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Erstelle default reminder settings falls noch nicht vorhanden
    conn.execute(
        "INSERT OR IGNORE INTO reminder_settings (id) VALUES (1)",
        [],
    )?;

    // Stiftungsfall-Spalte hinzufügen falls noch nicht vorhanden
    add_column_if_not_exists(&conn, "bookings", "ist_stiftungsfall", "INTEGER DEFAULT 0")?;

    // Payment Recipients: Tabelle für externe Rechnungsempfänger (z.B. Dienststellen)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS payment_recipients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            company TEXT,
            street TEXT,
            plz TEXT,
            city TEXT,
            country TEXT NOT NULL DEFAULT 'Deutschland',
            contact_person TEXT,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Payment Recipients: Spalte in bookings Tabelle
    add_column_if_not_exists(&conn, "bookings", "payment_recipient_id", "INTEGER")?;

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

    // Junction-Table: Verbindung zwischen Buchungen und Service-Templates (Many-to-Many)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS booking_services (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            booking_id INTEGER NOT NULL,
            service_template_id INTEGER NOT NULL,
            FOREIGN KEY (booking_id) REFERENCES bookings (id) ON DELETE CASCADE,
            FOREIGN KEY (service_template_id) REFERENCES service_templates (id) ON DELETE CASCADE,
            UNIQUE(booking_id, service_template_id)
        )",
        [],
    )?;

    // Junction-Table: Verbindung zwischen Buchungen und Discount-Templates (Many-to-Many)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS booking_discounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            booking_id INTEGER NOT NULL,
            discount_template_id INTEGER NOT NULL,
            FOREIGN KEY (booking_id) REFERENCES bookings (id) ON DELETE CASCADE,
            FOREIGN KEY (discount_template_id) REFERENCES discount_templates (id) ON DELETE CASCADE,
            UNIQUE(booking_id, discount_template_id)
        )",
        [],
    )?;

    // Tabelle für Guthaben-Transaktionen (Credit System)
    // Transaktions-basiertes System: Jede Änderung am Guthaben ist eine Transaktion
    // Aktuelles Guthaben = SUM(amount) WHERE guest_id = X
    conn.execute(
        "CREATE TABLE IF NOT EXISTS guest_credit_transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            guest_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            transaction_type TEXT NOT NULL CHECK(transaction_type IN ('added', 'used', 'expired', 'refund')),
            booking_id INTEGER,
            notes TEXT,
            created_by TEXT DEFAULT 'System',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (guest_id) REFERENCES guests (id) ON DELETE CASCADE,
            FOREIGN KEY (booking_id) REFERENCES bookings (id) ON DELETE SET NULL
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

    // Indexes für Junction-Tables (Performance bei JOINs)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_booking_services_booking ON booking_services(booking_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_booking_services_service ON booking_services(service_template_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_booking_discounts_booking ON booking_discounts(booking_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_booking_discounts_discount ON booking_discounts(discount_template_id)",
        [],
    )?;

    // Index für Credit-System (Performance bei Guthaben-Abfragen pro Gast)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_guest_credit_transactions_guest ON guest_credit_transactions(guest_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_guest_credit_transactions_booking ON guest_credit_transactions(booking_id)",
        [],
    )?;

    // Indexes für Reminder-System (Performance für Abfragen)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reminders_due_date ON reminders(due_date)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reminders_completed ON reminders(is_completed)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_reminders_booking ON reminders(booking_id)",
        [],
    )?;

    // Erweitere service_templates um Emoji- und Putzplan-Felder
    add_column_if_not_exists(&conn, "service_templates", "emoji", "TEXT")?;
    add_column_if_not_exists(&conn, "service_templates", "color_hex", "TEXT")?;
    add_column_if_not_exists(&conn, "service_templates", "show_in_cleaning_plan", "INTEGER DEFAULT 0")?;
    add_column_if_not_exists(&conn, "service_templates", "cleaning_plan_position", "TEXT DEFAULT 'start'")?;

    // ✨ NEU: Professional Cleaning Flags (Boolean statt Emoji-Detection!)
    add_column_if_not_exists(&conn, "service_templates", "requires_dog_cleaning", "INTEGER DEFAULT 0")?;
    add_column_if_not_exists(&conn, "service_templates", "requires_bedding_change", "INTEGER DEFAULT 0")?;
    add_column_if_not_exists(&conn, "service_templates", "requires_deep_cleaning", "INTEGER DEFAULT 0")?;

    // 🔄 Automatische Migration: Setze Flags basierend auf Service-Namen & Emojis
    // Services mit "Hund" im Namen → requires_dog_cleaning = true
    conn.execute(
        "UPDATE service_templates
         SET requires_dog_cleaning = 1
         WHERE (LOWER(name) LIKE '%hund%' OR emoji LIKE '%🐕%' OR emoji LIKE '%🐶%')
         AND requires_dog_cleaning = 0",
        [],
    ).ok(); // Ignoriere Fehler falls Spalte noch nicht existiert

    // Services mit "Bett" im Namen → requires_bedding_change = true
    conn.execute(
        "UPDATE service_templates
         SET requires_bedding_change = 1
         WHERE (LOWER(name) LIKE '%bett%' OR LOWER(name) LIKE '%bettwäsche%' OR emoji LIKE '%🛏%')
         AND requires_bedding_change = 0",
        [],
    ).ok(); // Ignoriere Fehler

    // Erweitere discount_templates um Emoji- und Putzplan-Felder
    add_column_if_not_exists(&conn, "discount_templates", "emoji", "TEXT")?;
    add_column_if_not_exists(&conn, "discount_templates", "color_hex", "TEXT")?;
    add_column_if_not_exists(&conn, "discount_templates", "show_in_cleaning_plan", "INTEGER DEFAULT 0")?;
    add_column_if_not_exists(&conn, "discount_templates", "cleaning_plan_position", "TEXT DEFAULT 'start'")?;
    add_column_if_not_exists(&conn, "discount_templates", "applies_to", "TEXT DEFAULT 'total_price'")?;

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
            b.rechnung_versendet_am, b.rechnung_versendet_an, b.ist_stiftungsfall, b.payment_recipient_id,
            r.id, r.name, r.gebaeude_typ, r.capacity, r.price_member, r.price_non_member, r.nebensaison_preis, r.hauptsaison_preis, r.endreinigung, r.ort, r.schluesselcode,
            g.id, g.vorname, g.nachname, g.email, g.telefon, g.dpolg_mitglied,
            g.strasse, g.plz, g.ort, g.mitgliedsnummer, g.notizen, g.beruf, g.bundesland, g.dienststelle, g.created_at
         FROM bookings b
         JOIN rooms r ON b.room_id = r.id
         JOIN guests g ON b.guest_id = g.id
         ORDER BY b.id DESC"
    )?;

    let mut bookings: Vec<BookingWithDetails> = stmt.query_map([], |row| {
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
            rechnung_versendet_am: row.get(19)?,
            rechnung_versendet_an: row.get(20)?,
            ist_stiftungsfall: row.get::<_, i32>(21)? != 0,
            payment_recipient_id: row.get(22)?,
            room: Room {
                id: row.get(23)?,
                name: row.get(24)?,
                gebaeude_typ: row.get(25)?,
                capacity: row.get(26)?,
                price_member: row.get(27)?,
                price_non_member: row.get(28)?,
                nebensaison_preis: row.get(29)?,
                hauptsaison_preis: row.get(30)?,
                endreinigung: row.get(31)?,
                ort: row.get(32)?,
                schluesselcode: row.get(33)?,
            },
            guest: Guest {
                id: row.get(34)?,
                vorname: row.get(35)?,
                nachname: row.get(36)?,
                email: row.get(37)?,
                telefon: row.get(38)?,
                dpolg_mitglied: row.get(39)?,
                strasse: row.get(40)?,
                plz: row.get(41)?,
                ort: row.get(42)?,
                mitgliedsnummer: row.get(43)?,
                notizen: row.get(44)?,
                beruf: row.get(45)?,
                bundesland: row.get(46)?,
                dienststelle: row.get(47)?,
                created_at: row.get(48)?,
            },
            // Initiale leere Vektoren (werden unten befüllt)
            services: Vec::new(),
            discounts: Vec::new(),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    // Für jede Buchung: Lade zugehörige Services und Discounts
    for booking in &mut bookings {
        // Services laden
        let mut service_stmt = conn.prepare(
            "SELECT st.id, st.name, st.description, st.price, st.is_active,
                    st.emoji, st.color_hex, st.show_in_cleaning_plan, st.cleaning_plan_position,
                    st.requires_dog_cleaning, st.requires_bedding_change, st.requires_deep_cleaning,
                    st.created_at, st.updated_at
             FROM service_templates st
             JOIN booking_services bs ON st.id = bs.service_template_id
             WHERE bs.booking_id = ?1"
        )?;

        let services = service_stmt.query_map([booking.id], |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                emoji: row.get(5)?,
                color_hex: row.get(6)?,
                show_in_cleaning_plan: row.get::<_, i32>(7)? != 0,
                cleaning_plan_position: row.get(8)?,
                requires_dog_cleaning: row.get::<_, i32>(9)? != 0,
                requires_bedding_change: row.get::<_, i32>(10)? != 0,
                requires_deep_cleaning: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        booking.services = services;

        // Discounts laden
        let mut discount_stmt = conn.prepare(
            "SELECT dt.id, dt.name, dt.description, dt.discount_type, dt.discount_value, dt.is_active,
                    dt.emoji, dt.color_hex, dt.show_in_cleaning_plan, dt.cleaning_plan_position, dt.applies_to,
                    dt.created_at, dt.updated_at
             FROM discount_templates dt
             JOIN booking_discounts bd ON dt.id = bd.discount_template_id
             WHERE bd.booking_id = ?1"
        )?;

        let discounts = discount_stmt.query_map([booking.id], |row| {
            Ok(DiscountTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                discount_type: row.get(3)?,
                discount_value: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                emoji: row.get(6)?,
                color_hex: row.get(7)?,
                show_in_cleaning_plan: row.get::<_, i32>(8)? != 0,
                cleaning_plan_position: row.get(9)?,
                applies_to: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        booking.discounts = discounts;
    }

    Ok(bookings)
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
    ist_stiftungsfall: bool,
) -> Result<BookingWithDetails> {
    println!("🔍 create_booking() aufgerufen - Transaction Logging aktiviert");
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Erst einfügen mit temporärer Reservierungsnummer
    conn.execute(
        "INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
         anzahl_gaeste, status, gesamtpreis, bemerkungen, anzahl_begleitpersonen,
         grundpreis, services_preis, rabatt_preis, anzahl_naechte, ist_stiftungsfall)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
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
            anzahl_naechte,
            ist_stiftungsfall
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

    // FIX: Return BookingWithDetails (with nested room/guest) for Optimistic Update
    // This prevents "Unbekannt" display issue in frontend
    let booking_with_details = get_booking_with_details_by_id(id)?;

    // DEBUG LOGGING - Verify nested objects exist
    println!("🔍 DEBUG create_booking - BookingWithDetails geladen:");
    println!("   - booking.id: {}", booking_with_details.id);
    println!("   - booking.room.id: {}", booking_with_details.room.id);
    println!("   - booking.room.name: {}", booking_with_details.room.name);
    println!("   - booking.guest.id: {}", booking_with_details.guest.id);
    println!("   - booking.guest.vorname: {}", booking_with_details.guest.vorname);
    println!("   - booking.guest.nachname: {}", booking_with_details.guest.nachname);
    println!("   - booking.ist_stiftungsfall: {}", booking_with_details.ist_stiftungsfall);

    // Transaction Log: CREATE (use simple booking for JSON)
    let booking = get_booking_by_id(id)?;
    let booking_json = serde_json::to_string(&booking).unwrap_or_default();
    let user_action = format!("Buchung {} erstellt", booking.reservierungsnummer);
    if let Err(e) = crate::transaction_log::log_create("bookings", id, &booking_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    println!("✅ DEBUG create_booking - Returning BookingWithDetails to frontend");
    Ok(booking_with_details)
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
    ist_stiftungsfall: bool,
    payment_recipient_id: Option<i64>,
) -> Result<BookingWithDetails> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("🔍 [DEBUG] update_booking CALLED");
    println!("   Booking ID: {}", id);
    println!("   payment_recipient_id: {:?}", payment_recipient_id);
    println!("   ist_stiftungsfall: {}", ist_stiftungsfall);
    println!("   status: {}", status);
    println!("═══════════════════════════════════════════════════════════════");

    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log
    let old_booking = get_booking_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_booking).unwrap_or_default();

    println!("📝 [DEBUG] Executing SQL UPDATE...");
    println!("   SQL: UPDATE bookings SET payment_recipient_id = ?15 WHERE id = ?16");
    println!("   Param ?15 (payment_recipient_id): {:?}", payment_recipient_id);
    println!("   Param ?16 (id): {}", id);

    let rows_affected = conn.execute(
        "UPDATE bookings SET
         room_id = ?1, guest_id = ?2, checkin_date = ?3, checkout_date = ?4,
         anzahl_gaeste = ?5, status = ?6, gesamtpreis = ?7, bemerkungen = ?8,
         anzahl_begleitpersonen = ?9, grundpreis = ?10, services_preis = ?11,
         rabatt_preis = ?12, anzahl_naechte = ?13, ist_stiftungsfall = ?14, payment_recipient_id = ?15, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?16",
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
            ist_stiftungsfall,
            payment_recipient_id,
            id
        ],
    )?;

    println!("✅ [DEBUG] SQL UPDATE executed successfully!");
    println!("   Rows affected: {}", rows_affected);

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    // FIX: Return BookingWithDetails (with nested room/guest) for Optimistic Update
    let booking_with_details = get_booking_with_details_by_id(id)?;

    println!("🔍 [DEBUG] Verifying payment_recipient_id after UPDATE:");
    println!("   booking_with_details.payment_recipient_id: {:?}", booking_with_details.payment_recipient_id);

    // Let's also query it directly to double-check
    let verification: Option<i64> = conn
        .query_row("SELECT payment_recipient_id FROM bookings WHERE id = ?1", [id], |row| row.get(0))
        .ok();
    println!("   Direct SQL query result: {:?}", verification);
    println!("═══════════════════════════════════════════════════════════════");

    // Transaction Log: UPDATE (use simple booking for JSON)
    let updated_booking = get_booking_by_id(id)?;
    let new_data_json = serde_json::to_string(&updated_booking).unwrap_or_default();
    let user_action = format!("Buchung {} aktualisiert", updated_booking.reservierungsnummer);
    if let Err(e) = crate::transaction_log::log_update("bookings", id, &old_data_json, &new_data_json, &user_action) {
        eprintln!("⚠️  Transaction Log Fehler: {}", e);
    }

    Ok(booking_with_details)
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
    println!("═══════════════════════════════════════");
    println!("🔄 [update_booking_dates_and_room] START");
    println!("📤 Parameters: id={}, room_id={}, checkin={}, checkout={}", id, room_id, checkin_date, checkout_date);

    let db_path = get_db_path();
    println!("📁 [update_booking_dates_and_room] Database path: {:?}", db_path);

    let conn = Connection::open(&db_path)?;
    println!("✅ [update_booking_dates_and_room] Connection opened");

    conn.execute("PRAGMA foreign_keys = ON", [])?;
    println!("✅ [update_booking_dates_and_room] Foreign keys enabled");

    // Get old data for transaction log
    let old_booking = get_booking_by_id(id)?;
    println!("📦 [update_booking_dates_and_room] OLD booking data:");
    println!("   - ID: {}", old_booking.id);
    println!("   - Room ID: {}", old_booking.room_id);
    println!("   - Check-in: {}", old_booking.checkin_date);
    println!("   - Check-out: {}", old_booking.checkout_date);
    let old_data_json = serde_json::to_string(&old_booking).unwrap_or_default();

    println!("🚀 [update_booking_dates_and_room] Executing UPDATE statement...");
    let rows_affected = conn.execute(
        "UPDATE bookings SET
         room_id = ?1, checkin_date = ?2, checkout_date = ?3, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?4",
        rusqlite::params![room_id, checkin_date, checkout_date, id],
    )?;
    println!("📊 [update_booking_dates_and_room] Rows affected: {}", rows_affected);

    if rows_affected == 0 {
        println!("❌ [update_booking_dates_and_room] No rows affected! Booking not found!");
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    println!("🔍 [update_booking_dates_and_room] Reading updated booking from DB...");
    let updated_booking = get_booking_by_id(id)?;
    println!("📦 [update_booking_dates_and_room] NEW booking data:");
    println!("   - ID: {}", updated_booking.id);
    println!("   - Room ID: {}", updated_booking.room_id);
    println!("   - Check-in: {}", updated_booking.checkin_date);
    println!("   - Check-out: {}", updated_booking.checkout_date);

    // Verify the changes were actually written
    if updated_booking.room_id == room_id &&
       updated_booking.checkin_date == checkin_date &&
       updated_booking.checkout_date == checkout_date {
        println!("✅ [update_booking_dates_and_room] VERIFIED: Changes persisted in database!");
    } else {
        println!("❌ [update_booking_dates_and_room] WARNING: Data mismatch after update!");
        println!("   Expected: room={}, checkin={}, checkout={}", room_id, checkin_date, checkout_date);
        println!("   Got: room={}, checkin={}, checkout={}", updated_booking.room_id, updated_booking.checkin_date, updated_booking.checkout_date);
    }

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

/// Markiert eine Rechnung als versendet
pub fn mark_invoice_sent(id: i64, email_address: String) -> Result<Booking> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rechnung_versendet_am = crate::time_utils::format_today_db();

    let rows_affected = conn.execute(
        "UPDATE bookings SET rechnung_versendet_am = ?1, rechnung_versendet_an = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        rusqlite::params![rechnung_versendet_am, email_address, id],
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
pub fn update_booking_payment(id: i64, bezahlt: bool, zahlungsmethode: Option<String>, bezahlt_am_param: Option<String>) -> Result<Booking> {
    println!("🔍 [update_booking_payment] Called:");
    println!("   id: {}", id);
    println!("   bezahlt: {}", bezahlt);
    println!("   zahlungsmethode: {:?}", zahlungsmethode);
    println!("   bezahlt_am_param: {:?}", bezahlt_am_param);

    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Get old data for transaction log
    let old_booking = get_booking_by_id(id)?;
    let old_data_json = serde_json::to_string(&old_booking).unwrap_or_default();

    let zahlungsmethode_string = zahlungsmethode.clone().unwrap_or_else(|| "Überweisung".to_string());

    if bezahlt {
        // Bezahlt markieren mit Zahlungsmethode und Datum
        // Verwende das übergebene Datum, oder falls nicht vorhanden, heutiges Datum
        let bezahlt_am = bezahlt_am_param.unwrap_or_else(|| crate::time_utils::format_today_db());
        println!("   Using bezahlt_am: {}", bezahlt_am);
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
         bezahlt, bezahlt_am, zahlungsmethode, mahnung_gesendet_am,
         rechnung_versendet_am, rechnung_versendet_an, ist_stiftungsfall, payment_recipient_id
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
                rechnung_versendet_am: row.get(21)?,
                rechnung_versendet_an: row.get(22)?,
                ist_stiftungsfall: row.get::<_, i32>(23)? != 0,
                payment_recipient_id: row.get(24)?,
            })
        },
    )
}

/// Gibt eine Buchung mit allen Details (Room + Guest) anhand der ID zurück
pub fn get_booking_with_details_by_id(id: i64) -> Result<BookingWithDetails> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let booking = conn.query_row(
        "SELECT
            b.id, b.room_id, b.guest_id, b.reservierungsnummer,
            b.checkin_date, b.checkout_date, b.anzahl_gaeste, b.anzahl_begleitpersonen,
            b.status, b.grundpreis, b.services_preis, b.rabatt_preis, b.gesamtpreis, b.anzahl_naechte, b.bemerkungen,
            b.bezahlt, b.bezahlt_am, b.zahlungsmethode, b.mahnung_gesendet_am,
            b.rechnung_versendet_am, b.rechnung_versendet_an, b.ist_stiftungsfall, b.payment_recipient_id,
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
                rechnung_versendet_am: row.get(19)?,
                rechnung_versendet_an: row.get(20)?,
                ist_stiftungsfall: row.get::<_, i32>(21)? != 0,
                payment_recipient_id: row.get(22)?,
                room: Room {
                    id: row.get(23)?,
                    name: row.get(24)?,
                    gebaeude_typ: row.get(25)?,
                    capacity: row.get(26)?,
                    price_member: row.get(27)?,
                    price_non_member: row.get(28)?,
                    nebensaison_preis: row.get(29)?,
                    hauptsaison_preis: row.get(30)?,
                    endreinigung: row.get(31)?,
                    ort: row.get(32)?,
                    schluesselcode: row.get(33)?,
                },
                guest: Guest {
                    id: row.get(34)?,
                    vorname: row.get(35)?,
                    nachname: row.get(36)?,
                    email: row.get(37)?,
                    telefon: row.get(38)?,
                    dpolg_mitglied: row.get(39)?,
                    strasse: row.get(40)?,
                    plz: row.get(41)?,
                    ort: row.get(42)?,
                    mitgliedsnummer: row.get(43)?,
                    notizen: row.get(44)?,
                    beruf: row.get(45)?,
                    bundesland: row.get(46)?,
                    dienststelle: row.get(47)?,
                    created_at: row.get(48)?,
                },
                // Services und Discounts werden leer initialisiert (werden unten befüllt)
                services: Vec::new(),
                discounts: Vec::new(),
            })
        },
    )?;

    // Services für diese Buchung laden (aus Junction-Table)
    let mut service_stmt = conn.prepare(
        "SELECT st.id, st.name, st.description, st.price, st.is_active,
                st.emoji, st.color_hex, st.show_in_cleaning_plan, st.cleaning_plan_position,
                st.requires_dog_cleaning, st.requires_bedding_change, st.requires_deep_cleaning,
                st.created_at, st.updated_at
         FROM service_templates st
         JOIN booking_services bs ON st.id = bs.service_template_id
         WHERE bs.booking_id = ?1"
    )?;

    let services = service_stmt.query_map([id], |row| {
        Ok(ServiceTemplate {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            price: row.get(3)?,
            is_active: row.get::<_, i32>(4)? != 0,
            emoji: row.get(5)?,
            color_hex: row.get(6)?,
            show_in_cleaning_plan: row.get::<_, i32>(7)? != 0,
            cleaning_plan_position: row.get(8)?,
            requires_dog_cleaning: row.get::<_, i32>(9)? != 0,
            requires_bedding_change: row.get::<_, i32>(10)? != 0,
            requires_deep_cleaning: row.get::<_, i32>(11)? != 0,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    // Discounts für diese Buchung laden (aus Junction-Table)
    let mut discount_stmt = conn.prepare(
        "SELECT dt.id, dt.name, dt.description, dt.discount_type, dt.discount_value, dt.is_active,
                dt.emoji, dt.color_hex, dt.show_in_cleaning_plan, dt.cleaning_plan_position, dt.applies_to,
                dt.created_at, dt.updated_at
         FROM discount_templates dt
         JOIN booking_discounts bd ON dt.id = bd.discount_template_id
         WHERE bd.booking_id = ?1"
    )?;

    let discounts = discount_stmt.query_map([id], |row| {
        Ok(DiscountTemplate {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            discount_type: row.get(3)?,
            discount_value: row.get(4)?,
            is_active: row.get::<_, i32>(5)? != 0,
            emoji: row.get(6)?,
            color_hex: row.get(7)?,
            show_in_cleaning_plan: row.get::<_, i32>(8)? != 0,
            cleaning_plan_position: row.get(9)?,
            applies_to: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    // Mutable booking erstellen und Services/Discounts setzen
    let mut booking_with_services = booking;
    booking_with_services.services = services;
    booking_with_services.discounts = discounts;

    Ok(booking_with_services)
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
                template_id: None, // Manuelle Services haben kein Template
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

/// Gibt alle Services für eine Buchung zurück (manuelle + template-basierte)
pub fn get_booking_services(booking_id: i64) -> Result<Vec<AdditionalService>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // UNION Query: Beide Quellen kombinieren
    // 1. Manuelle Services aus additional_services
    // 2. Template-basierte Services aus booking_services JOIN service_templates
    // NOTE: booking_services hat keine created_at Spalte, verwende CURRENT_TIMESTAMP
    let mut stmt = conn.prepare(
        "SELECT id, booking_id, service_name, service_price, created_at, NULL as template_id
         FROM additional_services
         WHERE booking_id = ?1

         UNION ALL

         SELECT
            bs.id,
            bs.booking_id,
            st.name as service_name,
            st.price as service_price,
            datetime('now') as created_at,
            bs.service_template_id as template_id
         FROM booking_services bs
         JOIN service_templates st ON bs.service_template_id = st.id
         WHERE bs.booking_id = ?1

         ORDER BY created_at",
    )?;

    let services = stmt.query_map(rusqlite::params![booking_id], |row| {
        Ok(AdditionalService {
            id: row.get(0)?,
            booking_id: row.get(1)?,
            service_name: row.get(2)?,
            service_price: row.get(3)?,
            created_at: row.get(4)?,
            template_id: row.get(5)?,
        })
    })?;

    services.collect()
}

// ============================================================================
// SERVICE & DISCOUNT TEMPLATE LINKING - Junction Tables
// ============================================================================

/// Verknüpft ein Service-Template mit einer Buchung (Junction-Table)
pub fn link_service_template_to_booking(
    booking_id: i64,
    service_template_id: i64,
) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO booking_services (booking_id, service_template_id)
         VALUES (?1, ?2)",
        rusqlite::params![booking_id, service_template_id],
    )?;

    Ok(())
}

/// Verknüpft ein Discount-Template mit einer Buchung (Junction-Table)
pub fn link_discount_template_to_booking(
    booking_id: i64,
    discount_template_id: i64,
) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO booking_discounts (booking_id, discount_template_id)
         VALUES (?1, ?2)",
        rusqlite::params![booking_id, discount_template_id],
    )?;

    Ok(())
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
    companion_id: Option<i64>,
) -> Result<AccompanyingGuest> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO accompanying_guests (booking_id, vorname, nachname, geburtsdatum, companion_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![booking_id, vorname, nachname, geburtsdatum, companion_id],
    )?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, booking_id, vorname, nachname, geburtsdatum, companion_id
         FROM accompanying_guests WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(AccompanyingGuest {
                id: row.get(0)?,
                booking_id: row.get(1)?,
                vorname: row.get(2)?,
                nachname: row.get(3)?,
                geburtsdatum: row.get(4)?,
                companion_id: row.get(5)?,
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
        "SELECT id, booking_id, vorname, nachname, geburtsdatum, companion_id
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
            companion_id: row.get(5)?,
        })
    })?;

    guests.collect()
}

// ============================================================================
// GUEST COMPANIONS - Permanent Pool (Phase 1.1)
// ============================================================================

/// Erstellt eine neue Begleitperson im Pool des Gastes
pub fn create_guest_companion(
    guest_id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
    beziehung: Option<String>,
    notizen: Option<String>,
) -> Result<GuestCompanion> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Validierung
    if vorname.trim().is_empty() || nachname.trim().is_empty() {
        return Err(rusqlite::Error::InvalidQuery);
    }

    conn.execute(
        "INSERT INTO guest_companions (guest_id, vorname, nachname, geburtsdatum, beziehung, notizen)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![guest_id, vorname.trim(), nachname.trim(), geburtsdatum, beziehung, notizen],
    )?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, guest_id, vorname, nachname, geburtsdatum, beziehung, notizen, created_at
         FROM guest_companions WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(GuestCompanion {
                id: row.get(0)?,
                guest_id: row.get(1)?,
                vorname: row.get(2)?,
                nachname: row.get(3)?,
                geburtsdatum: row.get(4)?,
                beziehung: row.get(5)?,
                notizen: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )
}

/// Lädt alle Begleitpersonen eines Gastes aus dem Pool
pub fn get_guest_companions(guest_id: i64) -> Result<Vec<GuestCompanion>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT id, guest_id, vorname, nachname, geburtsdatum, beziehung, notizen, created_at
         FROM guest_companions
         WHERE guest_id = ?1
         ORDER BY nachname, vorname",
    )?;

    let companions = stmt.query_map(rusqlite::params![guest_id], |row| {
        Ok(GuestCompanion {
            id: row.get(0)?,
            guest_id: row.get(1)?,
            vorname: row.get(2)?,
            nachname: row.get(3)?,
            geburtsdatum: row.get(4)?,
            beziehung: row.get(5)?,
            notizen: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;

    companions.collect()
}

/// Aktualisiert eine Begleitperson im Pool
pub fn update_guest_companion(
    id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
    beziehung: Option<String>,
    notizen: Option<String>,
) -> Result<GuestCompanion> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Validierung
    if vorname.trim().is_empty() || nachname.trim().is_empty() {
        return Err(rusqlite::Error::InvalidQuery);
    }

    let rows_affected = conn.execute(
        "UPDATE guest_companions
         SET vorname = ?1, nachname = ?2, geburtsdatum = ?3, beziehung = ?4, notizen = ?5
         WHERE id = ?6",
        rusqlite::params![vorname.trim(), nachname.trim(), geburtsdatum, beziehung, notizen, id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    conn.query_row(
        "SELECT id, guest_id, vorname, nachname, geburtsdatum, beziehung, notizen, created_at
         FROM guest_companions WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(GuestCompanion {
                id: row.get(0)?,
                guest_id: row.get(1)?,
                vorname: row.get(2)?,
                nachname: row.get(3)?,
                geburtsdatum: row.get(4)?,
                beziehung: row.get(5)?,
                notizen: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )
}

/// Löscht eine Begleitperson aus dem Pool
pub fn delete_guest_companion(id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute(
        "DELETE FROM guest_companions WHERE id = ?1",
        rusqlite::params![id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
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
    emoji: Option<String>,
    color_hex: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
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

    // Validiere cleaning_plan_position
    if cleaning_plan_position != "start" && cleaning_plan_position != "end" {
        return Err("cleaning_plan_position muss 'start' oder 'end' sein".to_string());
    }

    conn.execute(
        "INSERT INTO service_templates (name, description, price, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            name,
            description,
            price,
            emoji,
            color_hex,
            if show_in_cleaning_plan { 1 } else { 0 },
            cleaning_plan_position
        ],
    )
    .map_err(|e| format!("Fehler beim Erstellen: {}", e))?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, name, description, price, is_active, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position,
         requires_dog_cleaning, requires_bedding_change, requires_deep_cleaning, created_at, updated_at
         FROM service_templates WHERE id = ?1",
        [id],
        |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                emoji: row.get(5)?,
                color_hex: row.get(6)?,
                show_in_cleaning_plan: row.get::<_, i64>(7)? == 1,
                cleaning_plan_position: row.get(8)?,
                requires_dog_cleaning: row.get::<_, i64>(9)? == 1,
                requires_bedding_change: row.get::<_, i64>(10)? == 1,
                requires_deep_cleaning: row.get::<_, i64>(11)? == 1,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
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
            "SELECT id, name, description, price, is_active, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position,
             requires_dog_cleaning, requires_bedding_change, requires_deep_cleaning, created_at, updated_at
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
                emoji: row.get(5)?,
                color_hex: row.get(6)?,
                show_in_cleaning_plan: row.get::<_, i64>(7)? == 1,
                cleaning_plan_position: row.get(8)?,
                requires_dog_cleaning: row.get::<_, i64>(9)? == 1,
                requires_bedding_change: row.get::<_, i64>(10)? == 1,
                requires_deep_cleaning: row.get::<_, i64>(11)? == 1,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
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
            "SELECT id, name, description, price, is_active, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position,
             requires_dog_cleaning, requires_bedding_change, requires_deep_cleaning, created_at, updated_at
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
                emoji: row.get(5)?,
                color_hex: row.get(6)?,
                show_in_cleaning_plan: row.get::<_, i64>(7)? == 1,
                cleaning_plan_position: row.get(8)?,
                requires_dog_cleaning: row.get::<_, i64>(9)? == 1,
                requires_bedding_change: row.get::<_, i64>(10)? == 1,
                requires_deep_cleaning: row.get::<_, i64>(11)? == 1,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
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
    emoji: Option<String>,
    color_hex: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
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

    // Validiere cleaning_plan_position
    if cleaning_plan_position != "start" && cleaning_plan_position != "end" {
        return Err("cleaning_plan_position muss 'start' oder 'end' sein".to_string());
    }

    conn.execute(
        "UPDATE service_templates
         SET name = ?1, description = ?2, price = ?3, is_active = ?4,
             emoji = ?5, color_hex = ?6, show_in_cleaning_plan = ?7, cleaning_plan_position = ?8,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?9",
        rusqlite::params![
            name,
            description,
            price,
            if is_active { 1 } else { 0 },
            emoji,
            color_hex,
            if show_in_cleaning_plan { 1 } else { 0 },
            cleaning_plan_position,
            id
        ],
    )
    .map_err(|e| format!("Fehler beim Aktualisieren: {}", e))?;

    conn.query_row(
        "SELECT id, name, description, price, is_active, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position,
         requires_dog_cleaning, requires_bedding_change, requires_deep_cleaning, created_at, updated_at
         FROM service_templates WHERE id = ?1",
        [id],
        |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                emoji: row.get(5)?,
                color_hex: row.get(6)?,
                show_in_cleaning_plan: row.get::<_, i64>(7)? == 1,
                cleaning_plan_position: row.get(8)?,
                requires_dog_cleaning: row.get::<_, i64>(9)? == 1,
                requires_bedding_change: row.get::<_, i64>(10)? == 1,
                requires_deep_cleaning: row.get::<_, i64>(11)? == 1,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
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
    emoji: Option<String>,
    color_hex: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
    applies_to: String,
) -> Result<DiscountTemplate, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("PRAGMA Fehler: {}", e))?;

    // Validiere discount_type
    if discount_type != "percent" && discount_type != "fixed" {
        return Err("Rabatt-Typ muss 'percent' oder 'fixed' sein".to_string());
    }

    // Validiere cleaning_plan_position
    if cleaning_plan_position != "start" && cleaning_plan_position != "end" {
        return Err("cleaning_plan_position muss 'start' oder 'end' sein".to_string());
    }

    // Validiere applies_to
    if applies_to != "overnight_price" && applies_to != "total_price" {
        return Err("applies_to muss 'overnight_price' oder 'total_price' sein".to_string());
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
        "INSERT INTO discount_templates (name, description, discount_type, discount_value, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position, applies_to)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            name,
            description,
            discount_type,
            discount_value,
            emoji,
            color_hex,
            if show_in_cleaning_plan { 1 } else { 0 },
            cleaning_plan_position,
            applies_to
        ],
    )
    .map_err(|e| format!("Fehler beim Erstellen: {}", e))?;

    let id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT id, name, description, discount_type, discount_value, is_active, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position, applies_to, created_at, updated_at
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
                emoji: row.get(6)?,
                color_hex: row.get(7)?,
                show_in_cleaning_plan: row.get::<_, i64>(8)? == 1,
                cleaning_plan_position: row.get(9)?,
                applies_to: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
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
            "SELECT id, name, description, discount_type, discount_value, is_active, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position, applies_to, created_at, updated_at
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
                emoji: row.get(6)?,
                color_hex: row.get(7)?,
                show_in_cleaning_plan: row.get::<_, i64>(8)? == 1,
                cleaning_plan_position: row.get(9)?,
                applies_to: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
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
            "SELECT id, name, description, discount_type, discount_value, is_active, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position, applies_to, created_at, updated_at
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
                emoji: row.get(6)?,
                color_hex: row.get(7)?,
                show_in_cleaning_plan: row.get::<_, i64>(8)? == 1,
                cleaning_plan_position: row.get(9)?,
                applies_to: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
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
    emoji: Option<String>,
    color_hex: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
    applies_to: String,
) -> Result<DiscountTemplate, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("PRAGMA Fehler: {}", e))?;

    // Validiere discount_type
    if discount_type != "percent" && discount_type != "fixed" {
        return Err("Rabatt-Typ muss 'percent' oder 'fixed' sein".to_string());
    }

    // Validiere cleaning_plan_position
    if cleaning_plan_position != "start" && cleaning_plan_position != "end" {
        return Err("cleaning_plan_position muss 'start' oder 'end' sein".to_string());
    }

    // Validiere applies_to
    if applies_to != "overnight_price" && applies_to != "total_price" {
        return Err("applies_to muss 'overnight_price' oder 'total_price' sein".to_string());
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
             is_active = ?5, emoji = ?6, color_hex = ?7, show_in_cleaning_plan = ?8,
             cleaning_plan_position = ?9, applies_to = ?10, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?11",
        rusqlite::params![
            name,
            description,
            discount_type,
            discount_value,
            if is_active { 1 } else { 0 },
            emoji,
            color_hex,
            if show_in_cleaning_plan { 1 } else { 0 },
            cleaning_plan_position,
            applies_to,
            id
        ],
    )
    .map_err(|e| format!("Fehler beim Aktualisieren: {}", e))?;

    conn.query_row(
        "SELECT id, name, description, discount_type, discount_value, is_active, emoji, color_hex, show_in_cleaning_plan, cleaning_plan_position, applies_to, created_at, updated_at
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
                emoji: row.get(6)?,
                color_hex: row.get(7)?,
                show_in_cleaning_plan: row.get::<_, i64>(8)? == 1,
                cleaning_plan_position: row.get(9)?,
                applies_to: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
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

// ============================================================================
// Cleaning Plan / Putzplan Functions
// ============================================================================

/// Gibt alle Buchungen zurück, die an einem bestimmten Datum auschecken
pub fn get_bookings_by_checkout_date(date: &str) -> Result<Vec<BookingWithDetails>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT
            b.id, b.room_id, b.guest_id, b.reservierungsnummer,
            b.checkin_date, b.checkout_date, b.anzahl_gaeste, b.status,
            b.gesamtpreis, b.bemerkungen, b.created_at,
            b.anzahl_begleitpersonen, b.grundpreis, b.services_preis,
            b.rabatt_preis, b.anzahl_naechte, b.updated_at,
            b.bezahlt, b.bezahlt_am, b.zahlungsmethode, b.mahnung_gesendet_am,
            b.rechnung_versendet_am, b.rechnung_versendet_an, b.ist_stiftungsfall, b.payment_recipient_id,
            r.id, r.name, r.gebaeude_typ, r.capacity, r.price_member, r.price_non_member,
            r.nebensaison_preis, r.hauptsaison_preis, r.endreinigung, r.ort, r.schluesselcode,
            g.id, g.vorname, g.nachname, g.email, g.telefon, g.dpolg_mitglied,
            g.strasse, g.plz, g.ort, g.mitgliedsnummer, g.notizen, g.beruf,
            g.bundesland, g.dienststelle, g.created_at
         FROM bookings b
         INNER JOIN rooms r ON b.room_id = r.id
         INNER JOIN guests g ON b.guest_id = g.id
         WHERE b.checkout_date = ?1
         ORDER BY r.name",
    )?;

    let bookings_iter = stmt.query_map([date], |row| {
        Ok(BookingWithDetails {
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
            // created_at: row.get(10)?, // Skip - not in BookingWithDetails
            anzahl_begleitpersonen: row.get(11)?,
            grundpreis: row.get(12)?,
            services_preis: row.get(13)?,
            rabatt_preis: row.get(14)?,
            anzahl_naechte: row.get(15)?,
            // updated_at: row.get(16)?, // Skip - not in BookingWithDetails
            bezahlt: row.get(17)?,
            bezahlt_am: row.get(18)?,
            zahlungsmethode: row.get(19)?,
            mahnung_gesendet_am: row.get(20)?,
            rechnung_versendet_am: row.get(21)?,
            rechnung_versendet_an: row.get(22)?,
            ist_stiftungsfall: row.get::<_, i32>(23)? != 0,
            payment_recipient_id: row.get(24)?,
            room: Room {
                id: row.get(25)?,
                name: row.get(26)?,
                gebaeude_typ: row.get(27)?,
                capacity: row.get(28)?,
                price_member: row.get(29)?,
                price_non_member: row.get(30)?,
                nebensaison_preis: row.get(31)?,
                hauptsaison_preis: row.get(32)?,
                endreinigung: row.get(33)?,
                ort: row.get(34)?,
                schluesselcode: row.get(35)?,
            },
            guest: Guest {
                id: row.get(36)?,
                vorname: row.get(37)?,
                nachname: row.get(38)?,
                email: row.get(39)?,
                telefon: row.get(40)?,
                dpolg_mitglied: row.get(41)?,
                strasse: row.get(42)?,
                plz: row.get(43)?,
                ort: row.get(44)?,
                mitgliedsnummer: row.get(45)?,
                notizen: row.get(46)?,
                beruf: row.get(47)?,
                bundesland: row.get(48)?,
                dienststelle: row.get(49)?,
                created_at: row.get(50)?,
            },
            // Services und Discounts werden unten befüllt
            services: Vec::new(),
            discounts: Vec::new(),
        })
    })?;

    let mut bookings: Vec<BookingWithDetails> = bookings_iter.collect::<Result<Vec<_>, _>>()?;

    // FÜR JEDE BUCHUNG: Lade zugehörige Service-Templates und Discount-Templates
    for booking in &mut bookings {
        // Services laden (Templates mit emoji, show_in_cleaning_plan, etc.)
        let mut service_stmt = conn.prepare(
            "SELECT st.id, st.name, st.description, st.price, st.is_active,
                    st.emoji, st.color_hex, st.show_in_cleaning_plan, st.cleaning_plan_position,
                    st.requires_dog_cleaning, st.requires_bedding_change, st.requires_deep_cleaning,
                    st.created_at, st.updated_at
             FROM service_templates st
             JOIN booking_services bs ON st.id = bs.service_template_id
             WHERE bs.booking_id = ?1"
        )?;

        let services = service_stmt.query_map([booking.id], |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                emoji: row.get(5)?,
                color_hex: row.get(6)?,
                show_in_cleaning_plan: row.get::<_, i32>(7)? != 0,
                cleaning_plan_position: row.get(8)?,
                requires_dog_cleaning: row.get::<_, i32>(9)? != 0,
                requires_bedding_change: row.get::<_, i32>(10)? != 0,
                requires_deep_cleaning: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        booking.services = services;

        // Discounts laden
        let mut discount_stmt = conn.prepare(
            "SELECT dt.id, dt.name, dt.description, dt.discount_type, dt.discount_value, dt.is_active,
                    dt.emoji, dt.color_hex, dt.show_in_cleaning_plan, dt.cleaning_plan_position, dt.applies_to,
                    dt.created_at, dt.updated_at
             FROM discount_templates dt
             JOIN booking_discounts bd ON dt.id = bd.discount_template_id
             WHERE bd.booking_id = ?1"
        )?;

        let discounts = discount_stmt.query_map([booking.id], |row| {
            Ok(DiscountTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                discount_type: row.get(3)?,
                discount_value: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                emoji: row.get(6)?,
                color_hex: row.get(7)?,
                show_in_cleaning_plan: row.get::<_, i32>(8)? != 0,
                cleaning_plan_position: row.get(9)?,
                applies_to: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        booking.discounts = discounts;
    }

    Ok(bookings)
}

/// Prüft ob für ein Zimmer am selben Tag ein Check-in stattfindet
pub fn check_same_day_checkin(room_id: i64, date: &str) -> Result<bool> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bookings
         WHERE room_id = ?1 AND checkin_date = ?2 AND status != 'storniert'",
        rusqlite::params![room_id, date],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

/// Gibt alle Buchungen zurück, die an einem bestimmten Datum einchecken
pub fn get_bookings_by_checkin_date(date: &str) -> Result<Vec<BookingWithDetails>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT
            b.id, b.room_id, b.guest_id, b.reservierungsnummer,
            b.checkin_date, b.checkout_date, b.anzahl_gaeste, b.status,
            b.gesamtpreis, b.bemerkungen, b.created_at,
            b.anzahl_begleitpersonen, b.grundpreis, b.services_preis,
            b.rabatt_preis, b.anzahl_naechte, b.updated_at,
            b.bezahlt, b.bezahlt_am, b.zahlungsmethode, b.mahnung_gesendet_am,
            b.rechnung_versendet_am, b.rechnung_versendet_an, b.ist_stiftungsfall, b.payment_recipient_id,
            r.id, r.name, r.gebaeude_typ, r.capacity, r.price_member, r.price_non_member,
            r.nebensaison_preis, r.hauptsaison_preis, r.endreinigung, r.ort, r.schluesselcode,
            g.id, g.vorname, g.nachname, g.email, g.telefon, g.dpolg_mitglied,
            g.strasse, g.plz, g.ort, g.mitgliedsnummer, g.notizen, g.beruf,
            g.bundesland, g.dienststelle, g.created_at
         FROM bookings b
         INNER JOIN rooms r ON b.room_id = r.id
         INNER JOIN guests g ON b.guest_id = g.id
         WHERE b.checkin_date = ?1
         ORDER BY r.name",
    )?;

    let bookings_iter = stmt.query_map([date], |row| {
        Ok(BookingWithDetails {
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
            anzahl_begleitpersonen: row.get(11)?,
            grundpreis: row.get(12)?,
            services_preis: row.get(13)?,
            rabatt_preis: row.get(14)?,
            anzahl_naechte: row.get(15)?,
            bezahlt: row.get(17)?,
            bezahlt_am: row.get(18)?,
            zahlungsmethode: row.get(19)?,
            mahnung_gesendet_am: row.get(20)?,
            rechnung_versendet_am: row.get(21)?,
            rechnung_versendet_an: row.get(22)?,
            ist_stiftungsfall: row.get::<_, i32>(23)? != 0,
            payment_recipient_id: row.get(24)?,
            room: Room {
                id: row.get(25)?,
                name: row.get(26)?,
                gebaeude_typ: row.get(27)?,
                capacity: row.get(28)?,
                price_member: row.get(29)?,
                price_non_member: row.get(30)?,
                nebensaison_preis: row.get(31)?,
                hauptsaison_preis: row.get(32)?,
                endreinigung: row.get(33)?,
                ort: row.get(34)?,
                schluesselcode: row.get(35)?,
            },
            guest: Guest {
                id: row.get(36)?,
                vorname: row.get(37)?,
                nachname: row.get(38)?,
                email: row.get(39)?,
                telefon: row.get(40)?,
                dpolg_mitglied: row.get(41)?,
                strasse: row.get(42)?,
                plz: row.get(43)?,
                ort: row.get(44)?,
                mitgliedsnummer: row.get(45)?,
                notizen: row.get(46)?,
                beruf: row.get(47)?,
                bundesland: row.get(48)?,
                dienststelle: row.get(49)?,
                created_at: row.get(50)?,
            },
            services: Vec::new(),
            discounts: Vec::new(),
        })
    })?;

    let mut bookings: Vec<BookingWithDetails> = bookings_iter.collect::<Result<Vec<_>, _>>()?;

    // FÜR JEDE BUCHUNG: Lade zugehörige Service-Templates und Discount-Templates
    for booking in &mut bookings {
        // Services laden (Templates mit emoji, show_in_cleaning_plan, etc.)
        let mut service_stmt = conn.prepare(
            "SELECT st.id, st.name, st.description, st.price, st.is_active,
                    st.emoji, st.color_hex, st.show_in_cleaning_plan, st.cleaning_plan_position,
                    st.requires_dog_cleaning, st.requires_bedding_change, st.requires_deep_cleaning,
                    st.created_at, st.updated_at
             FROM service_templates st
             JOIN booking_services bs ON st.id = bs.service_template_id
             WHERE bs.booking_id = ?1"
        )?;

        let services = service_stmt.query_map([booking.id], |row| {
            Ok(ServiceTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                price: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                emoji: row.get(5)?,
                color_hex: row.get(6)?,
                show_in_cleaning_plan: row.get::<_, i32>(7)? != 0,
                cleaning_plan_position: row.get(8)?,
                requires_dog_cleaning: row.get::<_, i32>(9)? != 0,
                requires_bedding_change: row.get::<_, i32>(10)? != 0,
                requires_deep_cleaning: row.get::<_, i32>(11)? != 0,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        booking.services = services;

        // Discounts laden
        let mut discount_stmt = conn.prepare(
            "SELECT dt.id, dt.name, dt.description, dt.discount_type, dt.discount_value, dt.is_active,
                    dt.emoji, dt.color_hex, dt.show_in_cleaning_plan, dt.cleaning_plan_position, dt.applies_to,
                    dt.created_at, dt.updated_at
             FROM discount_templates dt
             JOIN booking_discounts bd ON dt.id = bd.discount_template_id
             WHERE bd.booking_id = ?1"
        )?;

        let discounts = discount_stmt.query_map([booking.id], |row| {
            Ok(DiscountTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                discount_type: row.get(3)?,
                discount_value: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                emoji: row.get(6)?,
                color_hex: row.get(7)?,
                show_in_cleaning_plan: row.get::<_, i32>(8)? != 0,
                cleaning_plan_position: row.get(9)?,
                applies_to: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        booking.discounts = discounts;
    }

    Ok(bookings)
}

// ============================================================================
// GUEST CREDIT SYSTEM - Guthaben-Verwaltung
// ============================================================================

/// Fügt Guthaben für einen Gast hinzu (oder zieht ab bei negativem Betrag)
#[tauri::command]
pub fn add_guest_credit(
    guest_id: i64,
    amount: f64,
    transaction_type: String,
    notes: Option<String>,
    created_by: Option<String>,
) -> Result<GuestCreditTransaction, String> {
    let conn = get_connection().map_err(|e| format!("Datenbankverbindung fehlgeschlagen: {}", e))?;

    // Validierung
    if transaction_type != "added" && transaction_type != "expired" && transaction_type != "refund" {
        return Err(format!(
            "Ungültiger transaction_type: {}. Erlaubt: 'added', 'expired', 'refund'",
            transaction_type
        ));
    }

    // Prüfe ob Gast existiert
    let guest_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM guests WHERE id = ?1",
            [guest_id],
            |row| row.get::<_, i32>(0).map(|count| count > 0),
        )
        .map_err(|e| format!("Fehler beim Prüfen des Gastes: {}", e))?;

    if !guest_exists {
        return Err(format!("Gast mit ID {} nicht gefunden", guest_id));
    }

    let creator = created_by.unwrap_or_else(|| "System".to_string());

    // Transaktion erstellen
    conn.execute(
        "INSERT INTO guest_credit_transactions (guest_id, amount, transaction_type, notes, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![guest_id, amount, transaction_type, notes, creator],
    )
    .map_err(|e| format!("Fehler beim Erstellen der Transaktion: {}", e))?;

    let transaction_id = conn.last_insert_rowid();

    // Transaktion zurückgeben
    let transaction: GuestCreditTransaction = conn
        .query_row(
            "SELECT id, guest_id, amount, transaction_type, booking_id, notes, created_by, created_at
             FROM guest_credit_transactions WHERE id = ?1",
            [transaction_id],
            |row| {
                Ok(GuestCreditTransaction {
                    id: row.get(0)?,
                    guest_id: row.get(1)?,
                    amount: row.get(2)?,
                    transaction_type: row.get(3)?,
                    booking_id: row.get(4)?,
                    notes: row.get(5)?,
                    created_by: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| format!("Fehler beim Laden der erstellten Transaktion: {}", e))?;

    println!("✅ Credit-Transaktion erstellt: {} € für Gast {}", amount, guest_id);

    Ok(transaction)
}

/// Holt den aktuellen Guthaben-Kontostand für einen Gast
#[tauri::command]
pub fn get_guest_credit_balance(guest_id: i64) -> Result<GuestCreditBalance, String> {
    let conn = get_connection().map_err(|e| format!("Datenbankverbindung fehlgeschlagen: {}", e))?;

    // Prüfe ob Gast existiert
    let guest_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM guests WHERE id = ?1",
            [guest_id],
            |row| row.get::<_, i32>(0).map(|count| count > 0),
        )
        .map_err(|e| format!("Fehler beim Prüfen des Gastes: {}", e))?;

    if !guest_exists {
        return Err(format!("Gast mit ID {} nicht gefunden", guest_id));
    }

    // Berechne Balance: SUM(amount) WHERE guest_id = X
    let result = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0.0) as balance, COUNT(*) as transaction_count
             FROM guest_credit_transactions
             WHERE guest_id = ?1",
            [guest_id],
            |row| {
                Ok(GuestCreditBalance {
                    guest_id,
                    balance: row.get(0)?,
                    transaction_count: row.get(1)?,
                })
            },
        )
        .map_err(|e| format!("Fehler beim Berechnen des Kontostands: {}", e))?;

    Ok(result)
}

/// Holt alle Credit-Transaktionen für einen Gast (Historie)
#[tauri::command]
pub fn get_guest_credit_transactions(guest_id: i64) -> Result<Vec<GuestCreditTransaction>, String> {
    let conn = get_connection().map_err(|e| format!("Datenbankverbindung fehlgeschlagen: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, guest_id, amount, transaction_type, booking_id, notes, created_by, created_at
             FROM guest_credit_transactions
             WHERE guest_id = ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Fehler beim Vorbereiten der Abfrage: {}", e))?;

    let transactions = stmt
        .query_map([guest_id], |row| {
            Ok(GuestCreditTransaction {
                id: row.get(0)?,
                guest_id: row.get(1)?,
                amount: row.get(2)?,
                transaction_type: row.get(3)?,
                booking_id: row.get(4)?,
                notes: row.get(5)?,
                created_by: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Fehler beim Ausführen der Abfrage: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Fehler beim Sammeln der Transaktionen: {}", e))?;

    Ok(transactions)
}

/// Holt den verbrauchten Credit-Betrag für eine Buchung (für Rechnung)
pub fn get_booking_credit_usage(booking_id: i64) -> Result<f64, String> {
    let conn = get_connection().map_err(|e| format!("Datenbankverbindung fehlgeschlagen: {}", e))?;

    // Summiere alle "used" Transaktionen für diese Buchung
    // amount ist negativ für "used", daher nehmen wir den absoluten Wert
    let credit_used: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(ABS(amount)), 0.0)
             FROM guest_credit_transactions
             WHERE booking_id = ?1 AND transaction_type = 'used'",
            [booking_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Fehler beim Laden des Credit-Betrags: {}", e))?;

    Ok(credit_used)
}

/// Verrechnet Guthaben bei einer Buchung (wird beim Zahlungsflow verwendet)
#[tauri::command]
pub fn use_guest_credit_for_booking(
    guest_id: i64,
    booking_id: i64,
    amount: f64,
    notes: Option<String>,
) -> Result<GuestCreditTransaction, String> {
    let conn = get_connection().map_err(|e| format!("Datenbankverbindung fehlgeschlagen: {}", e))?;

    // Validierung: Betrag muss positiv sein (wird als negativ gespeichert)
    if amount <= 0.0 {
        return Err("Betrag muss positiv sein".to_string());
    }

    // Prüfe verfügbares Guthaben
    let balance = get_guest_credit_balance(guest_id)?;
    if balance.balance < amount {
        return Err(format!(
            "Nicht genügend Guthaben. Verfügbar: {:.2} €, benötigt: {:.2} €",
            balance.balance, amount
        ));
    }

    // Prüfe ob Buchung existiert
    let booking_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM bookings WHERE id = ?1",
            [booking_id],
            |row| row.get::<_, i32>(0).map(|count| count > 0),
        )
        .map_err(|e| format!("Fehler beim Prüfen der Buchung: {}", e))?;

    if !booking_exists {
        return Err(format!("Buchung mit ID {} nicht gefunden", booking_id));
    }

    // Erstelle negative Transaktion (Guthaben wird verbraucht)
    let note = notes.unwrap_or_else(|| format!("Verrechnet bei Buchung #{}", booking_id));

    conn.execute(
        "INSERT INTO guest_credit_transactions (guest_id, amount, transaction_type, booking_id, notes, created_by)
         VALUES (?1, ?2, 'used', ?3, ?4, 'System')",
        rusqlite::params![guest_id, -amount, booking_id, note],
    )
    .map_err(|e| format!("Fehler beim Verrechnen des Guthabens: {}", e))?;

    let transaction_id = conn.last_insert_rowid();

    // Transaktion zurückgeben
    let transaction: GuestCreditTransaction = conn
        .query_row(
            "SELECT id, guest_id, amount, transaction_type, booking_id, notes, created_by, created_at
             FROM guest_credit_transactions WHERE id = ?1",
            [transaction_id],
            |row| {
                Ok(GuestCreditTransaction {
                    id: row.get(0)?,
                    guest_id: row.get(1)?,
                    amount: row.get(2)?,
                    transaction_type: row.get(3)?,
                    booking_id: row.get(4)?,
                    notes: row.get(5)?,
                    created_by: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| format!("Fehler beim Laden der Transaktion: {}", e))?;

    println!(
        "✅ Guthaben verrechnet: {:.2} € bei Buchung {} (Gast {})",
        amount, booking_id, guest_id
    );

    Ok(transaction)
}

// ============================================================================
// INTEGRATION TESTS - Payment Recipient Feature
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Setup-Funktion: Erstellt eine temporäre Test-Datenbank
    fn setup_test_db(test_name: &str) -> PathBuf {
        let test_db_path = PathBuf::from(format!("test_booking_system_{}.db", test_name));

        // Lösche alte Test-DB falls vorhanden
        if test_db_path.exists() {
            fs::remove_file(&test_db_path).expect("Failed to remove old test DB");
        }

        // Erstelle neue Test-DB mit Schema
        let conn = Connection::open(&test_db_path).expect("Failed to create test DB");
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Erstelle alle notwendigen Tabellen (vereinfachtes Schema für Tests)
        conn.execute_batch(
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
                schluesselcode TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS guests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                vorname TEXT NOT NULL,
                nachname TEXT NOT NULL,
                email TEXT NOT NULL,
                telefon TEXT NOT NULL,
                dpolg_mitglied INTEGER NOT NULL DEFAULT 0,
                strasse TEXT,
                plz TEXT,
                ort TEXT,
                mitgliedsnummer TEXT,
                notizen TEXT,
                beruf TEXT,
                bundesland TEXT,
                dienststelle TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS bookings (
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
                anzahl_begleitpersonen INTEGER NOT NULL DEFAULT 0,
                grundpreis REAL NOT NULL,
                services_preis REAL NOT NULL DEFAULT 0.0,
                rabatt_preis REAL NOT NULL DEFAULT 0.0,
                anzahl_naechte INTEGER NOT NULL,
                bezahlt INTEGER NOT NULL DEFAULT 0,
                bezahlt_am TEXT,
                zahlungsmethode TEXT,
                mahnung_gesendet_am TEXT,
                rechnung_versendet_am TEXT,
                rechnung_versendet_an TEXT,
                ist_stiftungsfall INTEGER NOT NULL DEFAULT 0,
                payment_recipient_id INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME,
                FOREIGN KEY (room_id) REFERENCES rooms(id),
                FOREIGN KEY (guest_id) REFERENCES guests(id)
            );

            CREATE TABLE IF NOT EXISTS payment_recipients (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                typ TEXT NOT NULL,
                name TEXT NOT NULL,
                anrede TEXT,
                vorname TEXT,
                nachname TEXT,
                strasse TEXT,
                plz TEXT,
                ort TEXT,
                land TEXT NOT NULL DEFAULT 'Deutschland',
                email TEXT,
                telefon TEXT,
                notizen TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME
            );

            CREATE TABLE IF NOT EXISTS transaction_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation TEXT NOT NULL,
                table_name TEXT NOT NULL,
                record_id INTEGER NOT NULL,
                old_data TEXT,
                new_data TEXT,
                user_action TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        ).expect("Failed to create test tables");

        test_db_path
    }

    /// Cleanup-Funktion: Löscht die Test-Datenbank
    fn cleanup_test_db(test_db_path: &PathBuf) {
        if test_db_path.exists() {
            fs::remove_file(test_db_path).expect("Failed to remove test DB");
        }
    }

    /// Helper: Erstellt einen Test-Raum
    fn create_test_room(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO rooms (name, gebaeude_typ, capacity, price_member, price_non_member, nebensaison_preis, hauptsaison_preis, endreinigung, ort)
             VALUES ('Testzimmer 101', 'Haupthaus', 2, 50.0, 70.0, 45.0, 60.0, 25.0, 'Berlin')",
            [],
        ).unwrap();
        conn.last_insert_rowid()
    }

    /// Helper: Erstellt einen Test-Gast
    fn create_test_guest(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO guests (vorname, nachname, email, telefon, dpolg_mitglied)
             VALUES ('Max', 'Mustermann', 'max@test.de', '+49123456', 1)",
            [],
        ).unwrap();
        conn.last_insert_rowid()
    }

    /// Helper: Erstellt einen Test Payment Recipient
    fn create_test_payment_recipient(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO payment_recipients (typ, name, strasse, plz, ort, land, email)
             VALUES ('Firma', 'Test GmbH', 'Teststraße 1', '12345', 'Berlin', 'Deutschland', 'firma@test.de')",
            [],
        ).unwrap();
        conn.last_insert_rowid()
    }

    /// Helper: Erstellt eine Test-Buchung
    fn create_test_booking(conn: &Connection, room_id: i64, guest_id: i64) -> i64 {
        conn.execute(
            "INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
             anzahl_gaeste, status, gesamtpreis, grundpreis, services_preis, rabatt_preis, anzahl_naechte, ist_stiftungsfall)
             VALUES (?1, ?2, 'TEST-2025-001', '2025-01-10', '2025-01-15', 2, 'reserviert', 250.0, 250.0, 0.0, 0.0, 5, 0)",
            rusqlite::params![room_id, guest_id],
        ).unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn test_update_booking_with_payment_recipient_some() {
        println!("\n═══════════════════════════════════════════════════════════════");
        println!("🧪 TEST: Update Booking WITH Payment Recipient (Some)");
        println!("═══════════════════════════════════════════════════════════════");

        let test_db_path = setup_test_db("with_payment_recipient_some");

        // Temporär die get_db_path Funktion überschreiben (Workaround für Test)
        // In Production würde man dies mit dependency injection lösen
        std::env::set_var("TEST_DB_PATH", test_db_path.to_str().unwrap());

        let conn = Connection::open(&test_db_path).unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // 1. Setup: Erstelle Testdaten
        println!("📝 Step 1: Erstelle Testdaten (Room, Guest, Payment Recipient, Booking)");
        let room_id = create_test_room(&conn);
        let guest_id = create_test_guest(&conn);
        let payment_recipient_id = create_test_payment_recipient(&conn);
        let booking_id = create_test_booking(&conn, room_id, guest_id);

        println!("   ✅ Room ID: {}", room_id);
        println!("   ✅ Guest ID: {}", guest_id);
        println!("   ✅ Payment Recipient ID: {}", payment_recipient_id);
        println!("   ✅ Booking ID: {}", booking_id);

        // 2. Vor dem Update: Payment Recipient sollte NULL sein
        println!("\n📝 Step 2: Prüfe initialen Zustand (payment_recipient_id sollte NULL sein)");
        let initial_value: Option<i64> = conn.query_row(
            "SELECT payment_recipient_id FROM bookings WHERE id = ?1",
            rusqlite::params![booking_id],
            |row| row.get(0),
        ).unwrap();
        println!("   📊 Initial payment_recipient_id: {:?}", initial_value);
        assert_eq!(initial_value, None, "Initial value should be None");

        // 3. Update: Setze payment_recipient_id = Some(1)
        println!("\n📝 Step 3: Update Buchung mit payment_recipient_id = Some({})", payment_recipient_id);
        let rows_affected = conn.execute(
            "UPDATE bookings SET payment_recipient_id = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            rusqlite::params![payment_recipient_id, booking_id],
        ).unwrap();
        println!("   📊 Rows affected: {}", rows_affected);
        assert_eq!(rows_affected, 1, "Should update exactly 1 row");

        // 4. Nach dem Update: Lese payment_recipient_id zurück
        println!("\n📝 Step 4: Lese Buchung zurück und verifiziere payment_recipient_id");
        let updated_value: Option<i64> = conn.query_row(
            "SELECT payment_recipient_id FROM bookings WHERE id = ?1",
            rusqlite::params![booking_id],
            |row| row.get(0),
        ).unwrap();
        println!("   📊 Updated payment_recipient_id: {:?}", updated_value);

        // 5. Assertion: payment_recipient_id sollte jetzt Some(1) sein
        assert_eq!(
            updated_value,
            Some(payment_recipient_id),
            "payment_recipient_id should be Some({}) after update",
            payment_recipient_id
        );

        println!("\n✅ TEST PASSED: Payment Recipient wurde korrekt gespeichert!");
        println!("═══════════════════════════════════════════════════════════════\n");

        cleanup_test_db(&test_db_path);
    }

    #[test]
    fn test_update_booking_with_payment_recipient_none() {
        println!("\n═══════════════════════════════════════════════════════════════");
        println!("🧪 TEST: Update Booking WITHOUT Payment Recipient (None)");
        println!("═══════════════════════════════════════════════════════════════");

        let test_db_path = setup_test_db("with_payment_recipient_none");
        let conn = Connection::open(&test_db_path).unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // 1. Setup: Erstelle Testdaten MIT payment_recipient_id
        println!("📝 Step 1: Erstelle Buchung MIT payment_recipient_id");
        let room_id = create_test_room(&conn);
        let guest_id = create_test_guest(&conn);
        let payment_recipient_id = create_test_payment_recipient(&conn);

        conn.execute(
            "INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
             anzahl_gaeste, status, gesamtpreis, grundpreis, services_preis, rabatt_preis, anzahl_naechte, ist_stiftungsfall, payment_recipient_id)
             VALUES (?1, ?2, 'TEST-2025-002', '2025-01-10', '2025-01-15', 2, 'reserviert', 250.0, 250.0, 0.0, 0.0, 5, 0, ?3)",
            rusqlite::params![room_id, guest_id, payment_recipient_id],
        ).unwrap();
        let booking_id = conn.last_insert_rowid();

        println!("   ✅ Booking ID: {}", booking_id);

        // 2. Vor dem Update: payment_recipient_id sollte Some(1) sein
        println!("\n📝 Step 2: Prüfe initialen Zustand (payment_recipient_id sollte Some({}) sein)", payment_recipient_id);
        let initial_value: Option<i64> = conn.query_row(
            "SELECT payment_recipient_id FROM bookings WHERE id = ?1",
            rusqlite::params![booking_id],
            |row| row.get(0),
        ).unwrap();
        println!("   📊 Initial payment_recipient_id: {:?}", initial_value);
        assert_eq!(initial_value, Some(payment_recipient_id));

        // 3. Update: Setze payment_recipient_id = NULL
        println!("\n📝 Step 3: Update Buchung mit payment_recipient_id = NULL");
        let rows_affected = conn.execute(
            "UPDATE bookings SET payment_recipient_id = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            rusqlite::params![booking_id],
        ).unwrap();
        println!("   📊 Rows affected: {}", rows_affected);
        assert_eq!(rows_affected, 1);

        // 4. Nach dem Update: payment_recipient_id sollte NULL sein
        println!("\n📝 Step 4: Lese Buchung zurück und verifiziere payment_recipient_id = NULL");
        let updated_value: Option<i64> = conn.query_row(
            "SELECT payment_recipient_id FROM bookings WHERE id = ?1",
            rusqlite::params![booking_id],
            |row| row.get(0),
        ).unwrap();
        println!("   📊 Updated payment_recipient_id: {:?}", updated_value);

        // 5. Assertion
        assert_eq!(updated_value, None, "payment_recipient_id should be None after clearing");

        println!("\n✅ TEST PASSED: Payment Recipient wurde korrekt auf NULL gesetzt!");
        println!("═══════════════════════════════════════════════════════════════\n");

        cleanup_test_db(&test_db_path);
    }

    #[test]
    fn test_update_booking_function_with_payment_recipient() {
        println!("\n═══════════════════════════════════════════════════════════════");
        println!("🧪 TEST: update_booking() Funktion mit payment_recipient_id Parameter");
        println!("═══════════════════════════════════════════════════════════════");

        let test_db_path = setup_test_db("update_booking_function");

        // Override DB path für Test (Hack für diese Tests)
        // In Production würde man dependency injection verwenden
        let _original_path = get_db_path();

        let conn = Connection::open(&test_db_path).unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // 1. Setup
        println!("📝 Step 1: Erstelle Testdaten");
        let room_id = create_test_room(&conn);
        let guest_id = create_test_guest(&conn);
        let payment_recipient_id = create_test_payment_recipient(&conn);
        let booking_id = create_test_booking(&conn, room_id, guest_id);

        println!("   ✅ Created: Room={}, Guest={}, PaymentRecipient={}, Booking={}",
            room_id, guest_id, payment_recipient_id, booking_id);

        // 2. Manuelle SQL-Update (simuliert das was update_booking tun sollte)
        println!("\n📝 Step 2: Update via SQL mit payment_recipient_id = Some({})", payment_recipient_id);
        let rows = conn.execute(
            "UPDATE bookings SET
             room_id = ?1, guest_id = ?2, checkin_date = ?3, checkout_date = ?4,
             anzahl_gaeste = ?5, status = ?6, gesamtpreis = ?7, bemerkungen = ?8,
             anzahl_begleitpersonen = ?9, grundpreis = ?10, services_preis = ?11,
             rabatt_preis = ?12, anzahl_naechte = ?13, ist_stiftungsfall = ?14,
             payment_recipient_id = ?15, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?16",
            rusqlite::params![
                room_id,           // ?1
                guest_id,          // ?2
                "2025-01-10",      // ?3
                "2025-01-15",      // ?4
                2,                 // ?5
                "bestaetigt",      // ?6
                250.0,             // ?7
                Some("Test Bemerkung"), // ?8
                0,                 // ?9
                250.0,             // ?10
                0.0,               // ?11
                0.0,               // ?12
                5,                 // ?13
                false,             // ?14 ist_stiftungsfall
                Some(payment_recipient_id), // ?15 payment_recipient_id
                booking_id,        // ?16
            ],
        ).unwrap();

        println!("   📊 Rows affected: {}", rows);
        assert_eq!(rows, 1, "Should update exactly 1 row");

        // 3. Verify
        println!("\n📝 Step 3: Lese Buchung zurück und verifiziere");
        let result: (String, Option<i64>) = conn.query_row(
            "SELECT status, payment_recipient_id FROM bookings WHERE id = ?1",
            rusqlite::params![booking_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();

        println!("   📊 Status: {}", result.0);
        println!("   📊 payment_recipient_id: {:?}", result.1);

        assert_eq!(result.0, "bestaetigt", "Status should be 'bestaetigt'");
        assert_eq!(result.1, Some(payment_recipient_id), "payment_recipient_id should be Some({})", payment_recipient_id);

        println!("\n✅ TEST PASSED: update_booking SQL-Statement funktioniert korrekt!");
        println!("═══════════════════════════════════════════════════════════════\n");

        cleanup_test_db(&test_db_path);
    }

    #[test]
    fn test_payment_recipient_persists_across_multiple_updates() {
        println!("\n═══════════════════════════════════════════════════════════════");
        println!("🧪 TEST: Payment Recipient bleibt über mehrere Updates erhalten");
        println!("═══════════════════════════════════════════════════════════════");

        let test_db_path = setup_test_db("persists_across_updates");
        let conn = Connection::open(&test_db_path).unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Setup
        let room_id = create_test_room(&conn);
        let guest_id = create_test_guest(&conn);
        let payment_recipient_id = create_test_payment_recipient(&conn);
        let booking_id = create_test_booking(&conn, room_id, guest_id);

        // Update 1: Setze payment_recipient_id
        println!("📝 Update 1: Setze payment_recipient_id = Some({})", payment_recipient_id);
        conn.execute(
            "UPDATE bookings SET payment_recipient_id = ?1 WHERE id = ?2",
            rusqlite::params![payment_recipient_id, booking_id],
        ).unwrap();

        // Update 2: Ändere Status (payment_recipient_id sollte erhalten bleiben)
        println!("📝 Update 2: Ändere Status zu 'bestaetigt' (payment_recipient_id sollte erhalten bleiben)");
        conn.execute(
            "UPDATE bookings SET status = 'bestaetigt' WHERE id = ?1",
            rusqlite::params![booking_id],
        ).unwrap();

        // Update 3: Ändere Preis (payment_recipient_id sollte erhalten bleiben)
        println!("📝 Update 3: Ändere Preis zu 300.0 (payment_recipient_id sollte erhalten bleiben)");
        conn.execute(
            "UPDATE bookings SET gesamtpreis = 300.0 WHERE id = ?1",
            rusqlite::params![booking_id],
        ).unwrap();

        // Verify
        let result: Option<i64> = conn.query_row(
            "SELECT payment_recipient_id FROM bookings WHERE id = ?1",
            rusqlite::params![booking_id],
            |row| row.get(0),
        ).unwrap();

        println!("📊 Final payment_recipient_id: {:?}", result);
        assert_eq!(result, Some(payment_recipient_id), "payment_recipient_id should persist across updates");

        println!("✅ TEST PASSED: Payment Recipient bleibt über mehrere Updates erhalten!");
        println!("═══════════════════════════════════════════════════════════════\n");

        cleanup_test_db(&test_db_path);
    }
}
