use crate::database_pg::{DbPool, DbResult, NotificationSettings};

pub struct NotificationSettingsRepository;

impl NotificationSettingsRepository {
    /// Get notification settings (Singleton)
    pub async fn get(pool: &DbPool) -> DbResult<NotificationSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, checkin_reminders_enabled, checkin_reminder_before_days,
                        payment_reminders_enabled, payment_reminder_after_days,
                        payment_reminder_repeat_days, scheduler_interval_hours,
                        auto_reminder_incomplete_data, auto_reminder_payment,
                        payment_reminder_before_days,
                        auto_reminder_checkin, auto_reminder_invoice,
                        updated_at::text as updated_at
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
                    id, checkin_reminders_enabled, checkin_reminder_before_days,
                    payment_reminders_enabled, payment_reminder_after_days,
                    payment_reminder_repeat_days, scheduler_interval_hours,
                    auto_reminder_incomplete_data, auto_reminder_payment,
                    payment_reminder_before_days,
                    auto_reminder_checkin, auto_reminder_invoice, updated_at
                 )
                 VALUES (1, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    checkin_reminders_enabled = EXCLUDED.checkin_reminders_enabled,
                    checkin_reminder_before_days = EXCLUDED.checkin_reminder_before_days,
                    payment_reminders_enabled = EXCLUDED.payment_reminders_enabled,
                    payment_reminder_after_days = EXCLUDED.payment_reminder_after_days,
                    payment_reminder_repeat_days = EXCLUDED.payment_reminder_repeat_days,
                    scheduler_interval_hours = EXCLUDED.scheduler_interval_hours,
                    auto_reminder_incomplete_data = EXCLUDED.auto_reminder_incomplete_data,
                    auto_reminder_payment = EXCLUDED.auto_reminder_payment,
                    payment_reminder_before_days = EXCLUDED.payment_reminder_before_days,
                    auto_reminder_checkin = EXCLUDED.auto_reminder_checkin,
                    auto_reminder_invoice = EXCLUDED.auto_reminder_invoice,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, checkin_reminders_enabled, checkin_reminder_before_days,
                           payment_reminders_enabled, payment_reminder_after_days,
                           payment_reminder_repeat_days, scheduler_interval_hours,
                           auto_reminder_incomplete_data, auto_reminder_payment,
                           payment_reminder_before_days,
                           auto_reminder_checkin, auto_reminder_invoice,
                           updated_at::text as updated_at",
                &[
                    &settings.checkin_reminders_enabled,
                    &settings.checkin_reminder_before_days,
                    &settings.payment_reminders_enabled,
                    &settings.payment_reminder_after_days,
                    &settings.payment_reminder_repeat_days,
                    &settings.scheduler_interval_hours,
                    &settings.auto_reminder_incomplete_data,
                    &settings.auto_reminder_payment,
                    &settings.payment_reminder_before_days,
                    &settings.auto_reminder_checkin,
                    &settings.auto_reminder_invoice,
                ],
            )
            .await?;

        Ok(NotificationSettings::from(row))
    }
}
