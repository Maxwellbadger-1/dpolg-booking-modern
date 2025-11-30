use crate::database_pg::{DbPool, DbResult, DbError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GuestCreditTransaction {
    pub id: i32,
    pub guest_id: i32,
    pub booking_id: Option<i32>,
    pub amount: f64,
    pub transaction_type: String, // 'credit', 'debit', 'refund'
    pub description: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
}

pub struct GuestCreditRepository;

impl GuestCreditRepository {
    /// Get credit balance for a guest
    pub async fn get_balance(pool: &DbPool, guest_id: i32) -> DbResult<f64> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT get_guest_credit_balance($1)::text as balance",
                &[&guest_id],
            )
            .await?;

        let balance: String = row.get("balance");
        Ok(balance.parse().unwrap_or(0.0))
    }

    /// Get all transactions for a guest
    pub async fn get_transactions_by_guest(
        pool: &DbPool,
        guest_id: i32,
    ) -> DbResult<Vec<GuestCreditTransaction>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, guest_id, booking_id, amount::numeric::text as amount,
                        transaction_type, description, created_at::text, created_by
                 FROM guest_credit_transactions
                 WHERE guest_id = $1
                 ORDER BY created_at DESC",
                &[&guest_id],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                GuestCreditTransaction {
                    id: row.get("id"),
                    guest_id: row.get("guest_id"),
                    booking_id: row.try_get("booking_id").ok().flatten(),
                    amount: row.get::<_, String>("amount").parse().unwrap_or(0.0),
                    transaction_type: row.get("transaction_type"),
                    description: row.try_get("description").ok().flatten(),
                    created_at: row.get("created_at"),
                    created_by: row.try_get("created_by").ok().flatten(),
                }
            })
            .collect())
    }

    /// Get transactions for a booking
    pub async fn get_transactions_by_booking(
        pool: &DbPool,
        booking_id: i32,
    ) -> DbResult<Vec<GuestCreditTransaction>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, guest_id, booking_id, amount::numeric::text as amount,
                        transaction_type, description, created_at::text, created_by
                 FROM guest_credit_transactions
                 WHERE booking_id = $1
                 ORDER BY created_at DESC",
                &[&booking_id],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                GuestCreditTransaction {
                    id: row.get("id"),
                    guest_id: row.get("guest_id"),
                    booking_id: row.try_get("booking_id").ok().flatten(),
                    amount: row.get::<_, String>("amount").parse().unwrap_or(0.0),
                    transaction_type: row.get("transaction_type"),
                    description: row.try_get("description").ok().flatten(),
                    created_at: row.get("created_at"),
                    created_by: row.try_get("created_by").ok().flatten(),
                }
            })
            .collect())
    }

    /// Add credit to a guest
    pub async fn add_credit(
        pool: &DbPool,
        guest_id: i32,
        amount: f64,
        description: Option<String>,
        created_by: Option<String>,
    ) -> DbResult<GuestCreditTransaction> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO guest_credit_transactions (guest_id, amount, transaction_type, description, created_by)
                 VALUES ($1, $2, 'credit', $3, $4)
                 RETURNING id, guest_id, booking_id, amount::numeric::text as amount,
                           transaction_type, description, created_at::text, created_by",
                &[&guest_id, &amount, &description, &created_by],
            )
            .await?;

        Ok(GuestCreditTransaction {
            id: row.get("id"),
            guest_id: row.get("guest_id"),
            booking_id: row.try_get("booking_id").ok().flatten(),
            amount: row.get::<_, String>("amount").parse().unwrap_or(0.0),
            transaction_type: row.get("transaction_type"),
            description: row.try_get("description").ok().flatten(),
            created_at: row.get("created_at"),
            created_by: row.try_get("created_by").ok().flatten(),
        })
    }

    /// Use credit for a booking
    pub async fn use_credit_for_booking(
        pool: &DbPool,
        guest_id: i32,
        booking_id: i32,
        amount: f64,
        description: Option<String>,
        created_by: Option<String>,
    ) -> DbResult<GuestCreditTransaction> {
        let client = pool.get().await?;

        // This will trigger the validation function that checks for sufficient balance
        let row = client
            .query_one(
                "INSERT INTO guest_credit_transactions (guest_id, booking_id, amount, transaction_type, description, created_by)
                 VALUES ($1, $2, $3, 'debit', $4, $5)
                 RETURNING id, guest_id, booking_id, amount::numeric::text as amount,
                           transaction_type, description, created_at::text, created_by",
                &[&guest_id, &booking_id, &amount, &description, &created_by],
            )
            .await
            .map_err(|e| {
                // Convert insufficient balance error to our DbError type
                if e.to_string().contains("Insufficient credit balance") {
                    DbError::ValidationError(e.to_string())
                } else {
                    DbError::from(e)
                }
            })?;

        Ok(GuestCreditTransaction {
            id: row.get("id"),
            guest_id: row.get("guest_id"),
            booking_id: row.try_get("booking_id").ok().flatten(),
            amount: row.get::<_, String>("amount").parse().unwrap_or(0.0),
            transaction_type: row.get("transaction_type"),
            description: row.try_get("description").ok().flatten(),
            created_at: row.get("created_at"),
            created_by: row.try_get("created_by").ok().flatten(),
        })
    }

    /// Refund credit from a booking
    pub async fn refund_credit(
        pool: &DbPool,
        guest_id: i32,
        booking_id: Option<i32>,
        amount: f64,
        description: Option<String>,
        created_by: Option<String>,
    ) -> DbResult<GuestCreditTransaction> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO guest_credit_transactions (guest_id, booking_id, amount, transaction_type, description, created_by)
                 VALUES ($1, $2, $3, 'refund', $4, $5)
                 RETURNING id, guest_id, booking_id, amount::numeric::text as amount,
                           transaction_type, description, created_at::text, created_by",
                &[&guest_id, &booking_id, &amount, &description, &created_by],
            )
            .await?;

        Ok(GuestCreditTransaction {
            id: row.get("id"),
            guest_id: row.get("guest_id"),
            booking_id: row.try_get("booking_id").ok().flatten(),
            amount: row.get::<_, String>("amount").parse().unwrap_or(0.0),
            transaction_type: row.get("transaction_type"),
            description: row.try_get("description").ok().flatten(),
            created_at: row.get("created_at"),
            created_by: row.try_get("created_by").ok().flatten(),
        })
    }

    /// Delete a transaction (admin only - use with caution)
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute(
                "DELETE FROM guest_credit_transactions WHERE id = $1",
                &[&id],
            )
            .await?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!(
                "Transaction with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Get total credit used for a booking
    pub async fn get_booking_credit_usage(pool: &DbPool, booking_id: i32) -> DbResult<f64> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT get_booking_credit_usage($1)::text as total",
                &[&booking_id],
            )
            .await?;

        let total: String = row.get("total");
        Ok(total.parse().unwrap_or(0.0))
    }

    /// Get statistics for all guest credits
    pub async fn get_statistics(pool: &DbPool) -> DbResult<GuestCreditStatistics> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT
                    COUNT(DISTINCT guest_id) as guests_with_credit,
                    COALESCE(SUM(CASE WHEN transaction_type = 'credit' THEN amount ELSE 0 END)::numeric::text, '0') as total_credits,
                    COALESCE(SUM(CASE WHEN transaction_type = 'debit' THEN amount ELSE 0 END)::numeric::text, '0') as total_debits,
                    COALESCE(SUM(CASE WHEN transaction_type = 'refund' THEN amount ELSE 0 END)::numeric::text, '0') as total_refunds,
                    COUNT(*) as total_transactions
                 FROM guest_credit_transactions",
                &[],
            )
            .await?;

        Ok(GuestCreditStatistics {
            guests_with_credit: row.get::<_, i64>("guests_with_credit") as i32,
            total_credits: row.get::<_, String>("total_credits").parse().unwrap_or(0.0),
            total_debits: row.get::<_, String>("total_debits").parse().unwrap_or(0.0),
            total_refunds: row.get::<_, String>("total_refunds").parse().unwrap_or(0.0),
            total_transactions: row.get::<_, i64>("total_transactions") as i32,
        })
    }

    /// Run migration to create tables and functions
    pub async fn run_migration(pool: &DbPool) -> DbResult<String> {
        let client = pool.get().await?;

        let migration_sql = include_str!("../../../../migrations/001_guest_credit_system.sql");

        // Execute migration
        client
            .batch_execute(migration_sql)
            .await
            .map_err(|e| DbError::QueryError(format!("Migration failed: {}", e)))?;

        // Fix: Add missing columns if table already exists with old structure
        let fix_sql = r#"
            -- Add description column if missing
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_name = 'guest_credit_transactions'
                    AND column_name = 'description'
                ) THEN
                    ALTER TABLE guest_credit_transactions ADD COLUMN description TEXT;
                    RAISE NOTICE '✅ Added description column';
                END IF;
            END $$;

            -- Add created_by column if missing
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_name = 'guest_credit_transactions'
                    AND column_name = 'created_by'
                ) THEN
                    ALTER TABLE guest_credit_transactions ADD COLUMN created_by VARCHAR(100);
                    RAISE NOTICE '✅ Added created_by column';
                END IF;
            END $$;
        "#;

        client
            .batch_execute(fix_sql)
            .await
            .map_err(|e| DbError::QueryError(format!("Column fix failed: {}", e)))?;

        Ok("Guest credit system migration completed successfully".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestCreditStatistics {
    pub guests_with_credit: i32,
    pub total_credits: f64,
    pub total_debits: f64,
    pub total_refunds: f64,
    pub total_transactions: i32,
}
