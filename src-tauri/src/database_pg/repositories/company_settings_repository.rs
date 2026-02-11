use crate::database_pg::{DbPool, DbResult, CompanySettings};

pub struct CompanySettingsRepository;

impl CompanySettingsRepository {
    /// Get company settings (Singleton - should only have 1 record)
    pub async fn get(pool: &DbPool) -> DbResult<CompanySettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, company_name, street_address, plz, city, country, phone, fax, email, website, tax_id, ceo_name, registry_court, logo_path, logo_data, logo_mime_type, updated_at::text as updated_at
                 FROM company_settings
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|e| {
                crate::database_pg::DbError::NotFound(format!("Company settings not found: {:?}", e))
            })?;

        Ok(CompanySettings::from(row))
    }

    /// Update company settings
    pub async fn update(
        pool: &DbPool,
        settings: &CompanySettings,
    ) -> DbResult<CompanySettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO company_settings (id, company_name, street_address, plz, city, country, phone, fax, email, website, tax_id, ceo_name, registry_court, logo_path, logo_data, logo_mime_type, updated_at)
                 VALUES (1, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    company_name = EXCLUDED.company_name,
                    street_address = EXCLUDED.street_address,
                    plz = EXCLUDED.plz,
                    city = EXCLUDED.city,
                    country = EXCLUDED.country,
                    phone = EXCLUDED.phone,
                    fax = EXCLUDED.fax,
                    email = EXCLUDED.email,
                    website = EXCLUDED.website,
                    tax_id = EXCLUDED.tax_id,
                    ceo_name = EXCLUDED.ceo_name,
                    registry_court = EXCLUDED.registry_court,
                    logo_path = EXCLUDED.logo_path,
                    logo_data = EXCLUDED.logo_data,
                    logo_mime_type = EXCLUDED.logo_mime_type,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, company_name, street_address, plz, city, country, phone, fax, email, website, tax_id, ceo_name, registry_court, logo_path, logo_data, logo_mime_type, updated_at::text as updated_at",
                &[
                    &settings.company_name,
                    &settings.street_address,
                    &settings.plz,
                    &settings.city,
                    &settings.country,
                    &settings.phone,
                    &settings.fax,
                    &settings.email,
                    &settings.website,
                    &settings.tax_id,
                    &settings.ceo_name,
                    &settings.registry_court,
                    &settings.logo_path,
                    &settings.logo_data,
                    &settings.logo_mime_type,
                ],
            )
            .await?;

        Ok(CompanySettings::from(row))
    }
}
