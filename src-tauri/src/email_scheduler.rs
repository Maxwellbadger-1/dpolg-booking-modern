use rusqlite::{Connection, Result};
use std::thread;
use std::time::Duration;
use crate::database::get_db_path;
use crate::email::{send_reminder_email, send_payment_reminder_email};

/// Prüft täglich, ob Erinnerungs- oder Zahlungserinnerungs-Emails versendet werden müssen
pub fn start_email_scheduler() {
    thread::spawn(|| {
        loop {
            // Lade scheduler_interval_hours aus notification_settings
            let interval_hours = match get_scheduler_interval() {
                Ok(hours) => hours,
                Err(e) => {
                    eprintln!("❌ Fehler beim Laden des Scheduler-Intervalls: {}. Verwende Default: 1 Stunde", e);
                    1
                }
            };

            // Warte X Stunden zwischen Checks (konfigurierbar via Settings)
            let interval_secs = (interval_hours as u64) * 3600;
            thread::sleep(Duration::from_secs(interval_secs));

            println!("📧 Email-Scheduler: Prüfe anstehende Emails... (Intervall: {} Stunden)", interval_hours);

            // Check-in Erinnerungen versenden (1 Tag vor Check-in)
            if let Err(e) = check_and_send_checkin_reminders() {
                eprintln!("❌ Fehler beim Check-in Reminder Check: {}", e);
            }

            // Zahlungserinnerungen versenden
            if let Err(e) = check_and_send_payment_reminders() {
                eprintln!("❌ Fehler beim Payment Reminder Check: {}", e);
            }
        }
    });
}

/// Lädt das Scheduler-Intervall aus den notification_settings
fn get_scheduler_interval() -> Result<i64, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let interval: i64 = conn.query_row(
        "SELECT scheduler_interval_hours FROM notification_settings WHERE id = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| format!("Fehler beim Laden des Intervalls: {}", e))?;

    Ok(interval)
}

/// Prüft und versendet Check-in Erinnerungen für Buchungen mit Check-in morgen
fn check_and_send_checkin_reminders() -> Result<(), String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Prüfe ob Check-in Erinnerungen aktiviert sind
    let enabled: bool = conn.query_row(
        "SELECT checkin_reminders_enabled FROM notification_settings WHERE id = 1",
        [],
        |row| row.get(0)
    ).unwrap_or(true); // Default: aktiviert

    if !enabled {
        println!("   ⏭️ Check-in Erinnerungen sind deaktiviert");
        return Ok(());
    }

    // Suche Buchungen mit Check-in MORGEN
    let tomorrow = crate::time_utils::add_days(1)
        .format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT b.id
         FROM bookings b
         WHERE b.checkin_date = ?1
         AND b.status != 'storniert'"
    ).map_err(|e| format!("SQL Fehler: {}", e))?;

    let booking_ids: Vec<i64> = stmt.query_map([&tomorrow], |row| {
        row.get(0)
    })
    .map_err(|e| format!("Query Fehler: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    println!("   Gefunden: {} Buchungen mit Check-in morgen ({})", booking_ids.len(), tomorrow);

    // Versende Erinnerung für jede Buchung
    for booking_id in booking_ids {
        // Prüfe ob bereits eine Erinnerung versendet wurde
        let already_sent: bool = conn.query_row(
            "SELECT COUNT(*) > 0
             FROM email_log
             WHERE booking_id = ?1
             AND template_type = 'reminder'
             AND status = 'sent'",
            [booking_id],
            |row| row.get(0)
        ).unwrap_or(false);

        if already_sent {
            println!("   ⏭️ Erinnerung für Buchung {} bereits versendet", booking_id);
            continue;
        }

        // Versende Erinnerung asynchron
        println!("   📨 Versende Check-in Erinnerung für Buchung {}", booking_id);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(send_reminder_email(booking_id)) {
            Ok(_) => println!("   ✅ Erinnerung erfolgreich versendet"),
            Err(e) => eprintln!("   ❌ Fehler beim Versenden: {}", e),
        }
    }

    Ok(())
}

/// Prüft und versendet Zahlungserinnerungen für unbezahlte Buchungen
fn check_and_send_payment_reminders() -> Result<(), String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Prüfe ob Zahlungserinnerungen aktiviert sind
    let enabled: bool = conn.query_row(
        "SELECT payment_reminders_enabled FROM notification_settings WHERE id = 1",
        [],
        |row| row.get(0)
    ).unwrap_or(true); // Default: aktiviert

    if !enabled {
        println!("   ⏭️ Zahlungserinnerungen sind deaktiviert");
        return Ok(());
    }

    // Hole reminder_after_days aus notification_settings
    let reminder_after_days: i64 = conn.query_row(
        "SELECT payment_reminder_after_days FROM notification_settings WHERE id = 1",
        [],
        |row| row.get(0)
    ).unwrap_or(14); // Default: 14 Tage

    // Berechne Datum vor X Tagen
    let reminder_date = crate::time_utils::add_days(-reminder_after_days)
        .format("%Y-%m-%d").to_string();

    println!("   Suche unbezahlte Buchungen erstellt vor {} (älter als {} Tage)", reminder_date, reminder_after_days);

    // Suche unbezahlte Buchungen, die älter als reminder_after_days sind (basierend auf created_at)
    let mut stmt = conn.prepare(
        "SELECT b.id, b.reservierungsnummer
         FROM bookings b
         WHERE b.bezahlt = 0
         AND b.status != 'storniert'
         AND date(b.created_at) <= date(?1)"
    ).map_err(|e| format!("SQL Fehler: {}", e))?;

    let bookings: Vec<(i64, String)> = stmt.query_map([&reminder_date], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })
    .map_err(|e| format!("Query Fehler: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    println!("   Gefunden: {} unbezahlte Buchungen", bookings.len());

    // Versende Zahlungserinnerung für jede Buchung
    for (booking_id, reservation_num) in bookings {
        // Hole repeat_days aus notification_settings für Wiederholungsintervall
        let repeat_days: i64 = conn.query_row(
            "SELECT payment_reminder_repeat_days FROM notification_settings WHERE id = 1",
            [],
            |row| row.get(0)
        ).unwrap_or(14); // Default: 14 Tage

        // Prüfe wann letzte Zahlungserinnerung versendet wurde
        let last_reminder: Option<String> = conn.query_row(
            "SELECT sent_at
             FROM email_log
             WHERE booking_id = ?1
             AND template_type = 'payment_reminder'
             AND status = 'sent'
             ORDER BY sent_at DESC
             LIMIT 1",
            [booking_id],
            |row| row.get(0)
        ).ok();

        // Versende nur wenn noch keine Erinnerung versendet wurde
        // ODER letzte Erinnerung mehr als repeat_days Tage her ist
        let should_send = match &last_reminder {
            None => true,
            Some(last_sent) => {
                // Parse last_sent und prüfe ob > repeat_days her
                let repeat_threshold = crate::time_utils::add_days(-repeat_days)
                    .format("%Y-%m-%d %H:%M:%S").to_string();
                last_sent < &repeat_threshold
            }
        };

        if !should_send {
            println!("   ⏭️ Zahlungserinnerung für {} kürzlich versendet", reservation_num);
            continue;
        }

        // Versende Zahlungserinnerung asynchron
        println!("   💰 Versende Zahlungserinnerung für {}", reservation_num);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(send_payment_reminder_email(booking_id)) {
            Ok(_) => println!("   ✅ Zahlungserinnerung erfolgreich versendet"),
            Err(e) => eprintln!("   ❌ Fehler beim Versenden: {}", e),
        }
    }

    Ok(())
}

/// Manueller Trigger für Email-Check (für Testing)
#[tauri::command]
pub fn trigger_email_check() -> Result<String, String> {
    println!("🔔 Manueller Email-Check getriggert...");

    check_and_send_checkin_reminders()?;
    check_and_send_payment_reminders()?;

    Ok("Email-Check abgeschlossen".to_string())
}

/// Geplante Email-Informationen (ohne Versand)
#[derive(serde::Serialize)]
pub struct ScheduledEmail {
    pub booking_id: i64,
    pub reservierungsnummer: String,
    pub guest_name: String,
    pub guest_email: String,
    pub email_type: String,
    pub scheduled_date: String,
    pub reason: String,
}

/// Zeigt alle geplanten automatischen Emails
#[tauri::command]
pub fn get_scheduled_emails() -> Result<Vec<ScheduledEmail>, String> {
    println!("🔍 get_scheduled_emails called");
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let mut scheduled = Vec::new();
    let today = crate::time_utils::now_utc_plus_2().format("%Y-%m-%d").to_string();

    // Prüfe ob Check-in Erinnerungen aktiviert sind
    let checkin_enabled: bool = conn.query_row(
        "SELECT checkin_reminders_enabled FROM notification_settings WHERE id = 1",
        [],
        |row| row.get(0)
    ).unwrap_or(true); // Default: aktiviert

    // 1. Check-in Erinnerungen (alle zukünftigen Buchungen) - nur wenn aktiviert
    if checkin_enabled {
        println!("📅 Searching for all future check-ins (from today: {})", today);

        let mut stmt = conn.prepare(
        "SELECT b.id, b.reservierungsnummer, b.checkin_date,
                g.vorname, g.nachname, g.email
         FROM bookings b
         JOIN guests g ON b.guest_id = g.id
         WHERE b.checkin_date > ?1
         AND b.status != 'storniert'
         ORDER BY b.checkin_date ASC"
    ).map_err(|e| format!("SQL Fehler: {}", e))?;

    let checkin_reminders: Vec<_> = stmt.query_map([&today], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })
    .map_err(|e| format!("Query Fehler: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    println!("✅ Found {} future bookings for check-in reminders", checkin_reminders.len());

    for (booking_id, res_num, checkin, vorname, nachname, email) in checkin_reminders {
        // Berechne das Datum für die Erinnerung (1 Tag vor Check-in)
        let checkin_date = chrono::NaiveDate::parse_from_str(&checkin, "%Y-%m-%d")
            .map_err(|e| format!("Fehler beim Parsen des Datums: {}", e))?;
        let reminder_date = checkin_date - chrono::Duration::days(1);
        let reminder_date_str = reminder_date.format("%Y-%m-%d").to_string();

        let already_sent: bool = conn.query_row(
            "SELECT COUNT(*) > 0
             FROM email_log
             WHERE booking_id = ?1
             AND template_name LIKE '%Erinnerung%'
             AND status = 'gesendet'",
            [booking_id],
            |row| row.get(0)
        ).unwrap_or(false);

        println!("  Booking {}: checkin={}, reminder_date={}, already_sent={}",
                 booking_id, checkin, reminder_date_str, already_sent);

        if !already_sent {
            scheduled.push(ScheduledEmail {
                booking_id,
                reservierungsnummer: res_num.clone(),
                guest_name: format!("{} {}", vorname, nachname),
                guest_email: email,
                email_type: "Erinnerung".to_string(),
                scheduled_date: reminder_date_str,
                reason: format!("Check-in Erinnerung (1 Tag vor Check-in am {})", checkin),
            });
        }
    }
    } else {
        println!("   ⏭️ Check-in Erinnerungen sind deaktiviert - keine geplanten Emails");
    }

    // Prüfe ob Zahlungserinnerungen aktiviert sind
    let payment_enabled: bool = conn.query_row(
        "SELECT payment_reminders_enabled FROM notification_settings WHERE id = 1",
        [],
        |row| row.get(0)
    ).unwrap_or(true); // Default: aktiviert

    // 2. Zahlungserinnerungen - nur wenn aktiviert
    if payment_enabled {
        let reminder_after_days: i64 = conn.query_row(
            "SELECT payment_reminder_after_days FROM notification_settings WHERE id = 1",
            [],
            |row| row.get(0)
        ).unwrap_or(14); // Default: 14 Tage

        let reminder_date = crate::time_utils::add_days(-reminder_after_days)
            .format("%Y-%m-%d").to_string();

        println!("   Suche unbezahlte Buchungen erstellt vor {} (älter als {} Tage)", reminder_date, reminder_after_days);

        let mut stmt2 = conn.prepare(
            "SELECT b.id, b.reservierungsnummer, b.checkin_date, b.gesamtpreis,
                    g.vorname, g.nachname, g.email
             FROM bookings b
             JOIN guests g ON b.guest_id = g.id
             WHERE b.bezahlt = 0
             AND b.status != 'storniert'
             AND date(b.created_at) <= date(?1)"
        ).map_err(|e| format!("SQL Fehler: {}", e))?;

        let payment_reminders: Vec<_> = stmt2.query_map([&reminder_date], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| format!("Query Fehler: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

        println!("✅ Found {} payment reminders", payment_reminders.len());

        // Hole repeat_days für Wiederholungsintervall
        let repeat_days: i64 = conn.query_row(
            "SELECT payment_reminder_repeat_days FROM notification_settings WHERE id = 1",
            [],
            |row| row.get(0)
        ).unwrap_or(14); // Default: 14 Tage

        for (booking_id, res_num, checkin, total, vorname, nachname, email) in payment_reminders {
            let last_reminder: Option<String> = conn.query_row(
                "SELECT sent_at
                 FROM email_log
                 WHERE booking_id = ?1
                 AND template_name LIKE '%Zahlungs%'
                 AND status = 'gesendet'
                 ORDER BY sent_at DESC
                 LIMIT 1",
                [booking_id],
                |row| row.get(0)
            ).ok();

            let should_send = match &last_reminder {
                None => true,
                Some(last_sent) => {
                    let repeat_threshold = crate::time_utils::add_days(-repeat_days)
                        .format("%Y-%m-%d %H:%M:%S").to_string();
                    last_sent < &repeat_threshold
                }
            };

            if should_send {
                let next_send_date = match &last_reminder {
                    None => "Sofort".to_string(),
                    Some(_) => "In < 14 Tagen".to_string(),
                };

                scheduled.push(ScheduledEmail {
                    booking_id,
                    reservierungsnummer: res_num.clone(),
                    guest_name: format!("{} {}", vorname, nachname),
                    guest_email: email,
                    email_type: "Zahlungserinnerung".to_string(),
                    scheduled_date: next_send_date,
                    reason: format!("Unbezahlt ({:.2}€) - Check-in: {}", total, checkin),
                });
            }
        }
    } else {
        println!("   ⏭️ Zahlungserinnerungen sind deaktiviert - keine geplanten Emails");
    }

    println!("📧 Total scheduled emails: {}", scheduled.len());
    Ok(scheduled)
}
