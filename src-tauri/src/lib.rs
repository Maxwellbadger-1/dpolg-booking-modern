// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod validation;
mod pricing;
mod email;
mod pdf_generator;

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
fn update_booking_dates_and_room_command(
    id: i64,
    room_id: i64,
    checkin_date: String,
    checkout_date: String,
) -> Result<database::Booking, String> {
    database::update_booking_dates_and_room(id, room_id, checkin_date, checkout_date)
        .map_err(|e| format!("Fehler beim Aktualisieren der Buchungsdaten: {}", e))
}

#[tauri::command]
fn update_booking_statuses_command() -> Result<usize, String> {
    database::update_booking_statuses_by_date()
        .map_err(|e| format!("Fehler beim Aktualisieren der Buchungs-Status: {}", e))
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
    email::get_all_templates()
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
    email::update_template(id, subject, body, description)
}

#[tauri::command]
async fn send_confirmation_email_command(booking_id: i64) -> Result<String, String> {
    email::send_confirmation_email(booking_id).await
}

#[tauri::command]
async fn send_reminder_email_command(booking_id: i64) -> Result<String, String> {
    email::send_reminder_email(booking_id).await
}

#[tauri::command]
async fn send_invoice_email_command(booking_id: i64) -> Result<String, String> {
    email::send_invoice_email(booking_id).await
}

#[tauri::command]
fn get_email_logs_for_booking_command(booking_id: i64) -> Result<Vec<database::EmailLog>, String> {
    email::get_email_logs_for_booking(booking_id)
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
    database::mark_booking_as_paid(booking_id, zahlungsmethode)
        .map_err(|e| format!("Fehler beim Markieren der Buchung als bezahlt: {}", e))
}

// ============================================================================
// PDF INVOICE GENERATION - Phase 7.1
// ============================================================================

#[tauri::command]
fn generate_invoice_pdf_command(booking_id: i64) -> Result<String, String> {
    use std::path::PathBuf;

    // 1. Buchung mit Details laden
    let booking = database::get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    // 2. App-Data Ordner ermitteln (Platform-spezifisch)
    let app_data_dir = if cfg!(target_os = "macos") {
        let home = std::env::var("HOME")
            .map_err(|_| "Konnte HOME nicht ermitteln".to_string())?;
        PathBuf::from(home).join("Library/Application Support/com.dpolg.booking")
    } else if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Konnte APPDATA nicht ermitteln".to_string())?;
        PathBuf::from(appdata).join("com.dpolg.booking")
    } else {
        // Linux
        let home = std::env::var("HOME")
            .map_err(|_| "Konnte HOME nicht ermitteln".to_string())?;
        PathBuf::from(home).join(".local/share/com.dpolg.booking")
    };

    // Invoices-Unterordner erstellen
    let invoices_dir = app_data_dir.join("invoices");
    std::fs::create_dir_all(&invoices_dir)
        .map_err(|e| format!("Fehler beim Erstellen des Invoices-Ordners: {}", e))?;

    // 3. PDF-Dateiname generieren
    let pdf_filename = format!("Rechnung_{}.pdf", booking.reservierungsnummer);
    let pdf_path = invoices_dir.join(&pdf_filename);

    // 4. PDF generieren
    let guest_name = format!("{} {}", booking.guest.vorname, booking.guest.nachname);

    // Adresse zusammenbauen
    let guest_address = booking.guest.strasse.as_deref();
    let guest_city = if let (Some(plz), Some(ort)) = (&booking.guest.plz, &booking.guest.ort) {
        Some(format!("{} {}", plz, ort))
    } else {
        None
    };
    let guest_city_str = guest_city.as_deref();

    pdf_generator::generate_invoice_pdf(
        booking.id,
        &booking.reservierungsnummer,
        &guest_name,
        guest_address,
        guest_city_str,
        Some("DEUTSCHLAND"), // Standardmäßig Deutschland
        &booking.room.name,
        &booking.checkin_date,
        &booking.checkout_date,
        booking.anzahl_gaeste,
        booking.grundpreis,
        booking.services_preis,
        booking.rabatt_preis,
        booking.gesamtpreis,
        &pdf_path,
    )?;

    // 5. Pfad zurückgeben
    Ok(pdf_path.to_string_lossy().to_string())
}

/// Generiert PDF-Rechnung UND sendet sie automatisch per Email
#[tauri::command]
async fn generate_and_send_invoice_command(booking_id: i64) -> Result<String, String> {
    use std::path::PathBuf;

    println!("📧 Generiere PDF-Rechnung und sende Email für Buchung {}...", booking_id);

    // 1. Buchung mit Details laden
    let booking = database::get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    // 2. App-Data Ordner ermitteln (Platform-spezifisch)
    let app_data_dir = if cfg!(target_os = "macos") {
        let home = std::env::var("HOME")
            .map_err(|_| "Konnte HOME nicht ermitteln".to_string())?;
        PathBuf::from(home).join("Library/Application Support/com.dpolg.booking")
    } else if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Konnte APPDATA nicht ermitteln".to_string())?;
        PathBuf::from(appdata).join("com.dpolg.booking")
    } else {
        // Linux
        let home = std::env::var("HOME")
            .map_err(|_| "Konnte HOME nicht ermitteln".to_string())?;
        PathBuf::from(home).join(".local/share/com.dpolg.booking")
    };

    // Invoices-Unterordner erstellen
    let invoices_dir = app_data_dir.join("invoices");
    std::fs::create_dir_all(&invoices_dir)
        .map_err(|e| format!("Fehler beim Erstellen des Invoices-Ordners: {}", e))?;

    // 3. PDF-Dateiname generieren
    let pdf_filename = format!("Rechnung_{}.pdf", booking.reservierungsnummer);
    let pdf_path = invoices_dir.join(&pdf_filename);

    // 4. PDF generieren
    let guest_name = format!("{} {}", booking.guest.vorname, booking.guest.nachname);

    // Adresse zusammenbauen
    let guest_address = booking.guest.strasse.as_deref();
    let guest_city = if let (Some(plz), Some(ort)) = (&booking.guest.plz, &booking.guest.ort) {
        Some(format!("{} {}", plz, ort))
    } else {
        None
    };
    let guest_city_str = guest_city.as_deref();

    pdf_generator::generate_invoice_pdf(
        booking.id,
        &booking.reservierungsnummer,
        &guest_name,
        guest_address,
        guest_city_str,
        Some("DEUTSCHLAND"), // Standardmäßig Deutschland
        &booking.room.name,
        &booking.checkin_date,
        &booking.checkout_date,
        booking.anzahl_gaeste,
        booking.grundpreis,
        booking.services_preis,
        booking.rabatt_preis,
        booking.gesamtpreis,
        &pdf_path,
    )?;

    println!("✅ PDF erstellt: {:?}", pdf_path);

    // 5. Email mit PDF versenden
    let email_result = email::send_invoice_email_with_pdf(booking_id, pdf_path.clone()).await?;

    println!("✅ Email gesendet: {}", email_result);

    Ok(format!("PDF erstellt und Email gesendet: {}", email_result))
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
            update_booking_dates_and_room_command,
            update_booking_statuses_command,
            delete_booking_command,
            cancel_booking_command,
            get_booking_by_id_command,
            get_booking_with_details_by_id_command,
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
            send_payment_reminder_email_command,
            send_cancellation_email_command,
            mark_booking_as_paid_command,
            // PDF Invoice Generation
            generate_invoice_pdf_command,
            generate_and_send_invoice_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}