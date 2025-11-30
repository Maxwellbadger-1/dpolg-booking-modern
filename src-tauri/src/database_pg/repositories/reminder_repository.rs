use crate::database_pg::{DbPool, DbResult, Reminder};

pub struct ReminderRepository;

impl ReminderRepository {
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<Reminder>> {
        let client = pool.get().await?;
        let rows = client.query(
            "SELECT id, booking_id, reminder_type, title, description, due_date,
                    priority, is_completed, completed_at::text as completed_at, is_snoozed, snoozed_until::text as snoozed_until,
                    created_at::text as created_at, updated_at::text as updated_at
             FROM reminders ORDER BY due_date ASC, priority DESC",
            &[],
        ).await?;
        Ok(rows.into_iter().map(Reminder::from).collect())
    }

    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<Reminder> {
        let client = pool.get().await?;
        let row = client.query_one(
            "SELECT id, booking_id, reminder_type, title, description, due_date,
                    priority, is_completed, completed_at::text as completed_at, is_snoozed, snoozed_until::text as snoozed_until,
                    created_at::text as created_at, updated_at::text as updated_at
             FROM reminders WHERE id = $1",
            &[&id],
        ).await.map_err(|_| crate::database_pg::DbError::NotFound(format!("Reminder {} not found", id)))?;
        Ok(Reminder::from(row))
    }

    pub async fn get_by_booking(pool: &DbPool, booking_id: i32) -> DbResult<Vec<Reminder>> {
        let client = pool.get().await?;
        let rows = client.query(
            "SELECT id, booking_id, reminder_type, title, description, due_date,
                    priority, is_completed, completed_at::text as completed_at, is_snoozed, snoozed_until::text as snoozed_until,
                    created_at::text as created_at, updated_at::text as updated_at
             FROM reminders WHERE booking_id = $1 ORDER BY due_date ASC",
            &[&booking_id],
        ).await?;
        Ok(rows.into_iter().map(Reminder::from).collect())
    }

    pub async fn get_active(pool: &DbPool) -> DbResult<Vec<Reminder>> {
        let client = pool.get().await?;
        let rows = client.query(
            "SELECT id, booking_id, reminder_type, title, description, due_date,
                    priority, is_completed, completed_at::text as completed_at, is_snoozed, snoozed_until::text as snoozed_until,
                    created_at::text as created_at, updated_at::text as updated_at
             FROM reminders
             WHERE is_completed = FALSE AND is_snoozed = FALSE
             ORDER BY due_date ASC, priority DESC",
            &[],
        ).await?;
        Ok(rows.into_iter().map(Reminder::from).collect())
    }

    pub async fn create(
        pool: &DbPool,
        booking_id: Option<i32>,
        reminder_type: String,
        title: String,
        description: Option<String>,
        due_date: String,
        priority: String,
    ) -> DbResult<Reminder> {
        let client = pool.get().await?;
        let row = client.query_one(
            "INSERT INTO reminders (
                booking_id, reminder_type, title, description, due_date, priority,
                is_completed, is_snoozed, created_at, updated_at
             ) VALUES ($1, $2, $3, $4, $5, $6, FALSE, FALSE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id, booking_id, reminder_type, title, description, due_date,
                       priority, is_completed, completed_at::text as completed_at, is_snoozed, snoozed_until::text as snoozed_until,
                       created_at::text as created_at, updated_at::text as updated_at",
            &[&booking_id, &reminder_type, &title, &description, &due_date, &priority],
        ).await?;
        Ok(Reminder::from(row))
    }

    pub async fn update(
        pool: &DbPool,
        id: i32,
        title: String,
        description: Option<String>,
        due_date: String,
        priority: String,
    ) -> DbResult<Reminder> {
        let client = pool.get().await?;
        let row = client.query_one(
            "UPDATE reminders SET
                title = $2, description = $3, due_date = $4, priority = $5,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id, booking_id, reminder_type, title, description, due_date,
                       priority, is_completed, completed_at::text as completed_at, is_snoozed, snoozed_until::text as snoozed_until,
                       created_at::text as created_at, updated_at::text as updated_at",
            &[&id, &title, &description, &due_date, &priority],
        ).await.map_err(|_| crate::database_pg::DbError::NotFound(format!("Reminder {} not found", id)))?;
        Ok(Reminder::from(row))
    }

    pub async fn complete(pool: &DbPool, id: i32) -> DbResult<Reminder> {
        let client = pool.get().await?;
        let row = client.query_one(
            "UPDATE reminders SET
                is_completed = TRUE, completed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id, booking_id, reminder_type, title, description, due_date,
                       priority, is_completed, completed_at::text as completed_at, is_snoozed, snoozed_until::text as snoozed_until,
                       created_at::text as created_at, updated_at::text as updated_at",
            &[&id],
        ).await.map_err(|_| crate::database_pg::DbError::NotFound(format!("Reminder {} not found", id)))?;
        Ok(Reminder::from(row))
    }

    pub async fn snooze(pool: &DbPool, id: i32, snoozed_until: String) -> DbResult<Reminder> {
        let client = pool.get().await?;
        let row = client.query_one(
            "UPDATE reminders SET
                is_snoozed = TRUE, snoozed_until = $2, updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id, booking_id, reminder_type, title, description, due_date,
                       priority, is_completed, completed_at::text as completed_at, is_snoozed, snoozed_until::text as snoozed_until,
                       created_at::text as created_at, updated_at::text as updated_at",
            &[&id, &snoozed_until],
        ).await.map_err(|_| crate::database_pg::DbError::NotFound(format!("Reminder {} not found", id)))?;
        Ok(Reminder::from(row))
    }

    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;
        let rows = client.execute("DELETE FROM reminders WHERE id = $1", &[&id]).await?;
        if rows == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!("Reminder {} not found", id)));
        }
        Ok(())
    }

    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) as count FROM reminders", &[]).await?;
        Ok(row.get("count"))
    }
}
