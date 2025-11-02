// ============================================================================
// PRICING LOGIC MODULE - Single Source of Truth Architecture
// ============================================================================
//
// Dieses Modul enthält ALLE Funktionen zur Preisberechnung für Buchungen.
// Alle Funktionen folgen den Business Rules aus dem MIGRATION_GUIDE.
//
// ARCHITEKTUR-PRINZIP: Single Source of Truth
// - EINE zentrale Funktion berechnet ALLE Preise
// - Frontend ruft NUR diese Funktion auf
// - Frontend berechnet NIEMALS selbst
//
// Berechnungen:
// - Anzahl Nächte = Differenz zwischen Check-in und Check-out
// - Grundpreis = Nächte × Zimmerpreis (Mitglied/Nicht-Mitglied)
// - Services-Summe = Summe aller zusätzlichen Services (inkl. %)
// - Rabatte anwenden (wenn vorhanden)
// - Gesamtpreis = Grundpreis + Services - Rabatte
// ============================================================================

use chrono::{NaiveDate, Datelike};
use rusqlite::Connection;
use serde::{Serialize, Deserialize};

// ============================================================================
// NEUE ARCHITEKTUR: Vollständige Preis-Strukturen
// ============================================================================

/// Input: Service-Daten vom Frontend
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceInput {
    pub name: String,
    pub price_type: String,        // "fixed" | "percent"
    pub original_value: f64,       // Template-Wert (z.B. 19.0 für 19%)
    pub applies_to: String,        // "overnight_price" | "total_price"
    #[serde(default)]
    pub template_id: Option<i64>,
}

/// Output: Berechneter Service mit allen Details
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCalculation {
    pub name: String,
    pub price_type: String,
    pub original_value: f64,
    pub applies_to: String,
    pub calculated_price: f64,     // FINALER berechneter Preis
    pub base_amount: Option<f64>,  // Basis für % Berechnung
}

/// Input: Rabatt-Daten vom Frontend
#[derive(Debug, Clone, Deserialize)]
pub struct DiscountInput {
    pub name: String,
    pub discount_type: String,     // "fixed" | "percent"
    pub value: f64,
    #[serde(default)]
    pub template_id: Option<i64>,
}

/// Output: Berechneter Rabatt mit allen Details
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscountCalculation {
    pub name: String,
    pub discount_type: String,
    pub original_value: f64,
    pub calculated_amount: f64,    // FINALER berechneter Betrag
    pub base_amount: Option<f64>,  // Basis für % Berechnung
}

/// Output: VOLLSTÄNDIGER Preis-Breakdown (Single Source of Truth)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullPriceBreakdown {
    // Basis-Daten
    pub base_price: f64,           // Grundpreis (Zimmer × Nächte)
    pub nights: i32,
    pub price_per_night: f64,
    pub is_hauptsaison: bool,

    // Services (MIT ALLEN Details!)
    pub services: Vec<ServiceCalculation>,
    pub services_total: f64,

    // Rabatte (MIT ALLEN Details!)
    pub discounts: Vec<DiscountCalculation>,
    pub discounts_total: f64,

    // Summen
    pub subtotal: f64,             // Grundpreis + Services
    pub total: f64,                // Nach Rabatten

    // Meta
    pub calculation_timestamp: String,
}

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

/// Prüft ob ein Datum in der Hauptsaison liegt (basierend auf pricing_settings).
///
/// Lädt die Hauptsaison-Zeiträume dynamisch aus pricing_settings Tabelle.
/// Wenn hauptsaison_aktiv=false, gibt immer false zurück (=> Nebensaison).
///
/// # Arguments
/// * `date` - Datum im Format YYYY-MM-DD
/// * `conn` - Datenbankverbindung
///
/// # Returns
/// * `Ok(true)` - Hauptsaison
/// * `Ok(false)` - Nebensaison oder Hauptsaison deaktiviert
/// * `Err(String)` - Ungültiges Datum oder DB-Fehler
pub fn is_hauptsaison_with_settings(date: &str, conn: &Connection) -> Result<bool, String> {
    // Lade Pricing Settings
    let (hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende): (bool, String, String) = conn
        .query_row(
            "SELECT hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende FROM pricing_settings WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Fehler beim Laden der Preiseinstellungen: {}", e))?;

    // Wenn Hauptsaison deaktiviert, immer Nebensaison
    if !hauptsaison_aktiv {
        return Ok(false);
    }

    let date_parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| format!("Ungültiges Datum: {}. Erwartetes Format: YYYY-MM-DD", date))?;

    // Parse Saison-Start/Ende (Format: MM-DD)
    let (start_month, start_day) = parse_mmdd(&hauptsaison_start)?;
    let (end_month, end_day) = parse_mmdd(&hauptsaison_ende)?;

    // Prüfe ob Datum in der Hauptsaison liegt
    let is_in_season = is_date_in_season(date_parsed, start_month, start_day, end_month, end_day);

    Ok(is_in_season)
}

/// Hilfsfunktion: Parst MM-DD Format zu (Monat, Tag)
fn parse_mmdd(mmdd: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = mmdd.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("Ungültiges MM-DD Format: {}", mmdd));
    }

    let month = parts[0].parse::<u32>()
        .map_err(|_| format!("Ungültiger Monat: {}", parts[0]))?;
    let day = parts[1].parse::<u32>()
        .map_err(|_| format!("Ungültiger Tag: {}", parts[1]))?;

    Ok((month, day))
}

/// Hilfsfunktion: Prüft ob ein Datum in einer Saison (Monat/Tag-Range) liegt
fn is_date_in_season(date: NaiveDate, start_month: u32, start_day: u32, end_month: u32, end_day: u32) -> bool {
    let year = date.year();

    // Erstelle Start- und End-Datum für das Jahr des Datums
    let season_start = NaiveDate::from_ymd_opt(year, start_month, start_day);
    let season_end = NaiveDate::from_ymd_opt(year, end_month, end_day);

    if let (Some(start), Some(end)) = (season_start, season_end) {
        if start <= end {
            // Normale Saison (z.B. Juni-August)
            date >= start && date <= end
        } else {
            // Über Jahreswechsel (z.B. Dezember-Februar)
            // Prüfe ob in aktuellem Jahr (nach Start) oder nächstem Jahr (vor Ende)
            let next_year_end = NaiveDate::from_ymd_opt(year + 1, end_month, end_day);
            date >= start || (next_year_end.is_some() && date <= next_year_end.unwrap())
        }
    } else {
        false
    }
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
/// # Neue Preisberechnung (Preisliste 2025 - Dynamisch konfigurierbar):
/// - Saisonpreise: Hauptsaison vs. Nebensaison (konfigurierbar in pricing_settings)
/// - DPolG-Rabatt: Prozentsatz und Aktivierung konfigurierbar (aus pricing_settings)
/// - Rabatt-Basis: Wählbar zwischen "zimmerpreis" (nur Zimmer) oder "gesamtpreis" (Zimmer + Services)
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

                // Saisonerkennung basierend auf pricing_settings
                let is_hs = is_hauptsaison_with_settings(checkin, conn).unwrap_or(false);
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

    // 6. DPolG-Rabatt automatisch für Mitglieder laden (aus pricing_settings)
    let mut rabatt_preis = 0.0;
    if is_member {
        let (rabatt_aktiv, rabatt_prozent, rabatt_basis): (bool, f64, String) = conn
            .query_row(
                "SELECT mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis FROM pricing_settings WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap_or((true, 15.0, "zimmerpreis".to_string())); // Fallback

        // Nur Rabatt anwenden wenn aktiv
        if rabatt_aktiv {
            let rabatt_basis_betrag = match rabatt_basis.as_str() {
                "zimmerpreis" => grundpreis,  // Nur Zimmerpreis (ohne Services)
                "gesamtpreis" => subtotal,    // Zimmerpreis + Services
                _ => grundpreis,              // Fallback: nur Zimmerpreis
            };

            rabatt_preis += apply_discount_percentage(rabatt_basis_betrag, rabatt_prozent);
        }
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
    // Tests für is_hauptsaison_with_settings
    // ========================================================================

    #[test]
    fn test_is_hauptsaison_nebensaison() {
        // Test: Januar = Nebensaison
        let conn = setup_test_db();
        let result = is_hauptsaison_with_settings("2025-01-15", &conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_is_hauptsaison_hauptsaison_summer() {
        // Test: Juli = Hauptsaison (01.06-31.08 basierend auf setup_test_db)
        let conn = setup_test_db();
        let result = is_hauptsaison_with_settings("2025-07-20", &conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_is_hauptsaison_hauptsaison_winter() {
        // Test: Januar 2026 = Nebensaison (Standard-Config: 06-01 bis 08-31)
        let conn = setup_test_db();
        let result = is_hauptsaison_with_settings("2026-01-10", &conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_is_hauptsaison_start_date_summer() {
        // Test: Erster Tag der Hauptsaison (01.06.2025)
        let conn = setup_test_db();
        let result = is_hauptsaison_with_settings("2025-06-01", &conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_is_hauptsaison_end_date_summer() {
        // Test: Letzter Tag der Hauptsaison (31.08.2025 basierend auf setup_test_db)
        let conn = setup_test_db();
        let result = is_hauptsaison_with_settings("2025-08-31", &conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_is_hauptsaison_after_summer() {
        // Test: Tag nach Hauptsaison (01.09.2025) = Nebensaison
        let conn = setup_test_db();
        let result = is_hauptsaison_with_settings("2025-09-01", &conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_is_hauptsaison_before_summer() {
        // Test: Tag vor Hauptsaison (31.05.2025) = Nebensaison
        let conn = setup_test_db();
        let result = is_hauptsaison_with_settings("2025-05-31", &conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_is_hauptsaison_invalid_date() {
        // Test: Ungültiges Datum
        let conn = setup_test_db();
        let result = is_hauptsaison_with_settings("2025-13-01", &conn);
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

        conn.execute(
            "CREATE TABLE pricing_settings (
                id INTEGER PRIMARY KEY,
                hauptsaison_aktiv INTEGER NOT NULL DEFAULT 1,
                hauptsaison_start TEXT NOT NULL DEFAULT '06-01',
                hauptsaison_ende TEXT NOT NULL DEFAULT '08-31',
                mitglieder_rabatt_aktiv INTEGER NOT NULL DEFAULT 1,
                mitglieder_rabatt_prozent REAL NOT NULL DEFAULT 15.0,
                rabatt_basis TEXT NOT NULL DEFAULT 'zimmerpreis'
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

        // Pricing Settings mit Standard-Konfiguration
        conn.execute(
            "INSERT INTO pricing_settings (id, hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende, mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis)
             VALUES (1, 1, '06-01', '08-31', 1, 15.0, 'zimmerpreis')",
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
        assert_eq!(rabatt_preis, 20.25); // 15% DPolG-Rabatt auf grundpreis (135.0)
        assert_eq!(gesamtpreis, 134.75); // 155 - 20.25
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
        assert_eq!(rabatt_preis, 20.25); // 15% DPolG-Rabatt auf grundpreis (135.0)
        assert_eq!(gesamtpreis, 159.75); // 180 - 20.25
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
        // DPolG-Rabatt: 15% von grundpreis (135) = 20.25
        // Frühbucher: 10% von subtotal (155) = 15.5
        assert_eq!(rabatt_preis, 35.75); // 20.25 + 15.5
        assert_eq!(gesamtpreis, 119.25); // 155 - 35.75
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
        // DPolG-Rabatt: 15% von grundpreis (135) = 20.25
        // Gutschein: 20.0
        assert_eq!(rabatt_preis, 40.25); // 20.25 + 20.0
        assert_eq!(gesamtpreis, 114.75); // 155 - 40.25
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
        // DPolG: 15% von grundpreis (135) = 20.25
        // Frühbucher: 10% von subtotal (155) = 15.5
        // Gutschein: 10.0
        assert_eq!(rabatt_preis, 45.75); // 20.25 + 15.5 + 10.0
        assert_eq!(gesamtpreis, 109.25); // 155 - 45.75
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
        // DPolG: 15% von grundpreis (425) = 63.75
        // Frühbucher: 10% von subtotal (490) = 49.0
        assert_eq!(rabatt_preis, 112.75); // 63.75 + 49.0
        assert_eq!(gesamtpreis, 377.25); // 490 - 112.75
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
        // DPolG: 15% von grundpreis (135) = 20.25
        // Großer Rabatt: 155.0 (gekappt)
        assert_eq!(rabatt_preis, 175.25); // 20.25 + 155.0
        assert_eq!(gesamtpreis, 0.0); // 155 - 175.25 (min 0)
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
        assert_eq!(rabatt_preis, 6.75); // 15% DPolG-Rabatt auf grundpreis (45)
        assert_eq!(gesamtpreis, 58.25); // 65 - 6.75
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
        assert_eq!(rabatt_preis, 202.5); // 15% DPolG-Rabatt auf grundpreis (1350)
        assert_eq!(gesamtpreis, 1167.5); // 1370 - 202.5
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
        assert_eq!(rabatt_preis, 20.25); // 15% DPolG-Rabatt auf grundpreis (135)
        assert_eq!(gesamtpreis, 134.75); // 155 - 20.25
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
        assert_eq!(rabatt_preis, 24.75); // 15% DPolG-Rabatt auf grundpreis (165)
        assert_eq!(gesamtpreis, 160.25); // 185 - 24.75
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
        assert_eq!(grundpreis, 425.0); // 5 × 85.0 (Nebensaison - Jan nicht in 06-01..08-31)
        assert_eq!(services_preis, 40.0); // Endreinigung Suite
        // Subtotal = 465.0
        assert_eq!(rabatt_preis, 63.75); // 15% von grundpreis (425)
        assert_eq!(gesamtpreis, 401.25); // 465 - 63.75
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
        assert_eq!(rabatt_preis, 20.25); // 15% von grundpreis (135)
        assert_eq!(gesamtpreis, 159.75); // 180 - 20.25
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
        // Test: Letzter Tag der Hauptsaison (31.08.2025 basierend auf setup_test_db)
        let conn = setup_test_db();
        let result = calculate_booking_total(
            1,
            "2025-08-29",
            "2025-09-01", // 3 Nächte (29.08, 30.08, 31.08)
            false,
            &[],
            &[],
            &conn,
        );

        assert!(result.is_ok());
        let (grundpreis, _, _, _, _) = result.unwrap();

        assert_eq!(grundpreis, 165.0); // 3 × 55.0 (Hauptsaison - alle 3 Tage in Hauptsaison)
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

// ============================================================================
// SINGLE SOURCE OF TRUTH: Zentrale Preisberechnung
// ============================================================================

/// HAUPTFUNKTION: Berechnet ALLE Preise mit vollständigem Breakdown
///
/// Diese Funktion ist die EINZIGE Stelle für Preisberechnungen.
/// Frontend ruft NUR diese Funktion auf - berechnet NIEMALS selbst!
///
/// # Arguments
/// * `room_id` - Zimmer-ID
/// * `checkin` - Check-in Datum (YYYY-MM-DD)
/// * `checkout` - Check-out Datum (YYYY-MM-DD)
/// * `is_member` - Ist Gast DPolG-Mitglied?
/// * `services` - Liste von Services mit ALLEN Daten (price_type, original_value, etc.)
/// * `discounts` - Liste von Rabatten
/// * `conn` - Datenbankverbindung
///
/// # Returns
/// * `Ok(FullPriceBreakdown)` - Vollständiger Breakdown mit ALLEN Details
/// * `Err(String)` - Fehlermeldung
pub fn calculate_full_booking_price(
    room_id: i64,
    checkin: &str,
    checkout: &str,
    is_member: bool,
    services: &[ServiceInput],
    discounts: &[DiscountInput],
    conn: &Connection,
) -> Result<FullPriceBreakdown, String> {

    // SCHRITT 1: Grundpreis berechnen (Zimmer × Nächte, mit Saison-Logik)
    let nights = calculate_nights(checkin, checkout)?;
    let is_hauptsaison = is_hauptsaison_with_settings(checkin, conn)?;

    // Hole Zimmer-Preise
    let (preis_nebensaison, preis_hauptsaison, endreinigung_preis): (f64, f64, f64) = conn
        .query_row(
            "SELECT nebensaison_preis, hauptsaison_preis, endreinigung FROM rooms WHERE id = ?1",
            rusqlite::params![room_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Zimmer nicht gefunden: {}", e))?;

    // Wähle Preis basierend auf Saison
    let price_per_night = if is_hauptsaison {
        preis_hauptsaison
    } else {
        preis_nebensaison
    };

    let mut base_price = price_per_night * nights as f64;

    // DPolG-Mitglieder-Rabatt auf Grundpreis (wenn aktiv)
    if is_member {
        let (rabatt_aktiv, rabatt_prozent, rabatt_basis): (bool, f64, String) = conn
            .query_row(
                "SELECT mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis FROM pricing_settings WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap_or((true, 15.0, "zimmerpreis".to_string()));

        if rabatt_aktiv && rabatt_basis == "zimmerpreis" {
            let rabatt_betrag = base_price * (rabatt_prozent / 100.0);
            base_price -= rabatt_betrag;
        }
    }

    // SCHRITT 2: Services berechnen (inkl. prozentuale Services!)
    let mut calculated_services = Vec::new();
    let mut services_running_total = 0.0;

    // Endreinigung als erster Service (immer Festbetrag)
    if endreinigung_preis > 0.0 {
        calculated_services.push(ServiceCalculation {
            name: "Endreinigung".to_string(),
            price_type: "fixed".to_string(),
            original_value: endreinigung_preis,
            applies_to: "overnight_price".to_string(),
            calculated_price: endreinigung_preis,
            base_amount: Some(base_price),
        });
        services_running_total += endreinigung_preis;
    }

    // Weitere Services berechnen
    for service in services {
        let calculated_price = match service.price_type.as_str() {
            "percent" => {
                // Basis für Prozent-Berechnung ermitteln
                let base = match service.applies_to.as_str() {
                    "overnight_price" => base_price,
                    "total_price" => base_price + services_running_total,
                    _ => base_price,
                };
                base * (service.original_value / 100.0)
            },
            "fixed" => service.original_value,
            _ => service.original_value,
        };

        services_running_total += calculated_price;

        calculated_services.push(ServiceCalculation {
            name: service.name.clone(),
            price_type: service.price_type.clone(),
            original_value: service.original_value,
            applies_to: service.applies_to.clone(),
            calculated_price,
            base_amount: Some(base_price),
        });
    }

    // SCHRITT 3: Rabatte berechnen
    let subtotal = base_price + services_running_total;
    let mut calculated_discounts = Vec::new();
    let mut discounts_total = 0.0;

    // DPolG-Mitglieder-Rabatt (wenn auf Gesamtpreis)
    if is_member {
        let (rabatt_aktiv, rabatt_prozent, rabatt_basis): (bool, f64, String) = conn
            .query_row(
                "SELECT mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis FROM pricing_settings WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap_or((true, 15.0, "zimmerpreis".to_string()));

        if rabatt_aktiv && rabatt_basis == "gesamtpreis" {
            let rabatt_betrag = subtotal * (rabatt_prozent / 100.0);
            discounts_total += rabatt_betrag;

            calculated_discounts.push(DiscountCalculation {
                name: "DPolG-Mitgliederrabatt".to_string(),
                discount_type: "percent".to_string(),
                original_value: rabatt_prozent,
                calculated_amount: rabatt_betrag,
                base_amount: Some(subtotal),
            });
        }
    }

    // Weitere Rabatte
    for discount in discounts {
        let calculated_amount = match discount.discount_type.as_str() {
            "percent" => subtotal * (discount.value / 100.0),
            "fixed" => discount.value,
            _ => 0.0,
        };

        discounts_total += calculated_amount;

        calculated_discounts.push(DiscountCalculation {
            name: discount.name.clone(),
            discount_type: discount.discount_type.clone(),
            original_value: discount.value,
            calculated_amount,
            base_amount: Some(subtotal),
        });
    }

    // SCHRITT 4: Finale Summen
    let total = subtotal - discounts_total;
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Rückgabe: VOLLSTÄNDIGER Breakdown
    Ok(FullPriceBreakdown {
        base_price,
        nights,
        price_per_night,
        is_hauptsaison,
        services: calculated_services,
        services_total: services_running_total,
        discounts: calculated_discounts,
        discounts_total,
        subtotal,
        total,
        calculation_timestamp: timestamp,
    })
}