# Multi-User Migration Plan
## DPolG Buchungssystem - SQLite → PostgreSQL auf Oracle Cloud

---

## 🎯 Ziel

Desktop Tauri App **multi-user-fähig** machen:
- Alle Clients installieren die **gleiche Desktop App**
- Zentrale **PostgreSQL Datenbank** auf Oracle Cloud Free Tier
- **5 gleichzeitige User** möglich
- **Keine Web-App** nötig - bleibt Desktop-only!

---

## 📊 Aktuelle vs. Zukünftige Architektur

### **VORHER (Jetzt):**
```
┌─────────────────────┐
│ Desktop Client      │
│ (Tauri App)         │
│   └─ SQLite (lokal) │
└─────────────────────┘
```
**Problem:** Jeder Client hat seine eigene Datenbank → Keine Synchronisation

### **NACHHER (Multi-User):**
```
┌─────────────────────┐
│ Desktop Client 1    │ ──┐
│ (Tauri App)         │   │
└─────────────────────┘   │
                          │
┌─────────────────────┐   │    ┌──────────────────────┐
│ Desktop Client 2    │ ──┼───→│  Oracle Cloud VM     │
│ (Tauri App)         │   │    │  PostgreSQL Database │
└─────────────────────┘   │    └──────────────────────┘
                          │
┌─────────────────────┐   │
│ Desktop Client 3-5  │ ──┘
│ (Tauri App)         │
└─────────────────────┘
```
**Lösung:** Alle Clients verbinden sich zu einer zentralen PostgreSQL DB

---

## ⏱️ Geschätzter Aufwand

| Phase | Aufwand | Details |
|-------|---------|---------|
| **1. Oracle Cloud Setup** | 4 Std | VM + PostgreSQL + Firewall |
| **2. Rust Backend Migration** | 2-3 Tage | `rusqlite` → `sqlx` + Query-Anpassungen |
| **3. Config-System** | 4 Std | DB-Verbindung konfigurierbar machen |
| **4. User Management** | 1 Tag | Login-System + User-Rollen |
| **5. Testing** | 1 Tag | Multi-Client Tests + Debugging |
| **6. Daten-Migration** | 4 Std | SQLite → PostgreSQL Transfer |
| **7. Deployment** | 4 Std | Production Setup + Client-Rollout |
| **GESAMT** | **5-6 Tage** | ~1 Woche konzentrierte Arbeit |

**Mit Zeiterfassungs-Erfahrung als Vorlage:** Noch schneller!

---

## 📋 Detaillierter Migrations-Plan

### **Phase 1: Oracle Cloud Setup (4 Std)**

#### 1.1 VM erstellen
- Login zu Oracle Cloud Console
- Compute → Instances → Create Instance
- Shape: **VM.Standard.E2.1.Micro** (Always Free)
- Image: **Ubuntu 22.04 LTS**
- Public IP: Notieren!

#### 1.2 PostgreSQL installieren
```bash
# SSH in die VM
ssh ubuntu@<oracle-vm-ip>

# PostgreSQL installieren
sudo apt update
sudo apt install -y postgresql postgresql-contrib

# PostgreSQL starten
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### 1.3 Datenbank erstellen
```bash
# Als postgres User
sudo -u postgres psql

-- Datenbank + User erstellen
CREATE DATABASE dpolg_booking;
CREATE USER booking_user WITH ENCRYPTED PASSWORD 'SICHERES_PASSWORD';
GRANT ALL PRIVILEGES ON DATABASE dpolg_booking TO booking_user;

-- Extensions
\c dpolg_booking
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

\q
```

#### 1.4 PostgreSQL für Remote-Zugriff konfigurieren
```bash
# postgresql.conf editieren
sudo nano /etc/postgresql/14/main/postgresql.conf

# Ändern:
listen_addresses = '*'

# pg_hba.conf editieren
sudo nano /etc/postgresql/14/main/pg_hba.conf

# Hinzufügen:
host    dpolg_booking    booking_user    0.0.0.0/0    scram-sha-256

# PostgreSQL neustarten
sudo systemctl restart postgresql
```

#### 1.5 Oracle Cloud Firewall öffnen
- Networking → Virtual Cloud Networks
- Security Lists → Default Security List
- Ingress Rule hinzufügen:
  - Source: `0.0.0.0/0`
  - Port: `5432` (PostgreSQL)
  - Protocol: TCP

#### 1.6 Ubuntu Firewall
```bash
sudo ufw allow 5432/tcp
sudo ufw enable
```

---

### **Phase 2: Rust Backend Migration (2-3 Tage)**

#### 2.1 Dependencies ändern

**Cargo.toml:**
```toml
[dependencies]
# ENTFERNEN:
# rusqlite = { version = "0.32.1", features = ["bundled"] }

# HINZUFÜGEN:
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "postgres", "chrono"] }
tokio = { version = "1", features = ["full"] }
```

#### 2.2 Database Connection (database.rs)

**VORHER:**
```rust
use rusqlite::{Connection, params, Result};

pub fn init_database() -> Result<Connection> {
    let conn = Connection::open("booking.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS bookings (...)",
        [],
    )?;

    Ok(conn)
}
```

**NACHHER:**
```rust
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::sync::Arc;
use tokio::sync::RwLock;

// Globaler Pool
pub type DbPool = Arc<RwLock<Pool<Postgres>>>;

pub async fn init_database(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    // Connection Pool erstellen
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    // Tabellen erstellen (PostgreSQL Syntax!)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bookings (
            id SERIAL PRIMARY KEY,
            guest_id INTEGER,
            room_id INTEGER,
            checkin_date DATE NOT NULL,
            checkout_date DATE NOT NULL,
            status VARCHAR(50),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
```

#### 2.3 Queries anpassen

**SQLite → PostgreSQL Unterschiede:**

| SQLite | PostgreSQL |
|--------|------------|
| `INTEGER PRIMARY KEY` | `SERIAL PRIMARY KEY` |
| `AUTOINCREMENT` | `SERIAL` |
| `?1, ?2, ?3` | `$1, $2, $3` |
| `datetime('now')` | `CURRENT_TIMESTAMP` |
| `strftime(...)` | `to_char(...)` |

**Beispiel Query-Anpassung:**

```rust
// VORHER (rusqlite):
conn.execute(
    "INSERT INTO bookings (guest_id, room_id, checkin_date) VALUES (?1, ?2, ?3)",
    params![guest_id, room_id, checkin],
)?;

// NACHHER (sqlx):
sqlx::query(
    "INSERT INTO bookings (guest_id, room_id, checkin_date) VALUES ($1, $2, $3)"
)
.bind(guest_id)
.bind(room_id)
.bind(checkin)
.execute(&pool)
.await?;
```

#### 2.4 Betroffene Dateien

Alle `.rs` Dateien mit DB-Zugriff müssen angepasst werden:

- `src-tauri/src/database.rs` - Connection + Schema
- `src-tauri/src/pricing.rs` - Pricing Queries (WICHTIG: Schema-Fix!)
- `src-tauri/src/main.rs` - Tauri Commands
- `src-tauri/src/lib.rs` - Exports
- Alle anderen Command-Handler

**Geschätzt:** ~30 Dateien, ~500 Zeilen Code

---

### **Phase 3: Config-System (4 Std)**

#### 3.1 Config-Datei erstellen

**config.toml** (neben der .exe):
```toml
[database]
host = "132.145.xxx.xxx"  # Oracle Cloud VM IP
port = 5432
database = "dpolg_booking"
username = "booking_user"
password = "SICHERES_PASSWORD"
ssl_mode = "prefer"  # Optional: require für Produktion
```

#### 3.2 Config laden (Rust)

**src-tauri/src/config.rs** (NEU):
```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub database: DatabaseConfig,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = "config.toml";
        let contents = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }
}
```

#### 3.3 App-Start mit Config

**src-tauri/src/main.rs:**
```rust
use tauri::Manager;

#[tokio::main]
async fn main() {
    // Config laden
    let config = Config::load()
        .expect("Fehler beim Laden der Konfiguration!");

    let database_url = config.database_url();

    // Datenbank initialisieren
    let pool = init_database(&database_url)
        .await
        .expect("Datenbankverbindung fehlgeschlagen!");

    tauri::Builder::default()
        .manage(Arc::new(RwLock::new(pool)))
        .invoke_handler(tauri::generate_handler![
            // ... alle Commands
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der App");
}
```

#### 3.4 Settings-Dialog (Optional, schöner!)

Statt manueller `config.toml` → Settings im UI:

```typescript
// src/components/Settings/DatabaseSettings.tsx
export function DatabaseSettings() {
  const [config, setConfig] = useState({
    host: '',
    port: 5432,
    database: 'dpolg_booking',
    username: 'booking_user',
    password: ''
  });

  const testConnection = async () => {
    try {
      await invoke('test_database_connection', config);
      alert('Verbindung erfolgreich!');
    } catch (err) {
      alert('Verbindung fehlgeschlagen: ' + err);
    }
  };

  const saveConfig = async () => {
    await invoke('save_database_config', config);
    alert('Konfiguration gespeichert!');
  };

  return (
    <div className="p-6">
      <h2>Datenbankverbindung</h2>
      <input
        value={config.host}
        onChange={e => setConfig({...config, host: e.target.value})}
        placeholder="Server-IP"
      />
      {/* ... weitere Felder */}
      <button onClick={testConnection}>Verbindung testen</button>
      <button onClick={saveConfig}>Speichern</button>
    </div>
  );
}
```

---

### **Phase 4: User Management (1 Tag)**

#### 4.1 User-Tabelle erstellen

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(100),
    email VARCHAR(100),
    role VARCHAR(20) DEFAULT 'user',  -- 'admin' oder 'user'
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);

-- Standard Admin-User erstellen (Passwort: admin123)
INSERT INTO users (username, password_hash, full_name, role)
VALUES ('admin', '$argon2id$...', 'Administrator', 'admin');
```

#### 4.2 Passwort-Hashing (Rust)

**Cargo.toml:**
```toml
[dependencies]
argon2 = "0.5"
```

**src-tauri/src/auth.rs** (NEU):
```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2
};

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Fehler beim Hashen: {}", e))?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Ungültiger Hash: {}", e))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub full_name: Option<String>,
    pub role: String,
}

#[tauri::command]
pub async fn login(
    username: String,
    password: String,
    pool: State<'_, DbPool>
) -> Result<User, String> {
    let pool = pool.read().await;

    // User aus DB holen
    let row = sqlx::query!(
        "SELECT id, username, password_hash, full_name, role, active
         FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|e| format!("Datenbankfehler: {}", e))?
    .ok_or("Benutzer nicht gefunden")?;

    // Aktiv?
    if !row.active {
        return Err("Benutzer deaktiviert".into());
    }

    // Passwort prüfen
    if !verify_password(&password, &row.password_hash)? {
        return Err("Falsches Passwort".into());
    }

    // Last login aktualisieren
    sqlx::query!(
        "UPDATE users SET last_login = CURRENT_TIMESTAMP WHERE id = $1",
        row.id
    )
    .execute(&*pool)
    .await
    .ok();

    Ok(User {
        id: row.id,
        username: row.username,
        full_name: row.full_name,
        role: row.role,
    })
}
```

#### 4.3 Login-Dialog (Frontend)

**src/components/Auth/LoginDialog.tsx** (NEU):
```typescript
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface User {
  id: number;
  username: string;
  fullName: string | null;
  role: string;
}

export function LoginDialog({ onLogin }: { onLogin: (user: User) => void }) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleLogin = async () => {
    setLoading(true);
    setError('');

    try {
      const user = await invoke<User>('login', { username, password });
      onLogin(user);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center">
      <div className="bg-white rounded-xl p-8 w-96 shadow-2xl">
        <h1 className="text-2xl font-bold mb-6">DPolG Buchungssystem</h1>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1">
              Benutzername
            </label>
            <input
              type="text"
              value={username}
              onChange={e => setUsername(e.target.value)}
              className="w-full px-4 py-2 border rounded-lg"
              onKeyPress={e => e.key === 'Enter' && handleLogin()}
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1">
              Passwort
            </label>
            <input
              type="password"
              value={password}
              onChange={e => setPassword(e.target.value)}
              className="w-full px-4 py-2 border rounded-lg"
              onKeyPress={e => e.key === 'Enter' && handleLogin()}
            />
          </div>

          {error && (
            <div className="text-red-500 text-sm bg-red-50 p-3 rounded">
              {error}
            </div>
          )}

          <button
            onClick={handleLogin}
            disabled={loading || !username || !password}
            className="w-full bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Anmelden...' : 'Anmelden'}
          </button>
        </div>
      </div>
    </div>
  );
}
```

#### 4.4 App.tsx mit Login

```typescript
function App() {
  const [user, setUser] = useState<User | null>(null);

  if (!user) {
    return <LoginDialog onLogin={setUser} />;
  }

  return (
    <div>
      <header>
        <span>Angemeldet als: {user.fullName || user.username}</span>
        <button onClick={() => setUser(null)}>Abmelden</button>
      </header>

      {/* Rest der App */}
    </div>
  );
}
```

#### 4.5 Permissions (Optional)

```typescript
// Nur Admins dürfen Zimmer/Gäste löschen
const canDelete = user?.role === 'admin';

<button disabled={!canDelete} onClick={deleteRoom}>
  Zimmer löschen
</button>
```

---

### **Phase 5: Testing (1 Tag)**

#### 5.1 Lokaler PostgreSQL Test

```bash
# Lokal PostgreSQL installieren (Windows)
# Download: https://www.postgresql.org/download/windows/

# Datenbank erstellen
psql -U postgres
CREATE DATABASE dpolg_booking_test;
\q

# config.toml für lokalen Test
[database]
host = "localhost"
port = 5432
database = "dpolg_booking_test"
username = "postgres"
password = "dein_passwort"
```

#### 5.2 Test-Szenarien

1. **Single-Client Test:**
   - App starten
   - Login
   - Buchung erstellen
   - Prüfen: Daten in PostgreSQL?

2. **Multi-Client Test:**
   - 2 Instanzen der App starten (2 PCs oder VMs)
   - Beide einloggen (verschiedene User!)
   - Client 1: Buchung erstellen
   - Client 2: Buchung erscheint sofort?
   - Client 2: Buchung bearbeiten
   - Client 1: Änderung sichtbar?

3. **Stress-Test:**
   - 5 Clients gleichzeitig
   - Alle erstellen Buchungen
   - Konflikte? (z.B. gleicher Raum, gleiches Datum)

#### 5.3 Performance-Monitoring

```rust
// Logging aktivieren
[dependencies]
env_logger = "0.11"

// In main.rs:
env_logger::init();

// In Commands:
log::info!("Booking erstellt: {}", booking_id);
log::warn!("Langsame Query: {}ms", duration);
```

---

### **Phase 6: Daten-Migration (4 Std)**

#### 6.1 SQLite Daten exportieren

**migrate.py** (Python Script):
```python
import sqlite3
import psycopg2
from datetime import datetime

# SQLite verbinden
sqlite_conn = sqlite3.connect('booking.db')
sqlite_cur = sqlite_conn.cursor()

# PostgreSQL verbinden
pg_conn = psycopg2.connect(
    host="132.145.xxx.xxx",
    port=5432,
    database="dpolg_booking",
    user="booking_user",
    password="PASSWORD"
)
pg_cur = pg_conn.cursor()

# Tabellen migrieren
tables = ['rooms', 'guests', 'bookings', 'reminders', 'email_config']

for table in tables:
    print(f"Migriere {table}...")

    # SQLite Daten holen
    sqlite_cur.execute(f"SELECT * FROM {table}")
    rows = sqlite_cur.fetchall()

    # PostgreSQL Schema holen
    sqlite_cur.execute(f"PRAGMA table_info({table})")
    columns = [col[1] for col in sqlite_cur.fetchall()]

    # Daten einfügen
    for row in rows:
        placeholders = ','.join(['%s'] * len(row))
        cols = ','.join(columns)

        pg_cur.execute(
            f"INSERT INTO {table} ({cols}) VALUES ({placeholders})",
            row
        )

    pg_conn.commit()
    print(f"  → {len(rows)} Zeilen migriert")

pg_conn.close()
sqlite_conn.close()
print("Migration abgeschlossen!")
```

#### 6.2 Sequence IDs korrigieren

```sql
-- Nach Migration: Auto-Increment IDs korrigieren
SELECT setval('rooms_id_seq', (SELECT MAX(id) FROM rooms));
SELECT setval('guests_id_seq', (SELECT MAX(id) FROM guests));
SELECT setval('bookings_id_seq', (SELECT MAX(id) FROM bookings));
```

---

### **Phase 7: Deployment (4 Std)**

#### 7.1 Production Datenbank Setup

```bash
# Oracle Cloud VM (Production)
ssh ubuntu@<oracle-vm-ip>

# Backup-System einrichten
sudo apt install -y automysqlbackup postgresql-client

# Backup-Script
cat > /home/ubuntu/backup-db.sh << 'EOF'
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
pg_dump -U booking_user -h localhost dpolg_booking > /backups/dpolg_backup_$DATE.sql
# Alte Backups löschen (älter als 30 Tage)
find /backups -name "dpolg_backup_*.sql" -mtime +30 -delete
EOF

chmod +x /home/ubuntu/backup-db.sh

# Cron Job (täglich 3 Uhr nachts)
crontab -e
# Hinzufügen:
0 3 * * * /home/ubuntu/backup-db.sh
```

#### 7.2 App-Installer vorbereiten

**config.toml Template:**
```toml
# WICHTIG: Nach Installation anpassen!
[database]
host = "132.145.XXX.XXX"  # ← Oracle Cloud IP hier eintragen!
port = 5432
database = "dpolg_booking"
username = "booking_user"
password = "PASSWORT_HIER"  # ← Sicheres Passwort!
```

**Installation Checklist:**
```
☐ 1. dpolg-booking-setup.exe downloaden
☐ 2. Installation ausführen
☐ 3. config.toml bearbeiten:
     - Oracle Cloud IP eintragen
     - Passwort eintragen
☐ 4. App starten
☐ 5. Login mit Admin-Account
☐ 6. Funktionstest
```

#### 7.3 Monitoring (Optional)

```bash
# PostgreSQL Monitoring
sudo apt install -y postgresql-contrib

# Aktive Verbindungen checken
psql -U booking_user -d dpolg_booking -c "
  SELECT count(*) as connections, usename
  FROM pg_stat_activity
  GROUP BY usename;
"

# Slow Queries loggen
# postgresql.conf:
log_min_duration_statement = 1000  # Queries > 1s loggen
```

---

## 🎯 Success Criteria

Die Migration ist erfolgreich wenn:

- ✅ Alle 5 Clients können sich verbinden
- ✅ Login-System funktioniert
- ✅ Buchungen werden sofort synchronisiert (Echtzeit)
- ✅ Keine Daten gehen verloren (Migration erfolgreich)
- ✅ Performance ist gut (< 200ms für normale Queries)
- ✅ Backups laufen automatisch

---

## 🚨 Risiken & Lösungen

| Risiko | Wahrscheinlichkeit | Lösung |
|--------|-------------------|--------|
| **Oracle Firewall blockiert** | Hoch | Port 5432 in Security List + ufw freigeben |
| **Performance-Probleme** | Mittel | Indexes auf häufig genutzte Spalten |
| **Connection Timeouts** | Mittel | Connection Pooling (sqlx macht das automatisch) |
| **Daten-Konflikte** | Niedrig | Transactions + Optimistic Locking |
| **Migration schlägt fehl** | Niedrig | Backup vorher + Dry-Run auf Test-DB |

---

## 📊 Kosten

**Oracle Cloud Free Tier:**
- **Kosten: 0€** (Always Free)
- 2x VM.Standard.E2.1.Micro
- 200 GB Block Storage
- 10 TB/Monat Outbound Traffic

**Ausreichend für:**
- ✅ 5-10 gleichzeitige User
- ✅ 100.000+ Buchungen
- ✅ Automatische Backups
- ✅ 24/7 Verfügbarkeit

---

## 🎓 Lessons Learned (Zeiterfassungs-Projekt)

**Was gut funktioniert hat:**
1. Settings-Dialog statt manueller config.toml
2. Connection Pooling (sqlx macht das automatisch)
3. Tägliche Backups in Cron Job
4. Login beim App-Start (kein Token-Refresh nötig)

**Was verbessert werden kann:**
1. Offline-Mode (bei Netzwerk-Ausfall)
2. Automatische Reconnect bei Connection-Loss
3. Conflict Resolution (bei gleichzeitigen Edits)

---

## 📚 Weiterführende Infos

**Dokumentation:**
- sqlx: https://github.com/launchbadge/sqlx
- PostgreSQL: https://www.postgresql.org/docs/
- Oracle Cloud: https://docs.oracle.com/en-us/iaas/

**Ähnliche Projekte:**
- Zeiterfassungs-Programm (als Referenz nutzen!)

---

## ✅ Nächste Schritte

**Wenn bereit für Migration:**

1. Zeiterfassungs-Code zeigen (als Vorlage)
2. Oracle Cloud VM Status checken (frei?)
3. Phase 1 starten: Oracle Setup
4. Parallel: Rust Dependencies vorbereiten

**Fragen vorher klären:**
- Welche User brauchen Admin-Rechte?
- Wie soll die config.toml verteilt werden?
- Backup-Strategie final bestätigen

---

**Version:** 1.0
**Erstellt:** 2025-11-02
**Status:** 📋 Planung (wartet auf Zeiterfassungs-Referenz)
