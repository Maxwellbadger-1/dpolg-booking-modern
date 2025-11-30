use crate::database_pg::{DbPool, DbResult, DbError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledEmail {
    pub id: i32,
    pub booking_id: Option<i32>,
    pub guest_id: Option<i32>,
    pub template_name: String,
    pub recipient_email: String,
    pub subject: String,
    pub scheduled_for: String,
    pub status: String,
    pub sent_at: Option<String>,
    pub error_message: Option<String>,
}

pub struct ScheduledEmailRepository;

impl ScheduledEmailRepository {
    /// Get all scheduled emails
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<ScheduledEmail>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, guest_id, template_name, recipient_email,
                        subject, scheduled_for::text, status, sent_at::text, error_message
                 FROM scheduled_emails
                 ORDER BY scheduled_for DESC",
                &[],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| ScheduledEmail {
                id: row.get("id"),
                booking_id: row.try_get("booking_id").ok().flatten(),
                guest_id: row.try_get("guest_id").ok().flatten(),
                template_name: row.get("template_name"),
                recipient_email: row.get("recipient_email"),
                subject: row.get("subject"),
                scheduled_for: row.get("scheduled_for"),
                status: row.get("status"),
                sent_at: row.try_get("sent_at").ok().flatten(),
                error_message: row.try_get("error_message").ok().flatten(),
            })
            .collect())
    }

    /// Get pending emails that should be sent
    pub async fn get_pending(pool: &DbPool) -> DbResult<Vec<ScheduledEmail>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, guest_id, template_name, recipient_email, subject,
                        scheduled_for::text as scheduled_for
                 FROM get_pending_scheduled_emails()",
                &[],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| ScheduledEmail {
                id: row.get("id"),
                booking_id: row.try_get("booking_id").ok().flatten(),
                guest_id: row.try_get("guest_id").ok().flatten(),
                template_name: row.get("template_name"),
                recipient_email: row.get("recipient_email"),
                subject: row.get("subject"),
                scheduled_for: row.get("scheduled_for"),
                status: "pending".to_string(),
                sent_at: None,
                error_message: None,
            })
            .collect())
    }

    /// Update email status (generic)
    pub async fn update_status(pool: &DbPool, id: i32, status: &str) -> DbResult<()> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE scheduled_emails
                 SET status = $2, sent_at = CASE WHEN $2 = 'sent' THEN CURRENT_TIMESTAMP ELSE sent_at END
                 WHERE id = $1",
                &[&id, &status],
            )
            .await?;

        Ok(())
    }

    /// Update email status with error message
    pub async fn update_status_with_error(pool: &DbPool, id: i32, status: &str, error: &str) -> DbResult<()> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE scheduled_emails
                 SET status = $2, error_message = $3
                 WHERE id = $1",
                &[&id, &status, &error],
            )
            .await?;

        Ok(())
    }

    /// Mark email as sent (legacy)
    pub async fn mark_sent(pool: &DbPool, id: i32) -> DbResult<()> {
        Self::update_status(pool, id, "sent").await
    }

    /// Mark email as failed (legacy)
    pub async fn mark_failed(pool: &DbPool, id: i32, error: String) -> DbResult<()> {
        Self::update_status_with_error(pool, id, "failed", &error).await
    }

    /// Cancel scheduled email
    pub async fn cancel(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        client
            .execute(
                "UPDATE scheduled_emails SET status = 'cancelled' WHERE id = $1",
                &[&id],
            )
            .await?;

        Ok(())
    }

    /// Create scheduled email
    pub async fn create(
        pool: &DbPool,
        booking_id: Option<i32>,
        guest_id: Option<i32>,
        template_name: String,
        recipient_email: String,
        subject: String,
        scheduled_for: String,
    ) -> DbResult<ScheduledEmail> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO scheduled_emails (booking_id, guest_id, template_name, recipient_email, subject, scheduled_for)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 RETURNING id, booking_id, guest_id, template_name, recipient_email, subject, scheduled_for::text, status, sent_at::text, error_message",
                &[&booking_id, &guest_id, &template_name, &recipient_email, &subject, &scheduled_for],
            )
            .await?;

        Ok(ScheduledEmail {
            id: row.get("id"),
            booking_id: row.try_get("booking_id").ok().flatten(),
            guest_id: row.try_get("guest_id").ok().flatten(),
            template_name: row.get("template_name"),
            recipient_email: row.get("recipient_email"),
            subject: row.get("subject"),
            scheduled_for: row.get("scheduled_for"),
            status: row.get("status"),
            sent_at: row.try_get("sent_at").ok().flatten(),
            error_message: row.try_get("error_message").ok().flatten(),
        })
    }

    /// Run migration
    pub async fn run_migration(pool: &DbPool) -> DbResult<String> {
        let client = pool.get().await?;

        let migration_sql = include_str!("../../../../migrations/003_email_automation_system.sql");

        client
            .batch_execute(migration_sql)
            .await
            .map_err(|e| DbError::QueryError(format!("Migration failed: {}", e)))?;

        Ok("Email automation system migration completed successfully".to_string())
    }
}
