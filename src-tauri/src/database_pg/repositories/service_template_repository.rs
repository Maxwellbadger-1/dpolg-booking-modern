use crate::database_pg::{DbPool, DbResult, ServiceTemplate};

pub struct ServiceTemplateRepository;

impl ServiceTemplateRepository {
    /// Get all service templates
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<ServiceTemplate>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, name, description, price_type, price, applies_to, is_active,
                        emoji, show_in_cleaning_plan, cleaning_plan_position, created_at::text as created_at
                 FROM service_templates
                 ORDER BY name ASC",
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
                "SELECT id, name, description, price_type, price, applies_to, is_active,
                        emoji, show_in_cleaning_plan, cleaning_plan_position, created_at::text as created_at
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
                "SELECT id, name, description, price_type, price, applies_to, is_active,
                        emoji, show_in_cleaning_plan, cleaning_plan_position, created_at::text as created_at
                 FROM service_templates
                 WHERE is_active = TRUE
                 ORDER BY name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(ServiceTemplate::from).collect())
    }

    /// Create new service template
    pub async fn create(
        pool: &DbPool,
        service_name: String,
        description: Option<String>,
        price_type: String,
        price: f64,
        applies_to: String,
        emoji: Option<String>,
        show_in_cleaning_plan: bool,
        cleaning_plan_position: String,
    ) -> DbResult<ServiceTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO service_templates (
                    name, description, price_type, price, applies_to, is_active,
                    emoji, show_in_cleaning_plan, cleaning_plan_position, created_at::text as created_at
                 ) VALUES ($1, $2, $3, $4, $5, TRUE, $6, $7, $8, CURRENT_TIMESTAMP)
                 RETURNING id, name, description, price_type, price, applies_to, is_active,
                           emoji, show_in_cleaning_plan, cleaning_plan_position, created_at::text as created_at",
                &[&service_name, &description, &price_type, &price, &applies_to,
                  &emoji, &show_in_cleaning_plan, &cleaning_plan_position],
            )
            .await?;

        Ok(ServiceTemplate::from(row))
    }

    /// Update service template
    pub async fn update(
        pool: &DbPool,
        id: i32,
        service_name: String,
        description: Option<String>,
        price_type: String,
        price: f64,
        applies_to: String,
        is_active: bool,
        emoji: Option<String>,
        show_in_cleaning_plan: bool,
        cleaning_plan_position: String,
    ) -> DbResult<ServiceTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE service_templates SET
                    name = $2, description = $3, price_type = $4, price = $5,
                    applies_to = $6, is_active = $7, emoji = $8,
                    show_in_cleaning_plan = $9, cleaning_plan_position = $10
                 WHERE id = $1
                 RETURNING id, name, description, price_type, price, applies_to, is_active,
                           emoji, show_in_cleaning_plan, cleaning_plan_position, created_at::text as created_at",
                &[&id, &service_name, &description, &price_type, &price, &applies_to,
                  &is_active, &emoji, &show_in_cleaning_plan, &cleaning_plan_position],
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
                 RETURNING id, name, description, price_type, price, applies_to, is_active,
                           emoji, show_in_cleaning_plan, cleaning_plan_position, created_at::text as created_at",
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
