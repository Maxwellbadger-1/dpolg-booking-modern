// Modern PostgreSQL-based Tauri Application (2025 Best Practices)
// This is the new version that will replace lib.rs once fully tested

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::config::AppConfig;
use crate::database_pg::{
    self,
    pool::DbPool,
    repositories::{
        room_repository::RoomRepository,
        guest_repository::GuestRepository,
        booking_repository::BookingRepository,
        additional_service_repository::AdditionalServiceRepository,
        discount_repository::DiscountRepository,
        email_log_repository::EmailLogRepository,
        reminder_repository::ReminderRepository,
        accompanying_guest_repository::AccompanyingGuestRepository,
        service_template_repository::ServiceTemplateRepository,
        discount_template_repository::DiscountTemplateRepository,
        payment_recipient_repository::PaymentRecipientRepository,
        company_settings_repository::CompanySettingsRepository,
        pricing_settings_repository::PricingSettingsRepository,
        email_config_repository::EmailConfigRepository,
        email_template_repository::EmailTemplateRepository,
        notification_settings_repository::NotificationSettingsRepository,
        payment_settings_repository::PaymentSettingsRepository,
    },
};
use crate::turso_sync;
use tauri::{Manager, State};
use lettre::{
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use serde::{Serialize, Deserialize};
use qrcode::QrCode;
use image::Luma;
use base64::{Engine as _, engine::general_purpose};
use chrono::NaiveDate;

// ============================================================================
// PERFORMANCE MONITORING (Connection Pool Stats)
// ============================================================================

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PoolStats {
    size: usize,
    available: usize,
    waiting: usize,
    max_size: usize,
    utilization_percent: f64,
}

#[tauri::command]
async fn get_pool_stats(pool: State<'_, DbPool>) -> Result<PoolStats, String> {
    let status = pool.status();
    let max_size = 20; // Default pool size (configured in pool.rs)
    let utilization = (status.size as f64 / max_size as f64) * 100.0;

    Ok(PoolStats {
        size: status.size as usize,
        available: status.available as usize,
        waiting: status.waiting as usize,
        max_size,
        utilization_percent: utilization,
    })
}

#[tauri::command]
async fn check_db_connection(pool: State<'_, DbPool>) -> Result<bool, String> {
    use tokio::time::{timeout, Duration};

    // Fast timeout: 3 seconds max
    let result = timeout(Duration::from_secs(3), async {
        match pool.get().await {
            Ok(client) => {
                // Try a simple query to verify connection
                match client.query("SELECT 1", &[]).await {
                    Ok(_) => Ok(true),
                    Err(e) => {
                        eprintln!("❌ Database query failed: {}", e);
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Database connection failed: {}", e);
                Ok(false)
            }
        }
    }).await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            eprintln!("⏱️ Database connection timeout (3s)");
            Ok(false)
        }
    }
}

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
// BOOKING COMMANDS (PostgreSQL) - WITH OPTIMISTIC LOCKING
// ============================================================================

/// Update booking with Optimistic Locking support (2025 Best Practice)
///
/// If `expected_updated_at` is provided, the update will only succeed if the record
/// hasn't been modified by another user. Otherwise, returns a ConflictError.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn update_booking_pg(
    pool: State<'_, DbPool>,
    id: i32,
    room_id: i32,
    guest_id: i32,
    reservierungsnummer: String,
    checkin_date: String,
    checkout_date: String,
    anzahl_gaeste: i32,
    status: String,
    gesamtpreis: f64,
    bemerkungen: Option<String>,
    anzahl_begleitpersonen: Option<i32>,
    grundpreis: Option<f64>,
    services_preis: Option<f64>,
    rabatt_preis: Option<f64>,
    anzahl_naechte: Option<i32>,
    bezahlt: Option<bool>,
    bezahlt_am: Option<String>,
    zahlungsmethode: Option<String>,
    mahnung_gesendet_am: Option<String>,
    rechnung_versendet_am: Option<String>,
    rechnung_versendet_an: Option<String>,
    ist_stiftungsfall: Option<bool>,
    payment_recipient_id: Option<i32>,
    putzplan_checkout_date: Option<String>,
    ist_dpolg_mitglied: Option<bool>,
    expected_updated_at: Option<String>,  // ← OPTIMISTIC LOCKING!
) -> Result<database_pg::Booking, String> {
    println!("update_booking_pg called: id={}, optimistic_locking={}",
             id, expected_updated_at.is_some());

    // 1. Update booking in PostgreSQL (trigger will auto-update cleaning tasks)
    let booking = match BookingRepository::update(
        &pool,
        id,
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
        bezahlt,
        bezahlt_am,
        zahlungsmethode,
        mahnung_gesendet_am,
        rechnung_versendet_am,
        rechnung_versendet_an,
        ist_stiftungsfall,
        payment_recipient_id,
        putzplan_checkout_date,
        ist_dpolg_mitglied,
        expected_updated_at,  // Pass through to repository
    ).await {
        Ok(booking) => {
            println!("Successfully updated booking: {}", booking.reservierungsnummer);
            booking
        }
        Err(e) => {
            // Check if it's a conflict error
            let error_msg = e.to_string();
            if error_msg.contains("was modified by another user") {
                eprintln!("⚠️ CONFLICT: Booking {} was modified concurrently", id);
            } else {
                eprintln!("Error updating booking: {}", e);
            }
            return Err(error_msg);
        }
    };

    // 2. Sync updated cleaning tasks to Turso (mobile app)
    println!("🔄 Auto-syncing updated cleaning tasks to Turso for booking {}", id);
    if let Err(e) = turso_sync::sync_booking_tasks_to_turso(&pool, id).await {
        eprintln!("⚠️ Failed to sync cleaning tasks to Turso: {}", e);
        // Don't fail the booking update - mobile sync can be retried later
    }

    Ok(booking)
}

// ============================================================================
// EMAIL SCHEDULER (Background Task)
// ============================================================================

/// Start the email scheduler background task
fn start_email_scheduler(pool: DbPool) {
    use std::time::Duration;

    tauri::async_runtime::spawn(async move {
        println!("📧 Email Scheduler started");

        // Wait for 60 seconds before first check (give app time to fully start)
        tokio::time::sleep(Duration::from_secs(60)).await;

        loop {
            // Get scheduler interval from notification_settings
            let interval_hours = match get_scheduler_interval(&pool).await {
                Ok(hours) => hours,
                Err(e) => {
                    eprintln!("⚠️ Failed to get scheduler interval: {}, using default 1 hour", e);
                    1
                }
            };

            println!("📧 [Scheduler] Checking for pending emails... (interval: {} hours)", interval_hours);

            // Run email check
            match run_email_check(&pool).await {
                Ok(message) => println!("📧 [Scheduler] {}", message),
                Err(e) => eprintln!("❌ [Scheduler] Email check failed: {}", e),
            }

            // Sleep for configured interval
            let sleep_duration = Duration::from_secs(interval_hours * 3600);
            println!("📧 [Scheduler] Next check in {} hours", interval_hours);
            tokio::time::sleep(sleep_duration).await;
        }
    });
}

/// Get scheduler interval from notification_settings
async fn get_scheduler_interval(pool: &DbPool) -> Result<u64, String> {
    let settings = NotificationSettingsRepository::get(pool)
        .await
        .map_err(|e| format!("Failed to get notification settings: {}", e))?;

    Ok(settings.scheduler_interval_hours.unwrap_or(1) as u64)
}

/// Run email check (same logic as trigger_email_check command)
async fn run_email_check(pool: &DbPool) -> Result<String, String> {
    use crate::database_pg::repositories::ScheduledEmailRepository;
    use crate::database_pg::repositories::EmailLogRepository;

    let pending = ScheduledEmailRepository::get_pending(pool)
        .await
        .map_err(|e| format!("Failed to get pending emails: {}", e))?;

    let mut sent_count = 0;
    let mut failed_count = 0;

    for scheduled_email in pending {
        // Check if email is due
        let now = chrono::Local::now().naive_local();
        let scheduled_for = chrono::NaiveDateTime::parse_from_str(&scheduled_email.scheduled_for, "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|_| now);

        if scheduled_for > now {
            continue;
        }

        // Send email
        let result = if let Some(booking_id) = scheduled_email.booking_id {
            match process_scheduled_booking_email(pool, booking_id, &scheduled_email.template_name).await {
                Ok(_) => {
                    sent_count += 1;
                    let _ = ScheduledEmailRepository::update_status(pool, scheduled_email.id, "sent").await;
                    let _ = EmailLogRepository::create(
                        pool,
                        Some(booking_id),
                        scheduled_email.guest_id.unwrap_or(0),
                        scheduled_email.template_name.clone(),
                        scheduled_email.recipient_email.clone(),
                        scheduled_email.subject.clone(),
                        "gesendet".to_string(),
                        None,
                    ).await;
                    Ok(())
                }
                Err(e) => {
                    failed_count += 1;
                    let _ = ScheduledEmailRepository::update_status_with_error(pool, scheduled_email.id, "failed", &e).await;
                    let _ = EmailLogRepository::create(
                        pool,
                        Some(booking_id),
                        scheduled_email.guest_id.unwrap_or(0),
                        scheduled_email.template_name.clone(),
                        scheduled_email.recipient_email.clone(),
                        scheduled_email.subject.clone(),
                        "fehler".to_string(),
                        Some(e.clone()),
                    ).await;
                    Err(e)
                }
            }
        } else {
            continue;
        };

        if result.is_ok() {
            println!("✅ [Scheduler] Email {} sent successfully", scheduled_email.id);
        }
    }

    Ok(format!("{} versendet, {} fehlgeschlagen", sent_count, failed_count))
}

// ============================================================================
// APPLICATION SETUP & RUN
// ============================================================================

pub fn run_pg() {
    // Load configuration from environment
    let config = AppConfig::from_env();
    let db_url_for_listener = config.database_url(); // Clone for LISTEN/NOTIFY listener

    println!("🚀 Starting DPolG Booking System (PostgreSQL Edition)");
    println!("   Environment: {:?}", config.environment);
    println!("   Database: {}", config.database_url());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(move |app| {
            // Create PostgreSQL connection pool in async context
            tauri::async_runtime::block_on(async {
                match database_pg::init_database().await {
                    Ok(pool) => {
                        println!("✅ PostgreSQL connection pool initialized");

                        // Store pool in app state (accessible by all commands)
                        app.manage(pool.clone());

                        // Run Real-Time NOTIFY triggers migration (idempotent)
                        println!("🔧 Running Real-Time NOTIFY triggers migration...");
                        let migration_sql = include_str!("../../migrations/007_realtime_notify_triggers.sql");
                        match pool.get().await {
                            Ok(client) => {
                                if let Err(e) = client.batch_execute(migration_sql).await {
                                    eprintln!("⚠️ Real-Time NOTIFY migration warning: {}", e);
                                    eprintln!("   (This is OK if triggers already exist)");
                                } else {
                                    println!("✅ Real-Time NOTIFY triggers ready");
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️ Could not run Real-Time migration: {}", e);
                            }
                        }

                        // Run Pricing Precision migration (f32 -> f64, idempotent)
                        println!("🔧 Running Pricing Precision migration...");
                        let pricing_migration_sql = include_str!("../../migrations/008_fix_pricing_precision.sql");
                        match pool.get().await {
                            Ok(client) => {
                                if let Err(e) = client.batch_execute(pricing_migration_sql).await {
                                    eprintln!("⚠️ Pricing Precision migration warning: {}", e);
                                    eprintln!("   (This is OK if columns already have correct type)");
                                } else {
                                    println!("✅ Pricing Precision migration complete");
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️ Could not run Pricing Precision migration: {}", e);
                            }
                        }

                        // Run Logo Data migration (store logo as Base64 in DB)
                        println!("🔧 Running Logo Data migration...");
                        let logo_migration_sql = include_str!("../../migrations/009_logo_data_in_db.sql");
                        match pool.get().await {
                            Ok(client) => {
                                if let Err(e) = client.batch_execute(logo_migration_sql).await {
                                    eprintln!("⚠️ Logo Data migration warning: {}", e);
                                } else {
                                    println!("✅ Logo Data migration complete");
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️ Could not run Logo Data migration: {}", e);
                            }
                        }

                        // Start PostgreSQL LISTEN/NOTIFY listener for real-time updates
                        let app_handle = app.handle().clone();
                        database_pg::listener::start_pg_listener(app_handle, db_url_for_listener.clone());
                        println!("✅ PostgreSQL LISTEN/NOTIFY listener started");

                        // Start email scheduler background task
                        start_email_scheduler(pool.clone());

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
            // Performance Monitoring
            get_pool_stats,
            check_db_connection,

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

            // Booking Management (PostgreSQL) - WITH OPTIMISTIC LOCKING
            get_all_bookings_pg,
            get_booking_with_details_by_id_pg,
            create_booking_pg,
            update_booking_pg,
            delete_booking_pg,
            update_booking_status_pg,
            update_booking_statuses_pg,
            update_booking_payment_pg,
            update_booking_dates_and_room_pg,

            // Cleaning Tasks / Putzplan (Legacy Mobile App Support)
            delete_booking_tasks,
            get_cleaning_stats_stub,
            sync_week_ahead,

            // Pricing & Validation
            calculate_full_booking_price_pg,
            calculate_batch_booking_prices_pg,
            check_room_availability_pg,

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

            // Guest Companions & Credit
            get_guest_companions_command,
            get_guest_credit_balance,
            get_guest_credit_transactions,
            add_guest_credit,
            run_guest_credit_migration,
            fix_guest_credit_table_structure,

            // Cleaning Tasks
            run_cleaning_tasks_migration,

            // Email Automation
            run_email_automation_migration,
            run_discount_calculated_amount_migration,
            backfill_scheduled_emails,

            // Real-Time Multi-User
            run_realtime_notify_migration,
            get_updates_since,

            // Template Linking (STUB)
            link_service_template_to_booking_command,
            link_discount_template_to_booking_command,

            // Missing STUB Commands (Bug #6-10)
            cancel_booking_command,
            validate_email_command,
            validate_date_range_command,
            calculate_nights_command,
            get_report_stats_command,
            get_room_occupancy_command,

            // Invoice & Credit Commands
            get_invoice_pdfs_for_booking_command,

            // Cleaning & Email Commands (STUB)
            cleanup_cleaning_tasks,
            send_confirmation_email_command,

            // All Missing Commands (STUB) - Found via Frontend Analysis
            add_guest_credit,
            create_backup_command,
            create_guest_companion_command,
            delete_backup_command,
            export_cleaning_timeline_pdf,
            generate_and_send_invoice_command,
            generate_invoice_pdf_command,
            get_backup_settings_command,
            get_booking_command,
            get_guest_credit_transactions,
            get_recent_email_logs_command,
            get_recent_transactions_command,
            get_reminder_settings_command,
            get_scheduled_emails,
            debug_scheduled_emails_pg,
            list_backups_command,
            mark_invoice_sent_command,
            migrate_to_price_list_2025_command,
            fix_booking_status_values_command,
            fix_deadlock_triggers_command,
            open_backup_folder_command,
            open_invoices_folder_command,
            open_pdf_file_command,
            open_putzplan_folder,
            restore_backup_command,
            save_backup_settings_command,
            save_reminder_settings_command,
            send_cancellation_email_command,
            send_invoice_email_command,
            send_reminder_email_command,
            send_test_email_command,
            sync_affected_dates,
            test_email_connection_command,
            trigger_email_check,
            undo_transaction_command,
            update_template_command,
            upload_logo_command,
            use_guest_credit_for_booking,
            get_booking_credit_usage,
            get_email_logs_command,
            clear_email_logs_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================================
// BOOKING READ COMMANDS (PostgreSQL)
// ============================================================================

#[tauri::command]
async fn get_all_bookings_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::Booking>, String> {
    println!("get_all_bookings_pg called");
    let mut bookings = BookingRepository::get_all(&pool).await.map_err(|e| {
        eprintln!("❌ Error loading bookings: {}", e);
        e.to_string()
    })?;

    // Load services and discounts for TapeChart emoji display
    if !bookings.is_empty() {
        let client = pool.get().await.map_err(|e| format!("Pool error: {}", e))?;
        let booking_ids: Vec<i64> = bookings.iter().map(|b| b.id as i64).collect();

        // Load all services with template emoji info
        let service_rows = client
            .query(
                "SELECT a.id, a.booking_id, COALESCE(st.name, a.service_name) as name,
                        st.emoji, st.color_hex, st.cleaning_plan_position
                 FROM additional_services a
                 LEFT JOIN service_templates st ON a.template_id = st.id
                 WHERE a.booking_id = ANY($1)",
                &[&booking_ids],
            )
            .await
            .unwrap_or_else(|e| {
                eprintln!("⚠️ Warning: Could not load services: {}", e);
                Vec::new()
            });

        // Load all discounts (discounts table has no template_id, so no JOIN needed)
        let discount_rows = client
            .query(
                "SELECT d.id, d.booking_id, d.discount_name as name
                 FROM discounts d
                 WHERE d.booking_id = ANY($1)",
                &[&booking_ids],
            )
            .await
            .unwrap_or_else(|e| {
                eprintln!("⚠️ Warning: Could not load discounts: {}", e);
                Vec::new()
            });

        // Group services by booking_id
        let mut services_map: std::collections::HashMap<i32, Vec<database_pg::ServiceEmoji>> = std::collections::HashMap::new();
        println!("📊 [BOOKINGS] Found {} service rows", service_rows.len());
        for row in service_rows {
            let booking_id: i64 = row.get("booking_id");
            let emoji: Option<String> = row.try_get("emoji").ok().flatten();
            let name: String = row.get("name");
            println!("   Service for booking {}: {} -> emoji: {:?}", booking_id, name, emoji);
            let service = database_pg::ServiceEmoji {
                id: row.get("id"),
                name,
                emoji,
                color_hex: row.try_get("color_hex").ok().flatten(),
                cleaning_plan_position: row.try_get("cleaning_plan_position").ok().flatten(),
            };
            services_map.entry(booking_id as i32).or_default().push(service);
        }

        // Group discounts by booking_id
        let mut discounts_map: std::collections::HashMap<i32, Vec<database_pg::DiscountEmoji>> = std::collections::HashMap::new();
        for row in discount_rows {
            let booking_id: i64 = row.get("booking_id");
            let discount = database_pg::DiscountEmoji {
                id: row.get("id"),
                name: row.get("name"),
                emoji: None, // discounts table has no emoji column
                color_hex: None, // discounts table has no color_hex column
            };
            discounts_map.entry(booking_id as i32).or_default().push(discount);
        }

        // Assign services and discounts to bookings
        for booking in &mut bookings {
            let services = services_map.remove(&booking.id);
            let discounts = discounts_map.remove(&booking.id);
            booking.services = if services.as_ref().map_or(true, |v| v.is_empty()) { None } else { services };
            booking.discounts = if discounts.as_ref().map_or(true, |v| v.is_empty()) { None } else { discounts };
        }
    }

    println!("✅ Successfully got {} bookings from PostgreSQL", bookings.len());
    Ok(bookings)
}

#[tauri::command]
async fn get_booking_with_details_by_id_pg(
    pool: State<'_, DbPool>,
    id: i32,
) -> Result<database_pg::BookingWithDetails, String> {
    println!("get_booking_with_details_by_id_pg called: id={}", id);
    BookingRepository::get_with_details(&pool, id).await.map_err(|e| {
        eprintln!("❌ Error loading booking with details: {}", e);
        e.to_string()
    })
}

// ============================================================================
// BOOKING WRITE COMMANDS (PostgreSQL)
// ============================================================================

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn create_booking_pg(
    pool: State<'_, DbPool>,
    room_id: i32,
    guest_id: i32,
    reservierungsnummer: String,
    checkin_date: String,
    checkout_date: String,
    anzahl_gaeste: i32,
    status: String,
    gesamtpreis: f64,
    bemerkungen: Option<String>,
    anzahl_begleitpersonen: Option<i32>,
    grundpreis: Option<f64>,
    services_preis: Option<f64>,
    rabatt_preis: Option<f64>,
    anzahl_naechte: Option<i32>,
    bezahlt: Option<bool>,
    bezahlt_am: Option<String>,
    zahlungsmethode: Option<String>,
    mahnung_gesendet_am: Option<String>,
    rechnung_versendet_am: Option<String>,
    rechnung_versendet_an: Option<String>,
    ist_stiftungsfall: Option<bool>,
    payment_recipient_id: Option<i32>,
    putzplan_checkout_date: Option<String>,
    ist_dpolg_mitglied: Option<bool>,
) -> Result<database_pg::Booking, String> {
    println!("create_booking_pg called with availability check");

    // 1. Create booking in PostgreSQL with atomic availability check (prevents double bookings)
    // Uses transaction with row-level locking to prevent race conditions
    let booking = BookingRepository::create_with_availability_check(
        &pool,
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
        bezahlt,
        bezahlt_am,
        zahlungsmethode,
        mahnung_gesendet_am,
        rechnung_versendet_am,
        rechnung_versendet_an,
        ist_stiftungsfall,
        payment_recipient_id,
        putzplan_checkout_date,
        ist_dpolg_mitglied,
    )
    .await
    .map_err(|e| {
        eprintln!("❌ Error creating booking: {}", e);
        // Special error prefix for double booking - frontend can detect this
        match e {
            database_pg::DbError::DoubleBookingError(msg) => format!("DOUBLE_BOOKING:{}", msg),
            _ => e.to_string()
        }
    })?;

    // 2. Sync cleaning tasks to Turso (mobile app)
    println!("🔄 Auto-syncing cleaning tasks to Turso for booking {}", booking.id);
    if let Err(e) = turso_sync::sync_booking_tasks_to_turso(&pool, booking.id).await {
        eprintln!("⚠️ Failed to sync cleaning tasks to Turso: {}", e);
        // Don't fail the booking creation - mobile sync can be retried later
    }

    Ok(booking)
}

#[tauri::command]
async fn delete_booking_pg(pool: State<'_, DbPool>, id: i32) -> Result<(), String> {
    use crate::database_pg::repositories::CleaningTaskRepository;

    println!("delete_booking_pg called: id={}", id);

    // 1. Delete cleaning tasks from Turso first (before PostgreSQL CASCADE deletes them)
    println!("🗑️ Deleting cleaning tasks from Turso for booking {}", id);
    if let Err(e) = turso_sync::delete_tasks_from_turso(id).await {
        eprintln!("⚠️ Failed to delete tasks from Turso: {}", e);
        // Continue anyway - PostgreSQL is source of truth
    }

    // 2. Delete cleaning tasks from PostgreSQL
    println!("🗑️ Deleting cleaning tasks from PostgreSQL for booking {}", id);
    if let Err(e) = CleaningTaskRepository::delete_for_booking(&pool, id).await {
        eprintln!("⚠️ Failed to delete tasks from PostgreSQL: {}", e);
        // Continue anyway - booking deletion has CASCADE
    }

    // 3. Delete booking (will CASCADE delete any remaining related data)
    BookingRepository::delete(&pool, id).await.map_err(|e| {
        eprintln!("❌ Error deleting booking: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn update_booking_status_pg(
    pool: State<'_, DbPool>,
    id: i32,
    status: String,
) -> Result<database_pg::Booking, String> {
    println!("update_booking_status_pg called: id={}, status={}", id, status);

    // Get current booking to preserve other fields
    let booking = BookingRepository::get_by_id(&pool, id).await.map_err(|e| {
        eprintln!("❌ Error getting booking: {}", e);
        e.to_string()
    })?;

    // Update with new status
    BookingRepository::update(
        &pool,
        id,
        booking.room_id,
        booking.guest_id,
        booking.reservierungsnummer,
        booking.checkin_date,
        booking.checkout_date,
        booking.anzahl_gaeste,
        status, // ← Only field changing
        booking.gesamtpreis,
        booking.bemerkungen,
        booking.anzahl_begleitpersonen,
        booking.grundpreis,
        booking.services_preis,
        booking.rabatt_preis,
        booking.anzahl_naechte,
        booking.bezahlt,
        booking.bezahlt_am,
        booking.zahlungsmethode,
        booking.mahnung_gesendet_am,
        booking.rechnung_versendet_am,
        booking.rechnung_versendet_an,
        booking.ist_stiftungsfall,
        booking.payment_recipient_id,
        booking.putzplan_checkout_date,
        booking.ist_dpolg_mitglied,
        booking.updated_at, // Optimistic locking
    )
    .await
    .map_err(|e| {
        eprintln!("❌ Error updating booking status: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn update_booking_statuses_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::Booking>, String> {
    println!("update_booking_statuses_pg called - updating all bookings based on current date");

    use chrono::NaiveDate;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Get all bookings
    let bookings = BookingRepository::get_all(&pool).await.map_err(|e| {
        eprintln!("❌ Error getting bookings: {}", e);
        e.to_string()
    })?;

    let mut updated_bookings = Vec::new();

    for booking in bookings {
        // Parse dates
        let checkin = NaiveDate::parse_from_str(&booking.checkin_date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid checkin date: {}", e))?;
        let checkout = NaiveDate::parse_from_str(&booking.checkout_date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid checkout date: {}", e))?;
        let today_date = NaiveDate::parse_from_str(&today, "%Y-%m-%d")
            .map_err(|e| format!("Invalid today date: {}", e))?;

        // Determine new status based on booking dates
        // Status progression: reserviert -> bestaetigt -> eingecheckt -> ausgecheckt
        let new_status = if today_date < checkin {
            // Before check-in: keep as 'reserviert' or 'bestaetigt' (don't auto-change confirmed bookings)
            if booking.status == "bestaetigt" {
                "bestaetigt".to_string()
            } else {
                "reserviert".to_string()
            }
        } else if today_date >= checkin && today_date < checkout {
            "eingecheckt".to_string()
        } else {
            "ausgecheckt".to_string()
        };

        // Only update if status changed (skip storniert and Stiftungsfall bookings)
        if booking.status != new_status
           && booking.status != "storniert"
           && !booking.ist_stiftungsfall.unwrap_or(false) {
            let updated = BookingRepository::update(
                &pool,
                booking.id,
                booking.room_id,
                booking.guest_id,
                booking.reservierungsnummer,
                booking.checkin_date,
                booking.checkout_date,
                booking.anzahl_gaeste,
                new_status,
                booking.gesamtpreis,
                booking.bemerkungen,
                booking.anzahl_begleitpersonen,
                booking.grundpreis,
                booking.services_preis,
                booking.rabatt_preis,
                booking.anzahl_naechte,
                booking.bezahlt,
                booking.bezahlt_am,
                booking.zahlungsmethode,
                booking.mahnung_gesendet_am,
                booking.rechnung_versendet_am,
                booking.rechnung_versendet_an,
                booking.ist_stiftungsfall,
                booking.payment_recipient_id,
                booking.putzplan_checkout_date,
                booking.ist_dpolg_mitglied,
                booking.updated_at,
            )
            .await
            .map_err(|e| {
                eprintln!("❌ Error updating booking {}: {}", booking.id, e);
                e.to_string()
            })?;

            updated_bookings.push(updated);
        }
    }

    println!("✅ Updated {} booking statuses", updated_bookings.len());
    Ok(updated_bookings)
}

#[tauri::command]
async fn update_booking_payment_pg(
    pool: State<'_, DbPool>,
    id: i32,
    bezahlt: bool,
    bezahlt_am: Option<String>,
    zahlungsmethode: Option<String>,
) -> Result<database_pg::Booking, String> {
    println!("update_booking_payment_pg called: id={}, bezahlt={}", id, bezahlt);

    let booking = BookingRepository::get_by_id(&pool, id).await.map_err(|e| {
        eprintln!("❌ Error getting booking: {}", e);
        e.to_string()
    })?;

    BookingRepository::update(
        &pool,
        id,
        booking.room_id,
        booking.guest_id,
        booking.reservierungsnummer,
        booking.checkin_date,
        booking.checkout_date,
        booking.anzahl_gaeste,
        booking.status,
        booking.gesamtpreis,
        booking.bemerkungen,
        booking.anzahl_begleitpersonen,
        booking.grundpreis,
        booking.services_preis,
        booking.rabatt_preis,
        booking.anzahl_naechte,
        Some(bezahlt), // ← Payment fields changing
        bezahlt_am,
        zahlungsmethode,
        booking.mahnung_gesendet_am,
        booking.rechnung_versendet_am,
        booking.rechnung_versendet_an,
        booking.ist_stiftungsfall,
        booking.payment_recipient_id,
        booking.putzplan_checkout_date,
        booking.ist_dpolg_mitglied,
        booking.updated_at,
    )
    .await
    .map_err(|e| {
        eprintln!("❌ Error updating booking payment: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn update_booking_dates_and_room_pg(
    pool: State<'_, DbPool>,
    id: i32,
    room_id: i32,
    checkin_date: String,
    checkout_date: String,
) -> Result<database_pg::Booking, String> {
    println!("update_booking_dates_and_room_pg called with availability check: id={}, room={}, checkin={}, checkout={}",
             id, room_id, checkin_date, checkout_date);

    // Get current booking to retrieve expected_updated_at for optimistic locking
    let booking = BookingRepository::get_by_id(&pool, id).await.map_err(|e| {
        eprintln!("❌ Error getting booking: {}", e);
        e.to_string()
    })?;

    // 1. Update booking with atomic availability check (prevents double bookings)
    let updated_booking = BookingRepository::update_dates_with_availability_check(
        &pool,
        id,
        room_id,
        checkin_date,
        checkout_date,
        booking.updated_at, // Optimistic locking
    )
    .await
    .map_err(|e| {
        eprintln!("❌ Error updating booking dates and room: {}", e);
        // Special error prefix for double booking - frontend can detect this
        match e {
            database_pg::DbError::DoubleBookingError(msg) => format!("DOUBLE_BOOKING:{}", msg),
            database_pg::DbError::ConflictError(msg) => format!("CONFLICT:{}", msg),
            _ => e.to_string()
        }
    })?;

    // 2. Sync updated cleaning tasks to Turso (mobile app)
    println!("🔄 Auto-syncing updated cleaning tasks to Turso for booking {}", id);
    if let Err(e) = turso_sync::sync_booking_tasks_to_turso(&pool, id).await {
        eprintln!("⚠️ Failed to sync cleaning tasks to Turso: {}", e);
        // Don't fail the booking update - mobile sync can be retried later
    }

    Ok(updated_booking)
}

// ============================================================================
// CLEANING TASKS / PUTZPLAN COMMANDS (Stubs - Legacy Mobile App Support)
// ============================================================================

/// STUB: Legacy command for cleaning tasks synchronization
/// The mobile cleaning app (Turso DB) syncs tasks separately
/// This command is called by frontend after booking CRUD operations
#[tauri::command]
async fn delete_booking_tasks(booking_id: i32, pool: State<'_, DbPool>) -> Result<(), String> {
    use crate::database_pg::repositories::CleaningTaskRepository;
    use crate::turso_sync;

    println!("🗑️ delete_booking_tasks called: booking_id={}", booking_id);

    // Delete from PostgreSQL
    CleaningTaskRepository::delete_for_booking(&pool, booking_id)
        .await
        .map_err(|e| format!("Failed to delete tasks from PostgreSQL: {}", e))?;

    println!("✅ Cleaning tasks deleted from PostgreSQL for booking {}", booking_id);

    // Delete from Turso
    match turso_sync::delete_tasks_from_turso(booking_id).await {
        Ok(_) => println!("✅ Cleaning tasks deleted from Turso"),
        Err(e) => println!("⚠️ Turso delete failed (non-critical): {}", e),
    }

    Ok(())
}

#[tauri::command]
async fn get_cleaning_stats_stub(pool: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    use crate::database_pg::repositories::CleaningTaskRepository;

    println!("📊 get_cleaning_stats called");

    let stats = CleaningTaskRepository::get_stats(&pool)
        .await
        .map_err(|e| format!("Failed to get cleaning stats: {}", e))?;

    println!("✅ Stats: today={}, tomorrow={}, this_week={}, total={}",
             stats.today, stats.tomorrow, stats.this_week, stats.total);

    Ok(serde_json::json!({
        "today": stats.today,
        "tomorrow": stats.tomorrow,
        "this_week": stats.this_week,
        "total": stats.total
    }))
}

#[tauri::command]
async fn sync_week_ahead(pool: State<'_, DbPool>) -> Result<String, String> {
    use crate::turso_sync;

    println!("🔄 sync_week_ahead called - PostgreSQL → Turso sync (3 months)");

    // Initialize Turso schema if needed
    turso_sync::init_turso_schema().await?;

    // Calculate date range (today + 90 days = 3 months)
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let three_months_from_now = chrono::Local::now()
        .checked_add_signed(chrono::Duration::days(90))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();

    println!("📅 Syncing tasks from {} to {}", today, three_months_from_now);

    // Sync tasks to Turso
    let synced_count = turso_sync::sync_tasks_to_turso(&pool, today, three_months_from_now).await?;

    Ok(format!("✅ {} Putzaufgaben für die nächsten 3 Monate nach Turso synchronisiert", synced_count))
}

// ============================================================================
// PRICING & VALIDATION COMMANDS (Stubs - TODO: Implement full logic)
// ============================================================================

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceInput {
    #[serde(alias = "name", alias = "serviceName", default)]
    service_name: String,
    #[serde(alias = "price", alias = "originalValue", default)]
    original_value: f64,  // Changed from f32 for precision consistency
    #[serde(alias = "priceType", default = "default_price_type")]
    price_type: String,
    #[serde(alias = "appliesTo", default = "default_applies_to")]
    applies_to: String,
}

fn default_price_type() -> String {
    "fixed".to_string()
}

fn default_applies_to() -> String {
    "overnight_price".to_string()
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscountInput {
    #[serde(alias = "name", alias = "discountName", default)]
    discount_name: String,
    #[serde(alias = "value", alias = "originalValue", default)]
    original_value: f64,  // Changed from f32 for precision consistency
    #[serde(alias = "priceType", alias = "discountType", default = "default_discount_type")]
    price_type: String,
    #[serde(alias = "appliesTo", default = "default_applies_to")]
    applies_to: String,
}

fn default_discount_type() -> String {
    "percent".to_string()
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceCalculation {
    name: String,
    price_type: String,
    original_value: f64,
    applies_to: String,
    calculated_price: f64,
    base_amount: Option<f64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscountCalculation {
    name: String,
    discount_type: String,
    original_value: f64,
    calculated_amount: f64,
    base_amount: Option<f64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FullPriceBreakdown {
    base_price: f64,
    nights: i32,
    price_per_night: f64,
    is_hauptsaison: bool,
    hauptsaison_nights: i32,
    nebensaison_nights: i32,
    services: Vec<ServiceCalculation>,
    services_total: f64,
    discounts: Vec<DiscountCalculation>,
    discounts_total: f64,
    subtotal: f64,
    total: f64,
}

/// TODO: Implement full price calculation logic with PostgreSQL
#[tauri::command]
async fn calculate_full_booking_price_pg(
    pool: State<'_, DbPool>,
    room_id: i32,
    checkin: String,
    checkout: String,
    is_member: bool,
    services: Option<Vec<ServiceInput>>,
    discounts: Option<Vec<DiscountInput>>,
) -> Result<FullPriceBreakdown, String> {
    use chrono::{NaiveDate, Datelike};

    // Get room details
    let room = RoomRepository::get_by_id(&pool, room_id)
        .await
        .map_err(|e| format!("Fehler beim Laden des Zimmers: {}", e))?;

    // Load PricingSettings for Hauptsaison and Mitgliederrabatt
    let pricing_settings = PricingSettingsRepository::get(&pool)
        .await
        .unwrap_or_else(|_| database_pg::PricingSettings {
            id: 1,
            hauptsaison_aktiv: Some(false),
            hauptsaison_start: Some("06-01".to_string()),
            hauptsaison_ende: Some("08-31".to_string()),
            mitglieder_rabatt_aktiv: Some(false),
            mitglieder_rabatt_prozent: Some(15.0),
            rabatt_basis: Some("zimmerpreis".to_string()),
            updated_at: None,
        });

    println!("🔧 [PRICING TEST] Loaded PricingSettings: rabatt_aktiv={}, rabatt_prozent={}, rabatt_basis={}, is_member={}",
             pricing_settings.mitglieder_rabatt_aktiv.unwrap_or(false),
             pricing_settings.mitglieder_rabatt_prozent.unwrap_or(0.0),
             pricing_settings.rabatt_basis.as_ref().unwrap_or(&"none".to_string()),
             is_member);

    // Calculate nights
    let checkin_date = NaiveDate::parse_from_str(&checkin, "%Y-%m-%d")
        .map_err(|e| format!("Ungültiges Check-in Datum: {}", e))?;
    let checkout_date = NaiveDate::parse_from_str(&checkout, "%Y-%m-%d")
        .map_err(|e| format!("Ungültiges Check-out Datum: {}", e))?;
    let nights = (checkout_date - checkin_date).num_days() as i32;

    if nights <= 0 {
        return Err("Check-out muss nach Check-in liegen".to_string());
    }

    // Fix 5: Per-night season calculation - check each night individually
    let nebensaison_price = room.nebensaison_preis.unwrap_or(0.0);
    let hauptsaison_price = room.hauptsaison_preis.unwrap_or(nebensaison_price);

    let (base_price, hauptsaison_nights, nebensaison_nights) = if pricing_settings.hauptsaison_aktiv.unwrap_or(false) {
        let start_str = pricing_settings.hauptsaison_start.clone().unwrap_or("06-01".to_string());
        let ende_str = pricing_settings.hauptsaison_ende.clone().unwrap_or("08-31".to_string());

        let mut total = 0.0;
        let mut hs_nights = 0i32;
        for offset in 0..nights {
            let night_date = checkin_date + chrono::Duration::days(offset as i64);
            let night_mmdd = format!("{:02}-{:02}", night_date.month(), night_date.day());
            let is_hs = if start_str > ende_str {
                night_mmdd >= start_str || night_mmdd <= ende_str
            } else {
                night_mmdd >= start_str && night_mmdd <= ende_str
            };
            if is_hs {
                total += hauptsaison_price;
                hs_nights += 1;
            } else {
                total += nebensaison_price;
            }
        }
        (total, hs_nights, nights - hs_nights)
    } else {
        (nebensaison_price * nights as f64, 0, nights)
    };

    let is_hauptsaison = hauptsaison_nights > 0;
    let price_per_night = if nights > 0 { base_price / nights as f64 } else { 0.0 };

    println!("📊 Hauptsaison: {} Nächte HS + {} Nächte NS, Durchschnitt {:.2}€/Nacht",
             hauptsaison_nights, nebensaison_nights, price_per_night);

    // Bug 6 fix: Two-pass service calculation for correct applies_to handling
    let services_list = services.unwrap_or_default();

    // Pass 1: Calculate fixed services and percent services that apply to overnight_price
    let mut service_calculations: Vec<ServiceCalculation> = Vec::new();
    let mut first_pass_total: f64 = 0.0;

    for s in &services_list {
        if s.price_type == "fixed" || s.applies_to == "overnight_price" {
            let calculated = if s.price_type == "percent" {
                base_price * (s.original_value / 100.0)  // percent of overnight_price
            } else {
                s.original_value  // fixed amount
            };
            first_pass_total += calculated;
            service_calculations.push(ServiceCalculation {
                name: s.service_name.clone(),
                price_type: s.price_type.clone(),
                original_value: s.original_value,
                applies_to: s.applies_to.clone(),
                calculated_price: calculated,
                base_amount: Some(base_price),
            });
        }
    }

    // Add Endreinigung (cleaning fee) from room if it exists AND not already in services
    // This prevents double-counting when services already include Endreinigung from DB
    let has_endreinigung = services_list.iter()
        .any(|s| s.service_name.to_lowercase().contains("endreinigung") || s.service_name.to_lowercase().contains("cleaning"));

    if !has_endreinigung {
        if let Some(endreinigung) = room.endreinigung {
            if endreinigung > 0.0 {
                first_pass_total += endreinigung;
                service_calculations.push(ServiceCalculation {
                    name: "Endreinigung".to_string(),
                    price_type: "fixed".to_string(),
                    original_value: endreinigung,
                    applies_to: "total_price".to_string(),
                    calculated_price: endreinigung,
                    base_amount: Some(base_price),
                });
            }
        }
    }

    // Pass 2: Calculate percent services that apply to total_price
    let total_price_base = base_price + first_pass_total;
    for s in &services_list {
        if s.price_type == "percent" && s.applies_to == "total_price" {
            let calculated = total_price_base * (s.original_value / 100.0);
            service_calculations.push(ServiceCalculation {
                name: s.service_name.clone(),
                price_type: s.price_type.clone(),
                original_value: s.original_value,
                applies_to: s.applies_to.clone(),
                calculated_price: calculated,
                base_amount: Some(total_price_base),
            });
        }
    }

    let services_total: f64 = service_calculations.iter().map(|s| s.calculated_price).sum();

    // Calculate discounts
    let subtotal_before_discount = base_price + services_total;
    let mut discount_calculations: Vec<DiscountCalculation> = discounts
        .unwrap_or_default()
        .iter()
        .map(|d| {
            let calculated = if d.price_type == "percent" {
                subtotal_before_discount * (d.original_value / 100.0)
            } else {
                d.original_value
            };
            DiscountCalculation {
                name: d.discount_name.clone(),
                discount_type: d.price_type.clone(),
                original_value: d.original_value,
                calculated_amount: calculated,
                base_amount: Some(subtotal_before_discount),
            }
        })
        .collect();

    // Apply automatic Mitgliederrabatt from PricingSettings if member and enabled
    if is_member && pricing_settings.mitglieder_rabatt_aktiv.unwrap_or(false) {
        let rabatt_prozent = pricing_settings.mitglieder_rabatt_prozent.unwrap_or(15.0);
        let rabatt_basis = pricing_settings.rabatt_basis.clone().unwrap_or("zimmerpreis".to_string());

        println!("🔧 [PRICING TEST] Calculating Mitgliederrabatt: prozent={}%, basis={}", rabatt_prozent, rabatt_basis);

        // Calculate base for discount
        let rabatt_base = if rabatt_basis == "gesamtpreis" {
            println!("🔧 [PRICING TEST] Using gesamtpreis as basis: {:.2}€", subtotal_before_discount);
            subtotal_before_discount
        } else {
            println!("🔧 [PRICING TEST] Using zimmerpreis as basis: {:.2}€", base_price);
            base_price // zimmerpreis
        };

        let auto_rabatt = rabatt_base * (rabatt_prozent / 100.0);

        println!("🔧 [PRICING TEST] Calculated auto_rabatt: {:.2}€ ({:.2}€ * {}%)", auto_rabatt, rabatt_base, rabatt_prozent);

        // Check if DPolG Rabatt already exists in discounts (avoid duplicates)
        let has_dpolg_rabatt = discount_calculations.iter()
            .any(|d| d.name.to_lowercase().contains("dpolg") || d.name.to_lowercase().contains("mitglieder"));

        println!("🔧 [PRICING TEST] has_dpolg_rabatt={}, auto_rabatt={:.2}", has_dpolg_rabatt, auto_rabatt);

        if !has_dpolg_rabatt && auto_rabatt > 0.0 {
            println!("📊 Adding automatic DPolG Mitgliederrabatt: {}% = {:.2}€ (basis: {})",
                     rabatt_prozent, auto_rabatt, rabatt_basis);

            discount_calculations.push(DiscountCalculation {
                name: format!("DPolG Mitgliederrabatt ({}%)", rabatt_prozent),
                discount_type: "percent".to_string(),
                original_value: rabatt_prozent,
                calculated_amount: auto_rabatt,
                base_amount: Some(rabatt_base),
            });
        }
    }

    let discounts_total: f64 = discount_calculations.iter().map(|d| d.calculated_amount).sum();

    let subtotal = base_price + services_total;
    let total = subtotal - discounts_total;

    Ok(FullPriceBreakdown {
        base_price,
        nights,
        price_per_night,
        is_hauptsaison,
        hauptsaison_nights,
        nebensaison_nights,
        services: service_calculations,
        services_total,
        discounts: discount_calculations,
        discounts_total,
        subtotal,
        total,
    })
}

// ============================================================================
// BATCH PRICE CALCULATION - For Lists (BookingList, GuestDetails, Statistics)
// ============================================================================

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BookingPriceResult {
    booking_id: i64,
    nights: i32,
    base_price: f64,
    services_total: f64,
    discounts_total: f64,
    total: f64,
}

/// Calculate prices for multiple bookings at once (batch operation)
/// This is more efficient than calling calculate_full_booking_price_pg for each booking
#[tauri::command]
async fn calculate_batch_booking_prices_pg(
    pool: State<'_, DbPool>,
    booking_ids: Vec<i64>,
) -> Result<Vec<BookingPriceResult>, String> {
    use chrono::{NaiveDate, Datelike};

    println!("═══════════════════════════════════════════════════════════════");
    println!("🔍 calculate_batch_booking_prices_pg CALLED");
    println!("═══════════════════════════════════════════════════════════════");
    println!("📥 booking_ids: {:?}", booking_ids);

    // Load PricingSettings once for all bookings (Bug 1 fix)
    let pricing_settings = PricingSettingsRepository::get(&pool)
        .await
        .unwrap_or_else(|_| database_pg::PricingSettings {
            id: 1,
            hauptsaison_aktiv: Some(false),
            hauptsaison_start: None,
            hauptsaison_ende: None,
            mitglieder_rabatt_aktiv: Some(false),
            mitglieder_rabatt_prozent: Some(15.0),
            rabatt_basis: Some("zimmerpreis".to_string()),
            updated_at: None,
        });

    let mut results = Vec::new();

    for booking_id in booking_ids {
        // Get booking details (convert i64 to i32 for repository)
        let booking = match BookingRepository::get_by_id(&pool, booking_id as i32).await {
            Ok(b) => b,
            Err(e) => {
                println!("⚠️ Could not load booking {}: {}", booking_id, e);
                continue;
            }
        };

        // Fix 3: Use stored snapshot values when available (konsistente Preise)
        if booking.grundpreis.is_some() {
            let grundpreis = booking.grundpreis.unwrap_or(0.0);
            let services_preis = booking.services_preis.unwrap_or(0.0);
            let rabatt_preis = booking.rabatt_preis.unwrap_or(0.0);
            let nights = booking.anzahl_naechte.unwrap_or_else(|| {
                // Fallback: Nächte aus Datum berechnen
                NaiveDate::parse_from_str(&booking.checkout_date, "%Y-%m-%d")
                    .and_then(|co| NaiveDate::parse_from_str(&booking.checkin_date, "%Y-%m-%d").map(|ci| (co - ci).num_days() as i32))
                    .unwrap_or(1)
            });

            results.push(BookingPriceResult {
                booking_id,
                nights,
                base_price: grundpreis,
                services_total: services_preis,
                discounts_total: rabatt_preis,
                total: booking.gesamtpreis,
            });
            continue; // Keine Neuberechnung nötig
        }

        // Fallback: Neuberechnung für alte Buchungen ohne Snapshots
        // Get room for price calculation
        let room = match RoomRepository::get_by_id(&pool, booking.room_id).await {
            Ok(r) => r,
            Err(e) => {
                println!("⚠️ Could not load room for booking {}: {}", booking_id, e);
                continue;
            }
        };

        // Get guest for member status
        let guest = match GuestRepository::get_by_id(&pool, booking.guest_id).await {
            Ok(g) => g,
            Err(e) => {
                println!("⚠️ Could not load guest for booking {}: {}", booking_id, e);
                continue;
            }
        };

        // Calculate nights
        let checkin_date = match NaiveDate::parse_from_str(&booking.checkin_date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };
        let checkout_date = match NaiveDate::parse_from_str(&booking.checkout_date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };
        let nights = (checkout_date - checkin_date).num_days() as i32;

        // Fix 5: Per-night season calculation
        let ns_price = room.nebensaison_preis.unwrap_or(0.0);
        let hs_price = room.hauptsaison_preis.unwrap_or(ns_price);

        let base_price = if pricing_settings.hauptsaison_aktiv.unwrap_or(false) {
            let start_str = pricing_settings.hauptsaison_start.clone().unwrap_or("06-01".to_string());
            let ende_str = pricing_settings.hauptsaison_ende.clone().unwrap_or("08-31".to_string());
            let mut total = 0.0;
            for offset in 0..nights {
                let night_date = checkin_date + chrono::Duration::days(offset as i64);
                let night_mmdd = format!("{:02}-{:02}", night_date.month(), night_date.day());
                let is_hs = if start_str > ende_str {
                    night_mmdd >= start_str || night_mmdd <= ende_str
                } else {
                    night_mmdd >= start_str && night_mmdd <= ende_str
                };
                total += if is_hs { hs_price } else { ns_price };
            }
            total
        } else {
            ns_price * nights as f64
        };

        // Get services for this booking
        let services = AdditionalServiceRepository::get_by_booking(&pool, booking_id)
            .await
            .unwrap_or_default();

        // Two-pass service calculation for correct applies_to handling
        let mut first_pass_total: f64 = 0.0;
        for s in &services {
            if s.price_type == "fixed" || s.applies_to == "overnight_price" {
                let calculated = if s.price_type == "percent" {
                    base_price * (s.original_value / 100.0)
                } else {
                    s.original_value
                };
                first_pass_total += calculated;
            }
        }

        let total_price_base = base_price + first_pass_total;
        let mut second_pass_total: f64 = 0.0;
        for s in &services {
            if s.price_type == "percent" && s.applies_to == "total_price" {
                second_pass_total += total_price_base * (s.original_value / 100.0);
            }
        }

        let services_total = first_pass_total + second_pass_total;

        // Get discounts for this booking
        let discounts = DiscountRepository::get_by_booking(&pool, booking_id)
            .await
            .unwrap_or_default();

        // Calculate manual discounts total
        let subtotal_before_discount = base_price + services_total;
        let mut discounts_total: f64 = discounts.iter().map(|d| {
            // Use stored calculated_amount if available
            d.calculated_amount.unwrap_or_else(|| {
                if d.discount_type == "percent" {
                    subtotal_before_discount * (d.discount_value) / 100.0
                } else {
                    d.discount_value
                }
            })
        }).sum();

        // Apply automatic DPolG member discount from PricingSettings
        if guest.dpolg_mitglied && pricing_settings.mitglieder_rabatt_aktiv.unwrap_or(false) {
            let rabatt_prozent = pricing_settings.mitglieder_rabatt_prozent.unwrap_or(15.0);
            let rabatt_basis = pricing_settings.rabatt_basis.clone().unwrap_or("zimmerpreis".to_string());

            let rabatt_base = if rabatt_basis == "gesamtpreis" {
                subtotal_before_discount
            } else {
                base_price
            };

            let auto_rabatt = rabatt_base * (rabatt_prozent / 100.0);

            let has_dpolg_rabatt = discounts.iter()
                .any(|d| d.discount_name.to_lowercase().contains("dpolg")
                      || d.discount_name.to_lowercase().contains("mitglieder"));

            if !has_dpolg_rabatt && auto_rabatt > 0.0 {
                discounts_total += auto_rabatt;
            }
        }

        let total = base_price + services_total - discounts_total;

        results.push(BookingPriceResult {
            booking_id,
            nights,
            base_price,
            services_total,
            discounts_total,
            total,
        });
    }

    println!("✅ Calculated prices for {} bookings", results.len());
    Ok(results)
}

/// TODO: Implement room availability check with PostgreSQL
#[tauri::command]
async fn check_room_availability_pg(
    pool: State<'_, DbPool>,
    room_id: i32,
    checkin: String,
    checkout: String,
    exclude_booking_id: Option<i32>,
) -> Result<bool, String> {
    println!("🏠 Checking room availability: room_id={}, {} to {}, exclude={:?}",
             room_id, checkin, checkout, exclude_booking_id);

    let client = pool.get().await.map_err(|e| e.to_string())?;

    // Check for overlapping bookings
    // Date overlap logic: (checkin < other_checkout) AND (checkout > other_checkin)
    let query = if let Some(exclude_id) = exclude_booking_id {
        "SELECT COUNT(*) as count
         FROM bookings
         WHERE room_id = $1
           AND checkin_date < $3
           AND checkout_date > $2
           AND status != 'storniert'
           AND id != $4"
    } else {
        "SELECT COUNT(*) as count
         FROM bookings
         WHERE room_id = $1
           AND checkin_date < $3
           AND checkout_date > $2
           AND status != 'storniert'"
    };

    let row = if let Some(exclude_id) = exclude_booking_id {
        client.query_one(query, &[&room_id, &checkin, &checkout, &exclude_id])
            .await
            .map_err(|e| format!("Fehler bei Verfügbarkeitsprüfung: {}", e))?
    } else {
        client.query_one(query, &[&room_id, &checkin, &checkout])
            .await
            .map_err(|e| format!("Fehler bei Verfügbarkeitsprüfung: {}", e))?
    };

    let count: i64 = row.get("count");
    let is_available = count == 0;

    println!("✅ Room {} is {}: {} overlapping bookings found",
             room_id,
             if is_available { "available" } else { "NOT available" },
             count);

    Ok(is_available)
}

// ============================================================================
// ADDITIONAL SERVICES COMMANDS
// ============================================================================

#[tauri::command]
async fn get_all_additional_services_pg(pool: State<'_, DbPool>) -> Result<Vec<database_pg::AdditionalService>, String> {
    AdditionalServiceRepository::get_all(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_additional_service_by_id_pg(pool: State<'_, DbPool>, id: i64) -> Result<database_pg::AdditionalService, String> {
    AdditionalServiceRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_additional_services_by_booking_pg(pool: State<'_, DbPool>, booking_id: i64) -> Result<Vec<database_pg::AdditionalService>, String> {
    AdditionalServiceRepository::get_by_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_additional_service_pg(
    pool: State<'_, DbPool>,
    booking_id: i64,
    service_name: String,
    service_price: f64,  // Changed from f32
    template_id: Option<i32>,
    price_type: String,
    original_value: f64,  // Changed from f32
    applies_to: String,
) -> Result<database_pg::AdditionalService, String> {
    AdditionalServiceRepository::create(&pool, booking_id, service_name, service_price, template_id, price_type, original_value, applies_to)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_additional_service_pg(
    pool: State<'_, DbPool>,
    id: i64,
    booking_id: i64,
    service_name: String,
    service_price: f64,  // Changed from f32
    template_id: Option<i32>,
    price_type: String,
    original_value: f64,  // Changed from f32
    applies_to: String,
) -> Result<database_pg::AdditionalService, String> {
    AdditionalServiceRepository::update(&pool, id, booking_id, service_name, service_price, template_id, price_type, original_value, applies_to)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_additional_service_pg(pool: State<'_, DbPool>, id: i64) -> Result<(), String> {
    AdditionalServiceRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn calculate_additional_services_total_pg(pool: State<'_, DbPool>, booking_id: i64) -> Result<f64, String> {
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
async fn get_discount_by_id_pg(pool: State<'_, DbPool>, id: i64) -> Result<database_pg::Discount, String> {
    DiscountRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_discounts_by_booking_pg(pool: State<'_, DbPool>, booking_id: i64) -> Result<Vec<database_pg::Discount>, String> {
    DiscountRepository::get_by_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_discount_pg(
    pool: State<'_, DbPool>,
    booking_id: i64,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
    calculated_amount: Option<f64>,
) -> Result<database_pg::Discount, String> {
    DiscountRepository::create(&pool, booking_id, discount_name, discount_type, discount_value, calculated_amount)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_discount_pg(
    pool: State<'_, DbPool>,
    id: i64,
    booking_id: i64,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
    calculated_amount: Option<f64>,
) -> Result<database_pg::Discount, String> {
    DiscountRepository::update(&pool, id, booking_id, discount_name, discount_type, discount_value, calculated_amount)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_discount_pg(pool: State<'_, DbPool>, id: i64) -> Result<(), String> {
    DiscountRepository::delete(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn calculate_discounts_total_pg(pool: State<'_, DbPool>, booking_id: i64) -> Result<f64, String> {
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
    println!("get_active_reminders_pg called");
    ReminderRepository::get_active(&pool).await.map_err(|e| {
        eprintln!("❌ Error getting active reminders: {}", e);
        e.to_string()
    })
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
async fn get_accompanying_guest_by_id_pg(pool: State<'_, DbPool>, id: i64) -> Result<database_pg::AccompanyingGuest, String> {
    AccompanyingGuestRepository::get_by_id(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_accompanying_guests_by_booking_pg(pool: State<'_, DbPool>, booking_id: i64) -> Result<Vec<database_pg::AccompanyingGuest>, String> {
    AccompanyingGuestRepository::get_by_booking(&pool, booking_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_accompanying_guest_pg(
    pool: State<'_, DbPool>,
    booking_id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
    _companion_id: Option<i32>,
) -> Result<database_pg::AccompanyingGuest, String> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("🔍 [DEBUG] create_accompanying_guest_pg CALLED");
    println!("═══════════════════════════════════════════════════════════════");
    println!("📥 booking_id: {}", booking_id);
    println!("📥 vorname: {}", vorname);
    println!("📥 nachname: {}", nachname);
    println!("📥 geburtsdatum: {:?}", geburtsdatum);

    let result = AccompanyingGuestRepository::create(&pool, booking_id, vorname.clone(), nachname.clone(), geburtsdatum.clone())
        .await;

    match &result {
        Ok(guest) => {
            println!("✅ [DEBUG] AccompanyingGuest created successfully!");
            println!("   id: {}", guest.id);
            println!("   booking_id: {}", guest.booking_id);
            println!("   vorname: {}", guest.vorname);
            println!("   nachname: {}", guest.nachname);
        }
        Err(e) => {
            println!("❌ [DEBUG] Error creating AccompanyingGuest: {}", e);
        }
    }
    println!("═══════════════════════════════════════════════════════════════");

    result.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_accompanying_guest_pg(
    pool: State<'_, DbPool>,
    id: i64,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
) -> Result<database_pg::AccompanyingGuest, String> {
    AccompanyingGuestRepository::update(&pool, id, vorname, nachname, geburtsdatum)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_accompanying_guest_pg(pool: State<'_, DbPool>, id: i64) -> Result<(), String> {
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
    println!("get_active_service_templates_pg called");
    ServiceTemplateRepository::get_active(&pool).await.map_err(|e| {
        eprintln!("❌ Error getting service templates: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn create_service_template_pg(
    pool: State<'_, DbPool>,
    name: String,
    description: Option<String>,
    price_type: String,
    price: f32,
    applies_to: String,
    emoji: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
) -> Result<database_pg::ServiceTemplate, String> {
    ServiceTemplateRepository::create(
        &pool, name, description, price_type, price.into(), applies_to,
        emoji, show_in_cleaning_plan, cleaning_plan_position
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_service_template_pg(
    pool: State<'_, DbPool>,
    id: i32,
    name: String,
    description: Option<String>,
    price_type: String,
    price: f32,
    applies_to: String,
    is_active: bool,
    emoji: Option<String>,
    show_in_cleaning_plan: bool,
    cleaning_plan_position: String,
) -> Result<database_pg::ServiceTemplate, String> {
    ServiceTemplateRepository::update(
        &pool, id, name, description, price_type, price.into(), applies_to, is_active,
        emoji, show_in_cleaning_plan, cleaning_plan_position
    ).await.map_err(|e| e.to_string())
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
    println!("get_active_discount_templates_pg called");
    DiscountTemplateRepository::get_active(&pool).await.map_err(|e| {
        eprintln!("❌ Error getting discount templates: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn create_discount_template_pg(
    pool: State<'_, DbPool>,
    discount_name: String,
    discount_type: String,
    discount_value: f64,
    emoji: Option<String>,
) -> Result<database_pg::DiscountTemplate, String> {
    DiscountTemplateRepository::create(&pool, discount_name, discount_type, discount_value, emoji)
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
    emoji: Option<String>,
) -> Result<database_pg::DiscountTemplate, String> {
    DiscountTemplateRepository::update(&pool, id, discount_name, discount_type, discount_value, is_active, emoji)
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
    println!("get_all_payment_recipients_pg called");
    PaymentRecipientRepository::get_all(&pool).await.map_err(|e| {
        eprintln!("❌ Error getting payment recipients: {}", e);
        e.to_string()
    })
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
    company: Option<String>,
    street: Option<String>,
    plz: Option<String>,
    city: Option<String>,
    country: Option<String>,
    contact_person: Option<String>,
    notes: Option<String>,
) -> Result<database_pg::PaymentRecipient, String> {
    PaymentRecipientRepository::create(&pool, name, company, street, plz, city, country, contact_person, notes)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_payment_recipient_pg(
    pool: State<'_, DbPool>,
    id: i32,
    name: String,
    company: Option<String>,
    street: Option<String>,
    plz: Option<String>,
    city: Option<String>,
    country: Option<String>,
    contact_person: Option<String>,
    notes: Option<String>,
) -> Result<database_pg::PaymentRecipient, String> {
    PaymentRecipientRepository::update(&pool, id, name, company, street, plz, city, country, contact_person, notes)
        .await.map_err(|e| e.to_string())
}

// Removed toggle_active command - payment_recipients table no longer has is_active column

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
    let mut settings = CompanySettingsRepository::get(&pool).await.map_err(|e| e.to_string())?;

    // Auto-migrate: If logo_path exists but logo_data is empty, try to read the file locally
    if settings.logo_data.is_none() || settings.logo_data.as_deref() == Some("") {
        if let Some(ref logo_path) = settings.logo_path {
            if !logo_path.is_empty() && std::path::Path::new(logo_path).exists() {
                println!("🔄 Auto-migrating logo from file path to DB: {}", logo_path);
                match std::fs::read(logo_path) {
                    Ok(file_bytes) => {
                        if file_bytes.len() <= 2 * 1024 * 1024 {
                            let base64_data = general_purpose::STANDARD.encode(&file_bytes);
                            let mime_type = if logo_path.to_lowercase().ends_with(".png") {
                                "image/png"
                            } else if logo_path.to_lowercase().ends_with(".jpg") || logo_path.to_lowercase().ends_with(".jpeg") {
                                "image/jpeg"
                            } else {
                                "image/png"
                            };
                            settings.logo_data = Some(base64_data);
                            settings.logo_mime_type = Some(mime_type.to_string());
                            // Save back to DB
                            if let Err(e) = CompanySettingsRepository::update(&pool, &settings).await {
                                eprintln!("⚠️ Failed to auto-migrate logo to DB: {}", e);
                            } else {
                                println!("✅ Logo auto-migrated to DB ({} bytes)", file_bytes.len());
                            }
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Could not read logo file for auto-migration: {}", e);
                    }
                }
            }
        }
    }

    Ok(settings)
}

#[tauri::command]
async fn update_company_settings_pg(
    pool: State<'_, DbPool>,
    settings: database_pg::CompanySettings,
) -> Result<database_pg::CompanySettings, String> {
    CompanySettingsRepository::update(&pool, &settings)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_pricing_settings_pg(pool: State<'_, DbPool>) -> Result<database_pg::PricingSettings, String> {
    PricingSettingsRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_pricing_settings_pg(
    pool: State<'_, DbPool>,
    settings: database_pg::PricingSettings,
) -> Result<database_pg::PricingSettings, String> {
    PricingSettingsRepository::update(&pool, &settings)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_email_config_pg(pool: State<'_, DbPool>) -> Result<database_pg::EmailConfig, String> {
    EmailConfigRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_email_config_pg(
    pool: State<'_, DbPool>,
    settings: database_pg::EmailConfig,
) -> Result<database_pg::EmailConfig, String> {
    EmailConfigRepository::update(&pool, &settings)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_notification_settings_pg(pool: State<'_, DbPool>) -> Result<database_pg::NotificationSettings, String> {
    NotificationSettingsRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_notification_settings_pg(
    pool: State<'_, DbPool>,
    settings: database_pg::NotificationSettings,
) -> Result<database_pg::NotificationSettings, String> {
    NotificationSettingsRepository::update(&pool, &settings)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_payment_settings_pg(pool: State<'_, DbPool>) -> Result<database_pg::PaymentSettings, String> {
    PaymentSettingsRepository::get(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_payment_settings_pg(
    pool: State<'_, DbPool>,
    settings: database_pg::PaymentSettings,
) -> Result<database_pg::PaymentSettings, String> {
    PaymentSettingsRepository::update(&pool, &settings)
        .await.map_err(|e| e.to_string())
}

// ============================================================================
// GUEST COMPANIONS & CREDIT (STUB IMPLEMENTATION)
// TODO: Implement full functionality when guest_companions table is migrated
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GuestCompanion {
    id: i32,
    guest_id: i32,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
    beziehung: Option<String>,
    notizen: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GuestCreditBalance {
    balance: f64,
}

#[tauri::command]
async fn get_guest_companions_command(
    pool: State<'_, DbPool>,
    guest_id: i32,
) -> Result<Vec<GuestCompanion>, String> {
    println!("🔍 [DEBUG] get_guest_companions_command called for guest_id={}", guest_id);

    let client = pool.get().await.map_err(|e| format!("Pool error: {}", e))?;

    let rows = client
        .query(
            "SELECT id, guest_id, vorname, nachname, geburtsdatum, beziehung, notizen
             FROM guest_companions
             WHERE guest_id = $1
             ORDER BY nachname, vorname",
            &[&guest_id],
        )
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let companions: Vec<GuestCompanion> = rows
        .iter()
        .map(|row| GuestCompanion {
            id: row.get("id"),
            guest_id: row.get("guest_id"),
            vorname: row.get("vorname"),
            nachname: row.get("nachname"),
            geburtsdatum: row.try_get("geburtsdatum").ok().flatten(),
            beziehung: row.try_get("beziehung").ok().flatten(),
            notizen: row.try_get("notizen").ok().flatten(),
        })
        .collect();

    println!("✅ [DEBUG] Found {} companions for guest_id={}", companions.len(), guest_id);
    Ok(companions)
}

#[tauri::command]
async fn get_guest_credit_balance(guest_id: i64, pool: State<'_, DbPool>) -> Result<GuestCreditBalance, String> {
    use crate::database_pg::repositories::GuestCreditRepository;

    println!("💰 get_guest_credit_balance called for guest_id={}", guest_id);

    let balance = GuestCreditRepository::get_balance(&pool, guest_id as i32)
        .await
        .map_err(|e| format!("Failed to get credit balance: {}", e))?;

    println!("✅ Guest {} balance: {:.2}€", guest_id, balance);
    Ok(GuestCreditBalance { balance })
}

#[tauri::command]
async fn run_guest_credit_migration(pool: State<'_, DbPool>) -> Result<String, String> {
    use crate::database_pg::repositories::GuestCreditRepository;

    println!("🔧 Running guest credit system migration...");

    let result = GuestCreditRepository::run_migration(&pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    println!("✅ Migration completed successfully");
    Ok(result)
}

#[tauri::command]
async fn run_cleaning_tasks_migration(pool: State<'_, DbPool>) -> Result<String, String> {
    use crate::database_pg::repositories::CleaningTaskRepository;

    println!("🔧 Running cleaning tasks system migration...");

    let result = CleaningTaskRepository::run_migration(&pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    println!("✅ Cleaning tasks migration completed successfully");
    Ok(result)
}

#[tauri::command]
async fn run_email_automation_migration(pool: State<'_, DbPool>) -> Result<String, String> {
    use crate::database_pg::repositories::ScheduledEmailRepository;

    println!("🔧 Running email automation system migration...");

    let result = ScheduledEmailRepository::run_migration(&pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    println!("✅ Email automation migration completed successfully");
    Ok(result)
}

#[tauri::command]
async fn run_discount_calculated_amount_migration(pool: State<'_, DbPool>) -> Result<String, String> {
    println!("🔧 Running discount calculated_amount migration...");

    let result = DiscountRepository::run_migration(&pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    println!("✅ Discount calculated_amount migration completed successfully");
    Ok(result)
}

#[tauri::command]
async fn run_realtime_notify_migration(pool: State<'_, DbPool>) -> Result<String, String> {
    println!("🔧 Running Real-Time NOTIFY triggers migration...");

    let client = pool.get().await.map_err(|e| format!("Database error: {}", e))?;

    // Read migration SQL file
    let migration_sql = include_str!("../../migrations/007_realtime_notify_triggers.sql");

    // Execute migration
    client
        .batch_execute(migration_sql)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    println!("✅ Real-Time NOTIFY triggers migration completed successfully");
    Ok("Real-Time NOTIFY triggers created successfully".to_string())
}

// ============================================================================
// REAL-TIME MULTI-USER POLLING API
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatesSinceResponse {
    pub bookings: Vec<database_pg::Booking>,
    pub guests: Vec<database_pg::Guest>,
    pub rooms: Vec<database_pg::Room>,
    pub timestamp: String,
}

#[tauri::command]
async fn get_updates_since(
    pool: State<'_, DbPool>,
    since_timestamp: String,
) -> Result<UpdatesSinceResponse, String> {
    let client = pool.get().await.map_err(|e| format!("Database error: {}", e))?;

    // Get all bookings updated since timestamp
    let bookings_query = "
        SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
               anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
               anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
               anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
               mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
               ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
               ist_dpolg_mitglied, NULL::double precision as credit_used
        FROM bookings
        WHERE updated_at > $1
        ORDER BY updated_at DESC
    ";

    let booking_rows = client
        .query(bookings_query, &[&since_timestamp])
        .await
        .map_err(|e| format!("Error fetching updated bookings: {}", e))?;

    let bookings: Vec<database_pg::Booking> = booking_rows
        .into_iter()
        .map(database_pg::Booking::from)
        .collect();

    // Guests table has no updated_at column, so we can't track changes
    // Return empty vector - guests rarely change and full refresh handles updates
    let guests: Vec<database_pg::Guest> = Vec::new();

    // Rooms table has no updated_at column, so we can't track changes
    // Return empty vector - rooms rarely change and full refresh handles updates
    let rooms: Vec<database_pg::Room> = Vec::new();

    // Current timestamp for next poll
    let current_timestamp = chrono::Utc::now().to_rfc3339();

    println!("📊 Updates since {}: {} bookings, {} guests, {} rooms",
        since_timestamp, bookings.len(), guests.len(), rooms.len());

    Ok(UpdatesSinceResponse {
        bookings,
        guests,
        rooms,
        timestamp: current_timestamp,
    })
}

#[tauri::command]
async fn fix_guest_credit_table_structure(pool: State<'_, DbPool>) -> Result<String, String> {
    println!("🔧 [FIX] Adding missing columns to guest_credit_transactions...");

    let client = pool.get().await
        .map_err(|e| format!("DB Pool Error: {}", e))?;

    // Add description column if missing
    let fix_description = r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'guest_credit_transactions'
                AND column_name = 'description'
            ) THEN
                ALTER TABLE guest_credit_transactions ADD COLUMN description TEXT;
                RAISE NOTICE '✅ Added description column';
            ELSE
                RAISE NOTICE 'ℹ️ description column already exists';
            END IF;
        END $$;
    "#;

    client.batch_execute(fix_description).await
        .map_err(|e| format!("Failed to add description: {:?}", e))?;

    println!("✅ description column checked/added");

    // Add created_by column if missing
    let fix_created_by = r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'guest_credit_transactions'
                AND column_name = 'created_by'
            ) THEN
                ALTER TABLE guest_credit_transactions ADD COLUMN created_by VARCHAR(100);
                RAISE NOTICE '✅ Added created_by column';
            ELSE
                RAISE NOTICE 'ℹ️ created_by column already exists';
            END IF;
        END $$;
    "#;

    client.batch_execute(fix_created_by).await
        .map_err(|e| format!("Failed to add created_by: {:?}", e))?;

    println!("✅ created_by column checked/added");

    // Verify table structure
    let verify = client.query(
        "SELECT column_name, data_type FROM information_schema.columns
         WHERE table_name = 'guest_credit_transactions'
         ORDER BY ordinal_position",
        &[]
    ).await.map_err(|e| format!("Verification failed: {:?}", e))?;

    println!("📋 Table structure:");
    for row in verify {
        let col_name: String = row.get(0);
        let col_type: String = row.get(1);
        println!("   - {}: {}", col_name, col_type);
    }

    Ok("✅ guest_credit_transactions table structure fixed successfully".to_string())
}

#[tauri::command]
async fn backfill_scheduled_emails(pool: State<'_, DbPool>) -> Result<String, String> {
    println!("🔧 Backfilling scheduled emails for existing bookings...");

    let client = pool.get().await
        .map_err(|e| format!("DB Pool Error: {}", e))?;

    // Check notification settings
    let settings_check = "SELECT checkin_reminders_enabled FROM notification_settings WHERE id = 1";
    let settings_row = client.query_one(settings_check, &[]).await
        .map_err(|e| format!("Settings check failed: {:?}", e))?;
    let reminders_enabled: bool = settings_row.get(0);

    println!("⚙️  Notification Settings: checkin_reminders_enabled = {}", reminders_enabled);

    if !reminders_enabled {
        return Ok("❌ Check-in reminders are DISABLED in notification settings. Enable them first in Settings > Notifications.".to_string());
    }

    // First, check how many bookings qualify
    let check_query = "
        SELECT COUNT(*) as total
        FROM bookings b
        JOIN guests g ON g.id = b.guest_id
        WHERE b.status NOT IN ('cancelled', 'storniert')
          AND g.email IS NOT NULL
          AND g.email != ''
          AND b.checkin_date::date >= CURRENT_DATE
    ";

    let total_row = client.query_one(check_query, &[])
        .await
        .map_err(|e| format!("Check query failed: {:?}", e))?;
    let total_qualifying: i64 = total_row.get(0);

    println!("📊 Found {} qualifying bookings (future, not cancelled, with email)", total_qualifying);

    // Check how many already have PENDING scheduled emails
    let existing_query = "
        SELECT COUNT(DISTINCT booking_id) as existing
        FROM scheduled_emails
        WHERE status = 'pending'
    ";

    let existing_row = client.query_one(existing_query, &[])
        .await
        .map_err(|e| format!("Existing check failed: {:?}", e))?;
    let existing_count: i64 = existing_row.get(0);

    println!("📊 {} bookings already have PENDING scheduled emails", existing_count);

    // Call the trigger function for all active bookings
    let result = client
        .execute(
            "INSERT INTO scheduled_emails (booking_id, guest_id, template_name, recipient_email, subject, scheduled_for, status)
             SELECT
                 b.id,
                 b.guest_id,
                 'booking_reminder',
                 g.email,
                 'Erinnerung: Ihre Buchung #' || b.id::text,
                 (b.checkin_date::date - INTERVAL '3 days')::TIMESTAMP,
                 'pending'
             FROM bookings b
             JOIN guests g ON g.id = b.guest_id
             WHERE b.status NOT IN ('cancelled', 'storniert')
               AND g.email IS NOT NULL
               AND g.email != ''
               AND b.checkin_date::date >= CURRENT_DATE
               AND NOT EXISTS (
                   SELECT 1 FROM scheduled_emails se
                   WHERE se.booking_id = b.id
                   AND se.template_name = 'booking_reminder'
                   AND se.status = 'pending'
               )",
            &[]
        )
        .await
        .map_err(|e| {
            let error_msg = format!("Backfill SQL Error: {:?}", e);
            eprintln!("❌ {}", error_msg);
            error_msg
        })?;

    let count = result;
    println!("✅ Created {} scheduled emails for existing bookings", count);
    Ok(format!("Created {} scheduled emails (from {} qualifying bookings, {} already had emails)", count, total_qualifying, existing_count))
}

#[tauri::command]
async fn link_service_template_to_booking_command(
    booking_id: i64,
    service_template_id: i64,
    pool: State<'_, DbPool>,
) -> Result<(), String> {
    use crate::database_pg::repositories::{ServiceTemplateRepository, AdditionalServiceRepository};

    println!("📦 link_service_template_to_booking_command called");
    println!("   booking_id: {}, service_template_id: {}", booking_id, service_template_id);

    // Get the service template
    let template = ServiceTemplateRepository::get_by_id(&pool, service_template_id as i32)
        .await
        .map_err(|e| format!("Failed to get service template: {}", e))?;

    println!("   Found template: {} (price_type: {}, value: {})",
             template.service_name, template.price_type, template.original_value);

    // Create additional service for this booking
    // Note: service_price will be calculated later based on booking details
    let service = AdditionalServiceRepository::create(
        &pool,
        booking_id,
        template.service_name.clone(),
        template.original_value, // Initial price = template value (now f64)
        Some(service_template_id as i32),
        template.price_type,
        template.original_value,  // Now f64
        template.applies_to,
    )
    .await
    .map_err(|e| format!("Failed to create additional service: {}", e))?;

    println!("✅ Created additional service with id: {}", service.id);

    Ok(())
}

#[tauri::command]
async fn link_discount_template_to_booking_command(
    booking_id: i64,
    discount_template_id: i64,
    pool: State<'_, DbPool>,
) -> Result<(), String> {
    use crate::database_pg::repositories::{DiscountTemplateRepository, DiscountRepository};

    println!("📦 link_discount_template_to_booking_command called");
    println!("   booking_id: {}, discount_template_id: {}", booking_id, discount_template_id);

    // Get the discount template
    let template = DiscountTemplateRepository::get_by_id(&pool, discount_template_id as i32)
        .await
        .map_err(|e| format!("Failed to get discount template: {}", e))?;

    println!("   Found template: {} (type: {}, value: {})",
             template.discount_name, template.discount_type, template.discount_value);

    // Create discount for this booking (no pre-calculated amount for template-linked discounts)
    let discount = DiscountRepository::create(
        &pool,
        booking_id,
        template.discount_name.clone(),
        template.discount_type,
        template.discount_value,
        None,
    )
    .await
    .map_err(|e| format!("Failed to create discount: {}", e))?;

    println!("✅ Created discount with id: {}", discount.id);

    Ok(())
}

// ============================================================================
// MISSING STUB COMMANDS (Bug #6-10)
// ============================================================================

#[tauri::command]
async fn cancel_booking_command(pool: State<'_, DbPool>, booking_id: i64) -> Result<(), String> {
    println!("🚫 Cancelling booking {}", booking_id);

    // Load existing booking
    let booking = BookingRepository::get_by_id(&pool, booking_id as i32)
        .await
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    // Update with status 'storniert'
    BookingRepository::update(
        &pool,
        booking.id,
        booking.room_id,
        booking.guest_id,
        booking.reservierungsnummer,
        booking.checkin_date,
        booking.checkout_date,
        booking.anzahl_gaeste,
        "storniert".to_string(),  // ← Changed status
        booking.gesamtpreis,
        booking.bemerkungen,
        booking.anzahl_begleitpersonen,
        booking.grundpreis,
        booking.services_preis,
        booking.rabatt_preis,
        booking.anzahl_naechte,
        booking.bezahlt,
        booking.bezahlt_am,
        booking.zahlungsmethode,
        booking.mahnung_gesendet_am,
        booking.rechnung_versendet_am,
        booking.rechnung_versendet_an,
        booking.ist_stiftungsfall,
        booking.payment_recipient_id,
        booking.putzplan_checkout_date,
        booking.ist_dpolg_mitglied,
        None,  // No optimistic locking for cancel
    )
    .await
    .map_err(|e| format!("Fehler beim Stornieren: {}", e))?;

    println!("✅ Booking {} cancelled successfully", booking_id);
    Ok(())
}

#[tauri::command]
async fn validate_email_command(email: String) -> Result<bool, String> {
    println!("📧 Validating email: {}", email);

    // RFC 5322 simplified email validation
    let email_regex = regex::Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();

    let is_valid = email_regex.is_match(&email);
    println!("✅ Email validation: {} -> {}", email, is_valid);
    Ok(is_valid)
}

#[tauri::command]
async fn validate_date_range_command(
    checkin: String,
    checkout: String,
) -> Result<bool, String> {
    println!("📅 Validating date range: {} to {}", checkin, checkout);

    use chrono::NaiveDate;
    match (
        NaiveDate::parse_from_str(&checkin, "%Y-%m-%d"),
        NaiveDate::parse_from_str(&checkout, "%Y-%m-%d"),
    ) {
        (Ok(check_in), Ok(check_out)) => {
            let is_valid = check_out > check_in;
            println!("✅ Date range validation: {} -> {}", if is_valid { "valid" } else { "invalid" }, is_valid);
            Ok(is_valid)
        },
        _ => {
            println!("❌ Invalid date format");
            Err("Ungültiges Datumsformat. Erwartet: YYYY-MM-DD".to_string())
        }
    }
}

#[tauri::command]
async fn calculate_nights_command(
    checkin: String,
    checkout: String,
) -> Result<i32, String> {
    println!("📅 Calculating nights between {} and {}", checkin, checkout);

    use chrono::NaiveDate;
    match (
        NaiveDate::parse_from_str(&checkin, "%Y-%m-%d"),
        NaiveDate::parse_from_str(&checkout, "%Y-%m-%d"),
    ) {
        (Ok(check_in), Ok(check_out)) => {
            let nights = (check_out - check_in).num_days() as i32;
            println!("✅ Calculated {} nights", nights);
            Ok(nights)
        }
        _ => {
            println!("❌ Invalid date format");
            Err("Ungültiges Datumsformat. Erwartet: YYYY-MM-DD".to_string())
        }
    }
}

#[derive(Serialize)]
struct ReportStats {
    total_bookings: i64,
    active_bookings: i64,
    total_revenue: f64,
    total_nights: i64,
    average_price_per_night: f64,
    occupancy_rate: f64,
}

#[tauri::command]
async fn get_report_stats_command(
    pool: State<'_, DbPool>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<ReportStats, String> {
    println!("📊 get_report_stats_command called");
    println!("   Period: {:?} to {:?}", start_date, end_date);

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Build date filter (using correct column names: checkin_date, checkout_date, gesamtpreis)
    let date_filter = match (&start_date, &end_date) {
        (Some(start), Some(end)) =>
            format!("WHERE checkin_date >= '{}' AND checkout_date <= '{}'", start, end),
        (Some(start), None) =>
            format!("WHERE checkin_date >= '{}'", start),
        (None, Some(end)) =>
            format!("WHERE checkout_date <= '{}'", end),
        (None, None) => "".to_string(),
    };

    // Get statistics with correct column names
    let query = format!(
        "SELECT
            COUNT(*) as total_bookings,
            COUNT(*) FILTER (WHERE status != 'storniert') as active_bookings,
            COALESCE(SUM(gesamtpreis) FILTER (WHERE status != 'storniert'), 0)::numeric::text as total_revenue,
            COALESCE(SUM(
                CASE
                    WHEN anzahl_naechte IS NOT NULL THEN anzahl_naechte
                    ELSE (checkout_date::date - checkin_date::date)
                END
            ) FILTER (WHERE status != 'storniert'), 0) as total_nights
         FROM bookings
         {}",
        date_filter,
    );

    let row = client
        .query_one(&query, &[])
        .await
        .map_err(|e| format!("Fehler beim Laden der Statistiken: {}", e))?;

    let total_bookings: i64 = row.get("total_bookings");
    let active_bookings: i64 = row.get("active_bookings");
    let total_revenue: String = row.get("total_revenue");
    let total_nights: i64 = row.get("total_nights");
    let average_price_per_night = if total_nights > 0 {
        total_revenue.parse::<f64>().unwrap_or(0.0) / total_nights as f64
    } else {
        0.0
    };

    // Calculate real occupancy rate (booked nights / available nights * 100)
    let occupancy_rate = if active_bookings > 0 {
        // Get number of rooms
        let room_count: i64 = client
            .query_one("SELECT COUNT(*) FROM rooms", &[])
            .await
            .map(|r| r.get(0))
            .unwrap_or(10);

        // Calculate available nights (rooms * days in period)
        let available_nights = match (&start_date, &end_date) {
            (Some(start), Some(end)) => {
                let days = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d")
                    .and_then(|e| chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").map(|s| (e - s).num_days()))
                    .unwrap_or(30);
                room_count * days.max(1)
            }
            _ => room_count * 30,
        };

        if available_nights > 0 {
            (total_nights as f64 / available_nights as f64 * 100.0).min(100.0)
        } else {
            0.0
        }
    } else {
        0.0
    };

    println!("✅ Stats: {} bookings ({} active), {:.2}€ revenue, {} nights",
             total_bookings, active_bookings, total_revenue.parse::<f64>().unwrap_or(0.0), total_nights);

    Ok(ReportStats {
        total_bookings,
        active_bookings,
        total_revenue: total_revenue.parse().unwrap_or(0.0),
        total_nights,
        average_price_per_night,
        occupancy_rate,
    })
}

#[derive(Serialize)]
struct RoomOccupancy {
    room_id: i32,
    room_name: String,
    total_bookings: i32,
    total_nights: i32,
    total_revenue: f64,
    occupancy_rate: f64,
}

#[tauri::command]
async fn get_room_occupancy_command(
    pool: State<'_, DbPool>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<RoomOccupancy>, String> {
    println!("🏠 get_room_occupancy_command called");
    println!("   Period: {:?} to {:?}", start_date, end_date);

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Build date filter
    let date_filter = match (start_date.clone(), end_date.clone()) {
        (Some(start), Some(end)) => format!("AND b.checkin_date >= '{}' AND b.checkout_date <= '{}'", start, end),
        (Some(start), None) => format!("AND b.checkin_date >= '{}'", start),
        (None, Some(end)) => format!("AND b.checkout_date <= '{}'", end),
        (None, None) => "".to_string(),
    };

    // Calculate occupancy per room
    let query = format!(
        "SELECT
            r.id as room_id,
            r.name as room_name,
            COUNT(b.id) as booking_count,
            COALESCE(SUM(
                CASE
                    WHEN b.anzahl_naechte IS NOT NULL THEN b.anzahl_naechte
                    ELSE (b.checkout_date::date - b.checkin_date::date)
                END
            ), 0) as total_nights_booked,
            COALESCE(SUM(b.gesamtpreis), 0)::numeric::text as total_revenue
         FROM rooms r
         LEFT JOIN bookings b ON b.room_id = r.id
            AND b.status NOT IN ('cancelled', 'storniert')
            {}
         GROUP BY r.id, r.name
         ORDER BY r.name",
        date_filter
    );

    let rows = client
        .query(&query, &[])
        .await
        .map_err(|e| format!("Fehler beim Laden der Auslastung: {}", e))?;

    // Calculate total available nights in period
    let total_days = match (start_date, end_date) {
        (Some(start), Some(end)) => {
            use chrono::NaiveDate;
            let start_date = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
                .map_err(|e| format!("Ungültiges Startdatum: {}", e))?;
            let end_date = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
                .map_err(|e| format!("Ungültiges Enddatum: {}", e))?;
            (end_date - start_date).num_days() as i32
        },
        _ => 365,
    };

    let occupancies: Vec<RoomOccupancy> = rows
        .iter()
        .map(|row| {
            let room_id: i32 = row.get("room_id");
            let room_name: String = row.get("room_name");
            let booking_count: i64 = row.get("booking_count");
            let total_nights_booked: i64 = row.get("total_nights_booked");
            let total_revenue_str: String = row.get("total_revenue");
            let total_revenue = total_revenue_str.parse::<f64>().unwrap_or(0.0);

            let occupancy_rate = if total_days > 0 {
                (total_nights_booked as f64 / total_days as f64) * 100.0
            } else {
                0.0
            };

            RoomOccupancy {
                room_id,
                room_name,
                total_bookings: booking_count as i32,
                total_nights: total_nights_booked as i32,
                total_revenue,
                occupancy_rate,
            }
        })
        .collect();

    println!("✅ Loaded occupancy for {} rooms", occupancies.len());
    Ok(occupancies)
}

// ============================================================================
// INVOICE & CREDIT COMMANDS (STUB)
// ============================================================================

/// STUB: Get all invoice PDFs for a booking
/// Returns list of generated invoice PDF files
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InvoicePdfInfo {
    filename: String,
    path: String,
    size: u64,
}

#[tauri::command]
async fn get_invoice_pdfs_for_booking_command(
    app: tauri::AppHandle,
    booking_id: i64,
) -> Result<Vec<InvoicePdfInfo>, String> {
    use tauri::Manager;
    use crate::database_pg::repositories::booking_repository::BookingRepository;

    println!("📄 get_invoice_pdfs_for_booking_command called for booking_id={}", booking_id);

    // Get the booking's reservierungsnummer from database
    let pool = app.state::<crate::database_pg::pool::DbPool>();
    let booking = BookingRepository::get_by_id(pool.inner(), booking_id as i32)
        .await
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let reservierungsnummer = booking.reservierungsnummer;
    println!("📄 Booking reservierungsnummer: {}", reservierungsnummer);

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Fehler beim Abrufen des App-Verzeichnisses: {}", e))?;

    let invoices_dir = app_data_dir.join("invoices");

    if !invoices_dir.exists() {
        println!("📂 Invoices directory does not exist yet");
        return Ok(vec![]);
    }

    let mut pdfs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&invoices_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();

                // Only match PDFs that contain this booking's reservierungsnummer
                // Formats: "Rechnung_2025-169.pdf", "Rechnung_RES-2025-169.pdf", etc.
                if filename_str.ends_with(".pdf") && filename_str.contains(&reservierungsnummer) {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        pdfs.push(InvoicePdfInfo {
                            filename: filename_str.to_string(),
                            path: path.to_string_lossy().to_string(),
                            size: metadata.len(),
                        });
                    }
                }
            }
        }
    }

    println!("📄 Found {} invoice PDFs for booking {}", pdfs.len(), reservierungsnummer);
    Ok(pdfs)
}

// ============================================================================
// CLEANING & EMAIL COMMANDS (STUB)
// ============================================================================

#[tauri::command]
async fn cleanup_cleaning_tasks(pool: State<'_, DbPool>) -> Result<String, String> {
    use crate::database_pg::repositories::CleaningTaskRepository;
    use crate::turso_sync;

    println!("🧹 cleanup_cleaning_tasks called - Reinitializing Turso schema");

    // 1. Cleanup PostgreSQL (older than 90 days)
    let pg_deleted = CleaningTaskRepository::cleanup_old_tasks(&pool, 90)
        .await
        .map_err(|e| format!("Failed to cleanup PostgreSQL tasks: {}", e))?;

    println!("✅ Deleted {} old cleaning tasks from PostgreSQL", pg_deleted);

    // 2. Reinitialize Turso schema (drops and recreates table)
    turso_sync::init_turso_schema()
        .await
        .map_err(|e| format!("Failed to reinitialize Turso schema: {}", e))?;

    println!("✅ Turso schema reinitialized - all old tasks cleared");

    Ok(format!(
        "✅ Cleanup abgeschlossen: {} aus PostgreSQL gelöscht, Turso-Tabelle neu initialisiert.\n\nBitte jetzt '3 Monate synchronisieren' klicken!",
        pg_deleted
    ))
}

/// STUB: Send confirmation email for a booking
#[tauri::command]
async fn send_confirmation_email_command(pool: State<'_, DbPool>, booking_id: i64) -> Result<String, String> {
    println!("📧 Sending confirmation email for booking {}", booking_id);

    let booking = BookingRepository::get_by_id(&pool, booking_id as i32)
        .await
        .map_err(|e| format!("Buchung nicht gefunden: {}", e))?;

    let guest = GuestRepository::get_by_id(&pool, booking.guest_id)
        .await
        .map_err(|e| format!("Gast nicht gefunden: {}", e))?;

    let guest_email = if guest.email.is_empty() {
        return Err("Gast hat keine Email-Adresse".to_string());
    } else {
        guest.email.clone()
    };

    let room = RoomRepository::get_by_id(&pool, booking.room_id)
        .await
        .map_err(|e| format!("Zimmer nicht gefunden: {}", e))?;

    let (subject, body) = match EmailTemplateRepository::get_by_name(&pool, "bestaetigung".to_string()).await {
        Ok(template) => (
            replace_template_placeholders(&template.subject, &booking, &guest, &room.name),
            replace_template_placeholders(&template.body, &booking, &guest, &room.name),
        ),
        Err(_) => {
            let subject = format!("Buchungsbestätigung #{}", booking_id);
            let body = format!(
                "Sehr geehrte/r {} {},\n\n\
                vielen Dank für Ihre Buchung! Wir freuen uns, Ihnen folgende Reservierung zu bestätigen:\n\n\
                Buchungsnummer: {}\n\
                Zimmer: {}\n\
                Check-in: {}\n\
                Check-out: {}\n\
                Anzahl Nächte: {}\n\
                Anzahl Gäste: {}\n\
                Gesamtpreis: {:.2} €\n\n\
                Bei Fragen stehen wir Ihnen jederzeit gerne zur Verfügung.\n\n\
                Mit freundlichen Grüßen,\n\
                Ihr DPolG Buchungsteam",
                guest.vorname, guest.nachname, booking_id, room.name,
                booking.checkin_date, booking.checkout_date,
                booking.anzahl_naechte.unwrap_or(1),
                booking.anzahl_gaeste, booking.gesamtpreis
            );
            (subject, body)
        }
    };

    let send_result = send_email_helper(&pool, &guest_email, &format!("{} {}", guest.vorname, guest.nachname), &subject, &body).await;

    // Log email send result
    {
        use crate::database_pg::repositories::EmailLogRepository;
        let (status, error_msg) = match &send_result {
            Ok(_) => ("gesendet".to_string(), None),
            Err(e) => ("fehler".to_string(), Some(e.clone())),
        };
        let _ = EmailLogRepository::create(
            &pool,
            Some(booking_id as i32),
            guest.id,
            "confirmation".to_string(),
            guest_email.clone(),
            subject.clone(),
            status,
            error_msg,
        ).await;
    }

    send_result?;

    println!("✅ Confirmation email sent to {}", guest_email);
    Ok(format!("Buchungsbestätigung an {} gesendet", guest_email))
}

// ============================================================================
// ALL MISSING COMMANDS (STUB) - Found via Frontend Analysis
// ============================================================================

#[tauri::command]
async fn add_guest_credit(guest_id: i64, amount: f64, description: String, pool: State<'_, DbPool>) -> Result<(), String> {
    use crate::database_pg::repositories::GuestCreditRepository;

    println!("➕ add_guest_credit: guest_id={}, amount={:.2}€, description={}", guest_id, amount, description);

    GuestCreditRepository::add_credit(
        &pool,
        guest_id as i32,
        amount,
        Some(description),
        Some("System".to_string()),
    )
    .await
    .map_err(|e| format!("Failed to add credit: {}", e))?;

    println!("✅ Credit added successfully");
    Ok(())
}

#[tauri::command]
async fn create_backup_command(pool: State<'_, DbPool>) -> Result<String, String> {
    use chrono::Local;
    use std::io::Write;

    println!("💾 Creating database backup...");

    // Get database connection
    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Create backup directory if it doesn't exist
    // Store backups OUTSIDE src-tauri to avoid triggering file watcher
    let backup_dir = std::path::Path::new("../backups");
    if !backup_dir.exists() {
        std::fs::create_dir_all(backup_dir)
            .map_err(|e| format!("Fehler beim Erstellen des Backup-Ordners: {}", e))?;
    }

    // Generate backup filename with timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_filename = format!("backup_{}.sql", timestamp);
    let backup_path = backup_dir.join(&backup_filename);

    println!("📁 Backup file: {:?}", backup_path);

    // Create SQL backup file
    let mut backup_file = std::fs::File::create(&backup_path)
        .map_err(|e| format!("Fehler beim Erstellen der Backup-Datei: {}", e))?;

    // Write backup header
    writeln!(backup_file, "-- PostgreSQL Database Backup")
        .map_err(|e| format!("Fehler beim Schreiben: {}", e))?;
    writeln!(backup_file, "-- Generated: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))
        .map_err(|e| format!("Fehler beim Schreiben: {}", e))?;
    writeln!(backup_file, "-- Database: dpolg_booking\n")
        .map_err(|e| format!("Fehler beim Schreiben: {}", e))?;

    // List of all tables to backup (in correct order due to foreign keys)
    let tables = vec![
        "company_settings",
        "pricing_settings",
        "payment_settings",
        "notification_settings",
        "reminder_settings",
        "email_config",
        "rooms",
        "guests",
        "bookings",
        "additional_services",
        "discounts",
        "accompanying_guests",
        "service_templates",
        "discount_templates",
        "payment_recipients",
        "email_templates",
        "email_logs",
        "scheduled_emails",
        "guest_credit_transactions",
        "cleaning_tasks",
    ];

    println!("📊 Backing up {} tables...", tables.len());

    for table in tables {
        println!("  → Backing up table: {}", table);

        // Get table structure
        let schema_query = format!(
            "SELECT 'CREATE TABLE IF NOT EXISTS {} (' || string_agg(column_name || ' ' || data_type || \
             CASE WHEN character_maximum_length IS NOT NULL THEN '(' || character_maximum_length || ')' ELSE '' END || \
             CASE WHEN is_nullable = 'NO' THEN ' NOT NULL' ELSE '' END, ', ') || ');' \
             FROM information_schema.columns WHERE table_name = '{}' GROUP BY table_name",
            table, table
        );

        // Write table structure comment
        writeln!(backup_file, "\n-- Table: {}", table)
            .map_err(|e| format!("Fehler beim Schreiben: {}", e))?;

        // Get all data from table
        let data_query = format!("SELECT * FROM {}", table);
        let rows = client.query(&data_query, &[]).await
            .map_err(|e| format!("Fehler beim Lesen von {}: {}", table, e))?;

        if rows.is_empty() {
            writeln!(backup_file, "-- No data in {}\n", table)
                .map_err(|e| format!("Fehler beim Schreiben: {}", e))?;
            continue;
        }

        println!("  ✓ {} rows from {}", rows.len(), table);

        // Write INSERT statements
        for row in rows {
            let columns: Vec<String> = (0..row.len())
                .map(|i| row.columns()[i].name().to_string())
                .collect();

            let values: Vec<String> = (0..row.len())
                .map(|i| {
                    let col_type = row.columns()[i].type_();
                    match col_type.name() {
                        "int4" | "int8" => {
                            let val: Option<i64> = row.try_get(i).ok();
                            val.map(|v| v.to_string()).unwrap_or("NULL".to_string())
                        },
                        "bool" => {
                            let val: Option<bool> = row.try_get(i).ok();
                            val.map(|v| v.to_string()).unwrap_or("NULL".to_string())
                        },
                        "float8" | "numeric" => {
                            let val: Option<f64> = row.try_get(i).ok();
                            val.map(|v| v.to_string()).unwrap_or("NULL".to_string())
                        },
                        _ => {
                            // Text, varchar, timestamp, etc.
                            let val: Option<String> = row.try_get(i).ok();
                            val.map(|v| format!("'{}'", v.replace("'", "''"))).unwrap_or("NULL".to_string())
                        }
                    }
                })
                .collect();

            let insert = format!(
                "INSERT INTO {} ({}) VALUES ({});\n",
                table,
                columns.join(", "),
                values.join(", ")
            );

            write!(backup_file, "{}", insert)
                .map_err(|e| format!("Fehler beim Schreiben: {}", e))?;
        }
    }

    println!("✅ Backup erfolgreich erstellt: {}", backup_filename);
    Ok(backup_filename)
}

#[tauri::command]
async fn create_guest_companion_command(
    pool: State<'_, DbPool>,
    guest_id: i32,
    vorname: String,
    nachname: String,
    geburtsdatum: Option<String>,
    beziehung: Option<String>,
    notizen: Option<String>,
) -> Result<GuestCompanion, String> {
    println!("🔍 [DEBUG] create_guest_companion_command: guest_id={}, vorname={}, nachname={}", guest_id, vorname, nachname);

    let client = pool.get().await.map_err(|e| format!("Pool error: {}", e))?;

    let row = client
        .query_one(
            "INSERT INTO guest_companions (guest_id, vorname, nachname, geburtsdatum, beziehung, notizen)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, guest_id, vorname, nachname, geburtsdatum, beziehung, notizen",
            &[&guest_id, &vorname, &nachname, &geburtsdatum, &beziehung, &notizen],
        )
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let companion = GuestCompanion {
        id: row.get("id"),
        guest_id: row.get("guest_id"),
        vorname: row.get("vorname"),
        nachname: row.get("nachname"),
        geburtsdatum: row.try_get("geburtsdatum").ok().flatten(),
        beziehung: row.try_get("beziehung").ok().flatten(),
        notizen: row.try_get("notizen").ok().flatten(),
    };

    println!("✅ [DEBUG] Created guest companion with id={}", companion.id);
    Ok(companion)
}


#[tauri::command]
async fn delete_backup_command(backup_name: String) -> Result<(), String> {
    println!("🗑️ Deleting backup: {}", backup_name);

    // Security check: only allow .sql files
    if !backup_name.ends_with(".sql") {
        return Err("Ungültiger Dateiname. Nur .sql Dateien können gelöscht werden.".to_string());
    }

    // Security check: prevent path traversal
    if backup_name.contains("..") || backup_name.contains("/") || backup_name.contains("\\") {
        return Err("Ungültiger Dateiname. Pfad-Traversal nicht erlaubt.".to_string());
    }

    let backup_path = std::path::Path::new("../backups").join(&backup_name);

    // Check if file exists
    if !backup_path.exists() {
        return Err(format!("Backup-Datei '{}' nicht gefunden", backup_name));
    }

    // Delete the file
    std::fs::remove_file(&backup_path)
        .map_err(|e| format!("Fehler beim Löschen der Backup-Datei: {}", e))?;

    println!("✅ Backup '{}' erfolgreich gelöscht", backup_name);
    Ok(())
}

#[tauri::command]
async fn export_cleaning_timeline_pdf(
    pool: State<'_, DbPool>,
    app: tauri::AppHandle,
    year: i32,
    month: i32,
) -> Result<String, String> {
    // Delegate to the proper timeline PDF generator (now with PostgreSQL pool)
    crate::cleaning_timeline_pdf::export_cleaning_timeline_pdf(pool, app, year, month as u32).await
}

#[tauri::command]
async fn generate_and_send_invoice_command(
    app: tauri::AppHandle,
    pool: State<'_, DbPool>,
    booking_id: i64,
) -> Result<String, String> {
    println!("📧💼 generate_and_send_invoice_command called for booking_id={}", booking_id);

    // Step 1: Generate PDF
    println!("📄 Step 1: Generating invoice PDF...");
    let pdf_path = generate_invoice_pdf_command(app.clone(), booking_id).await?;
    println!("✅ PDF generated: {}", pdf_path);

    // Step 2: Send email with PDF attachment
    println!("📧 Step 2: Sending invoice email...");
    send_invoice_email_command(pool.clone(), booking_id).await?;
    println!("✅ Email sent");

    // Step 3: Mark invoice as sent
    println!("📝 Step 3: Marking invoice as sent...");
    mark_invoice_sent_command(pool, booking_id).await?;
    println!("✅ Invoice marked as sent");

    Ok(format!("Rechnung generiert und versendet: {}", pdf_path))
}

#[tauri::command]
async fn generate_invoice_pdf_command(
    app: tauri::AppHandle,
    booking_id: i64,
) -> Result<String, String> {
    use tauri::Manager;
    use headless_chrome::{Browser, LaunchOptions};
    use headless_chrome::types::PrintToPdfOptions;

    println!("════════════════════════════════════════════════════════");
    println!("🔵 PDF GENERATION STARTED (PostgreSQL)");
    println!("📋 Booking ID: {}", booking_id);
    println!("════════════════════════════════════════════════════════");

    // 1. Load booking data from PostgreSQL
    let pool = app.state::<crate::database_pg::pool::DbPool>();
    let booking = BookingRepository::get_with_details(pool.inner(), booking_id as i32)
        .await
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let reservierungsnummer = &booking.booking.reservierungsnummer;
    println!("✅ Booking loaded: {}", reservierungsnummer);

    // 2. Get app data directory for PDF storage
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("App directory error: {}", e))?;
    let invoices_dir = app_data_dir.join("invoices");
    std::fs::create_dir_all(&invoices_dir)
        .map_err(|e| format!("Create invoices dir error: {}", e))?;

    // 3. Generate filename
    let filename = format!("Rechnung_{}.pdf", reservierungsnummer);
    let pdf_path = invoices_dir.join(&filename);

    // 4. Load company settings for invoice
    let company_settings = CompanySettingsRepository::get(pool.inner())
        .await
        .map_err(|e| format!("Fehler beim Laden der Firmeneinstellungen: {}", e))?;

    // 5. Load payment settings
    let payment_settings = PaymentSettingsRepository::get(pool.inner())
        .await
        .map_err(|e| format!("Fehler beim Laden der Zahlungseinstellungen: {}", e))?;

    // 5b. Load external payment recipient if set
    let payment_recipient = if let Some(recipient_id) = booking.booking.payment_recipient_id {
        match PaymentRecipientRepository::get_by_id(pool.inner(), recipient_id).await {
            Ok(recipient) => {
                println!("✅ Loaded external payment recipient: {} (ID: {})", recipient.name, recipient_id);
                Some(recipient)
            }
            Err(e) => {
                println!("⚠️ Could not load payment recipient {}: {}", recipient_id, e);
                None
            }
        }
    } else {
        None
    };

    // 5c. Load pricing settings for discount basis (Bug 2 fix)
    let pricing_settings = PricingSettingsRepository::get(pool.inner())
        .await
        .unwrap_or_else(|_| database_pg::PricingSettings {
            id: 1,
            hauptsaison_aktiv: Some(false),
            hauptsaison_start: None,
            hauptsaison_ende: None,
            mitglieder_rabatt_aktiv: Some(false),
            mitglieder_rabatt_prozent: Some(15.0),
            rabatt_basis: Some("zimmerpreis".to_string()),
            updated_at: None,
        });

    // 6. Generate HTML invoice
    let guest = booking.guest.as_ref().ok_or("Kein Gast gefunden")?;
    let room = booking.room.as_ref().ok_or("Kein Zimmer gefunden")?;

    let html = generate_invoice_html_pg(
        &booking,
        guest,
        room,
        &company_settings,
        &payment_settings,
        payment_recipient.as_ref(),
        &pricing_settings,  // Bug 2 fix: Pass pricing settings
    )?;

    // 7. Launch headless Chrome and generate PDF
    println!("🚀 Starting headless Chrome...");

    let launch_options = LaunchOptions::default_builder()
        .headless(true)
        .sandbox(true)
        .build()
        .map_err(|e| format!("Chrome launch options error: {}", e))?;

    let browser = Browser::new(launch_options)
        .map_err(|e| format!("Chrome start error: {}", e))?;

    let tab = browser.new_tab()
        .map_err(|e| format!("Chrome tab error: {}", e))?;

    // Load HTML content
    let html_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &html);
    let data_url = format!("data:text/html;base64,{}", html_base64);

    tab.navigate_to(&data_url)
        .map_err(|e| format!("Navigate error: {}", e))?;

    tab.wait_until_navigated()
        .map_err(|e| format!("Wait error: {}", e))?;

    // Generate PDF
    let pdf_options = PrintToPdfOptions {
        landscape: Some(false),
        print_background: Some(true),
        paper_width: Some(8.27),  // A4
        paper_height: Some(11.69),
        margin_top: Some(0.4),
        margin_bottom: Some(0.4),
        margin_left: Some(0.4),
        margin_right: Some(0.4),
        ..Default::default()
    };

    let pdf_data = tab.print_to_pdf(Some(pdf_options))
        .map_err(|e| format!("PDF generation error: {}", e))?;

    // Save PDF
    std::fs::write(&pdf_path, pdf_data)
        .map_err(|e| format!("Save PDF error: {}", e))?;

    let path_str = pdf_path.to_string_lossy().to_string();
    println!("✅ PDF SUCCESSFULLY GENERATED!");
    println!("📄 Path: {}", path_str);
    println!("════════════════════════════════════════════════════════");

    Ok(path_str)
}


// ============================================================================
// INVOICE HTML GENERATION - PORTED FROM OLD SQLite VERSION (commit 73cebf9)
// ============================================================================
// This is adapted to PostgreSQL while keeping the EXACT same HTML logic

/// Generiert EPC QR-Code für SEPA-Zahlungen
/// Format: European Payment Council Standard
fn generate_payment_qr_code(
    iban: &str,
    bic: &str,
    account_holder: &str,
    amount: f64,
    reference: &str,
) -> Result<String, String> {
    // EPC QR Code Format (European Payment Council)
    // https://www.europeanpaymentscouncil.eu/document-library/guidance-documents/quick-response-code-guidelines-enable-data-capture-initiation
    let epc_data = format!(
        "BCD\n002\n1\nSCT\n{}\n{}\n{}\nEUR{:.2}\n\n\n{}",
        bic,
        account_holder,
        iban,
        amount,
        reference
    );

    println!("🔧 [QR] Generating EPC QR Code:");
    println!("   IBAN: {}", iban);
    println!("   BIC: {}", bic);
    println!("   Amount: {:.2} EUR", amount);
    println!("   Reference: {}", reference);

    // Generiere QR Code
    let qr_code = QrCode::new(epc_data.as_bytes())
        .map_err(|e| format!("Fehler beim Erstellen des QR-Codes: {}", e))?;

    // Rendere als PNG Bild (5px pro Modul für bessere Lesbarkeit)
    let image = qr_code.render::<Luma<u8>>()
        .min_dimensions(200, 200)
        .max_dimensions(300, 300)
        .build();

    // Konvertiere zu PNG Bytes
    let mut png_bytes = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Fehler beim Kodieren des PNG: {}", e))?;

    // Encode als Base64
    let base64_image = general_purpose::STANDARD.encode(&png_bytes);

    println!("✅ [QR] QR Code generated successfully ({} bytes)", png_bytes.len());

    Ok(format!("data:image/png;base64,{}", base64_image))
}

fn generate_invoice_html_pg(
    booking: &crate::database_pg::BookingWithDetails,
    guest: &crate::database_pg::models::Guest,
    room: &crate::database_pg::models::Room,
    company: &crate::database_pg::models::CompanySettings,
    payment: &crate::database_pg::models::PaymentSettings,
    payment_recipient: Option<&crate::database_pg::models::PaymentRecipient>,
    pricing_settings: &crate::database_pg::models::PricingSettings,  // Bug 2 fix: Added for discount basis
) -> Result<String, String> {
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  INVOICE HTML GENERATOR (PostgreSQL - PORTED)       │");
    println!("│  Booking ID: {}                                    │", booking.booking.id);
    println!("└─────────────────────────────────────────────────────┘");

    let b = &booking.booking;

    // 1. Lade Template (EXAKT GLEICH WIE SQLite VERSION)
    let template = include_str!("../../invoice_modern_template.html");
    let mut html = template.to_string();

    // ============================================================================
    // HELPER FUNCTIONS (EXAKT GLEICH WIE SQLite VERSION)
    // ============================================================================

    let format_german_date = |date_str: &str| -> String {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            date.format("%d.%m.%Y").to_string()
        } else {
            date_str.to_string()
        }
    };

    let calculate_nights = |checkin: &str, checkout: &str| -> i32 {
        if let (Ok(date_in), Ok(date_out)) = (
            NaiveDate::parse_from_str(checkin, "%Y-%m-%d"),
            NaiveDate::parse_from_str(checkout, "%Y-%m-%d")
        ) {
            (date_out - date_in).num_days() as i32
        } else {
            1
        }
    };

    let format_currency = |amount: f64| -> String {
        format!("{:.2} €", amount).replace(".", ",")
    };

    let nights = calculate_nights(&b.checkin_date, &b.checkout_date);

    // ============================================================================
    // HEADER - Company Info (EXAKT GLEICH WIE SQLite VERSION)
    // ============================================================================
    html = html.replace("{{COMPANY_NAME}}", &company.company_name);
    html = html.replace("{{STREET_ADDRESS}}", company.street_address.as_deref().unwrap_or(""));
    html = html.replace("{{PLZ}}", company.plz.as_deref().unwrap_or(""));
    html = html.replace("{{CITY}}", company.city.as_deref().unwrap_or(""));
    html = html.replace("{{COUNTRY}}", company.country.as_deref().unwrap_or("Deutschland"));
    html = html.replace("{{PHONE}}", company.phone.as_deref().unwrap_or(""));
    html = html.replace("{{EMAIL}}", company.email.as_deref().unwrap_or(""));
    html = html.replace("{{TAX_ID}}", company.tax_id.as_deref().unwrap_or(""));

    // Logo (als Base64 Data-URL für PDF-Generation mit headless-chrome)
    // Primary: Use logo_data from DB (Base64). Fallback: Read from logo_path file.
    let logo_html = if let (Some(ref logo_data), Some(ref mime_type)) = (&company.logo_data, &company.logo_mime_type) {
        if !logo_data.is_empty() {
            let data_url = format!("data:{};base64,{}", mime_type, logo_data);
            println!("🖼️ [INVOICE] Logo aus DB geladen ({} bytes Base64)", logo_data.len());
            format!(r#"<img src="{}" alt="Company Logo" style="max-width: 100%; max-height: 100%; object-fit: contain;" />"#, data_url)
        } else {
            format!(r#"<div class="logo-placeholder">[LOGO]<br>{}</div>"#, &company.company_name)
        }
    } else if let Some(ref logo_path) = company.logo_path {
        if !logo_path.is_empty() && std::path::Path::new(logo_path).exists() {
            match std::fs::read(logo_path) {
                Ok(logo_bytes) => {
                    let base64_logo = general_purpose::STANDARD.encode(&logo_bytes);
                    let mime_type = if logo_path.to_lowercase().ends_with(".png") {
                        "image/png"
                    } else if logo_path.to_lowercase().ends_with(".jpg") || logo_path.to_lowercase().ends_with(".jpeg") {
                        "image/jpeg"
                    } else {
                        "image/png"
                    };
                    let data_url = format!("data:{};base64,{}", mime_type, base64_logo);
                    println!("🖼️ [INVOICE] Logo aus Datei geladen: {} ({} bytes)", logo_path, logo_bytes.len());
                    format!(r#"<img src="{}" alt="Company Logo" style="max-width: 100%; max-height: 100%; object-fit: contain;" />"#, data_url)
                },
                Err(e) => {
                    println!("⚠️ [INVOICE] Fehler beim Laden des Logos: {}", e);
                    format!(r#"<div class="logo-placeholder">[LOGO]<br>{}</div>"#, &company.company_name)
                }
            }
        } else {
            println!("⚠️ [INVOICE] Logo-Pfad existiert nicht: {:?}", logo_path);
            format!(r#"<div class="logo-placeholder">[LOGO]<br>{}</div>"#, &company.company_name)
        }
    } else {
        println!("⚠️ [INVOICE] Kein Logo in Company Settings");
        format!(r#"<div class="logo-placeholder">[LOGO]<br>{}</div>"#, &company.company_name)
    };
    html = html.replace("{{LOGO_HTML}}", &logo_html);

    // ============================================================================
    // HEADER - Invoice Number (EXAKT GLEICH WIE SQLite VERSION)
    // ============================================================================
    let invoice_number = format!("#{}-{:04}",
        chrono::Local::now().format("%Y"),
        b.id
    );
    html = html.replace("{{INVOICE_NUMBER}}", &invoice_number);

    // ============================================================================
    // RECIPIENTS - Gast (EXAKT GLEICH WIE SQLite VERSION)
    // ============================================================================
    let guest_name = format!("{} {}", guest.vorname, guest.nachname);
    html = html.replace("{{GUEST_NAME}}", &guest_name);

    let guest_address = format!(
        "{}<br>{} {}",
        guest.strasse.as_deref().unwrap_or(""),
        guest.plz.as_deref().unwrap_or(""),
        guest.ort.as_deref().unwrap_or("")
    );
    html = html.replace("{{GUEST_ADDRESS}}", &guest_address);

    // Rechnungsempfänger (optional) - EXAKT GLEICH WIE SQLite VERSION
    println!("🔍 [INVOICE] payment_recipient provided: {}", payment_recipient.is_some());

    let invoice_recipient_card = if let Some(recipient) = payment_recipient {
        println!("✅ [INVOICE] External payment recipient: {} ({})",
            recipient.name,
            recipient.city.as_deref().unwrap_or("")
        );

        let recipient_address = format!(
            "{}{}{} {}{}",
            recipient.street.as_ref().map(|s| format!("{}<br>", s)).unwrap_or_default(),
            recipient.plz.as_ref().map(|p| format!("{} ", p)).unwrap_or_default(),
            recipient.city.as_deref().unwrap_or(""),
            if recipient.country.as_deref().unwrap_or("Deutschland") != "Deutschland" {
                format!("<br>{}", recipient.country.as_deref().unwrap_or(""))
            } else {
                "".to_string()
            },
            if let Some(ref contact) = recipient.contact_person {
                format!("<br><small style='color: #64748b;'>Ansprechpartner: {}</small>", contact)
            } else {
                "".to_string()
            }
        );

        let card_html = format!(
            r#"<div class="recipient-card" style="padding: 8px; background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); border-radius: 8px; border: 1px solid #1d4ed8; display: flex; flex-direction: column; justify-content: center; min-height: 85px;">
                <div style="font-size: 8px; color: rgba(255,255,255,0.8); text-transform: uppercase; font-weight: 600; letter-spacing: 0.05em; margin-bottom: 5px;">⚠️ EXTERNE RECHNUNG</div>
                <div style="font-size: 11px; font-weight: bold; color: white; margin-bottom: 3px;">{}{}</div>
                <div style="font-size: 9px; color: rgba(255,255,255,0.9); line-height: 1.3;">{}</div>
                <div style="margin-top: 6px; padding-top: 6px; border-top: 1px solid rgba(255,255,255,0.2); font-size: 8px; color: rgba(255,255,255,0.7); line-height: 1.3;">
                    Diese Rechnung wird an den oben angegebenen externen Empfänger adressiert.
                </div>
            </div>"#,
            recipient.name,
            recipient.company.as_ref().map(|c| format!("<br><span style='font-size: 10px; font-weight: normal;'>{}</span>", c)).unwrap_or_default(),
            recipient_address
        );

        println!("📦 [INVOICE] Generated card HTML ({} bytes)", card_html.len());
        card_html
    } else {
        println!("ℹ️ [INVOICE] payment_recipient is None - no external recipient");
        "".to_string()
    };

    println!("🔄 [INVOICE] Replacing {{{{INVOICE_RECIPIENT_CARD}}}} with content ({} bytes)", invoice_recipient_card.len());
    html = html.replace("{{INVOICE_RECIPIENT_CARD}}", &invoice_recipient_card);
    println!("✅ [INVOICE] Replacement complete");

    // ============================================================================
    // RECIPIENTS - Booking Details (EXAKT GLEICH WIE SQLite VERSION)
    // ============================================================================
    html = html.replace("{{ROOM_NAME}}", &format!("Zimmer {} - {}", room.name, room.gebaeude_typ));
    html = html.replace("{{CHECKIN_DATE}}", &format_german_date(&b.checkin_date));
    html = html.replace("{{CHECKOUT_DATE}}", &format_german_date(&b.checkout_date));
    html = html.replace("{{NIGHTS_COUNT}}", &nights.to_string());

    let guest_count = b.anzahl_gaeste;
    let guest_label = if guest_count == 1 { "Gast" } else { "Gäste" };
    html = html.replace("{{GUEST_COUNT}}", &guest_count.to_string());
    html = html.replace("{{GUEST_COUNT_LABEL}}", guest_label);

    // ============================================================================
    // META - Dates & Booking Number (EXAKT GLEICH WIE SQLite VERSION)
    // ============================================================================
    let invoice_date = chrono::Local::now().format("%d.%m.%Y").to_string();
    html = html.replace("{{INVOICE_DATE}}", &invoice_date);

    let stay_period = format!(
        "{} - {}",
        format_german_date(&b.checkin_date),
        format_german_date(&b.checkout_date)
    );
    html = html.replace("{{STAY_PERIOD}}", &stay_period);

    // Fälligkeitsdatum (EXAKT GLEICH WIE SQLite VERSION)
    let due_date = chrono::Local::now()
        .checked_add_signed(chrono::Duration::days(payment.payment_due_days.unwrap_or(14) as i64))
        .unwrap()
        .format("%d.%m.%Y")
        .to_string();
    html = html.replace("{{DUE_DATE}}", &due_date);

    html = html.replace("{{BOOKING_NUMBER}}", &b.reservierungsnummer);

    // Mitgliedsnummer (optional) - EXAKT GLEICH WIE SQLite VERSION
    let membership_meta = if let Some(ref membership_id) = guest.mitgliedsnummer {
        if !membership_id.is_empty() {
            format!(
                r#"<div class="meta-item">
                    <div class="meta-label">Mitgliedsnummer</div>
                    <div class="meta-value">{}</div>
                </div>"#,
                membership_id
            )
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    html = html.replace("{{MEMBERSHIP_META}}", &membership_meta);

    // ============================================================================
    // SERVICES TABLE (EXAKT GLEICH WIE SQLite VERSION)
    // ============================================================================
    let mut service_rows = Vec::new();
    let mut pos = 1;

    // 1. Zimmerpreis (Übernachtung)
    let grundpreis = b.grundpreis.unwrap_or_else(|| {
        // Fallback für alte Buchungen ohne gespeicherten Grundpreis:
        // Berechne aus aktuellem Zimmerpreis × Nächte
        let price = room.nebensaison_preis.unwrap_or(0.0);
        price * nights as f64
    });
    let room_price_per_night = grundpreis / nights as f64;
    service_rows.push(format!(
        r#"<tr>
            <td>{:02}</td>
            <td>
                <div class="item-description">Übernachtung {}</div>
                <div class="item-details">Zimmer {}, {} Personen</div>
            </td>
            <td>{} Nächte</td>
            <td>{}</td>
            <td>{}</td>
        </tr>"#,
        pos,
        room.gebaeude_typ,
        room.name,
        b.anzahl_gaeste,
        nights,
        format_currency(room_price_per_night),
        format_currency(grundpreis)
    ));
    pos += 1;

    // 2. Endreinigung - NUR anzeigen wenn NICHT schon als Service vorhanden
    // FIX: Verhindert Doppelzählung wenn Endreinigung bereits in Services-Liste ist
    let has_endreinigung_in_services = booking.services.iter()
        .any(|s| s.service_name.to_lowercase().contains("endreinigung")
              || s.service_name.to_lowercase().contains("cleaning"));

    let endreinigung = room.endreinigung.unwrap_or(0.0);
    if endreinigung > 0.0 && !has_endreinigung_in_services {
        service_rows.push(format!(
            r#"<tr>
                <td>{:02}</td>
                <td>
                    <div class="item-description">Endreinigung</div>
                    <div class="item-details">Zimmer {}</div>
                </td>
                <td>1 Pauschal</td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
            pos,
            room.name,
            format_currency(endreinigung),
            format_currency(endreinigung)
        ));
        pos += 1;
    }

    // 3. Services (Zusatzleistungen) - EXAKT GLEICH WIE SQLite VERSION
    for service in &booking.services {
        let quantity_label = "1 Pauschal".to_string();
        let total_price = service.service_price as f64;

        service_rows.push(format!(
            r#"<tr>
                <td>{:02}</td>
                <td>
                    <div class="item-description">{}</div>
                    <div class="item-details">{}</div>
                </td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
            pos,
            service.service_name,
            "", // AdditionalService hat keine description
            quantity_label,
            format_currency(total_price),
            format_currency(total_price)
        ));
        pos += 1;
    }

    html = html.replace("{{SERVICE_ROWS}}", &service_rows.join("\n"));

    // ============================================================================
    // TOTALS - Calculate prices (FIX: Berechne Services direkt!)
    // ============================================================================
    // ⚠️ WICHTIG: Alle Preise in Deutschland sind BRUTTOPREISE (inkl. MwSt.)

    // Services aus booking.services summieren (NICHT aus b.services_preis!)
    let services_from_list: f64 = booking.services.iter()
        .map(|s| s.service_price)
        .sum();

    // Endreinigung NUR hinzufügen wenn NICHT schon in Services enthalten
    // FIX: has_endreinigung_in_services wurde oben bereits ermittelt
    let endreinigung_to_add = if has_endreinigung_in_services {
        0.0  // Schon in services_from_list enthalten
    } else {
        room.endreinigung.unwrap_or(0.0)
    };

    // Gesamte Zusatzleistungen = Services + Endreinigung (wenn nicht bereits enthalten)
    let services_preis = services_from_list + endreinigung_to_add;

    // Subtotal = Übernachtung + Alle Zusatzleistungen
    let subtotal = grundpreis + services_preis; // Brutto (inkl. MwSt.)

    println!("📊 [INVOICE] Price breakdown:");
    println!("   Grundpreis (Übernachtung): {:.2} €", grundpreis);
    println!("   Services aus Liste: {:.2} €", services_from_list);
    println!("   Endreinigung (hinzugefügt): {:.2} € (in Services: {})", endreinigung_to_add, has_endreinigung_in_services);
    println!("   Services gesamt: {:.2} €", services_preis);
    println!("   Subtotal: {:.2} €", subtotal);

    // MwSt.-Anteil berechnen (informativ - bereits im Preis enthalten!)
    let tax_7_base = grundpreis; // Übernachtung = 7% MwSt.
    let tax_19_base = services_preis; // Services = 19% MwSt.

    let tax_7 = tax_7_base / 1.07 * 0.07; // In Brutto enthaltene MwSt.
    let tax_19 = tax_19_base / 1.19 * 0.19; // In Brutto enthaltene MwSt.

    println!("📊 [INVOICE] Tax calculation:");
    println!("   Davon MwSt. 7%: {:.2} €", tax_7);
    println!("   Davon MwSt. 19%: {:.2} €", tax_19);

    // Rabatt-Basis für Discount-Berechnung (wird vor rabatt_preis benötigt)
    let rabatt_basis = pricing_settings.rabatt_basis.clone().unwrap_or("zimmerpreis".to_string());
    let discount_base = if rabatt_basis == "gesamtpreis" {
        subtotal
    } else {
        grundpreis
    };

    // Rabatte + Guthaben (ANGEPASST FÜR PostgreSQL)
    let rabatt_preis = b.rabatt_preis.unwrap_or_else(|| {
        // Fallback für Buchungen mit NULL rabatt_preis:
        // Summe aus Discount-Records berechnen
        booking.discounts.iter().map(|d| {
            d.calculated_amount.unwrap_or_else(|| {
                if d.discount_type == "percent" {
                    discount_base * (d.discount_value / 100.0)
                } else {
                    d.discount_value
                }
            })
        }).sum()
    });
    let credit_used = b.credit_used.unwrap_or(0.0);
    println!("💰 [INVOICE] Credit used for booking {}: {:.2} €", b.id, credit_used);

    // ✅ FIX: MwSt. ist bereits in Subtotal enthalten
    let grand_total = subtotal - rabatt_preis - credit_used;
    println!("💵 [INVOICE] Grand total: {:.2} €", grand_total);

    html = html.replace("{{SUBTOTAL}}", &format_currency(subtotal));

    // Tax Rows (EXAKT GLEICH WIE SQLite VERSION)
    let mut tax_rows = String::new();
    if tax_7 > 0.0 {
        tax_rows.push_str(&format!(
            r#"<div class="total-row tax">
                <span class="total-label">MwSt. 7% (Übernachtung)</span>
                <span>{}</span>
            </div>"#,
            format_currency(tax_7)
        ));
    }
    if tax_19 > 0.0 {
        tax_rows.push_str(&format!(
            r#"<div class="total-row tax">
                <span class="total-label">MwSt. 19% (Zusatzleistungen)</span>
                <span>{}</span>
            </div>"#,
            format_currency(tax_19)
        ));
    }
    html = html.replace("{{TAX_ROWS}}", &tax_rows);

    // Discount Rows - ALLE Rabatte direkt aus der discounts-Tabelle anzeigen
    // Verwende gespeicherten calculated_amount wenn vorhanden (Fix 2: Konsistente Rabatte)
    let mut discount_rows = String::new();

    for d in &booking.discounts {
        // Gespeicherten Betrag verwenden, Fallback: Neuberechnung für alte Buchungen
        let amount = d.calculated_amount.unwrap_or_else(|| {
            if d.discount_type == "percent" {
                discount_base * (d.discount_value / 100.0)
            } else {
                d.discount_value
            }
        });

        if !discount_rows.is_empty() {
            discount_rows.push_str("\n");
        }

        discount_rows.push_str(&format!(
            r#"<div class="total-row" style="color: var(--success); font-size: 13px;">
                <span class="total-label">{}</span>
                <span>- {}</span>
            </div>"#,
            d.discount_name,
            format_currency(amount)
        ));
    }

    // 💰 Credit Row (wenn Guthaben verrechnet wurde)
    if credit_used > 0.0 {
        let credit_row = format!(
            r#"<div class="total-row" style="color: #10b981; font-size: 13px; font-weight: 600;">
                <span class="total-label">💰 Verrechnetes Gast-Guthaben</span>
                <span>- {}</span>
            </div>"#,
            format_currency(credit_used)
        );
        if !discount_rows.is_empty() {
            discount_rows.push_str("\n");
        }
        discount_rows.push_str(&credit_row);
    }

    html = html.replace("{{DISCOUNT_ROWS}}", &discount_rows);
    html = html.replace("{{GRAND_TOTAL}}", &format_currency(grand_total));

    // ============================================================================
    // FOOTER - Payment Info (EXAKT GLEICH WIE SQLite VERSION)
    // ============================================================================
    html = html.replace("{{ACCOUNT_HOLDER}}", payment.account_holder.as_deref().unwrap_or(""));
    html = html.replace("{{BANK_NAME}}", payment.bank_name.as_deref().unwrap_or(""));
    html = html.replace("{{IBAN}}", payment.iban.as_deref().unwrap_or(""));
    html = html.replace("{{BIC}}", payment.bic.as_deref().unwrap_or(""));

    let payment_reference = format!("{} / {}", invoice_number, b.reservierungsnummer);
    html = html.replace("{{PAYMENT_REFERENCE}}", &payment_reference);

    // ============================================================================
    // QR CODE - Generate EPC QR Code für SEPA-Zahlungen (EXAKT GLEICH)
    // ============================================================================
    let qr_code_data_url = generate_payment_qr_code(
        payment.iban.as_deref().unwrap_or(""),
        payment.bic.as_deref().unwrap_or(""),
        payment.account_holder.as_deref().unwrap_or(""),
        grand_total,
        &payment_reference,
    )?;

    // Ersetze [QR CODE] Platzhalter mit echtem QR-Code Bild (EXAKT GLEICH)
    let qr_code_html = format!(
        r#"<img src="{}" alt="QR Code für Zahlung" style="width: 100%; height: 100%; object-fit: contain;" />"#,
        qr_code_data_url
    );

    println!("🔍 [DEBUG] Looking for [QR CODE] placeholder in HTML...");
    println!("🔍 [DEBUG] Placeholder found: {}", html.contains("[QR CODE]"));
    println!("🔍 [DEBUG] QR code data URL length: {} characters", qr_code_data_url.len());

    html = html.replace("[QR CODE]", &qr_code_html);

    println!("🔍 [DEBUG] After replacement, [QR CODE] still present: {}", html.contains("[QR CODE]"));
    println!("✅ [DEBUG] QR Code successfully inserted into HTML");

    println!("✅ Invoice HTML generated successfully (PostgreSQL - PORTED VERSION)");
    Ok(html)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupSettings {
    auto_backup_enabled: bool,
    backup_interval_days: i32,
    backup_path: String,
}

#[tauri::command]
async fn get_backup_settings_command() -> Result<BackupSettings, String> {
    println!("⚙️ Loading backup settings...");

    let settings_path = std::path::Path::new("backup_settings.json");

    if settings_path.exists() {
        // Load existing settings
        let content = std::fs::read_to_string(settings_path)
            .map_err(|e| format!("Fehler beim Lesen der Einstellungen: {}", e))?;

        let settings: BackupSettings = serde_json::from_str(&content)
            .map_err(|e| format!("Fehler beim Parsen der Einstellungen: {}", e))?;

        println!("✅ Backup settings loaded");
        Ok(settings)
    } else {
        // Return default settings
        println!("ℹ️  Using default backup settings");
        Ok(BackupSettings {
            auto_backup_enabled: false,
            backup_interval_days: 7,
            backup_path: "backups".to_string(),
        })
    }
}

#[tauri::command]
async fn get_booking_command(
    pool: State<'_, DbPool>,
    id: i64,
) -> Result<database_pg::BookingWithDetails, String> {
    println!("📋 get_booking_command called for id={}", id);

    let booking = BookingRepository::get_with_details(&pool, id as i32)
        .await
        .map_err(|e| {
            eprintln!("❌ Error loading booking: {}", e);
            e.to_string()
        })?;

    println!("✅ Successfully loaded booking {}", id);
    Ok(booking)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreditTransaction {
    id: i64,
    guest_id: i64,
    booking_id: Option<i64>,
    amount: f64,
    transaction_type: String,
    description: String,
    created_at: String,
}

#[tauri::command]
async fn get_guest_credit_transactions(guest_id: i64, pool: State<'_, DbPool>) -> Result<Vec<CreditTransaction>, String> {
    use crate::database_pg::repositories::GuestCreditRepository;

    println!("💳 get_guest_credit_transactions called for guest_id={}", guest_id);

    let transactions = GuestCreditRepository::get_transactions_by_guest(&pool, guest_id as i32)
        .await
        .map_err(|e| format!("Failed to get transactions: {}", e))?;

    let result: Vec<CreditTransaction> = transactions
        .into_iter()
        .map(|t| CreditTransaction {
            id: t.id as i64,
            guest_id: t.guest_id as i64,
            booking_id: t.booking_id.map(|id| id as i64),
            amount: t.amount,
            transaction_type: t.transaction_type,
            description: t.description.unwrap_or_default(),
            created_at: t.created_at,
        })
        .collect();

    println!("✅ Found {} transactions for guest {}", result.len(), guest_id);
    Ok(result)
}


#[tauri::command]
async fn get_recent_email_logs_command(pool: State<'_, DbPool>, limit: i32) -> Result<Vec<database_pg::EmailLog>, String> {
    // Convert i32 to i64 for PostgreSQL LIMIT parameter
    let limit_i64 = limit as i64;

    let client = pool.get().await
        .map_err(|e| format!("Database pool error: {}", e))?;

    let rows = client
        .query(
            "SELECT id, booking_id, guest_id, template_name, recipient_email,
                    subject, status, error_message, sent_at::text as sent_at
             FROM email_logs
             WHERE sent_at IS NOT NULL
             ORDER BY sent_at DESC
             LIMIT $1",
            &[&limit_i64],
        )
        .await
        .map_err(|e| format!("SQL query error: {}", e))?;

    let logs: Vec<database_pg::EmailLog> = rows
        .into_iter()
        .map(database_pg::EmailLog::from)
        .collect();

    Ok(logs)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionEntry {
    id: i64,
    action: String,
    entity_type: String,
    entity_id: i64,
    timestamp: String,
}

#[tauri::command]
async fn get_recent_transactions_command(
    pool: State<'_, DbPool>,
) -> Result<Vec<TransactionEntry>, String> {
    println!("💳 get_recent_transactions_command called");

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Get recent guest credit transactions (last 100)
    let rows = client
        .query(
            "SELECT id, guest_id, booking_id, transaction_type, amount::text, created_at::text
             FROM guest_credit_transactions
             ORDER BY created_at DESC
             LIMIT 100",
            &[],
        )
        .await
        .map_err(|e| format!("Fehler beim Laden der Transaktionen: {}", e))?;

    let transactions: Vec<TransactionEntry> = rows
        .iter()
        .map(|row| {
            let transaction_type: String = row.get("transaction_type");
            let action = match transaction_type.as_str() {
                "credit" => "Guthaben hinzugefügt",
                "debit" => "Guthaben verwendet",
                "refund" => "Rückerstattung",
                _ => "Transaktion",
            };

            TransactionEntry {
                id: row.get::<_, i32>("id") as i64,
                action: action.to_string(),
                entity_type: "guest_credit".to_string(),
                entity_id: row.get::<_, i32>("guest_id") as i64,
                timestamp: row.get("created_at"),
            }
        })
        .collect();

    println!("✅ Found {} recent transactions", transactions.len());
    Ok(transactions)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReminderSettings {
    auto_send_confirmation: bool,
    days_before_checkin: i32,
    auto_send_reminder: bool,
}

#[tauri::command]
async fn get_reminder_settings_command(pool: State<'_, DbPool>) -> Result<ReminderSettings, String> {
    println!("📧 Loading reminder settings...");

    // Load notification settings from database
    let settings = NotificationSettingsRepository::get(&pool)
        .await
        .map_err(|e| format!("Fehler beim Laden der Einstellungen: {}", e))?;

    Ok(ReminderSettings {
        auto_send_confirmation: false, // TODO: Implement in notification_settings table
        days_before_checkin: settings.payment_reminder_after_days.unwrap_or(3),
        auto_send_reminder: settings.checkin_reminders_enabled.unwrap_or(false),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduledEmail {
    booking_id: i64,
    reservierungsnummer: String,
    guest_name: String,
    guest_email: String,
    email_type: String,
    scheduled_date: String,
    reason: String,
}

#[tauri::command]
async fn get_scheduled_emails(pool: State<'_, DbPool>) -> Result<Vec<ScheduledEmail>, String> {
    println!("╔═══════════════════════════════════════════════════════════");
    println!("║ 📧 get_scheduled_emails BACKEND CALLED");
    println!("╚═══════════════════════════════════════════════════════════");

    println!("🔄 Step 1: Getting database client from pool...");
    let client = pool.get().await
        .map_err(|e| {
            let err_msg = format!("❌ DATABASE POOL ERROR: {}", e);
            println!("{}", err_msg);
            err_msg
        })?;
    println!("✅ Step 1: Database client obtained successfully");

    println!("🔄 Step 2: Executing SQL query...");
    let sql = "SELECT
                se.booking_id,
                b.reservierungsnummer,
                CONCAT(g.vorname, ' ', g.nachname) as guest_name,
                se.recipient_email,
                se.template_name,
                se.scheduled_for::text as scheduled_for,
                se.status,
                b.checkin_date::text as check_in,
                b.checkout_date::text as check_out
             FROM scheduled_emails se
             JOIN bookings b ON se.booking_id = b.id
             JOIN guests g ON b.guest_id = g.id
             WHERE se.status = 'pending'
             ORDER BY se.scheduled_for ASC";

    println!("📝 SQL Query:");
    println!("{}", sql);

    let rows = client
        .query(sql, &[])
        .await
        .map_err(|e| {
            let err_msg = format!("❌ SQL QUERY ERROR: {}", e);
            println!("{}", err_msg);
            println!("🔍 Error details: {:?}", e);
            err_msg
        })?;

    println!("✅ Step 2: Query executed successfully");
    println!("📊 Found {} rows in database", rows.len());

    if rows.is_empty() {
        println!("⚠️  WARNING: No scheduled emails found in database");
        println!("   Check if there are any rows with status='pending' in scheduled_emails table");
        return Ok(vec![]);
    }

    println!("🔄 Step 3: Mapping rows to ScheduledEmail structs...");
    let total_rows = rows.len();
    let result: Vec<ScheduledEmail> = rows
        .into_iter()
        .enumerate()
        .map(|(idx, row)| {
            println!("  📧 Processing email {}/{}...", idx + 1, total_rows);

            println!("    🔍 Extracting booking_id...");
            let booking_id: i32 = row.get("booking_id");
            println!("    ✅ booking_id = {}", booking_id);

            println!("    🔍 Extracting reservierungsnummer...");
            let reservierungsnummer: String = row.get("reservierungsnummer");
            println!("    ✅ reservierungsnummer = {}", reservierungsnummer);

            println!("    🔍 Extracting guest_name...");
            let guest_name: String = row.get("guest_name");
            println!("    ✅ guest_name = {}", guest_name);

            println!("    🔍 Extracting recipient_email...");
            let guest_email: String = row.get("recipient_email");
            println!("    ✅ guest_email = {}", guest_email);

            println!("    🔍 Extracting template_name...");
            let email_type: String = row.get("template_name");
            println!("    ✅ email_type = {}", email_type);

            println!("    🔍 Extracting scheduled_for...");
            let scheduled_for: String = row.get("scheduled_for");
            println!("    ✅ scheduled_for = {}", scheduled_for);

            println!("    🔍 Extracting check_in...");
            let check_in: String = row.get("check_in");
            println!("    ✅ check_in = {}", check_in);

            println!("    🔍 Extracting check_out...");
            let check_out: String = row.get("check_out");
            println!("    ✅ check_out = {}", check_out);

            // Generate reason based on email type and dates
            println!("    🔍 Generating reason from email_type...");
            let reason = match email_type.as_str() {
                "reminder" => format!("Erinnerung 7 Tage vor Anreise ({})", check_in),
                "payment_reminder" => format!("Zahlungserinnerung 14 Tage vor Anreise ({})", check_in),
                "invoice" => format!("Rechnung nach Abreise ({})", check_out),
                _ => format!("Geplanter Email-Versand"),
            };
            println!("    ✅ reason = {}", reason);

            let scheduled_email = ScheduledEmail {
                booking_id: booking_id as i64,
                reservierungsnummer,
                guest_name,
                guest_email,
                email_type,
                scheduled_date: scheduled_for,
                reason,
            };

            println!("    ✅ ScheduledEmail struct created successfully");
            scheduled_email
        })
        .collect();

    println!("✅ Step 3: Successfully mapped all rows");
    println!("📦 Returning {} ScheduledEmail objects to frontend", result.len());
    println!("╔═══════════════════════════════════════════════════════════");
    println!("║ ✅ get_scheduled_emails BACKEND COMPLETE");
    println!("╚═══════════════════════════════════════════════════════════");

    Ok(result)
}

#[tauri::command]
async fn debug_scheduled_emails_pg(pool: State<'_, DbPool>) -> Result<String, String> {
    let client = pool.get().await
        .map_err(|e| format!("DB Pool Error: {}", e))?;

    // 1. Check if table exists
    let table_check = client.query(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'scheduled_emails')",
        &[]
    ).await.map_err(|e| format!("Table check error: {}", e))?;
    let table_exists: bool = table_check[0].get(0);

    if !table_exists {
        return Ok("❌ Table 'scheduled_emails' does not exist!".to_string());
    }

    // 2. Count total rows
    let count_result = client.query("SELECT COUNT(*) FROM scheduled_emails", &[])
        .await.map_err(|e| format!("Count error: {}", e))?;
    let total_count: i64 = count_result[0].get(0);

    // 3. Count by status
    let status_result = client.query(
        "SELECT status, COUNT(*) FROM scheduled_emails GROUP BY status",
        &[]
    ).await.map_err(|e| format!("Status count error: {}", e))?;

    let mut status_counts = String::new();
    for row in status_result {
        let status: String = row.get(0);
        let count: i64 = row.get(1);
        status_counts.push_str(&format!("\n  - {}: {}", status, count));
    }

    // 4. Get sample rows
    let sample_result = client.query(
        "SELECT id, booking_id, template_name, status, scheduled_for::text FROM scheduled_emails LIMIT 5",
        &[]
    ).await.map_err(|e| format!("Sample query error: {}", e))?;

    let mut samples = String::new();
    for row in sample_result {
        let id: i32 = row.get(0);
        let booking_id: i32 = row.get(1);
        let template: String = row.get(2);
        let status: String = row.get(3);
        let scheduled: String = row.get(4);
        samples.push_str(&format!("\n  - ID={}, Booking={}, Template={}, Status={}, Scheduled={}",
            id, booking_id, template, status, scheduled));
    }

    // 5. Check trigger exists
    let trigger_check = client.query(
        "SELECT EXISTS (SELECT FROM pg_trigger WHERE tgname = 'trg_schedule_reminder_emails')",
        &[]
    ).await.map_err(|e| format!("Trigger check error: {}", e))?;
    let trigger_exists: bool = trigger_check[0].get(0);

    // 6. Check notification settings
    let settings_check = client.query(
        "SELECT checkin_reminders_enabled FROM notification_settings WHERE id = 1",
        &[]
    ).await.map_err(|e| format!("Settings check error: {}", e))?;
    let reminders_enabled: bool = if settings_check.is_empty() {
        false
    } else {
        settings_check[0].get(0)
    };

    Ok(format!(
        "📊 Scheduled Emails Debug Info:\n\
        ✅ Table exists: {}\n\
        📈 Total rows: {}\n\
        📊 By status:{}\n\
        🔔 Trigger exists: {}\n\
        ⚙️  Reminders enabled: {}\n\
        📧 Sample rows:{}\n",
        table_exists, total_count, status_counts, trigger_exists, reminders_enabled, samples
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupEntry {
    name: String,
    size: u64,
    created_at: String,
}

#[tauri::command]
async fn list_backups_command() -> Result<Vec<BackupEntry>, String> {
    println!("📂 Listing backup files...");

    let backup_dir = std::path::Path::new("../backups");

    // Create directory if it doesn't exist
    if !backup_dir.exists() {
        println!("ℹ️  Backup directory doesn't exist yet");
        return Ok(vec![]);
    }

    let mut backups: Vec<BackupEntry> = vec![];

    // Read directory entries
    let entries = std::fs::read_dir(backup_dir)
        .map_err(|e| format!("Fehler beim Lesen des Backup-Ordners: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Fehler beim Lesen eines Eintrags: {}", e))?;
        let path = entry.path();

        // Only include .sql files
        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
            let metadata = std::fs::metadata(&path)
                .map_err(|e| format!("Fehler beim Lesen der Metadaten: {}", e))?;

            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let created_at = metadata.modified()
                .map(|time| {
                    let datetime: chrono::DateTime<chrono::Local> = time.into();
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|_| "unknown".to_string());

            backups.push(BackupEntry {
                name,
                size: metadata.len(),
                created_at,
            });
        }
    }

    // Sort by creation time (newest first)
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    println!("✅ Found {} backup files", backups.len());
    Ok(backups)
}

#[tauri::command]
async fn mark_invoice_sent_command(
    pool: State<'_, DbPool>,
    booking_id: i64,
) -> Result<(), String> {
    println!("📧 mark_invoice_sent_command called for booking_id={}", booking_id);

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Get guest email from booking
    let guest_email: Option<String> = client
        .query_one(
            "SELECT g.email FROM bookings b
             JOIN guests g ON b.guest_id = g.id
             WHERE b.id = $1",
            &[&(booking_id as i32)],
        )
        .await
        .ok()
        .and_then(|row| row.get::<_, Option<String>>(0));

    // Update booking with invoice sent timestamp and recipient email
    client
        .execute(
            "UPDATE bookings
             SET rechnung_versendet_am = $1,
                 rechnung_versendet_an = $2
             WHERE id = $3",
            &[&now, &guest_email, &(booking_id as i32)],
        )
        .await
        .map_err(|e| format!("Fehler beim Aktualisieren: {}", e))?;

    println!("✅ Invoice marked as sent for booking {}", booking_id);
    Ok(())
}

#[tauri::command]
async fn migrate_to_price_list_2025_command(
    pool: State<'_, DbPool>,
) -> Result<String, String> {
    println!("🔄 migrate_to_price_list_2025_command called");

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Update pricing settings for 2025
    // Example: Set new season dates for 2025
    let updated_rows = client
        .execute(
            "UPDATE pricing_settings
             SET hauptsaison_start = '2025-06-01',
                 hauptsaison_ende = '2025-09-30',
                 hauptsaison_aktiv = TRUE,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = 1",
            &[],
        )
        .await
        .map_err(|e| format!("Fehler beim Aktualisieren der Preisliste: {}", e))?;

    if updated_rows == 0 {
        // Create default pricing settings if not exists
        client
            .execute(
                "INSERT INTO pricing_settings (id, hauptsaison_start, hauptsaison_ende, hauptsaison_aktiv, mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent)
                 VALUES (1, '2025-06-01', '2025-09-30', TRUE, TRUE, 10.0)
                 ON CONFLICT (id) DO NOTHING",
                &[],
            )
            .await
            .map_err(|e| format!("Fehler beim Erstellen der Preisliste: {}", e))?;
    }

    println!("✅ Price list migrated to 2025");
    Ok("Preisliste erfolgreich auf 2025 migriert".to_string())
}

#[tauri::command]
async fn fix_booking_status_values_command(pool: State<'_, DbPool>) -> Result<String, String> {
    println!("🔄 Fixing booking status values (old SQLite -> new PostgreSQL)...");

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Update old status values to new ones
    let gebucht_count = client
        .execute("UPDATE bookings SET status = 'reserviert' WHERE status = 'gebucht'", &[])
        .await
        .map_err(|e| format!("Fehler beim Update: {}", e))?;

    let aktiv_count = client
        .execute("UPDATE bookings SET status = 'eingecheckt' WHERE status = 'aktiv'", &[])
        .await
        .map_err(|e| format!("Fehler beim Update: {}", e))?;

    let abgeschlossen_count = client
        .execute("UPDATE bookings SET status = 'ausgecheckt' WHERE status = 'abgeschlossen'", &[])
        .await
        .map_err(|e| format!("Fehler beim Update: {}", e))?;

    // Add CHECK constraint to ensure only valid status values
    // First drop any existing constraint if it exists
    client
        .execute("ALTER TABLE bookings DROP CONSTRAINT IF EXISTS bookings_status_check", &[])
        .await
        .map_err(|e| format!("Fehler beim Löschen der Constraint: {}", e))?;

    // Add new constraint with correct values
    client
        .execute(
            "ALTER TABLE bookings
             ADD CONSTRAINT bookings_status_check
             CHECK (status IN ('reserviert', 'bestaetigt', 'eingecheckt', 'ausgecheckt', 'storniert'))",
            &[],
        )
        .await
        .map_err(|e| format!("Fehler beim Erstellen der Constraint: {}", e))?;

    let total_updated = gebucht_count + aktiv_count + abgeschlossen_count;
    let message = format!(
        "✅ Status-Werte korrigiert:\n  • {} 'gebucht' → 'reserviert'\n  • {} 'aktiv' → 'eingecheckt'\n  • {} 'abgeschlossen' → 'ausgecheckt'\n  • CHECK constraint hinzugefügt",
        gebucht_count, aktiv_count, abgeschlossen_count
    );

    println!("{}", message);
    println!("✅ Total {} bookings updated", total_updated);
    Ok(message)
}

#[tauri::command]
async fn fix_deadlock_triggers_command(pool: State<'_, DbPool>) -> Result<String, String> {
    println!("🔄 Fixing deadlock issues in database triggers...");

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Read migration file
    let migration_sql = include_str!("../../migrations/005_fix_deadlock_triggers.sql");

    // Execute migration
    client
        .batch_execute(migration_sql)
        .await
        .map_err(|e| format!("Fehler beim Ausführen der Migration: {}", e))?;

    let message = "✅ Deadlock-Fix erfolgreich:\n  • FOR SHARE Locks hinzugefügt\n  • Unique Constraint für scheduled_emails\n  • Exception Handling verbessert\n  • ON CONFLICT Handling optimiert".to_string();

    println!("{}", message);
    Ok(message)
}

#[tauri::command]
async fn open_backup_folder_command() -> Result<(), String> {
    use std::process::Command;

    println!("📂 Opening backup folder...");

    let backup_dir = std::path::Path::new("../backups");

    // Create directory if it doesn't exist
    if !backup_dir.exists() {
        std::fs::create_dir_all(backup_dir)
            .map_err(|e| format!("Fehler beim Erstellen des Backup-Ordners: {}", e))?;
    }

    // Open folder in file explorer (cross-platform)
    #[cfg(target_os = "macos")]
    Command::new("open")
        .arg(backup_dir)
        .spawn()
        .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;

    #[cfg(target_os = "windows")]
    Command::new("explorer")
        .arg(backup_dir)
        .spawn()
        .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;

    #[cfg(target_os = "linux")]
    Command::new("xdg-open")
        .arg(backup_dir)
        .spawn()
        .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;

    println!("✅ Backup folder opened");
    Ok(())
}

#[tauri::command]
async fn open_invoices_folder_command(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;

    // Get app data directory
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("App directory error: {}", e))?;
    let invoices_dir = app_data_dir.join("invoices");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&invoices_dir)
        .map_err(|e| format!("Create invoices dir error: {}", e))?;

    let folder_path = invoices_dir.to_string_lossy().to_string();
    println!("📂 open_invoices_folder_command: opening {}", folder_path);

    // Open with system file manager
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen des Ordners: {}", e))?;
    }

    println!("✅ Ordner geöffnet: {}", folder_path);
    Ok(())
}

#[tauri::command]
async fn open_pdf_file_command(path: String) -> Result<(), String> {
    println!("📄 open_pdf_file_command: opening {}", path);

    // Check if file exists
    if !std::path::Path::new(&path).exists() {
        return Err(format!("PDF-Datei nicht gefunden: {}", path));
    }

    // Open with system default application
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen der PDF: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen der PDF: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen der PDF: {}", e))?;
    }

    println!("✅ PDF geöffnet: {}", path);
    Ok(())
}

#[tauri::command]
async fn open_putzplan_folder() -> Result<(), String> {
    println!("📁 Opening Putzplan folder...");

    let putzplan_dir = std::path::Path::new("putzplan");

    // Create if doesn't exist
    if !putzplan_dir.exists() {
        std::fs::create_dir_all(putzplan_dir)
            .map_err(|e| format!("Fehler beim Erstellen des Ordners: {}", e))?;
    }

    // Open folder in file explorer
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(putzplan_dir).spawn()
        .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(putzplan_dir).spawn()
        .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(putzplan_dir).spawn()
        .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;

    println!("✅ Putzplan folder opened");
    Ok(())
}

#[tauri::command]
async fn restore_backup_command(backup_name: String, _pool: State<'_, DbPool>) -> Result<(), String> {
    use std::process::Command;

    println!("⚠️ RESTORING BACKUP: {}", backup_name);
    println!("⚠️ WARNING: This will replace all current database data!");

    // Security checks
    if !backup_name.ends_with(".sql") {
        return Err("Ungültiger Dateiname. Nur .sql Dateien können wiederhergestellt werden.".to_string());
    }

    if backup_name.contains("..") || backup_name.contains("/") || backup_name.contains("\\") {
        return Err("Ungültiger Dateiname. Pfad-Traversal nicht erlaubt.".to_string());
    }

    let backup_path = std::path::Path::new("../backups").join(&backup_name);

    // Check if backup file exists
    if !backup_path.exists() {
        return Err(format!("Backup-Datei '{}' nicht gefunden", backup_name));
    }

    println!("📁 Restoring from: {:?}", backup_path);

    // Execute psql restore command
    // WARNING: This will drop and recreate all tables!
    let output = Command::new("psql")
        .args(&[
            "--host=213.215.205.44",
            "--port=6432",
            "--username=postgres",
            "--dbname=postgres",
            "--file", backup_path.to_str().unwrap(),
        ])
        .env("PGPASSWORD", "feggmaxi1912")
        .output()
        .map_err(|e| format!("Fehler beim Ausführen von psql: {}. Stelle sicher, dass psql installiert ist.", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("psql restore fehlgeschlagen: {}", error));
    }

    println!("✅ Backup '{}' erfolgreich wiederhergestellt", backup_name);
    println!("⚠️ Bitte Anwendung neu starten für vollständige Aktualisierung");

    Ok(())
}

#[tauri::command]
async fn save_backup_settings_command(settings: BackupSettings) -> Result<(), String> {
    println!("💾 Saving backup settings...");

    let settings_path = std::path::Path::new("backup_settings.json");

    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Fehler beim Serialisieren: {}", e))?;

    std::fs::write(settings_path, json)
        .map_err(|e| format!("Fehler beim Speichern der Einstellungen: {}", e))?;

    println!("✅ Backup settings saved");
    Ok(())
}

#[tauri::command]
async fn save_reminder_settings_command(
    pool: State<'_, DbPool>,
    settings: ReminderSettings
) -> Result<(), String> {
    println!("📧 Saving reminder settings...");

    // Load current notification settings
    let mut current = NotificationSettingsRepository::get(&pool)
        .await
        .map_err(|e| format!("Fehler beim Laden der Einstellungen: {}", e))?;

    // Update reminder-specific fields
    current.checkin_reminders_enabled = Some(settings.auto_send_reminder);
    current.payment_reminder_after_days = Some(settings.days_before_checkin);

    // Save updated settings
    NotificationSettingsRepository::update(&pool, &current)
        .await
        .map_err(|e| format!("Fehler beim Speichern: {}", e))?;

    println!("✅ Reminder settings saved successfully");
    Ok(())
}

/// Helper function to create SMTP transport from email config
fn create_smtp_transport(
    smtp_server: &str,
    smtp_port: u16,
    smtp_username: &str,
    smtp_password: &str,
    use_tls: bool,
) -> Result<SmtpTransport, String> {
    let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());

    if smtp_port == 465 {
        SmtpTransport::relay(smtp_server)
            .map(|b| b.credentials(creds).port(smtp_port).build())
            .map_err(|e| format!("SMTP-Fehler: {}", e))
    } else if use_tls {
        SmtpTransport::starttls_relay(smtp_server)
            .map(|b| b.credentials(creds).port(smtp_port).build())
            .map_err(|e| format!("SMTP-Fehler: {}", e))
    } else {
        Ok(SmtpTransport::builder_dangerous(smtp_server)
            .credentials(creds)
            .port(smtp_port)
            .build())
    }
}

/// Helper function to send an email using the configured SMTP settings
async fn send_email_helper(
    pool: &DbPool,
    to_email: &str,
    to_name: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    // Load email config
    let config = EmailConfigRepository::get(pool)
        .await
        .map_err(|e| format!("Email-Konfiguration fehlt: {}", e))?;

    let smtp_server = config.smtp_server.ok_or("SMTP-Server nicht konfiguriert")?;
    let smtp_port = config.smtp_port.ok_or("SMTP-Port nicht konfiguriert")? as u16;
    let smtp_username = config.smtp_username.ok_or("SMTP-Benutzername nicht konfiguriert")?;
    let smtp_password = config.smtp_password.ok_or("SMTP-Passwort nicht konfiguriert")?;
    let from_email = config.from_email.ok_or("Absender-Email nicht konfiguriert")?;
    let from_name = config.from_name.unwrap_or_else(|| "DPolG Buchungssystem".to_string());
    let use_tls = config.use_tls.unwrap_or(1) == 1;

    // Build email
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse().map_err(|e| format!("Ungültige Absender-Email: {}", e))?)
        .to(format!("{} <{}>", to_name, to_email).parse().map_err(|e| format!("Ungültige Empfänger-Email: {}", e))?)
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?;

    // Send
    let mailer = create_smtp_transport(&smtp_server, smtp_port, &smtp_username, &smtp_password, use_tls)?;
    mailer.send(&email).map_err(|e| format!("Email-Versand fehlgeschlagen: {}", e))?;

    Ok(())
}

/// Replace template placeholders with actual booking data
fn replace_template_placeholders(template: &str, booking: &database_pg::Booking, guest: &database_pg::Guest, room_name: &str) -> String {
    template
        .replace("{{gast_name}}", &format!("{} {}", guest.vorname, guest.nachname))
        .replace("{{gast_vorname}}", &guest.vorname)
        .replace("{{gast_nachname}}", &guest.nachname)
        .replace("{{zimmer}}", room_name)
        .replace("{{checkin}}", &booking.checkin_date)
        .replace("{{checkout}}", &booking.checkout_date)
        .replace("{{naechte}}", &booking.anzahl_naechte.unwrap_or(1).to_string())
        .replace("{{gesamtpreis}}", &format!("{:.2} €", booking.gesamtpreis).replace(".", ","))
        .replace("{{buchungs_id}}", &booking.id.to_string())
        .replace("{{status}}", &booking.status)
}

#[tauri::command]
async fn send_cancellation_email_command(pool: State<'_, DbPool>, booking_id: i64) -> Result<String, String> {
    println!("📧 Sending cancellation email for booking {}", booking_id);

    // Load booking
    let booking = BookingRepository::get_by_id(&pool, booking_id as i32)
        .await
        .map_err(|e| format!("Buchung nicht gefunden: {}", e))?;

    // Load guest
    let guest = GuestRepository::get_by_id(&pool, booking.guest_id)
        .await
        .map_err(|e| format!("Gast nicht gefunden: {}", e))?;

    let guest_email = if guest.email.is_empty() {
        return Err("Gast hat keine Email-Adresse".to_string());
    } else {
        guest.email.clone()
    };

    // Load room
    let room = RoomRepository::get_by_id(&pool, booking.room_id)
        .await
        .map_err(|e| format!("Zimmer nicht gefunden: {}", e))?;

    // Try to load template, fallback to default
    let (subject, body) = match EmailTemplateRepository::get_by_name(&pool, "stornierung".to_string()).await {
        Ok(template) => (
            replace_template_placeholders(&template.subject, &booking, &guest, &room.name),
            replace_template_placeholders(&template.body, &booking, &guest, &room.name),
        ),
        Err(_) => {
            // Default template
            let subject = format!("Stornierungsbestätigung - Buchung #{}", booking_id);
            let body = format!(
                "Sehr geehrte/r {} {},\n\n\
                hiermit bestätigen wir die Stornierung Ihrer Buchung:\n\n\
                Buchungsnummer: {}\n\
                Zimmer: {}\n\
                Zeitraum: {} - {}\n\n\
                Bei Fragen stehen wir Ihnen gerne zur Verfügung.\n\n\
                Mit freundlichen Grüßen,\n\
                Ihr DPolG Buchungsteam",
                guest.vorname, guest.nachname, booking_id, room.name, booking.checkin_date, booking.checkout_date
            );
            (subject, body)
        }
    };

    // Send email
    let send_result = send_email_helper(&pool, &guest_email, &format!("{} {}", guest.vorname, guest.nachname), &subject, &body).await;

    // Log email send result
    {
        use crate::database_pg::repositories::EmailLogRepository;
        let (status, error_msg) = match &send_result {
            Ok(_) => ("gesendet".to_string(), None),
            Err(e) => ("fehler".to_string(), Some(e.clone())),
        };
        let _ = EmailLogRepository::create(
            &pool,
            Some(booking_id as i32),
            guest.id,
            "cancellation".to_string(),
            guest_email.clone(),
            subject.clone(),
            status,
            error_msg,
        ).await;
    }

    send_result?;

    println!("✅ Cancellation email sent to {}", guest_email);
    Ok(format!("Stornierungsbestätigung an {} gesendet", guest_email))
}

#[tauri::command]
async fn send_invoice_email_command(pool: State<'_, DbPool>, booking_id: i64) -> Result<String, String> {
    println!("📧 Sending invoice email for booking {}", booking_id);

    // Load booking
    let booking = BookingRepository::get_by_id(&pool, booking_id as i32)
        .await
        .map_err(|e| format!("Buchung nicht gefunden: {}", e))?;

    // Load guest
    let guest = GuestRepository::get_by_id(&pool, booking.guest_id)
        .await
        .map_err(|e| format!("Gast nicht gefunden: {}", e))?;

    let guest_email = if guest.email.is_empty() {
        return Err("Gast hat keine Email-Adresse".to_string());
    } else {
        guest.email.clone()
    };

    // Load room
    let room = RoomRepository::get_by_id(&pool, booking.room_id)
        .await
        .map_err(|e| format!("Zimmer nicht gefunden: {}", e))?;

    // Try to load template, fallback to default
    let (subject, body) = match EmailTemplateRepository::get_by_name(&pool, "rechnung".to_string()).await {
        Ok(template) => (
            replace_template_placeholders(&template.subject, &booking, &guest, &room.name),
            replace_template_placeholders(&template.body, &booking, &guest, &room.name),
        ),
        Err(_) => {
            // Default template
            let subject = format!("Rechnung zu Ihrer Buchung #{}", booking_id);
            let body = format!(
                "Sehr geehrte/r {} {},\n\n\
                anbei erhalten Sie die Rechnung zu Ihrer Buchung:\n\n\
                Buchungsnummer: {}\n\
                Zimmer: {}\n\
                Zeitraum: {} - {} ({} Nächte)\n\
                Gesamtbetrag: {:.2} €\n\n\
                Die Rechnung finden Sie im Anhang dieser Email.\n\n\
                Mit freundlichen Grüßen,\n\
                Ihr DPolG Buchungsteam",
                guest.vorname, guest.nachname, booking_id, room.name,
                booking.checkin_date, booking.checkout_date, booking.anzahl_naechte.unwrap_or(1),
                booking.gesamtpreis
            );
            (subject, body)
        }
    };

    // Send email
    let send_result = send_email_helper(&pool, &guest_email, &format!("{} {}", guest.vorname, guest.nachname), &subject, &body).await;

    // Log email send result
    {
        use crate::database_pg::repositories::EmailLogRepository;
        let (status, error_msg) = match &send_result {
            Ok(_) => ("gesendet".to_string(), None),
            Err(e) => ("fehler".to_string(), Some(e.clone())),
        };
        let _ = EmailLogRepository::create(
            &pool,
            Some(booking_id as i32),
            guest.id,
            "invoice".to_string(),
            guest_email.clone(),
            subject.clone(),
            status,
            error_msg,
        ).await;
    }

    send_result?;

    println!("✅ Invoice email sent to {}", guest_email);
    Ok(format!("Rechnung an {} gesendet", guest_email))
}

#[tauri::command]
async fn send_reminder_email_command(pool: State<'_, DbPool>, booking_id: i64) -> Result<String, String> {
    println!("📧 Sending reminder email for booking {}", booking_id);

    // Load booking
    let booking = BookingRepository::get_by_id(&pool, booking_id as i32)
        .await
        .map_err(|e| format!("Buchung nicht gefunden: {}", e))?;

    // Load guest
    let guest = GuestRepository::get_by_id(&pool, booking.guest_id)
        .await
        .map_err(|e| format!("Gast nicht gefunden: {}", e))?;

    let guest_email = if guest.email.is_empty() {
        return Err("Gast hat keine Email-Adresse".to_string());
    } else {
        guest.email.clone()
    };

    // Load room
    let room = RoomRepository::get_by_id(&pool, booking.room_id)
        .await
        .map_err(|e| format!("Zimmer nicht gefunden: {}", e))?;

    // Try to load template, fallback to default
    let (subject, body) = match EmailTemplateRepository::get_by_name(&pool, "erinnerung".to_string()).await {
        Ok(template) => (
            replace_template_placeholders(&template.subject, &booking, &guest, &room.name),
            replace_template_placeholders(&template.body, &booking, &guest, &room.name),
        ),
        Err(_) => {
            // Default template
            let subject = format!("Erinnerung: Ihre Buchung #{} steht bevor", booking_id);
            let body = format!(
                "Sehr geehrte/r {} {},\n\n\
                wir möchten Sie an Ihre bevorstehende Buchung erinnern:\n\n\
                Buchungsnummer: {}\n\
                Zimmer: {}\n\
                Check-in: {}\n\
                Check-out: {}\n\n\
                Wir freuen uns auf Ihren Besuch!\n\n\
                Mit freundlichen Grüßen,\n\
                Ihr DPolG Buchungsteam",
                guest.vorname, guest.nachname, booking_id, room.name, booking.checkin_date, booking.checkout_date
            );
            (subject, body)
        }
    };

    // Send email
    let send_result = send_email_helper(&pool, &guest_email, &format!("{} {}", guest.vorname, guest.nachname), &subject, &body).await;

    // Log email send result
    {
        use crate::database_pg::repositories::EmailLogRepository;
        let (status, error_msg) = match &send_result {
            Ok(_) => ("gesendet".to_string(), None),
            Err(e) => ("fehler".to_string(), Some(e.clone())),
        };
        let _ = EmailLogRepository::create(
            &pool,
            Some(booking_id as i32),
            guest.id,
            "reminder".to_string(),
            guest_email.clone(),
            subject.clone(),
            status,
            error_msg,
        ).await;
    }

    send_result?;

    println!("✅ Reminder email sent to {}", guest_email);
    Ok(format!("Erinnerung an {} gesendet", guest_email))
}

#[tauri::command]
async fn sync_affected_dates(booking_id: i64, old_checkout: String, new_checkout: String, pool: State<'_, DbPool>) -> Result<(), String> {
    use crate::turso_sync;

    println!("🔄 sync_affected_dates: booking_id={}, old={}, new={}", booking_id, old_checkout, new_checkout);

    // Cleaning tasks are auto-updated by triggers in PostgreSQL
    println!("✅ PostgreSQL: Cleaning tasks auto-updated by database triggers");

    // Determine affected date range
    let min_date = if old_checkout < new_checkout { &old_checkout } else { &new_checkout };
    let max_date = if old_checkout > new_checkout { &old_checkout } else { &new_checkout };

    println!("📅 Syncing affected dates: {} to {}", min_date, max_date);

    // Sync affected dates to Turso
    match turso_sync::sync_tasks_to_turso(&pool, min_date.to_string(), max_date.to_string()).await {
        Ok(count) => println!("✅ Turso: {} affected tasks synced", count),
        Err(e) => println!("⚠️ Turso sync failed (non-critical): {}", e),
    }

    Ok(())
}

#[tauri::command]
async fn test_email_connection_command(pool: State<'_, DbPool>) -> Result<String, String> {
    println!("📧 Testing email connection...");

    // Load email config from database
    let config = EmailConfigRepository::get(&pool)
        .await
        .map_err(|e| format!("Fehler beim Laden der Email-Konfiguration: {}", e))?;

    // Validate required fields
    let smtp_server = config.smtp_server.ok_or("SMTP-Server ist nicht konfiguriert")?;
    let smtp_port = config.smtp_port.ok_or("SMTP-Port ist nicht konfiguriert")? as u16;
    let smtp_username = config.smtp_username.ok_or("SMTP-Benutzername ist nicht konfiguriert")?;
    let smtp_password = config.smtp_password.ok_or("SMTP-Passwort ist nicht konfiguriert")?;
    let use_tls = config.use_tls.unwrap_or(1) == 1;

    println!("📧 Connecting to {} (port {}, TLS: {})", smtp_server, smtp_port, use_tls);

    // Create credentials
    let creds = Credentials::new(smtp_username.clone(), smtp_password);

    // Build SMTP transport based on port and TLS setting
    let mailer_result = if smtp_port == 465 {
        // Port 465: Implicit TLS (SMTPS)
        SmtpTransport::relay(&smtp_server)
            .map(|builder| {
                builder
                    .credentials(creds)
                    .port(smtp_port)
                    .build()
            })
    } else if use_tls {
        // Port 587: STARTTLS
        SmtpTransport::starttls_relay(&smtp_server)
            .map(|builder| {
                builder
                    .credentials(creds)
                    .port(smtp_port)
                    .build()
            })
    } else {
        // No TLS (not recommended)
        Ok(SmtpTransport::builder_dangerous(&smtp_server)
            .credentials(creds)
            .port(smtp_port)
            .build())
    };

    let mailer = mailer_result.map_err(|e| format!("Fehler beim Erstellen der SMTP-Verbindung: {}", e))?;

    // Test the connection
    match mailer.test_connection() {
        Ok(true) => {
            println!("✅ Email connection test successful!");
            Ok("Verbindung erfolgreich! SMTP-Server ist erreichbar und Anmeldedaten sind korrekt.".to_string())
        }
        Ok(false) => {
            println!("❌ Email connection test failed: connection returned false");
            Err("Verbindung fehlgeschlagen: Server nicht erreichbar oder Anmeldedaten ungültig.".to_string())
        }
        Err(e) => {
            println!("❌ Email connection test error: {}", e);
            // Provide more helpful error messages
            let error_msg = format!("{}", e);
            if error_msg.contains("authentication") || error_msg.contains("535") {
                Err(format!("Authentifizierung fehlgeschlagen: Bitte prüfen Sie Benutzername und Passwort. Bei Gmail benötigen Sie ein App-Passwort."))
            } else if error_msg.contains("timeout") || error_msg.contains("timed out") {
                Err(format!("Verbindungszeitüberschreitung: Server {} ist nicht erreichbar. Prüfen Sie die Serveradresse und den Port.", smtp_server))
            } else if error_msg.contains("certificate") || error_msg.contains("TLS") {
                Err(format!("TLS/SSL-Fehler: Zertifikatsproblem mit {}. Versuchen Sie einen anderen Port.", smtp_server))
            } else if error_msg.contains("connection refused") {
                Err(format!("Verbindung abgelehnt: Port {} auf {} ist nicht erreichbar.", smtp_port, smtp_server))
            } else {
                Err(format!("Verbindungsfehler: {}", e))
            }
        }
    }
}

#[tauri::command]
async fn send_test_email_command(pool: State<'_, DbPool>, recipient_email: String) -> Result<String, String> {
    println!("📧 Sending test email to: {}", recipient_email);

    // Load email config from database
    let config = EmailConfigRepository::get(&pool)
        .await
        .map_err(|e| format!("Fehler beim Laden der Email-Konfiguration: {}", e))?;

    // Validate required fields
    let smtp_server = config.smtp_server.ok_or("SMTP-Server ist nicht konfiguriert")?;
    let smtp_port = config.smtp_port.ok_or("SMTP-Port ist nicht konfiguriert")? as u16;
    let smtp_username = config.smtp_username.ok_or("SMTP-Benutzername ist nicht konfiguriert")?;
    let smtp_password = config.smtp_password.ok_or("SMTP-Passwort ist nicht konfiguriert")?;
    let from_email = config.from_email.ok_or("Absender-Email ist nicht konfiguriert")?;
    let from_name = config.from_name.unwrap_or_else(|| "DPolG Buchungssystem".to_string());
    let use_tls = config.use_tls.unwrap_or(1) == 1;

    // Build email message
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse().map_err(|e| format!("Ungültige Absender-Email: {}", e))?)
        .to(recipient_email.parse().map_err(|e| format!("Ungültige Empfänger-Email: {}", e))?)
        .subject("Test-Email vom DPolG Buchungssystem")
        .body(format!(
            "Dies ist eine Test-Email vom DPolG Buchungssystem.\n\n\
            Wenn Sie diese Email erhalten haben, ist Ihre Email-Konfiguration korrekt eingerichtet.\n\n\
            Konfigurierte Einstellungen:\n\
            - SMTP-Server: {}\n\
            - Port: {}\n\
            - TLS: {}\n\n\
            Mit freundlichen Grüßen,\n\
            Ihr DPolG Buchungssystem",
            smtp_server,
            smtp_port,
            if use_tls { "Aktiviert" } else { "Deaktiviert" }
        ))
        .map_err(|e| format!("Fehler beim Erstellen der Email: {}", e))?;

    // Create credentials
    let creds = Credentials::new(smtp_username, smtp_password);

    // Build SMTP transport based on port and TLS setting
    let mailer_result = if smtp_port == 465 {
        // Port 465: Implicit TLS (SMTPS)
        SmtpTransport::relay(&smtp_server)
            .map(|builder| {
                builder
                    .credentials(creds)
                    .port(smtp_port)
                    .build()
            })
    } else if use_tls {
        // Port 587: STARTTLS
        SmtpTransport::starttls_relay(&smtp_server)
            .map(|builder| {
                builder
                    .credentials(creds)
                    .port(smtp_port)
                    .build()
            })
    } else {
        // No TLS (not recommended)
        Ok(SmtpTransport::builder_dangerous(&smtp_server)
            .credentials(creds)
            .port(smtp_port)
            .build())
    };

    let mailer = mailer_result.map_err(|e| format!("Fehler beim Erstellen der SMTP-Verbindung: {}", e))?;

    // Send the email
    match mailer.send(&email) {
        Ok(_) => {
            println!("✅ Test email sent successfully to {}", recipient_email);
            Ok(format!("Test-Email erfolgreich an {} gesendet!", recipient_email))
        }
        Err(e) => {
            println!("❌ Failed to send test email: {}", e);
            let error_msg = format!("{}", e);
            if error_msg.contains("authentication") || error_msg.contains("535") {
                Err("Authentifizierung fehlgeschlagen: Bitte prüfen Sie Benutzername und Passwort.".to_string())
            } else {
                Err(format!("Email-Versand fehlgeschlagen: {}", e))
            }
        }
    }
}

#[tauri::command]
async fn trigger_email_check(pool: State<'_, DbPool>) -> Result<String, String> {
    use crate::database_pg::repositories::ScheduledEmailRepository;
    use crate::database_pg::repositories::EmailLogRepository;

    println!("📧 trigger_email_check called - checking for pending emails...");

    // Get all pending emails that are due
    let pending = ScheduledEmailRepository::get_pending(&pool)
        .await
        .map_err(|e| format!("Failed to get pending emails: {}", e))?;

    println!("✅ Found {} pending emails", pending.len());

    let mut sent_count = 0;
    let mut failed_count = 0;

    // Process each pending email
    for scheduled_email in pending {
        println!("📧 Processing scheduled email {} (template: {})", scheduled_email.id, scheduled_email.template_name);

        // Check if email is due (scheduled_for <= now)
        let now = chrono::Local::now().naive_local();
        let scheduled_for = chrono::NaiveDateTime::parse_from_str(&scheduled_email.scheduled_for, "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|_| now);

        if scheduled_for > now {
            println!("⏳ Email {} not yet due (scheduled for {})", scheduled_email.id, scheduled_email.scheduled_for);
            continue;
        }

        // Get booking details if booking_id exists
        let result = if let Some(booking_id) = scheduled_email.booking_id {
            match process_scheduled_booking_email(&pool, booking_id, &scheduled_email.template_name).await {
                Ok(_) => {
                    sent_count += 1;

                    // Update scheduled_email status to 'sent'
                    if let Err(e) = ScheduledEmailRepository::update_status(&pool, scheduled_email.id, "sent").await {
                        eprintln!("❌ Failed to update scheduled_email status: {}", e);
                    }

                    // Log successful send
                    let log_result = EmailLogRepository::create(
                        &pool,
                        Some(booking_id),
                        scheduled_email.guest_id.unwrap_or(0),
                        scheduled_email.template_name.clone(),
                        scheduled_email.recipient_email.clone(),
                        scheduled_email.subject.clone(),
                        "gesendet".to_string(),
                        None,
                    ).await;

                    if let Err(e) = log_result {
                        eprintln!("⚠️ Failed to create email log: {}", e);
                    }

                    Ok(())
                }
                Err(e) => {
                    failed_count += 1;
                    eprintln!("❌ Failed to send email {}: {}", scheduled_email.id, e);

                    // Update scheduled_email status to 'failed'
                    if let Err(update_err) = ScheduledEmailRepository::update_status_with_error(&pool, scheduled_email.id, "failed", &e).await {
                        eprintln!("❌ Failed to update scheduled_email status: {}", update_err);
                    }

                    // Log failed send
                    let log_result = EmailLogRepository::create(
                        &pool,
                        Some(booking_id),
                        scheduled_email.guest_id.unwrap_or(0),
                        scheduled_email.template_name.clone(),
                        scheduled_email.recipient_email.clone(),
                        scheduled_email.subject.clone(),
                        "fehler".to_string(),
                        Some(e.clone()),
                    ).await;

                    if let Err(log_err) = log_result {
                        eprintln!("⚠️ Failed to create email log: {}", log_err);
                    }

                    Err(e)
                }
            }
        } else {
            println!("⚠️ Scheduled email {} has no booking_id, skipping", scheduled_email.id);
            continue;
        };

        if result.is_ok() {
            println!("✅ Email {} sent successfully", scheduled_email.id);
        }
    }

    let message = format!(
        "✅ Email-Check abgeschlossen: {} versendet, {} fehlgeschlagen",
        sent_count,
        failed_count
    );

    println!("{}", message);
    Ok(message)
}

/// Helper function to process a scheduled booking email
async fn process_scheduled_booking_email(
    pool: &DbPool,
    booking_id: i32,
    template_name: &str,
) -> Result<(), String> {
    // Load booking
    let booking = BookingRepository::get_by_id(pool, booking_id)
        .await
        .map_err(|e| format!("Buchung nicht gefunden: {}", e))?;

    // Load guest
    let guest = GuestRepository::get_by_id(pool, booking.guest_id)
        .await
        .map_err(|e| format!("Gast nicht gefunden: {}", e))?;

    // Check if guest has email
    if guest.email.is_empty() {
        return Err("Gast hat keine Email-Adresse".to_string());
    }

    // Load room
    let room = RoomRepository::get_by_id(pool, booking.room_id)
        .await
        .map_err(|e| format!("Zimmer nicht gefunden: {}", e))?;

    // Load email template
    let template = EmailTemplateRepository::get_by_name(pool, template_name.to_string())
        .await
        .map_err(|e| format!("Email-Template '{}' nicht gefunden: {}", template_name, e))?;

    // Replace placeholders
    let subject = replace_template_placeholders(&template.subject, &booking, &guest, &room.name);
    let body = replace_template_placeholders(&template.body, &booking, &guest, &room.name);

    // Send email
    send_email_helper(
        pool,
        &guest.email,
        &format!("{} {}", guest.vorname, guest.nachname),
        &subject,
        &body,
    ).await?;

    println!("✅ Sent '{}' email to {} for booking {}", template_name, guest.email, booking_id);
    Ok(())
}

#[tauri::command]
async fn undo_transaction_command(
    pool: State<'_, DbPool>,
    transaction_id: i64,
) -> Result<(), String> {
    use crate::database_pg::repositories::GuestCreditRepository;

    println!("↩️ undo_transaction_command called for transaction_id={}", transaction_id);

    // Use the repository's delete method which includes validation
    GuestCreditRepository::delete(&pool, transaction_id as i32)
        .await
        .map_err(|e| {
            eprintln!("❌ Error deleting transaction: {}", e);
            e.to_string()
        })?;

    println!("✅ Transaction {} successfully deleted", transaction_id);
    Ok(())
}


#[tauri::command]
async fn update_template_command(
    pool: State<'_, DbPool>,
    id: i64,
    subject: String,
    body: String,
    _description: Option<String>,
) -> Result<(), String> {
    println!("📧 Updating email template {}", id);

    // Load existing template to get template_name and is_active
    let existing = EmailTemplateRepository::get_by_id(&pool, id as i32)
        .await
        .map_err(|e| format!("Template nicht gefunden: {}", e))?;

    // Update template
    EmailTemplateRepository::update(
        &pool,
        id as i32,
        existing.template_name,
        subject,
        body,
        existing.is_active,
    )
    .await
    .map_err(|e| format!("Fehler beim Speichern: {}", e))?;

    println!("✅ Template {} erfolgreich aktualisiert", id);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogoUploadResult {
    logo_data: String,
    logo_mime_type: String,
    file_name: String,
}

#[tauri::command]
async fn upload_logo_command(source_path: String) -> Result<LogoUploadResult, String> {
    println!("📷 upload_logo_command called with source_path: {}", source_path);

    let source_path_obj = std::path::Path::new(&source_path);

    // Validate file size (max 2 MB)
    let metadata = std::fs::metadata(&source_path)
        .map_err(|e| format!("Fehler beim Lesen der Datei: {}", e))?;
    if metadata.len() > 2 * 1024 * 1024 {
        return Err("Logo-Datei ist zu groß (max. 2 MB)".to_string());
    }

    // Determine MIME type from extension
    let extension = source_path_obj
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime_type = match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => return Err("Nicht unterstütztes Bildformat. Bitte PNG oder JPG verwenden.".to_string()),
    };

    // Read file and encode as Base64
    let file_bytes = std::fs::read(&source_path)
        .map_err(|e| format!("Fehler beim Lesen der Logo-Datei: {}", e))?;
    let base64_data = general_purpose::STANDARD.encode(&file_bytes);

    let file_name = source_path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("logo.png")
        .to_string();

    println!("✅ Logo erfolgreich als Base64 geladen: {} ({} bytes, {})", file_name, file_bytes.len(), mime_type);

    Ok(LogoUploadResult {
        logo_data: base64_data,
        logo_mime_type: mime_type.to_string(),
        file_name,
    })
}

#[tauri::command]
async fn use_guest_credit_for_booking(guest_id: i64, booking_id: i64, amount: f64, pool: State<'_, DbPool>) -> Result<(), String> {
    use crate::database_pg::repositories::GuestCreditRepository;

    println!("💸 use_guest_credit_for_booking: guest_id={}, booking_id={}, amount={:.2}€", guest_id, booking_id, amount);

    GuestCreditRepository::use_credit_for_booking(
        &pool,
        guest_id as i32,
        booking_id as i32,
        amount,
        Some(format!("Applied to booking #{}", booking_id)),
        Some("System".to_string()),
    )
    .await
    .map_err(|e| {
        // Return user-friendly error message
        if e.to_string().contains("Insufficient credit balance") {
            format!("Unzureichendes Guthaben. {}", e)
        } else {
            format!("Fehler beim Verwenden von Guthaben: {}", e)
        }
    })?;

    println!("✅ Credit applied to booking successfully");
    Ok(())
}

#[tauri::command]
async fn get_booking_credit_usage(pool: State<'_, DbPool>, booking_id: i32) -> Result<f64, String> {
    println!("💰 get_booking_credit_usage called for booking_id: {}", booking_id);

    let booking = BookingRepository::get_by_id(&pool, booking_id)
        .await
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    let credit_used = booking.credit_used.unwrap_or(0.0);
    println!("💰 Credit used for booking {}: {:.2} €", booking_id, credit_used);

    Ok(credit_used)
}

// ============================================================================
// EMAIL LOGS COMMANDS
// ============================================================================

#[tauri::command]
async fn get_email_logs_command(
    pool: State<'_, DbPool>,
    limit: Option<i32>,
) -> Result<Vec<database_pg::EmailLog>, String> {
    println!("📧 Loading email logs (limit: {:?})", limit);

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let rows = client
        .query(
            "SELECT id, booking_id, guest_id, template_name, recipient_email,
                    subject, status, error_message, sent_at::text as sent_at
             FROM email_logs
             ORDER BY sent_at DESC NULLS LAST
             LIMIT $1",
            &[&limit.unwrap_or(100)],
        )
        .await
        .map_err(|e| format!("Fehler beim Laden der Email-Logs: {}", e))?;

    let logs: Vec<database_pg::EmailLog> = rows
        .into_iter()
        .map(database_pg::EmailLog::from)
        .collect();

    println!("✅ Loaded {} email logs", logs.len());
    Ok(logs)
}

#[tauri::command]
async fn clear_email_logs_command(
    pool: State<'_, DbPool>,
    days_to_keep: Option<i32>,
) -> Result<i64, String> {
    let days = days_to_keep.unwrap_or(90);
    println!("🗑️ Clearing email logs older than {} days...", days);

    let client = pool.get().await
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let rows_deleted = client
        .execute(
            "DELETE FROM email_logs WHERE sent_at < NOW() - ($1 || ' days')::interval",
            &[&days],
        )
        .await
        .map_err(|e| format!("Fehler beim Löschen der Email-Logs: {}", e))?;

    println!("✅ Deleted {} old email logs", rows_deleted);
    Ok(rows_deleted as i64)
}

