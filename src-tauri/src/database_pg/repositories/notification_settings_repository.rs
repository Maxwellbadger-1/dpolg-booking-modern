use crate::database_pg::{DbPool, DbResult, NotificationSettings};

pub struct NotificationSettingsRepository;

impl NotificationSettingsRepository {
    /// Get notification settings (Singleton)
    pub async fn get(pool: &DbPool) -> DbResult<NotificationSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, checkin_reminders_enabled, payment_reminders_enabled,
                        payment_reminder_after_days, payment_reminder_repeat_days,
                        scheduler_interval_hours, updated_at::text as updated_at
                 FROM notification_settings
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|e| {
                println!("NotificationSettings query error: {:?}", e);
                crate::database_pg::DbError::NotFound("Notification settings not found".to_string())
            })?;

        Ok(NotificationSettings::from(row))
    }

    /// Update notification settings (UPSERT)
    pub async fn update(
        pool: &DbPool,
        settings: &NotificationSettings,
    ) -> DbResult<NotificationSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO notification_settings (
                    id, checkin_reminders_enabled, payment_reminders_enabled,
                    payment_reminder_after_days, payment_reminder_repeat_days,
                    scheduler_interval_hours, updated_at
                 )
                 VALUES (1, $1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    checkin_reminders_enabled = EXCLUDED.checkin_reminders_enabled,
                    payment_reminders_enabled = EXCLUDED.payment_reminders_enabled,
                    payment_reminder_after_days = EXCLUDED.payment_reminder_after_days,
                    payment_reminder_repeat_days = EXCLUDED.payment_reminder_repeat_days,
                    scheduler_interval_hours = EXCLUDED.scheduler_interval_hours,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, checkin_reminders_enabled, payment_reminders_enabled,
                           payment_reminder_after_days, payment_reminder_repeat_days,
                           scheduler_interval_hours, updated_at::text as updated_at",
                &[
                    &settings.checkin_reminders_enabled,
                    &settings.payment_reminders_enabled,
                    &settings.payment_reminder_after_days,
                    &settings.payment_reminder_repeat_days,
                    &settings.scheduler_interval_hours,
                ],
            )
            .await?;

        Ok(NotificationSettings::from(row))
    }
}
