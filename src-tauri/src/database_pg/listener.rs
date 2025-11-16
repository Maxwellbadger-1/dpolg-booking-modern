// ============================================================================
// POSTGRESQL LISTEN/NOTIFY - Real-Time Event Listener
// ============================================================================
// Created: 2025-11-16
// Purpose: Listen to PostgreSQL NOTIFY events and broadcast to frontend
// Architecture: PostgreSQL NOTIFY → Rust Listener → Tauri Events → Frontend
// ============================================================================

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_postgres::{Client, Error, NoTls};

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

    /// Old data (only for UPDATE, otherwise null)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_data: Option<serde_json::Value>,

    /// New data (for INSERT and UPDATE, otherwise null)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_data: Option<serde_json::Value>,
}

/// Type alias for the broadcast sender
pub type EventSender = Arc<broadcast::Sender<DbChangeEvent>>;

/// Type alias for the broadcast receiver
pub type EventReceiver = broadcast::Receiver<DbChangeEvent>;

/// PostgreSQL NOTIFY Listener
///
/// Connects to PostgreSQL and listens for NOTIFY events on specified channels.
/// Broadcasts received events to all subscribed frontends via Tauri events.
pub struct PgListener {
    client: Client,
    event_sender: EventSender,
}

impl PgListener {
    /// Create a new PostgreSQL listener
    ///
    /// # Arguments
    /// * `connection_string` - PostgreSQL connection string (e.g., "postgres://user:pass@host/db")
    /// * `event_sender` - Broadcast channel sender for distributing events
    ///
    /// # Returns
    /// Result with PgListener instance or connection error
    pub async fn new(
        connection_string: &str,
        event_sender: EventSender,
    ) -> Result<Self, Error> {
        println!("🔌 [PgListener] Connecting to PostgreSQL...");
        println!("   URL: {}", Self::sanitize_url(connection_string));

        // Connect to PostgreSQL
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        // Spawn connection handler in background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("❌ [PgListener] Connection error: {}", e);
            }
        });

        println!("✅ [PgListener] Connected to PostgreSQL");

        Ok(Self {
            client,
            event_sender,
        })
    }

    /// Start listening to PostgreSQL NOTIFY channels
    ///
    /// Subscribes to multiple channels and forwards notifications to the broadcast channel.
    /// This is a blocking operation that runs indefinitely.
    ///
    /// # Channels
    /// - `booking_changes` - Bookings table changes
    /// - `guest_changes` - Guests table changes
    /// - `room_changes` - Rooms table changes
    /// - `table_changes` - Fallback for other tables
    pub async fn listen(&self) -> Result<(), Error> {
        // Subscribe to all notification channels
        self.client.execute("LISTEN booking_changes", &[]).await?;
        self.client.execute("LISTEN guest_changes", &[]).await?;
        self.client.execute("LISTEN room_changes", &[]).await?;
        self.client.execute("LISTEN table_changes", &[]).await?;

        println!("✅ [PgListener] Subscribed to channels:");
        println!("   - booking_changes");
        println!("   - guest_changes");
        println!("   - room_changes");
        println!("   - table_changes");
        println!("");
        println!("🎧 [PgListener] Listening for PostgreSQL NOTIFY events...");

        // Listen loop - runs forever
        loop {
            // Wait for notification
            let notification = self.client.notifications().recv().await?;

            println!("📡 [NOTIFY] Received on channel '{}'", notification.channel());

            // Parse notification payload
            match serde_json::from_str::<DbChangeEvent>(notification.payload()) {
                Ok(event) => {
                    println!("   Table: {}, Action: {}, ID: {}", event.table, event.action, event.id);

                    // Broadcast to all subscribed frontends
                    match self.event_sender.send(event.clone()) {
                        Ok(receiver_count) => {
                            if receiver_count > 0 {
                                println!("   ✅ Broadcasted to {} frontend(s)", receiver_count);
                            } else {
                                println!("   ⚠️  No active frontends (event dropped)");
                            }
                        }
                        Err(e) => {
                            eprintln!("   ❌ Broadcast failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("   ❌ Failed to parse notification: {}", e);
                    eprintln!("   Raw payload: {}", notification.payload());
                }
            }
        }
    }

    /// Sanitize connection URL for logging (hide password)
    fn sanitize_url(url: &str) -> String {
        url.split('@')
            .enumerate()
            .map(|(i, part)| {
                if i == 0 {
                    // Before @: hide password
                    let parts: Vec<&str> = part.split(':').collect();
                    if parts.len() >= 3 {
                        format!("{}://{}:***", parts[0], parts[1].trim_start_matches("//"))
                    } else {
                        part.to_string()
                    }
                } else {
                    // After @: show as is
                    part.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("@")
    }
}

/// Start the PostgreSQL listener in a background task
///
/// This is a convenience function that creates a PgListener and starts listening
/// in a new Tokio task. It's designed to be called from the Tauri app setup.
///
/// # Arguments
/// * `connection_string` - PostgreSQL connection string
/// * `event_sender` - Broadcast channel sender
///
/// # Example
/// ```rust
/// let (event_sender, _) = broadcast::channel::<DbChangeEvent>(100);
/// let event_sender_arc = Arc::new(event_sender);
/// start_listener_background(&db_url, event_sender_arc);
/// ```
pub fn start_listener_background(connection_string: String, event_sender: EventSender) {
    tokio::spawn(async move {
        println!("🚀 [PgListener] Starting PostgreSQL LISTEN background task...");

        match PgListener::new(&connection_string, event_sender.clone()).await {
            Ok(listener) => {
                println!("✅ [PgListener] Listener created successfully");

                // Start listening (blocks forever)
                if let Err(e) = listener.listen().await {
                    eprintln!("❌ [PgListener] Listen error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("❌ [PgListener] Failed to create listener: {}", e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_url() {
        let url = "postgres://user:secret@localhost:5432/mydb";
        let sanitized = PgListener::sanitize_url(url);
        assert!(sanitized.contains("***"));
        assert!(!sanitized.contains("secret"));
        assert!(sanitized.contains("localhost:5432/mydb"));
    }

    #[test]
    fn test_event_serialization() {
        let event = DbChangeEvent {
            table: "bookings".to_string(),
            action: "UPDATE".to_string(),
            id: 123,
            timestamp: "2025-11-16T10:00:00Z".to_string(),
            old_data: None,
            new_data: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"table\":\"bookings\""));
        assert!(json.contains("\"action\":\"UPDATE\""));
        assert!(json.contains("\"id\":123"));
    }
}
