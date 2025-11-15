use crate::database_pg::{DbPool, DbResult, NotificationSettings};

pub struct NotificationSettingsRepository;

impl NotificationSettingsRepository {
    /// Get notification settings (Singleton)
    pub async fn get(pool: &DbPool) -> DbResult<NotificationSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, enable_email_notifications, enable_reminder_notifications,
                        reminder_days_before, updated_at
                 FROM notification_settings
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound("Notification settings not found".to_string()))?;

        Ok(NotificationSettings::from(row))
    }

    /// Update notification settings (UPSERT)
    pub async fn update(
        pool: &DbPool,
        enable_email_notifications: bool,
        enable_reminder_notifications: bool,
        reminder_days_before: i32,
    ) -> DbResult<NotificationSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO notification_settings (
                    id, enable_email_notifications, enable_reminder_notifications,
                    reminder_days_before, updated_at
                 )
                 VALUES (1, $1, $2, $3, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    enable_email_notifications = EXCLUDED.enable_email_notifications,
                    enable_reminder_notifications = EXCLUDED.enable_reminder_notifications,
                    reminder_days_before = EXCLUDED.reminder_days_before,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, enable_email_notifications, enable_reminder_notifications,
                           reminder_days_before, updated_at",
                &[
                    &enable_email_notifications,
                    &enable_reminder_notifications,
                    &reminder_days_before,
                ],
            )
            .await?;

        Ok(NotificationSettings::from(row))
    }
}
