use crate::database_pg::{DbPool, DbResult, EmailConfig};

pub struct EmailConfigRepository;

impl EmailConfigRepository {
    /// Get email config (Singleton)
    pub async fn get(pool: &DbPool) -> DbResult<EmailConfig> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, provider, smtp_host, smtp_port, smtp_user, smtp_password,
                        from_email, from_name, use_tls, updated_at
                 FROM email_config
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound("Email config not found".to_string()))?;

        Ok(EmailConfig::from(row))
    }

    /// Update email config (UPSERT)
    pub async fn update(
        pool: &DbPool,
        provider: String,
        smtp_host: String,
        smtp_port: i32,
        smtp_user: String,
        smtp_password: String,
        from_email: String,
        from_name: String,
        use_tls: bool,
    ) -> DbResult<EmailConfig> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO email_config (
                    id, provider, smtp_host, smtp_port, smtp_user, smtp_password,
                    from_email, from_name, use_tls, updated_at
                 )
                 VALUES (1, $1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    provider = EXCLUDED.provider,
                    smtp_host = EXCLUDED.smtp_host,
                    smtp_port = EXCLUDED.smtp_port,
                    smtp_user = EXCLUDED.smtp_user,
                    smtp_password = EXCLUDED.smtp_password,
                    from_email = EXCLUDED.from_email,
                    from_name = EXCLUDED.from_name,
                    use_tls = EXCLUDED.use_tls,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, provider, smtp_host, smtp_port, smtp_user, smtp_password,
                           from_email, from_name, use_tls, updated_at",
                &[
                    &provider,
                    &smtp_host,
                    &smtp_port,
                    &smtp_user,
                    &smtp_password,
                    &from_email,
                    &from_name,
                    &use_tls,
                ],
            )
            .await?;

        Ok(EmailConfig::from(row))
    }
}
