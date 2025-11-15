// Modern PostgreSQL-based Tauri Application (2025 Best Practices)
// This is the new version that will replace lib.rs once fully tested

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod database_pg;

use config::AppConfig;
use database_pg::{
    DbPool,
    RoomRepository,
    GuestRepository,
    BookingRepository,
    AdditionalServiceRepository,
    DiscountRepository,
    EmailLogRepository,
    ReminderRepository,
    AccompanyingGuestRepository,
    ServiceTemplateRepository,
    DiscountTemplateRepository,
    PaymentRecipientRepository,
    CompanySettingsRepository,
    PricingSettingsRepository,
    EmailConfigRepository,
    EmailTemplateRepository,
    NotificationSettingsRepository,
    PaymentSettingsRepository,
};
use tauri::State;

// ============================================================================
// ROOM MANAGEMENT COMMANDS (PostgreSQL Version)
// ============================================================================

#[tauri::command]
async fn get_all_rooms_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::Room>, String> {
    println!("get_all_rooms_pg called");

    match RoomRepository::get_all(&pool).await {
        Ok(rooms) => {
            println!("Successfully got {} rooms from PostgreSQL", rooms.len());
            Ok(rooms)
        }
        Err(e) => {
            eprintln!("Error getting rooms: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_room_by_id_pg(
    pool: State<'_, DbPool>,
    id: i32,
) -> Result<database_pg::Room, String> {
    println!("get_room_by_id_pg called with id: {}", id);

    match RoomRepository::get_by_id(&pool, id).await {
        Ok(room) => {
            println!("Successfully got room: {}", room.name);
            Ok(room)
        }
        Err(e) => {
            eprintln!("Error getting room: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn create_room_pg(
    pool: State<'_, DbPool>,
    name: String,
    gebaeude_typ: String,
    capacity: i32,
    price_member: f64,
    price_non_member: f64,
    ort: String,
    nebensaison_preis: Option<f64>,
    hauptsaison_preis: Option<f64>,
    endreinigung: Option<f64>,
    schluesselcode: Option<String>,
    street_address: Option<String>,
    postal_code: Option<String>,
    city: Option<String>,
    notizen: Option<String>,
) -> Result<database_pg::Room, String> {
    println!("create_room_pg called: {}", name);

    match RoomRepository::create(
        &pool,
        name,
        gebaeude_typ,
        capacity,
        price_member,
        price_non_member,
        ort,
        nebensaison_preis,
        hauptsaison_preis,
        endreinigung,
        schluesselcode,
        street_address,
        postal_code,
        city,
        notizen,
    ).await {
        Ok(room) => {
            println!("Successfully created room: {}", room.name);
            Ok(room)
        }
        Err(e) => {
            eprintln!("Error creating room: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn update_room_pg(
    pool: State<'_, DbPool>,
    id: i32,
    name: String,
    gebaeude_typ: String,
    capacity: i32,
    price_member: f64,
    price_non_member: f64,
    ort: String,
    nebensaison_preis: Option<f64>,
    hauptsaison_preis: Option<f64>,
    endreinigung: Option<f64>,
    schluesselcode: Option<String>,
    street_address: Option<String>,
    postal_code: Option<String>,
    city: Option<String>,
    notizen: Option<String>,
) -> Result<database_pg::Room, String> {
    println!("update_room_pg called: id={}, name={}", id, name);

    match RoomRepository::update(
        &pool,
        id,
        name,
        gebaeude_typ,
        capacity,
        price_member,
        price_non_member,
        ort,
        nebensaison_preis,
        hauptsaison_preis,
        endreinigung,
        schluesselcode,
        street_address,
        postal_code,
        city,
        notizen,
    ).await {
        Ok(room) => {
            println!("Successfully updated room: {}", room.name);
            Ok(room)
        }
        Err(e) => {
            eprintln!("Error updating room: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn delete_room_pg(
    pool: State<'_, DbPool>,
    id: i32,
) -> Result<(), String> {
    println!("delete_room_pg called: id={}", id);

    match RoomRepository::delete(&pool, id).await {
        Ok(_) => {
            println!("Successfully deleted room with id: {}", id);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error deleting room: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn search_rooms_pg(
    pool: State<'_, DbPool>,
    query: String,
) -> Result<Vec<database_pg::Room>, String> {
    println!("search_rooms_pg called: query={}", query);

    match RoomRepository::search(&pool, &query).await {
        Ok(rooms) => {
            println!("Found {} rooms matching query", rooms.len());
            Ok(rooms)
        }
        Err(e) => {
            eprintln!("Error searching rooms: {}", e);
            Err(e.to_string())
        }
    }
}

// ============================================================================
// GUEST MANAGEMENT COMMANDS (PostgreSQL Version)
// ============================================================================

#[tauri::command]
async fn get_all_guests_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::Guest>, String> {
    println!("get_all_guests_pg called");

    match GuestRepository::get_all(&pool).await {
        Ok(guests) => {
            println!("Successfully got {} guests from PostgreSQL", guests.len());
            Ok(guests)
        }
        Err(e) => {
            eprintln!("Error getting guests: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_guest_by_id_pg(
    pool: State<'_, DbPool>,
    id: i32,
) -> Result<database_pg::Guest, String> {
    println!("get_guest_by_id_pg called with id: {}", id);

    match GuestRepository::get_by_id(&pool, id).await {
        Ok(guest) => {
            println!("Successfully got guest: {} {}", guest.vorname, guest.nachname);
            Ok(guest)
        }
        Err(e) => {
            eprintln!("Error getting guest: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn create_guest_pg(
    pool: State<'_, DbPool>,
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
    automail: Option<bool>,
    automail_sprache: Option<String>,
) -> Result<database_pg::Guest, String> {
    println!("create_guest_pg called: {} {}", vorname, nachname);

    match GuestRepository::create(
        &pool,
        vorname, nachname, email, telefon, dpolg_mitglied,
        strasse, plz, ort, mitgliedsnummer, notizen, beruf,
        bundesland, dienststelle, anrede, geschlecht, land,
        telefon_geschaeftlich, telefon_privat, telefon_mobil,
        fax, geburtsdatum, geburtsort, sprache, nationalitaet,
        identifikationsnummer, debitorenkonto, kennzeichen,
        rechnungs_email, marketing_einwilligung, leitweg_id,
        kostenstelle, tags, automail, automail_sprache,
    ).await {
        Ok(guest) => {
            println!("Successfully created guest: {} {}", guest.vorname, guest.nachname);
            Ok(guest)
        }
        Err(e) => {
            eprintln!("Error creating guest: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn update_guest_pg(
    pool: State<'_, DbPool>,
    id: i32,
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
    automail: Option<bool>,
    automail_sprache: Option<String>,
) -> Result<database_pg::Guest, String> {
    println!("update_guest_pg called: id={}", id);

    match GuestRepository::update(
        &pool,
        id, vorname, nachname, email, telefon, dpolg_mitglied,
        strasse, plz, ort, mitgliedsnummer, notizen, beruf,
        bundesland, dienststelle, anrede, geschlecht, land,
        telefon_geschaeftlich, telefon_privat, telefon_mobil,
        fax, geburtsdatum, geburtsort, sprache, nationalitaet,
        identifikationsnummer, debitorenkonto, kennzeichen,
        rechnungs_email, marketing_einwilligung, leitweg_id,
        kostenstelle, tags, automail, automail_sprache,
    ).await {
        Ok(guest) => {
            println!("Successfully updated guest: {} {}", guest.vorname, guest.nachname);
            Ok(guest)
        }
        Err(e) => {
            eprintln!("Error updating guest: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn delete_guest_pg(
    pool: State<'_, DbPool>,
    id: i32,
) -> Result<(), String> {
    println!("delete_guest_pg called: id={}", id);

    match GuestRepository::delete(&pool, id).await {
        Ok(_) => {
            println!("Successfully deleted guest with id: {}", id);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error deleting guest: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn search_guests_pg(
    pool: State<'_, DbPool>,
    query: String,
) -> Result<Vec<database_pg::Guest>, String> {
    println!("search_guests_pg called: query={}", query);

    match GuestRepository::search(&pool, query).await {
        Ok(guests) => {
            println!("Found {} guests matching query", guests.len());
            Ok(guests)
        }
        Err(e) => {
            eprintln!("Error searching guests: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_guests_by_membership_pg(
    pool: State<'_, DbPool>,
    is_member: bool,
) -> Result<Vec<database_pg::Guest>, String> {
    println!("get_guests_by_membership_pg called: is_member={}", is_member);

    match GuestRepository::get_by_membership(&pool, is_member).await {
        Ok(guests) => {
            println!("Found {} guests with membership={}", guests.len(), is_member);
            Ok(guests)
        }
        Err(e) => {
            eprintln!("Error getting guests by membership: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_guest_count_pg(pool: State<'_, DbPool>) -> Result<i64, String> {
    println!("get_guest_count_pg called");

    match GuestRepository::count(&pool).await {
        Ok(count) => {
            println!("Total guests: {}", count);
            Ok(count)
        }
        Err(e) => {
            eprintln!("Error counting guests: {}", e);
            Err(e.to_string())
        }
    }
}

// ============================================================================
// APPLICATION SETUP & RUN
// ============================================================================

pub fn run_pg() {
    // Load configuration from environment
    let config = AppConfig::from_env();

    println!("🚀 Starting DPolG Booking System (PostgreSQL Edition)");
    println!("   Environment: {:?}", config.environment);
    println!("   Database: {}", config.database_url());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Create PostgreSQL connection pool in async context
            tauri::async_runtime::block_on(async {
                match database_pg::init_database().await {
                    Ok(pool) => {
                        println!("✅ PostgreSQL connection pool initialized");

                        // Store pool in app state (accessible by all commands)
                        app.manage(pool);

                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to initialize database: {}", e);
                        Err(Box::new(e) as Box<dyn std::error::Error>)
                    }
                }
            })
        })
        .invoke_handler(tauri::generate_handler![
            // Room Management (PostgreSQL)
            get_all_rooms_pg,
            get_room_by_id_pg,
            create_room_pg,
            update_room_pg,
            delete_room_pg,
            search_rooms_pg,

            // Guest Management (PostgreSQL)
            get_all_guests_pg,
            get_guest_by_id_pg,
            create_guest_pg,
            update_guest_pg,
            delete_guest_pg,
            search_guests_pg,
            get_guests_by_membership_pg,
            get_guest_count_pg,

            // Additional Services
            get_all_additional_services_pg,
            get_additional_service_by_id_pg,
            get_additional_services_by_booking_pg,
            create_additional_service_pg,
            update_additional_service_pg,
            delete_additional_service_pg,
            calculate_additional_services_total_pg,

            // Discounts
            get_all_discounts_pg,
            get_discount_by_id_pg,
            get_discounts_by_booking_pg,
            create_discount_pg,
            update_discount_pg,
            delete_discount_pg,
            calculate_discounts_total_pg,

            // Email Logs
            get_all_email_logs_pg,
            get_email_log_by_id_pg,
            get_email_logs_by_booking_pg,
            get_email_logs_by_guest_pg,
            get_email_logs_by_status_pg,
            get_failed_email_logs_pg,
            create_email_log_pg,
            update_email_log_status_pg,
            delete_email_log_pg,

            // Reminders
            get_all_reminders_pg,
            get_reminder_by_id_pg,
            get_reminders_by_booking_pg,
            get_active_reminders_pg,
            create_reminder_pg,
            update_reminder_pg,
            complete_reminder_pg,
            snooze_reminder_pg,
            delete_reminder_pg,

            // Accompanying Guests
            get_all_accompanying_guests_pg,
            get_accompanying_guest_by_id_pg,
            get_accompanying_guests_by_booking_pg,
            create_accompanying_guest_pg,
            update_accompanying_guest_pg,
            delete_accompanying_guest_pg,

            // Service Templates
            get_all_service_templates_pg,
            get_service_template_by_id_pg,
            get_active_service_templates_pg,
            create_service_template_pg,
            update_service_template_pg,
            toggle_service_template_active_pg,
            delete_service_template_pg,

            // Discount Templates
            get_all_discount_templates_pg,
            get_discount_template_by_id_pg,
            get_active_discount_templates_pg,
            create_discount_template_pg,
            update_discount_template_pg,
            toggle_discount_template_active_pg,
            delete_discount_template_pg,

            // Payment Recipients
            get_all_payment_recipients_pg,
            get_payment_recipient_by_id_pg,
            get_active_payment_recipients_pg,
            create_payment_recipient_pg,
            update_payment_recipient_pg,
            toggle_payment_recipient_active_pg,
            delete_payment_recipient_pg,

            // Email Templates
            get_all_email_templates_pg,
            get_email_template_by_id_pg,
            get_email_template_by_name_pg,
            get_active_email_templates_pg,
            create_email_template_pg,
            update_email_template_pg,
            toggle_email_template_active_pg,
            delete_email_template_pg,

            // Settings - Company
            get_company_settings_pg,
            update_company_settings_pg,

            // Settings - Pricing
            get_pricing_settings_pg,
            update_pricing_settings_pg,

            // Settings - Email Config
            get_email_config_pg,
            update_email_config_pg,

            // Settings - Notifications
            get_notification_settings_pg,
            update_notification_settings_pg,

            // Settings - Payment
            get_payment_settings_pg,
            update_payment_settings_pg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================================
// ADDITIONAL SERVICES COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_additional_services_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::AdditionalService>, String> {
    AdditionalServiceRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_additional_service_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::AdditionalService, String> {
    AdditionalServiceRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_additional_services_by_booking_pg(pool: State<'_, DbPool>, booking_id: i32) -> Result<Vec<database_pg::AdditionalService>, String> {
    AdditionalServiceRepository::get_by_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_additional_service_pg(
    pool: State<'_, DbPool>,
    booking_id: i32,
    service_name: String,
    service_price: f64,
    template_id: Option<i32>,
    price_type: String,
    original_value: f64,
    applies_to: String,
) -> Result<database_pg::AdditionalService, String> {
    AdditionalServiceRepository::create(&pool, booking_id, service_name, service_price, template_id, price_type, original_value, applies_to)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_additional_service_pg(
    pool: State<'_, DbPool>,
    id: i32,
    service_name: String,
    service_price: f64,
    template_id: Option<i32>,
    price_type: String,
    original_value: f64,
    applies_to: String,
) -> Result<database_pg::AdditionalService, String> {
    AdditionalServiceRepository::update(&pool, id, service_name, service_price, template_id, price_type, original_value, applies_to)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_additional_service_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    AdditionalServiceRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn calculate_additional_services_total_pg(pool: State<'_, DbPool>, booking_id: i32) -> Result<f64, String> {
    AdditionalServiceRepository::calculate_total_for_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

// ============================================================================
// DISCOUNTS COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_discounts_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::Discount>, String> {
    DiscountRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_discount_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::Discount, String> {
    DiscountRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_discounts_by_booking_pg(pool: State<'_, DbPool>, booking_id: i32) -> Result<Vec<database_pg::Discount>, String> {
    DiscountRepository::get_by_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_discount_pg(
    pool: State<'_, DbPool>,
    booking_id: i32,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
) -> Result<database_pg::Discount, String> {
    DiscountRepository::create(&pool, booking_id, discount_name, discount_type, discount_value)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_discount_pg(
    pool: State<'_, DbPool>,
    id: i32,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
) -> Result<database_pg::Discount, String> {
    DiscountRepository::update(&pool, id, discount_name, discount_type, discount_value)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_discount_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    DiscountRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn calculate_discounts_total_pg(pool: State<'_, DbPool>, booking_id: i32) -> Result<f64, String> {
    DiscountRepository::calculate_total_for_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

// ============================================================================
// EMAIL LOG COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_email_logs_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::EmailLog>, String> {
    EmailLogRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_email_log_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::EmailLog, String> {
    EmailLogRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_email_logs_by_booking_pg(pool: State<'_, DbPool>, booking_id: i32) -> Result<Vec<database_pg::EmailLog>, String> {
    EmailLogRepository::get_by_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_email_logs_by_guest_pg(pool: State<'_, DbPool>, guest_id: i32) -> Result<Vec<database_pg::EmailLog>, String> {
    EmailLogRepository::get_by_guest(&pool, guest_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_email_logs_by_status_pg(pool: State<'_, DbPool>, status: String) -> Result<Vec<database_pg::EmailLog>, String> {
    EmailLogRepository::get_by_status(&pool, status).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_failed_email_logs_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::EmailLog>, String> {
    EmailLogRepository::get_failed(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_email_log_pg(
    pool: State<'_, DbPool>,
    booking_id: Option<i32>,
    guest_id: i32,
    template_name: String,
    recipient_email: String,
    subject: String,
    status: String,
    error_message: Option<String>,
) -> Result<database_pg::EmailLog, String> {
    EmailLogRepository::create(&pool, booking_id, guest_id, template_name, recipient_email, subject, status, error_message)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_email_log_status_pg(
    pool: State<'_, DbPool>,
    id: i32,
    status: String,
    error_message: Option<String>,
) -> Result<database_pg::EmailLog, String> {
    EmailLogRepository::update_status(&pool, id, status, error_message).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_email_log_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    EmailLogRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}


// ============================================================================
// REMINDERS COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_reminders_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::Reminder>, String> {
    ReminderRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_reminder_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::Reminder, String> {
    ReminderRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_reminders_by_booking_pg(pool: State<'_, DbPool>, booking_id: i32) -> Result<Vec<database_pg::Reminder>, String> {
    ReminderRepository::get_by_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_active_reminders_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::Reminder>, String> {
    ReminderRepository::get_active(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_reminder_pg(
    pool: State<'_, DbPool>,
    booking_id: Option<i32>,
    reminder_type: String,
    title: String,
    description: Option<String>,
    due_date: String,
    priority: String,
) -> Result<database_pg::Reminder, String> {
    ReminderRepository::create(&pool, booking_id, reminder_type, title, description, due_date, priority)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_reminder_pg(
    pool: State<'_, DbPool>,
    id: i32,
    title: String,
    description: Option<String>,
    due_date: String,
    priority: String,
) -> Result<database_pg::Reminder, String> {
    ReminderRepository::update(&pool, id, title, description, due_date, priority)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn complete_reminder_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::Reminder, String> {
    ReminderRepository::complete(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn snooze_reminder_pg(pool: State<'_, DbPool>, id: i32, snoozed_until: String) -> Result<database_pg::Reminder, String> {
    ReminderRepository::snooze(&pool, id, snoozed_until).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_reminder_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    ReminderRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

// ============================================================================
// ACCOMPANYING GUESTS COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_accompanying_guests_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::AccompanyingGuest>, String> {
    AccompanyingGuestRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_accompanying_guest_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::AccompanyingGuest, String> {
    AccompanyingGuestRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_accompanying_guests_by_booking_pg(pool: State<'_, DbPool>, booking_id: i32) -> Result<Vec<database_pg::AccompanyingGuest>, String> {
    AccompanyingGuestRepository::get_by_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_accompanying_guest_pg(
    pool: State<'_, DbPool>,
    booking_id: i32,
    first_name: String,
    last_name: String,
    birth_date: Option<String>,
) -> Result<database_pg::AccompanyingGuest, String> {
    AccompanyingGuestRepository::create(&pool, booking_id, first_name, last_name, birth_date)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_accompanying_guest_pg(
    pool: State<'_, DbPool>,
    id: i32,
    first_name: String,
    last_name: String,
    birth_date: Option<String>,
) -> Result<database_pg::AccompanyingGuest, String> {
    AccompanyingGuestRepository::update(&pool, id, first_name, last_name, birth_date)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_accompanying_guest_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    AccompanyingGuestRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

// ============================================================================
// SERVICE TEMPLATES COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_service_templates_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::ServiceTemplate>, String> {
    ServiceTemplateRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_service_template_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::ServiceTemplate, String> {
    ServiceTemplateRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_active_service_templates_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::ServiceTemplate>, String> {
    ServiceTemplateRepository::get_active(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_service_template_pg(
    pool: State<'_, DbPool>,
    service_name: String,
    price_type: String,
    original_value: f64,
    applies_to: String,
) -> Result<database_pg::ServiceTemplate, String> {
    ServiceTemplateRepository::create(&pool, service_name, price_type, original_value, applies_to)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_service_template_pg(
    pool: State<'_, DbPool>,
    id: i32,
    service_name: String,
    price_type: String,
    original_value: f64,
    applies_to: String,
    is_active: bool,
) -> Result<database_pg::ServiceTemplate, String> {
    ServiceTemplateRepository::update(&pool, id, service_name, price_type, original_value, applies_to, is_active)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_service_template_active_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::ServiceTemplate, String> {
    ServiceTemplateRepository::toggle_active(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_service_template_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    ServiceTemplateRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

// ============================================================================
// DISCOUNT TEMPLATES COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_discount_templates_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::DiscountTemplate>, String> {
    DiscountTemplateRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_discount_template_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::DiscountTemplate, String> {
    DiscountTemplateRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_active_discount_templates_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::DiscountTemplate>, String> {
    DiscountTemplateRepository::get_active(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_discount_template_pg(
    pool: State<'_, DbPool>,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
) -> Result<database_pg::DiscountTemplate, String> {
    DiscountTemplateRepository::create(&pool, discount_name, discount_type, discount_value)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_discount_template_pg(
    pool: State<'_, DbPool>,
    id: i32,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
    is_active: bool,
) -> Result<database_pg::DiscountTemplate, String> {
    DiscountTemplateRepository::update(&pool, id, discount_name, discount_type, discount_value, is_active)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_discount_template_active_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::DiscountTemplate, String> {
    DiscountTemplateRepository::toggle_active(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_discount_template_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    DiscountTemplateRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}


// ============================================================================
// PAYMENT RECIPIENTS COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_payment_recipients_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::PaymentRecipient>, String> {
    PaymentRecipientRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_payment_recipient_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::PaymentRecipient, String> {
    PaymentRecipientRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_active_payment_recipients_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::PaymentRecipient>, String> {
    PaymentRecipientRepository::get_active(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_payment_recipient_pg(
    pool: State<'_, DbPool>,
    name: String,
    bank_name: String,
    iban: String,
    bic: Option<String>,
) -> Result<database_pg::PaymentRecipient, String> {
    PaymentRecipientRepository::create(&pool, name, bank_name, iban, bic)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_payment_recipient_pg(
    pool: State<'_, DbPool>,
    id: i32,
    name: String,
    bank_name: String,
    iban: String,
    bic: Option<String>,
    is_active: bool,
) -> Result<database_pg::PaymentRecipient, String> {
    PaymentRecipientRepository::update(&pool, id, name, bank_name, iban, bic, is_active)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_payment_recipient_active_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::PaymentRecipient, String> {
    PaymentRecipientRepository::toggle_active(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_payment_recipient_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    PaymentRecipientRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

// ============================================================================
// EMAIL TEMPLATES COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_email_templates_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::EmailTemplate>, String> {
    EmailTemplateRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_email_template_by_id_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::EmailTemplate, String> {
    EmailTemplateRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_email_template_by_name_pg(pool: State<'_, DbPool>, template_name: String) -> Result<database_pg::EmailTemplate, String> {
    EmailTemplateRepository::get_by_name(&pool, template_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_active_email_templates_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::EmailTemplate>, String> {
    EmailTemplateRepository::get_active(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_email_template_pg(
    pool: State<'_, DbPool>,
    template_name: String,
    subject: String,
    body: String,
) -> Result<database_pg::EmailTemplate, String> {
    EmailTemplateRepository::create(&pool, template_name, subject, body)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_email_template_pg(
    pool: State<'_, DbPool>,
    id: i32,
    template_name: String,
    subject: String,
    body: String,
    is_active: bool,
) -> Result<database_pg::EmailTemplate, String> {
    EmailTemplateRepository::update(&pool, id, template_name, subject, body, is_active)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_email_template_active_pg(pool: State<'_, DbPool>, id: i32) -> Result<database_pg::EmailTemplate, String> {
    EmailTemplateRepository::toggle_active(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_email_template_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    EmailTemplateRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

// ============================================================================
// SETTINGS COMMANDS (Singletons)
// ============================================================================

#[tauri::command]
async fn get_company_settings_pg(pool: State<'_, DbPool>) -> Result<database_pg::CompanySettings, String> {
    CompanySettingsRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_company_settings_pg(
    pool: State<'_, DbPool>,
    company_name: String,
    address: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    tax_id: Option<String>,
    logo_path: Option<String>,
) -> Result<database_pg::CompanySettings, String> {
    CompanySettingsRepository::update(&pool, company_name, address, phone, email, tax_id, logo_path)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_pricing_settings_pg(pool: State<'_, DbPool>) -> Result<database_pg::PricingSettings, String> {
    PricingSettingsRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_pricing_settings_pg(
    pool: State<'_, DbPool>,
    standard_discount_percent: f64,
    family_discount_percent: f64,
    member_discount_name: String,
    non_member_discount_name: String,
    vat_rate: f64,
) -> Result<database_pg::PricingSettings, String> {
    PricingSettingsRepository::update(&pool, standard_discount_percent, family_discount_percent, member_discount_name, non_member_discount_name, vat_rate)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_email_config_pg(pool: State<'_, DbPool>) -> Result<database_pg::EmailConfig, String> {
    EmailConfigRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_email_config_pg(
    pool: State<'_, DbPool>,
    provider: String,
    smtp_host: String,
    smtp_port: i32,
    smtp_user: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
    use_tls: bool,
) -> Result<database_pg::EmailConfig, String> {
    EmailConfigRepository::update(&pool, provider, smtp_host, smtp_port, smtp_user, smtp_password, from_email, from_name, use_tls)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_notification_settings_pg(pool: State<'_, DbPool>) -> Result<database_pg::NotificationSettings, String> {
    NotificationSettingsRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_notification_settings_pg(
    pool: State<'_, DbPool>,
    enable_email_notifications: bool,
    enable_reminder_notifications: bool,
    reminder_days_before: i32,
) -> Result<database_pg::NotificationSettings, String> {
    NotificationSettingsRepository::update(&pool, enable_email_notifications, enable_reminder_notifications, reminder_days_before)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_payment_settings_pg(pool: State<'_, DbPool>) -> Result<database_pg::PaymentSettings, String> {
    PaymentSettingsRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_payment_settings_pg(
    pool: State<'_, DbPool>,
    default_payment_method: String,
    require_payment_confirmation: bool,
    payment_deadline_days: i32,
) -> Result<database_pg::PaymentSettings, String> {
    PaymentSettingsRepository::update(&pool, default_payment_method, require_payment_confirmation, payment_deadline_days)
        .await.map_err(|e| e.to_string())
}

