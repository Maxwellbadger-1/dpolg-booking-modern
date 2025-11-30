use crate::database_pg::{DbPool, DbResult, DiscountTemplate};

pub struct DiscountTemplateRepository;

impl DiscountTemplateRepository {
    /// Get all discount templates
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<DiscountTemplate>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, name, discount_type, discount_value, is_active, emoji, created_at::text as created_at
                 FROM discount_templates
                 ORDER BY name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(DiscountTemplate::from).collect())
    }

    /// Get discount template by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<DiscountTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, name, discount_type, discount_value, is_active, emoji, created_at::text as created_at
                 FROM discount_templates
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Discount template with ID {} not found", id)))?;

        Ok(DiscountTemplate::from(row))
    }

    /// Get active discount templates
    pub async fn get_active(pool: &DbPool) -> DbResult<Vec<DiscountTemplate>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, name, discount_type, discount_value, is_active, emoji, created_at::text as created_at
                 FROM discount_templates
                 WHERE is_active = TRUE
                 ORDER BY name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(DiscountTemplate::from).collect())
    }

    /// Create new discount template
    pub async fn create(
        pool: &DbPool,
        discount_name: String,
        discount_type: String,
        discount_value: f64,
        emoji: Option<String>,
    ) -> DbResult<DiscountTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO discount_templates (
                    name, discount_type, discount_value, is_active, emoji, created_at::text as created_at
                 ) VALUES ($1, $2, $3, TRUE, $4, CURRENT_TIMESTAMP)
                 RETURNING id, name, discount_type, discount_value, is_active, emoji, created_at::text as created_at",
                &[&discount_name, &discount_type, &discount_value, &emoji],
            )
            .await?;

        Ok(DiscountTemplate::from(row))
    }

    /// Update discount template
    pub async fn update(
        pool: &DbPool,
        id: i32,
        discount_name: String,
        discount_type: String,
        discount_value: f64,
        is_active: bool,
        emoji: Option<String>,
    ) -> DbResult<DiscountTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE discount_templates SET
                    name = $2, discount_type = $3, discount_value = $4, is_active = $5, emoji = $6
                 WHERE id = $1
                 RETURNING id, name, discount_type, discount_value, is_active, emoji, created_at::text as created_at",
                &[&id, &discount_name, &discount_type, &discount_value, &is_active, &emoji],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Discount template with ID {} not found", id)))?;

        Ok(DiscountTemplate::from(row))
    }

    /// Toggle discount template active status
    pub async fn toggle_active(pool: &DbPool, id: i32) -> DbResult<DiscountTemplate> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE discount_templates SET
                    is_active = NOT is_active
                 WHERE id = $1
                 RETURNING id, name, discount_type, discount_value, is_active, emoji, created_at::text as created_at",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Discount template with ID {} not found", id)))?;

        Ok(DiscountTemplate::from(row))
    }

    /// Delete discount template
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM discount_templates WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Discount template with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Count discount templates
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM discount_templates", &[])
            .await?;

        Ok(row.get("count"))
    }

    /// Count active discount templates
    pub async fn count_active(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT COUNT(*) as count FROM discount_templates WHERE is_active = TRUE",
                &[],
            )
            .await?;

        Ok(row.get("count"))
    }
}
