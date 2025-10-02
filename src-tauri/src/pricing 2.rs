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
    // 1. Lade Zimmerinformationen aus der Datenbank
    let room_price: f64 = conn
        .query_row(
            "SELECT price_member, price_non_member FROM rooms WHERE id = ?1",
            rusqlite::params![room_id],
            |row| {
                let price_member: f64 = row.get(0)?;
                let price_non_member: f64 = row.get(1)?;
                Ok(if is_member { price_member } else { price_non_member })
            },
        )
        .map_err(|e| format!("Fehler beim Laden des Zimmers: {}", e))?;

    // 2. Berechne Anzahl der Nächte
    let anzahl_naechte = calculate_nights(checkin, checkout)?;

    // 3. Berechne Grundpreis
    let grundpreis = calculate_base_price(anzahl_naechte, room_price);

    // 4. Berechne Services-Summe
    let services_preis = calculate_services_total(services);

    // 5. Berechne Zwischensumme (vor Rabatten)
    let subtotal = grundpreis + services_preis;

    // 6. Berechne alle Rabatte
    let mut rabatt_preis = 0.0;
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

    // 7. Berechne Gesamtpreis
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

        // Erstelle Tabelle
        conn.execute(
            "CREATE TABLE rooms (
                id INTEGER PRIMARY KEY,
                name TEXT,
                price_member REAL,
                price_non_member REAL
            )",
            [],
        )
        .unwrap();

        // Füge Testdaten ein
        conn.execute(
            "INSERT INTO rooms (id, name, price_member, price_non_member) VALUES (1, 'Zimmer 101', 45.0, 65.0)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO rooms (id, name, price_member, price_non_member) VALUES (2, 'Suite 201', 85.0, 110.0)",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_calculate_booking_total_member_no_extras() {
        // Test: Mitglied, 3 Nächte, keine Services/Rabatte
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
        assert_eq!(grundpreis, 135.0); // 3 × 45.0
        assert_eq!(services_preis, 0.0);
        assert_eq!(rabatt_preis, 0.0);
        assert_eq!(gesamtpreis, 135.0);
    }

    #[test]
    fn test_calculate_booking_total_non_member_no_extras() {
        // Test: Nicht-Mitglied, 3 Nächte, keine Services/Rabatte
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
        let (grundpreis, _, _, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 195.0); // 3 × 65.0
        assert_eq!(gesamtpreis, 195.0);
    }

    #[test]
    fn test_calculate_booking_total_with_services() {
        // Test: Mit Services
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
        assert_eq!(services_preis, 25.0);
        assert_eq!(rabatt_preis, 0.0);
        assert_eq!(gesamtpreis, 160.0); // 135 + 25
    }

    #[test]
    fn test_calculate_booking_total_with_percent_discount() {
        // Test: Mit prozentualem Rabatt
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
        assert_eq!(services_preis, 0.0);
        assert_eq!(rabatt_preis, 13.5); // 10% von 135
        assert_eq!(gesamtpreis, 121.5); // 135 - 13.5
    }

    #[test]
    fn test_calculate_booking_total_with_fixed_discount() {
        // Test: Mit fixem Rabatt
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
        let (_, _, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(rabatt_preis, 20.0);
        assert_eq!(gesamtpreis, 115.0); // 135 - 20
    }

    #[test]
    fn test_calculate_booking_total_multiple_discounts() {
        // Test: Mehrere Rabatte
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
        let (grundpreis, _, rabatt_preis, gesamtpreis, _) = result.unwrap();

        assert_eq!(grundpreis, 135.0);
        assert_eq!(rabatt_preis, 23.5); // 13.5 (10%) + 10.0
        assert_eq!(gesamtpreis, 111.5); // 135 - 23.5
    }

    #[test]
    fn test_calculate_booking_total_complete_scenario() {
        // Test: Vollständiges Szenario mit allem
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
        assert_eq!(grundpreis, 425.0); // 5 × 85.0
        assert_eq!(services_preis, 25.0); // 15 + 10
        // Subtotal = 450.0 (425 + 25)
        assert_eq!(rabatt_preis, 45.0); // 10% von 450
        assert_eq!(gesamtpreis, 405.0); // 450 - 45
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
        let (_, _, rabatt_preis, gesamtpreis, _) = result.unwrap();

        // Rabatt wird auf Subtotal begrenzt
        assert_eq!(rabatt_preis, 135.0); // gekappt auf Subtotal
        assert_eq!(gesamtpreis, 0.0);
    }

    #[test]
    fn test_calculate_booking_total_single_night() {
        // Test: Einzelne Nacht
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
        let (grundpreis, _, _, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 1);
        assert_eq!(grundpreis, 45.0);
        assert_eq!(gesamtpreis, 45.0);
    }

    #[test]
    fn test_calculate_booking_total_long_stay() {
        // Test: Langer Aufenthalt (30 Nächte)
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
        let (grundpreis, _, _, gesamtpreis, anzahl_naechte) = result.unwrap();

        assert_eq!(anzahl_naechte, 30);
        assert_eq!(grundpreis, 1350.0); // 30 × 45.0
        assert_eq!(gesamtpreis, 1350.0);
    }
}