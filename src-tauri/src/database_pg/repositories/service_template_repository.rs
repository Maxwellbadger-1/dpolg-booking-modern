use crate::database_pg::{DbPool, DbResult, ServiceTemplate};

pub struct ServiceTemplateRepository;

impl ServiceTemplateRepository {
    /// Get all service templates
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<ServiceTemplate>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, service_name, price_type, original_value, applies_to, is_active, created_at
                 FROM service_templates
                 ORDER BY service_name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(ServiceTemplate::from).collect())
    }

    /// Get service template by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<ServiceTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, service_name, price_type, original_value, applies_to, is_active, created_at
                 FROM service_templates
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Service template with ID {} not found", id)))?;

        Ok(ServiceTemplate::from(row))
    }

    /// Get active service templates
    pub async fn get_active(pool: &DbPool) -> DbResult<Vec<ServiceTemplate>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, service_name, price_type, original_value, applies_to, is_active, created_at
                 FROM service_templates
                 WHERE is_active = TRUE
                 ORDER BY service_name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(ServiceTemplate::from).collect())
    }

    /// Create new service template
    pub async fn create(
        pool: &DbPool,
        service_name: String,
        price_type: String,
        original_value: f64,
        applies_to: String,
    ) -> DbResult<ServiceTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO service_templates (
                    service_name, price_type, original_value, applies_to, is_active, created_at
                 ) VALUES ($1, $2, $3, $4, TRUE, CURRENT_TIMESTAMP)
                 RETURNING id, service_name, price_type, original_value, applies_to, is_active, created_at",
                &[&service_name, &price_type, &original_value, &applies_to],
            )
            .await?;

        Ok(ServiceTemplate::from(row))
    }

    /// Update service template
    pub async fn update(
        pool: &DbPool,
        id: i32,
        service_name: String,
        price_type: String,
        original_value: f64,
        applies_to: String,
        is_active: bool,
    ) -> DbResult<ServiceTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE service_templates SET
                    service_name = $2, price_type = $3, original_value = $4,
                    applies_to = $5, is_active = $6
                 WHERE id = $1
                 RETURNING id, service_name, price_type, original_value, applies_to, is_active, created_at",
                &[&id, &service_name, &price_type, &original_value, &applies_to, &is_active],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Service template with ID {} not found", id)))?;

        Ok(ServiceTemplate::from(row))
    }

    /// Toggle service template active status
    pub async fn toggle_active(pool: &DbPool, id: i32) -> DbResult<ServiceTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE service_templates SET
                    is_active = NOT is_active
                 WHERE id = $1
                 RETURNING id, service_name, price_type, original_value, applies_to, is_active, created_at",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Service template with ID {} not found", id)))?;

        Ok(ServiceTemplate::from(row))
    }

    /// Delete service template
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM service_templates WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Service template with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Count service templates
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM service_templates", &[])
            .await?;

        Ok(row.get("count"))
    }

    /// Count active service templates
    pub async fn count_active(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT COUNT(*) as count FROM service_templates WHERE is_active = TRUE",
                &[],
            )
            .await?;

        Ok(row.get("count"))
    }
}
