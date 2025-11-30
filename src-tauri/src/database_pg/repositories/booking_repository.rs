use crate::database_pg::{DbPool, DbResult, Booking};

pub struct BookingRepository;

impl BookingRepository {
    /// Get all bookings from the database
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<Booking>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                        anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                        anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                        anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                        mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                        ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                        ist_dpolg_mitglied, NULL::double precision as credit_used
                 FROM bookings
                 ORDER BY checkin_date DESC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(Booking::from).collect())
    }

    /// Get a booking by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<Booking> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                        anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                        anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                        anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                        mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                        ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                        ist_dpolg_mitglied, NULL::double precision as credit_used
                 FROM bookings
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Booking with ID {} not found", id)))?;

        Ok(Booking::from(row))
    }

    /// Get a booking with full room and guest details
    pub async fn get_with_details(pool: &DbPool, id: i32) -> DbResult<crate::database_pg::BookingWithDetails> {
        use crate::database_pg::repositories::{RoomRepository, GuestRepository, AdditionalServiceRepository, DiscountRepository};

        // Get booking
        let booking = Self::get_by_id(pool, id).await?;

        // Get room details
        let room = RoomRepository::get_by_id(pool, booking.room_id).await.ok();

        // Get guest details
        let guest = GuestRepository::get_by_id(pool, booking.guest_id).await.ok();

        // Get services for this booking
        let services = AdditionalServiceRepository::get_by_booking(pool, booking.id as i64)
            .await
            .unwrap_or_else(|_| Vec::new());

        // Get discounts for this booking
        let discounts = DiscountRepository::get_by_booking(pool, booking.id as i64)
            .await
            .unwrap_or_else(|_| Vec::new());

        Ok(crate::database_pg::BookingWithDetails {
            booking,
            room,
            guest,
            services,
            discounts,
            room_name: None,
            guest_name: None,
            guest_email: None,
        })
    }

    /// Create a new booking
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &DbPool,
        room_id: i32,
        guest_id: i32,
        reservierungsnummer: String,
        checkin_date: String,
        checkout_date: String,
        anzahl_gaeste: i32,
        status: String,
        gesamtpreis: f64,
        bemerkungen: Option<String>,
        anzahl_begleitpersonen: Option<i32>,
        grundpreis: Option<f64>,
        services_preis: Option<f64>,
        rabatt_preis: Option<f64>,
        anzahl_naechte: Option<i32>,
        bezahlt: Option<bool>,
        bezahlt_am: Option<String>,
        zahlungsmethode: Option<String>,
        mahnung_gesendet_am: Option<String>,
        rechnung_versendet_am: Option<String>,
        rechnung_versendet_an: Option<String>,
        ist_stiftungsfall: Option<bool>,
        payment_recipient_id: Option<i32>,
        putzplan_checkout_date: Option<String>,
        ist_dpolg_mitglied: Option<bool>,
    ) -> DbResult<Booking> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO bookings (
                    room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                    anzahl_gaeste, status, gesamtpreis, bemerkungen,
                    anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                    anzahl_naechte, bezahlt, bezahlt_am, zahlungsmethode,
                    mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                    ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                    ist_dpolg_mitglied, created_at, updated_at
                 ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
                 ) RETURNING id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                             anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                             anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                             anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                             mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                             ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                             ist_dpolg_mitglied",
                &[
                    &room_id, &guest_id, &reservierungsnummer, &checkin_date, &checkout_date,
                    &anzahl_gaeste, &status, &gesamtpreis, &bemerkungen,
                    &anzahl_begleitpersonen, &grundpreis, &services_preis, &rabatt_preis,
                    &anzahl_naechte, &bezahlt, &bezahlt_am, &zahlungsmethode,
                    &mahnung_gesendet_am, &rechnung_versendet_am, &rechnung_versendet_an,
                    &ist_stiftungsfall, &payment_recipient_id, &putzplan_checkout_date,
                    &ist_dpolg_mitglied,
                ],
            )
            .await?;

        Ok(Booking::from(row))
    }

    /// Update an existing booking with Optimistic Locking (2025 Best Practice)
    ///
    /// Uses `expected_updated_at` to detect if another user has modified the record.
    /// Returns ConflictError if the record was changed by another user.
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &DbPool,
        id: i32,
        room_id: i32,
        guest_id: i32,
        reservierungsnummer: String,
        checkin_date: String,
        checkout_date: String,
        anzahl_gaeste: i32,
        status: String,
        gesamtpreis: f64,
        bemerkungen: Option<String>,
        anzahl_begleitpersonen: Option<i32>,
        grundpreis: Option<f64>,
        services_preis: Option<f64>,
        rabatt_preis: Option<f64>,
        anzahl_naechte: Option<i32>,
        bezahlt: Option<bool>,
        bezahlt_am: Option<String>,
        zahlungsmethode: Option<String>,
        mahnung_gesendet_am: Option<String>,
        rechnung_versendet_am: Option<String>,
        rechnung_versendet_an: Option<String>,
        ist_stiftungsfall: Option<bool>,
        payment_recipient_id: Option<i32>,
        putzplan_checkout_date: Option<String>,
        ist_dpolg_mitglied: Option<bool>,
        expected_updated_at: Option<String>,  // ← NEW: Optimistic Locking parameter
    ) -> DbResult<Booking> {
        let client = pool.get().await?;

        // If expected_updated_at is provided, use optimistic locking
        if let Some(expected_ts) = expected_updated_at {
            // Try update with version check
            let rows_affected = client
                .execute(
                    "UPDATE bookings SET
                        room_id = $2, guest_id = $3, reservierungsnummer = $4,
                        checkin_date = $5, checkout_date = $6, anzahl_gaeste = $7,
                        status = $8, gesamtpreis = $9, bemerkungen = $10,
                        anzahl_begleitpersonen = $11, grundpreis = $12, services_preis = $13,
                        rabatt_preis = $14, anzahl_naechte = $15, bezahlt = $16,
                        bezahlt_am = $17, zahlungsmethode = $18, mahnung_gesendet_am = $19,
                        rechnung_versendet_am = $20, rechnung_versendet_an = $21,
                        ist_stiftungsfall = $22, payment_recipient_id = $23,
                        putzplan_checkout_date = $24, ist_dpolg_mitglied = $25,
                        updated_at = CURRENT_TIMESTAMP
                     WHERE id = $1 AND updated_at = $26",  // ← VERSION CHECK!
                    &[
                        &id, &room_id, &guest_id, &reservierungsnummer, &checkin_date, &checkout_date,
                        &anzahl_gaeste, &status, &gesamtpreis, &bemerkungen,
                        &anzahl_begleitpersonen, &grundpreis, &services_preis, &rabatt_preis,
                        &anzahl_naechte, &bezahlt, &bezahlt_am, &zahlungsmethode,
                        &mahnung_gesendet_am, &rechnung_versendet_am, &rechnung_versendet_an,
                        &ist_stiftungsfall, &payment_recipient_id, &putzplan_checkout_date,
                        &ist_dpolg_mitglied, &expected_ts,
                    ],
                )
                .await?;

            // Check if update succeeded (rows_affected should be 1)
            if rows_affected == 0 {
                return Err(crate::database_pg::DbError::ConflictError(format!(
                    "Booking {} was modified by another user. Please refresh and try again.",
                    id
                )));
            }

            // Return updated booking
            Self::get_by_id(pool, id).await
        } else {
            // Fallback: Update without optimistic locking (for backward compatibility)
            let row = client
                .query_one(
                    "UPDATE bookings SET
                        room_id = $2, guest_id = $3, reservierungsnummer = $4,
                        checkin_date = $5, checkout_date = $6, anzahl_gaeste = $7,
                        status = $8, gesamtpreis = $9, bemerkungen = $10,
                        anzahl_begleitpersonen = $11, grundpreis = $12, services_preis = $13,
                        rabatt_preis = $14, anzahl_naechte = $15, bezahlt = $16,
                        bezahlt_am = $17, zahlungsmethode = $18, mahnung_gesendet_am = $19,
                        rechnung_versendet_am = $20, rechnung_versendet_an = $21,
                        ist_stiftungsfall = $22, payment_recipient_id = $23,
                        putzplan_checkout_date = $24, ist_dpolg_mitglied = $25,
                        updated_at = CURRENT_TIMESTAMP
                     WHERE id = $1
                     RETURNING id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                               anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                               anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                               anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                               mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                               ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                               ist_dpolg_mitglied",
                    &[
                        &id, &room_id, &guest_id, &reservierungsnummer, &checkin_date, &checkout_date,
                        &anzahl_gaeste, &status, &gesamtpreis, &bemerkungen,
                        &anzahl_begleitpersonen, &grundpreis, &services_preis, &rabatt_preis,
                        &anzahl_naechte, &bezahlt, &bezahlt_am, &zahlungsmethode,
                        &mahnung_gesendet_am, &rechnung_versendet_am, &rechnung_versendet_an,
                        &ist_stiftungsfall, &payment_recipient_id, &putzplan_checkout_date,
                        &ist_dpolg_mitglied,
                    ],
                )
                .await
                .map_err(|_| crate::database_pg::DbError::NotFound(format!("Booking with ID {} not found", id)))?;

            Ok(Booking::from(row))
        }
    }

    /// Delete a booking
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM bookings WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Booking with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Get bookings by room ID
    pub async fn get_by_room(pool: &DbPool, room_id: i32) -> DbResult<Vec<Booking>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                        anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                        anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                        anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                        mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                        ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                        ist_dpolg_mitglied
                 FROM bookings
                 WHERE room_id = $1
                 ORDER BY checkin_date DESC",
                &[&room_id],
            )
            .await?;

        Ok(rows.into_iter().map(Booking::from).collect())
    }

    /// Get bookings by guest ID
    pub async fn get_by_guest(pool: &DbPool, guest_id: i32) -> DbResult<Vec<Booking>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                        anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                        anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                        anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                        mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                        ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                        ist_dpolg_mitglied
                 FROM bookings
                 WHERE guest_id = $1
                 ORDER BY checkin_date DESC",
                &[&guest_id],
            )
            .await?;

        Ok(rows.into_iter().map(Booking::from).collect())
    }

    /// Get bookings by status
    pub async fn get_by_status(pool: &DbPool, status: String) -> DbResult<Vec<Booking>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                        anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                        anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                        anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                        mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                        ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                        ist_dpolg_mitglied
                 FROM bookings
                 WHERE status = $1
                 ORDER BY checkin_date DESC",
                &[&status],
            )
            .await?;

        Ok(rows.into_iter().map(Booking::from).collect())
    }

    /// Get bookings by date range
    pub async fn get_by_date_range(
        pool: &DbPool,
        start_date: String,
        end_date: String,
    ) -> DbResult<Vec<Booking>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                        anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                        anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                        anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                        mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                        ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                        ist_dpolg_mitglied
                 FROM bookings
                 WHERE checkin_date >= $1 AND checkout_date <= $2
                 ORDER BY checkin_date ASC",
                &[&start_date, &end_date],
            )
            .await?;

        Ok(rows.into_iter().map(Booking::from).collect())
    }

    /// Get unpaid bookings
    pub async fn get_unpaid(pool: &DbPool) -> DbResult<Vec<Booking>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                        anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                        anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                        anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                        mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                        ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                        ist_dpolg_mitglied
                 FROM bookings
                 WHERE bezahlt = FALSE OR bezahlt IS NULL
                 ORDER BY checkin_date DESC",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(Booking::from).collect())
    }

    /// Search bookings by reservation number
    pub async fn search(pool: &DbPool, query: String) -> DbResult<Vec<Booking>> {
        let client = pool.get().await?;

        let search_pattern = format!("%{}%", query.to_uppercase());

        let rows = client
            .query(
                "SELECT id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date,
                        anzahl_gaeste, status, gesamtpreis, bemerkungen, created_at::text as created_at,
                        anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis,
                        anzahl_naechte, updated_at::text as updated_at, bezahlt, bezahlt_am, zahlungsmethode,
                        mahnung_gesendet_am, rechnung_versendet_am, rechnung_versendet_an,
                        ist_stiftungsfall, payment_recipient_id, putzplan_checkout_date,
                        ist_dpolg_mitglied
                 FROM bookings
                 WHERE UPPER(reservierungsnummer) LIKE $1
                 ORDER BY checkin_date DESC",
                &[&search_pattern],
            )
            .await?;

        Ok(rows.into_iter().map(Booking::from).collect())
    }

    /// Get booking count
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM bookings", &[])
            .await?;

        Ok(row.get("count"))
    }
}
