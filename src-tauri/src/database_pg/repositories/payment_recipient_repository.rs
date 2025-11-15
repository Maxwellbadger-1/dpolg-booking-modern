use crate::database_pg::{DbPool, DbResult, PaymentRecipient};

pub struct PaymentRecipientRepository;

impl PaymentRecipientRepository {
    /// Get all payment recipients
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<PaymentRecipient>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, name, bank_name, iban, bic, is_active, created_at
                 FROM payment_recipients
                 ORDER BY name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(PaymentRecipient::from).collect())
    }

    /// Get payment recipient by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<PaymentRecipient> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, name, bank_name, iban, bic, is_active, created_at
                 FROM payment_recipients
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Payment recipient with ID {} not found", id)))?;

        Ok(PaymentRecipient::from(row))
    }

    /// Get active payment recipients
    pub async fn get_active(pool: &DbPool) -> DbResult<Vec<PaymentRecipient>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, name, bank_name, iban, bic, is_active, created_at
                 FROM payment_recipients
                 WHERE is_active = TRUE
                 ORDER BY name ASC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(PaymentRecipient::from).collect())
    }

    /// Create new payment recipient
    pub async fn create(
        pool: &DbPool,
        name: String,
        bank_name: String,
        iban: String,
        bic: Option<String>,
    ) -> DbResult<PaymentRecipient> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO payment_recipients (
                    name, bank_name, iban, bic, is_active, created_at
                 ) VALUES ($1, $2, $3, $4, TRUE, CURRENT_TIMESTAMP)
                 RETURNING id, name, bank_name, iban, bic, is_active, created_at",
                &[&name, &bank_name, &iban, &bic],
            )
            .await?;

        Ok(PaymentRecipient::from(row))
    }

    /// Update payment recipient
    pub async fn update(
        pool: &DbPool,
        id: i32,
        name: String,
        bank_name: String,
        iban: String,
        bic: Option<String>,
        is_active: bool,
    ) -> DbResult<PaymentRecipient> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE payment_recipients SET
                    name = $2, bank_name = $3, iban = $4, bic = $5, is_active = $6
                 WHERE id = $1
                 RETURNING id, name, bank_name, iban, bic, is_active, created_at",
                &[&id, &name, &bank_name, &iban, &bic, &is_active],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Payment recipient with ID {} not found", id)))?;

        Ok(PaymentRecipient::from(row))
    }

    /// Toggle payment recipient active status
    pub async fn toggle_active(pool: &DbPool, id: i32) -> DbResult<PaymentRecipient> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE payment_recipients SET
                    is_active = NOT is_active
                 WHERE id = $1
                 RETURNING id, name, bank_name, iban, bic, is_active, created_at",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Payment recipient with ID {} not found", id)))?;

        Ok(PaymentRecipient::from(row))
    }

    /// Delete payment recipient
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM payment_recipients WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Payment recipient with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Count payment recipients
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM payment_recipients", &[])
            .await?;

        Ok(row.get("count"))
    }

    /// Count active payment recipients
    pub async fn count_active(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT COUNT(*) as count FROM payment_recipients WHERE is_active = TRUE",
                &[],
            )
            .await?;

        Ok(row.get("count"))
    }
}
