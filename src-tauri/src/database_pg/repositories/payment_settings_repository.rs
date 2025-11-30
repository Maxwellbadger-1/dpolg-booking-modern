use crate::database_pg::{DbPool, DbResult, PaymentSettings};

pub struct PaymentSettingsRepository;

impl PaymentSettingsRepository {
    /// Get payment settings (Singleton)
    pub async fn get(pool: &DbPool) -> DbResult<PaymentSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, bank_name, iban, bic, account_holder, mwst_rate,
                        payment_due_days, reminder_after_days, payment_text,
                        updated_at::text as updated_at, dpolg_rabatt
                 FROM payment_settings
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|e| {
                println!("PaymentSettings query error: {:?}", e);
                crate::database_pg::DbError::NotFound("Payment settings not found".to_string())
            })?;

        Ok(PaymentSettings::from(row))
    }

    /// Update payment settings (UPSERT)
    pub async fn update(
        pool: &DbPool,
        settings: &PaymentSettings,
    ) -> DbResult<PaymentSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO payment_settings (
                    id, bank_name, iban, bic, account_holder, mwst_rate,
                    payment_due_days, reminder_after_days, payment_text, updated_at, dpolg_rabatt
                 )
                 VALUES (1, $1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP, $9)
                 ON CONFLICT (id) DO UPDATE SET
                    bank_name = EXCLUDED.bank_name,
                    iban = EXCLUDED.iban,
                    bic = EXCLUDED.bic,
                    account_holder = EXCLUDED.account_holder,
                    mwst_rate = EXCLUDED.mwst_rate,
                    payment_due_days = EXCLUDED.payment_due_days,
                    reminder_after_days = EXCLUDED.reminder_after_days,
                    payment_text = EXCLUDED.payment_text,
                    dpolg_rabatt = EXCLUDED.dpolg_rabatt,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, bank_name, iban, bic, account_holder, mwst_rate,
                           payment_due_days, reminder_after_days, payment_text,
                           updated_at::text as updated_at, dpolg_rabatt",
                &[
                    &settings.bank_name,
                    &settings.iban,
                    &settings.bic,
                    &settings.account_holder,
                    &settings.mwst_rate,
                    &settings.payment_due_days,
                    &settings.reminder_after_days,
                    &settings.payment_text,
                    &settings.dpolg_rabatt,
                ],
            )
            .await?;

        Ok(PaymentSettings::from(row))
    }
}
