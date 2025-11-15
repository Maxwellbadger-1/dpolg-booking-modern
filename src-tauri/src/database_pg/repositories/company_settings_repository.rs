use crate::database_pg::{DbPool, DbResult, CompanySettings};

pub struct CompanySettingsRepository;

impl CompanySettingsRepository {
    /// Get company settings (Singleton - should only have 1 record)
    pub async fn get(pool: &DbPool) -> DbResult<CompanySettings> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, company_name, address, phone, email, tax_id, logo_path, updated_at
                 FROM company_settings
                 LIMIT 1",
                &[],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound("Company settings not found".to_string()))?;

        Ok(CompanySettings::from(row))
    }

    /// Update company settings
    pub async fn update(
        pool: &DbPool,
        company_name: String,
        address: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        tax_id: Option<String>,
        logo_path: Option<String>,
    ) -> DbResult<CompanySettings> {
        let client = pool.get().await?;

        // Update first record (or insert if none exists)
        let row = client
            .query_one(
                "INSERT INTO company_settings (id, company_name, address, phone, email, tax_id, logo_path, updated_at)
                 VALUES (1, $1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
                 ON CONFLICT (id) DO UPDATE SET
                    company_name = EXCLUDED.company_name,
                    address = EXCLUDED.address,
                    phone = EXCLUDED.phone,
                    email = EXCLUDED.email,
                    tax_id = EXCLUDED.tax_id,
                    logo_path = EXCLUDED.logo_path,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id, company_name, address, phone, email, tax_id, logo_path, updated_at",
                &[&company_name, &address, &phone, &email, &tax_id, &logo_path],
            )
            .await?;

        Ok(CompanySettings::from(row))
    }
}
