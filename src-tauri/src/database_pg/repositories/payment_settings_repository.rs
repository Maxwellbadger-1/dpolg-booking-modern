use crate::database_pg::{DbPool, DbResult, PaymentSettings};

pub struct PaymentSettingsRepository;

impl PaymentSettingsRepository {
    /// Get payment settings (Singleton)
    pub async fn get(pool: &DbPool) -> DbResult<PaymentSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, default_payment_method, require_payment_confirmation,
                        payment_deadline_days, updated_at
                 FROM payment_settings
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound("Payment settings not found".to_string()))?;

        Ok(PaymentSettings::from(row))
    }

    /// Update payment settings (UPSERT)
    pub async fn update(
        pool: &DbPool,
        default_payment_method: String,
        require_payment_confirmation: bool,
        payment_deadline_days: i32,
    ) -> DbResult<PaymentSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO payment_settings (
                    id, default_payment_method, require_payment_confirmation,
                    payment_deadline_days, updated_at
                 )
                 VALUES (1, $1, $2, $3, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    default_payment_method = EXCLUDED.default_payment_method,
                    require_payment_confirmation = EXCLUDED.require_payment_confirmation,
                    payment_deadline_days = EXCLUDED.payment_deadline_days,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, default_payment_method, require_payment_confirmation,
                           payment_deadline_days, updated_at",
                &[
                    &default_payment_method,
                    &require_payment_confirmation,
                    &payment_deadline_days,
                ],
            )
            .await?;

        Ok(PaymentSettings::from(row))
    }
}
