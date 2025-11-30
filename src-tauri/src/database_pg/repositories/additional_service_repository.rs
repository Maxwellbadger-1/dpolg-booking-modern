use crate::database_pg::{DbPool, DbResult, AdditionalService};

pub struct AdditionalServiceRepository;

impl AdditionalServiceRepository {
    /// Get all additional services
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<AdditionalService>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT a.id, a.booking_id, a.service_name, a.service_price,
                        a.template_id, a.price_type, a.original_value, a.applies_to,
                        st.emoji
                 FROM additional_services a
                 LEFT JOIN service_templates st ON a.template_id = st.id
                 ORDER BY a.id DESC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(AdditionalService::from).collect())
    }

    /// Get service by ID
    pub async fn get_by_id(pool: &DbPool, id: i64) -> DbResult<AdditionalService> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT a.id, a.booking_id, a.service_name, a.service_price,
                        a.template_id, a.price_type, a.original_value, a.applies_to,
                        st.emoji
                 FROM additional_services a
                 LEFT JOIN service_templates st ON a.template_id = st.id
                 WHERE a.id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Service with ID {} not found", id)))?;

        Ok(AdditionalService::from(row))
    }

    /// Get services by booking ID
    pub async fn get_by_booking(pool: &DbPool, booking_id: i64) -> DbResult<Vec<AdditionalService>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT a.id, a.booking_id, a.service_name, a.service_price,
                        a.template_id, a.price_type, a.original_value, a.applies_to,
                        st.emoji
                 FROM additional_services a
                 LEFT JOIN service_templates st ON a.template_id = st.id
                 WHERE a.booking_id = $1
                 ORDER BY a.id ASC",
                &[&booking_id],
            )
            .await?;

        Ok(rows.into_iter().map(AdditionalService::from).collect())
    }

    /// Create new service
    pub async fn create(
        pool: &DbPool,
        booking_id: i64,
        service_name: String,
        service_price: f32,
        template_id: Option<i32>,
        price_type: String,
        original_value: f32,
        applies_to: String,
    ) -> DbResult<AdditionalService> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "WITH inserted AS (
                    INSERT INTO additional_services (
                        booking_id, service_name, service_price, template_id,
                        price_type, original_value, applies_to
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                    RETURNING *
                 )
                 SELECT i.id, i.booking_id, i.service_name, i.service_price,
                        i.template_id, i.price_type, i.original_value, i.applies_to,
                        st.emoji
                 FROM inserted i
                 LEFT JOIN service_templates st ON i.template_id = st.id",
                &[
                    &booking_id, &service_name, &service_price, &template_id,
                    &price_type, &original_value, &applies_to,
                ],
            )
            .await?;

        Ok(AdditionalService::from(row))
    }

    /// Update existing service
    pub async fn update(
        pool: &DbPool,
        id: i64,
        booking_id: i64,
        service_name: String,
        service_price: f32,
        template_id: Option<i32>,
        price_type: String,
        original_value: f32,
        applies_to: String,
    ) -> DbResult<AdditionalService> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "WITH updated AS (
                    UPDATE additional_services SET
                        booking_id = $2, service_name = $3, service_price = $4,
                        template_id = $5, price_type = $6, original_value = $7,
                        applies_to = $8
                    WHERE id = $1
                    RETURNING *
                 )
                 SELECT u.id, u.booking_id, u.service_name, u.service_price,
                        u.template_id, u.price_type, u.original_value, u.applies_to,
                        st.emoji
                 FROM updated u
                 LEFT JOIN service_templates st ON u.template_id = st.id",
                &[
                    &id, &booking_id, &service_name, &service_price,
                    &template_id, &price_type, &original_value, &applies_to,
                ],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Service with ID {} not found", id)))?;

        Ok(AdditionalService::from(row))
    }

    /// Delete service
    pub async fn delete(pool: &DbPool, id: i64) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM additional_services WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Service with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Calculate total services price for booking
    pub async fn calculate_total_for_booking(pool: &DbPool, booking_id: i64) -> DbResult<f64> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT COALESCE(SUM(service_price), 0) as total
                 FROM additional_services
                 WHERE booking_id = $1",
                &[&booking_id],
            )
            .await?;

        Ok(row.get("total"))
    }

    /// Get service count
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM additional_services", &[])
            .await?;

        Ok(row.get("count"))
    }
}
