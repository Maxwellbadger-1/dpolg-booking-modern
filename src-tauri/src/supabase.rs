use serde::{Deserialize, Serialize};
use reqwest::Client;

// TODO: In Config auslagern oder Umgebungsvariablen
const SUPABASE_URL: &str = "https://your-project.supabase.co";
const SUPABASE_KEY: &str = "your-anon-key-here";

#[derive(Debug, Serialize, Deserialize)]
pub struct CleaningTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub date: String,
    pub room_name: String,
    pub room_id: Option<i64>,
    pub guest_name: Option<String>,
    pub checkout_time: Option<String>,
    pub checkin_time: Option<String>,
    pub priority: String, // "high", "normal", "low"
    pub notes: Option<String>,
    pub status: String, // "pending", "done"
    pub has_dog: bool,
    pub needs_bedding: bool,
    pub guest_count: i32,
    pub extras: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// Synchronisiert Cleaning Tasks für ein bestimmtes Datum zu Supabase
#[tauri::command]
pub async fn sync_cleaning_tasks(date: String) -> Result<String, String> {
    // Hole Buchungen für dieses Datum
    let checkouts = crate::database::get_bookings_by_checkout_date(&date)
        .map_err(|e| format!("Fehler beim Laden der Checkouts: {}", e))?;

    let client = Client::new();

    // Lösche alte Tasks für dieses Datum
    let delete_url = format!("{}/rest/v1/cleaning_tasks?date=eq.{}", SUPABASE_URL, date);
    let _ = client
        .delete(&delete_url)
        .header("apikey", SUPABASE_KEY)
        .header("Authorization", format!("Bearer {}", SUPABASE_KEY))
        .send()
        .await
        .map_err(|e| format!("Löschen fehlgeschlagen: {}", e))?;

    if checkouts.is_empty() {
        return Ok("Keine Aufgaben für dieses Datum".to_string());
    }

    // Erstelle Tasks aus Checkouts
    let mut tasks = Vec::new();

    for booking in checkouts {
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
        let mut extras_map = serde_json::Map::new();
        extras_map.insert("has_dog".to_string(), serde_json::json!(has_dog));
        extras_map.insert("needs_bedding".to_string(), serde_json::json!(needs_bedding));
        extras_map.insert("guest_count".to_string(), serde_json::json!(booking.anzahl_gaeste));

        let service_names: Vec<String> = services.iter().map(|s| s.service_name.clone()).collect();
        extras_map.insert("services".to_string(), serde_json::json!(service_names));

        let task = CleaningTask {
            id: None,
            date: date.clone(),
            room_name: booking.room.name.clone(),
            room_id: Some(booking.room_id),
            guest_name: Some(format!("{} {}", booking.guest.vorname, booking.guest.nachname)),
            checkout_time: Some(booking.checkout_date.clone()),
            checkin_time: if same_day_checkin { Some(date.clone()) } else { None },
            priority: priority.to_string(),
            notes: booking.bemerkungen.clone(),
            status: "pending".to_string(),
            has_dog,
            needs_bedding,
            guest_count: booking.anzahl_gaeste,
            extras: serde_json::Value::Object(extras_map),
            completed_at: None,
            created_at: None,
        };

        tasks.push(task);
    }

    // Sende an Supabase
    let create_url = format!("{}/rest/v1/cleaning_tasks", SUPABASE_URL);
    let response = client
        .post(&create_url)
        .header("apikey", SUPABASE_KEY)
        .header("Authorization", format!("Bearer {}", SUPABASE_KEY))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&tasks)
        .send()
        .await
        .map_err(|e| format!("Upload fehlgeschlagen: {}", e))?;

    if response.status().is_success() {
        Ok(format!("✅ {} Aufgaben synchronisiert", tasks.len()))
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unbekannter Fehler".to_string());
        Err(format!("Fehler {}: {}", status, error_text))
    }
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
                if let Some(count) = msg.split_whitespace().next().and_then(|s| s.parse::<i32>().ok()) {
                    total_synced += count;
                }
            }
            Err(e) => {
                results.push(format!("{}: Fehler - {}", date_str, e));
            }
        }
    }

    Ok(format!("✅ Gesamt {} Aufgaben für 7 Tage synchronisiert\n\n{}", total_synced, results.join("\n")))
}
