use crate::database_pg::{DbPool, DbError, DbResult, models::ActiveLock};

/// Repository for managing advisory locks (presence system)
pub struct LockRepository;

impl LockRepository {
    /// Acquire a lock on a booking for a user
    /// Returns error if booking is already locked by another user
    pub async fn acquire_lock(
        pool: &DbPool,
        booking_id: i32,
        user_name: String,
    ) -> DbResult<ActiveLock> {
        let client = pool.get().await?;

        // Clean up stale locks first (older than 5 minutes)
        client
            .execute("SELECT cleanup_stale_locks()", &[])
            .await?;

        // Try to insert lock (will fail if already locked due to UNIQUE constraint)
        let result = client
            .query_one(
                "INSERT INTO active_locks (booking_id, user_name, locked_at, last_heartbeat)
                 VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 ON CONFLICT (booking_id) DO NOTHING
                 RETURNING id, booking_id, user_name, locked_at, last_heartbeat",
                &[&booking_id, &user_name],
            )
            .await;

        match result {
            Ok(row) => Ok(ActiveLock::from(row)),
            Err(_) => {
                // Lock already exists - check who owns it
                let existing_lock = client
                    .query_one(
                        "SELECT id, booking_id, user_name, locked_at, last_heartbeat
                         FROM active_locks
                         WHERE booking_id = $1",
                        &[&booking_id],
                    )
                    .await?;

                let owner: String = existing_lock.get("user_name");

                if owner == user_name {
                    // Same user - allow (refresh heartbeat)
                    Self::update_heartbeat(pool, booking_id).await?;
                    Ok(ActiveLock::from(existing_lock))
                } else {
                    // Different user - return conflict error
                    Err(DbError::ConflictError(format!(
                        "Buchung wird bereits von '{}' bearbeitet",
                        owner
                    )))
                }
            }
        }
    }

    /// Release a lock on a booking
    pub async fn release_lock(pool: &DbPool, booking_id: i32, user_name: String) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute(
                "DELETE FROM active_locks WHERE booking_id = $1 AND user_name = $2",
                &[&booking_id, &user_name],
            )
            .await?;

        if rows_affected == 0 {
            // No lock found - this is OK (might have timed out already)
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Update heartbeat for a lock (keep-alive)
    pub async fn update_heartbeat(pool: &DbPool, booking_id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE active_locks
                 SET last_heartbeat = CURRENT_TIMESTAMP
                 WHERE booking_id = $1",
                &[&booking_id],
            )
            .await?;

        Ok(())
    }

    /// Get all active locks
    pub async fn get_all_locks(pool: &DbPool) -> DbResult<Vec<ActiveLock>> {
        let client = pool.get().await?;

        // Clean up stale locks first
        client
            .execute("SELECT cleanup_stale_locks()", &[])
            .await?;

        let rows = client
            .query(
                "SELECT id, booking_id, user_name, locked_at, last_heartbeat
                 FROM active_locks
                 ORDER BY locked_at DESC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(ActiveLock::from).collect())
    }

    /// Get lock for specific booking (if exists)
    pub async fn get_lock_for_booking(
        pool: &DbPool,
        booking_id: i32,
    ) -> DbResult<Option<ActiveLock>> {
        let client = pool.get().await?;

        // Clean up stale locks first
        client
            .execute("SELECT cleanup_stale_locks()", &[])
            .await?;

        let result = client
            .query_opt(
                "SELECT id, booking_id, user_name, locked_at, last_heartbeat
                 FROM active_locks
                 WHERE booking_id = $1",
                &[&booking_id],
            )
            .await?;

        Ok(result.map(ActiveLock::from))
    }

    /// Force release all locks for a user (e.g., on logout or crash)
    pub async fn release_all_user_locks(pool: &DbPool, user_name: String) -> DbResult<()> {
        let client = pool.get().await?;

        client
            .execute(
                "DELETE FROM active_locks WHERE user_name = $1",
                &[&user_name],
            )
            .await?;

        Ok(())
    }
}
