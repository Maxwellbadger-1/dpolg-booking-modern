// Turso SQLite Cloud Sync for Mobile App
// Synchronizes cleaning_tasks from PostgreSQL to Turso

use libsql::Builder;
use crate::database_pg::repositories::CleaningTaskRepository;
use crate::database_pg::pool::DbPool;

// Turso credentials (from dpolg-cleaning-mobile/api/tasks.js)
const TURSO_URL: &str = "https://dpolg-cleaning-maxwellbadger-1.aws-eu-west-1.turso.io";
const TURSO_TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NTk4NDI1MzcsImlkIjoiZjY1ZWY2YzMtYWNhMS00NjZiLWExYjgtODU0MTlmYjlmNDNiIiwicmlkIjoiMTRjNDc4YjAtYTAwMy00ZmZmLThiYTUtYTZhOWIwYjZiODdmIn0.JSyu72rlp3pQ_vFxozglKoV-XMHW12j_hVfhTKjbEGwSyWnWBq2kziJNx2WwvvwD09NU-TMoLLszq2Mm9OlLDw";

/// Create Turso database connection
async fn get_turso_connection() -> Result<libsql::Connection, String> {
    let db = Builder::new_remote(TURSO_URL.to_string(), TURSO_TOKEN.to_string())
        .build()
        .await
        .map_err(|e| format!("Failed to build Turso client: {}", e))?;

    let conn = db.connect()
        .map_err(|e| format!("Failed to connect to Turso: {}", e))?;

    Ok(conn)
}

/// Initialize Turso schema (create table if not exists)
pub async fn init_turso_schema() -> Result<(), String> {
    println!("📊 Initializing Turso schema...");

    let conn = get_turso_connection().await?;

    // Drop existing table to ensure schema is up-to-date
    conn.execute("DROP TABLE IF EXISTS cleaning_tasks", ())
        .await
        .map_err(|e| format!("Failed to drop table: {}", e))?;

    // Create cleaning_tasks table matching mobile app expectations
    conn.execute(
        "CREATE TABLE cleaning_tasks (
            id INTEGER PRIMARY KEY,
            booking_id INTEGER NOT NULL,
            reservierungsnummer TEXT,
            date TEXT NOT NULL,
            room_name TEXT,
            room_id INTEGER NOT NULL,
            room_location TEXT,
            guest_name TEXT,
            checkout_time TEXT,
            checkin_time TEXT,
            priority TEXT NOT NULL,
            notes TEXT,
            status TEXT DEFAULT 'pending',
            guest_count INTEGER,
            extras TEXT,
            emojis_start TEXT,
            emojis_end TEXT,
            has_dog INTEGER DEFAULT 0,
            change_bedding INTEGER DEFAULT 1,
            completed_at TEXT,
            completed_by TEXT,
            synced_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        ()
    )
    .await
    .map_err(|e| format!("Failed to create table: {}", e))?;

    println!("✅ Turso schema initialized");
    Ok(())
}

/// Sync cleaning tasks from PostgreSQL to Turso
pub async fn sync_tasks_to_turso(
    pg_pool: &DbPool,
    start_date: String,
    end_date: String,
) -> Result<usize, String> {
    println!("🔄 Syncing tasks to Turso: {} to {}", start_date, end_date);

    // 1. Load tasks from PostgreSQL
    let tasks = CleaningTaskRepository::get_for_period(pg_pool, start_date.clone(), end_date.clone())
        .await
        .map_err(|e| format!("Failed to load tasks from PostgreSQL: {}", e))?;

    println!("📋 Found {} tasks in PostgreSQL", tasks.len());

    if tasks.is_empty() {
        println!("⚠️ No tasks to sync");
        return Ok(0);
    }

    // 2. Connect to Turso
    let conn = get_turso_connection().await?;

    // 3. Delete old tasks in date range (to avoid duplicates)
    conn.execute(
        "DELETE FROM cleaning_tasks WHERE date >= ? AND date <= ?",
        libsql::params![start_date, end_date]
    )
    .await
    .map_err(|e| format!("Failed to delete old tasks: {}", e))?;

    println!("🗑️ Cleared old tasks from Turso");

    // 4. Insert new tasks
    let mut synced_count = 0;
    let total_tasks = tasks.len();

    for task in tasks {
        let result = conn.execute(
            "INSERT OR REPLACE INTO cleaning_tasks (
                id, booking_id, reservierungsnummer, date,
                room_name, room_id, room_location, guest_name,
                checkout_time, checkin_time, priority, notes,
                status, guest_count, extras,
                emojis_start, emojis_end,
                has_dog, change_bedding, completed_at, completed_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            libsql::params![
                task.id,
                task.booking_id,
                task.reservierungsnummer.unwrap_or_default(),
                task.task_date,
                task.room_number.unwrap_or_else(|| format!("{}", task.room_id)),
                task.room_id,
                task.room_location.unwrap_or_default(),
                task.guest_name.unwrap_or_default(),
                task.checkout_time.unwrap_or_default(),
                task.checkin_time.unwrap_or_default(),
                task.priority,
                task.notes.unwrap_or_default(),
                task.status,
                task.guest_count.unwrap_or(0),
                task.extras.unwrap_or_default(),
                task.emojis_start.unwrap_or_default(),
                task.emojis_end.unwrap_or_default(),
                if task.has_dog { 1 } else { 0 },
                if task.change_bedding { 1 } else { 0 },
                task.completed_at.unwrap_or_default(),
                task.completed_by.unwrap_or_default(),
            ]
        )
        .await;

        match result {
            Ok(_) => synced_count += 1,
            Err(e) => println!("⚠️ Failed to sync task {}: {}", task.id, e),
        }
    }

    println!("✅ Synced {}/{} tasks to Turso", synced_count, total_tasks);
    Ok(synced_count)
}

/// Delete tasks from Turso by booking_id
pub async fn delete_tasks_from_turso(booking_id: i32) -> Result<(), String> {
    println!("🗑️ Deleting tasks for booking {} from Turso", booking_id);

    let conn = get_turso_connection().await?;

    conn.execute(
        "DELETE FROM cleaning_tasks WHERE booking_id = ?",
        libsql::params![booking_id]
    )
    .await
    .map_err(|e| format!("Failed to delete tasks: {}", e))?;

    println!("✅ Tasks deleted from Turso");
    Ok(())
}

/// Cleanup old completed tasks from Turso (older than 30 days)
pub async fn cleanup_old_tasks_turso() -> Result<usize, String> {
    println!("🧹 Cleaning up old tasks from Turso...");

    let conn = get_turso_connection().await?;

    // Delete tasks older than 30 days
    let thirty_days_ago = chrono::Local::now()
        .checked_sub_signed(chrono::Duration::days(30))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();

    let result = conn.execute(
        "DELETE FROM cleaning_tasks WHERE date < ? AND status = 'completed'",
        libsql::params![thirty_days_ago]
    )
    .await
    .map_err(|e| format!("Failed to cleanup tasks: {}", e))?;

    println!("✅ Cleaned up old tasks from Turso");
    Ok(result as usize)
}
