use crate::database_pg::{DbPool, DbResult, PricingSettings};

pub struct PricingSettingsRepository;

impl PricingSettingsRepository {
    /// Get pricing settings (Singleton - should only have 1 record)
    pub async fn get(pool: &DbPool) -> DbResult<PricingSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende,
                        mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis,
                        updated_at::text as updated_at
                 FROM pricing_settings
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|e| {
                println!("PricingSettings query error: {:?}", e);
                crate::database_pg::DbError::NotFound("Pricing settings not found".to_string())
            })?;

        Ok(PricingSettings::from(row))
    }

    /// Update pricing settings (UPSERT)
    pub async fn update(
        pool: &DbPool,
        settings: &PricingSettings,
    ) -> DbResult<PricingSettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO pricing_settings (
                    id, hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende,
                    mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis, updated_at
                 )
                 VALUES (1, $1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    hauptsaison_aktiv = EXCLUDED.hauptsaison_aktiv,
                    hauptsaison_start = EXCLUDED.hauptsaison_start,
                    hauptsaison_ende = EXCLUDED.hauptsaison_ende,
                    mitglieder_rabatt_aktiv = EXCLUDED.mitglieder_rabatt_aktiv,
                    mitglieder_rabatt_prozent = EXCLUDED.mitglieder_rabatt_prozent,
                    rabatt_basis = EXCLUDED.rabatt_basis,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, hauptsaison_aktiv, hauptsaison_start, hauptsaison_ende,
                           mitglieder_rabatt_aktiv, mitglieder_rabatt_prozent, rabatt_basis,
                           updated_at::text as updated_at",
                &[
                    &settings.hauptsaison_aktiv,
                    &settings.hauptsaison_start,
                    &settings.hauptsaison_ende,
                    &settings.mitglieder_rabatt_aktiv,
                    &settings.mitglieder_rabatt_prozent,
                    &settings.rabatt_basis,
                ],
            )
            .await?;

        Ok(PricingSettings::from(row))
    }
}
