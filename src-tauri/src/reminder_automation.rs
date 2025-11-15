// ============================================================================
// AUTOMATIC REMINDER CREATION SYSTEM (2025 Best Practices)
// ============================================================================
// Hybrid Approach: Event-Driven + Scheduled
// Based on industry standards from Cloudbeds, Mews, Buildium
// ============================================================================

use rusqlite::{Connection, Result};
use crate::database::{get_db_path, BookingWithDetails, ReminderSettings};
use crate::reminders::{create_reminder, get_reminder_settings};
use std::sync::Mutex;
use once_cell::sync::Lazy;

// ============================================================================
// SETTINGS CACHE (Performance Optimization 2025)
// ============================================================================
// Cache für Reminder Settings (wird bei jedem Auto-Complete geladen)
// Invalidierung: Bei Settings-Änderung über `invalidate_settings_cache()`
static SETTINGS_CACHE: Lazy<Mutex<Option<ReminderSettings>>> = Lazy::new(|| Mutex::new(None));

/// Lädt Settings mit Caching (Performance-Optimierung)
fn get_cached_reminder_settings() -> std::result::Result<ReminderSettings, String> {
    // Prüfe Cache
    let cache = SETTINGS_CACHE.lock().unwrap();
    if let Some(settings) = cache.as_ref() {
        return Ok(settings.clone());
    }
    drop(cache);

    // Cache Miss → Lade von DB
    let settings = get_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Reminder-Settings: {}", e))?;

    // Speichere im Cache
    *SETTINGS_CACHE.lock().unwrap() = Some(settings.clone());

    Ok(settings)
}

/// Invalidiert Settings-Cache (nach Settings-Änderung aufrufen)
pub fn invalidate_settings_cache() {
    *SETTINGS_CACHE.lock().unwrap() = None;
    println!("🔄 [CACHE] Settings-Cache invalidiert");
}

// ============================================================================
// EVENT-DRIVEN TRIGGERS (Sofort bei Aktionen)
// ============================================================================

/// Erstellt automatisch Reminder bei neuer Buchung
/// Trigger: create_booking() aufgerufen
pub fn on_booking_created(booking: &BookingWithDetails) -> Result<(), String> {
    println!("🔔 [AUTO-REMINDER] Prüfe Reminder für neue Buchung {}", booking.id);

    let settings = get_cached_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Reminder-Settings: {}", e))?;

    // DEBUG: Zeige alle Settings
    println!("   [DEBUG] Settings geladen:");
    println!("   - auto_reminder_invoice: {}", settings.auto_reminder_invoice);
    println!("   - auto_reminder_incomplete_data: {}", settings.auto_reminder_incomplete_data);
    println!("   - auto_reminder_payment: {}", settings.auto_reminder_payment);
    println!("   - auto_reminder_checkin: {}", settings.auto_reminder_checkin);

    let mut created_count = 0;

    // 1. Buchungsbestätigung versenden (falls aktiviert)
    println!("   [DEBUG] Prüfe Confirmation-Reminder...");
    if settings.auto_reminder_invoice {  // Verwenden wir gleiche Setting wie Invoice
        println!("   [DEBUG] Confirmation-Reminder ist aktiviert");
        let exists = reminder_exists(booking.id, "auto_confirmation")?;
        println!("   [DEBUG] Reminder existiert bereits: {}", exists);

        if !exists {
            println!("   [DEBUG] Erstelle Confirmation-Reminder...");
            match create_reminder(
                Some(booking.id),
                "auto_confirmation".to_string(),
                format!("Buchungsbestätigung versenden für {}", booking.reservierungsnummer),
                Some(format!("Bestätigung an {} {} senden ({})",
                    booking.guest.vorname, booking.guest.nachname, booking.guest.email)),
                crate::time_utils::format_today_db(),
                "high".to_string(),
            ) {
                Ok(_) => {
                    created_count += 1;
                    println!("   ✅ Reminder erstellt: Buchungsbestätigung versenden (HIGH)");
                }
                Err(e) => {
                    println!("   ❌ FEHLER beim Erstellen: {}", e);
                    return Err(format!("Fehler beim Erstellen des Confirmation-Reminders: {}", e));
                }
            }
        } else {
            println!("   ℹ️  Confirmation-Reminder existiert bereits");
        }
    } else {
        println!("   [DEBUG] Confirmation-Reminder ist DEAKTIVIERT");
    }

    // 2. Rechnung versenden (falls aktiviert)
    println!("   [DEBUG] Prüfe Invoice-Reminder...");
    if settings.auto_reminder_invoice {
        println!("   [DEBUG] Invoice-Reminder ist aktiviert");
        let exists = reminder_exists(booking.id, "auto_invoice")?;
        println!("   [DEBUG] Reminder existiert bereits: {}", exists);

        if !exists {
            println!("   [DEBUG] Erstelle Invoice-Reminder...");
            match create_reminder(
                Some(booking.id),
                "auto_invoice".to_string(),
                format!("Rechnung versenden für {}", booking.reservierungsnummer),
                Some(format!("Erstelle und versende Rechnung an {} {}",
                    booking.guest.vorname, booking.guest.nachname)),
                crate::time_utils::format_today_db(),
                "medium".to_string(),
            ) {
                Ok(_) => {
                    created_count += 1;
                    println!("   ✅ Reminder erstellt: Rechnung versenden");
                }
                Err(e) => {
                    println!("   ❌ FEHLER beim Erstellen: {}", e);
                    return Err(format!("Fehler beim Erstellen des Invoice-Reminders: {}", e));
                }
            }
        } else {
            println!("   ℹ️  Invoice-Reminder existiert bereits");
        }
    } else {
        println!("   [DEBUG] Invoice-Reminder ist DEAKTIVIERT");
    }

    // 3. Unvollständige Gästdaten (falls aktiviert)
    println!("   [DEBUG] Prüfe Incomplete-Data-Reminder...");
    if settings.auto_reminder_incomplete_data {
        println!("   [DEBUG] Incomplete-Data-Reminder ist aktiviert");

        let is_incomplete = is_guest_data_incomplete(booking);
        println!("   [DEBUG] Gästdaten unvollständig: {}", is_incomplete);

        if is_incomplete {
            let exists = reminder_exists(booking.id, "auto_incomplete_data")?;
            println!("   [DEBUG] Reminder existiert bereits: {}", exists);

            if !exists {
                println!("   [DEBUG] Erstelle Incomplete-Data-Reminder...");
                match create_reminder(
                    Some(booking.id),
                    "auto_incomplete_data".to_string(),
                    format!("Gästdaten vervollständigen für {}", booking.reservierungsnummer),
                    Some(format!("Fehlende Daten von {} {} nachfordern (Adresse, Telefon, etc.)",
                        booking.guest.vorname, booking.guest.nachname)),
                    crate::time_utils::format_today_db(),
                    "high".to_string(),
                ) {
                    Ok(_) => {
                        created_count += 1;
                        println!("   ✅ Reminder erstellt: Gästdaten vervollständigen (HOCH)");
                    }
                    Err(e) => {
                        println!("   ❌ FEHLER beim Erstellen: {}", e);
                        return Err(format!("Fehler beim Erstellen des Incomplete-Data-Reminders: {}", e));
                    }
                }
            } else {
                println!("   ℹ️  Incomplete-Data-Reminder existiert bereits");
            }
        } else {
            println!("   ℹ️  Gästdaten sind vollständig - kein Reminder nötig");
        }
    } else {
        println!("   [DEBUG] Incomplete-Data-Reminder ist DEAKTIVIERT");
    }

    // 4. Zahlung ausstehend (falls aktiviert und nicht bezahlt)
    println!("   [DEBUG] Prüfe Payment-Reminder...");
    if settings.auto_reminder_payment {
        println!("   [DEBUG] Payment-Reminder ist aktiviert");
        println!("   [DEBUG] Buchung bezahlt: {}", booking.bezahlt);

        if !booking.bezahlt {
            let exists = reminder_exists(booking.id, "auto_payment")?;
            println!("   [DEBUG] Reminder existiert bereits: {}", exists);

            if !exists {
                println!("   [DEBUG] Erstelle Payment-Reminder...");
                match create_reminder(
                    Some(booking.id),
                    "auto_payment".to_string(),
                    format!("Zahlung überwachen für {}", booking.reservierungsnummer),
                    Some(format!("Offener Betrag: {:.2} € von {} {}",
                        booking.gesamtpreis, booking.guest.vorname, booking.guest.nachname)),
                    crate::time_utils::format_today_db(),
                    "high".to_string(),
                ) {
                    Ok(_) => {
                        created_count += 1;
                        println!("   ✅ Reminder erstellt: Zahlung ausstehend (HOCH)");
                    }
                    Err(e) => {
                        println!("   ❌ FEHLER beim Erstellen: {}", e);
                        return Err(format!("Fehler beim Erstellen des Payment-Reminders: {}", e));
                    }
                }
            } else {
                println!("   ℹ️  Payment-Reminder existiert bereits");
            }
        } else {
            println!("   ℹ️  Buchung bereits bezahlt - kein Payment-Reminder nötig");
        }
    } else {
        println!("   [DEBUG] Payment-Reminder ist DEAKTIVIERT");
    }

    // 5. Check-in vorbereiten (falls aktiviert)
    println!("   [DEBUG] Prüfe Check-in-Reminder...");
    if settings.auto_reminder_checkin {
        println!("   [DEBUG] Check-in-Reminder ist aktiviert");

        let exists = reminder_exists(booking.id, "auto_checkin")?;
        println!("   [DEBUG] Reminder existiert bereits: {}", exists);

        if !exists {
            println!("   [DEBUG] Erstelle Check-in-Reminder...");
            match create_reminder(
                Some(booking.id),
                "auto_checkin".to_string(),
                format!("Check-in vorbereiten für {}", booking.reservierungsnummer),
                Some(format!("Check-in am {} für {} {} - Zimmer {} vorbereiten",
                    booking.checkin_date, booking.guest.vorname, booking.guest.nachname,
                    booking.room.name)),
                crate::time_utils::format_today_db(),
                "medium".to_string(),
            ) {
                Ok(_) => {
                    created_count += 1;
                    println!("   ✅ Reminder erstellt: Check-in vorbereiten (MEDIUM)");
                }
                Err(e) => {
                    println!("   ❌ FEHLER beim Erstellen: {}", e);
                    return Err(format!("Fehler beim Erstellen des Check-in-Reminders: {}", e));
                }
            }
        } else {
            println!("   ℹ️  Check-in-Reminder existiert bereits");
        }
    } else {
        println!("   [DEBUG] Check-in-Reminder ist DEAKTIVIERT");
    }

    if created_count > 0 {
        println!("   🎉 {} automatische Reminder erstellt", created_count);
    } else {
        println!("   ℹ️  Keine Reminder erstellt (bereits vorhanden oder deaktiviert)");
    }

    Ok(())
}

/// Prüft ob Gästdaten unvollständig sind
fn is_guest_data_incomplete(booking: &BookingWithDetails) -> bool {
    // Prüfe kritische Felder
    let missing_address = booking.guest.strasse.is_none() ||
                         booking.guest.plz.is_none() ||
                         booking.guest.ort.is_none();

    let missing_phone = booking.guest.telefon.is_none();

    let missing_email = booking.guest.email.is_empty();

    // DEBUG: Zeige welche Felder fehlen
    println!("   [DEBUG] Gästdaten-Check:");
    println!("      - Strasse: {:?}", booking.guest.strasse);
    println!("      - PLZ: {:?}", booking.guest.plz);
    println!("      - Ort: {:?}", booking.guest.ort);
    println!("      - Telefon: {:?}", booking.guest.telefon);
    println!("      - Email: {}", booking.guest.email);
    println!("      → missing_address: {}", missing_address);
    println!("      → missing_phone: {}", missing_phone);
    println!("      → missing_email: {}", missing_email);

    missing_address || missing_phone || missing_email
}

/// Schließt automatisch Reminder wenn Bedingung erfüllt
/// Trigger: mark_booking_paid() aufgerufen
pub fn on_booking_paid(booking_id: i64) -> Result<(), String> {
    println!("💰 [AUTO-REMINDER] Buchung {} wurde bezahlt - schließe Zahlungs-Reminder", booking_id);

    let settings = get_cached_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Reminder-Settings: {}", e))?;

    if settings.auto_reminder_payment {
        complete_reminders_by_type(booking_id, "auto_payment")?;
        println!("   ✅ Zahlungs-Reminder geschlossen");
    }

    Ok(())
}

/// Aktualisiert Reminder wenn Buchung geändert wird
/// Trigger: update_booking() aufgerufen
pub fn on_booking_updated(booking: &BookingWithDetails) -> Result<(), String> {
    println!("🔄 [AUTO-REMINDER] Buchung {} aktualisiert - prüfe Reminder-Status", booking.id);

    let settings = get_cached_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Reminder-Settings: {}", e))?;

    // Wenn Gästdaten jetzt vollständig sind → Incomplete-Data-Reminder schließen
    if settings.auto_reminder_incomplete_data {
        if !is_guest_data_incomplete(booking) {
            complete_reminders_by_type(booking.id, "auto_incomplete_data")?;
            println!("   ✅ Gästdaten-Reminder geschlossen (Daten vollständig)");
        }
    }

    // Wenn Zahlung eingegangen → Payment-Reminder schließen
    if booking.bezahlt && settings.auto_reminder_payment {
        complete_reminders_by_type(booking.id, "auto_payment")?;
        println!("   ✅ Zahlungs-Reminder geschlossen");
    }

    Ok(())
}

/// Schließt Invoice-Reminder wenn Rechnung versendet wurde
/// Trigger: mark_invoice_sent() aufgerufen
/// (2025 CRM Best Practice: Auto-complete tasks on status change)
pub fn on_invoice_sent(booking_id: i64) -> Result<(), String> {
    println!("📧 [AUTO-REMINDER] Rechnung für Buchung {} versendet - schließe Invoice-Reminder", booking_id);

    let settings = get_cached_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Reminder-Settings: {}", e))?;

    if settings.auto_reminder_invoice {
        complete_reminders_by_type(booking_id, "auto_invoice")?;
        println!("   ✅ Invoice-Reminder geschlossen");
    }

    Ok(())
}

/// Schließt automatisch Confirmation-Reminder wenn Bestätigung versendet
/// Trigger: send_confirmation_email() aufgerufen
/// (2025 CRM Best Practice: Auto-complete tasks on status change)
pub fn on_confirmation_sent(booking_id: i64) -> Result<(), String> {
    println!("📧 [AUTO-REMINDER] Bestätigung für Buchung {} versendet - schließe Confirmation-Reminder", booking_id);

    let settings = get_cached_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Reminder-Settings: {}", e))?;

    if settings.auto_reminder_confirmation {
        complete_reminders_by_type(booking_id, "auto_confirmation")?;
        println!("   ✅ Confirmation-Reminder geschlossen");
    }

    Ok(())
}

/// Prüft alle Buchungen eines Gastes wenn Gast-Daten aktualisiert wurden
/// Trigger: update_guest() aufgerufen
pub fn on_guest_updated(
    guest_id: i64,
    strasse: Option<String>,
    plz: Option<String>,
    ort: Option<String>,
    telefon: Option<String>,
) -> Result<(), String> {
    println!("👤 [AUTO-REMINDER] Gast {} aktualisiert - prüfe Incomplete-Data-Reminder", guest_id);

    // Prüfe ob Daten jetzt vollständig sind
    let missing_address = strasse.is_none() || plz.is_none() || ort.is_none();
    let missing_phone = telefon.is_none();

    if missing_address || missing_phone {
        println!("   ⚠️  Gästdaten noch unvollständig - keine Aktion");
        return Ok(());
    }

    // Daten sind vollständig → Hole alle Buchungen dieses Gastes
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id FROM bookings WHERE guest_id = ?1")
        .map_err(|e| format!("SQL Fehler: {}", e))?;

    let booking_ids: Vec<i64> = stmt
        .query_map([guest_id], |row| row.get(0))
        .map_err(|e| format!("Fehler beim Laden der Buchungen: {}", e))?
        .collect::<Result<Vec<i64>, _>>()
        .map_err(|e| format!("Fehler beim Sammeln der Buchungs-IDs: {}", e))?;

    let settings = get_cached_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Reminder-Settings: {}", e))?;

    if !settings.auto_reminder_incomplete_data {
        println!("   ℹ️  Incomplete-Data-Reminder deaktiviert");
        return Ok(());
    }

    // Schließe Incomplete-Data-Reminder für alle Buchungen
    for booking_id in booking_ids {
        complete_reminders_by_type(booking_id, "auto_incomplete_data")?;
    }

    println!("   ✅ Gästdaten vollständig - Incomplete-Data-Reminder geschlossen");

    Ok(())
}

/// Löscht alle Reminder einer stornierten Buchung
/// Trigger: Buchung wird auf "storniert" gesetzt
pub fn on_booking_cancelled(booking_id: i64) -> Result<(), String> {
    println!("❌ [AUTO-REMINDER] Buchung {} storniert - lösche alle Reminder", booking_id);

    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    conn.execute(
        "DELETE FROM reminders WHERE booking_id = ?1",
        [booking_id],
    ).map_err(|e| format!("Fehler beim Löschen der Reminder: {}", e))?;

    println!("   ✅ Alle Reminder für Buchung {} gelöscht", booking_id);

    Ok(())
}

// ============================================================================
// SCHEDULED CHECKS (Täglich zeitbasierte Reminder)
// ============================================================================

/// Täglicher Check für zeitbasierte Reminder (z.B. Check-in Vorbereitung)
/// Sollte täglich um 06:00 Uhr laufen
pub fn daily_reminder_check() -> Result<(), String> {
    println!("🕐 [AUTO-REMINDER] Starte täglichen Reminder-Check...");

    let settings = get_cached_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Reminder-Settings: {}", e))?;

    let mut created_count = 0;

    // 1. Check-in Vorbereitung (1 Tag vor Check-in)
    if settings.auto_reminder_checkin {
        created_count += create_checkin_preparation_reminders()?;
    }

    // 2. Zahlungserinnerung (7 Tage vor Check-in, wenn unbezahlt)
    if settings.auto_reminder_payment {
        created_count += create_payment_reminders()?;
    }

    println!("   🎉 {} neue zeitbasierte Reminder erstellt", created_count);

    Ok(())
}

/// Erstellt Check-in Vorbereitungs-Reminder (1 Tag vorher)
fn create_checkin_preparation_reminders() -> Result<usize, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Buchungen mit Check-in MORGEN
    let tomorrow = crate::time_utils::add_days(1).format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT b.id, b.reservierungsnummer, b.zimmer_nummer, g.vorname, g.nachname
         FROM bookings b
         JOIN guests g ON b.guest_id = g.id
         WHERE b.checkin_date = ?1
         AND b.status != 'storniert'"
    ).map_err(|e| format!("SQL Fehler: {}", e))?;

    let bookings: Vec<(i64, String, Option<String>, String, String)> = stmt.query_map([&tomorrow], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    })
    .map_err(|e| format!("Query Fehler: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    let mut count = 0;

    for (booking_id, res_num, zimmer, vorname, nachname) in bookings {
        // Prüfe ob Reminder bereits existiert
        if reminder_exists(booking_id, "auto_checkin")? {
            continue;
        }

        let zimmer_text = zimmer.unwrap_or_else(|| "noch nicht zugewiesen".to_string());

        create_reminder(
            Some(booking_id),
            "auto_checkin".to_string(),
            format!("Check-in Vorbereitung: {} {}", vorname, nachname),
            Some(format!("Zimmer {} vorbereiten, Schlüssel bereitstellen, Ankunft vorbereiten. Buchung: {}",
                zimmer_text, res_num)),
            tomorrow.clone(),
            "medium".to_string(),
        ).map_err(|e| format!("Fehler beim Erstellen: {}", e))?;

        count += 1;
        println!("   ✅ Check-in Vorbereitung für {} (Zimmer {})", res_num, zimmer_text);
    }

    Ok(count)
}

/// Erstellt Zahlungs-Reminder (7 Tage vor Check-in, wenn unbezahlt)
fn create_payment_reminders() -> Result<usize, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Buchungen mit Check-in in 7 TAGEN und unbezahlt
    let in_7_days = crate::time_utils::add_days(7).format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT b.id, b.reservierungsnummer, b.gesamtpreis, g.vorname, g.nachname
         FROM bookings b
         JOIN guests g ON b.guest_id = g.id
         WHERE b.checkin_date = ?1
         AND b.bezahlt = 0
         AND b.status != 'storniert'"
    ).map_err(|e| format!("SQL Fehler: {}", e))?;

    let bookings: Vec<(i64, String, f64, String, String)> = stmt.query_map([&in_7_days], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    })
    .map_err(|e| format!("Query Fehler: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    let mut count = 0;

    for (booking_id, res_num, total, vorname, nachname) in bookings {
        // Prüfe ob Reminder bereits existiert
        if reminder_exists(booking_id, "auto_payment")? {
            continue;
        }

        create_reminder(
            Some(booking_id),
            "auto_payment".to_string(),
            format!("Zahlung anfordern: {} {}", vorname, nachname),
            Some(format!("Buchung {} noch nicht bezahlt ({:.2}€). Check-in in 7 Tagen - Zahlung anfordern!",
                res_num, total)),
            crate::time_utils::format_today_db(),
            "high".to_string(), // Hohe Priorität
        ).map_err(|e| format!("Fehler beim Erstellen: {}", e))?;

        count += 1;
        println!("   ✅ Zahlungserinnerung für {} ({:.2}€)", res_num, total);
    }

    Ok(count)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Prüft ob ein Reminder bereits existiert (Deduplication)
fn reminder_exists(booking_id: i64, reminder_type: &str) -> Result<bool, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM reminders
         WHERE booking_id = ?1
         AND reminder_type = ?2
         AND is_completed = 0",
        rusqlite::params![booking_id, reminder_type],
        |row| row.get(0)
    ).unwrap_or(false);

    Ok(exists)
}

/// Schließt alle Reminder eines bestimmten Typs für eine Buchung (automatisch)
fn complete_reminders_by_type(booking_id: i64, reminder_type: &str) -> Result<(), String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Validierung: Prüfe ob offene Reminder existieren
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM reminders
         WHERE booking_id = ?1
         AND reminder_type = ?2
         AND is_completed = 0",
        rusqlite::params![booking_id, reminder_type],
        |row| row.get(0)
    ).unwrap_or(0);

    if count == 0 {
        // Kein offener Reminder → Nichts zu tun (kein Fehler!)
        return Ok(());
    }

    // Update: Schließe alle offenen Reminder
    let updated = conn.execute(
        "UPDATE reminders
         SET is_completed = 1,
             completed_at = CURRENT_TIMESTAMP,
             completed_by = 'auto',
             updated_at = CURRENT_TIMESTAMP
         WHERE booking_id = ?1
         AND reminder_type = ?2
         AND is_completed = 0",
        rusqlite::params![booking_id, reminder_type],
    ).map_err(|e| format!("Fehler beim Schließen der Reminder: {}", e))?;

    if updated > 0 {
        println!("   ✅ {} Reminder(s) vom Typ '{}' geschlossen", updated, reminder_type);
    }

    Ok(())
}

// ============================================================================
// TESTING & MANUAL TRIGGERS
// ============================================================================

/// Manueller Trigger für täglichen Check (für Testing)
#[tauri::command]
pub fn trigger_daily_reminder_check() -> Result<String, String> {
    println!("🔔 Manueller Daily-Reminder-Check getriggert...");

    daily_reminder_check()?;

    Ok("Daily Reminder Check abgeschlossen".to_string())
}
