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

    // LISTEN auf alle Channels
    client.execute("LISTEN booking_changes", &[]).await?;
    client.execute("LISTEN guest_changes", &[]).await?;
    client.execute("LISTEN room_changes", &[]).await?;
    client.execute("LISTEN table_changes", &[]).await?;

    println!("🎧 [PgListener] Listening on channels: booking_changes, guest_changes, room_changes, table_changes");

    // Wichtig: Die Connection muss als Stream verarbeitet werden
    // poll_message() wartet auf die nächste Nachricht
    loop {
        match poll_fn(|cx| connection.poll_message(cx)).await {
            Some(Ok(AsyncMessage::Notification(notification))) => {
                println!(
                    "📨 [PgListener] Received: {} -> {}",
                    notification.channel(),
                    notification.payload()
                );

                // Parse JSON payload
                match serde_json::from_str::<DbChangeEvent>(notification.payload()) {
                    Ok(event) => {
                        // Emit to frontend via Tauri event system
                        if let Err(e) = app_handle.emit("db-change", &event) {
                            eprintln!("❌ [PgListener] Failed to emit event: {}", e);
                        } else {
                            println!("✅ [PgListener] Emitted db-change: {:?}", event);
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
            Some(Ok(AsyncMessage::Notice(notice))) => {
                // PostgreSQL NOTICE messages (info, warnings)
                println!("📝 [PgListener] Notice: {}", notice.message());
            }
            Some(Ok(_)) => {
                // Other async messages (ignore)
            }
            Some(Err(e)) => {
                eprintln!("❌ [PgListener] Connection error: {}", e);
                return Err(Box::new(e));
            }
            None => {
                // Connection geschlossen
                println!("⚠️ [PgListener] Connection closed by server");
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
