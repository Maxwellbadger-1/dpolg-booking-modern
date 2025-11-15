use crate::database_pg::{DbPool, DbResult, AccompanyingGuest};

pub struct AccompanyingGuestRepository;

impl AccompanyingGuestRepository {
    /// Get all accompanying guests
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<AccompanyingGuest>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, first_name, last_name, birth_date, created_at
                 FROM accompanying_guests
                 ORDER BY created_at DESC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(AccompanyingGuest::from).collect())
    }

    /// Get accompanying guest by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<AccompanyingGuest> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, booking_id, first_name, last_name, birth_date, created_at
                 FROM accompanying_guests
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Accompanying guest with ID {} not found", id)))?;

        Ok(AccompanyingGuest::from(row))
    }

    /// Get accompanying guests by booking ID
    pub async fn get_by_booking(pool: &DbPool, booking_id: i32) -> DbResult<Vec<AccompanyingGuest>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, booking_id, first_name, last_name, birth_date, created_at
                 FROM accompanying_guests
                 WHERE booking_id = $1
                 ORDER BY last_name, first_name",
                &[&booking_id],
            )
            .await?;

        Ok(rows.into_iter().map(AccompanyingGuest::from).collect())
    }

    /// Create new accompanying guest
    pub async fn create(
        pool: &DbPool,
        booking_id: i32,
        first_name: String,
        last_name: String,
        birth_date: Option<String>,
    ) -> DbResult<AccompanyingGuest> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO accompanying_guests (
                    booking_id, first_name, last_name, birth_date, created_at
                 ) VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
                 RETURNING id, booking_id, first_name, last_name, birth_date, created_at",
                &[&booking_id, &first_name, &last_name, &birth_date],
            )
            .await?;

        Ok(AccompanyingGuest::from(row))
    }

    /// Update accompanying guest
    pub async fn update(
        pool: &DbPool,
        id: i32,
        first_name: String,
        last_name: String,
        birth_date: Option<String>,
    ) -> DbResult<AccompanyingGuest> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE accompanying_guests SET
                    first_name = $2, last_name = $3, birth_date = $4
                 WHERE id = $1
                 RETURNING id, booking_id, first_name, last_name, birth_date, created_at",
                &[&id, &first_name, &last_name, &birth_date],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Accompanying guest with ID {} not found", id)))?;

        Ok(AccompanyingGuest::from(row))
    }

    /// Delete accompanying guest
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM accompanying_guests WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Accompanying guest with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Count accompanying guests
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM accompanying_guests", &[])
            .await?;

        Ok(row.get("count"))
    }

    /// Count accompanying guests for a booking
    pub async fn count_for_booking(pool: &DbPool, booking_id: i32) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT COUNT(*) as count FROM accompanying_guests WHERE booking_id = $1",
                &[&booking_id],
            )
            .await?;

        Ok(row.get("count"))
    }
}
