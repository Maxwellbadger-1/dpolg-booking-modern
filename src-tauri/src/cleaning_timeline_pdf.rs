use serde::{Deserialize, Serialize};
use reqwest::Client;
use tauri::{AppHandle, Manager};
use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::types::PrintToPdfOptions;
use std::collections::HashMap;
use chrono::Datelike;

// Turso Config
const TURSO_URL: &str = "https://dpolg-cleaning-maxwellbadger-1.aws-eu-west-1.turso.io";
const TURSO_TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NTk4NDI1MzcsImlkIjoiZjY1ZWY2YzMtYWNhMS00NjZiLWExYjgtODU0MTlmYjlmNDNiIiwicmlkIjoiMTRjNDc4YjAtYTAwMy00ZmZmLThiYTUtYTZhOWIwYjZiODdmIn0.JSyu72rlp3pQ_vFxozglKoV-XMHW12j_hVfhTKjbEGwSyWnWBq2kziJNx2WwvvwD09NU-TMoLLszq2Mm9OlLDw";

#[derive(Debug, Serialize)]
struct TursoStmt {
    sql: String,
}

#[derive(Debug, Serialize)]
struct TursoRequestItem {
    #[serde(rename = "type")]
    request_type: String,
    stmt: TursoStmt,
}

#[derive(Debug, Serialize)]
struct TursoRequest {
    requests: Vec<TursoRequestItem>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CleaningTask {
    pub id: i64,
    pub booking_id: i64,
    pub reservierungsnummer: String,
    pub date: String,
    pub room_name: String,
    pub room_id: i64,
    pub room_location: Option<String>,
    pub guest_name: String,
    pub checkout_time: String,
    pub checkin_time: Option<String>,
    pub priority: String,
    pub notes: Option<String>,
    pub status: String,
    pub guest_count: i64,
    pub extras: String,
    pub emojis_start: String,
    pub emojis_end: String,
}

/// Hole CleaningTasks von Turso für einen bestimmten Monat
async fn get_cleaning_tasks_for_month(year: i32, month: u32) -> Result<Vec<CleaningTask>, String> {
    println!("📅 [get_cleaning_tasks_for_month] Jahr: {}, Monat: {}", year, month);

    let start_date = format!("{}-{:02}-01", year, month);
    let days_in_month = match month {
        2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    let end_date = format!("{}-{:02}-{:02}", year, month, days_in_month);

    println!("📆 [get_cleaning_tasks_for_month] Zeitraum: {} bis {}", start_date, end_date);

    let client = Client::new();
    let sql = format!(
        "SELECT id, booking_id, reservierungsnummer, date, room_name, room_id, room_location, \
         guest_name, checkout_time, checkin_time, priority, notes, status, guest_count, extras, \
         emojis_start, emojis_end \
         FROM cleaning_tasks \
         WHERE date >= '{}' AND date <= '{}' \
         ORDER BY room_name, date",
        start_date, end_date
    );

    let request_body = TursoRequest {
        requests: vec![
            TursoRequestItem {
                request_type: "execute".to_string(),
                stmt: TursoStmt { sql },
            },
            TursoRequestItem {
                request_type: "close".to_string(),
                stmt: TursoStmt { sql: String::new() },
            },
        ],
    };

    let response = client
        .post(format!("{}/v2/pipeline", TURSO_URL))
        .header("Authorization", format!("Bearer {}", TURSO_TOKEN))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP Request fehlgeschlagen: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Turso Fehler: {}", response.status()));
    }

    let text = response.text().await
        .map_err(|e| format!("Text Parse Error: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("JSON Parse Error: {}", e))?;

    let mut tasks = Vec::new();

    if let Some(results) = json.get("results") {
        if let Some(first_result) = results.get(0) {
            if let Some(response_obj) = first_result.get("response") {
                if let Some(result) = response_obj.get("result") {
                    if let Some(rows) = result.get("rows") {
                        if let Some(rows_array) = rows.as_array() {
                            for row in rows_array {
                                if let Some(row_array) = row.as_array() {
                                    let get_value = |idx: usize| -> Option<String> {
                                        row_array.get(idx)
                                            .and_then(|v| v.get("value"))
                                            .and_then(|v| {
                                                if v.is_null() {
                                                    None
                                                } else {
                                                    v.as_str().map(|s| s.to_string())
                                                }
                                            })
                                    };

                                    let get_i64 = |idx: usize| -> i64 {
                                        get_value(idx)
                                            .and_then(|s| s.parse::<i64>().ok())
                                            .unwrap_or(0)
                                    };

                                    let task = CleaningTask {
                                        id: get_i64(0),
                                        booking_id: get_i64(1),
                                        reservierungsnummer: get_value(2).unwrap_or_default(),
                                        date: get_value(3).unwrap_or_default(),
                                        room_name: get_value(4).unwrap_or_default(),
                                        room_id: get_i64(5),
                                        room_location: get_value(6),
                                        guest_name: get_value(7).unwrap_or_default(),
                                        checkout_time: get_value(8).unwrap_or_default(),
                                        checkin_time: get_value(9),
                                        priority: get_value(10).unwrap_or_default(),
                                        notes: get_value(11),
                                        status: get_value(12).unwrap_or_default(),
                                        guest_count: get_i64(13),
                                        extras: get_value(14).unwrap_or_default(),
                                        emojis_start: get_value(15).unwrap_or_default(),
                                        emojis_end: get_value(16).unwrap_or_default(),
                                    };

                                    tasks.push(task);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("✅ [get_cleaning_tasks_for_month] {} Tasks gefunden", tasks.len());
    Ok(tasks)
}

/// Generiert HTML für Timeline-PDF mit modernem Design (A4 Querformat, 2 Tabellen)
fn generate_timeline_html(tasks: Vec<CleaningTask>, year: i32, month: u32) -> String {
    let month_names = [
        "JANUAR", "FEBRUAR", "MÄRZ", "APRIL", "MAI", "JUNI",
        "JULI", "AUGUST", "SEPTEMBER", "OKTOBER", "NOVEMBER", "DEZEMBER"
    ];
    let month_name = month_names.get((month - 1) as usize).unwrap_or(&"");

    let days_in_month = match month {
        2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };

    // Gruppiere Tasks nach Zimmer
    let mut room_tasks: HashMap<String, Vec<CleaningTask>> = HashMap::new();
    for task in tasks.clone() {
        room_tasks.entry(task.room_name.clone())
            .or_insert_with(Vec::new)
            .push(task);
    }

    // Sortiere Zimmer
    let mut rooms: Vec<String> = room_tasks.keys().cloned().collect();
    rooms.sort_by(|a, b| {
        let a_location = room_tasks.get(a)
            .and_then(|tasks| tasks.first())
            .and_then(|t| t.room_location.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");
        let b_location = room_tasks.get(b)
            .and_then(|tasks| tasks.first())
            .and_then(|t| t.room_location.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");

        if a_location != b_location {
            a_location.cmp(b_location)
        } else {
            alphanumeric_sort::compare_str(a, b)
        }
    });

    // Teile den Monat: Tag 1-15 und Tag 16-Ende
    let split_day = 16;

    // Generiere beide Tabellen
    let table1 = generate_table_html(&rooms, &room_tasks, year, month, 1, split_day - 1);
    let table2 = generate_table_html(&rooms, &room_tasks, year, month, split_day, days_in_month);

    // Count Stats
    let total_checkouts: usize = tasks.iter().filter(|t| !t.emojis_end.is_empty()).count();
    let total_rooms = rooms.len();

    format!(
        r#"<!DOCTYPE html>
<html lang="de">
<head>
    <meta charset="UTF-8">
    <title>Putzplan - {} {}</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet">
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}

        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            background: #f8fafc;
            color: #0f172a;
            font-size: 9px;
            line-height: 1.3;
        }}

        @page {{
            size: A4 landscape;
            margin: 0;
        }}

        .container {{
            width: 297mm;
            min-height: 210mm;
            margin: 0 auto;
            padding: 8mm;
            background: white;
        }}

        /* Header */
        .header {{
            background: linear-gradient(135deg, #3b82f6 0%, #6366f1 100%);
            color: white;
            padding: 15px 20px;
            border-radius: 10px;
            margin-bottom: 12px;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
        }}

        .header-row {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 6px;
        }}

        .header-row:last-child {{ margin-bottom: 0; }}

        .title {{
            font-size: 18px;
            font-weight: 800;
            letter-spacing: -0.5px;
        }}

        .month-year {{
            font-size: 15px;
            font-weight: 600;
            opacity: 0.95;
        }}

        .stats {{
            display: flex;
            gap: 20px;
            font-size: 11px;
        }}

        .stat {{
            display: flex;
            align-items: center;
            gap: 6px;
            background: rgba(255, 255, 255, 0.15);
            padding: 4px 10px;
            border-radius: 6px;
            backdrop-filter: blur(10px);
        }}

        .stat-label {{ opacity: 0.9; }}
        .stat-value {{ font-weight: 700; font-size: 13px; }}
        .stat-value.highlight {{ color: #fbbf24; }}

        /* Tapechart Table */
        .tapechart {{
            width: 100%;
            border-collapse: separate;
            border-spacing: 0;
            margin-bottom: 15px;
            background: white;
            border-radius: 10px;
            overflow: hidden;
            box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
        }}

        .tapechart thead th {{
            padding: 6px 2px;
            text-align: center;
            font-weight: 500;
            background: #f8fafc;
            border-bottom: 2px solid #e2e8f0;
            border-right: 1px solid #e2e8f0;
            font-size: 8px;
        }}

        .tapechart thead th:last-child {{ border-right: none; }}

        .tapechart thead th:first-child {{
            width: 90px;
            text-align: left;
            padding-left: 12px;
            background: #1e293b;
            color: white;
            font-weight: 600;
            font-size: 9px;
            letter-spacing: 0.5px;
            text-transform: uppercase;
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
        }}

        .date-header {{
            display: flex;
            flex-direction: column;
            align-items: center;
            line-height: 1.2;
        }}

        .day-name {{
            font-size: 7px;
            color: #94a3b8;
            font-weight: 500;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}

        .day-number {{
            font-size: 11px;
            font-weight: 700;
            margin: 1px 0;
            color: #0f172a;
        }}

        .month-name {{
            font-size: 6px;
            color: #94a3b8;
            text-transform: uppercase;
        }}

        th.weekend {{
            background: #fef3c7 !important;
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
        }}

        th.weekend .day-name,
        th.weekend .month-name {{ color: #92400e; }}
        th.weekend .day-number {{ color: #92400e; }}

        /* Table Body */
        .tapechart tbody td {{
            padding: 0;
            height: 38px;
            position: relative;
            border-bottom: 1px solid #e2e8f0;
            border-right: 1px solid #f1f5f9;
            background: white;
        }}

        .tapechart tbody td:last-child {{ border-right: none; }}
        .tapechart tbody tr:last-child td {{ border-bottom: none; }}

        .tapechart tbody td:first-child {{
            padding: 6px 12px;
            font-weight: 600;
            background: #f8fafc;
            font-size: 9px;
            border-right: 2px solid #e2e8f0;
            color: #334155;
            line-height: 1.3;
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
        }}

        .room-subtitle {{
            font-size: 7px;
            color: #64748b;
            font-weight: 400;
            display: block;
            margin-top: 1px;
        }}

        /* Booking Bars */
        .booking {{
            position: absolute;
            top: 2px;
            bottom: 2px;
            left: 2px;
            right: 2px;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 0 6px;
            font-size: 7px;
            font-weight: 500;
            overflow: hidden;
            white-space: normal;
            word-break: break-word;
            text-align: center;
            line-height: 1.2;
            border-radius: 5px;
            transition: all 0.2s ease;
        }}

        .booking.occupied {{
            background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
            color: white;
            box-shadow: 0 1px 2px 0 rgba(59, 130, 246, 0.3);
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
        }}

        .booking.checkout {{
            background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%) !important;
            color: white;
            font-weight: 700;
            box-shadow: 0 2px 4px 0 rgba(239, 68, 68, 0.4);
            text-transform: uppercase;
            letter-spacing: 0.5px;
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
        }}

        .emojis {{
            font-size: 16px;
        }}

        /* Legend */
        .legend {{
            margin-top: 12px;
            padding: 10px 15px;
            background: #f8fafc;
            border-radius: 10px;
            display: flex;
            gap: 25px;
            font-size: 10px;
            align-items: center;
        }}

        .legend-item {{
            display: flex;
            align-items: center;
            gap: 6px;
        }}

        .legend-color {{
            width: 20px;
            height: 12px;
            border-radius: 3px;
            box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
        }}

        .legend-color.occupied {{
            background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
        }}

        .legend-color.checkout {{
            background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
            -webkit-print-color-adjust: exact;
            print-color-adjust: exact;
        }}

        .legend-text {{
            color: #475569;
            font-weight: 500;
        }}

        @media print {{
            body {{ margin: 0; background: white; }}
            .container {{ width: 100%; height: 100%; margin: 0; padding: 8mm; }}
            .header, .tapechart thead th, .tapechart tbody td, .booking, .legend {{
                -webkit-print-color-adjust: exact !important;
                print-color-adjust: exact !important;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="header-row">
                <div class="title">🧹 REINIGUNGSPLAN</div>
                <div class="month-year">{} {}</div>
            </div>
            <div class="header-row">
                <div class="stats">
                    <div class="stat">
                        <span class="stat-label">Zimmer gesamt</span>
                        <span class="stat-value">{}</span>
                    </div>
                    <div class="stat">
                        <span class="stat-label">Abreisen gesamt</span>
                        <span class="stat-value highlight">{}</span>
                    </div>
                </div>
            </div>
        </div>

        {}

        {}

        <div class="legend">
            <div class="legend-item">
                <div class="legend-color occupied"></div>
                <span class="legend-text">Belegt</span>
            </div>
            <div class="legend-item">
                <div class="legend-color checkout"></div>
                <span class="legend-text">Abreise (Reinigung erforderlich)</span>
            </div>
        </div>
    </div>
</body>
</html>"#,
        month_name, year,
        month_name, year,
        total_rooms,
        total_checkouts,
        table1,
        table2
    )
}

/// Informationen über einen Tag im Zeitraum
#[derive(Debug, Clone)]
struct DayInfo {
    guest_name: String,
    is_checkin: bool,
    is_checkout: bool,
    emojis_start: String,
    emojis_end: String,
    guest_count: i64,
}

/// Generiert eine einzelne Tabelle für einen Tag-Bereich
fn generate_table_html(
    rooms: &[String],
    room_tasks: &HashMap<String, Vec<CleaningTask>>,
    year: i32,
    month: u32,
    start_day: u32,
    end_day: u32,
) -> String {
    let month_names_short = ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"];
    let month_short = month_names_short.get((month - 1) as usize).unwrap_or(&"");

    // Header generieren
    let mut header_html = String::from("<tr><th>Zimmer</th>");
    for day in start_day..=end_day {
        let date_str = format!("{}-{:02}-{:02}", year, month, day);
        let date_obj = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok();

        let (weekday_short, is_weekend) = date_obj
            .map(|d| {
                let wd = d.weekday();
                let short = match wd {
                    chrono::Weekday::Mon => "MO",
                    chrono::Weekday::Tue => "DI",
                    chrono::Weekday::Wed => "MI",
                    chrono::Weekday::Thu => "DO",
                    chrono::Weekday::Fri => "FR",
                    chrono::Weekday::Sat => "SA",
                    chrono::Weekday::Sun => "SO",
                };
                let weekend = matches!(wd, chrono::Weekday::Sat | chrono::Weekday::Sun);
                (short, weekend)
            })
            .unwrap_or(("", false));

        let weekend_class = if is_weekend { " weekend" } else { "" };

        header_html.push_str(&format!(
            r#"<th class="{}">
                <div class="date-header">
                    <span class="day-name">{}</span>
                    <span class="day-number">{}</span>
                    <span class="month-name">{}</span>
                </div>
            </th>"#,
            weekend_class.trim(),
            weekday_short,
            day,
            month_short.to_uppercase()
        ));
    }
    header_html.push_str("</tr>");

    // Body generieren
    let mut rows_html = String::new();

    for room in rooms {
        let tasks = room_tasks.get(room).unwrap();
        let location = tasks.first()
            .and_then(|t| t.room_location.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");

        let location_icon = match location {
            "Fall" => "🍂",
            "Langenprozelten" => "🏡",
            _ => "📍",
        };

        rows_html.push_str(&format!(
            r#"<tr><td>{}<span class="room-subtitle">{} {}</span></td>"#,
            room, location_icon, location
        ));

        // Gruppiere Tasks nach booking_id und verwende ORIGINAL checkin/checkout times
        // FIX: Alle Felder müssen geupdated werden, nicht nur bei or_insert()!
        let mut booking_ranges: HashMap<i64, (String, String, String, String, String, i64)> = HashMap::new();

        println!("🔍 DEBUG: Verarbeite {} tasks für Zimmer {}", tasks.len(), room);
        for task in tasks {
            println!("  🔍 Task: booking_id={}, date={}, checkin_time={:?}, checkout_time='{}', emojis_start='{}', emojis_end='{}', guest_count={}",
                     task.booking_id, task.date, task.checkin_time, task.checkout_time, task.emojis_start, task.emojis_end, task.guest_count);

            let entry = booking_ranges.entry(task.booking_id).or_insert((
                String::new(),
                String::new(),
                task.guest_name.clone(),
                String::new(), // emojis_start
                String::new(), // emojis_end
                task.guest_count, // guest_count
            ));

            // Update checkin_time wenn verfügbar (vom Check-IN Task)
            if let Some(ref checkin) = task.checkin_time {
                if !checkin.is_empty() {
                    println!("    ✅ Update checkin: {}", checkin);
                    entry.0 = checkin.clone();
                }
            }

            // Update checkout_time wenn verfügbar (vom Check-OUT Task)
            if !task.checkout_time.is_empty() {
                println!("    ✅ Update checkout: {}", task.checkout_time);
                entry.1 = task.checkout_time.clone();
            }

            // Sammle emojis_start (vom Check-in Task)
            if !task.emojis_start.is_empty() {
                entry.3 = task.emojis_start.clone();
            }

            // Sammle emojis_end (vom Check-out Task)
            if !task.emojis_end.is_empty() {
                entry.4 = task.emojis_end.clone();
            }
        }

        println!("🔍 DEBUG: Aggregierte {} booking_ranges:", booking_ranges.len());
        for (booking_id, (checkin, checkout, guest, emojis_s, emojis_e, guest_cnt)) in &booking_ranges {
            println!("  📋 Booking {}: checkin='{}', checkout='{}', guest='{}', emojis_start='{}', emojis_end='{}', guest_count={}",
                     booking_id, checkin, checkout, guest, emojis_s, emojis_e, guest_cnt);
        }

        // Erstelle vollständige Timeline mit allen Tagen zwischen Check-in und Check-out
        let mut day_info_map: HashMap<String, DayInfo> = HashMap::new();

        for (_booking_id, (checkin_date, checkout_date, guest_name, emojis_start, emojis_end, guest_count)) in booking_ranges {
            // FIX (2025-10-21): Verwende ORIGINAL Daten - KEINE Monatsgrenzen als Fallback!
            // Dies verhindert, dass der gesamte Monat gefüllt wird

            let effective_start = &checkin_date;
            let effective_end = &checkout_date;

            // Skip wenn Daten fehlen (keine Monatsgrenzen mehr!)
            if checkin_date.is_empty() || checkout_date.is_empty() {
                continue; // Skip diese Buchung komplett
            }

            let start = chrono::NaiveDate::parse_from_str(effective_start, "%Y-%m-%d").ok();
            let end = chrono::NaiveDate::parse_from_str(effective_end, "%Y-%m-%d").ok();

            if let (Some(start_date), Some(end_date)) = (start, end) {
                let mut current = start_date;

                while current <= end_date {
                    let date_str = current.format("%Y-%m-%d").to_string();
                    // FIX: is_checkin IMMER true am ersten Tag, egal ob Emojis vorhanden oder nicht!
                    let is_checkin = current == start_date;
                    let is_checkout = current == end_date; // IMMER rot bei Check-out, egal ob Emojis oder nicht

                    day_info_map.insert(date_str, DayInfo {
                        guest_name: guest_name.clone(),
                        is_checkin,
                        is_checkout,
                        emojis_start: if is_checkin { emojis_start.clone() } else { String::new() },
                        emojis_end: if is_checkout { emojis_end.clone() } else { String::new() },
                        guest_count,
                    });

                    current = current.succ_opt().unwrap();
                }
            }
        }

        // Durchlaufe jeden Tag und zeige die passende Zelle
        for day in start_day..=end_day {
            let date_str = format!("{}-{:02}-{:02}", year, month, day);

            if let Some(info) = day_info_map.get(&date_str) {
                if info.is_checkout {
                    // Check-out Tag - rote Zelle nur mit Emojis
                    rows_html.push_str(&format!(
                        r#"<td style="position: relative;">
                            <div class="booking checkout"><span class="emojis">{}</span></div>
                        </td>"#,
                        info.emojis_end
                    ));
                } else if info.is_checkin {
                    // Check-in Tag - blaue Zelle mit Emojis + Personenanzahl
                    let person_badge = if info.guest_count > 0 {
                        format!(r#"<span style="position: absolute; top: 2px; right: 2px; background: rgba(255,255,255,0.9); color: #1e293b; padding: 1px 4px; border-radius: 3px; font-size: 7px; font-weight: 600;">{} Pers.</span>"#, info.guest_count)
                    } else {
                        String::new()
                    };

                    rows_html.push_str(&format!(
                        r#"<td style="position: relative;">
                            <div class="booking occupied"><span class="emojis">{}</span>{}</div>
                        </td>"#,
                        info.emojis_start,
                        person_badge
                    ));
                } else {
                    // Belegt Tag dazwischen - blaue Zelle mit Gästename
                    rows_html.push_str(&format!(
                        r#"<td style="position: relative;">
                            <div class="booking occupied">{}</div>
                        </td>"#,
                        info.guest_name
                    ));
                }
            } else {
                // Leere Zelle
                rows_html.push_str("<td></td>");
            }
        }

        rows_html.push_str("</tr>");
    }

    format!(
        r#"<table class="tapechart">
            <thead>{}</thead>
            <tbody>{}</tbody>
        </table>"#,
        header_html, rows_html
    )
}

/// Exportiert Timeline als PDF mit headless_chrome
#[tauri::command]
pub async fn export_cleaning_timeline_pdf(
    _app: AppHandle,
    year: i32,
    month: u32,
) -> Result<String, String> {
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  PUTZPLAN PDF EXPORT                                │");
    println!("│  Monat: {}/{:02}                                      │", year, month);
    println!("└─────────────────────────────────────────────────────┘");

    // 1. Hole Cleaning Tasks von Turso
    println!("📥 Loading cleaning tasks from Turso...");
    let tasks = get_cleaning_tasks_for_month(year, month).await?;

    if tasks.is_empty() {
        return Err(format!("Keine Aufgaben für {}/{:02} gefunden", month, year));
    }

    // 2. Generiere HTML
    println!("📝 Generating timeline HTML...");
    let html = generate_timeline_html(tasks, year, month);

    // 3. Putzplan-Ordner im App-Datenverzeichnis (NICHT in src-tauri!)
    // FIX: Verhindert Endlosschleife durch Cargo-Rebuild bei Dateiänderungen
    let app_data_dir = _app.path().app_data_dir()
        .map_err(|e| format!("Fehler app_data_dir: {}", e))?;

    let putzplan_dir = app_data_dir.join("putzplan");
    std::fs::create_dir_all(&putzplan_dir)
        .map_err(|e| format!("Fehler create_dir: {}", e))?;

    println!("📁 Putzplan-Ordner: {:?}", putzplan_dir);

    // 4. Temp HTML schreiben
    let month_names = [
        "Januar", "Februar", "Maerz", "April", "Mai", "Juni",
        "Juli", "August", "September", "Oktober", "November", "Dezember"
    ];
    let month_name = month_names.get((month - 1) as usize).unwrap_or(&"Monat");

    let temp_html_path = putzplan_dir.join(format!("temp_putzplan_{}_{:02}.html", year, month));
    std::fs::write(&temp_html_path, &html)
        .map_err(|e| format!("Fehler HTML schreiben: {}", e))?;

    // 5. Headless Chrome starten
    println!("🌐 Starting headless Chrome...");
    let launch_options = LaunchOptions::default_builder()
        .headless(true)
        .build()
        .map_err(|e| format!("Chrome launch options: {}", e))?;

    let browser = Browser::new(launch_options)
        .map_err(|e| format!("Chrome starten Fehler: {}", e))?;
    println!("✅ Chrome started");

    let tab = browser.new_tab()
        .map_err(|e| format!("Chrome tab Fehler: {}", e))?;

    // 6. HTML laden
    let file_url = format!("file://{}", temp_html_path.display());
    tab.navigate_to(&file_url)
        .map_err(|e| format!("Chrome navigate Fehler: {}", e))?;

    tab.wait_until_navigated()
        .map_err(|e| format!("Chrome wait Fehler: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // 7. PDF generieren (A4 Querformat!)
    println!("📄 Generating PDF...");
    let pdf_data = tab.print_to_pdf(Some(PrintToPdfOptions {
        landscape: Some(true),
        display_header_footer: Some(false),
        print_background: Some(true),
        scale: Some(1.0),
        paper_width: Some(11.69), // A4 width in inches
        paper_height: Some(8.27), // A4 height in inches
        margin_top: Some(0.0),
        margin_bottom: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: Some(true),
        transfer_mode: None,
        generate_document_outline: Some(false),
        generate_tagged_pdf: Some(false),
    }))
    .map_err(|e| format!("PDF generieren Fehler: {}", e))?;

    // 8. PDF speichern
    let file_name = format!("DPolG_Putzplan_{}_{}.pdf", month_name, year);
    let output_path = putzplan_dir.join(&file_name);

    std::fs::write(&output_path, pdf_data)
        .map_err(|e| format!("PDF speichern Fehler: {}", e))?;

    // 9. Temp HTML löschen
    std::fs::remove_file(&temp_html_path).ok();

    println!("✅ PDF erstellt: {:?}", output_path);

    // 10. PDF automatisch öffnen
    println!("📂 Öffne PDF...");

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&output_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", &output_path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&output_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    Ok(output_path.to_string_lossy().to_string())
}

/// Öffnet den Putzplan-Ordner im Explorer/Finder
#[tauri::command]
pub fn open_putzplan_folder() -> Result<String, String> {
    let exe_dir = std::env::current_dir()
        .map_err(|e| format!("Fehler current_dir: {}", e))?;

    let putzplan_dir = exe_dir.join("putzplan");
    std::fs::create_dir_all(&putzplan_dir)
        .map_err(|e| format!("Fehler create_dir: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&putzplan_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&putzplan_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&putzplan_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    Ok(putzplan_dir.to_string_lossy().to_string())
}
