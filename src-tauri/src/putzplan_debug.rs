use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PutzplanDebugInfo {
    pub bookings: Vec<BookingDebugInfo>,
    pub turso_tasks: Vec<TursoTaskDebugInfo>,
    pub pdf_aggregation: Vec<PDFAggregationDebugInfo>,
    pub skipped_bookings: Vec<SkippedBookingInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookingDebugInfo {
    pub id: i64,
    pub reservierungsnummer: String,
    pub room_name: String,
    pub guest_name: String,
    pub checkin_date: String,
    pub checkout_date: String,
    pub status: String,
    pub services_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TursoTaskDebugInfo {
    pub booking_id: i64,
    pub date: String,
    pub checkin_time: Option<String>,
    pub checkout_time: String,
    pub emojis_start: String,
    pub emojis_end: String,
    pub task_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PDFAggregationDebugInfo {
    pub booking_id: i64,
    pub checkin_from_hash: String,
    pub checkout_from_hash: String,
    pub guest_name: String,
    pub emojis_start: String,
    pub emojis_end: String,
    pub will_be_skipped: bool,
    pub skip_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkippedBookingInfo {
    pub booking_id: i64,
    pub reason: String,
    pub checkin_date: String,
    pub checkout_date: String,
}

/// DEBUG COMMAND: Sammelt ALLE Debug-Informationen für Putzplan-PDF
#[tauri::command]
pub async fn get_putzplan_debug_info(year: i32, month: u32) -> Result<PutzplanDebugInfo, String> {
    println!("🔍 [DEBUG] get_putzplan_debug_info aufgerufen für {}-{:02}", year, month);

    // 1. Hole ALLE Buchungen für den Monat
    let start_date = format!("{}-{:02}-01", year, month);
    let end_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
        _ => return Err("Invalid month".to_string()),
    };
    let end_date = format!("{}-{:02}-{:02}", year, month, end_day);

    let bookings = crate::database::get_bookings_in_date_range(&start_date, &end_date)
        .map_err(|e| format!("Fehler beim Laden der Buchungen: {}", e))?;

    let bookings_debug: Vec<BookingDebugInfo> = bookings.iter().map(|b| BookingDebugInfo {
        id: b.id,
        reservierungsnummer: b.reservierungsnummer.clone(),
        room_name: b.room.name.clone(),
        guest_name: format!("{} {}", b.guest.vorname, b.guest.nachname),
        checkin_date: b.checkin_date.clone(),
        checkout_date: b.checkout_date.clone(),
        status: b.status.clone(),
        services_count: b.services.len(),
    }).collect();

    // 2. Hole ALLE Turso Tasks für den Monat
    let turso_tasks = fetch_all_turso_tasks_for_month(year, month).await?;

    let turso_debug: Vec<TursoTaskDebugInfo> = turso_tasks.iter().map(|t| TursoTaskDebugInfo {
        booking_id: t.booking_id,
        date: t.date.clone(),
        checkin_time: t.checkin_time.clone(),
        checkout_time: t.checkout_time.clone(),
        emojis_start: t.emojis_start.clone(),
        emojis_end: t.emojis_end.clone(),
        task_type: if !t.emojis_start.is_empty() { "CHECK-IN".to_string() }
                   else if !t.emojis_end.is_empty() { "CHECK-OUT".to_string() }
                   else { "OCCUPIED".to_string() },
    }).collect();

    // 3. Simuliere PDF-Aggregations-Logik
    let (pdf_aggregation, skipped) = simulate_pdf_aggregation(&turso_tasks);

    Ok(PutzplanDebugInfo {
        bookings: bookings_debug,
        turso_tasks: turso_debug,
        pdf_aggregation,
        skipped_bookings: skipped,
    })
}

#[derive(Debug, Clone)]
struct CleaningTask {
    booking_id: i64,
    date: String,
    checkin_time: Option<String>,
    checkout_time: String,
    guest_name: String,
    emojis_start: String,
    emojis_end: String,
}

async fn fetch_all_turso_tasks_for_month(year: i32, month: u32) -> Result<Vec<CleaningTask>, String> {
    // Generiere alle Daten des Monats und hole Tasks
    let end_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
        _ => return Err("Invalid month".to_string()),
    };

    let mut all_tasks = Vec::new();

    for day in 1..=end_day {
        let date = format!("{}-{:02}-{:02}", year, month, day);

        // Turso-Query (vereinfacht - nur zur Demo)
        let sql = format!("SELECT booking_id, date, guest_name, checkin_time, checkout_time, emojis_start, emojis_end FROM cleaning_tasks WHERE date = '{}'", date);

        // TODO: Echte Turso-Abfrage implementieren
        // Für jetzt: Placeholder
    }

    Ok(all_tasks)
}

fn simulate_pdf_aggregation(tasks: &[TursoTaskDebugInfo]) -> (Vec<PDFAggregationDebugInfo>, Vec<SkippedBookingInfo>) {
    use std::collections::HashMap;

    let mut booking_ranges: HashMap<i64, (String, String, String, String, String)> = HashMap::new();
    let mut skipped = Vec::new();

    // Simuliere exakt die PDF-Logik
    for task in tasks {
        let entry = booking_ranges.entry(task.booking_id).or_insert((
            String::new(),
            String::new(),
            "Unknown Guest".to_string(),
            String::new(),
            String::new(),
        ));

        // Update checkin_time
        if let Some(ref checkin) = task.checkin_time {
            if !checkin.is_empty() {
                entry.0 = checkin.clone();
            }
        }

        // Update checkout_time
        if !task.checkout_time.is_empty() {
            entry.1 = task.checkout_time.clone();
        }

        // Emojis
        if !task.emojis_start.is_empty() {
            entry.3 = task.emojis_start.clone();
        }
        if !task.emojis_end.is_empty() {
            entry.4 = task.emojis_end.clone();
        }
    }

    let mut pdf_aggregation = Vec::new();

    for (booking_id, (checkin, checkout, guest, emojis_s, emojis_e)) in &booking_ranges {
        let will_skip = checkin.is_empty() || checkout.is_empty();
        let skip_reason = if checkin.is_empty() && checkout.is_empty() {
            Some("Beide Daten fehlen".to_string())
        } else if checkin.is_empty() {
            Some("Check-in Datum fehlt".to_string())
        } else if checkout.is_empty() {
            Some("Check-out Datum fehlt".to_string())
        } else {
            None
        };

        pdf_aggregation.push(PDFAggregationDebugInfo {
            booking_id: *booking_id,
            checkin_from_hash: checkin.clone(),
            checkout_from_hash: checkout.clone(),
            guest_name: guest.clone(),
            emojis_start: emojis_s.clone(),
            emojis_end: emojis_e.clone(),
            will_be_skipped: will_skip,
            skip_reason: skip_reason.clone(),
        });

        if will_skip {
            skipped.push(SkippedBookingInfo {
                booking_id: *booking_id,
                reason: skip_reason.unwrap_or_else(|| "Unknown".to_string()),
                checkin_date: checkin.clone(),
                checkout_date: checkout.clone(),
            });
        }
    }

    (pdf_aggregation, skipped)
}
