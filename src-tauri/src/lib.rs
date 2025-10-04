// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod validation;
mod pricing;
mod email;
mod pdf_generator;
mod pdf_generator_html;
mod time_utils;
mod email_scheduler;

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
fn get_all_email_logs_command() -> Result<Vec<database::EmailLog>, String> {
    email::get_all_email_logs()
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
    database::update_booking_status(booking_id, new_status)
        .map_err(|e| format!("Fehler beim Ändern des Status: {}", e))
}

#[tauri::command]
fn update_booking_payment_command(
    booking_id: i64,
    bezahlt: bool,
) -> Result<database::Booking, String> {
    database::update_booking_payment(booking_id, bezahlt)
        .map_err(|e| format!("Fehler beim Ändern des Bezahlt-Status: {}", e))
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
        .plugin(tauri_plugin_dialog::init())
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
            get_all_email_logs_command,
            send_payment_reminder_email_command,
            send_cancellation_email_command,
            mark_booking_as_paid_command,
            update_booking_statuses_command,
            update_booking_status_command,
            update_booking_payment_command,
            // Email Scheduler
            email_scheduler::trigger_email_check,
            email_scheduler::get_scheduled_emails,
            // Company Settings
            get_company_settings_command,
            save_company_settings_command,
            upload_logo_command,
            // Payment Settings
            get_payment_settings_command,
            save_payment_settings_command,
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
        ])
        .setup(|_app| {
            // Starte Email-Scheduler im Hintergrund
            println!("🚀 Starte Email-Scheduler...");
            email_scheduler::start_email_scheduler();
            println!("✅ Email-Scheduler aktiv");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================================
// SERVICE TEMPLATES COMMANDS
// ============================================================================

#[tauri::command]
fn create_service_template_command(
    name: String,
    description: Option<String>,
    price: f64,
) -> Result<database::ServiceTemplate, String> {
    database::create_service_template(name, description, price)
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
) -> Result<database::ServiceTemplate, String> {
    database::update_service_template(id, name, description, price, is_active)
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
) -> Result<database::DiscountTemplate, String> {
    database::create_discount_template(name, description, discount_type, discount_value)
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
) -> Result<database::DiscountTemplate, String> {
    database::update_discount_template(id, name, description, discount_type, discount_value, is_active)
}

#[tauri::command]
fn delete_discount_template_command(id: i64) -> Result<(), String> {
    database::delete_discount_template(id)
}