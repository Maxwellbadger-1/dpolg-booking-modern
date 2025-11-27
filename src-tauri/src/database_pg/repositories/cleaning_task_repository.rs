use crate::database_pg::{DbPool, DbResult, DbError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CleaningTask {
    pub id: i32,
    pub booking_id: i32,
    pub room_id: i32,
    pub room_number: Option<String>,
    pub room_location: Option<String>,  // NEW: Ort des Zimmers (Fall, Langenprozelten)
    pub reservierungsnummer: Option<String>,  // NEW: Booking-Nummer
    pub task_date: String,
    pub checkout_time: Option<String>,
    pub checkin_time: Option<String>,
    pub priority: String,
    pub has_dog: bool,
    pub change_bedding: bool,
    pub guest_count: Option<i32>,
    pub guest_name: Option<String>,
    pub emojis_start: Option<String>,  // NEW: Services vor Anreise
    pub emojis_end: Option<String>,  // NEW: Services nach Abreise
    pub status: String,
    pub completed_at: Option<String>,
    pub completed_by: Option<String>,
    pub notes: Option<String>,
    pub extras: Option<String>,  // NEW: JSON mit zusätzlichen Daten
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleaningStats {
    pub today: i32,
    pub tomorrow: i32,
    pub this_week: i32,
    pub total: i32,
}

pub struct CleaningTaskRepository;

impl CleaningTaskRepository {
    /// Get all cleaning tasks (without date filter)
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<CleaningTask>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT
                    ct.id, ct.booking_id, ct.room_id,
                    r.name as room_number,
                    r.ort as room_location,
                    b.reservierungsnummer,
                    ct.task_date::text,
                    b.checkin_date::text,
                    b.checkout_date::text,
                    ct.checkout_time::text,
                    ct.checkin_time::text,
                    ct.priority,
                    ct.has_dog,
                    ct.change_bedding,
                    ct.guest_count,
                    ct.guest_name,
                    ct.status,
                    ct.completed_at::text,
                    ct.completed_by,
                    ct.notes,
                    STRING_AGG(CASE WHEN st.cleaning_plan_position = 'start' AND st.show_in_cleaning_plan = true THEN st.emoji ELSE NULL END, ',') FILTER (WHERE st.emoji IS NOT NULL) as emojis_start,
                    STRING_AGG(CASE WHEN st.cleaning_plan_position = 'end' AND st.show_in_cleaning_plan = true THEN st.emoji ELSE NULL END, ',') FILTER (WHERE st.emoji IS NOT NULL) as emojis_end
                 FROM cleaning_tasks ct
                 JOIN rooms r ON r.id = ct.room_id
                 JOIN bookings b ON b.id = ct.booking_id
                 LEFT JOIN additional_services asrv ON asrv.booking_id = b.id
                 LEFT JOIN service_templates st ON st.id = asrv.template_id
                 GROUP BY ct.id, ct.booking_id, ct.room_id, ct.task_date, ct.checkout_time, ct.checkin_time,
                          ct.priority, ct.has_dog, ct.change_bedding, ct.guest_count, ct.guest_name,
                          ct.status, ct.completed_at, ct.completed_by, ct.notes,
                          r.id, r.name, r.ort, b.reservierungsnummer, b.checkin_date, b.checkout_date
                 ORDER BY ct.task_date DESC, ct.priority DESC, r.id",
                &[],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let checkin_date: Option<String> = row.try_get("checkin_date").ok();
                let checkout_date: Option<String> = row.try_get("checkout_date").ok();
                let guest_count: Option<i32> = row.try_get("guest_count").ok().flatten();

                // Build extras JSON
                let extras = serde_json::json!({
                    "original_checkin": checkin_date,
                    "original_checkout": checkout_date,
                    "guest_count": guest_count,
                }).to_string();

                CleaningTask {
                    id: row.get("id"),
                    booking_id: row.get("booking_id"),
                    room_id: row.get("room_id"),
                    room_number: row.try_get("room_number").ok(),
                    room_location: row.try_get("room_location").ok(),
                    reservierungsnummer: row.try_get("reservierungsnummer").ok(),
                    task_date: row.get("task_date"),
                    checkout_time: row.try_get("checkout_time").ok().flatten(),
                    checkin_time: row.try_get("checkin_time").ok().flatten(),
                    priority: row.get("priority"),
                    has_dog: row.get("has_dog"),
                    change_bedding: row.get("change_bedding"),
                    guest_count,
                    guest_name: row.try_get("guest_name").ok().flatten(),
                    emojis_start: row.try_get("emojis_start").ok().flatten(),
                    emojis_end: row.try_get("emojis_end").ok().flatten(),
                    status: row.get("status"),
                    completed_at: row.try_get("completed_at").ok().flatten(),
                    completed_by: row.try_get("completed_by").ok().flatten(),
                    notes: row.try_get("notes").ok().flatten(),
                    extras: Some(extras),
                }
            })
            .collect())
    }

    /// Get all cleaning tasks for a date range
    pub async fn get_for_period(
        pool: &DbPool,
        start_date: String,
        end_date: String,
    ) -> DbResult<Vec<CleaningTask>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT
                    ct.id, ct.booking_id, ct.room_id,
                    r.name as room_number,
                    r.ort as room_location,
                    b.reservierungsnummer,
                    ct.task_date::text,
                    b.checkin_date::text,
                    b.checkout_date::text,
                    ct.checkout_time::text,
                    ct.checkin_time::text,
                    ct.priority,
                    ct.has_dog,
                    ct.change_bedding,
                    ct.guest_count,
                    ct.guest_name,
                    ct.status,
                    ct.completed_at::text,
                    ct.completed_by,
                    ct.notes,
                    STRING_AGG(CASE WHEN st.cleaning_plan_position = 'start' AND st.show_in_cleaning_plan = true THEN st.emoji ELSE NULL END, ',') FILTER (WHERE st.emoji IS NOT NULL) as emojis_start,
                    STRING_AGG(CASE WHEN st.cleaning_plan_position = 'end' AND st.show_in_cleaning_plan = true THEN st.emoji ELSE NULL END, ',') FILTER (WHERE st.emoji IS NOT NULL) as emojis_end
                 FROM cleaning_tasks ct
                 JOIN rooms r ON r.id = ct.room_id
                 JOIN bookings b ON b.id = ct.booking_id
                 LEFT JOIN additional_services asrv ON asrv.booking_id = b.id
                 LEFT JOIN service_templates st ON st.id = asrv.template_id
                 WHERE ct.task_date::text BETWEEN $1 AND $2
                 GROUP BY ct.id, ct.booking_id, ct.room_id, ct.task_date, ct.checkout_time, ct.checkin_time,
                          ct.priority, ct.has_dog, ct.change_bedding, ct.guest_count, ct.guest_name,
                          ct.status, ct.completed_at, ct.completed_by, ct.notes,
                          r.id, r.name, r.ort, b.reservierungsnummer, b.checkin_date, b.checkout_date
                 ORDER BY ct.task_date, ct.priority DESC, r.id",
                &[&start_date, &end_date],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let checkin_date: Option<String> = row.try_get("checkin_date").ok();
                let checkout_date: Option<String> = row.try_get("checkout_date").ok();
                let guest_count: Option<i32> = row.try_get("guest_count").ok().flatten();

                // Build extras JSON
                let extras = serde_json::json!({
                    "original_checkin": checkin_date,
                    "original_checkout": checkout_date,
                    "guest_count": guest_count,
                }).to_string();

                CleaningTask {
                    id: row.get("id"),
                    booking_id: row.get("booking_id"),
                    room_id: row.get("room_id"),
                    room_number: row.try_get("room_number").ok(),
                    room_location: row.try_get("room_location").ok(),
                    reservierungsnummer: row.try_get("reservierungsnummer").ok(),
                    task_date: row.get("task_date"),
                    checkout_time: row.try_get("checkout_time").ok().flatten(),
                    checkin_time: row.try_get("checkin_time").ok().flatten(),
                    priority: row.get("priority"),
                    has_dog: row.get("has_dog"),
                    change_bedding: row.get("change_bedding"),
                    guest_count,
                    guest_name: row.try_get("guest_name").ok().flatten(),
                    emojis_start: row.try_get("emojis_start").ok().flatten(),
                    emojis_end: row.try_get("emojis_end").ok().flatten(),
                    status: row.get("status"),
                    completed_at: row.try_get("completed_at").ok().flatten(),
                    completed_by: row.try_get("completed_by").ok().flatten(),
                    notes: row.try_get("notes").ok().flatten(),
                    extras: Some(extras),
                }
            })
            .collect())
    }

    /// Get tasks for the next 7 days (week ahead)
    pub async fn get_week_ahead(pool: &DbPool) -> DbResult<Vec<CleaningTask>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT
                    ct.id, ct.booking_id, ct.room_id,
                    r.name as room_number,
                    r.ort as room_location,
                    b.reservierungsnummer,
                    ct.task_date::text,
                    b.checkin_date::text,
                    b.checkout_date::text,
                    ct.checkout_time::text,
                    ct.checkin_time::text,
                    ct.priority,
                    ct.has_dog,
                    ct.change_bedding,
                    ct.guest_count,
                    ct.guest_name,
                    ct.status,
                    ct.completed_at::text,
                    ct.completed_by,
                    ct.notes,
                    STRING_AGG(CASE WHEN st.cleaning_plan_position = 'start' AND st.show_in_cleaning_plan = true THEN st.emoji ELSE NULL END, ',') FILTER (WHERE st.emoji IS NOT NULL) as emojis_start,
                    STRING_AGG(CASE WHEN st.cleaning_plan_position = 'end' AND st.show_in_cleaning_plan = true THEN st.emoji ELSE NULL END, ',') FILTER (WHERE st.emoji IS NOT NULL) as emojis_end
                 FROM cleaning_tasks ct
                 JOIN rooms r ON r.id = ct.room_id
                 JOIN bookings b ON b.id = ct.booking_id
                 LEFT JOIN additional_services asrv ON asrv.booking_id = b.id
                 LEFT JOIN service_templates st ON st.id = asrv.template_id
                 WHERE ct.task_date BETWEEN CURRENT_DATE AND CURRENT_DATE + INTERVAL '7 days'
                   AND ct.status != 'cancelled'
                 GROUP BY ct.id, ct.booking_id, ct.room_id, ct.task_date, ct.checkout_time, ct.checkin_time,
                          ct.priority, ct.has_dog, ct.change_bedding, ct.guest_count, ct.guest_name,
                          ct.status, ct.completed_at, ct.completed_by, ct.notes,
                          r.id, r.name, r.ort, b.reservierungsnummer, b.checkin_date, b.checkout_date
                 ORDER BY ct.task_date, ct.priority DESC, r.name",
                &[],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let checkin_date: Option<String> = row.try_get("checkin_date").ok();
                let checkout_date: Option<String> = row.try_get("checkout_date").ok();
                let guest_count: Option<i32> = row.try_get("guest_count").ok().flatten();

                // Build extras JSON
                let extras = serde_json::json!({
                    "original_checkin": checkin_date,
                    "original_checkout": checkout_date,
                    "guest_count": guest_count,
                }).to_string();

                CleaningTask {
                    id: row.get("id"),
                    booking_id: row.get("booking_id"),
                    room_id: row.get("room_id"),
                    room_number: row.try_get("room_number").ok(),
                    room_location: row.try_get("room_location").ok(),
                    reservierungsnummer: row.try_get("reservierungsnummer").ok(),
                    task_date: row.get("task_date"),
                    checkout_time: row.try_get("checkout_time").ok().flatten(),
                    checkin_time: row.try_get("checkin_time").ok().flatten(),
                    priority: row.get("priority"),
                    has_dog: row.get("has_dog"),
                    change_bedding: row.get("change_bedding"),
                    guest_count,
                    guest_name: row.try_get("guest_name").ok().flatten(),
                    emojis_start: row.try_get("emojis_start").ok().flatten(),
                    emojis_end: row.try_get("emojis_end").ok().flatten(),
                    status: row.get("status"),
                    completed_at: row.try_get("completed_at").ok().flatten(),
                    completed_by: row.try_get("completed_by").ok().flatten(),
                    notes: row.try_get("notes").ok().flatten(),
                    extras: Some(extras),
                }
            })
            .collect())
    }

    /// Get cleaning statistics
    pub async fn get_stats(pool: &DbPool) -> DbResult<CleaningStats> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT * FROM get_cleaning_stats()", &[])
            .await?;

        Ok(CleaningStats {
            today: row.get("today"),
            tomorrow: row.get("tomorrow"),
            this_week: row.get("this_week"),
            total: row.get("total"),
        })
    }

    /// Update task status
    pub async fn update_status(
        pool: &DbPool,
        id: i32,
        status: String,
        completed_by: Option<String>,
    ) -> DbResult<CleaningTask> {
        let client = pool.get().await?;

        let completed_at = if status == "done" {
            Some("CURRENT_TIMESTAMP")
        } else {
            None
        };

        let query = if completed_at.is_some() {
            "UPDATE cleaning_tasks
             SET status = $2, completed_at = CURRENT_TIMESTAMP, completed_by = $3
             WHERE id = $1
             RETURNING id, booking_id, room_id, task_date::text, checkout_time::text,
                       checkin_time::text, priority, has_dog, change_bedding, guest_count,
                       guest_name, status, completed_at::text, completed_by, notes"
        } else {
            "UPDATE cleaning_tasks
             SET status = $2, completed_by = $3
             WHERE id = $1
             RETURNING id, booking_id, room_id, task_date::text, checkout_time::text,
                       checkin_time::text, priority, has_dog, change_bedding, guest_count,
                       guest_name, status, completed_at::text, completed_by, notes"
        };

        let row = client
            .query_one(query, &[&id, &status, &completed_by])
            .await
            .map_err(|_| DbError::NotFound(format!("Cleaning task {} not found", id)))?;

        Ok(CleaningTask {
            id: row.get("id"),
            booking_id: row.get("booking_id"),
            room_id: row.get("room_id"),
            room_number: None,
            room_location: None,
            reservierungsnummer: None,
            task_date: row.get("task_date"),
            checkout_time: row.try_get("checkout_time").ok().flatten(),
            checkin_time: row.try_get("checkin_time").ok().flatten(),
            priority: row.get("priority"),
            has_dog: row.get("has_dog"),
            change_bedding: row.get("change_bedding"),
            guest_count: row.try_get("guest_count").ok().flatten(),
            guest_name: row.try_get("guest_name").ok().flatten(),
            emojis_start: None,
            emojis_end: None,
            status: row.get("status"),
            completed_at: row.try_get("completed_at").ok().flatten(),
            completed_by: row.try_get("completed_by").ok().flatten(),
            notes: row.try_get("notes").ok().flatten(),
            extras: None,
        })
    }

    /// Delete old completed tasks (cleanup)
    pub async fn cleanup_old_tasks(pool: &DbPool, days_old: i32) -> DbResult<i32> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute(
                "DELETE FROM cleaning_tasks
                 WHERE status IN ('done', 'cancelled')
                   AND task_date < CURRENT_DATE - $1::integer",
                &[&days_old],
            )
            .await?;

        Ok(rows_affected as i32)
    }

    /// Delete tasks for a specific booking
    pub async fn delete_for_booking(pool: &DbPool, booking_id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        client
            .execute(
                "DELETE FROM cleaning_tasks WHERE booking_id = $1",
                &[&booking_id],
            )
            .await?;

        Ok(())
    }

    /// Mark task as synced to Turso
    pub async fn mark_synced(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE cleaning_tasks
                 SET synced_to_turso = TRUE, turso_sync_at = CURRENT_TIMESTAMP
                 WHERE id = $1",
                &[&id],
            )
            .await?;

        Ok(())
    }

    /// Get unsynced tasks
    pub async fn get_unsynced(pool: &DbPool) -> DbResult<Vec<CleaningTask>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT
                    ct.id, ct.booking_id, ct.room_id,
                    r.room_number,
                    ct.task_date::text,
                    ct.checkout_time::text,
                    ct.checkin_time::text,
                    ct.priority,
                    ct.has_dog,
                    ct.change_bedding,
                    ct.guest_count,
                    ct.guest_name,
                    ct.status,
                    ct.completed_at::text,
                    ct.completed_by,
                    ct.notes
                 FROM cleaning_tasks ct
                 JOIN rooms r ON r.id = ct.room_id
                 WHERE ct.synced_to_turso = FALSE
                   AND ct.status != 'cancelled'
                 ORDER BY ct.task_date",
                &[],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| CleaningTask {
                id: row.get("id"),
                booking_id: row.get("booking_id"),
                room_id: row.get("room_id"),
                room_number: row.try_get("room_number").ok(),
                room_location: None,
                reservierungsnummer: None,
                task_date: row.get("task_date"),
                checkout_time: row.try_get("checkout_time").ok().flatten(),
                checkin_time: row.try_get("checkin_time").ok().flatten(),
                priority: row.get("priority"),
                has_dog: row.get("has_dog"),
                change_bedding: row.get("change_bedding"),
                guest_count: row.try_get("guest_count").ok().flatten(),
                guest_name: row.try_get("guest_name").ok().flatten(),
                emojis_start: None,
                emojis_end: None,
                status: row.get("status"),
                completed_at: row.try_get("completed_at").ok().flatten(),
                completed_by: row.try_get("completed_by").ok().flatten(),
                notes: row.try_get("notes").ok().flatten(),
                extras: None,
            })
            .collect())
    }

    /// Run migration to create table and functions
    pub async fn run_migration(pool: &DbPool) -> DbResult<String> {
        let client = pool.get().await?;

        let migration_sql = include_str!("../../../../migrations/002_cleaning_tasks_system.sql");

        // Execute migration
        client
            .batch_execute(migration_sql)
            .await
            .map_err(|e| DbError::QueryError(format!("Migration failed: {}", e)))?;

        Ok("Cleaning tasks system migration completed successfully".to_string())
    }
}
