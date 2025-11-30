use crate::database_pg::{DbPool, DbResult, PaymentRecipient};

pub struct PaymentRecipientRepository;

impl PaymentRecipientRepository {
    /// Get all payment recipients
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<PaymentRecipient>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, name, company, street, plz, city, country, contact_person, notes, created_at::text as created_at, updated_at::text as updated_at
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
                "SELECT id, name, company, street, plz, city, country, contact_person, notes, created_at::text as created_at, updated_at::text as updated_at
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
                "SELECT id, name, company, street, plz, city, country, contact_person, notes, created_at::text as created_at, updated_at::text as updated_at
                 FROM payment_recipients
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
        company: Option<String>,
        street: Option<String>,
        plz: Option<String>,
        city: Option<String>,
        country: Option<String>,
        contact_person: Option<String>,
        notes: Option<String>,
    ) -> DbResult<PaymentRecipient> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO payment_recipients (
                    name, company, street, plz, city, country, contact_person, notes, created_at, updated_at
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 RETURNING id, name, company, street, plz, city, country, contact_person, notes, created_at::text as created_at, updated_at::text as updated_at",
                &[&name, &company, &street, &plz, &city, &country, &contact_person, &notes],
            )
            .await?;

        Ok(PaymentRecipient::from(row))
    }

    /// Update payment recipient
    pub async fn update(
        pool: &DbPool,
        id: i32,
        name: String,
        company: Option<String>,
        street: Option<String>,
        plz: Option<String>,
        city: Option<String>,
        country: Option<String>,
        contact_person: Option<String>,
        notes: Option<String>,
    ) -> DbResult<PaymentRecipient> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE payment_recipients SET
                    name = $2, company = $3, street = $4, plz = $5, city = $6, country = $7,
                    contact_person = $8, notes = $9, updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1
                 RETURNING id, name, company, street, plz, city, country, contact_person, notes, created_at::text as created_at, updated_at::text as updated_at",
                &[&id, &name, &company, &street, &plz, &city, &country, &contact_person, &notes],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Payment recipient with ID {} not found", id)))?;

        Ok(PaymentRecipient::from(row))
    }

    /// Delete payment recipient (removed is_active toggle - not in current schema)
    /// Note: The payment_recipients table does not have an is_active column anymore

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

}
