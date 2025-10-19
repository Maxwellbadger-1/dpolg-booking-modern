// ============================================================================
// VALIDATION MODULE - Phase 2: Business Logic & Validierung
// ============================================================================
//
// Dieses Modul enthält alle Validierungsfunktionen für das DPolG Stiftung Buchungssystem.
// Entwickelt nach TDD-Prinzipien: Jede Funktion hat umfassende Unit-Tests.

use chrono::NaiveDate;
use once_cell::sync::Lazy;
use regex::Regex;
use rusqlite::Connection;

// ============================================================================
// DATE VALIDATION FUNCTIONS
// ============================================================================

/// Validiert und parst ein Datum im Format YYYY-MM-DD
///
/// # Arguments
/// * `date_str` - Datum-String im Format YYYY-MM-DD
///
/// # Returns
/// * `Ok(NaiveDate)` - Erfolgreich geparster Datum
/// * `Err(String)` - Fehlermeldung in Deutsch
///
/// # Example
/// ```
/// let date = validate_date_format("2025-10-15")?;
/// ```
pub fn validate_date_format(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| "Datum muss im Format YYYY-MM-DD sein".to_string())
}

/// Validiert einen Datumsbereich für Check-in und Check-out
///
/// Prüft:
/// - Beide Daten sind gültig
/// - Check-out ist nach Check-in
/// - Mindestens 1 Nacht Aufenthalt
///
/// # Arguments
/// * `checkin` - Check-in Datum als String (YYYY-MM-DD)
/// * `checkout` - Check-out Datum als String (YYYY-MM-DD)
///
/// # Returns
/// * `Ok(())` - Datumsbereich ist gültig
/// * `Err(String)` - Fehlermeldung in Deutsch
pub fn validate_date_range(checkin: &str, checkout: &str) -> Result<(), String> {
    let checkin_date = validate_date_format(checkin)?;
    let checkout_date = validate_date_format(checkout)?;

    if checkout_date <= checkin_date {
        return Err("Check-out muss nach Check-in liegen (mindestens 1 Nacht)".to_string());
    }

    Ok(())
}

// ============================================================================
// EMAIL VALIDATION
// ============================================================================

// Statische Regex für Email-Validierung (RFC5322-konform, vereinfacht)
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

/// Validiert eine E-Mail-Adresse
///
/// Verwendet RFC5322-konforme Regex-Validierung
///
/// # Arguments
/// * `email` - E-Mail-Adresse als String
///
/// # Returns
/// * `Ok(())` - E-Mail ist gültig
/// * `Err(String)` - Fehlermeldung in Deutsch
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.trim().is_empty() {
        return Err("E-Mail-Adresse darf nicht leer sein".to_string());
    }

    if !EMAIL_REGEX.is_match(email) {
        return Err("Ungültige E-Mail-Adresse".to_string());
    }

    Ok(())
}

// ============================================================================
// RESERVATION NUMBER GENERATION
// ============================================================================

/// Generiert eine Reservierungsnummer basierend auf Jahr und Buchungs-ID
///
/// Format: YYYY-BOOKING_ID
/// - YYYY: Aktuelles Jahr
/// - BOOKING_ID: Buchungs-ID aus der Datenbank
///
/// # Arguments
/// * `booking_id` - Die ID der Buchung aus der Datenbank
///
/// # Returns
/// * Reservierungsnummer als String
///
/// # Example
/// ```
/// let nummer = generate_reservation_number(123);
/// // Beispiel: "2025-123"
/// ```
pub fn generate_reservation_number_with_id(booking_id: i64) -> String {
    let now = crate::time_utils::now_utc_plus_2();
    let year = now.format("%Y").to_string();

    // Format: YYYY-BOOKING_ID
    format!("{}-{}", year, booking_id)
}

// ============================================================================
// ROOM AVAILABILITY
// ============================================================================

/// Prüft die Verfügbarkeit eines Raums für einen Datumsbereich
///
/// Ein Raum ist NICHT verfügbar, wenn es überlappende Buchungen gibt.
/// Überlappung bedeutet: Eine Buchung existiert, deren Datumsbereich sich mit
/// dem gewünschten Zeitraum überschneidet.
///
/// # Arguments
/// * `room_id` - ID des zu prüfenden Raums
/// * `checkin` - Gewünschtes Check-in Datum (YYYY-MM-DD)
/// * `checkout` - Gewünschtes Check-out Datum (YYYY-MM-DD)
/// * `exclude_booking_id` - Optional: Buchungs-ID die ignoriert werden soll (bei Updates)
/// * `conn` - SQLite Datenbankverbindung
///
/// # Returns
/// * `Ok(true)` - Raum ist verfügbar
/// * `Ok(false)` - Raum ist NICHT verfügbar (Überschneidung)
/// * `Err(String)` - Datenbankfehler oder ungültiges Datum
///
/// # Overlap Detection Logic
/// Zwei Zeiträume überschneiden sich, wenn:
/// - Der neue Check-in VOR dem bestehenden Check-out liegt UND
/// - Der neue Check-out NACH dem bestehenden Check-in liegt
///
/// SQL: `checkin_date < ?2 AND checkout_date > ?1`
pub fn check_room_availability(
    room_id: i64,
    checkin: &str,
    checkout: &str,
    exclude_booking_id: Option<i64>,
    conn: &Connection,
) -> Result<bool, String> {
    println!("🔍 [validation::check_room_availability] Check gestartet:");
    println!("   room_id: {}", room_id);
    println!("   checkin: {}", checkin);
    println!("   checkout: {}", checkout);
    println!("   exclude_booking_id: {:?}", exclude_booking_id);

    // Validiere zuerst die Daten
    validate_date_range(checkin, checkout)?;

    // Query für überlappende Buchungen
    // Überschneidungslogik: Neue Periode überschneidet sich mit bestehender, wenn:
    // - new_checkin < existing_checkout UND new_checkout > existing_checkin
    let query = match exclude_booking_id {
        Some(_) => {
            "SELECT COUNT(*) FROM bookings
             WHERE room_id = ?1
             AND checkin_date < ?3
             AND checkout_date > ?2
             AND status != 'storniert'
             AND id != ?4"
        }
        None => {
            "SELECT COUNT(*) FROM bookings
             WHERE room_id = ?1
             AND checkin_date < ?3
             AND checkout_date > ?2
             AND status != 'storniert'"
        }
    };

    println!("🔍 [validation] SQL Query: {}", query);

    let count: i64 = if let Some(booking_id) = exclude_booking_id {
        conn.query_row(
            query,
            rusqlite::params![room_id, checkin, checkout, booking_id],
            |row| row.get(0),
        )
    } else {
        conn.query_row(
            query,
            rusqlite::params![room_id, checkin, checkout],
            |row| row.get(0),
        )
    }
    .map_err(|e| format!("Datenbankfehler bei Verfügbarkeitsprüfung: {}", e))?;

    println!("🔍 [validation] Anzahl überlappender Buchungen: {}", count);

    // Raum ist verfügbar, wenn KEINE überlappenden Buchungen existieren
    let is_available = count == 0;
    println!("✅ [validation] Zimmer verfügbar: {}", is_available);

    Ok(is_available)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use rusqlite::Connection;

    // ========================================================================
    // DATE VALIDATION TESTS
    // ========================================================================

    #[test]
    fn test_validate_date_format_valid() {
        // Gültige Daten
        assert!(validate_date_format("2025-10-15").is_ok());
        assert!(validate_date_format("2025-01-01").is_ok());
        assert!(validate_date_format("2025-12-31").is_ok());

        // Prüfe, dass das Datum korrekt geparst wird
        let date = validate_date_format("2025-10-15").unwrap();
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_validate_date_format_invalid() {
        // Ungültige Formate
        assert!(validate_date_format("15.10.2025").is_err());
        assert!(validate_date_format("2025/10/15").is_err());
        assert!(validate_date_format("10-15-2025").is_err());
        assert!(validate_date_format("not-a-date").is_err());
        assert!(validate_date_format("").is_err());

        // Ungültige Daten (31. Februar existiert nicht)
        assert!(validate_date_format("2025-02-31").is_err());
        assert!(validate_date_format("2025-13-01").is_err());

        // Prüfe Fehlermeldung
        let result = validate_date_format("15.10.2025");
        assert_eq!(result.unwrap_err(), "Datum muss im Format YYYY-MM-DD sein");
    }

    #[test]
    fn test_validate_date_range_valid() {
        // Gültige Zeiträume
        assert!(validate_date_range("2025-10-01", "2025-10-05").is_ok());
        assert!(validate_date_range("2025-10-01", "2025-10-02").is_ok()); // 1 Nacht
        assert!(validate_date_range("2025-12-31", "2026-01-05").is_ok()); // Jahreswechsel
    }

    #[test]
    fn test_validate_date_range_invalid() {
        // Check-out vor Check-in
        let result = validate_date_range("2025-10-05", "2025-10-01");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Check-out muss nach Check-in liegen (mindestens 1 Nacht)"
        );

        // Gleiche Daten (0 Nächte)
        let result = validate_date_range("2025-10-01", "2025-10-01");
        assert!(result.is_err());

        // Ungültige Datumsformate
        assert!(validate_date_range("invalid", "2025-10-05").is_err());
        assert!(validate_date_range("2025-10-01", "invalid").is_err());
    }

    // ========================================================================
    // EMAIL VALIDATION TESTS
    // ========================================================================

    #[test]
    fn test_validate_email_valid() {
        // Gültige E-Mails
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name@example.com").is_ok());
        assert!(validate_email("user+tag@example.co.uk").is_ok());
        assert!(validate_email("firstname.lastname@example.de").is_ok());
        assert!(validate_email("test123@test-domain.com").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        // Ungültige E-Mails
        assert!(validate_email("").is_err());
        assert!(validate_email("   ").is_err());
        assert!(validate_email("notanemail").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user @example.com").is_err());
        assert!(validate_email("user@.com").is_err());

        // Prüfe Fehlermeldung
        let result = validate_email("");
        assert_eq!(result.unwrap_err(), "E-Mail-Adresse darf nicht leer sein");

        let result = validate_email("notanemail");
        assert_eq!(result.unwrap_err(), "Ungültige E-Mail-Adresse");
    }

    // ========================================================================
    // ROOM AVAILABILITY TESTS
    // ========================================================================

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Erstelle Test-Tabellen
        conn.execute(
            "CREATE TABLE rooms (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE bookings (
                id INTEGER PRIMARY KEY,
                room_id INTEGER NOT NULL,
                checkin_date TEXT NOT NULL,
                checkout_date TEXT NOT NULL,
                status TEXT NOT NULL,
                FOREIGN KEY (room_id) REFERENCES rooms (id)
            )",
            [],
        )
        .unwrap();

        // Füge Test-Raum hinzu
        conn.execute("INSERT INTO rooms (id, name) VALUES (1, 'Test Room')", [])
            .unwrap();

        conn
    }

    #[test]
    fn test_check_room_availability_empty_db() {
        let conn = setup_test_db();

        // Raum sollte verfügbar sein, wenn keine Buchungen existieren
        let result = check_room_availability(
            1,
            "2025-10-01",
            "2025-10-05",
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_check_room_availability_no_overlap() {
        let conn = setup_test_db();

        // Existierende Buchung: 2025-10-01 bis 2025-10-05
        conn.execute(
            "INSERT INTO bookings (room_id, checkin_date, checkout_date, status)
             VALUES (1, '2025-10-01', '2025-10-05', 'bestaetigt')",
            [],
        )
        .unwrap();

        // Neue Buchung NACH der existierenden (keine Überschneidung)
        let result = check_room_availability(
            1,
            "2025-10-05", // Check-in am Check-out-Tag der existierenden Buchung
            "2025-10-10",
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Neue Buchung VOR der existierenden (keine Überschneidung)
        let result = check_room_availability(
            1,
            "2025-09-25",
            "2025-10-01", // Check-out am Check-in-Tag der existierenden Buchung
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_check_room_availability_with_overlap() {
        let conn = setup_test_db();

        // Existierende Buchung: 2025-10-01 bis 2025-10-05
        conn.execute(
            "INSERT INTO bookings (room_id, checkin_date, checkout_date, status)
             VALUES (1, '2025-10-01', '2025-10-05', 'bestaetigt')",
            [],
        )
        .unwrap();

        // Überschneidung: Check-in während bestehender Buchung
        let result = check_room_availability(
            1,
            "2025-10-03",
            "2025-10-07",
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        // Überschneidung: Check-out während bestehender Buchung
        let result = check_room_availability(
            1,
            "2025-09-28",
            "2025-10-03",
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        // Überschneidung: Komplett umschließend
        let result = check_room_availability(
            1,
            "2025-09-28",
            "2025-10-10",
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        // Überschneidung: Komplett eingeschlossen
        let result = check_room_availability(
            1,
            "2025-10-02",
            "2025-10-04",
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_check_room_availability_exclude_booking() {
        let conn = setup_test_db();

        // Existierende Buchung mit ID 1
        conn.execute(
            "INSERT INTO bookings (id, room_id, checkin_date, checkout_date, status)
             VALUES (1, 1, '2025-10-01', '2025-10-05', 'bestaetigt')",
            [],
        )
        .unwrap();

        // Bei Update der gleichen Buchung sollte sie ignoriert werden
        let result = check_room_availability(
            1,
            "2025-10-01",
            "2025-10-05",
            Some(1), // Exclude booking ID 1
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Aber andere überschneidende Buchungen werden trotzdem erkannt
        conn.execute(
            "INSERT INTO bookings (id, room_id, checkin_date, checkout_date, status)
             VALUES (2, 1, '2025-10-03', '2025-10-07', 'bestaetigt')",
            [],
        )
        .unwrap();

        let result = check_room_availability(
            1,
            "2025-10-01",
            "2025-10-05",
            Some(1),
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_check_room_availability_ignores_cancelled() {
        let conn = setup_test_db();

        // Stornierte Buchung sollte ignoriert werden
        conn.execute(
            "INSERT INTO bookings (room_id, checkin_date, checkout_date, status)
             VALUES (1, '2025-10-01', '2025-10-05', 'storniert')",
            [],
        )
        .unwrap();

        let result = check_room_availability(
            1,
            "2025-10-01",
            "2025-10-05",
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_check_room_availability_invalid_dates() {
        let conn = setup_test_db();

        // Ungültiges Datumsformat
        let result = check_room_availability(
            1,
            "invalid",
            "2025-10-05",
            None,
            &conn,
        );
        assert!(result.is_err());

        // Check-out vor Check-in
        let result = check_room_availability(
            1,
            "2025-10-05",
            "2025-10-01",
            None,
            &conn,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_check_room_availability_different_rooms() {
        let conn = setup_test_db();

        // Füge zweiten Raum hinzu
        conn.execute("INSERT INTO rooms (id, name) VALUES (2, 'Test Room 2')", [])
            .unwrap();

        // Buchung für Raum 1
        conn.execute(
            "INSERT INTO bookings (room_id, checkin_date, checkout_date, status)
             VALUES (1, '2025-10-01', '2025-10-05', 'bestaetigt')",
            [],
        )
        .unwrap();

        // Raum 2 sollte trotzdem verfügbar sein
        let result = check_room_availability(
            2,
            "2025-10-01",
            "2025-10-05",
            None,
            &conn,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}