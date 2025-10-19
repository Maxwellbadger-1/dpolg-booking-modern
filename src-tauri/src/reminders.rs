// ============================================================================
// REMINDER SYSTEM - CRUD OPERATIONS
// ============================================================================

use rusqlite::{Connection, Result};
use crate::database::{get_db_path, Reminder, ReminderSettings};

// ============================================================================
// REMINDER CRUD OPERATIONS
// ============================================================================

/// Erstellt eine neue Erinnerung
pub fn create_reminder(
    booking_id: Option<i64>,
    reminder_type: String,
    title: String,
    description: Option<String>,
    due_date: String,
    priority: String,
) -> Result<Reminder> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "INSERT INTO reminders (booking_id, reminder_type, title, description, due_date, priority)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![booking_id, reminder_type, title, description, due_date, priority],
    )?;

    let id = conn.last_insert_rowid();
    get_reminder_by_id(id)
}

/// Gibt eine Erinnerung anhand der ID zurück
pub fn get_reminder_by_id(id: i64) -> Result<Reminder> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, booking_id, reminder_type, title, description, due_date, priority,
                is_completed, completed_at, is_snoozed, snoozed_until, created_at, updated_at
         FROM reminders WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Reminder {
                id: row.get(0)?,
                booking_id: row.get(1)?,
                reminder_type: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                due_date: row.get(5)?,
                priority: row.get(6)?,
                is_completed: row.get::<_, i32>(7)? != 0,
                completed_at: row.get(8)?,
                is_snoozed: row.get::<_, i32>(9)? != 0,
                snoozed_until: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        },
    )
}

/// Gibt alle Erinnerungen zurück (optional gefiltert nach Status)
pub fn get_all_reminders(include_completed: bool) -> Result<Vec<Reminder>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let query = if include_completed {
        "SELECT id, booking_id, reminder_type, title, description, due_date, priority,
                is_completed, completed_at, is_snoozed, snoozed_until, created_at, updated_at
         FROM reminders ORDER BY due_date ASC"
    } else {
        "SELECT id, booking_id, reminder_type, title, description, due_date, priority,
                is_completed, completed_at, is_snoozed, snoozed_until, created_at, updated_at
         FROM reminders WHERE is_completed = 0 ORDER BY due_date ASC"
    };

    let mut stmt = conn.prepare(query)?;
    let reminders = stmt.query_map([], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            booking_id: row.get(1)?,
            reminder_type: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            due_date: row.get(5)?,
            priority: row.get(6)?,
            is_completed: row.get::<_, i32>(7)? != 0,
            completed_at: row.get(8)?,
            is_snoozed: row.get::<_, i32>(9)? != 0,
            snoozed_until: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    reminders.collect()
}

/// Gibt Erinnerungen für eine spezifische Buchung zurück
pub fn get_reminders_for_booking(booking_id: i64) -> Result<Vec<Reminder>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    let mut stmt = conn.prepare(
        "SELECT id, booking_id, reminder_type, title, description, due_date, priority,
                is_completed, completed_at, is_snoozed, snoozed_until, created_at, updated_at
         FROM reminders WHERE booking_id = ?1 ORDER BY due_date ASC"
    )?;

    let reminders = stmt.query_map(rusqlite::params![booking_id], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            booking_id: row.get(1)?,
            reminder_type: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            due_date: row.get(5)?,
            priority: row.get(6)?,
            is_completed: row.get::<_, i32>(7)? != 0,
            completed_at: row.get(8)?,
            is_snoozed: row.get::<_, i32>(9)? != 0,
            snoozed_until: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    reminders.collect()
}

/// Gibt überfällige/dringende Erinnerungen zurück (für Badge-Count)
pub fn get_urgent_reminders() -> Result<Vec<Reminder>> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Hole offene Erinnerungen die heute oder überfällig sind (nicht snoozed)
    let mut stmt = conn.prepare(
        "SELECT id, booking_id, reminder_type, title, description, due_date, priority,
                is_completed, completed_at, is_snoozed, snoozed_until, created_at, updated_at
         FROM reminders
         WHERE is_completed = 0
           AND is_snoozed = 0
           AND due_date <= date('now')
         ORDER BY due_date ASC, priority DESC"
    )?;

    let reminders = stmt.query_map([], |row| {
        Ok(Reminder {
            id: row.get(0)?,
            booking_id: row.get(1)?,
            reminder_type: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            due_date: row.get(5)?,
            priority: row.get(6)?,
            is_completed: row.get::<_, i32>(7)? != 0,
            completed_at: row.get(8)?,
            is_snoozed: row.get::<_, i32>(9)? != 0,
            snoozed_until: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    reminders.collect()
}

/// Aktualisiert eine Erinnerung
pub fn update_reminder(
    id: i64,
    title: String,
    description: Option<String>,
    due_date: String,
    priority: String,
) -> Result<Reminder> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "UPDATE reminders
         SET title = ?1, description = ?2, due_date = ?3, priority = ?4, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?5",
        rusqlite::params![title, description, due_date, priority, id],
    )?;

    get_reminder_by_id(id)
}

/// Markiert eine Erinnerung als erledigt
pub fn mark_reminder_completed(id: i64, completed: bool) -> Result<Reminder> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    if completed {
        conn.execute(
            "UPDATE reminders
             SET is_completed = 1, completed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            rusqlite::params![id],
        )?;
    } else {
        conn.execute(
            "UPDATE reminders
             SET is_completed = 0, completed_at = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            rusqlite::params![id],
        )?;
    }

    get_reminder_by_id(id)
}

/// Snooze eine Erinnerung (verschiebe Fälligkeit)
pub fn snooze_reminder(id: i64, snooze_until: String) -> Result<Reminder> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "UPDATE reminders
         SET is_snoozed = 1, snoozed_until = ?1, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?2",
        rusqlite::params![snooze_until, id],
    )?;

    get_reminder_by_id(id)
}

/// Löscht eine Erinnerung
pub fn delete_reminder(id: i64) -> Result<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute("DELETE FROM reminders WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}

// ============================================================================
// REMINDER SETTINGS (AUTO-REMINDER ON/OFF)
// ============================================================================

/// Gibt die Reminder-Settings zurück (id ist immer 1)
pub fn get_reminder_settings() -> Result<ReminderSettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.query_row(
        "SELECT id, auto_reminder_incomplete_data, auto_reminder_payment,
                auto_reminder_checkin, auto_reminder_invoice, updated_at
         FROM reminder_settings WHERE id = 1",
        [],
        |row| {
            Ok(ReminderSettings {
                id: row.get(0)?,
                auto_reminder_incomplete_data: row.get::<_, i32>(1)? != 0,
                auto_reminder_payment: row.get::<_, i32>(2)? != 0,
                auto_reminder_checkin: row.get::<_, i32>(3)? != 0,
                auto_reminder_invoice: row.get::<_, i32>(4)? != 0,
                updated_at: row.get(5)?,
            })
        },
    )
}

/// Speichert/Aktualisiert die Reminder-Settings (id ist immer 1)
pub fn save_reminder_settings(settings: ReminderSettings) -> Result<ReminderSettings> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "UPDATE reminder_settings
         SET auto_reminder_incomplete_data = ?1,
             auto_reminder_payment = ?2,
             auto_reminder_checkin = ?3,
             auto_reminder_invoice = ?4,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = 1",
        rusqlite::params![
            settings.auto_reminder_incomplete_data as i32,
            settings.auto_reminder_payment as i32,
            settings.auto_reminder_checkin as i32,
            settings.auto_reminder_invoice as i32,
        ],
    )?;

    get_reminder_settings()
}
