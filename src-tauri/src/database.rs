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
    pub status: String,
    pub gesamtpreis: f64,
    pub bemerkungen: Option<String>,
    pub room: Room,
    pub guest: Guest,
}

pub fn get_db_path() -> PathBuf {
    // Use a simple path in the project directory for now
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
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\nvielen Dank für Ihre Buchung!\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Check-in: {checkin_date}\n\
                Check-out: {checkout_date}\n\
                Anzahl Gäste: {anzahl_gaeste}\n\
                Anzahl Nächte: {anzahl_naechte}\n\
                Gesamtpreis: {gesamtpreis} €\n\n\
                Wir freuen uns auf Ihren Besuch!\n\n\
                Mit freundlichen Grüßen\n\
                DPolG Stiftung Buchungssystem",
                "Standard-Template für Buchungsbestätigungen"
            ],
        )?;

        // Check-in Reminder Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "reminder",
                "Erinnerung - Check-in morgen für Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\nwir möchten Sie daran erinnern, dass Ihr Check-in morgen ansteht.\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Check-in: {checkin_date}\n\
                Check-out: {checkout_date}\n\n\
                Wir freuen uns auf Ihren Besuch!\n\n\
                Mit freundlichen Grüßen\n\
                DPolG Stiftung Buchungssystem",
                "Reminder-Email einen Tag vor Check-in"
            ],
        )?;

        // Rechnung Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "invoice",
                "Rechnung - Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\nanbei erhalten Sie die Rechnung für Ihren Aufenthalt.\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Zeitraum: {checkin_date} - {checkout_date}\n\
                Anzahl Nächte: {anzahl_naechte}\n\n\
                Grundpreis: {grundpreis} €\n\
                Services: {services_preis} €\n\
                Rabatt: -{rabatt_preis} €\n\
                Gesamtpreis: {gesamtpreis} €\n\n\
                Vielen Dank für Ihren Aufenthalt!\n\n\
                Mit freundlichen Grüßen\n\
                DPolG Stiftung Buchungssystem",
                "Rechnungs-Email nach Check-out"
            ],
        )?;

        // Zahlungserinnerung Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "payment_reminder",
                "Zahlungserinnerung - Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\nwir möchten Sie freundlich daran erinnern, dass die Zahlung für Ihre Buchung noch aussteht.\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Zeitraum: {checkin_date} - {checkout_date}\n\
                Gesamtbetrag: {gesamtpreis} €\n\n\
                Bitte überweisen Sie den Betrag innerhalb der nächsten 7 Tage.\n\n\
                Bei Fragen stehen wir Ihnen gerne zur Verfügung.\n\n\
                Mit freundlichen Grüßen\n\
                DPolG Stiftung Buchungssystem",
                "Zahlungserinnerung nach 14 Tagen"
            ],
        )?;

        // Stornierungsbestätigung Template
        conn.execute(
            "INSERT INTO email_templates (template_name, subject, body, description) VALUES (?1, ?2, ?3, ?4)",
            [
                "cancellation",
                "Stornierungsbestätigung - Reservierung {reservierungsnummer}",
                "Sehr geehrte/r {gast_vorname} {gast_nachname},\n\nwir bestätigen die Stornierung Ihrer Buchung.\n\n\
                Reservierungsnummer: {reservierungsnummer}\n\
                Zimmer: {zimmer_name}\n\
                Ursprünglicher Zeitraum: {checkin_date} - {checkout_date}\n\
                Storniert am: {heute}\n\n\
                Falls Sie erneut buchen möchten, kontaktieren Sie uns gerne.\n\n\
                Mit freundlichen Grüßen\n\
                DPolG Stiftung Buchungssystem",
                "Bestätigung bei Stornierung"
            ],
        )?;
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

pub fn get_rooms() -> Result<Vec<Room>> {
    let conn = Connection::open(get_db_path())?;

    // WICHTIG: Foreign Keys aktivieren für diese Connection
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare("SELECT id, name, gebaeude_typ, capacity, price_member, price_non_member, ort, schluesselcode FROM rooms")?;

    let rooms = stmt.query_map([], |row| {
        Ok(Room {
            id: row.get(0)?,
            name: row.get(1)?,
            gebaeude_typ: row.get(2)?,
            capacity: row.get(3)?,
            price_member: row.get(4)?,
            price_non_member: row.get(5)?,
            ort: row.get(6)?,
            schluesselcode: row.get(7)?,
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
            b.checkin_date, b.checkout_date, b.anzahl_gaeste,
            b.status, b.gesamtpreis, b.bemerkungen,
            r.id, r.name, r.gebaeude_typ, r.capacity, r.price_member, r.price_non_member, r.ort, r.schluesselcode,
            g.id, g.vorname, g.nachname, g.email, g.telefon, g.dpolg_mitglied,
            g.strasse, g.plz, g.ort, g.mitgliedsnummer, g.notizen, g.beruf, g.bundesland, g.dienststelle, g.created_at
         FROM bookings b
         JOIN rooms r ON b.room_id = r.id
         JOIN guests g ON b.guest_id = g.id"
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
            status: row.get(7)?,
            gesamtpreis: row.get(8)?,
            bemerkungen: row.get(9)?,
            room: Room {
                id: row.get(10)?,
                name: row.get(11)?,
                gebaeude_typ: row.get(12)?,
                capacity: row.get(13)?,
                price_member: row.get(14)?,
                price_non_member: row.get(15)?,
                ort: row.get(16)?,
                schluesselcode: row.get(17)?,
            },
            guest: Guest {
                id: row.get(18)?,
                vorname: row.get(19)?,
                nachname: row.get(20)?,
                email: row.get(21)?,
                telefon: row.get(22)?,
                dpolg_mitglied: row.get(23)?,
                strasse: row.get(24)?,
                plz: row.get(25)?,
                ort: row.get(26)?,
                mitgliedsnummer: row.get(27)?,
                notizen: row.get(28)?,
                beruf: row.get(29)?,
                bundesland: row.get(30)?,
                dienststelle: row.get(31)?,
                created_at: row.get(32)?,
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
    get_guest_by_id(id)
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

    get_guest_by_id(id)
}

/// Löscht einen Gast (schlägt fehl, wenn der Gast Buchungen hat)
pub fn delete_guest(id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute("DELETE FROM guests WHERE id = ?1", rusqlite::params![id])?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
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
    ort: String,
    schluesselcode: Option<String>,
) -> Result<Room> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO rooms (name, gebaeude_typ, capacity, price_member, price_non_member, ort, schluesselcode)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            name,
            gebaeude_typ,
            capacity,
            price_member,
            price_non_member,
            ort,
            schluesselcode
        ],
    )?;

    let id = conn.last_insert_rowid();
    get_room_by_id(id)
}

/// Aktualisiert einen existierenden Raum
pub fn update_room(
    id: i64,
    name: String,
    gebaeude_typ: String,
    capacity: i32,
    price_member: f64,
    price_non_member: f64,
    ort: String,
    schluesselcode: Option<String>,
) -> Result<Room> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute(
        "UPDATE rooms SET
         name = ?1, gebaeude_typ = ?2, capacity = ?3, price_member = ?4,
         price_non_member = ?5, ort = ?6, schluesselcode = ?7
         WHERE id = ?8",
        rusqlite::params![
            name,
            gebaeude_typ,
            capacity,
            price_member,
            price_non_member,
            ort,
            schluesselcode,
            id
        ],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_room_by_id(id)
}

/// Löscht einen Raum (schlägt fehl, wenn der Raum Buchungen hat)
pub fn delete_room(id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute("DELETE FROM rooms WHERE id = ?1", rusqlite::params![id])?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
}

/// Gibt einen Raum anhand der ID zurück
pub fn get_room_by_id(id: i64) -> Result<Room> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, name, gebaeude_typ, capacity, price_member, price_non_member, ort, schluesselcode
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
                ort: row.get(6)?,
                schluesselcode: row.get(7)?,
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
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
         anzahl_gaeste, status, gesamtpreis, bemerkungen, anzahl_begleitpersonen,
         grundpreis, services_preis, rabatt_preis, anzahl_naechte)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            room_id,
            guest_id,
            reservierungsnummer,
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
    get_booking_by_id(id)
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

    get_booking_by_id(id)
}

/// Löscht eine Buchung (CASCADE löscht auch Services, Gäste, Rabatte)
pub fn delete_booking(id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let rows_affected = conn.execute("DELETE FROM bookings WHERE id = ?1", rusqlite::params![id])?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
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

    let rows_affected = conn.execute(
        "UPDATE bookings SET
         room_id = ?1, checkin_date = ?2, checkout_date = ?3, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?4",
        rusqlite::params![room_id, checkin_date, checkout_date, id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_booking_by_id(id)
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
    use chrono::Local;
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let bezahlt_am = Local::now().format("%Y-%m-%d").to_string();

    let rows_affected = conn.execute(
        "UPDATE bookings SET bezahlt = 1, bezahlt_am = ?1, zahlungsmethode = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        rusqlite::params![bezahlt_am, zahlungsmethode, id],
    )?;

    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_booking_by_id(id)
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