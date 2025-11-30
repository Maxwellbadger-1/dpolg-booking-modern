use crate::database_pg::{DbPool, DbResult, EmailConfig};

pub struct EmailConfigRepository;

impl EmailConfigRepository {
    /// Get email config (Singleton)
    pub async fn get(pool: &DbPool) -> DbResult<EmailConfig> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, smtp_server, smtp_port, smtp_username, smtp_password,
                        from_email, from_name, use_tls,
                        created_at::text as created_at, updated_at::text as updated_at
                 FROM email_config
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|e| {
                println!("EmailConfig query error: {:?}", e);
                crate::database_pg::DbError::NotFound("Email config not found".to_string())
            })?;

        Ok(EmailConfig::from(row))
    }

    /// Update email config (UPSERT)
    pub async fn update(
        pool: &DbPool,
        settings: &EmailConfig,
    ) -> DbResult<EmailConfig> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO email_config (
                    id, smtp_server, smtp_port, smtp_username, smtp_password,
                    from_email, from_name, use_tls, created_at, updated_at
                 )
                 VALUES (1, $1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    smtp_server = EXCLUDED.smtp_server,
                    smtp_port = EXCLUDED.smtp_port,
                    smtp_username = EXCLUDED.smtp_username,
                    smtp_password = EXCLUDED.smtp_password,
                    from_email = EXCLUDED.from_email,
                    from_name = EXCLUDED.from_name,
                    use_tls = EXCLUDED.use_tls,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, smtp_server, smtp_port, smtp_username, smtp_password,
                           from_email, from_name, use_tls,
                           created_at::text as created_at, updated_at::text as updated_at",
                &[
                    &settings.smtp_server,
                    &settings.smtp_port,
                    &settings.smtp_username,
                    &settings.smtp_password,
                    &settings.from_email,
                    &settings.from_name,
                    &settings.use_tls,
                ],
            )
            .await?;

        Ok(EmailConfig::from(row))
    }
}
