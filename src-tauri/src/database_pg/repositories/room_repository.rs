use crate::database_pg::{DbPool, DbError, DbResult, models::Room};

/// Repository for Room operations (Best Practice 2025)
///
/// Benefits:
/// - Encapsulates all room-related database operations
/// - Type-safe queries
/// - Reusable across the application
/// - Easy to test (can be mocked)
pub struct RoomRepository;

impl RoomRepository {
    /// Get all rooms from database
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<Room>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, name, gebaeude_typ, capacity,
                        nebensaison_preis, hauptsaison_preis, endreinigung, ort,
                        schluesselcode, street_address, postal_code, city, notizen
                 FROM rooms
                 ORDER BY name",
                &[],
            )
            .await?;

        let rooms = rows.into_iter().map(Room::from).collect();

        Ok(rooms)
    }

    /// Get room by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<Room> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, name, gebaeude_typ, capacity,
                        nebensaison_preis, hauptsaison_preis, endreinigung, ort,
                        schluesselcode, street_address, postal_code, city, notizen
                 FROM rooms
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("no rows") {
                    DbError::NotFound(format!("Room with ID {} not found", id))
                } else {
                    DbError::from(e)
                }
            })?;

        Ok(Room::from(row))
    }

    /// Create new room
    pub async fn create(
        pool: &DbPool,
        name: String,
        gebaeude_typ: String,
        capacity: i32,
        ort: String,
        nebensaison_preis: Option<f64>,
        hauptsaison_preis: Option<f64>,
        endreinigung: Option<f64>,
        schluesselcode: Option<String>,
        street_address: Option<String>,
        postal_code: Option<String>,
        city: Option<String>,
        notizen: Option<String>,
    ) -> DbResult<Room> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO rooms (
                    name, gebaeude_typ, capacity,
                    nebensaison_preis, hauptsaison_preis, endreinigung, ort,
                    schluesselcode, street_address, postal_code, city, notizen
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING id, name, gebaeude_typ, capacity,
                         nebensaison_preis, hauptsaison_preis, endreinigung, ort,
                         schluesselcode, street_address, postal_code, city, notizen",
                &[
                    &name,
                    &gebaeude_typ,
                    &capacity,
                    &nebensaison_preis,
                    &hauptsaison_preis,
                    &endreinigung,
                    &ort,
                    &schluesselcode,
                    &street_address,
                    &postal_code,
                    &city,
                    &notizen,
                ],
            )
            .await?;

        Ok(Room::from(row))
    }

    /// Update existing room
    pub async fn update(
        pool: &DbPool,
        id: i32,
        name: String,
        gebaeude_typ: String,
        capacity: i32,
        ort: String,
        nebensaison_preis: Option<f64>,
        hauptsaison_preis: Option<f64>,
        endreinigung: Option<f64>,
        schluesselcode: Option<String>,
        street_address: Option<String>,
        postal_code: Option<String>,
        city: Option<String>,
        notizen: Option<String>,
    ) -> DbResult<Room> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE rooms SET
                    name = $2,
                    gebaeude_typ = $3,
                    capacity = $4,
                    nebensaison_preis = $5,
                    hauptsaison_preis = $6,
                    endreinigung = $7,
                    ort = $8,
                    schluesselcode = $9,
                    street_address = $10,
                    postal_code = $11,
                    city = $12,
                    notizen = $13
                 WHERE id = $1
                 RETURNING id, name, gebaeude_typ, capacity,
                          nebensaison_preis, hauptsaison_preis, endreinigung, ort,
                          schluesselcode, street_address, postal_code, city, notizen",
                &[
                    &id,
                    &name,
                    &gebaeude_typ,
                    &capacity,
                    &nebensaison_preis,
                    &hauptsaison_preis,
                    &endreinigung,
                    &ort,
                    &schluesselcode,
                    &street_address,
                    &postal_code,
                    &city,
                    &notizen,
                ],
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("no rows") {
                    DbError::NotFound(format!("Room with ID {} not found", id))
                } else {
                    DbError::from(e)
                }
            })?;

        Ok(Room::from(row))
    }

    /// Delete room
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM rooms WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!("Room with ID {} not found", id)));
        }

        Ok(())
    }

    /// Search rooms by name or location
    pub async fn search(pool: &DbPool, query: &str) -> DbResult<Vec<Room>> {
        let client = pool.get().await?;

        let search_pattern = format!("%{}%", query);

        let rows = client
            .query(
                "SELECT id, name, gebaeude_typ, capacity,
                        nebensaison_preis, hauptsaison_preis, endreinigung, ort,
                        schluesselcode, street_address, postal_code, city, notizen
                 FROM rooms
                 WHERE name ILIKE $1 OR ort ILIKE $1 OR gebaeude_typ ILIKE $1
                 ORDER BY name",
                &[&search_pattern],
            )
            .await?;

        let rooms = rows.into_iter().map(Room::from).collect();

        Ok(rooms)
    }
}
