// ============================================================================
// PRICING LOGIC MODULE - Phase 2
// ============================================================================
//
// Dieses Modul enthält alle Funktionen zur Preisberechnung für Buchungen.
// Alle Funktionen folgen den Business Rules aus dem MIGRATION_GUIDE.
//
// Berechnungen:
// - Anzahl Nächte = Differenz zwischen Check-in und Check-out
// - Grundpreis = Nächte × Zimmerpreis (Mitglied/Nicht-Mitglied)
// - Services-Summe = Summe aller zusätzlichen Services
// - Rabatte anwenden (wenn vorhanden)
// - Gesamtpreis = Grundpreis + Services - Rabatte
// ============================================================================

use chrono::NaiveDate;
use rusqlite::Connection;

/// Berechnet die Anzahl der Nächte zwischen zwei Daten.
///
/// # Arguments
/// * `checkin` - Check-in Datum im Format YYYY-MM-DD
/// * `checkout` - Check-out Datum im Format YYYY-MM-DD
///
/// # Returns
/// * `Ok(i32)` - Anzahl der Nächte
/// * `Err(String)` - Fehlermeldung bei ungültigen Daten
///
/// # Errors
/// * Wenn das Datumsformat ungültig ist
/// * Wenn Check-out vor oder gleich Check-in liegt
pub fn calculate_nights(checkin: &str, checkout: &str) -> Result<i32, String> {
    // Parse Check-in Datum
    let checkin_date = NaiveDate::parse_from_str(checkin, "%Y-%m-%d")
        .map_err(|_| format!("Ungültiges Check-in Datum: {}. Erwartetes Format: YYYY-MM-DD", checkin))?;

    // Parse Check-out Datum
    let checkout_date = NaiveDate::parse_from_str(checkout, "%Y-%m-%d")
        .map_err(|_| format!("Ungültiges Check-out Datum: {}. Erwartetes Format: YYYY-MM-DD", checkout))?;

    // Prüfe, ob Check-out nach Check-in liegt
    if checkout_date <= checkin_date {
        return Err("Check-out Datum muss nach Check-in Datum liegen".to_string());
    }

    // Berechne Differenz in Tagen
    let duration = checkout_date.signed_duration_since(checkin_date);
    let nights = duration.num_days() as i32;

    Ok(nights)
}

/// Berechnet den Grundpreis basierend auf Anzahl Nächte und Preis pro Nacht.
///
/// # Arguments
/// * `nights` - Anzahl der Nächte
/// * `price_per_night` - Preis pro Nacht
///
/// # Returns
/// * `f64` - Grundpreis (Nächte × Preis pro Nacht)
///
/// # Note
/// Gibt 0.0 zurück wenn nights <= 0
pub fn calculate_base_price(nights: i32, price_per_night: f64) -> f64 {
    if nights <= 0 {
        return 0.0;
    }
    nights as f64 * price_per_night
}

/// Berechnet den Gesamtpreis einer Buchung.
///
/// # Arguments
/// * `grundpreis` - Grundpreis (Nächte × Zimmerpreis)
/// * `services_preis` - Summe aller Services
/// * `rabatt_preis` - Summe aller Rabatte
///
/// # Returns
/// * `f64` - Gesamtpreis (mindestens 0.0)
///
/// # Formula
/// Gesamtpreis = Grundpreis + Services - Rabatte (minimum 0.0)
pub fn calculate_total_price(grundpreis: f64, services_preis: f64, rabatt_preis: f64) -> f64 {
    let total = grundpreis + services_preis - rabatt_preis;
    // Stelle sicher, dass der Preis nicht negativ wird
    total.max(0.0)
}

/// Wendet einen prozentualen Rabatt an.
///
/// # Arguments
/// * `price` - Ursprünglicher Preis
/// * `percentage` - Rabatt in Prozent (0-100)
///
/// # Returns
/// * `f64` - Rabattbetrag (nicht der finale Preis!)
///
/// # Note
/// Prozentsatz wird bei 100% gekappt
pub fn apply_discount_percentage(price: f64, percentage: f64) -> f64 {
    // Kappe Prozentsatz bei 100%
    let capped_percentage = percentage.min(100.0).max(0.0);
    price * (capped_percentage / 100.0)
}

/// Wendet einen fixen Rabattbetrag an.
///
/// # Arguments
/// * `price` - Ursprünglicher Preis
/// * `discount_amount` - Fixer Rabattbetrag
///
/// # Returns
/// * `f64` - Rabattbetrag (maximal der ursprüngliche Preis)
///
/// # Note
/// Rabatt übersteigt niemals den ursprünglichen Preis
pub fn apply_discount_fixed(price: f64, discount_amount: f64) -> f64 {
    // Rabatt darf nicht den Preis übersteigen
    let capped_discount = discount_amount.min(price).max(0.0);
    capped_discount
}

/// Berechnet die Gesamtsumme aller Services.
///
/// # Arguments
/// * `services` - Array von Service-Tupeln (Name, Preis)
///
/// # Returns
/// * `f64` - Gesamtsumme aller Services
pub fn calculate_services_total(services: &[(String, f64)]) -> f64 {
    services.iter().map(|(_, price)| price).sum()
}

/// Prüft ob ein Datum in der Hauptsaison liegt.
///
/// Hauptsaison: 01.06-15.09.2025 und 22.12-28.02.2026
///
/// # Arguments
/// * `date` - Datum im Format YYYY-MM-DD
///
/// # Returns
/// * `Ok(true)` - Hauptsaison
/// * `Ok(false)` - Nebensaison
/// * `Err(String)` - Ungültiges Datum
pub fn is_hauptsaison(date: &str) -> Result<bool, String> {
    let date_parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| format!("Ungültiges Datum: {}. Erwartetes Format: YYYY-MM-DD", date))?;

    // Hauptsaison 1: 01.06.2025 - 15.09.2025
    let hs1_start = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
    let hs1_end = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap();

    // Hauptsaison 2: 22.12.2025 - 28.02.2026
    let hs2_start = NaiveDate::from_ymd_opt(2025, 12, 22).unwrap();
    let hs2_end = NaiveDate::from_ymd_opt(2026, 2, 28).unwrap();

    // Prüfe ob Datum in einer der Hauptsaison-Perioden liegt
    let is_in_hs = (date_parsed >= hs1_start && date_parsed <= hs1_end)
        || (date_parsed >= hs2_start && date_parsed <= hs2_end);

    Ok(is_in_hs)
}

/// Berechnet alle Preiskomponenten einer Buchung basierend auf Datenbank und Parametern.
///
/// Dies ist die Hauptfunktion für die Preiskalkulation einer Buchung.
///
/// # Arguments
/// * `room_id` - ID des Zimmers
/// * `checkin` - Check-in Datum (YYYY-MM-DD)
/// * `checkout` - Check-out Datum (YYYY-MM-DD)
/// * `is_member` - Ist der Gast DPolG Stiftung Mitglied?
/// * `services` - Array von Services (Name, Preis)
/// * `discounts` - Array von Rabatten (Name, Typ, Wert)
/// * `conn` - Datenbankverbindung
///
/// # Returns
/// * `Ok((grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte))`
/// * `Err(String)` - Fehlermeldung
///
/// # Neue Preisberechnung (Preisliste 2025):
/// - Saisonpreise: Hauptsaison (01.06-15.09 + 22.12-28.02) vs. Nebensaison
/// - DPolG-Rabatt: 15% automatisch für Mitglieder (aus payment_settings)
/// - Endreinigung: Automatisch pro Zimmer hinzugefügt
///
/// # Discount Types
/// * "percent" - Prozentualer Rabatt (0-100)
/// * "fixed" - Fixer Rabattbetrag
pub fn calculate_booking_total(
    room_id: i64,
    checkin: &str,
    checkout: &str,
    is_member: bool,
    services: &[(String, f64)],
    discounts: &[(String, String, f64)], // (Name, Typ, Wert)
    conn: &Connection,
) -> Result<(f64, f64, f64, f64, i32), String> {
    // 1. Lade Zimmerinformationen und Endreinigung aus der Datenbank
    let (room_price, endreinigung): (f64, f64) = conn
        .query_row(
            "SELECT nebensaison_preis, hauptsaison_preis, endreinigung FROM rooms WHERE id = ?1",
            rusqlite::params![room_id],
            |row| {
                let nebensaison: f64 = row.get(0)?;
                let hauptsaison: f64 = row.get(1)?;
                let endreinigung: f64 = row.get(2)?;

                // Saisonerkennung basierend auf Check-in Datum
                let is_hs = is_hauptsaison(checkin).unwrap_or(false);
                let price = if is_hs { hauptsaison } else { nebensaison };

                Ok((price, endreinigung))
            },
        )
        .map_err(|e| format!("Fehler beim Laden des Zimmers: {}", e))?;

    // 2. Berechne Anzahl der Nächte
    let anzahl_naechte = calculate_nights(checkin, checkout)?;

    // 3. Berechne Grundpreis (Nächte × Saisonpreis)
    let grundpreis = calculate_base_price(anzahl_naechte, room_price);

    // 4. Berechne Services-Summe (inkl. Endreinigung)
    let mut services_preis = calculate_services_total(services);
    services_preis += endreinigung; // Endreinigung automatisch hinzufügen

    // 5. Berechne Zwischensumme (vor Rabatten)
    let subtotal = grundpreis + services_preis;

    // 6. DPolG-Rabatt automatisch für Mitglieder laden
    let mut rabatt_preis = 0.0;
    if is_member {
        let dpolg_rabatt: f64 = conn
            .query_row(
                "SELECT dpolg_rabatt FROM payment_settings LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(15.0); // Fallback: 15% wenn nicht in DB

        rabatt_preis += apply_discount_percentage(subtotal, dpolg_rabatt);
    }

    // 7. Berechne zusätzliche Rabatte
    for (_, discount_type, discount_value) in discounts {
        let discount_amount = match discount_type.as_str() {
            "percent" => apply_discount_percentage(subtotal, *discount_value),
            "fixed" => apply_discount_fixed(subtotal, *discount_value),
            _ => {
                return Err(format!(
                    "Ungültiger Rabatt-Typ: {}. Erlaubt: 'percent' oder 'fixed'",
                    discount_type
                ))
            }
        };
        rabatt_preis += discount_amount;
    }

    // 8. Berechne Gesamtpreis
    let gesamtpreis = calculate_total_price(grundpreis, services_preis, rabatt_preis);

    Ok((grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte))
}

// ============================================================================
// UNIT TESTS - TDD Approach
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // ========================================================================
    // Tests für calculate_nights
    // ========================================================================

    #[test]
    fn test_calculate_nights_valid() {
        // Test: 3 Nächte
        let result = calculate_nights("2025-10-01", "2025-10-04");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_calculate_nights_single_night() {
        // Test: 1 Nacht
        let result = calculate_nights("2025-10-01", "2025-10-02");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_calculate_nights_long_stay() {
        // Test: 30 Nächte
        let result = calculate_nights("2025-10-01", "2025-10-31");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30);
    }

    #[test]
    fn test_calculate_nights_same_day_error() {
        // Test: Gleiches Datum -> Fehler
        let result = calculate_nights("2025-10-01", "2025-10-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Check-out Datum muss nach Check-in"));
    }

    #[test]
    fn test_calculate_nights_checkout_before_checkin() {
        // Test: Check-out vor Check-in -> Fehler
        let result = calculate_nights("2025-10-05", "2025-10-01");
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_nights_invalid_checkin_format() {
        // Test: Ungültiges Check-in Format
        let result = calculate_nights("01-10-2025", "2025-10-05");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ungültiges Check-in Datum"));
    }

    #[test]
    fn test_calculate_nights_invalid_checkout_format() {
        // Test: Ungültiges Check-out Format
        let result = calculate_nights("2025-10-01", "10/05/2025");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ungültiges Check-out Datum"));
    }

    #[test]
    fn test_calculate_nights_invalid_date() {
        // Test: Ungültiges Datum (31. Februar)
        let result = calculate_nights("2025-02-31", "2025-03-01");
        assert!(result.is_err());
    }

    // ========================================================================
    // Tests für calculate_base_price
    // ========================================================================

    #[test]
    fn test_calculate_base_price_normal() {
        // Test: 3 Nächte × 45.0 EUR = 135.0 EUR
        let result = calculate_base_price(3, 45.0);
        assert_eq!(result, 135.0);
    }

    #[test]
    fn test_calculate_base_price_single_night() {
        // Test: 1 Nacht × 65.0 EUR = 65.0 EUR
        let result = calculate_base_price(1, 65.0);
        assert_eq!(result, 65.0);
    }

    #[test]
    fn test_calculate_base_price_zero_nights() {
        // Test: 0 Nächte -> 0.0 EUR
        let result = calculate_base_price(0, 45.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_calculate_base_price_negative_nights() {
        // Test: Negative Nächte -> 0.0 EUR
        let result = calculate_base_price(-1, 45.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_calculate_base_price_decimal_price() {
        // Test: Dezimalpreis
        let result = calculate_base_price(5, 47.50);
        assert_eq!(result, 237.50);
    }

    #[test]
    fn test_calculate_base_price_zero_price() {
        // Test: Preis 0.0
        let result = calculate_base_price(3, 0.0);
        assert_eq!(result, 0.0);
    }

    // ========================================================================
    // Tests für calculate_total_price
    // ========================================================================

    #[test]
    fn test_calculate_total_price_normal() {
        // Test: 135.0 + 20.0 - 10.0 = 145.0
        let result = calculate_total_price(135.0, 20.0, 10.0);
        assert_eq!(result, 145.0);
    }

    #[test]
    fn test_calculate_total_price_no_services() {
        // Test: Ohne Services
        let result = calculate_total_price(135.0, 0.0, 10.0);
        assert_eq!(result, 125.0);
    }

    #[test]
    fn test_calculate_total_price_no_discount() {
        // Test: Ohne Rabatt
        let result = calculate_total_price(135.0, 20.0, 0.0);
        assert_eq!(result, 155.0);
    }

    #[test]
    fn test_calculate_total_price_negative_result() {
        // Test: Rabatt übersteigt Summe -> 0.0 (nicht negativ)
        let result = calculate_total_price(100.0, 20.0, 150.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_calculate_total_price_exact_zero() {
        // Test: Exakt 0.0
        let result = calculate_total_price(100.0, 0.0, 100.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_calculate_total_price_all_zero() {
        // Test: Alle Werte 0.0
        let result = calculate_total_price(0.0, 0.0, 0.0);
        assert_eq!(result, 0.0);
    }

    // ========================================================================
    // Tests für apply_discount_percentage
    // ========================================================================

    #[test]
    fn test_apply_discount_percentage_10_percent() {
        // Test: 10% von 100.0 = 10.0
        let result = apply_discount_percentage(100.0, 10.0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_apply_discount_percentage_50_percent() {
        // Test: 50% von 200.0 = 100.0
        let result = apply_discount_percentage(200.0, 50.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_apply_discount_percentage_100_percent() {
        // Test: 100% von 150.0 = 150.0
        let result = apply_discount_percentage(150.0, 100.0);
        assert_eq!(result, 150.0);
    }

    #[test]
    fn test_apply_discount_percentage_over_100_percent() {
        // Test: Über 100% wird bei 100% gekappt
        let result = apply_discount_percentage(100.0, 150.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_apply_discount_percentage_zero_percent() {
        // Test: 0% Rabatt
        let result = apply_discount_percentage(100.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_apply_discount_percentage_negative_percent() {
        // Test: Negativer Prozentsatz -> 0.0
        let result = apply_discount_percentage(100.0, -10.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_apply_discount_percentage_decimal() {
        // Test: Dezimaler Prozentsatz (12.5%)
        let result = apply_discount_percentage(200.0, 12.5);
        assert_eq!(result, 25.0);
    }

    // ========================================================================
    // Tests für apply_discount_fixed
    // ========================================================================

    #[test]
    fn test_apply_discount_fixed_normal() {
        // Test: Fixer Rabatt 20.0 von 100.0
        let result = apply_discount_fixed(100.0, 20.0);
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_apply_discount_fixed_exceeds_price() {
        // Test: Rabatt übersteigt Preis -> wird gekappt
        let result = apply_discount_fixed(100.0, 150.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_apply_discount_fixed_exact_price() {
        // Test: Rabatt gleich Preis
        let result = apply_discount_fixed(100.0, 100.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_apply_discount_fixed_zero() {
        // Test: Kein Rabatt
        let result = apply_discount_fixed(100.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_apply_discount_fixed_negative() {
        // Test: Negativer Rabatt -> 0.0
        let result = apply_discount_fixed(100.0, -10.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_apply_discount_fixed_decimal() {
        // Test: Dezimaler Rabatt
        let result = apply_discount_fixed(100.0, 12.50);
        assert_eq!(result, 12.50);
    }

    // ========================================================================
    // Tests für is_hauptsaison
    // ========================================================================

    #[test]
    fn test_is_hauptsaison_nebensaison() {
        // Test: Januar = Nebensaison
        let result = is_hauptsaison("2025-01-15");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_is_hauptsaison_hauptsaison_summer() {
        // Test: Juli = Hauptsaison (01.06-15.09)
        let result = is_hauptsaison("2025-07-20");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_is_hauptsaison_hauptsaison_winter() {
        // Test: Januar 2026 = Hauptsaison (22.12-28.02)
        let result = is_hauptsaison("2026-01-10");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_is_hauptsaison_start_date_summer() {
        // Test: Erster Tag der Hauptsaison (01.06.2025)
        let result = is_hauptsaison("2025-06-01");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_is_hauptsaison_end_date_summer() {
        // Test: Letzter Tag der Hauptsaison (15.09.2025)
        let result = is_hauptsaison("2025-09-15");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_is_hauptsaison_after_summer() {
        // Test: Tag nach Hauptsaison (16.09.2025) = Nebensaison
        let result = is_hauptsaison("2025-09-16");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_is_hauptsaison_before_summer() {
        // Test: Tag vor Hauptsaison (31.05.2025) = Nebensaison
        let result = is_hauptsaison("2025-05-31");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_is_hauptsaison_invalid_date() {
        // Test: Ungültiges Datum
        let result = is_hauptsaison("2025-13-01");
        assert!(result.is_err());
    }

    // ========================================================================
    // Tests für calculate_services_total
    // ========================================================================

    #[test]
    fn test_calculate_services_total_multiple() {
        // Test: Mehrere Services
        let services = vec![
            ("Frühstück".to_string(), 15.0),
            ("Parkplatz".to_string(), 10.0),
            ("Minibar".to_string(), 8.50),
        ];
        let result = calculate_services_total(&services);
        assert_eq!(result, 33.50);
    }

    #[test]
    fn test_calculate_services_total_single() {
        // Test: Ein Service
        let services = vec![("Frühstück".to_string(), 15.0)];
        let result = calculate_services_total(&services);
        assert_eq!(result, 15.0);
    }

    #[test]
    fn test_calculate_services_total_empty() {
        // Test: Keine Services
        let services: Vec<(String, f64)> = vec![];
        let result = calculate_services_total(&services);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_calculate_services_total_zero_prices() {
        // Test: Services mit Preis 0.0
        let services = vec![
            ("Kostenloser Service".to_string(), 0.0),
            ("Frühstück".to_string(), 15.0),
        ];
        let result = calculate_services_total(&services);
        assert_eq!(result, 15.0);
    }

    // ========================================================================
    // Tests für calculate_booking_total (Integration Tests)
    // ========================================================================

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // Erstelle Tabellen
        conn.execute(
            "CREATE TABLE rooms (
                id INTEGER PRIMARY KEY,
                name TEXT,
                price_member REAL,
                price_non_member REAL,
                nebensaison_preis REAL,
                hauptsaison_preis REAL,
                endreinigung REAL
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE payment_settings (
                id INTEGER PRIMARY KEY,
                dpolg_rabatt REAL DEFAULT 15.0
            )",
            [],
        )
        .unwrap();

        // Füge Testdaten ein (Zimmer mit Nebensaison/Hauptsaison Preisen)
        conn.execute(
            "INSERT INTO rooms (id, name, price_member, price_non_member, nebensaison_preis, hauptsaison_preis, endreinigung)
             VALUES (1, 'Zimmer 101', 45.0, 65.0, 45.0, 55.0, 20.0)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO rooms (id, name, price_member, price_non_member, nebensaison_preis, hauptsaison_preis, endreinigung)
             VALUES (2, 'Suite 201', 85.0, 110.0, 85.0, 100.0, 40.0)",
            [],
        )
        .unwrap();

        // Payment Settings mit DPolG-Rabatt 15%
        conn.execute(
            "INSERT INTO payment_settings (id, dpolg_rabatt) VALUES (1, 15.0)",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_calculate_booking_total_member_no_extras() {
        // Test: Mitglied, 3 Nächte, keine extra Services (nur Endreinigung)
        // Oktober = Nebensaison
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-04",
            true,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 3);
        assert_eq!(grundpreis, 135.0); // 3 × 45.0 (Nebensaison)
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 155.0
        assert_eq!(rabatt_preis, 23.25); // 15% DPolG-Rabatt
        assert_eq!(gesamtpreis, 131.75); // 155 - 23.25
    }

    #[test]
    fn test_calculate_booking_total_non_member_no_extras() {
        // Test: Nicht-Mitglied, 3 Nächte, keine extra Services
        // Oktober = Nebensaison, kein DPolG-Rabatt
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-04",
            false,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0); // 3 × 45.0 (Nebensaison)
        assert_eq!(services_preis, 20.0); // Endreinigung
        assert_eq!(rabatt_preis, 0.0); // Kein DPolG-Rabatt
        assert_eq!(gesamtpreis, 155.0); // 135 + 20
    }

    #[test]
    fn test_calculate_booking_total_with_services() {
        // Test: Mit Services + Endreinigung + DPolG-Rabatt
        let conn = setup_test_db();
        let services = vec![
            ("Frühstück".to_string(), 15.0),
            ("Parkplatz".to_string(), 10.0),
        ];
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-04",
            true,
            &services,
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0);
        assert_eq!(services_preis, 45.0); // Endreinigung (20) + Services (25)
        // Subtotal = 180.0
        assert_eq!(rabatt_preis, 27.0); // 15% DPolG-Rabatt
        assert_eq!(gesamtpreis, 153.0); // 180 - 27
    }

    #[test]
    fn test_calculate_booking_total_with_percent_discount() {
        // Test: Mit prozentualem Rabatt + DPolG-Rabatt + Endreinigung
        let conn = setup_test_db();
        let discounts = vec![
            ("Frühbucher".to_string(), "percent".to_string(), 10.0),
        ];
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-04",
            true,
            &[],
            &discounts,
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0);
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 155.0
        // DPolG-Rabatt: 15% von 155 = 23.25
        // Frühbucher: 10% von 155 = 15.5
        assert_eq!(rabatt_preis, 38.75); // 23.25 + 15.5
        assert_eq!(gesamtpreis, 116.25); // 155 - 38.75
    }

    #[test]
    fn test_calculate_booking_total_with_fixed_discount() {
        // Test: Mit fixem Rabatt + DPolG-Rabatt + Endreinigung
        let conn = setup_test_db();
        let discounts = vec![
            ("Gutschein".to_string(), "fixed".to_string(), 20.0),
        ];
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-04",
            true,
            &[],
            &discounts,
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0);
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 155.0
        // DPolG-Rabatt: 15% von 155 = 23.25
        // Gutschein: 20.0
        assert_eq!(rabatt_preis, 43.25); // 23.25 + 20.0
        assert_eq!(gesamtpreis, 111.75); // 155 - 43.25
    }

    #[test]
    fn test_calculate_booking_total_multiple_discounts() {
        // Test: Mehrere Rabatte + DPolG-Rabatt + Endreinigung
        let conn = setup_test_db();
        let discounts = vec![
            ("Frühbucher".to_string(), "percent".to_string(), 10.0),
            ("Gutschein".to_string(), "fixed".to_string(), 10.0),
        ];
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-04",
            true,
            &[],
            &discounts,
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0);
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 155.0
        // DPolG: 15% von 155 = 23.25
        // Frühbucher: 10% von 155 = 15.5
        // Gutschein: 10.0
        assert_eq!(rabatt_preis, 48.75); // 23.25 + 15.5 + 10.0
        assert_eq!(gesamtpreis, 106.25); // 155 - 48.75
    }

    #[test]
    fn test_calculate_booking_total_complete_scenario() {
        // Test: Vollständiges Szenario mit allem (Nebensaison)
        let conn = setup_test_db();
        let services = vec![
            ("Frühstück".to_string(), 15.0),
            ("Parkplatz".to_string(), 10.0),
        ];
        let discounts = vec![
            ("Frühbucher".to_string(), "percent".to_string(), 10.0),
        ];
        let result = calculate_booking_total(
            2, // Suite mit höherem Preis
            "2025-10-01",
            "2025-10-06",
            true,
            &services,
            &discounts,
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 5);
        assert_eq!(grundpreis, 425.0); // 5 × 85.0 (Nebensaison)
        assert_eq!(services_preis, 65.0); // Endreinigung (40) + Services (25)
        // Subtotal = 490.0 (425 + 65)
        // DPolG: 15% von 490 = 73.5
        // Frühbucher: 10% von 490 = 49.0
        assert_eq!(rabatt_preis, 122.5); // 73.5 + 49.0
        assert_eq!(gesamtpreis, 367.5); // 490 - 122.5
    }

    #[test]
    fn test_calculate_booking_total_invalid_room() {
        // Test: Ungültige Zimmer-ID
        let conn = setup_test_db();
        let result = calculate_booking_total(
            999,
            "2025-10-01",
            "2025-10-04",
            true,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Fehler beim Laden des Zimmers"));
    }

    #[test]
    fn test_calculate_booking_total_invalid_dates() {
        // Test: Ungültige Daten
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-10-05",
            "2025-10-01",
            true,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_booking_total_invalid_discount_type() {
        // Test: Ungültiger Rabatt-Typ
        let conn = setup_test_db();
        let discounts = vec![
            ("Invalid".to_string(), "invalid_type".to_string(), 10.0),
        ];
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-04",
            true,
            &[],
            &discounts,
            &conn,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ungültiger Rabatt-Typ"));
    }

    #[test]
    fn test_calculate_booking_total_excessive_discount() {
        // Test: Rabatt übersteigt Gesamtpreis -> 0.0
        let conn = setup_test_db();
        let discounts = vec![
            ("Großer Rabatt".to_string(), "fixed".to_string(), 500.0),
        ];
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-04",
            true,
            &[],
            &discounts,
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0);
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 155.0
        // DPolG: 15% von 155 = 23.25
        // Großer Rabatt: 155.0 (gekappt)
        assert_eq!(rabatt_preis, 178.25); // 23.25 + 155.0
        assert_eq!(gesamtpreis, 0.0); // 155 - 178.25 (min 0)
    }

    #[test]
    fn test_calculate_booking_total_single_night() {
        // Test: Einzelne Nacht (Nebensaison, Mitglied)
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-02",
            true,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 1);
        assert_eq!(grundpreis, 45.0); // 1 × 45.0 (Nebensaison)
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 65.0
        assert_eq!(rabatt_preis, 9.75); // 15% DPolG-Rabatt
        assert_eq!(gesamtpreis, 55.25); // 65 - 9.75
    }

    #[test]
    fn test_calculate_booking_total_long_stay() {
        // Test: Langer Aufenthalt (30 Nächte, Nebensaison, Mitglied)
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-10-01",
            "2025-10-31",
            true,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 30);
        assert_eq!(grundpreis, 1350.0); // 30 × 45.0 (Nebensaison)
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 1370.0
        assert_eq!(rabatt_preis, 205.5); // 15% DPolG-Rabatt
        assert_eq!(gesamtpreis, 1164.5); // 1370 - 205.5
    }

    // ========================================================================
    // Tests für Preisliste 2025 (Saisonpreise + DPolG-Rabatt + Endreinigung)
    // ========================================================================

    #[test]
    fn test_calculate_booking_total_nebensaison_member_with_endreinigung() {
        // Test: Nebensaison, Mitglied, mit Endreinigung
        // Januar 2025 = Nebensaison
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-01-10",
            "2025-01-13", // 3 Nächte
            true,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 3);
        assert_eq!(grundpreis, 135.0); // 3 × 45.0 (Nebensaison)
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 155.0 (135 + 20)
        assert_eq!(rabatt_preis, 23.25); // 15% DPolG-Rabatt von 155.0
        assert_eq!(gesamtpreis, 131.75); // 155 - 23.25
    }

    #[test]
    fn test_calculate_booking_total_hauptsaison_member_with_endreinigung() {
        // Test: Hauptsaison, Mitglied, mit Endreinigung
        // Juli 2025 = Hauptsaison
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-07-10",
            "2025-07-13", // 3 Nächte
            true,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 3);
        assert_eq!(grundpreis, 165.0); // 3 × 55.0 (Hauptsaison)
        assert_eq!(services_preis, 20.0); // Endreinigung
        // Subtotal = 185.0 (165 + 20)
        assert_eq!(rabatt_preis, 27.75); // 15% DPolG-Rabatt von 185.0
        assert_eq!(gesamtpreis, 157.25); // 185 - 27.75
    }

    #[test]
    fn test_calculate_booking_total_nebensaison_non_member_with_endreinigung() {
        // Test: Nebensaison, Nicht-Mitglied (kein DPolG-Rabatt)
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-01-10",
            "2025-01-13", // 3 Nächte
            false,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0); // 3 × 45.0 (Nebensaison)
        assert_eq!(services_preis, 20.0); // Endreinigung
        assert_eq!(rabatt_preis, 0.0); // Kein DPolG-Rabatt
        assert_eq!(gesamtpreis, 155.0); // 135 + 20
    }

    #[test]
    fn test_calculate_booking_total_hauptsaison_winter() {
        // Test: Hauptsaison Winter (22.12 - 28.02)
        // Januar 2026 = Hauptsaison
        let conn = setup_test_db();
        let result = calculate_booking_total(
            2, // Suite
            "2026-01-15",
            "2026-01-20", // 5 Nächte
            true,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 5);
        assert_eq!(grundpreis, 500.0); // 5 × 100.0 (Hauptsaison)
        assert_eq!(services_preis, 40.0); // Endreinigung Suite
        // Subtotal = 540.0
        assert_eq!(rabatt_preis, 81.0); // 15% von 540
        assert_eq!(gesamtpreis, 459.0); // 540 - 81
    }

    #[test]
    fn test_calculate_booking_total_with_additional_services_and_endreinigung() {
        // Test: Services zusätzlich zur Endreinigung
        let conn = setup_test_db();
        let services = vec![
            ("Frühstück".to_string(), 15.0),
            ("Parkplatz".to_string(), 10.0),
        ];
        let result = calculate_booking_total(
            1,
            "2025-01-10",
            "2025-01-13", // 3 Nächte
            true,
            &services,
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, services_preis, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0); // 3 × 45.0 (Nebensaison)
        assert_eq!(services_preis, 45.0); // Endreinigung (20) + Services (25)
        // Subtotal = 180.0
        assert_eq!(rabatt_preis, 27.0); // 15% von 180
        assert_eq!(gesamtpreis, 153.0); // 180 - 27
    }

    #[test]
    fn test_calculate_booking_total_season_boundary_start() {
        // Test: Erster Tag der Hauptsaison (01.06.2025)
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-06-01",
            "2025-06-04", // 3 Nächte
            false,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, _, _, _, _) = result.unwrap();

        assert_eq!(grundpreis, 165.0); // 3 × 55.0 (Hauptsaison)
    }

    #[test]
    fn test_calculate_booking_total_season_boundary_end() {
        // Test: Letzter Tag der Hauptsaison (15.09.2025)
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-09-15",
            "2025-09-18", // 3 Nächte
            false,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, _, _, _, _) = result.unwrap();

        assert_eq!(grundpreis, 165.0); // 3 × 55.0 (Hauptsaison)
    }

    #[test]
    fn test_calculate_booking_total_after_hauptsaison() {
        // Test: Tag nach Hauptsaison (16.09.2025) = Nebensaison
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-09-16",
            "2025-09-19", // 3 Nächte
            false,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, _, _, _, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0); // 3 × 45.0 (Nebensaison)
    }
}