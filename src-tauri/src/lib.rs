// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// PostgreSQL version (new, modern architecture)
pub mod lib_pg;
pub mod config;
pub mod database_pg;

// SQLite version (legacy, will be phased out)
mod database;
mod validation;
mod pricing;
mod email;
mod pdf_generator;
mod pdf_generator_html;
mod time_utils;
mod email_scheduler;
mod backup;
mod transaction_log;
mod supabase;
mod reminders;
mod reminder_automation;
mod invoice_html;
mod payment_recipients;
mod cleaning_timeline_pdf;

use database::{init_database, get_rooms, get_bookings_with_details};
use rusqlite::Connection;

#[tauri::command]
fn get_all_rooms() -> Result<Vec<database::Room>, String> {
    println!("get_all_rooms called");
    match get_rooms() {
        Ok(rooms) => {
            println!("Successfully got {} rooms", rooms.len());
            Ok(rooms)
        }
        Err(e) => {
            eprintln!("Error getting rooms: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_all_bookings() -> Result<Vec<database::BookingWithDetails>, String> {
    println!("get_all_bookings called");
    match get_bookings_with_details() {
        Ok(bookings) => {
            println!("Successfully got {} bookings", bookings.len());
            Ok(bookings)
        }
        Err(e) => {
            eprintln!("Error getting bookings: {}", e);
            Err(e.to_string())
        }
    }
}

// ============================================================================
// GUEST MANAGEMENT COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
fn create_guest_command(
    vorname: String,
    nachname: String,
    email: String,
    telefon: Option<String>,
    dpolg_mitglied: bool,
    strasse: Option<String>,
    plz: Option<String>,
    ort: Option<String>,
    mitgliedsnummer: Option<String>,
    notizen: Option<String>,
    beruf: Option<String>,
    bundesland: Option<String>,
    dienststelle: Option<String>,
    // NEW: 21 additional fields from CSV import
    anrede: Option<String>,
    geschlecht: Option<String>,
    land: Option<String>,
    telefon_geschaeftlich: Option<String>,
    telefon_privat: Option<String>,
    telefon_mobil: Option<String>,
    fax: Option<String>,
    geburtsdatum: Option<String>,
    geburtsort: Option<String>,
    sprache: Option<String>,
    nationalitaet: Option<String>,
    identifikationsnummer: Option<String>,
    debitorenkonto: Option<String>,
    kennzeichen: Option<String>,
    rechnungs_email: Option<String>,
    marketing_einwilligung: Option<bool>,
    leitweg_id: Option<String>,
    kostenstelle: Option<String>,
    tags: Option<String>,
) -> Result<database::Guest, String> {
    database::create_guest(
        vorname,
        nachname,
        email,
        telefon,
        dpolg_mitglied,
        strasse,
        plz,
        ort,
        mitgliedsnummer,
        notizen,
        beruf,
        bundesland,
        dienststelle,
        anrede,
        geschlecht,
        land,
        telefon_geschaeftlich,
        telefon_privat,
        telefon_mobil,
        fax,
        geburtsdatum,
        geburtsort,
        sprache,
        nationalitaet,
        identifikationsnummer,
        debitorenkonto,
        kennzeichen,
        rechnungs_email,
        marketing_einwilligung,
        leitweg_id,
        kostenstelle,
        tags,
    )
    .map_err(|e| format!("Fehler beim Erstellen des Gastes: {}", e))
}

#[tauri::command]
fn update_guest_command(
    id: i64,
    vorname: String,
    nachname: String,
    email: String,
    telefon: Option<String>,
    dpolg_mitglied: bool,
    strasse: Option<String>,
    plz: Option<String>,
    ort: Option<String>,
    mitgliedsnummer: Option<String>,
    notizen: Option<String>,
    beruf: Option<String>,
    bundesland: Option<String>,
    dienststelle: Option<String>,
    // NEW: 21 additional fields from CSV import
    anrede: Option<String>,
    geschlecht: Option<String>,
    land: Option<String>,
    telefon_geschaeftlich: Option<String>,
    telefon_privat: Option<String>,
    telefon_mobil: Option<String>,
    fax: Option<String>,
    geburtsdatum: Option<String>,
    geburtsort: Option<String>,
    sprache: Option<String>,
    nationalitaet: Option<String>,
    identifikationsnummer: Option<String>,
    debitorenkonto: Option<String>,
    kennzeichen: Option<String>,
    rechnungs_email: Option<String>,
    marketing_einwilligung: Option<bool>,
    leitweg_id: Option<String>,
    kostenstelle: Option<String>,
    tags: Option<String>,
) -> Result<database::Guest, String> {
    let result = database::update_guest(
        id,
        vorname,
        nachname,
        email,
        telefon.clone(),
        dpolg_mitglied,
        strasse.clone(),
        plz.clone(),
        ort.clone(),
        mitgliedsnummer,
        notizen,
        beruf,
        bundesland,
        dienststelle,
        anrede,
        geschlecht,
        land,
        telefon_geschaeftlich,
        telefon_privat,
        telefon_mobil,
        fax,
        geburtsdatum,
        geburtsort,
        sprache,
        nationalitaet,
        identifikationsnummer,
        debitorenkonto,
        kennzeichen,
        rechnungs_email,
        marketing_einwilligung,
        leitweg_id,
        kostenstelle,
        tags,
    )
    .map_err(|e| format!("Fehler beim Aktualisieren des Gastes: {}", e))?;

    // AUTO-COMPLETE: Prüfe alle Buchungen dieses Gastes auf Incomplete-Data-Reminder
    // Wenn Gast-Daten jetzt vollständig sind → schließe Reminder
    if let Err(e) = reminder_automation::on_guest_updated(id, strasse, plz, ort, telefon) {
        eprintln!("⚠️ Warnung: Konnte Incomplete-Data-Reminder nicht prüfen: {}", e);
    }

    Ok(result)
}

#[tauri::command]
fn delete_guest_command(id: i64) -> Result<(), String> {
    database::delete_guest(id)
        .map_err(|e| format!("Fehler beim Löschen des Gastes: {}", e))
}

#[tauri::command]
fn get_guest_by_id_command(id: i64) -> Result<database::Guest, String> {
    database::get_guest_by_id(id)
        .map_err(|e| format!("Fehler beim Abrufen des Gastes: {}", e))
}

#[tauri::command]
fn search_guests_command(query: String) -> Result<Vec<database::Guest>, String> {
    database::search_guests(query)
        .map_err(|e| format!("Fehler beim Suchen der Gäste: {}", e))
}

#[tauri::command]
fn get_all_guests_command() -> Result<Vec<database::Guest>, String> {
    database::get_all_guests()
        .map_err(|e| format!("Fehler beim Abrufen der Gäste: {}", e))
}

// ============================================================================
// ROOM MANAGEMENT COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
fn create_room_command(
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
    street_address: Option<String>,
    postal_code: Option<String>,
    city: Option<String>,
    notizen: Option<String>,
) -> Result<database::Room, String> {
    database::create_room(
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
        street_address,
        postal_code,
        city,
        notizen,
    )
    .map_err(|e| format!("Fehler beim Erstellen des Raums: {}", e))
}

#[tauri::command]
fn update_room_command(
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
    street_address: Option<String>,
    postal_code: Option<String>,
    city: Option<String>,
    notizen: Option<String>,
) -> Result<database::Room, String> {
    database::update_room(
        id,
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
        street_address,
        postal_code,
        city,
        notizen,
    )
    .map_err(|e| format!("Fehler beim Aktualisieren des Raums: {}", e))
}

#[tauri::command]
fn delete_room_command(id: i64) -> Result<(), String> {
    database::delete_room(id)
        .map_err(|e| format!("Fehler beim Löschen des Raums: {}", e))
}

#[tauri::command]
fn get_room_by_id_command(id: i64) -> Result<database::Room, String> {
    database::get_room_by_id(id)
        .map_err(|e| format!("Fehler beim Abrufen des Raums: {}", e))
}

#[tauri::command]
fn migrate_to_price_list_2025_command() -> Result<(), String> {
    database::migrate_to_price_list_2025()
        .map_err(|e| format!("Fehler bei der Migration: {}", e))
}

// ============================================================================
// BOOKING MANAGEMENT COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
async fn create_booking_command(
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
    payment_recipient_id: Option<i64>,
    putzplan_checkout_date: Option<String>,
    ist_dpolg_mitglied: bool,
) -> Result<database::BookingWithDetails, String> {
    println!("🔍 DEBUG: create_booking_command aufgerufen");
    println!("   room_id: {}", room_id);
    println!("   guest_id: {}", guest_id);
    println!("   ist_stiftungsfall: {}", ist_stiftungsfall);
    println!("   ist_dpolg_mitglied: {}", ist_dpolg_mitglied);
    println!("   payment_recipient_id: {:?}", payment_recipient_id);
    println!("   putzplan_checkout_date: {:?}", putzplan_checkout_date);

    match database::create_booking(
        room_id,
        guest_id,
        reservierungsnummer,
        checkin_date.clone(),
        checkout_date.clone(),
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
        putzplan_checkout_date,
        ist_dpolg_mitglied,
    ) {
        Ok(booking) => {
            println!("✅ DEBUG: create_booking_command - Buchung erfolgreich erstellt");
            println!("   booking.id: {}", booking.id);
            println!("   booking.room.name: {}", booking.room.name);
            println!("   booking.guest.vorname: {}", booking.guest.vorname);
            println!("   booking.guest.nachname: {}", booking.guest.nachname);
            println!("   booking.ist_stiftungsfall: {}", booking.ist_stiftungsfall);
            println!("📦 DEBUG: Returning BookingWithDetails to Frontend:");
            println!("   {:#?}", booking);

            // 🔄 AUTO-SYNC für neue Buchungen (wichtig für Buchungen OHNE Services!)
            // Synchronisiere BEIDE Tage: Check-IN und Check-OUT
            println!("🔄 [create_booking] Auto-Sync für Check-IN und Check-OUT");

            let checkin = booking.checkin_date.clone();
            let checkout = booking.checkout_date.clone();
            let bid = booking.id; // Booking ID für Logging

            // Sync Check-IN Tag (async in Background)
            tokio::spawn(async move {
                if let Err(e) = supabase::sync_cleaning_tasks(checkin.clone()).await {
                    eprintln!("❌ [create_booking #{}] Check-IN Sync fehlgeschlagen: {}", bid, e);
                } else {
                    println!("✅ [create_booking #{}] Check-IN Sync erfolgreich: {}", bid, checkin);
                }
            });

            // Sync Check-OUT Tag (async in Background)
            let bid2 = bid; // Clone für zweiten Spawn
            tokio::spawn(async move {
                if let Err(e) = supabase::sync_cleaning_tasks(checkout.clone()).await {
                    eprintln!("❌ [create_booking #{}] Check-OUT Sync fehlgeschlagen: {}", bid2, e);
                } else {
                    println!("✅ [create_booking #{}] Check-OUT Sync erfolgreich: {}", bid2, checkout);
                }
            });

            Ok(booking)
        }
        Err(e) => {
            eprintln!("❌ DEBUG: create_booking_command - Fehler: {}", e);
            Err(format!("Fehler beim Erstellen der Buchung: {}", e))
        }
    }
}

#[tauri::command]
fn update_booking_command(
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
    putzplan_checkout_date: Option<String>,
    ist_dpolg_mitglied: bool,
) -> Result<database::BookingWithDetails, String> {
    println!("🔍 DEBUG: update_booking_command called");
    println!("  id: {}", id);
    println!("  room_id: {}", room_id);
    println!("  guest_id: {}", guest_id);
    println!("  checkin_date: {}", checkin_date);
    println!("  checkout_date: {}", checkout_date);
    println!("  anzahl_gaeste: {}", anzahl_gaeste);
    println!("  status: {}", status);
    println!("  gesamtpreis: {}", gesamtpreis);
    println!("  bemerkungen: {:?}", bemerkungen);
    println!("  anzahl_begleitpersonen: {}", anzahl_begleitpersonen);
    println!("  grundpreis: {}", grundpreis);
    println!("  services_preis: {}", services_preis);
    println!("  rabatt_preis: {}", rabatt_preis);
    println!("  anzahl_naechte: {}", anzahl_naechte);
    println!("  ist_stiftungsfall: {}", ist_stiftungsfall);
    println!("  payment_recipient_id: {:?}", payment_recipient_id);
    println!("  putzplan_checkout_date: {:?}", putzplan_checkout_date);
    println!("  ist_dpolg_mitglied: {}", ist_dpolg_mitglied);

    database::update_booking(
        id,
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
        putzplan_checkout_date,
        ist_dpolg_mitglied,
    )
    .map_err(|e| format!("Fehler beim Aktualisieren der Buchung: {}", e))
}

#[tauri::command]
fn delete_booking_command(id: i64) -> Result<(), String> {
    database::delete_booking(id)
        .map_err(|e| format!("Fehler beim Löschen der Buchung: {}", e))
}

#[tauri::command]
fn update_booking_dates_and_room_command(
    id: i64,
    room_id: i64,
    checkin_date: String,
    checkout_date: String,
) -> Result<database::Booking, String> {
    println!("🔄 [update_booking_dates_and_room_command] Called with id: {}, room: {}, checkin: {}, checkout: {}", id, room_id, checkin_date, checkout_date);
    match database::update_booking_dates_and_room(id, room_id, checkin_date, checkout_date) {
        Ok(booking) => {
            println!("✅ [update_booking_dates_and_room_command] Successfully updated booking {}", booking.reservierungsnummer);
            Ok(booking)
        }
        Err(e) => {
            eprintln!("❌ [update_booking_dates_and_room_command] Error: {}", e);
            Err(format!("Fehler beim Aktualisieren der Buchungsdaten: {}", e))
        }
    }
}

#[tauri::command]
fn cancel_booking_command(id: i64) -> Result<database::Booking, String> {
    database::cancel_booking(id)
        .map_err(|e| format!("Fehler beim Stornieren der Buchung: {}", e))
}

#[tauri::command]
fn get_booking_by_id_command(id: i64) -> Result<database::Booking, String> {
    println!("🔍 get_booking_by_id_command called with id: {}", id);
    match database::get_booking_by_id(id) {
        Ok(booking) => {
            println!("✅ Successfully got booking: {}", booking.reservierungsnummer);
            Ok(booking)
        }
        Err(e) => {
            eprintln!("❌ Error getting booking: {}", e);
            Err(format!("Fehler beim Abrufen der Buchung: {}", e))
        }
    }
}

#[tauri::command]
fn get_booking_with_details_by_id_command(id: i64) -> Result<database::BookingWithDetails, String> {
    println!("🔍 get_booking_with_details_by_id_command called with id: {}", id);
    match database::get_booking_with_details_by_id(id) {
        Ok(booking) => {
            println!("✅ Successfully got booking with details: {}", booking.reservierungsnummer);
            Ok(booking)
        }
        Err(e) => {
            eprintln!("❌ Error getting booking with details: {}", e);
            Err(format!("Fehler beim Abrufen der Buchungsdetails: {}", e))
        }
    }
}

// ============================================================================
// ADDITIONAL SERVICES COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
fn add_service_command(
    booking_id: i64,
    service_name: String,
    service_price: f64,
    price_type: String,        // 'fixed' oder 'percent'
    original_value: f64,       // Festbetrag ODER Prozentsatz
    applies_to: String,        // 'overnight_price' oder 'total_price'
) -> Result<database::AdditionalService, String> {
    database::add_service_to_booking(booking_id, service_name, service_price, price_type, original_value, applies_to)
        .map_err(|e| format!("Fehler beim Hinzufügen des Services: {}", e))
}

#[tauri::command]
fn delete_service_command(service_id: i64) -> Result<(), String> {
    database::delete_service(service_id)
        .map_err(|e| format!("Fehler beim Löschen des Services: {}", e))
}

#[tauri::command]
fn get_booking_services_command(booking_id: i64) -> Result<Vec<database::AdditionalService>, String> {
    database::get_booking_services(booking_id)
        .map_err(|e| format!("Fehler beim Abrufen der Services: {}", e))
}

// ============================================================================
// SERVICE & DISCOUNT TEMPLATE LINKING COMMANDS - Junction Tables
// ============================================================================

#[tauri::command]
async fn link_service_template_to_booking_command(
    booking_id: i64,
    service_template_id: i64,
) -> Result<(), String> {
    println!("🔗 [link_service_template_to_booking_command] Called:");
    println!("   booking_id: {}", booking_id);
    println!("   service_template_id: {}", service_template_id);

    match database::link_service_template_to_booking(booking_id, service_template_id) {
        Ok(()) => {
            println!("✅ [link_service_template_to_booking_command] Successfully linked!");

            // 🔄 AUTO-SYNC: Service mit Emoji hinzugefügt → Mobile App synchronisieren
            // Hole checkin_date UND checkout_date der Buchung für Sync
            match database::get_booking_by_id(booking_id) {
                Ok(booking) => {
                    let checkin = booking.checkin_date.clone();
                    let checkout = booking.checkout_date.clone();

                    println!("🔄 [link_service_template_to_booking_command] Auto-Sync für beide Daten:");
                    println!("   📅 CHECK-IN:  {} (für emojis_start)", checkin);
                    println!("   📅 CHECK-OUT: {} (für emojis_end)", checkout);

                    // Fire-and-forget: Sync BEIDE Daten im Hintergrund (parallel)
                    tokio::spawn(async move {
                        // Sync beide Daten parallel mit tokio::join!
                        let (result_checkin, result_checkout) = tokio::join!(
                            supabase::sync_cleaning_tasks(checkin.clone()),
                            supabase::sync_cleaning_tasks(checkout.clone())
                        );

                        // Log Ergebnisse
                        match result_checkin {
                            Ok(_) => println!("✅ [link_service_template_to_booking_command] CHECK-IN Sync erfolgreich!"),
                            Err(e) => eprintln!("❌ [link_service_template_to_booking_command] CHECK-IN Sync Error: {}", e),
                        }

                        match result_checkout {
                            Ok(_) => println!("✅ [link_service_template_to_booking_command] CHECK-OUT Sync erfolgreich!"),
                            Err(e) => eprintln!("❌ [link_service_template_to_booking_command] CHECK-OUT Sync Error: {}", e),
                        }
                    });
                }
                Err(e) => {
                    eprintln!("⚠️  [link_service_template_to_booking_command] Konnte Buchung nicht laden für Auto-Sync: {}", e);
                    // Nicht kritisch - Service wurde trotzdem verknüpft
                }
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ [link_service_template_to_booking_command] Error: {}", e);
            Err(format!("Fehler beim Verknüpfen des Service-Templates: {}", e))
        }
    }
}

#[tauri::command]
fn link_discount_template_to_booking_command(
    booking_id: i64,
    discount_template_id: i64,
) -> Result<(), String> {
    println!("🔗 [link_discount_template_to_booking_command] Called:");
    println!("   booking_id: {}", booking_id);
    println!("   discount_template_id: {}", discount_template_id);

    match database::link_discount_template_to_booking(booking_id, discount_template_id) {
        Ok(()) => {
            println!("✅ [link_discount_template_to_booking_command] Successfully linked!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ [link_discount_template_to_booking_command] Error: {}", e);
            Err(format!("Fehler beim Verknüpfen des Discount-Templates: {}", e))
        }
    }
}

// ============================================================================
// ACCOMPANYING GUESTS COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
fn add_accompanying_guest_command(
    booking_id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
    companion_id: Option<i64>,
) -> Result<database::AccompanyingGuest, String> {
    database::add_accompanying_guest(booking_id, vorname, nachname, geburtsdatum, companion_id)
        .map_err(|e| format!("Fehler beim Hinzufügen der Begleitperson: {}", e))
}

#[tauri::command]
fn delete_accompanying_guest_command(guest_id: i64) -> Result<(), String> {
    database::delete_accompanying_guest(guest_id)
        .map_err(|e| format!("Fehler beim Löschen der Begleitperson: {}", e))
}

#[tauri::command]
fn get_booking_accompanying_guests_command(booking_id: i64) -> Result<Vec<database::AccompanyingGuest>, String> {
    database::get_booking_accompanying_guests(booking_id)
        .map_err(|e| format!("Fehler beim Abrufen der Begleitpersonen: {}", e))
}

// ============================================================================
// GUEST COMPANIONS - Permanent Pool (Phase 1.1)
// ============================================================================

#[tauri::command]
fn create_guest_companion_command(
    guest_id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
    beziehung: Option<String>,
    notizen: Option<String>,
) -> Result<database::GuestCompanion, String> {
    database::create_guest_companion(guest_id, vorname, nachname, geburtsdatum, beziehung, notizen)
        .map_err(|e| format!("Fehler beim Erstellen der Begleitperson: {}", e))
}

#[tauri::command]
fn get_guest_companions_command(guest_id: i64) -> Result<Vec<database::GuestCompanion>, String> {
    database::get_guest_companions(guest_id)
        .map_err(|e| format!("Fehler beim Abrufen der Begleitpersonen: {}", e))
}

#[tauri::command]
fn update_guest_companion_command(
    id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
    beziehung: Option<String>,
    notizen: Option<String>,
) -> Result<database::GuestCompanion, String> {
    database::update_guest_companion(id, vorname, nachname, geburtsdatum, beziehung, notizen)
        .map_err(|e| format!("Fehler beim Aktualisieren der Begleitperson: {}", e))
}

#[tauri::command]
fn delete_guest_companion_command(id: i64) -> Result<(), String> {
    database::delete_guest_companion(id)
        .map_err(|e| format!("Fehler beim Löschen der Begleitperson: {}", e))
}

// ============================================================================
// DISCOUNTS COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
fn add_discount_command(
    booking_id: i64,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
) -> Result<database::Discount, String> {
    database::add_discount_to_booking(booking_id, discount_name, discount_type, discount_value)
        .map_err(|e| format!("Fehler beim Hinzufügen des Rabatts: {}", e))
}

#[tauri::command]
fn delete_discount_command(discount_id: i64) -> Result<(), String> {
    database::delete_discount(discount_id)
        .map_err(|e| format!("Fehler beim Löschen des Rabatts: {}", e))
}

#[tauri::command]
fn get_booking_discounts_command(booking_id: i64) -> Result<Vec<database::Discount>, String> {
    database::get_booking_discounts(booking_id)
        .map_err(|e| format!("Fehler beim Abrufen der Rabatte: {}", e))
}

// ============================================================================
// VALIDATION COMMANDS - Phase 2
// ============================================================================

#[tauri::command]
fn validate_email_command(email: String) -> Result<String, String> {
    validation::validate_email(&email)?;
    Ok("Email ist gültig".to_string())
}

#[tauri::command]
fn validate_date_range_command(checkin: String, checkout: String) -> Result<String, String> {
    validation::validate_date_range(&checkin, &checkout)?;
    Ok("Datumsbereich ist gültig".to_string())
}

#[tauri::command]
fn check_room_availability_command(
    room_id: i64,
    checkin: String,
    checkout: String,
    exclude_booking_id: Option<i64>,
) -> Result<bool, String> {
    let conn = Connection::open(database::get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    validation::check_room_availability(room_id, &checkin, &checkout, exclude_booking_id, &conn)
}

// ============================================================================
// PRICING COMMANDS - Phase 2
// ============================================================================

#[tauri::command]
fn calculate_nights_command(checkin: String, checkout: String) -> Result<i32, String> {
    pricing::calculate_nights(&checkin, &checkout)
}


// ============================================================================
// NEUE ARCHITEKTUR: Single Source of Truth Preisberechnung
// ============================================================================

/// NEUER COMMAND: Vollständige Preisberechnung mit allen Details
///
/// Dieser Command ersetzt calculate_booking_price_command() und liefert
/// ALLE Preis-Informationen in einem strukturierten Format.
///
/// Frontend schickt komplette Service/Discount-Objekte (nicht nur Tupel!)
/// Backend berechnet ALLES und gibt vollständigen Breakdown zurück.
#[tauri::command]
fn calculate_full_booking_price_command(
    room_id: i64,
    checkin: String,
    checkout: String,
    is_member: bool,
    services: Option<Vec<pricing::ServiceInput>>,
    discounts: Option<Vec<pricing::DiscountInput>>,
) -> Result<pricing::FullPriceBreakdown, String> {
    let conn = Connection::open(database::get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let services = services.unwrap_or_default();
    let discounts = discounts.unwrap_or_default();

    // Eine Funktion macht ALLES - Single Source of Truth!
    pricing::calculate_full_booking_price(
        room_id,
        &checkin,
        &checkout,
        is_member,
        &services,
        &discounts,
        &conn,
    )
}

// ============================================================================
// REPORTS & STATISTICS COMMANDS
// ============================================================================

#[tauri::command]
fn get_report_stats_command(start_date: String, end_date: String) -> Result<database::ReportStats, String> {
    database::get_report_stats(&start_date, &end_date)
        .map_err(|e| format!("Fehler beim Laden der Statistiken: {}", e))
}

#[tauri::command]
fn get_room_occupancy_command(start_date: String, end_date: String) -> Result<Vec<database::RoomOccupancy>, String> {
    database::get_room_occupancy(&start_date, &end_date)
        .map_err(|e| format!("Fehler beim Laden der Belegungsstatistiken: {}", e))
}

// ============================================================================
// EMAIL SYSTEM COMMANDS - Phase 6
// ============================================================================

#[tauri::command]
fn save_email_config_command(
    smtp_server: String,
    smtp_port: i32,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
    use_tls: bool,
) -> Result<database::EmailConfig, String> {
    email::save_email_config(smtp_server, smtp_port, smtp_username, smtp_password, from_email, from_name, use_tls)
}

#[tauri::command]
fn get_email_config_command() -> Result<database::EmailConfig, String> {
    email::get_email_config()
}

#[tauri::command]
async fn test_email_connection_command(
    smtp_server: String,
    smtp_port: i32,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
    test_recipient: String,
) -> Result<String, String> {
    email::test_email_connection(smtp_server, smtp_port, smtp_username, smtp_password, from_email, from_name, test_recipient).await
}

#[tauri::command]
fn get_all_templates_command() -> Result<Vec<database::EmailTemplate>, String> {
    database::get_all_templates()
}

#[tauri::command]
fn get_template_by_name_command(template_name: String) -> Result<database::EmailTemplate, String> {
    email::get_template_by_name(&template_name)
}

#[tauri::command]
fn update_template_command(
    id: i64,
    subject: String,
    body: String,
    description: Option<String>,
) -> Result<database::EmailTemplate, String> {
    database::update_template(id, subject, body, description)
}

#[tauri::command]
async fn send_confirmation_email_command(app: tauri::AppHandle, booking_id: i64) -> Result<String, String> {
    email::send_confirmation_email(app, booking_id).await
}

#[tauri::command]
async fn send_reminder_email_command(booking_id: i64) -> Result<String, String> {
    email::send_reminder_email(booking_id).await
}

#[tauri::command]
async fn send_invoice_email_command(app: tauri::AppHandle, booking_id: i64) -> Result<String, String> {
    use crate::pdf_generator_html::generate_invoice_pdf_html;
    use crate::database::get_booking_with_details_by_id;
    use std::path::PathBuf;

    // 1. Buchung laden
    let booking = get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    // 2. PDF generieren
    let pdf_path_str = generate_invoice_pdf_html(app.clone(), booking)
        .map_err(|e| format!("Fehler beim Generieren des PDFs: {}", e))?;

    // 3. String zu PathBuf konvertieren
    let pdf_path = PathBuf::from(pdf_path_str);

    // 4. Email mit PDF-Anhang senden
    email::send_invoice_email_with_pdf(app, booking_id, pdf_path).await
}

#[tauri::command]
fn get_email_logs_for_booking_command(booking_id: i64) -> Result<Vec<database::EmailLog>, String> {
    email::get_email_logs_for_booking(booking_id)
}

#[tauri::command]
fn get_all_email_logs_command() -> Result<Vec<database::EmailLog>, String> {
    email::get_all_email_logs()
}

#[tauri::command]
fn get_recent_email_logs_command(limit: i32) -> Result<Vec<database::EmailLog>, String> {
    email::get_recent_email_logs(limit)
}

#[tauri::command]
fn delete_email_log_command(log_id: i64) -> Result<(), String> {
    email::delete_email_log(log_id)
}

#[tauri::command]
async fn send_payment_reminder_email_command(booking_id: i64) -> Result<String, String> {
    email::send_payment_reminder_email(booking_id).await
}

#[tauri::command]
async fn send_cancellation_email_command(booking_id: i64) -> Result<String, String> {
    email::send_cancellation_email(booking_id).await
}

#[tauri::command]
fn mark_booking_as_paid_command(
    booking_id: i64,
    zahlungsmethode: String,
) -> Result<database::Booking, String> {
    let result = database::mark_booking_as_paid(booking_id, zahlungsmethode)
        .map_err(|e| format!("Fehler beim Markieren der Buchung als bezahlt: {}", e))?;

    // AUTO-COMPLETE: Schließe Zahlungs-Reminder automatisch (2025 Best Practice)
    if let Err(e) = reminder_automation::on_booking_paid(booking_id) {
        eprintln!("⚠️ Fehler beim Auto-Complete von Zahlungs-Reminder: {}", e);
    }

    Ok(result)
}

#[tauri::command]
fn mark_invoice_sent_command(
    booking_id: i64,
    email_address: String,
) -> Result<database::Booking, String> {
    let result = database::mark_invoice_sent(booking_id, email_address)
        .map_err(|e| format!("Fehler beim Markieren der Rechnung als versendet: {}", e))?;

    // AUTO-COMPLETE: Schließe Invoice-Reminder automatisch (2025 Best Practice)
    if let Err(e) = reminder_automation::on_invoice_sent(booking_id) {
        eprintln!("⚠️ Fehler beim Auto-Complete von Invoice-Reminder: {}", e);
    }

    Ok(result)
}

#[tauri::command]
fn update_booking_statuses_command() -> Result<usize, String> {
    database::update_booking_statuses_by_date()
        .map_err(|e| format!("Fehler beim Status-Update: {}", e))
}

#[tauri::command]
fn update_booking_status_command(
    booking_id: i64,
    new_status: String,
) -> Result<database::Booking, String> {
    let result = database::update_booking_status(booking_id, new_status.clone())
        .map_err(|e| format!("Fehler beim Ändern des Status: {}", e))?;

    // AUTO-COMPLETE: Wenn Buchung storniert → Alle Reminder löschen
    if new_status.to_lowercase() == "storniert" {
        if let Err(e) = reminder_automation::on_booking_cancelled(booking_id) {
            eprintln!("⚠️ Warnung: Konnte Reminder nicht löschen: {}", e);
        }
    }

    Ok(result)
}

#[tauri::command]
fn update_booking_payment_command(
    app: tauri::AppHandle,
    booking_id: i64,
    bezahlt: bool,
    zahlungsmethode: Option<String>,
    bezahlt_am: Option<String>,  // NEU: Bezahldatum vom User
) -> Result<database::Booking, String> {
    println!("🔍 [update_booking_payment_command] Called:");
    println!("   booking_id: {}", booking_id);
    println!("   bezahlt: {}", bezahlt);
    println!("   zahlungsmethode: {:?}", zahlungsmethode);
    println!("   bezahlt_am: {:?}", bezahlt_am);

    let result = database::update_booking_payment(booking_id, bezahlt, zahlungsmethode, bezahlt_am)
        .map_err(|e| format!("Fehler beim Ändern des Bezahlt-Status: {}", e))?;

    // AUTO-REMINDER: Wenn als bezahlt markiert → Payment-Reminder schließen
    if bezahlt {
        if let Err(e) = reminder_automation::on_booking_paid(booking_id) {
            eprintln!("⚠️  Warnung: Konnte Payment-Reminder nicht auto-complete: {}", e);
            // Nicht kritisch - Fehler nicht weitergeben
        }

        // EMIT EVENT ZUM FRONTEND für Optimistic UI Update
        use tauri::Emitter;
        let _ = app.emit("reminder-updated", serde_json::json!({
            "reminderType": "auto_payment",
            "bookingId": booking_id
        }));
        println!("📤 [TAURI EVENT] reminder-updated event emitted to frontend (payment)");
    }

    Ok(result)
}

// ============================================================================
// COMPANY SETTINGS COMMANDS
// ============================================================================

#[tauri::command]
fn get_company_settings_command() -> Result<database::CompanySettings, String> {
    database::get_company_settings()
        .map_err(|e| format!("Fehler beim Laden der Einstellungen: {}", e))
}

#[tauri::command]
fn save_company_settings_command(settings: database::CompanySettings) -> Result<database::CompanySettings, String> {
    database::save_company_settings(settings)
        .map_err(|e| format!("Fehler beim Speichern der Einstellungen: {}", e))
}

// ============================================================================
// PAYMENT SETTINGS COMMANDS
// ============================================================================

#[tauri::command]
fn get_payment_settings_command() -> Result<database::PaymentSettings, String> {
    database::get_payment_settings()
        .map_err(|e| format!("Fehler beim Laden der Zahlungseinstellungen: {}", e))
}

#[tauri::command]
fn save_payment_settings_command(settings: database::PaymentSettings) -> Result<database::PaymentSettings, String> {
    database::save_payment_settings(settings)
        .map_err(|e| format!("Fehler beim Speichern der Zahlungseinstellungen: {}", e))
}

#[tauri::command]
fn get_notification_settings_command() -> Result<database::NotificationSettings, String> {
    database::get_notification_settings()
        .map_err(|e| format!("Fehler beim Laden der Benachrichtigungseinstellungen: {}", e))
}

#[tauri::command]
fn save_notification_settings_command(settings: database::NotificationSettings) -> Result<database::NotificationSettings, String> {
    database::save_notification_settings(settings)
        .map_err(|e| format!("Fehler beim Speichern der Benachrichtigungseinstellungen: {}", e))
}

// ============================================================================
// PRICING SETTINGS COMMANDS
// ============================================================================

#[tauri::command]
fn get_pricing_settings_command() -> Result<database::PricingSettings, String> {
    database::get_pricing_settings()
        .map_err(|e| format!("Fehler beim Laden der Preiseinstellungen: {}", e))
}

#[tauri::command]
fn save_pricing_settings_command(settings: database::PricingSettings) -> Result<database::PricingSettings, String> {
    database::save_pricing_settings(settings)
        .map_err(|e| format!("Fehler beim Speichern der Preiseinstellungen: {}", e))
}

#[tauri::command]
fn upload_logo_command(app: tauri::AppHandle, source_path: String) -> Result<String, String> {
    use tauri::Manager;

    println!("📁 Logo Upload gestartet...");
    println!("   Source: {}", source_path);

    // Get app data directory
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Fehler beim App-Verzeichnis: {}", e))?;

    // Create logos directory
    let logos_dir = app_data_dir.join("logos");
    std::fs::create_dir_all(&logos_dir)
        .map_err(|e| format!("Fehler beim Erstellen des Logo-Verzeichnisses: {}", e))?;

    println!("   Logos dir: {:?}", logos_dir);

    // Get filename from source
    let source = std::path::Path::new(&source_path);
    let filename = source.file_name()
        .ok_or("Ungültiger Dateiname")?;

    // Copy file to logos directory
    let dest_path = logos_dir.join(filename);
    std::fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("Fehler beim Kopieren der Datei: {}", e))?;

    let result = dest_path.to_string_lossy().to_string();
    println!("✅ Logo erfolgreich hochgeladen: {}", result);

    Ok(result)
}

// ============================================================================
// BACKUP COMMANDS
// ============================================================================

#[tauri::command]
fn create_backup_command(app: tauri::AppHandle) -> Result<backup::BackupInfo, String> {
    backup::create_backup(&app)
}

#[tauri::command]
fn list_backups_command(app: tauri::AppHandle) -> Result<Vec<backup::BackupInfo>, String> {
    backup::list_backups(&app)
}

#[tauri::command]
fn restore_backup_command(backup_path: String) -> Result<String, String> {
    backup::restore_backup(backup_path)
}

#[tauri::command]
fn delete_backup_command(backup_path: String) -> Result<String, String> {
    backup::delete_backup(backup_path)
}

#[tauri::command]
fn get_backup_settings_command() -> Result<backup::BackupSettings, String> {
    backup::get_backup_settings()
}

#[tauri::command]
fn save_backup_settings_command(settings: backup::BackupSettings) -> Result<backup::BackupSettings, String> {
    backup::save_backup_settings(settings)
}

#[tauri::command]
fn open_backup_folder_command(app: tauri::AppHandle) -> Result<String, String> {
    let backup_dir = backup::get_backup_dir(&app)?;

    // Open folder in file explorer
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&backup_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&backup_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&backup_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;
    }

    Ok(backup_dir.to_string_lossy().to_string())
}

// ============================================================================
// TRANSACTION LOG & UNDO/REDO COMMANDS
// ============================================================================

#[tauri::command]
fn get_recent_transactions_command(limit: i64) -> Result<Vec<transaction_log::TransactionLog>, String> {
    transaction_log::get_recent_transactions(limit)
}

#[tauri::command]
fn undo_transaction_command(log_id: i64) -> Result<String, String> {
    transaction_log::undo_transaction(log_id)
}

#[tauri::command]
fn cleanup_old_logs_command(days: i64) -> Result<usize, String> {
    transaction_log::cleanup_old_logs(days)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    println!("Initializing database...");
    match init_database() {
        Ok(_) => println!("Database initialized successfully!"),
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            eprintln!("Error details: {:?}", e);
            // Don't return, let the app start anyway for debugging
        }
    }

    // Initialize transaction log table
    println!("Initializing transaction log...");
    match transaction_log::init_transaction_log_table() {
        Ok(_) => println!("Transaction log initialized successfully!"),
        Err(e) => {
            eprintln!("Failed to initialize transaction log: {}", e);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_all_rooms,
            get_all_bookings,
            // Guest Management
            create_guest_command,
            update_guest_command,
            delete_guest_command,
            get_guest_by_id_command,
            search_guests_command,
            get_all_guests_command,
            // Room Management
            create_room_command,
            update_room_command,
            delete_room_command,
            get_room_by_id_command,
            migrate_to_price_list_2025_command,
            // Booking Management
            create_booking_command,
            update_booking_command,
            update_booking_dates_and_room_command,
            delete_booking_command,
            cancel_booking_command,
            get_booking_by_id_command,
            get_booking_with_details_by_id_command,
            // Additional Services
            add_service_command,
            delete_service_command,
            get_booking_services_command,
            // Service & Discount Template Linking
            link_service_template_to_booking_command,
            link_discount_template_to_booking_command,
            // Accompanying Guests
            add_accompanying_guest_command,
            delete_accompanying_guest_command,
            get_booking_accompanying_guests_command,
            // Guest Companions (Permanent Pool)
            create_guest_companion_command,
            get_guest_companions_command,
            update_guest_companion_command,
            delete_guest_companion_command,
            // Discounts
            add_discount_command,
            delete_discount_command,
            get_booking_discounts_command,
            // Validation
            validate_email_command,
            validate_date_range_command,
            check_room_availability_command,
            // Pricing
            calculate_nights_command,
            calculate_full_booking_price_command,  // Single Source of Truth
            // Reports & Statistics
            get_report_stats_command,
            get_room_occupancy_command,
            // Email System
            save_email_config_command,
            get_email_config_command,
            test_email_connection_command,
            get_all_templates_command,
            get_template_by_name_command,
            update_template_command,
            send_confirmation_email_command,
            send_reminder_email_command,
            send_invoice_email_command,
            get_email_logs_for_booking_command,
            get_all_email_logs_command,
            get_recent_email_logs_command,
            delete_email_log_command,
            send_payment_reminder_email_command,
            send_cancellation_email_command,
            mark_booking_as_paid_command,
            mark_invoice_sent_command,
            update_booking_statuses_command,
            update_booking_status_command,
            update_booking_payment_command,
            // Email Scheduler
            email_scheduler::trigger_email_check,
            email_scheduler::get_scheduled_emails,
            // Reminder Automation
            reminder_automation::trigger_daily_reminder_check,
            // Company Settings
            get_company_settings_command,
            save_company_settings_command,
            upload_logo_command,
            // Payment Settings
            get_payment_settings_command,
            save_payment_settings_command,
            get_notification_settings_command,
            save_notification_settings_command,
            // Pricing Settings
            get_pricing_settings_command,
            save_pricing_settings_command,
            // Invoice HTML Generation
            invoice_html::generate_invoice_html,
            // PDF Generation
            pdf_generator::generate_invoice_pdf_command,
            pdf_generator::get_invoice_pdfs_for_booking_command,
            pdf_generator::open_invoices_folder_command,
            pdf_generator::open_pdf_file_command,
            pdf_generator::generate_and_send_invoice_command,
            // Service Templates
            create_service_template_command,
            get_all_service_templates_command,
            get_active_service_templates_command,
            update_service_template_command,
            delete_service_template_command,
            // Discount Templates
            create_discount_template_command,
            get_all_discount_templates_command,
            get_active_discount_templates_command,
            update_discount_template_command,
            delete_discount_template_command,
            // Backup
            create_backup_command,
            list_backups_command,
            restore_backup_command,
            delete_backup_command,
            // Supabase / Cleaning Plan Sync
            supabase::sync_cleaning_tasks,
            supabase::sync_week_ahead,
            supabase::sync_affected_dates,
            supabase::delete_booking_tasks,
            supabase::cleanup_cleaning_tasks,
            supabase::get_cleaning_stats,
            supabase::migrate_cleaning_tasks_schema,
            supabase::test_emoji_sync,
            // Cleaning Timeline PDF Export
            cleaning_timeline_pdf::export_cleaning_timeline_pdf,
            cleaning_timeline_pdf::open_putzplan_folder,
            get_backup_settings_command,
            save_backup_settings_command,
            open_backup_folder_command,
            // Transaction Log & Undo/Redo
            get_recent_transactions_command,
            undo_transaction_command,
            cleanup_old_logs_command,
            // Reminder System
            create_reminder_command,
            get_all_reminders_command,
            get_reminders_for_booking_command,
            get_urgent_reminders_command,
            update_reminder_command,
            mark_reminder_completed_command,
            snooze_reminder_command,
            delete_reminder_command,
            restore_reminder_command,
            get_reminder_settings_command,
            save_reminder_settings_command,
            // Payment Recipients
            payment_recipients::get_payment_recipients,
            payment_recipients::get_payment_recipient,
            payment_recipients::create_payment_recipient,
            payment_recipients::update_payment_recipient,
            payment_recipients::delete_payment_recipient,
            // Guest Credit System
            database::add_guest_credit,
            database::get_guest_credit_balance,
            database::get_guest_credit_transactions,
            database::use_guest_credit_for_booking,
        ])
        .setup(|app| {
            // KRITISCH: Database Path ZUERST initialisieren (BEVOR irgendwelche DB-Operationen!)
            println!("📁 Initialisiere Datenbank-Pfad...");
            if let Err(e) = database::init_database_path(&app.handle()) {
                eprintln!("❌ FEHLER beim Initialisieren des Datenbank-Pfads: {}", e);
                // App trotzdem starten für Debugging
            }

            // Starte Email-Scheduler im Hintergrund
            println!("🚀 Starte Email-Scheduler...");
            email_scheduler::start_email_scheduler();
            println!("✅ Email-Scheduler aktiv");

            // Automatisches Backup beim App-Start (falls aktiviert)
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // Kurz warten damit Datenbank initialisiert ist
                std::thread::sleep(std::time::Duration::from_secs(2));

                match backup::get_backup_settings() {
                    Ok(settings) => {
                        if settings.auto_backup_enabled && settings.backup_interval == "on_startup" {
                            println!("💾 Erstelle automatisches Backup...");
                            match backup::create_backup(&app_handle) {
                                Ok(backup_info) => {
                                    println!("✅ Automatisches Backup erstellt: {}", backup_info.filename);
                                }
                                Err(e) => {
                                    eprintln!("❌ Fehler beim automatischen Backup: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Konnte Backup-Einstellungen nicht laden: {}", e);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================================
// REMINDER SYSTEM COMMANDS
// ============================================================================

#[tauri::command]
fn create_reminder_command(
    booking_id: Option<i64>,
    reminder_type: String,
    title: String,
    description: Option<String>,
    due_date: String,
    priority: String,
) -> Result<database::Reminder, String> {
    reminders::create_reminder(booking_id, reminder_type, title, description, due_date, priority)
        .map_err(|e| format!("Fehler beim Erstellen der Erinnerung: {}", e))
}

#[tauri::command]
fn get_all_reminders_command(include_completed: bool) -> Result<Vec<database::Reminder>, String> {
    reminders::get_all_reminders(include_completed)
        .map_err(|e| format!("Fehler beim Laden der Erinnerungen: {}", e))
}

#[tauri::command]
fn get_reminders_for_booking_command(booking_id: i64) -> Result<Vec<database::Reminder>, String> {
    reminders::get_reminders_for_booking(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Erinnerungen: {}", e))
}

#[tauri::command]
fn get_urgent_reminders_command() -> Result<Vec<database::Reminder>, String> {
    reminders::get_urgent_reminders()
        .map_err(|e| format!("Fehler beim Laden dringender Erinnerungen: {}", e))
}

#[tauri::command]
fn update_reminder_command(
    id: i64,
    title: String,
    description: Option<String>,
    due_date: String,
    priority: String,
) -> Result<database::Reminder, String> {
    reminders::update_reminder(id, title, description, due_date, priority)
        .map_err(|e| format!("Fehler beim Aktualisieren der Erinnerung: {}", e))
}

#[tauri::command]
fn mark_reminder_completed_command(id: i64, completed: bool) -> Result<database::Reminder, String> {
    reminders::mark_reminder_completed(id, completed)
        .map_err(|e| format!("Fehler beim Markieren der Erinnerung: {}", e))
}

#[tauri::command]
fn snooze_reminder_command(id: i64, snooze_until: String) -> Result<database::Reminder, String> {
    reminders::snooze_reminder(id, snooze_until)
        .map_err(|e| format!("Fehler beim Verschieben der Erinnerung: {}", e))
}

#[tauri::command]
fn delete_reminder_command(id: i64) -> Result<(), String> {
    reminders::delete_reminder(id)
        .map_err(|e| format!("Fehler beim Löschen der Erinnerung: {}", e))
}

#[tauri::command]
fn restore_reminder_command(reminder: database::Reminder) -> Result<database::Reminder, String> {
    reminders::restore_reminder(reminder)
        .map_err(|e| format!("Fehler beim Wiederherstellen der Erinnerung: {}", e))
}

#[tauri::command]
fn get_reminder_settings_command() -> Result<database::ReminderSettings, String> {
    reminders::get_reminder_settings()
        .map_err(|e| format!("Fehler beim Laden der Erinnerungs-Einstellungen: {}", e))
}

#[tauri::command]
fn save_reminder_settings_command(settings: database::ReminderSettings) -> Result<database::ReminderSettings, String> {
    reminders::save_reminder_settings(settings)
        .map_err(|e| format!("Fehler beim Speichern der Erinnerungs-Einstellungen: {}", e))
}

// ============================================================================
// SERVICE TEMPLATES COMMANDS
// ============================================================================

#[tauri::command]
fn create_service_template_command(
    name: String,
    description: Option<String>,
    price: f64,
    price_type: String,
    applies_to: String,
    emoji: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
) -> Result<database::ServiceTemplate, String> {
    database::create_service_template(name, description, price, price_type, applies_to, emoji, show_in_cleaning_plan, cleaning_plan_position)
}

#[tauri::command]
fn get_all_service_templates_command() -> Result<Vec<database::ServiceTemplate>, String> {
    database::get_all_service_templates()
}

#[tauri::command]
fn get_active_service_templates_command() -> Result<Vec<database::ServiceTemplate>, String> {
    database::get_active_service_templates()
}

#[tauri::command]
fn update_service_template_command(
    id: i64,
    name: String,
    description: Option<String>,
    price: f64,
    is_active: bool,
    price_type: String,
    applies_to: String,
    emoji: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
) -> Result<database::ServiceTemplate, String> {
    database::update_service_template(id, name, description, price, is_active, price_type, applies_to, emoji, show_in_cleaning_plan, cleaning_plan_position)
}

#[tauri::command]
fn delete_service_template_command(id: i64) -> Result<(), String> {
    database::delete_service_template(id)
}

// ============================================================================
// DISCOUNT TEMPLATES COMMANDS
// ============================================================================

#[tauri::command]
fn create_discount_template_command(
    name: String,
    description: Option<String>,
    discount_type: String,
    discount_value: f64,
    emoji: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
    applies_to: String,
) -> Result<database::DiscountTemplate, String> {
    database::create_discount_template(name, description, discount_type, discount_value, emoji, show_in_cleaning_plan, cleaning_plan_position, applies_to)
}

#[tauri::command]
fn get_all_discount_templates_command() -> Result<Vec<database::DiscountTemplate>, String> {
    database::get_all_discount_templates()
}

#[tauri::command]
fn get_active_discount_templates_command() -> Result<Vec<database::DiscountTemplate>, String> {
    database::get_active_discount_templates()
}

#[tauri::command]
fn update_discount_template_command(
    id: i64,
    name: String,
    description: Option<String>,
    discount_type: String,
    discount_value: f64,
    is_active: bool,
    emoji: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
    applies_to: String,
) -> Result<database::DiscountTemplate, String> {
    database::update_discount_template(id, name, description, discount_type, discount_value, is_active, emoji, show_in_cleaning_plan, cleaning_plan_position, applies_to)
}

#[tauri::command]
fn delete_discount_template_command(id: i64) -> Result<(), String> {
    database::delete_discount_template(id)
}