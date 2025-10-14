use serde::{Deserialize, Serialize};
use reqwest::Client;

// TODO: In Config auslagern oder Umgebungsvariablen
const TURSO_URL: &str = "https://dpolg-cleaning-maxwellbadger-1.aws-eu-west-1.turso.io";
const TURSO_TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NTk4NDI1MzcsImlkIjoiZjY1ZWY2YzMtYWNhMS00NjZiLWExYjgtODU0MTlmYjlmNDNiIiwicmlkIjoiMTRjNDc4YjAtYTAwMy00ZmZmLThiYTUtYTZhOWIwYjZiODdmIn0.JSyu72rlp3pQ_vFxozglKoV-XMHW12j_hVfhTKjbEGwSyWnWBq2kziJNx2WwvvwD09NU-TMoLLszq2Mm9OlLDw";

#[derive(Debug, Serialize, Deserialize)]
pub struct CleaningTask {
    pub booking_id: i64,    // NEU: Booking ID für eindeutige Identifikation!
    pub date: String,
    pub room_name: String,
    pub room_id: i64,
    pub room_location: Option<String>, // 🏔️ Ort (Fall, Langenprozelten, etc.)
    pub guest_name: String,
    pub checkout_time: String,
    pub checkin_time: Option<String>,
    pub priority: String, // "high", "normal", "low"
    pub notes: Option<String>,
    pub status: String, // "pending", "done"
    pub guest_count: i32,
    pub extras: String, // JSON als String
    pub emojis_start: String, // Komma-separierte Emojis für Anfang
    pub emojis_end: String, // Komma-separierte Emojis für Ende
}

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

/// Führt SQL-Statement auf Turso aus
async fn execute_turso_sql(sql: String) -> Result<(), String> {
    let client = Client::new();

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
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
        return Err(format!("Turso Fehler {}: {}", status, error_text));
    }

    // Turso gibt 200 OK auch bei Fehlern zurück, prüfen wir den Response
    let text = response.text().await
        .map_err(|e| format!("Text Parse Error: {}", e))?;

    if text.contains("\"error\"") {
        return Err(format!("SQL Error: {}", text));
    }

    Ok(())
}

/// BATCH EXECUTION - alle SQL Statements in EINEM HTTP Request
/// Performance: 3x schneller als einzelne Requests (Turso Research 2025)
async fn execute_turso_batch(sql_statements: Vec<String>) -> Result<(), String> {
    if sql_statements.is_empty() {
        return Ok(());
    }

    let client = Client::new();

    // Baue Batch Request mit allen Statements
    let mut requests = Vec::new();
    for sql in sql_statements {
        requests.push(TursoRequestItem {
            request_type: "execute".to_string(),
            stmt: TursoStmt { sql },
        });
    }
    // Close am Ende
    requests.push(TursoRequestItem {
        request_type: "close".to_string(),
        stmt: TursoStmt { sql: String::new() },
    });

    let request_body = TursoRequest { requests };

    let response = client
        .post(format!("{}/v2/pipeline", TURSO_URL))
        .header("Authorization", format!("Bearer {}", TURSO_TOKEN))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP Request fehlgeschlagen: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
        return Err(format!("Turso Fehler {}: {}", status, error_text));
    }

    let text = response.text().await
        .map_err(|e| format!("Text Parse Error: {}", e))?;

    if text.contains("\"error\"") {
        return Err(format!("SQL Batch Error: {}", text));
    }

    Ok(())
}

/// Synchronisiert Cleaning Tasks für ein bestimmtes Datum zu Turso
/// NEUE LOGIK: Erstellt ZWEI separate Tasks pro Buchung:
/// 1. Check-IN Task mit emojis_start (wenn vorhanden)
/// 2. Check-OUT Task mit emojis_end (wenn vorhanden oder same-day-checkin)
#[tauri::command]
pub async fn sync_cleaning_tasks(date: String) -> Result<String, String> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("🧹 [sync_cleaning_tasks] START für Datum: {}", date);
    println!("═══════════════════════════════════════════════════════════════");

    // Hole Buchungen für Check-OUT an diesem Datum
    let checkouts = crate::database::get_bookings_by_checkout_date(&date)
        .map_err(|e| format!("Fehler beim Laden der Checkouts: {}", e))?;

    // Hole Buchungen für Check-IN an diesem Datum
    let checkins = crate::database::get_bookings_by_checkin_date(&date)
        .map_err(|e| format!("Fehler beim Laden der Check-ins: {}", e))?;

    println!("📦 [sync_cleaning_tasks] {} Checkouts und {} Check-ins gefunden", checkouts.len(), checkins.len());

    if checkouts.is_empty() && checkins.is_empty() {
        println!("⚠️  [sync_cleaning_tasks] Keine Aufgaben für {}", date);
        return Ok("Keine Aufgaben für dieses Datum".to_string());
    }

    // Sammle ALLE SQL Statements für Batch-Ausführung
    let mut sql_statements = Vec::new();
    let mut task_count = 0;

    // DELETE alle existierenden Tasks für dieses Datum
    sql_statements.push(format!("DELETE FROM cleaning_tasks WHERE date = '{}'", date));

    // 1️⃣ Verarbeite Check-IN Tasks (emojis_start)
    for booking in &checkins {
        // Filtere Services mit position="start"
        let service_emojis_start: Vec<String> = booking.services.iter()
            .filter(|s| s.show_in_cleaning_plan && s.cleaning_plan_position == "start" && s.emoji.is_some())
            .map(|s| s.emoji.as_ref().unwrap().clone())
            .collect();

        // Nur erstellen wenn es START-Emojis gibt
        if !service_emojis_start.is_empty() {
            println!("\n┌───── CHECK-IN Task ─────────────────────────────────────────");
            println!("│ 📋 Booking #{}", booking.id);
            println!("│    Gast: {} {}", booking.guest.vorname, booking.guest.nachname);
            println!("│    Zimmer: {}", booking.room.name);
            println!("│    Check-IN: {}", booking.checkin_date);
            println!("│    Emojis START: {}", service_emojis_start.join(","));
            println!("└─────────────────────────────────────────────────────────────");

            let emojis_start_str = service_emojis_start.join(",");
            let room_name = booking.room.name.replace("'", "''");
            let guest_name = format!("{} {}", booking.guest.vorname, booking.guest.nachname).replace("'", "''");
            let emojis_start_escaped = emojis_start_str.replace("'", "''");

            let extras = serde_json::json!({
                "guest_count": booking.anzahl_gaeste,
                "original_checkin": booking.checkin_date,
                "original_checkout": booking.checkout_date,
                "emojis_start": service_emojis_start,
                "task_type": "checkin"
            }).to_string();
            let extras_escaped = extras.replace("'", "''");

            let notes_str = if let Some(ref notes) = booking.bemerkungen {
                format!("'{}'", notes.replace("'", "''"))
            } else {
                "NULL".to_string()
            };

            // 🏔️ Room Location von booking.room.ort
            let room_location = booking.room.ort.as_ref().map(|s| s.replace("'", "''")).unwrap_or_default();

            // INSERT Check-IN Task (mit room_location)
            let insert_sql = format!(
                "INSERT INTO cleaning_tasks (booking_id, date, room_name, room_id, room_location, guest_name, checkout_time, checkin_time, priority, notes, status, guest_count, extras, emojis_start, emojis_end) VALUES ({}, '{}', '{}', {}, '{}', '{}', '{}', '{}', 'normal', {}, 'pending', {}, '{}', '{}', '')",
                booking.id,
                date,  // Check-IN Tag
                room_name,
                booking.room_id,
                room_location,  // 🏔️ Ort hinzugefügt!
                guest_name,
                booking.checkout_date,  // Original Checkout bleibt gespeichert
                booking.checkin_date,   // Check-IN Zeit
                notes_str,
                booking.anzahl_gaeste,
                extras_escaped,
                emojis_start_escaped
            );

            sql_statements.push(insert_sql);
            task_count += 1;
        }
    }

    // 2️⃣ Verarbeite Check-OUT Tasks (emojis_end + Reinigung)
    for booking in &checkouts {
        // Filtere Services mit position="end"
        let service_emojis_end: Vec<String> = booking.services.iter()
            .filter(|s| s.show_in_cleaning_plan && s.cleaning_plan_position == "end" && s.emoji.is_some())
            .map(|s| s.emoji.as_ref().unwrap().clone())
            .collect();

        // Prüfe ob am selben Tag Check-in ist (Priorität)
        let same_day_checkin = crate::database::check_same_day_checkin(booking.room_id, &date)
            .unwrap_or(false);

        let priority = if same_day_checkin { "high" } else { "normal" };

        // Check-OUT Task erstellen (immer, weil Reinigung nötig)
        println!("\n┌───── CHECK-OUT Task ────────────────────────────────────────");
        println!("│ 📋 Booking #{}", booking.id);
        println!("│    Gast: {} {}", booking.guest.vorname, booking.guest.nachname);
        println!("│    Zimmer: {}", booking.room.name);
        println!("│    Check-OUT: {}", booking.checkout_date);
        println!("│    Emojis END: {}", service_emojis_end.join(","));
        println!("│    Same-day Check-in: {}", same_day_checkin);
        println!("└─────────────────────────────────────────────────────────────");

        let emojis_end_str = service_emojis_end.join(",");
        let room_name = booking.room.name.replace("'", "''");
        let guest_name = format!("{} {}", booking.guest.vorname, booking.guest.nachname).replace("'", "''");
        let emojis_end_escaped = emojis_end_str.replace("'", "''");

        let extras = serde_json::json!({
            "guest_count": booking.anzahl_gaeste,
            "original_checkin": booking.checkin_date,
            "original_checkout": booking.checkout_date,
            "emojis_end": service_emojis_end,
            "task_type": "checkout"
        }).to_string();
        let extras_escaped = extras.replace("'", "''");

        let notes_str = if let Some(ref notes) = booking.bemerkungen {
            format!("'{}'", notes.replace("'", "''"))
        } else {
            "NULL".to_string()
        };

        let checkin_time_str = if same_day_checkin {
            format!("'{}'", date)
        } else {
            "NULL".to_string()
        };

        // 🏔️ Room Location von booking.room.ort
        let room_location = booking.room.ort.as_ref().map(|s| s.replace("'", "''")).unwrap_or_default();

        // INSERT Check-OUT Task (mit room_location)
        let insert_sql = format!(
            "INSERT INTO cleaning_tasks (booking_id, date, room_name, room_id, room_location, guest_name, checkout_time, checkin_time, priority, notes, status, guest_count, extras, emojis_start, emojis_end) VALUES ({}, '{}', '{}', {}, '{}', '{}', '{}', {}, '{}', {}, 'pending', {}, '{}', '', '{}')",
            booking.id,
            date,  // Check-OUT Tag
            room_name,
            booking.room_id,
            room_location,  // 🏔️ Ort hinzugefügt!
            guest_name,
            booking.checkout_date,
            checkin_time_str,
            priority,
            notes_str,
            booking.anzahl_gaeste,
            extras_escaped,
            emojis_end_escaped
        );

        sql_statements.push(insert_sql);
        task_count += 1;
    }

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🚀 [sync_cleaning_tasks] Führe Batch aus mit {} Statements ({} Tasks)", sql_statements.len(), task_count);
    println!("═══════════════════════════════════════════════════════════════\n");

    // BATCH EXECUTION - alle Statements in EINEM Request!
    execute_turso_batch(sql_statements).await?;

    println!("✅ [sync_cleaning_tasks] Erfolgreich! {} Aufgaben synchronisiert\n", task_count);

    Ok(format!("✅ {} Aufgaben synchronisiert", task_count))
}

/// Synchronisiert automatisch alle Buchungen der nächsten 90 Tage (3 Monate)
#[tauri::command]
pub async fn sync_week_ahead() -> Result<String, String> {
    let mut total_synced = 0;
    let mut results = Vec::new();

    for days_ahead in 0..90 {
        let date = chrono::Local::now() + chrono::Duration::days(days_ahead);
        let date_str = date.format("%Y-%m-%d").to_string();

        match sync_cleaning_tasks(date_str.clone()).await {
            Ok(msg) => {
                // Parse die Anzahl aus der Nachricht
                if let Some(count_str) = msg.split_whitespace().nth(1) {
                    if let Ok(count) = count_str.parse::<i32>() {
                        if count > 0 {
                            results.push(format!("{}: {} Aufgaben", date_str, count));
                        }
                        total_synced += count;
                    }
                }
            }
            Err(e) => {
                results.push(format!("{}: Fehler - {}", date_str, e));
            }
        }
    }

    let summary = if results.len() > 10 {
        format!("Zeige erste 10 von {} Tagen:\n{}", results.len(), results[..10].join("\n"))
    } else {
        results.join("\n")
    };

    Ok(format!("✅ Gesamt {} Aufgaben für 90 Tage (3 Monate) synchronisiert\n\n{}", total_synced, summary))
}

/// Synchronisiert spezifische Daten (für Auto-Sync nach Buchungsänderungen)
/// WICHTIG: Synchronisiert ALTE + NEUE Daten bei Updates!
#[tauri::command]
pub async fn sync_affected_dates(old_checkout: Option<String>, new_checkout: String) -> Result<String, String> {
    println!("🔄 [sync_affected_dates] old_checkout: {:?}, new_checkout: {}", old_checkout, new_checkout);

    let mut synced_dates = Vec::new();

    // 1. Synchronisiere ALTES checkout_date (falls vorhanden)
    if let Some(ref old_date) = old_checkout {
        if old_date != &new_checkout {
            println!("🧹 [sync_affected_dates] Synchronisiere altes Datum: {}", old_date);
            match sync_cleaning_tasks(old_date.clone()).await {
                Ok(msg) => {
                    synced_dates.push(format!("{} (alt)", old_date));
                    println!("✅ [sync_affected_dates] Alt-Sync OK: {}", msg);
                }
                Err(e) => {
                    println!("❌ [sync_affected_dates] Alt-Sync Error: {}", e);
                    // Fehler nicht abbrechen - versuche wenigstens neues Datum
                }
            }
        }
    }

    // 2. Synchronisiere NEUES checkout_date
    println!("🧹 [sync_affected_dates] Synchronisiere neues Datum: {}", new_checkout);
    match sync_cleaning_tasks(new_checkout.clone()).await {
        Ok(msg) => {
            synced_dates.push(format!("{} (neu)", new_checkout));
            println!("✅ [sync_affected_dates] Neu-Sync OK: {}", msg);
        }
        Err(e) => {
            println!("❌ [sync_affected_dates] Neu-Sync Error: {}", e);
            return Err(format!("Fehler beim Sync des neuen Datums: {}", e));
        }
    }

    Ok(format!("✅ Synchronisiert: {}", synced_dates.join(", ")))
}

/// Struct für Cleaning Stats
#[derive(Debug, Serialize)]
pub struct CleaningStats {
    pub today: i32,
    pub tomorrow: i32,
    pub this_week: i32,
    pub total: i32,
}

/// Hole Cleaning Stats von Turso
#[tauri::command]
pub async fn get_cleaning_stats() -> Result<CleaningStats, String> {
    println!("🔍 [get_cleaning_stats] Command aufgerufen");
    let client = Client::new();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let tomorrow = (chrono::Local::now() + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
    let week_end = (chrono::Local::now() + chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
    let three_months_end = (chrono::Local::now() + chrono::Duration::days(90)).format("%Y-%m-%d").to_string();

    // Query für alle Stats auf einmal
    let sql = format!(
        "SELECT
            SUM(CASE WHEN date = '{}' THEN 1 ELSE 0 END) as today,
            SUM(CASE WHEN date = '{}' THEN 1 ELSE 0 END) as tomorrow,
            SUM(CASE WHEN date >= '{}' AND date <= '{}' THEN 1 ELSE 0 END) as this_week,
            COUNT(*) as total
        FROM cleaning_tasks",
        today, tomorrow, today, week_end
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

    println!("📦 [get_cleaning_stats] Turso Response: {}", text);

    // Parse Turso v2 Response
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("JSON Parse Error: {}", e))?;

    if let Some(results) = json.get("results") {
        if let Some(first_result) = results.get(0) {
            if let Some(response_obj) = first_result.get("response") {
                if let Some(result) = response_obj.get("result") {
                    if let Some(rows) = result.get("rows") {
                        if let Some(row) = rows.get(0) {
                            // Turso v2 Format: [{type: "integer", value: "5"}, ...]
                            // Helper function um Integer aus Turso value zu extrahieren
                            let parse_value = |val: &serde_json::Value| -> i32 {
                                if let Some(s) = val.as_str() {
                                    let parsed = s.parse().unwrap_or(0);
                                    println!("🔢 [get_cleaning_stats] Parsed string '{}' to {}", s, parsed);
                                    parsed
                                } else if let Some(n) = val.as_i64() {
                                    println!("🔢 [get_cleaning_stats] Got i64: {}", n);
                                    n as i32
                                } else {
                                    println!("⚠️  [get_cleaning_stats] Value is neither string nor i64: {:?}", val);
                                    0
                                }
                            };

                            let today = row.get(0)
                                .and_then(|v| v.get("value"))
                                .map(parse_value)
                                .unwrap_or(0);
                            let tomorrow = row.get(1)
                                .and_then(|v| v.get("value"))
                                .map(parse_value)
                                .unwrap_or(0);
                            let this_week = row.get(2)
                                .and_then(|v| v.get("value"))
                                .map(parse_value)
                                .unwrap_or(0);
                            let total = row.get(3)
                                .and_then(|v| v.get("value"))
                                .map(parse_value)
                                .unwrap_or(0);

                            let stats = CleaningStats {
                                today,
                                tomorrow,
                                this_week,
                                total,
                            };
                            println!("✅ [get_cleaning_stats] Stats berechnet: {:?}", stats);
                            return Ok(stats);
                        }
                    }
                }
            }
        }
    }

    Err("Keine Stats gefunden".to_string())
}

/// Migriert die Turso cleaning_tasks Tabelle zum neuen Schema mit booking_id
/// ACHTUNG: Löscht alle existierenden Daten!
#[tauri::command]
pub async fn migrate_cleaning_tasks_schema() -> Result<String, String> {
    println!("🔄 [migrate_cleaning_tasks_schema] Starte Schema-Migration...");

    let mut sql_statements = Vec::new();

    // 1. DROP alte Tabelle
    sql_statements.push("DROP TABLE IF EXISTS cleaning_tasks".to_string());

    // 2. CREATE neue Tabelle mit booking_id UND Emoji-Spalten UND room_location! 🎉
    sql_statements.push(
        "CREATE TABLE cleaning_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            booking_id INTEGER NOT NULL,
            date TEXT NOT NULL,
            room_name TEXT NOT NULL,
            room_id INTEGER NOT NULL,
            room_location TEXT,
            guest_name TEXT NOT NULL,
            checkout_time TEXT NOT NULL,
            checkin_time TEXT,
            priority TEXT NOT NULL,
            notes TEXT,
            status TEXT NOT NULL,
            guest_count INTEGER NOT NULL,
            extras TEXT NOT NULL,
            emojis_start TEXT NOT NULL DEFAULT '',
            emojis_end TEXT NOT NULL DEFAULT '',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )".to_string()
    );

    // 3. Index für Performance
    sql_statements.push("CREATE INDEX idx_cleaning_tasks_date ON cleaning_tasks(date)".to_string());
    sql_statements.push("CREATE INDEX idx_cleaning_tasks_booking_id ON cleaning_tasks(booking_id)".to_string());
    sql_statements.push("CREATE INDEX idx_cleaning_tasks_room_id ON cleaning_tasks(room_id)".to_string());

    println!("📝 [migrate_cleaning_tasks_schema] Führe {} SQL Statements aus", sql_statements.len());

    // Batch Execution
    execute_turso_batch(sql_statements).await?;

    println!("✅ [migrate_cleaning_tasks_schema] Schema erfolgreich migriert!");

    Ok("✅ Schema erfolgreich migriert! Führe jetzt einen 3-Monats-Sync durch um Daten zu füllen.".to_string())
}

/// 🧪 TEST COMMAND: Fragt Turso ab und zeigt Emoji-Daten
#[tauri::command]
pub async fn test_emoji_sync() -> Result<String, String> {
    println!("🧪 [test_emoji_sync] Starte Test-Abfrage...");

    let client = Client::new();

    // Query die ersten 10 Aufgaben mit Emojis
    let sql = "SELECT booking_id, room_name, guest_name, date, emojis_start, emojis_end FROM cleaning_tasks ORDER BY date LIMIT 15".to_string();

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

    println!("📦 [test_emoji_sync] Turso Response: {}", text);

    // Parse Response
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("JSON Parse Error: {}", e))?;

    let mut result = String::from("🧪 Emoji-Sync Test Ergebnisse:\n\n");
    let mut emoji_found_count = 0;
    let mut total_count = 0;

    if let Some(results) = json.get("results") {
        if let Some(first_result) = results.get(0) {
            if let Some(response_obj) = first_result.get("response") {
                if let Some(result_obj) = response_obj.get("result") {
                    if let Some(rows) = result_obj.get("rows") {
                        if let Some(rows_array) = rows.as_array() {
                            total_count = rows_array.len();

                            for (idx, row) in rows_array.iter().enumerate() {
                                if let Some(row_array) = row.as_array() {
                                    let booking_id = row_array.get(0).and_then(|v| v.get("value")).and_then(|v| v.as_str()).unwrap_or("");
                                    let room_name = row_array.get(1).and_then(|v| v.get("value")).and_then(|v| v.as_str()).unwrap_or("");
                                    let guest_name = row_array.get(2).and_then(|v| v.get("value")).and_then(|v| v.as_str()).unwrap_or("");
                                    let date = row_array.get(3).and_then(|v| v.get("value")).and_then(|v| v.as_str()).unwrap_or("");
                                    let emojis_start = row_array.get(4).and_then(|v| v.get("value")).and_then(|v| v.as_str()).unwrap_or("");
                                    let emojis_end = row_array.get(5).and_then(|v| v.get("value")).and_then(|v| v.as_str()).unwrap_or("");

                                    // Nur Zeilen mit Emojis anzeigen
                                    if !emojis_start.is_empty() || !emojis_end.is_empty() {
                                        emoji_found_count += 1;
                                        result.push_str(&format!(
                                            "{}. Booking #{} | {} | {}\n",
                                            emoji_found_count, booking_id, room_name, date
                                        ));
                                        result.push_str(&format!("   Gast: {}\n", guest_name));
                                        if !emojis_start.is_empty() {
                                            result.push_str(&format!("   ⬅️ START: {}\n", emojis_start));
                                        }
                                        if !emojis_end.is_empty() {
                                            result.push_str(&format!("   ➡️ END: {}\n", emojis_end));
                                        }
                                        result.push_str("\n");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    result.push_str(&format!("\n📊 Zusammenfassung:\n"));
    result.push_str(&format!("   Gesamt: {} Aufgaben in Turso\n", total_count));
    result.push_str(&format!("   Mit Emojis: {} Aufgaben\n", emoji_found_count));
    result.push_str(&format!("\n✅ Test erfolgreich!"));

    println!("✅ [test_emoji_sync] Test abgeschlossen");

    Ok(result)
}
