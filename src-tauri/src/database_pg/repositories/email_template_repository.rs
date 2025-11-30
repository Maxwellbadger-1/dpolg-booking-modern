use crate::database_pg::{DbPool, DbResult, EmailTemplate};

pub struct EmailTemplateRepository;

impl EmailTemplateRepository {
    /// Get all email templates
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<EmailTemplate>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, template_name, subject, body, is_active, created_at::text as created_at, updated_at::text as updated_at
                 FROM email_templates
                 ORDER BY template_name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(EmailTemplate::from).collect())
    }

    /// Get email template by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<EmailTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, template_name, subject, body, is_active, created_at::text as created_at, updated_at::text as updated_at
                 FROM email_templates
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Email template with ID {} not found", id)))?;

        Ok(EmailTemplate::from(row))
    }

    /// Get email template by name
    pub async fn get_by_name(pool: &DbPool, template_name: String) -> DbResult<EmailTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, template_name, subject, body, is_active, created_at::text as created_at, updated_at::text as updated_at
                 FROM email_templates
                 WHERE template_name = $1",
                &[&template_name],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Email template '{}' not found", template_name)))?;

        Ok(EmailTemplate::from(row))
    }

    /// Get active email templates
    pub async fn get_active(pool: &DbPool) -> DbResult<Vec<EmailTemplate>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, template_name, subject, body, is_active, created_at::text as created_at, updated_at::text as updated_at
                 FROM email_templates
                 WHERE is_active = TRUE
                 ORDER BY template_name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(EmailTemplate::from).collect())
    }

    /// Create new email template
    pub async fn create(
        pool: &DbPool,
        template_name: String,
        subject: String,
        body: String,
    ) -> DbResult<EmailTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO email_templates (
                    template_name, subject, body, is_active, created_at::text as created_at, updated_at::text as updated_at
                 ) VALUES ($1, $2, $3, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 RETURNING id, template_name, subject, body, is_active, created_at::text as created_at, updated_at::text as updated_at",
                &[&template_name, &subject, &body],
            )
            .await?;

        Ok(EmailTemplate::from(row))
    }

    /// Update email template
    pub async fn update(
        pool: &DbPool,
        id: i32,
        template_name: String,
        subject: String,
        body: String,
        is_active: bool,
    ) -> DbResult<EmailTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE email_templates SET
                    template_name = $2, subject = $3, body = $4, is_active = $5, updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1
                 RETURNING id, template_name, subject, body, is_active, created_at::text as created_at, updated_at::text as updated_at",
                &[&id, &template_name, &subject, &body, &is_active],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Email template with ID {} not found", id)))?;

        Ok(EmailTemplate::from(row))
    }

    /// Toggle email template active status
    pub async fn toggle_active(pool: &DbPool, id: i32) -> DbResult<EmailTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE email_templates SET
                    is_active = NOT is_active, updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1
                 RETURNING id, template_name, subject, body, is_active, created_at::text as created_at, updated_at::text as updated_at",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Email template with ID {} not found", id)))?;

        Ok(EmailTemplate::from(row))
    }

    /// Delete email template
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM email_templates WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Email template with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Count email templates
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM email_templates", &[])
            .await?;

        Ok(row.get("count"))
    }
}
