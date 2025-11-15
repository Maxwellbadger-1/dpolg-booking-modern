use crate::database_pg::{DbPool, DbResult, PricingSettings};

pub struct PricingSettingsRepository;

impl PricingSettingsRepository {
    /// Get pricing settings (Singleton - should only have 1 record)
    pub async fn get(pool: &DbPool) -> DbResult<PricingSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, standard_discount_percent, family_discount_percent,
                        member_discount_name, non_member_discount_name, vat_rate, updated_at
                 FROM pricing_settings
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound("Pricing settings not found".to_string()))?;

        Ok(PricingSettings::from(row))
    }

    /// Update pricing settings
    pub async fn update(
        pool: &DbPool,
        standard_discount_percent: f64,
        family_discount_percent: f64,
        member_discount_name: String,
        non_member_discount_name: String,
        vat_rate: f64,
    ) -> DbResult<PricingSettings> {
        let client = pool.get().await?;

        // Update first record (or insert if none exists)
        let row = client
            .query_one(
                "INSERT INTO pricing_settings (
                    id, standard_discount_percent, family_discount_percent,
                    member_discount_name, non_member_discount_name, vat_rate, updated_at
                 )
                 VALUES (1, $1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    standard_discount_percent = EXCLUDED.standard_discount_percent,
                    family_discount_percent = EXCLUDED.family_discount_percent,
                    member_discount_name = EXCLUDED.member_discount_name,
                    non_member_discount_name = EXCLUDED.non_member_discount_name,
                    vat_rate = EXCLUDED.vat_rate,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, standard_discount_percent, family_discount_percent,
                           member_discount_name, non_member_discount_name, vat_rate, updated_at",
                &[
                    &standard_discount_percent,
                    &family_discount_percent,
                    &member_discount_name,
                    &non_member_discount_name,
                    &vat_rate,
                ],
            )
            .await?;

        Ok(PricingSettings::from(row))
    }
}
