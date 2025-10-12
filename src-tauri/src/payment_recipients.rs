use crate::database::{get_db_path, PaymentRecipient};
use rusqlite::{Connection, Result};

/// Alle Rechnungsempfänger abrufen
#[tauri::command]
pub fn get_payment_recipients() -> Result<Vec<PaymentRecipient>, String> {
    let conn = Connection::open(get_db_path()).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, company, street, plz, city, country, contact_person, notes, created_at, updated_at
             FROM payment_recipients
             ORDER BY name ASC",
        )
        .map_err(|e| e.to_string())?;

    let recipients = stmt
        .query_map([], |row| {
            Ok(PaymentRecipient {
                id: row.get(0)?,
                name: row.get(1)?,
                company: row.get(2)?,
                street: row.get(3)?,
                plz: row.get(4)?,
                city: row.get(5)?,
                country: row.get(6)?,
                contact_person: row.get(7)?,
                notes: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(recipients)
}

/// Einen spezifischen Rechnungsempfänger abrufen
#[tauri::command]
pub fn get_payment_recipient(id: i64) -> Result<Option<PaymentRecipient>, String> {
    let conn = Connection::open(get_db_path()).map_err(|e| e.to_string())?;

    let result = conn
        .query_row(
            "SELECT id, name, company, street, plz, city, country, contact_person, notes, created_at, updated_at
             FROM payment_recipients WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(PaymentRecipient {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    company: row.get(2)?,
                    street: row.get(3)?,
                    plz: row.get(4)?,
                    city: row.get(5)?,
                    country: row.get(6)?,
                    contact_person: row.get(7)?,
                    notes: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        );

    match result {
        Ok(recipient) => Ok(Some(recipient)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Neuen Rechnungsempfänger erstellen
#[tauri::command]
pub fn create_payment_recipient(
    name: String,
    company: Option<String>,
    street: Option<String>,
    plz: Option<String>,
    city: Option<String>,
    country: Option<String>,
    contact_person: Option<String>,
    notes: Option<String>,
) -> Result<PaymentRecipient, String> {
    let conn = Connection::open(get_db_path()).map_err(|e| e.to_string())?;

    let country_value = country.unwrap_or_else(|| "Deutschland".to_string());

    conn.execute(
        "INSERT INTO payment_recipients (name, company, street, plz, city, country, contact_person, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![name, company, street, plz, city, country_value, contact_person, notes],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    // Rückgabe des neu erstellten Empfängers
    get_payment_recipient(id)?.ok_or_else(|| "Fehler: Empfänger wurde nicht gefunden".to_string())
}

/// Rechnungsempfänger aktualisieren
#[tauri::command]
pub fn update_payment_recipient(
    id: i64,
    name: String,
    company: Option<String>,
    street: Option<String>,
    plz: Option<String>,
    city: Option<String>,
    country: Option<String>,
    contact_person: Option<String>,
    notes: Option<String>,
) -> Result<PaymentRecipient, String> {
    let conn = Connection::open(get_db_path()).map_err(|e| e.to_string())?;

    let country_value = country.unwrap_or_else(|| "Deutschland".to_string());

    let rows_affected = conn
        .execute(
            "UPDATE payment_recipients
             SET name = ?1, company = ?2, street = ?3, plz = ?4, city = ?5,
                 country = ?6, contact_person = ?7, notes = ?8, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?9",
            rusqlite::params![
                name,
                company,
                street,
                plz,
                city,
                country_value,
                contact_person,
                notes,
                id
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows_affected == 0 {
        return Err("Rechnungsempfänger nicht gefunden".to_string());
    }

    get_payment_recipient(id)?.ok_or_else(|| "Fehler: Empfänger wurde nicht gefunden".to_string())
}

/// Rechnungsempfänger löschen
#[tauri::command]
pub fn delete_payment_recipient(id: i64) -> Result<(), String> {
    let conn = Connection::open(get_db_path()).map_err(|e| e.to_string())?;

    // Prüfen ob der Empfänger in Buchungen verwendet wird
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM bookings WHERE payment_recipient_id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Err(format!(
            "Dieser Rechnungsempfänger wird in {} Buchung(en) verwendet und kann nicht gelöscht werden",
            count
        ));
    }

    let rows_affected = conn
        .execute(
            "DELETE FROM payment_recipients WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| e.to_string())?;

    if rows_affected == 0 {
        return Err("Rechnungsempfänger nicht gefunden".to_string());
    }

    Ok(())
}
