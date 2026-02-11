use crate::database_pg::{DbPool, DbResult, EmailLog};

pub struct EmailLogRepository;

impl EmailLogRepository {
    /// Get all email logs
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<EmailLog>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, guest_id, template_name, recipient_email,
                        subject, status, error_message, sent_at::text as sent_at
                 FROM email_logs
                 ORDER BY sent_at DESC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(EmailLog::from).collect())
    }

    /// Get email log by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<EmailLog> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, booking_id, guest_id, template_name, recipient_email,
                        subject, status, error_message, sent_at::text as sent_at
                 FROM email_logs
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Email log with ID {} not found", id)))?;

        Ok(EmailLog::from(row))
    }

    /// Get email logs by booking ID
    pub async fn get_by_booking(pool: &DbPool, booking_id: i32) -> DbResult<Vec<EmailLog>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, guest_id, template_name, recipient_email,
                        subject, status, error_message, sent_at::text as sent_at
                 FROM email_logs
                 WHERE booking_id = $1
                 ORDER BY sent_at DESC",
                &[&booking_id],
            )
            .await?;

        Ok(rows.into_iter().map(EmailLog::from).collect())
    }

    /// Get email logs by guest ID
    pub async fn get_by_guest(pool: &DbPool, guest_id: i32) -> DbResult<Vec<EmailLog>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, guest_id, template_name, recipient_email,
                        subject, status, error_message, sent_at::text as sent_at
                 FROM email_logs
                 WHERE guest_id = $1
                 ORDER BY sent_at DESC",
                &[&guest_id],
            )
            .await?;

        Ok(rows.into_iter().map(EmailLog::from).collect())
    }

    /// Get email logs by status
    pub async fn get_by_status(pool: &DbPool, status: String) -> DbResult<Vec<EmailLog>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, guest_id, template_name, recipient_email,
                        subject, status, error_message, sent_at::text as sent_at
                 FROM email_logs
                 WHERE status = $1
                 ORDER BY sent_at DESC",
                &[&status],
            )
            .await?;

        Ok(rows.into_iter().map(EmailLog::from).collect())
    }

    /// Create new email log
    pub async fn create(
        pool: &DbPool,
        booking_id: Option<i32>,
        guest_id: i32,
        template_name: String,
        recipient_email: String,
        subject: String,
        status: String,
        error_message: Option<String>,
    ) -> DbResult<EmailLog> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO email_logs (
                    booking_id, guest_id, template_name, recipient_email,
                    subject, status, error_message, sent_at
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP)
                 RETURNING id, booking_id, guest_id, template_name, recipient_email,
                           subject, status, error_message, sent_at::text as sent_at",
                &[
                    &booking_id, &guest_id, &template_name, &recipient_email,
                    &subject, &status, &error_message,
                ],
            )
            .await?;

        Ok(EmailLog::from(row))
    }

    /// Update email log status
    pub async fn update_status(
        pool: &DbPool,
        id: i32,
        status: String,
        error_message: Option<String>,
    ) -> DbResult<EmailLog> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE email_logs SET
                    status = $2, error_message = $3
                 WHERE id = $1
                 RETURNING id, booking_id, guest_id, template_name, recipient_email,
                           subject, status, error_message, sent_at::text as sent_at",
                &[&id, &status, &error_message],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Email log with ID {} not found", id)))?;

        Ok(EmailLog::from(row))
    }

    /// Delete email log
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM email_logs WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Email log with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Get email log count
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM email_logs", &[])
            .await?;

        Ok(row.get("count"))
    }

    /// Get failed emails
    pub async fn get_failed(pool: &DbPool) -> DbResult<Vec<EmailLog>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, guest_id, template_name, recipient_email,
                        subject, status, error_message, sent_at::text as sent_at
                 FROM email_logs
                 WHERE status = 'fehler'
                 ORDER BY sent_at DESC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(EmailLog::from).collect())
    }
}
