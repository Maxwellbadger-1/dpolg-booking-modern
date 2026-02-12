use crate::database_pg::{DbPool, DbResult, Discount};

pub struct DiscountRepository;

impl DiscountRepository {
    /// Get all discounts
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<Discount>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT d.id, d.booking_id, d.discount_name, d.discount_type, d.discount_value,
                        d.calculated_amount,
                        dt.emoji
                 FROM discounts d
                 LEFT JOIN discount_templates dt ON d.discount_name = dt.name
                 ORDER BY d.id DESC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(Discount::from).collect())
    }

    /// Get discount by ID
    pub async fn get_by_id(pool: &DbPool, id: i64) -> DbResult<Discount> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT d.id, d.booking_id, d.discount_name, d.discount_type, d.discount_value,
                        d.calculated_amount,
                        dt.emoji
                 FROM discounts d
                 LEFT JOIN discount_templates dt ON d.discount_name = dt.name
                 WHERE d.id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Discount with ID {} not found", id)))?;

        Ok(Discount::from(row))
    }

    /// Get discounts by booking ID
    pub async fn get_by_booking(pool: &DbPool, booking_id: i64) -> DbResult<Vec<Discount>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT d.id, d.booking_id, d.discount_name, d.discount_type, d.discount_value,
                        d.calculated_amount,
                        dt.emoji
                 FROM discounts d
                 LEFT JOIN discount_templates dt ON d.discount_name = dt.name
                 WHERE d.booking_id = $1
                 ORDER BY d.id ASC",
                &[&booking_id],
            )
            .await?;

        Ok(rows.into_iter().map(Discount::from).collect())
    }

    /// Create new discount
    pub async fn create(
        pool: &DbPool,
        booking_id: i64,
        discount_name: String,
        discount_type: String,
        discount_value: f64,
        calculated_amount: Option<f64>,
    ) -> DbResult<Discount> {
        // Bug 7 fix: Validate discount value
        if discount_value < 0.0 {
            return Err(crate::database_pg::DbError::ValidationError(
                "Rabattwert darf nicht negativ sein".to_string()
            ));
        }
        if discount_type == "percent" && discount_value > 100.0 {
            return Err(crate::database_pg::DbError::ValidationError(
                "Prozent-Rabatt darf nicht über 100% sein".to_string()
            ));
        }

        let client = pool.get().await?;

        let row = client
            .query_one(
                "WITH inserted AS (
                    INSERT INTO discounts (
                        booking_id, discount_name, discount_type, discount_value, calculated_amount
                    ) VALUES ($1, $2, $3, $4, $5)
                    RETURNING *
                 )
                 SELECT i.id, i.booking_id, i.discount_name, i.discount_type, i.discount_value,
                        i.calculated_amount,
                        dt.emoji
                 FROM inserted i
                 LEFT JOIN discount_templates dt ON i.discount_name = dt.name",
                &[&booking_id, &discount_name, &discount_type, &discount_value, &calculated_amount],
            )
            .await?;

        Ok(Discount::from(row))
    }

    /// Update existing discount
    pub async fn update(
        pool: &DbPool,
        id: i64,
        booking_id: i64,
        discount_name: String,
        discount_type: String,
        discount_value: f64,
        calculated_amount: Option<f64>,
    ) -> DbResult<Discount> {
        // Bug 7 fix: Validate discount value
        if discount_value < 0.0 {
            return Err(crate::database_pg::DbError::ValidationError(
                "Rabattwert darf nicht negativ sein".to_string()
            ));
        }
        if discount_type == "percent" && discount_value > 100.0 {
            return Err(crate::database_pg::DbError::ValidationError(
                "Prozent-Rabatt darf nicht über 100% sein".to_string()
            ));
        }

        let client = pool.get().await?;

        let row = client
            .query_one(
                "WITH updated AS (
                    UPDATE discounts SET
                        booking_id = $2, discount_name = $3, discount_type = $4,
                        discount_value = $5, calculated_amount = $6
                    WHERE id = $1
                    RETURNING *
                 )
                 SELECT u.id, u.booking_id, u.discount_name, u.discount_type, u.discount_value,
                        u.calculated_amount,
                        dt.emoji
                 FROM updated u
                 LEFT JOIN discount_templates dt ON u.discount_name = dt.name",
                &[&id, &booking_id, &discount_name, &discount_type, &discount_value, &calculated_amount],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Discount with ID {} not found", id)))?;

        Ok(Discount::from(row))
    }

    /// Delete discount
    pub async fn delete(pool: &DbPool, id: i64) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM discounts WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Discount with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Calculate total discount value for booking
    pub async fn calculate_total_for_booking(pool: &DbPool, booking_id: i64) -> DbResult<f64> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT COALESCE(SUM(discount_value), 0) as total
                 FROM discounts
                 WHERE booking_id = $1",
                &[&booking_id],
            )
            .await?;

        Ok(row.get("total"))
    }

    /// Get discount count
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM discounts", &[])
            .await?;

        Ok(row.get("count"))
    }

    /// Update calculated_amount for a discount (used during price recalculation)
    pub async fn update_calculated_amount(
        pool: &DbPool,
        discount_id: i64,
        calculated_amount: f64,
    ) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute(
                "UPDATE discounts SET calculated_amount = $1 WHERE id = $2",
                &[&calculated_amount, &discount_id],
            )
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Discount with ID {} not found",
                discount_id
            )));
        }

        Ok(())
    }

    /// Run migration to add calculated_amount column
    pub async fn run_migration(pool: &DbPool) -> DbResult<String> {
        let client = pool.get().await?;

        client
            .execute(
                "ALTER TABLE discounts ADD COLUMN IF NOT EXISTS calculated_amount DOUBLE PRECISION",
                &[],
            )
            .await
            .map_err(|e| crate::database_pg::DbError::Other(format!("Migration failed: {}", e)))?;

        Ok("Discount calculated_amount migration completed".to_string())
    }
}
