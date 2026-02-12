// ============================================================================
// POSTGRESQL LISTEN/NOTIFY - Real-Time Event Listener
// ============================================================================
// Created: 2025-11-16
// Updated: 2025-02-02 - Fixed notification handling with poll_message
// Purpose: Listen to PostgreSQL NOTIFY events and broadcast to frontend
// Architecture: PostgreSQL NOTIFY → Rust Listener → Tauri Events → Frontend
// ============================================================================

use futures_util::future::poll_fn;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio_postgres::{AsyncMessage, NoTls};

/// Database change event from PostgreSQL NOTIFY
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbChangeEvent {
    /// Table name (e.g., "bookings", "guests", "rooms")
    pub table: String,

    /// Action type: "INSERT", "UPDATE", "DELETE"
    pub action: String,

    /// ID of the affected record
    pub id: i32,

    /// Timestamp when the change occurred
    pub timestamp: String,
}

/// Lock change event from PostgreSQL NOTIFY (Presence System)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockChangeEvent {
    /// Action: "ACQUIRED" or "RELEASED"
    pub action: String,

    /// Booking ID that was locked/unlocked
    pub booking_id: i32,

    /// User who owns the lock
    pub user_name: String,

    /// Timestamp when locked (only for ACQUIRED)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_at: Option<String>,
}

/// Start PostgreSQL NOTIFY listener in background
///
/// WICHTIG: Verwendet separate Connection (nicht aus Pool!)
/// weil LISTEN erfordert eine persistente Connection.
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle for emitting events to frontend
/// * `connection_string` - PostgreSQL connection string
pub fn start_pg_listener(app_handle: AppHandle, connection_string: String) {
    tauri::async_runtime::spawn(async move {
        println!("🚀 [PgListener] Starting PostgreSQL LISTEN background task...");

        loop {
            match connect_and_listen(&app_handle, &connection_string).await {
                Ok(_) => {
                    println!("⚠️ [PgListener] Connection closed, reconnecting in 5s...");
                }
                Err(e) => {
                    eprintln!("❌ [PgListener] Connection error: {}", e);
                }
            }

            // Reconnect nach 5 Sekunden
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

/// Connect to PostgreSQL and listen for notifications
async fn connect_and_listen(
    app_handle: &AppHandle,
    connection_string: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Separate Connection für LISTEN (nicht aus Pool!)
    let (client, mut connection) = tokio_postgres::connect(connection_string, NoTls).await?;

    println!("✅ [PgListener] Connected to PostgreSQL");

    // LISTEN auf alle Channels - mit explizitem Error Handling
    println!("🔧 [PgListener] Executing LISTEN commands...");

    if let Err(e) = client.execute("LISTEN booking_changes", &[]).await {
        eprintln!("❌ [PgListener] LISTEN booking_changes failed: {}", e);
        return Err(Box::new(e));
    }
    println!("✅ [PgListener] LISTEN booking_changes successful");

    if let Err(e) = client.execute("LISTEN guest_changes", &[]).await {
        eprintln!("❌ [PgListener] LISTEN guest_changes failed: {}", e);
        return Err(Box::new(e));
    }
    println!("✅ [PgListener] LISTEN guest_changes successful");

    if let Err(e) = client.execute("LISTEN room_changes", &[]).await {
        eprintln!("❌ [PgListener] LISTEN room_changes failed: {}", e);
        return Err(Box::new(e));
    }
    println!("✅ [PgListener] LISTEN room_changes successful");

    if let Err(e) = client.execute("LISTEN table_changes", &[]).await {
        eprintln!("❌ [PgListener] LISTEN table_changes failed: {}", e);
        return Err(Box::new(e));
    }
    println!("✅ [PgListener] LISTEN table_changes successful");

    if let Err(e) = client.execute("LISTEN lock_changes", &[]).await {
        eprintln!("❌ [PgListener] LISTEN lock_changes failed: {}", e);
        return Err(Box::new(e));
    }
    println!("✅ [PgListener] LISTEN lock_changes successful");

    if let Err(e) = client.execute("LISTEN reminder_changes", &[]).await {
        eprintln!("❌ [PgListener] LISTEN reminder_changes failed: {}", e);
        return Err(Box::new(e));
    }
    println!("✅ [PgListener] LISTEN reminder_changes successful");

    println!("🎧 [PgListener] All LISTEN commands executed successfully - now polling for notifications");

    // Poll connection for notifications (this loop keeps the function alive)
    loop {
            match poll_fn(|cx| connection.poll_message(cx)).await {
                Some(Ok(AsyncMessage::Notification(notification))) => {
                        println!(
                            "📨 [PgListener] Received: {} -> {}",
                            notification.channel(),
                            notification.payload()
                        );

                        // Parse and emit events
                        if notification.channel() == "lock_changes" {
                            match serde_json::from_str::<LockChangeEvent>(notification.payload()) {
                                Ok(event) => {
                                    if let Err(e) = app_handle.emit("lock-change", &event) {
                                        eprintln!("❌ [PgListener] Failed to emit lock-change: {}", e);
                                    } else {
                                        println!("✅ [PgListener] Emitted lock-change: {:?}", event);
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "⚠️ [PgListener] Failed to parse lock payload: {} - Error: {}",
                                        notification.payload(),
                                        e
                                    );
                                }
                            }
                        } else {
                            println!("🔧 [DEBUG PgListener] Parsing payload: {}", notification.payload());
                            match serde_json::from_str::<DbChangeEvent>(notification.payload()) {
                                Ok(event) => {
                                    println!("🔧 [DEBUG PgListener] Parsed event: table={}, action={}, id={}", event.table, event.action, event.id);

                                    if event.table == "reminders" {
                                        println!("📨 [DEBUG PgListener] REMINDER EVENT DETECTED!");
                                        println!("   Action: {}", event.action);
                                        println!("   ID: {}", event.id);
                                        println!("   Timestamp: {}", event.timestamp);
                                    }

                                    if let Err(e) = app_handle.emit("db-change", &event) {
                                        eprintln!("❌ [PgListener] Failed to emit event: {}", e);
                                    } else {
                                        println!("✅ [PgListener] Emitted db-change: {:?}", event);
                                        if event.table == "reminders" {
                                            println!("✅ [DEBUG PgListener] Reminder event successfully emitted to frontend!");
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "⚠️ [PgListener] Failed to parse payload: {} - Error: {}",
                                        notification.payload(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Some(Ok(AsyncMessage::Notice(notice))) => {
                        println!("📝 [PgListener] Notice: {}", notice.message());
                    }
                    Some(Ok(_)) => {
                        // Other messages
                    }
                    Some(Err(e)) => {
                        eprintln!("❌ [PgListener] Connection error: {}", e);
                        return Err(Box::new(e));
                    }
                    None => {
                        println!("⚠️ [PgListener] Connection closed");
                        return Ok(());
                    }
                }
            }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = DbChangeEvent {
            table: "bookings".to_string(),
            action: "UPDATE".to_string(),
            id: 123,
            timestamp: "2025-11-16T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"table\":\"bookings\""));
        assert!(json.contains("\"action\":\"UPDATE\""));
        assert!(json.contains("\"id\":123"));
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{"table":"guests","action":"INSERT","id":456,"timestamp":"2025-02-02T12:00:00Z"}"#;
        let event: DbChangeEvent = serde_json::from_str(json).unwrap();

        assert_eq!(event.table, "guests");
        assert_eq!(event.action, "INSERT");
        assert_eq!(event.id, 456);
    }
}
