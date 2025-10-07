use serde::{Deserialize, Serialize};
use reqwest::Client;

// TODO: In Config auslagern oder Umgebungsvariablen
const TURSO_URL: &str = "https://dpolg-cleaning-maxwellbadger-1.aws-eu-west-1.turso.io";
const TURSO_TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NTk4NDI1MzcsImlkIjoiZjY1ZWY2YzMtYWNhMS00NjZiLWExYjgtODU0MTlmYjlmNDNiIiwicmlkIjoiMTRjNDc4YjAtYTAwMy00ZmZmLThiYTUtYTZhOWIwYjZiODdmIn0.JSyu72rlp3pQ_vFxozglKoV-XMHW12j_hVfhTKjbEGwSyWnWBq2kziJNx2WwvvwD09NU-TMoLLszq2Mm9OlLDw";

#[derive(Debug, Serialize, Deserialize)]
pub struct CleaningTask {
    pub date: String,
    pub room_name: String,
    pub room_id: i64,
    pub guest_name: String,
    pub checkout_time: String,
    pub checkin_time: Option<String>,
    pub priority: String, // "high", "normal", "low"
    pub notes: Option<String>,
    pub status: String, // "pending", "done"
    pub has_dog: bool,
    pub needs_bedding: bool,
    pub guest_count: i32,
    pub extras: String, // JSON als String
}

#[derive(Debug, Serialize)]
struct TursoRequest {
    statements: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TursoResponse {
    results: Vec<TursoResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TursoResult {
    success: bool,
    #[serde(default)]
    error: Option<String>,
}

/// Führt SQL-Statement auf Turso aus
async fn execute_turso_sql(sql: String) -> Result<(), String> {
    let client = Client::new();

    let request_body = TursoRequest {
        statements: vec![sql],
    };

    let response = client
        .post(format!("{}/v2/pipeline", TURSO_URL))
        .header("Authorization", format!("Bearer {}", TURSO_TOKEN))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP Request fehlgeschlagen: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
        return Err(format!("Turso Fehler {}: {}", status, error_text));
    }

    let turso_response: TursoResponse = response.json().await
        .map_err(|e| format!("JSON Parse Error: {}", e))?;

    for result in turso_response.results {
        if !result.success {
            if let Some(error) = result.error {
                return Err(format!("SQL Error: {}", error));
            }
        }
    }

    Ok(())
}

/// Synchronisiert Cleaning Tasks für ein bestimmtes Datum zu Turso
#[tauri::command]
pub async fn sync_cleaning_tasks(date: String) -> Result<String, String> {
    // Hole Buchungen für dieses Datum
    let checkouts = crate::database::get_bookings_by_checkout_date(&date)
        .map_err(|e| format!("Fehler beim Laden der Checkouts: {}", e))?;

    // Lösche alte Tasks für dieses Datum
    let delete_sql = format!("DELETE FROM cleaning_tasks WHERE date = '{}'", date);
    execute_turso_sql(delete_sql).await?;

    if checkouts.is_empty() {
        return Ok("Keine Aufgaben für dieses Datum".to_string());
    }

    // Erstelle Tasks aus Checkouts
    for booking in &checkouts {
        // Hole Services für diese Buchung
        let services = crate::database::get_booking_services(booking.id)
            .unwrap_or_default();

        let has_dog = services.iter().any(|s| s.service_name.contains("Hund"));
        let needs_bedding = services.iter().any(|s| s.service_name.contains("Bettwäsche"));

        // Prüfe ob am selben Tag Check-in ist (Priorität)
        let same_day_checkin = crate::database::check_same_day_checkin(booking.room_id, &date)
            .unwrap_or(false);

        let priority = if same_day_checkin { "high" } else { "normal" };

        // Baue extras JSON
        let service_names: Vec<String> = services.iter().map(|s| s.service_name.clone()).collect();
        let extras = serde_json::json!({
            "has_dog": has_dog,
            "needs_bedding": needs_bedding,
            "guest_count": booking.anzahl_gaeste,
            "services": service_names
        }).to_string();

        // SQL-Escape für Strings
        let room_name = booking.room.name.replace("'", "''");
        let guest_name = format!("{} {}", booking.guest.vorname, booking.guest.nachname).replace("'", "''");
        let checkout_time = booking.checkout_date.replace("'", "''");
        let checkin_time_str = if same_day_checkin {
            format!("'{}'", date.replace("'", "''"))
        } else {
            "NULL".to_string()
        };
        let notes_str = if let Some(ref notes) = booking.bemerkungen {
            format!("'{}'", notes.replace("'", "''"))
        } else {
            "NULL".to_string()
        };
        let extras_escaped = extras.replace("'", "''");

        // INSERT Statement
        let insert_sql = format!(
            "INSERT INTO cleaning_tasks (date, room_name, room_id, guest_name, checkout_time, checkin_time, priority, notes, status, has_dog, needs_bedding, guest_count, extras) VALUES ('{}', '{}', {}, '{}', '{}', {}, '{}', {}, 'pending', {}, {}, {}, '{}')",
            date,
            room_name,
            booking.room_id,
            guest_name,
            checkout_time,
            checkin_time_str,
            priority,
            notes_str,
            if has_dog { 1 } else { 0 },
            if needs_bedding { 1 } else { 0 },
            booking.anzahl_gaeste,
            extras_escaped
        );

        execute_turso_sql(insert_sql).await?;
    }

    Ok(format!("✅ {} Aufgaben synchronisiert", checkouts.len()))
}

/// Synchronisiert automatisch alle Buchungen der nächsten 7 Tage
#[tauri::command]
pub async fn sync_week_ahead() -> Result<String, String> {
    let mut total_synced = 0;
    let mut results = Vec::new();

    for days_ahead in 0..7 {
        let date = chrono::Local::now() + chrono::Duration::days(days_ahead);
        let date_str = date.format("%Y-%m-%d").to_string();

        match sync_cleaning_tasks(date_str.clone()).await {
            Ok(msg) => {
                results.push(format!("{}: {}", date_str, msg));
                // Parse die Anzahl aus der Nachricht
                if let Some(count_str) = msg.split_whitespace().nth(1) {
                    if let Ok(count) = count_str.parse::<i32>() {
                        total_synced += count;
                    }
                }
            }
            Err(e) => {
                results.push(format!("{}: Fehler - {}", date_str, e));
            }
        }
    }

    Ok(format!("✅ Gesamt {} Aufgaben für 7 Tage synchronisiert\n\n{}", total_synced, results.join("\n")))
}
