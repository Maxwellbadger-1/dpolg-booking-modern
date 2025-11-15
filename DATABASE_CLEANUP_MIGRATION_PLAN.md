# 📊 DATABASE CLEANUP & ORACLE MIGRATION PLAN
## DPolG Booking System - SQLite zu PostgreSQL auf Oracle Cloud

---

## 🔍 TEIL 1: AKTUELLE DATENBANK-PROBLEME & LÖSUNGEN

### 1. INKONSISTENTE NAMING CONVENTIONS ⚠️

**Problem:** Gemischte Sprachen und Stile
```sql
-- AKTUELL (INKONSISTENT):
- Tabellen: Englisch (rooms, guests, bookings)
- Felder: Deutsch (anzahl_gaeste, bemerkungen, gebaeude_typ)
- Mischung: checkin_date (Englisch) vs checkout_date (Englisch) vs anzahl_naechte (Deutsch)
```

**LÖSUNG VOR MIGRATION:**
```sql
-- Schritt 1: View Layer mit konsistenten Namen erstellen
CREATE VIEW v_bookings AS
SELECT
    id,
    room_id,
    guest_id,
    reservierungsnummer as reservation_number,
    checkin_date,
    checkout_date,
    anzahl_gaeste as guest_count,
    anzahl_begleitpersonen as companion_count,
    status,
    grundpreis as base_price,
    services_preis as services_price,
    rabatt_preis as discount_amount,
    gesamtpreis as total_price,
    anzahl_naechte as nights_count,
    bemerkungen as notes,
    bezahlt as is_paid,
    bezahlt_am as paid_at,
    zahlungsmethode as payment_method,
    ist_stiftungsfall as is_foundation_case,
    ist_dpolg_mitglied as is_dpolg_member
FROM bookings;

-- Views für alle Tabellen erstellen (transitional layer)
```

### 2. FEHLENDE INDIZES FÜR PERFORMANCE 🚀

**Problem:** Keine Indizes auf häufig abgefragte Felder

**LÖSUNG SOFORT IMPLEMENTIEREN:**
```sql
-- Performance-kritische Indizes
CREATE INDEX IF NOT EXISTS idx_bookings_dates ON bookings(checkin_date, checkout_date);
CREATE INDEX IF NOT EXISTS idx_bookings_room_guest ON bookings(room_id, guest_id);
CREATE INDEX IF NOT EXISTS idx_bookings_status ON bookings(status) WHERE status != 'storniert';
CREATE INDEX IF NOT EXISTS idx_bookings_payment ON bookings(bezahlt) WHERE bezahlt = 0;

-- Guest Lookups
CREATE INDEX IF NOT EXISTS idx_guests_name ON guests(nachname, vorname);
CREATE INDEX IF NOT EXISTS idx_guests_email ON guests(email);
CREATE INDEX IF NOT EXISTS idx_guests_member ON guests(dpolg_mitglied) WHERE dpolg_mitglied = 1;

-- Room Availability
CREATE INDEX IF NOT EXISTS idx_rooms_type ON rooms(gebaeude_typ);

-- Reminders Performance
CREATE INDEX IF NOT EXISTS idx_reminders_due ON reminders(due_date, is_completed) WHERE is_completed = 0;
CREATE INDEX IF NOT EXISTS idx_reminders_booking ON reminders(booking_id);

-- Email Logs
CREATE INDEX IF NOT EXISTS idx_email_logs_booking ON email_logs(booking_id);
CREATE INDEX IF NOT EXISTS idx_email_logs_sent ON email_logs(sent_at);
```

### 3. FEHLENDE CONSTRAINTS & VALIDIERUNG ❌

**Problem:** Keine CHECK Constraints für Datenintegrität

**LÖSUNG:**
```sql
-- Neue Tabelle mit Constraints (für PostgreSQL vorbereitet)
ALTER TABLE bookings ADD CONSTRAINT chk_dates
    CHECK (checkout_date > checkin_date);

ALTER TABLE bookings ADD CONSTRAINT chk_price_positive
    CHECK (gesamtpreis >= 0);

ALTER TABLE bookings ADD CONSTRAINT chk_guest_count
    CHECK (anzahl_gaeste > 0);

ALTER TABLE rooms ADD CONSTRAINT chk_capacity
    CHECK (capacity > 0);

ALTER TABLE guests ADD CONSTRAINT chk_email_format
    CHECK (email LIKE '%@%.%');
```

### 4. DUPLIZIERTE DATEN & NORMALISIERUNG 📝

**Problem:** Redundante Felder zwischen Tabellen

**NORMALISIERUNG VOR MIGRATION:**

```sql
-- Problem: Preise in bookings UND rooms gespeichert
-- Lösung: Single Source of Truth

-- 1. Seasons Table (NEU)
CREATE TABLE IF NOT EXISTS seasons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE, -- 'nebensaison', 'hauptsaison', 'hochsaison'
    start_month INTEGER,
    start_day INTEGER,
    end_month INTEGER,
    end_day INTEGER,
    multiplier REAL DEFAULT 1.0,
    created_at TEXT DEFAULT (datetime('now'))
);

-- 2. Room Prices Table (NEU)
CREATE TABLE IF NOT EXISTS room_prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    price_per_night REAL NOT NULL,
    valid_from TEXT DEFAULT (date('now')),
    valid_until TEXT,
    FOREIGN KEY (room_id) REFERENCES rooms(id),
    FOREIGN KEY (season_id) REFERENCES seasons(id),
    UNIQUE(room_id, season_id, valid_from)
);

-- 3. Audit Trail Table (NEU)
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name TEXT NOT NULL,
    record_id INTEGER NOT NULL,
    action TEXT NOT NULL, -- 'INSERT', 'UPDATE', 'DELETE'
    old_values TEXT, -- JSON
    new_values TEXT, -- JSON
    user_id INTEGER,
    timestamp TEXT DEFAULT (datetime('now')),
    ip_address TEXT
);
```

### 5. TRANSACTION LOGGING FEHLT 📊

**Problem:** Keine Historie von Änderungen

**LÖSUNG:**
```sql
-- Transaction Log für alle kritischen Operationen
CREATE TRIGGER IF NOT EXISTS booking_audit_insert
AFTER INSERT ON bookings
BEGIN
    INSERT INTO audit_log (table_name, record_id, action, new_values)
    VALUES ('bookings', NEW.id, 'INSERT',
        json_object(
            'guest_id', NEW.guest_id,
            'room_id', NEW.room_id,
            'checkin_date', NEW.checkin_date,
            'checkout_date', NEW.checkout_date,
            'total_price', NEW.gesamtpreis
        )
    );
END;

CREATE TRIGGER IF NOT EXISTS booking_audit_update
AFTER UPDATE ON bookings
BEGIN
    INSERT INTO audit_log (table_name, record_id, action, old_values, new_values)
    VALUES ('bookings', NEW.id, 'UPDATE',
        json_object('status', OLD.status, 'price', OLD.gesamtpreis),
        json_object('status', NEW.status, 'price', NEW.gesamtpreis)
    );
END;
```

---

## 🚀 TEIL 2: ORACLE CLOUD SETUP (BEST PRACTICES 2025)

### ENTSCHEIDUNG: PostgreSQL auf Oracle VM! ✅

**Warum PostgreSQL statt MySQL:**
- ✅ Bessere JSON Support (für flexible Daten)
- ✅ Better Window Functions (für Reports)
- ✅ Materialized Views (für Performance)
- ✅ pgloader Tool für einfache Migration
- ✅ Mehr Ähnlichkeit zu SQLite Syntax

### ORACLE VM SETUP (4 STUNDEN)

```bash
# 1. VM Creation (Oracle Cloud Console)
- Shape: VM.Standard.E2.1.Micro (Free Tier)
- Image: Ubuntu 22.04 LTS
- Storage: 50 GB Boot Volume
- Network: Public Subnet with Internet Gateway
- SSH Key: Generieren und speichern!

# 2. PostgreSQL 16 Installation (Latest Stable)
ssh ubuntu@<your-oracle-vm-ip>

# PostgreSQL 16 Repository hinzufügen
sudo sh -c 'echo "deb https://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -
sudo apt update

# Install PostgreSQL 16 + pgBouncer
sudo apt install -y postgresql-16 postgresql-client-16 postgresql-contrib-16
sudo apt install -y pgbouncer

# Install Migration Tools
sudo apt install -y pgloader  # SQLite to PostgreSQL converter
```

### POSTGRESQL CONFIGURATION (BEST PRACTICES 2025)

```bash
# /etc/postgresql/16/main/postgresql.conf

# Connection Settings (für 1GB RAM Oracle VM)
max_connections = 100           # Reduced from default 200
superuser_reserved_connections = 3

# Memory (Optimized for 1GB RAM)
shared_buffers = 256MB          # 25% of RAM
effective_cache_size = 768MB    # 75% of RAM
maintenance_work_mem = 64MB
work_mem = 2MB                  # Per operation
wal_buffers = 8MB

# Checkpoint Settings
checkpoint_completion_target = 0.9
checkpoint_timeout = 10min
max_wal_size = 1GB
min_wal_size = 80MB

# Logging (für Debugging)
log_statement = 'all'           # Während Migration
log_duration = on
log_min_duration_statement = 100  # Log slow queries > 100ms

# Performance
random_page_cost = 1.1          # SSD optimized
effective_io_concurrency = 200  # SSD optimized
default_statistics_target = 100

# Enable Extensions
shared_preload_libraries = 'pg_stat_statements'
```

### PGBOUNCER SETUP (CONNECTION POOLING)

```ini
# /etc/pgbouncer/pgbouncer.ini

[databases]
dpolg_booking = host=127.0.0.1 port=5432 dbname=dpolg_booking

[pgbouncer]
listen_addr = *
listen_port = 6432
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt

# Pool Configuration (für 5-10 User)
pool_mode = transaction         # Best for web apps
default_pool_size = 20          # Connections per pool
min_pool_size = 5
reserve_pool_size = 5
max_client_conn = 100           # Total client connections
max_db_connections = 50         # Total to PostgreSQL

# Performance
server_reset_query = DISCARD ALL
server_check_query = select 1
server_check_delay = 30

# Timeouts
server_lifetime = 3600
server_idle_timeout = 600
server_connect_timeout = 15
client_idle_timeout = 0
client_login_timeout = 60

# Logging
log_connections = 1
log_disconnections = 1
stats_period = 60
```

---

## 🔄 TEIL 3: MIGRATION MIT PGLOADER (BEST PRACTICE)

### VORBEREITUNG

```bash
# 1. SQLite Datenbank vorbereiten
# Auf lokalem Rechner:
sqlite3 booking_system.db

# Enable Foreign Keys
PRAGMA foreign_keys = ON;

# Vacuum und Analyze
VACUUM;
ANALYZE;

# Export für Transfer
.backup booking_backup.db
```

### PGLOADER CONFIGURATION

```lisp
-- migration.load
LOAD DATABASE
     FROM sqlite:///home/ubuntu/booking_backup.db
     INTO postgresql://dpolg_user:password@localhost:5432/dpolg_booking

WITH
    -- Performance Settings
    workers = 4,
    concurrency = 1,
    multiple_readers = yes,
    rows per range = 10000,

    -- Migration Options
    create tables,
    include drop,
    create indexes,
    reset sequences,
    foreign keys,
    downcase identifiers,

    -- Data Handling
    data only = false,
    truncate = false,
    disable triggers = false,

    -- Safety
    on error stop,
    set work_mem to '16MB',
    set maintenance_work_mem to '512 MB'

CAST
    -- Type Conversions
    type datetime to timestamp drop default drop not null,
    type date drop default drop not null using zero-dates-to-null,
    type integer to bigint drop typemod,
    type real to double precision,
    type text to text drop typemod

ALTER SCHEMA 'main' RENAME TO 'public'

BEFORE LOAD DO
    $$
    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
    CREATE EXTENSION IF NOT EXISTS "pgcrypto";
    $$

AFTER LOAD DO
    $$
    -- Fix Sequences
    SELECT setval('bookings_id_seq', (SELECT MAX(id) FROM bookings));
    SELECT setval('guests_id_seq', (SELECT MAX(id) FROM guests));
    SELECT setval('rooms_id_seq', (SELECT MAX(id) FROM rooms));

    -- Add Missing Indexes
    CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_bookings_dates
        ON bookings(checkin_date, checkout_date);

    -- Update Statistics
    ANALYZE;
    $$;
```

### MIGRATION AUSFÜHREN

```bash
# Upload SQLite DB to Oracle VM
scp booking_backup.db ubuntu@<oracle-vm-ip>:/home/ubuntu/

# Auf Oracle VM
# PostgreSQL Database erstellen
sudo -u postgres createdb dpolg_booking
sudo -u postgres createuser dpolg_user
sudo -u postgres psql -c "ALTER USER dpolg_user WITH ENCRYPTED PASSWORD 'secure_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE dpolg_booking TO dpolg_user;"

# Migration mit pgloader
pgloader migration.load

# Verify Migration
psql -U dpolg_user -d dpolg_booking -c "\dt"  # List tables
psql -U dpolg_user -d dpolg_booking -c "SELECT COUNT(*) FROM bookings;"
```

---

## 🔧 TEIL 4: BACKEND ANPASSUNGEN (RUST)

### DEPENDENCIES UPDATE

```toml
# Cargo.toml
[dependencies]
# REMOVE
# rusqlite = { version = "0.32", features = ["bundled"] }

# ADD
sqlx = { version = "0.8", features = [
    "runtime-tokio-native-tls",
    "postgres",
    "chrono",
    "uuid",
    "migrate",
    "macros"
]}
tokio = { version = "1", features = ["full"] }
deadpool-postgres = "0.14"  # Connection Pooling
```

### DATABASE CONNECTION MODULE

```rust
// src-tauri/src/db.rs (NEU)
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    pub async fn new() -> Result<Self, sqlx::Error> {
        // Connection über PgBouncer!
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                format!(
                    "postgres://dpolg_user:{}@{}:6432/dpolg_booking",
                    std::env::var("DB_PASSWORD").unwrap_or("password".to_string()),
                    std::env::var("DB_HOST").unwrap_or("localhost".to_string())
                )
            });

        let pool = PgPoolOptions::new()
            .max_connections(20)  // PgBouncer handles actual pooling
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(600))
            .test_before_acquire(true)
            .connect(&database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}

// Global State
pub type DbPool = Arc<RwLock<DatabasePool>>;
```

### TAURI COMMAND CONVERSION

```rust
// VORHER (SQLite)
#[tauri::command]
fn get_all_bookings_command() -> Result<Vec<Booking>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT * FROM bookings")
        .map_err(|e| e.to_string())?;

    // ... query execution
}

// NACHHER (PostgreSQL)
#[tauri::command]
async fn get_all_bookings_command(
    db: tauri::State<'_, DbPool>
) -> Result<Vec<Booking>, String> {
    let pool = db.read().await;

    let bookings = sqlx::query_as!(
        Booking,
        r#"
        SELECT
            id, room_id, guest_id, reservierungsnummer,
            checkin_date, checkout_date, anzahl_gaeste,
            status, gesamtpreis, bemerkungen, created_at
        FROM bookings
        ORDER BY checkin_date DESC
        "#
    )
    .fetch_all(pool.get_pool())
    .await
    .map_err(|e| e.to_string())?;

    Ok(bookings)
}
```

---

## 🎨 TEIL 5: FRONTEND OPTIMIERUNGEN

### CONNECTION STATUS INDICATOR

```typescript
// src/hooks/useConnectionStatus.ts
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function useConnectionStatus() {
    const [status, setStatus] = useState<'connected' | 'disconnected' | 'connecting'>('connecting');
    const [latency, setLatency] = useState<number | null>(null);

    useEffect(() => {
        const checkConnection = async () => {
            const start = performance.now();
            try {
                await invoke('ping_database');
                const end = performance.now();
                setLatency(Math.round(end - start));
                setStatus('connected');
            } catch {
                setStatus('disconnected');
            }
        };

        checkConnection();
        const interval = setInterval(checkConnection, 5000);
        return () => clearInterval(interval);
    }, []);

    return { status, latency };
}
```

### OPTIMISTIC UPDATES MIT ROLLBACK

```typescript
// src/context/DataContext.tsx
const updateBooking = useCallback(async (id: number, updates: Partial<Booking>) => {
    // Optimistic Update
    const oldBooking = bookings.find(b => b.id === id);
    setBookings(prev => prev.map(b =>
        b.id === id ? { ...b, ...updates } : b
    ));

    try {
        await invoke('update_booking_command', { id, ...updates });
        // Success - keep optimistic update
    } catch (error) {
        // Rollback on error
        if (oldBooking) {
            setBookings(prev => prev.map(b =>
                b.id === id ? oldBooking : b
            ));
        }
        throw error;
    }
}, [bookings]);
```

---

## 📊 TEIL 6: MONITORING & BACKUP

### POSTGRESQL MONITORING QUERIES

```sql
-- Active Connections
SELECT count(*) FROM pg_stat_activity;

-- Slow Queries
SELECT
    query,
    calls,
    mean_exec_time,
    total_exec_time
FROM pg_stat_statements
WHERE mean_exec_time > 100
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Table Sizes
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Index Usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read
FROM pg_stat_user_indexes
ORDER BY idx_scan;
```

### AUTOMATED BACKUP SCRIPT

```bash
#!/bin/bash
# /home/ubuntu/backup-dpolg.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/home/ubuntu/backups"
DB_NAME="dpolg_booking"
DB_USER="dpolg_user"

# Create backup
pg_dump -U $DB_USER -d $DB_NAME -F c -b -v -f "$BACKUP_DIR/dpolg_$DATE.backup"

# Keep only last 7 days
find $BACKUP_DIR -name "dpolg_*.backup" -mtime +7 -delete

# Optional: Sync to Oracle Object Storage
# oci os object put --bucket-name dpolg-backups --file "$BACKUP_DIR/dpolg_$DATE.backup"

# Cron Setup
# 0 3 * * * /home/ubuntu/backup-dpolg.sh
```

---

## 📈 TEIL 7: PERFORMANCE OPTIMIERUNGEN

### MATERIALIZED VIEWS FÜR REPORTS

```sql
-- Booking Statistics View
CREATE MATERIALIZED VIEW mv_booking_stats AS
SELECT
    DATE_TRUNC('month', checkin_date::date) as month,
    COUNT(*) as booking_count,
    SUM(gesamtpreis) as total_revenue,
    AVG(anzahl_naechte) as avg_nights,
    COUNT(DISTINCT guest_id) as unique_guests
FROM bookings
WHERE status != 'storniert'
GROUP BY DATE_TRUNC('month', checkin_date::date);

-- Refresh Schedule
CREATE OR REPLACE FUNCTION refresh_materialized_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_booking_stats;
END;
$$ LANGUAGE plpgsql;

-- Schedule with pg_cron
SELECT cron.schedule('refresh-views', '0 1 * * *', 'SELECT refresh_materialized_views()');
```

### PARTITIONING FÜR GROSSE TABELLEN

```sql
-- Partition bookings by year
CREATE TABLE bookings_2024 PARTITION OF bookings
    FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');

CREATE TABLE bookings_2025 PARTITION OF bookings
    FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');

-- Auto-create future partitions
CREATE OR REPLACE FUNCTION create_monthly_partitions()
RETURNS void AS $$
DECLARE
    start_date date;
    end_date date;
    partition_name text;
BEGIN
    start_date := date_trunc('month', CURRENT_DATE);
    end_date := start_date + interval '1 month';
    partition_name := 'bookings_' || to_char(start_date, 'YYYY_MM');

    EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF bookings
                    FOR VALUES FROM (%L) TO (%L)',
                    partition_name, start_date, end_date);
END;
$$ LANGUAGE plpgsql;
```

---

## ✅ MIGRATIONS-CHECKLISTE

### PRE-MIGRATION (1 Tag)
- [ ] SQLite Backup erstellen
- [ ] Indizes hinzufügen (siehe oben)
- [ ] Constraints validieren
- [ ] Test-Migration auf lokalem PostgreSQL

### ORACLE SETUP (4 Stunden)
- [ ] VM provisionieren
- [ ] PostgreSQL 16 installieren
- [ ] PgBouncer konfigurieren
- [ ] Firewall Ports öffnen (5432, 6432)
- [ ] SSL Zertifikate einrichten

### MIGRATION (2-3 Stunden)
- [ ] pgloader configuration testen
- [ ] Daten migrieren
- [ ] Sequences korrigieren
- [ ] Indizes erstellen
- [ ] VACUUM ANALYZE ausführen

### BACKEND UPDATE (2 Tage)
- [ ] Dependencies updaten
- [ ] Connection Pool implementieren
- [ ] Commands zu async konvertieren
- [ ] Error Handling anpassen

### TESTING (1 Tag)
- [ ] Single User Test
- [ ] Multi User Test
- [ ] Performance Benchmark
- [ ] Backup/Restore Test

### GO-LIVE
- [ ] DNS/IP Configuration
- [ ] SSL aktivieren
- [ ] Monitoring einrichten
- [ ] Backup Cron aktivieren

---

## 🎯 ERWARTETE VERBESSERUNGEN

| Metrik | SQLite (Jetzt) | PostgreSQL (Nach Migration) | Verbesserung |
|--------|---------------|------------------------------|--------------|
| **Concurrent Users** | 1 | 10-20 | 10-20x |
| **Query Performance** | ~50-200ms | ~10-50ms | 4-5x |
| **Write Throughput** | 10 tx/sec | 100+ tx/sec | 10x |
| **Backup Time** | 5 min | < 1 min | 5x |
| **Failover** | Nicht möglich | < 1 min | ✅ |
| **Real-time Sync** | Nein | Ja (Logical Replication) | ✅ |

---

## 🚀 ZEITPLAN

**Woche 1:**
- Tag 1: Pre-Migration Cleanup
- Tag 2: Oracle Setup + PostgreSQL
- Tag 3-4: Backend Migration
- Tag 5: Testing

**Woche 2:**
- Tag 1-2: Bug Fixes
- Tag 3: Performance Tuning
- Tag 4: Documentation
- Tag 5: Go-Live

---

**Version:** 1.0
**Erstellt:** 2025-11-14
**Status:** Ready for Implementation