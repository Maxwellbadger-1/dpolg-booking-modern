// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod validation;
mod pricing;

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
    )
    .map_err(|e| format!("Fehler beim Erstellen des Gastes: {}", e))
}

#[tauri::command]
fn update_guest_command(
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
) -> Result<database::Guest, String> {
    database::update_guest(
        id,
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
    )
    .map_err(|e| format!("Fehler beim Aktualisieren des Gastes: {}", e))
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
    ort: String,
    schluesselcode: Option<String>,
) -> Result<database::Room, String> {
    database::create_room(
        name,
        gebaeude_typ,
        capacity,
        price_member,
        price_non_member,
        ort,
        schluesselcode,
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
    ort: String,
    schluesselcode: Option<String>,
) -> Result<database::Room, String> {
    database::update_room(
        id,
        name,
        gebaeude_typ,
        capacity,
        price_member,
        price_non_member,
        ort,
        schluesselcode,
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

// ============================================================================
// BOOKING MANAGEMENT COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
fn create_booking_command(
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
) -> Result<database::Booking, String> {
    database::create_booking(
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
        anzahl_naechte,
    )
    .map_err(|e| format!("Fehler beim Erstellen der Buchung: {}", e))
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
) -> Result<database::Booking, String> {
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
    )
    .map_err(|e| format!("Fehler beim Aktualisieren der Buchung: {}", e))
}

#[tauri::command]
fn delete_booking_command(id: i64) -> Result<(), String> {
    database::delete_booking(id)
        .map_err(|e| format!("Fehler beim Löschen der Buchung: {}", e))
}

#[tauri::command]
fn cancel_booking_command(id: i64) -> Result<database::Booking, String> {
    database::cancel_booking(id)
        .map_err(|e| format!("Fehler beim Stornieren der Buchung: {}", e))
}

#[tauri::command]
fn get_booking_by_id_command(id: i64) -> Result<database::Booking, String> {
    database::get_booking_by_id(id)
        .map_err(|e| format!("Fehler beim Abrufen der Buchung: {}", e))
}

// ============================================================================
// ADDITIONAL SERVICES COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
fn add_service_command(
    booking_id: i64,
    service_name: String,
    service_price: f64,
) -> Result<database::AdditionalService, String> {
    database::add_service_to_booking(booking_id, service_name, service_price)
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
// ACCOMPANYING GUESTS COMMANDS - Phase 1.2
// ============================================================================

#[tauri::command]
fn add_accompanying_guest_command(
    booking_id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
) -> Result<database::AccompanyingGuest, String> {
    database::add_accompanying_guest(booking_id, vorname, nachname, geburtsdatum)
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
) -> Result<bool, String> {
    let conn = Connection::open(database::get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;
    validation::check_room_availability(room_id, &checkin, &checkout, None, &conn)
}

// ============================================================================
// PRICING COMMANDS - Phase 2
// ============================================================================

#[tauri::command]
fn calculate_nights_command(checkin: String, checkout: String) -> Result<i32, String> {
    pricing::calculate_nights(&checkin, &checkout)
}

#[tauri::command]
fn calculate_booking_price_command(
    room_id: i64,
    checkin: String,
    checkout: String,
    is_member: bool,
) -> Result<serde_json::Value, String> {
    let conn = Connection::open(database::get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let services: Vec<(String, f64)> = vec![];
    let discounts: Vec<(String, String, f64)> = vec![];

    let (grundpreis, services_preis, rabatt_preis, gesamtpreis, anzahl_naechte) =
        pricing::calculate_booking_total(room_id, &checkin, &checkout, is_member, &services, &discounts, &conn)?;

    Ok(serde_json::json!({
        "grundpreis": grundpreis,
        "servicesPreis": services_preis,
        "rabattPreis": rabatt_preis,
        "gesamtpreis": gesamtpreis,
        "anzahlNaechte": anzahl_naechte
    }))
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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            // Booking Management
            create_booking_command,
            update_booking_command,
            delete_booking_command,
            cancel_booking_command,
            get_booking_by_id_command,
            // Additional Services
            add_service_command,
            delete_service_command,
            get_booking_services_command,
            // Accompanying Guests
            add_accompanying_guest_command,
            delete_accompanying_guest_command,
            get_booking_accompanying_guests_command,
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
            calculate_booking_price_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}