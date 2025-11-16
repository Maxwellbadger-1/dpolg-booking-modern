# 🔴 Real-Time Collaboration System - Design Document

**Status:** In Implementation
**Datum:** 2025-11-16
**Priorität:** TIER S - Critical for Multi-User Production

---

## 🎯 PROBLEM STATEMENT

**Aktuelle Situation:**
- User A öffnet Booking #123, ändert Preis
- User B öffnet GLEICHZEITIG Booking #123, ändert Gast
- User A speichert → ✅ OK (updated_at = T1)
- User B speichert → ❌ CONFLICT (updated_at ≠ T1)

**Resultat:**
- User B sieht rote Fehlermeldung: "Von anderem Benutzer geändert"
- User B muss F5 drücken, nochmal editieren, nochmal speichern
- **Schlechte User Experience!**

**Lösung:**
User B sieht SOFORT wenn User A speichert:
```
┌────────────────────────────────────────────┐
│ 👤 Anna hat diese Buchung gerade          │
│    aktualisiert (vor 2 Sekunden)           │
│                                             │
│ [Änderungen ansehen] [Ignorieren]          │
└────────────────────────────────────────────┘
```

---

## 🏗️ ARCHITEKTUR

### Component Overview

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│  Frontend 1  │         │  Frontend 2  │         │  Frontend N  │
│  (Browser)   │         │  (Browser)   │         │  (Browser)   │
└──────┬───────┘         └──────┬───────┘         └──────┬───────┘
       │                        │                        │
       │ EventSource (SSE)      │ EventSource (SSE)      │
       │                        │                        │
       └────────────────────────┼────────────────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │   Tauri Backend       │
                    │   (Rust)              │
                    │                       │
                    │  - SSE Endpoint       │
                    │  - Event Broadcaster  │
                    │  - Active Users Map   │
                    └───────────┬───────────┘
                                │
                                │ LISTEN
                                │
                                ▼
                    ┌───────────────────────┐
                    │   PostgreSQL 16       │
                    │                       │
                    │  - NOTIFY Triggers    │
                    │  - Event Channel      │
                    └───────────────────────┘
```

### Data Flow

**Szenario: User A speichert Booking #123**

```
1. User A: updateBooking(123, { price: 120 })
   ↓
2. Frontend A: invoke('update_booking_pg', {...})
   ↓
3. Tauri Backend: BookingRepository::update()
   ↓
4. PostgreSQL: UPDATE bookings SET price = 120 WHERE id = 123
   ↓
5. PostgreSQL Trigger: NOTIFY booking_changes '{"id": 123, "action": "update"}'
   ↓
6. Tauri Backend: LISTEN receives notification
   ↓
7. Tauri Backend: Broadcast to ALL connected SSE clients
   ↓
8. Frontend B, C, D: EventSource.onmessage() receives event
   ↓
9. Frontend B: Shows toast "Booking updated by Anna"
   ↓
10. Frontend B: Auto-refreshes booking data
```

---

## 🔧 IMPLEMENTATION PLAN

### Phase 1: PostgreSQL NOTIFY System (Backend)

**1.1 Create Notification Function**
```sql
-- Function that sends NOTIFY
CREATE OR REPLACE FUNCTION notify_booking_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Send notification with JSON payload
    PERFORM pg_notify(
        'booking_changes',
        json_build_object(
            'table', TG_TABLE_NAME,
            'action', TG_OP,
            'id', CASE
                WHEN TG_OP = 'DELETE' THEN OLD.id
                ELSE NEW.id
            END,
            'timestamp', CURRENT_TIMESTAMP
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

**1.2 Create Triggers**
```sql
-- Trigger on bookings table
CREATE TRIGGER booking_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON bookings
FOR EACH ROW EXECUTE FUNCTION notify_booking_change();

-- Trigger on guests table
CREATE TRIGGER guest_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON guests
FOR EACH ROW EXECUTE FUNCTION notify_booking_change();

-- Trigger on rooms table
CREATE TRIGGER room_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON rooms
FOR EACH ROW EXECUTE FUNCTION notify_booking_change();
```

**Datei:** `src-tauri/database_notifications.sql`

---

### Phase 2: Event Listener (Rust Backend)

**2.1 Add Dependencies**
```toml
# Cargo.toml
[dependencies]
tokio-postgres = { version = "0.7", features = ["with-serde_json-1"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
serde_json = "1.0"
```

**2.2 PostgreSQL Listener Module**

**Datei:** `src-tauri/src/database_pg/listener.rs`

```rust
use tokio_postgres::{AsyncMessage, Notification};
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbChangeEvent {
    pub table: String,
    pub action: String,  // INSERT, UPDATE, DELETE
    pub id: i32,
    pub timestamp: String,
}

pub type EventSender = broadcast::Sender<DbChangeEvent>;
pub type EventReceiver = broadcast::Receiver<DbChangeEvent>;

/// Start listening to PostgreSQL NOTIFY events
pub async fn start_listener(
    connection_string: &str,
    event_sender: EventSender,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to PostgreSQL
    let (client, mut connection) = tokio_postgres::connect(connection_string, tokio_postgres::NoTls).await?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("PostgreSQL connection error: {}", e);
        }
    });

    // Subscribe to notification channel
    client.execute("LISTEN booking_changes", &[]).await?;
    println!("✅ Subscribed to PostgreSQL NOTIFY channel: booking_changes");

    // Listen loop
    loop {
        while let Some(message) = connection.try_recv().await? {
            if let AsyncMessage::Notification(notif) = message {
                handle_notification(notif, &event_sender);
            }
        }
    }
}

fn handle_notification(notif: Notification, sender: &EventSender) {
    match serde_json::from_str::<DbChangeEvent>(&notif.payload()) {
        Ok(event) => {
            println!("📡 [NOTIFY] Received: {:?}", event);

            // Broadcast to all SSE clients
            if let Err(e) = sender.send(event) {
                eprintln!("Failed to broadcast event: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to parse notification: {}", e);
        }
    }
}
```

**2.3 Register Listener in main.rs**

```rust
// src-tauri/src/lib_pg.rs

use tokio::sync::broadcast;
use crate::database_pg::listener::{start_listener, EventSender};

pub fn run_pg() {
    // ... existing setup ...

    // Create broadcast channel for events (capacity: 100 buffered events)
    let (event_sender, _) = broadcast::channel::<DbChangeEvent>(100);

    // Clone sender for Tauri state
    let event_sender_clone = event_sender.clone();

    // Start PostgreSQL listener in background
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    tokio::spawn(async move {
        if let Err(e) = start_listener(&db_url, event_sender).await {
            eprintln!("❌ PostgreSQL listener error: {}", e);
        }
    });

    tauri::Builder::default()
        .manage(db_pool)
        .manage(event_sender_clone)  // ← NEW: Make event sender available to commands
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            subscribe_to_events_pg,  // ← NEW: SSE endpoint
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Phase 3: Server-Sent Events (SSE) Endpoint

**3.1 SSE Command**

**Datei:** `src-tauri/src/lib_pg.rs`

```rust
use tokio::sync::broadcast::Receiver;
use futures::StreamExt;

/// SSE endpoint for real-time updates
///
/// Usage from frontend:
/// ```typescript
/// const eventSource = new EventSource('/api/events');
/// eventSource.onmessage = (event) => {
///   const change = JSON.parse(event.data);
///   handleRealtimeChange(change);
/// };
/// ```
#[tauri::command]
async fn subscribe_to_events_pg(
    event_sender: State<'_, EventSender>,
    window: tauri::Window,
) -> Result<(), String> {
    println!("🔌 [SSE] New client connected");

    // Subscribe to event channel
    let mut receiver = event_sender.subscribe();

    // Spawn background task to forward events to frontend
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            // Emit event to frontend via Tauri events
            if let Err(e) = window.emit("db-change", &event) {
                eprintln!("Failed to emit event to frontend: {}", e);
                break;
            }
        }
        println!("🔌 [SSE] Client disconnected");
    });

    Ok(())
}
```

**ALTERNATIVE: HTTP SSE Endpoint (falls Tauri events nicht ausreichen)**

```rust
// Using axum for HTTP server
use axum::{
    response::sse::{Event, Sse},
    Router,
};
use futures::stream::{self, Stream};

async fn sse_handler(
    event_sender: State<EventSender>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = event_sender.subscribe();

    let stream = BroadcastStream::new(receiver)
        .filter_map(|event| async move {
            event.ok().map(|e| {
                Ok(Event::default()
                    .json_data(e)
                    .unwrap())
            })
        });

    Sse::new(stream)
}

// Mount in Tauri
let app = Router::new()
    .route("/api/events", get(sse_handler));
```

---

### Phase 4: Frontend Integration

**4.1 Event Listener Service**

**Datei:** `src/services/realtime.ts`

```typescript
import { listen } from '@tauri-apps/api/event';

export interface DbChangeEvent {
  table: string;
  action: 'INSERT' | 'UPDATE' | 'DELETE';
  id: number;
  timestamp: string;
}

export class RealtimeService {
  private listeners: Map<string, Set<(event: DbChangeEvent) => void>> = new Map();

  async start() {
    // Subscribe to Tauri events
    await invoke('subscribe_to_events_pg');

    // Listen for db-change events
    await listen('db-change', (event) => {
      const change = event.payload as DbChangeEvent;
      console.log('📡 [Realtime] Received change:', change);

      this.notifyListeners(change.table, change);
    });

    console.log('✅ [Realtime] Service started');
  }

  /**
   * Subscribe to changes on a specific table
   */
  on(table: string, callback: (event: DbChangeEvent) => void) {
    if (!this.listeners.has(table)) {
      this.listeners.set(table, new Set());
    }
    this.listeners.get(table)!.add(callback);

    // Return unsubscribe function
    return () => {
      this.listeners.get(table)?.delete(callback);
    };
  }

  private notifyListeners(table: string, event: DbChangeEvent) {
    this.listeners.get(table)?.forEach((callback) => {
      try {
        callback(event);
      } catch (error) {
        console.error('Error in realtime listener:', error);
      }
    });
  }
}

// Singleton instance
export const realtimeService = new RealtimeService();
```

**4.2 DataContext Integration**

**Datei:** `src/context/DataContext.tsx`

```typescript
import { realtimeService } from '../services/realtime';

export function DataProvider({ children }: { children: React.ReactNode }) {
  // ... existing state ...

  useEffect(() => {
    // Start realtime service
    realtimeService.start();

    // Subscribe to bookings changes
    const unsubBookings = realtimeService.on('bookings', (event) => {
      console.log('📡 [DataContext] Booking changed:', event);

      if (event.action === 'INSERT') {
        // Fetch new booking and add to list
        invoke<Booking>('get_booking_by_id_pg', { id: event.id })
          .then((booking) => {
            setBookings((prev) => [...prev, booking]);
            toast.info(`📋 Neue Buchung: ${booking.reservierungsnummer}`);
          });
      } else if (event.action === 'UPDATE') {
        // Refresh booking
        invoke<Booking>('get_booking_by_id_pg', { id: event.id })
          .then((booking) => {
            setBookings((prev) =>
              prev.map((b) => (b.id === event.id ? booking : b))
            );
            toast.info(`✏️ Buchung aktualisiert: ${booking.reservierungsnummer}`);
          });
      } else if (event.action === 'DELETE') {
        // Remove from list
        setBookings((prev) => prev.filter((b) => b.id !== event.id));
        toast.info(`🗑️ Buchung gelöscht`);
      }
    });

    // Subscribe to guests changes
    const unsubGuests = realtimeService.on('guests', (event) => {
      // Similar logic for guests
    });

    // Cleanup
    return () => {
      unsubBookings();
      unsubGuests();
    };
  }, []);

  // ... rest of DataContext ...
}
```

**4.3 UI Indicators**

**Datei:** `src/components/BookingManagement/BookingDetails.tsx`

```typescript
const [lastModified, setLastModified] = useState<string | null>(null);

useEffect(() => {
  const unsub = realtimeService.on('bookings', (event) => {
    if (event.id === booking.id && event.action === 'UPDATE') {
      setLastModified('Just now');

      // Show notification banner
      setTimeout(() => setLastModified(null), 5000);
    }
  });

  return unsub;
}, [booking.id]);

// In JSX:
{lastModified && (
  <div className="bg-blue-100 border border-blue-400 text-blue-700 px-4 py-3 rounded relative mb-4">
    <strong>📡 Live Update:</strong> This booking was just modified by another user.
    <button onClick={handleRefresh} className="ml-4 underline">
      Refresh to see changes
    </button>
  </div>
)}
```

---

## 📊 PERFORMANCE CONSIDERATIONS

### Scalability

**Current Design:**
- Each frontend maintains 1 WebSocket/SSE connection
- Broadcast channel capacity: 100 buffered events
- PostgreSQL NOTIFY overhead: ~1-2ms per event

**Capacity:**
- **50 concurrent users:** ✅ No problem
- **100 concurrent users:** ✅ OK (test recommended)
- **500+ concurrent users:** ⚠️ Consider Redis Pub/Sub instead

### Memory Usage

- **Per connection:** ~10KB RAM
- **50 connections:** ~500KB RAM
- **Event buffer:** ~10KB (100 events × 100 bytes)

**Total overhead:** ~1MB for 50 users → **Negligible!**

---

## 🧪 TESTING PLAN

### Manual Test Scenario

1. Open App in 2 browser tabs (User A + User B)
2. User A: Edit Booking #123, change price 100€ → 120€, Save
3. User B: Should see toast notification immediately
4. User B: Booking should auto-refresh (or show "Refresh" button)
5. Verify: User B sees updated price without F5

### Automated Test (Playwright)

```typescript
test('real-time updates between users', async ({ page, context }) => {
  // User A
  const userA = page;
  await userA.goto('/bookings/123');

  // User B
  const userB = await context.newPage();
  await userB.goto('/bookings/123');

  // User A: Change price
  await userA.fill('[name="gesamtpreis"]', '120');
  await userA.click('button:text("Save")');

  // User B: Should see notification
  await expect(userB.locator('.toast-info')).toContainText('aktualisiert');

  // User B: Click refresh
  await userB.click('button:text("Refresh")');

  // User B: Should see new price
  await expect(userB.locator('[name="gesamtpreis"]')).toHaveValue('120');
});
```

---

## 📚 REFERENCES

- [PostgreSQL LISTEN/NOTIFY Docs](https://www.postgresql.org/docs/current/sql-notify.html)
- [tokio-postgres Notifications](https://docs.rs/tokio-postgres/latest/tokio_postgres/)
- [Server-Sent Events MDN](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events)
- [Tauri Events Guide](https://tauri.app/v1/guides/features/events/)

---

**Status:** 🟡 In Implementation
**Next Step:** Create PostgreSQL notification triggers
