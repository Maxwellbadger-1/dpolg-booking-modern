# Enhanced Multi-User Migration Plan
## Mit Real-Time Updates & Optimierungen

---

## 🎯 ERWEITERTE ARCHITEKTUR

### Real-Time Synchronisation hinzugefügt:

```
┌─────────────────────┐
│ Desktop Client 1    │ ←─────┐
│ (Tauri + WebSocket) │       │
└─────────────────────┘       │
                              │
┌─────────────────────┐       │    ┌──────────────────────┐
│ Desktop Client 2    │ ←─────┼───→│  Oracle Cloud VM     │
│ (Tauri + WebSocket) │       │    │  - PostgreSQL DB     │
└─────────────────────┘       │    │  - WebSocket Server  │
                              │    │  - Redis (optional) │
┌─────────────────────┐       │    └──────────────────────┘
│ Desktop Client 3-5  │ ←─────┘         ↑
│ (Tauri + WebSocket) │                 │
└─────────────────────┘                 │
                                   Real-Time Updates
```

---

## 📋 ANGEPASSTER ZEITPLAN

| Phase | Original | NEU | Details |
|-------|----------|-----|---------|
| **1. Oracle Setup** | 4h | 4h | ✅ Gleich |
| **2. Rust Migration** | 2-3 Tage | **4-5 Tage** | 132 Commands! |
| **3. Real-Time System** | - | **1 Tag** | WebSockets |
| **4. Config System** | 4h | 4h | ✅ Gleich |
| **5. User Management** | 1 Tag | 1 Tag | ✅ Gleich |
| **6. Testing** | 1 Tag | **2 Tage** | Mehr Komplexität |
| **7. Migration** | 4h | 4h | ✅ Gleich |
| **GESAMT** | 5-6 Tage | **8-10 Tage** | Realistischer |

---

## 🔥 PHASE 0: VORBEREITUNG (NEU!)

### Bevor Sie starten:

#### 0.1 Dependency Check

```toml
# Cargo.toml - ALLE benötigten Dependencies
[dependencies]
# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio-native-tls",
    "postgres",
    "chrono",
    "uuid",
    "migrate"  # Für auto-migrations!
] }

# Async Runtime
tokio = { version = "1", features = ["full"] }

# WebSocket
tokio-tungstenite = "0.24"

# Authentication
argon2 = "0.5"
jsonwebtoken = "9"  # Für Session Tokens

# Existing
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
```

#### 0.2 TypeScript Types vorbereiten

```typescript
// src/types/multiuser.ts
export interface User {
  id: number;
  username: string;
  fullName: string | null;
  role: 'admin' | 'user';
  lastLogin: Date | null;
}

export interface Session {
  user: User;
  token: string;
  expiresAt: Date;
}

export interface RealtimeEvent {
  type: 'booking-created' | 'booking-updated' | 'booking-deleted' |
        'guest-updated' | 'room-updated' | 'reminder-created';
  payload: any;
  userId: number;
  timestamp: Date;
}
```

---

## 🚀 PHASE 2: RUST BACKEND MIGRATION (ERWEITERT)

### 2.1 Connection Pool Management

```rust
// src-tauri/src/db.rs (NEU!)
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::State;

pub struct AppState {
    pub db_pool: Pool<Postgres>,
    pub ws_clients: Arc<RwLock<Vec<WsClient>>>,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let db_pool = PgPoolOptions::new()
            .max_connections(20)  // Oracle VM: max 20
            .min_connections(5)   // Keep 5 alive
            .connect_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(600))
            .connect(database_url)
            .await?;

        Ok(Self {
            db_pool,
            ws_clients: Arc::new(RwLock::new(Vec::new())),
        })
    }
}
```

### 2.2 Migration Helper Script

```rust
// src-tauri/src/migrate.rs (NEU!)
use regex::Regex;

/// Konvertiert SQLite Query zu PostgreSQL
pub fn convert_query(sqlite_query: &str) -> String {
    let mut query = sqlite_query.to_string();

    // ?1, ?2 → $1, $2
    let re = Regex::new(r"\?(\d+)").unwrap();
    query = re.replace_all(&query, "$$$1").to_string();

    // AUTOINCREMENT → SERIAL
    query = query.replace("INTEGER PRIMARY KEY AUTOINCREMENT", "SERIAL PRIMARY KEY");
    query = query.replace("INTEGER PRIMARY KEY", "SERIAL PRIMARY KEY");

    // datetime('now') → CURRENT_TIMESTAMP
    query = query.replace("datetime('now')", "CURRENT_TIMESTAMP");

    // REAL → DOUBLE PRECISION
    query = query.replace(" REAL", " DOUBLE PRECISION");

    query
}

// Verwendung:
let old_query = "INSERT INTO bookings (guest_id, room_id) VALUES (?1, ?2)";
let new_query = convert_query(old_query);
// Result: "INSERT INTO bookings (guest_id, room_id) VALUES ($1, $2)"
```

### 2.3 Batch Command Conversion

```python
# tools/convert_commands.py (Helper Script)
import re
import os

def convert_rust_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Convert sync to async
    content = re.sub(
        r'#\[tauri::command\]\nfn (\w+)',
        r'#[tauri::command]\nasync fn \1',
        content
    )

    # Add pool parameter
    content = re.sub(
        r'\) -> Result<',
        r', pool: State<\'_, AppState>) -> Result<',
        content
    )

    # Convert Connection::open to pool
    content = re.sub(
        r'let conn = Connection::open\([^)]+\)[^;]+;',
        r'let pool = &pool.db_pool;',
        content
    )

    # Convert params! to bind
    content = re.sub(
        r'params!\[([^\]]+)\]',
        lambda m: '.bind(' + ').bind('.join(m.group(1).split(', ')) + ')',
        content
    )

    with open(filepath + '.converted', 'w') as f:
        f.write(content)

    print(f"Converted: {filepath}")

# Alle Rust-Dateien konvertieren
for root, dirs, files in os.walk('src-tauri/src'):
    for file in files:
        if file.endswith('.rs'):
            convert_rust_file(os.path.join(root, file))
```

---

## 🔄 PHASE 3: REAL-TIME UPDATES (NEU!)

### 3.1 WebSocket Server (Backend)

```rust
// src-tauri/src/websocket.rs
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct WsClient {
    pub user_id: i32,
    pub sender: Arc<RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>,
}

pub async fn start_websocket_server(port: u16, state: Arc<AppState>) {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind WebSocket port");

    println!("WebSocket server listening on port {}", port);

    while let Ok((stream, _)) = listener.accept().await {
        let state = state.clone();

        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("WebSocket handshake failed");

            let (sender, mut receiver) = ws_stream.split();
            let sender = Arc::new(RwLock::new(sender));

            // Erste Nachricht = Auth Token
            if let Some(Ok(Message::Text(token))) = receiver.next().await {
                if let Ok(user_id) = verify_token(&token) {
                    let client = WsClient { user_id, sender: sender.clone() };

                    // Client registrieren
                    state.ws_clients.write().await.push(client);

                    // Auf Nachrichten warten
                    while let Some(Ok(msg)) = receiver.next().await {
                        // Ping/Pong handling
                        if let Message::Ping(data) = msg {
                            let _ = sender.write().await.send(Message::Pong(data)).await;
                        }
                    }

                    // Client entfernen bei Disconnect
                    state.ws_clients.write().await.retain(|c| c.user_id != user_id);
                }
            }
        });
    }
}

// Broadcasting Helper
pub async fn broadcast_event(
    state: &AppState,
    event_type: &str,
    data: serde_json::Value,
    exclude_user: Option<i32>
) {
    let message = json!({
        "type": event_type,
        "data": data,
        "timestamp": chrono::Utc::now()
    });

    let clients = state.ws_clients.read().await;

    for client in clients.iter() {
        if exclude_user.map_or(true, |id| id != client.user_id) {
            let _ = client.sender
                .write()
                .await
                .send(Message::Text(message.to_string()))
                .await;
        }
    }
}
```

### 3.2 Frontend WebSocket Integration

```typescript
// src/hooks/useRealtimeSync.ts
import { useEffect, useRef, useCallback } from 'react';
import { useToast } from './useToast';

export function useRealtimeSync(
  token: string,
  onEvent: (event: RealtimeEvent) => void
) {
  const ws = useRef<WebSocket | null>(null);
  const reconnectTimer = useRef<NodeJS.Timeout>();
  const { showToast } = useToast();

  const connect = useCallback(() => {
    try {
      // Oracle VM IP hier
      ws.current = new WebSocket('ws://132.145.xxx.xxx:8080');

      ws.current.onopen = () => {
        console.log('WebSocket connected');
        // Authentifizierung
        ws.current?.send(token);
        showToast('Echtzeit-Sync verbunden', 'success');
      };

      ws.current.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          onEvent(data);
        } catch (error) {
          console.error('WebSocket parse error:', error);
        }
      };

      ws.current.onerror = (error) => {
        console.error('WebSocket error:', error);
        showToast('Verbindungsfehler', 'error');
      };

      ws.current.onclose = () => {
        console.log('WebSocket disconnected, reconnecting...');
        // Auto-reconnect nach 5 Sekunden
        reconnectTimer.current = setTimeout(connect, 5000);
      };
    } catch (error) {
      console.error('WebSocket connection failed:', error);
    }
  }, [token, onEvent, showToast]);

  useEffect(() => {
    connect();

    return () => {
      clearTimeout(reconnectTimer.current);
      ws.current?.close();
    };
  }, [connect]);

  // Ping alle 30 Sekunden für Keep-Alive
  useEffect(() => {
    const interval = setInterval(() => {
      if (ws.current?.readyState === WebSocket.OPEN) {
        ws.current.send(JSON.stringify({ type: 'ping' }));
      }
    }, 30000);

    return () => clearInterval(interval);
  }, []);
}
```

### 3.3 DataContext mit Real-Time Updates

```typescript
// src/context/DataContext.tsx (ERWEITERT)
export const DataProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [guests, setGuests] = useState<Guest[]>([]);
  const { user, token } = useAuth();

  // Real-time sync
  useRealtimeSync(token, (event) => {
    switch (event.type) {
      case 'booking-created':
        setBookings(prev => [...prev, event.payload]);
        break;

      case 'booking-updated':
        setBookings(prev => prev.map(b =>
          b.id === event.payload.id ? event.payload : b
        ));
        break;

      case 'booking-deleted':
        setBookings(prev => prev.filter(b => b.id !== event.payload.id));
        break;

      // ... andere Events
    }
  });

  // Initial load
  useEffect(() => {
    if (user) {
      loadAllData();
    }
  }, [user]);

  // Commands MIT Broadcasting
  const createBooking = useCallback(async (data: CreateBookingData) => {
    const booking = await invoke<Booking>('create_booking_command', {
      ...data,
      userId: user?.id  // User tracking!
    });

    // Optimistic update (wird durch WebSocket bestätigt)
    setBookings(prev => [...prev, booking]);

    return booking;
  }, [user]);

  // ... rest
};
```

---

## 🔒 PHASE 4: ENHANCED SECURITY

### 4.1 Row-Level Security (PostgreSQL)

```sql
-- Jeder User sieht nur seine Buchungen (optional)
CREATE POLICY user_bookings ON bookings
  FOR ALL
  USING (created_by = current_user_id());

-- Oder: Alle sehen alles (Standard für DPolG)
-- Keine Policy = Alle haben Zugriff
```

### 4.2 Audit Log

```sql
CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    table_name VARCHAR(50),
    operation VARCHAR(10),  -- INSERT, UPDATE, DELETE
    record_id INTEGER,
    old_data JSONB,
    new_data JSONB,
    user_id INTEGER,
    username VARCHAR(50),
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Trigger für Booking-Änderungen
CREATE OR REPLACE FUNCTION audit_booking_changes()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_log (table_name, operation, record_id, old_data, new_data, user_id, username)
    VALUES (
        'bookings',
        TG_OP,
        COALESCE(NEW.id, OLD.id),
        row_to_json(OLD),
        row_to_json(NEW),
        current_setting('app.user_id')::INTEGER,
        current_setting('app.username')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER booking_audit
AFTER INSERT OR UPDATE OR DELETE ON bookings
FOR EACH ROW EXECUTE FUNCTION audit_booking_changes();
```

---

## 🧪 PHASE 5: ERWEITERTE TESTS

### 5.1 Load Testing

```typescript
// tests/load-test.ts
import { spawn } from 'child_process';

async function simulateUser(userId: number) {
  console.log(`Starting user ${userId}...`);

  const process = spawn('npm', ['run', 'tauri', 'dev'], {
    env: {
      ...process.env,
      TEST_USER_ID: userId.toString()
    }
  });

  // Simuliere Actions
  setTimeout(() => {
    // Create booking
    // Update booking
    // Delete booking
  }, Math.random() * 5000);
}

// 5 User gleichzeitig
for (let i = 1; i <= 5; i++) {
  simulateUser(i);
}
```

### 5.2 Conflict Resolution Test

```typescript
// Test: 2 User bearbeiten gleiche Buchung
async function testConflict() {
  const bookingId = 123;

  // User 1
  const user1Update = invoke('update_booking_command', {
    id: bookingId,
    roomId: 5,
    version: 1  // Optimistic locking
  });

  // User 2 (gleichzeitig)
  const user2Update = invoke('update_booking_command', {
    id: bookingId,
    roomId: 7,
    version: 1
  });

  // Einer sollte fehlschlagen
  const results = await Promise.allSettled([user1Update, user2Update]);

  assert(results.filter(r => r.status === 'rejected').length === 1);
}
```

---

## 📊 PERFORMANCE OPTIMIERUNGEN

### Oracle VM Tuning

```bash
# PostgreSQL Tuning für 1GB RAM
sudo nano /etc/postgresql/14/main/postgresql.conf

# Optimierte Settings für Oracle Free Tier
shared_buffers = 256MB          # 25% von RAM
effective_cache_size = 768MB    # 75% von RAM
maintenance_work_mem = 64MB
work_mem = 4MB
max_connections = 30            # 5 User × 6 connections
checkpoint_completion_target = 0.9
wal_buffers = 8MB
default_statistics_target = 100
random_page_cost = 1.1          # SSD storage
```

### Indexes für Performance

```sql
-- Häufige Queries optimieren
CREATE INDEX idx_bookings_dates ON bookings(checkin_date, checkout_date);
CREATE INDEX idx_bookings_room ON bookings(room_id, checkin_date);
CREATE INDEX idx_bookings_guest ON bookings(guest_id);
CREATE INDEX idx_bookings_status ON bookings(status) WHERE status != 'storniert';

-- Reminders
CREATE INDEX idx_reminders_due ON reminders(due_date) WHERE completed = false;
CREATE INDEX idx_reminders_booking ON reminders(booking_id);

-- Audit Log
CREATE INDEX idx_audit_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_user ON audit_log(user_id);
```

---

## 🚀 DEPLOYMENT CHECKLIST

### Pre-Deployment
- [ ] Backup aktuelle SQLite DBs
- [ ] Oracle VM provisioniert
- [ ] PostgreSQL installiert & konfiguriert
- [ ] Firewall Ports geöffnet (5432, 8080)
- [ ] SSL Zertifikat (optional, aber empfohlen)

### Code Changes
- [ ] 132 Commands auf async migriert
- [ ] WebSocket Server implementiert
- [ ] Frontend Real-time Hooks
- [ ] Login System implementiert
- [ ] Config System fertig

### Testing
- [ ] Single-User Test bestanden
- [ ] Multi-User Test bestanden
- [ ] Conflict Resolution getestet
- [ ] Performance akzeptabel (< 200ms)
- [ ] WebSocket Reconnect funktioniert

### Go-Live
- [ ] Daten migriert (SQLite → PostgreSQL)
- [ ] Alle User Accounts erstellt
- [ ] Config.toml an Clients verteilt
- [ ] Monitoring aktiviert
- [ ] Backup Cron Job läuft

### Post-Deployment
- [ ] 24h Monitoring
- [ ] User Feedback sammeln
- [ ] Performance Baseline erstellen
- [ ] Dokumentation aktualisiert

---

## 💡 TIPPS AUS DER PRAXIS

1. **Start klein:** Erst 2 User testen, dann 5
2. **Backup alles:** Vor jeder Änderung
3. **Monitor früh:** Logging von Anfang an
4. **User Training:** Login-System erklären
5. **Fallback Plan:** SQLite Backup behalten

---

**Version:** 2.0 (Mit Real-Time & Optimierungen)
**Letzte Aktualisierung:** 2025-11-14
**Geschätzter Aufwand:** 8-10 Tage (realistischer)
**Status:** 📋 Bereit zur Umsetzung