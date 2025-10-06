use rusqlite::{Connection, Result};
use crate::database::get_db_path;
use serde::{Serialize, Deserialize};
use chrono::Local;

// ============================================================================
// TRANSACTION LOG STRUCTURES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionLog {
    pub id: i64,
    pub operation_type: String, // "CREATE", "UPDATE", "DELETE"
    pub table_name: String,
    pub record_id: i64,
    pub old_data: Option<String>, // JSON
    pub new_data: Option<String>, // JSON
    pub user_action: String, // Beschreibung der Aktion
    pub created_at: String,
    pub can_undo: bool,
}

// ============================================================================
// DATABASE INIT
// ============================================================================

/// Erstellt die Transaction Log Tabelle
pub fn init_transaction_log_table() -> Result<()> {
    let conn = Connection::open(get_db_path())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS transaction_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            operation_type TEXT NOT NULL,
            table_name TEXT NOT NULL,
            record_id INTEGER NOT NULL,
            old_data TEXT,
            new_data TEXT,
            user_action TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            can_undo INTEGER DEFAULT 1
        )",
        [],
    )?;

    // Index für schnellere Abfragen
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transaction_log_created ON transaction_log(created_at DESC)",
        [],
    )?;

    Ok(())
}

// ============================================================================
// LOG OPERATIONS
// ============================================================================

/// Loggt eine CREATE Operation
pub fn log_create(table_name: &str, record_id: i64, new_data: &str, user_action: &str) -> Result<()> {
    println!("🔍 [transaction_log::log_create] Called:");
    println!("   table_name: {}", table_name);
    println!("   record_id: {}", record_id);
    println!("   user_action: {}", user_action);
    println!("   new_data length: {}", new_data.len());

    let conn = Connection::open(get_db_path())?;

    let rows_affected = conn.execute(
        "INSERT INTO transaction_log (operation_type, table_name, record_id, new_data, user_action)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["CREATE", table_name, record_id, new_data, user_action],
    )?;

    println!("✅ [transaction_log::log_create] Inserted {} row(s)", rows_affected);

    Ok(())
}

/// Loggt eine UPDATE Operation
pub fn log_update(table_name: &str, record_id: i64, old_data: &str, new_data: &str, user_action: &str) -> Result<()> {
    println!("🔍 [transaction_log::log_update] Called:");
    println!("   table_name: {}", table_name);
    println!("   record_id: {}", record_id);
    println!("   user_action: {}", user_action);

    let conn = Connection::open(get_db_path())?;

    let rows_affected = conn.execute(
        "INSERT INTO transaction_log (operation_type, table_name, record_id, old_data, new_data, user_action)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params!["UPDATE", table_name, record_id, old_data, new_data, user_action],
    )?;

    println!("✅ [transaction_log::log_update] Inserted {} row(s)", rows_affected);

    Ok(())
}

/// Loggt eine DELETE Operation
pub fn log_delete(table_name: &str, record_id: i64, old_data: &str, user_action: &str) -> Result<()> {
    let conn = Connection::open(get_db_path())?;

    conn.execute(
        "INSERT INTO transaction_log (operation_type, table_name, record_id, old_data, user_action)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["DELETE", table_name, record_id, old_data, user_action],
    )?;

    Ok(())
}

// ============================================================================
// QUERY OPERATIONS
// ============================================================================

/// Gibt die letzten N Transaction Logs zurück
pub fn get_recent_transactions(limit: i64) -> Result<Vec<TransactionLog>, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Fehler beim Öffnen der Datenbank: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, operation_type, table_name, record_id, old_data, new_data, user_action,
                datetime(created_at, 'localtime') as created_at, can_undo
         FROM transaction_log
         WHERE can_undo = 1
         ORDER BY created_at DESC
         LIMIT ?1"
    ).map_err(|e| format!("Fehler beim Vorbereiten der Abfrage: {}", e))?;

    let logs = stmt.query_map([limit], |row| {
        Ok(TransactionLog {
            id: row.get(0)?,
            operation_type: row.get(1)?,
            table_name: row.get(2)?,
            record_id: row.get(3)?,
            old_data: row.get(4)?,
            new_data: row.get(5)?,
            user_action: row.get(6)?,
            created_at: row.get(7)?,
            can_undo: row.get(8)?,
        })
    })
    .map_err(|e| format!("Fehler beim Abfragen: {}", e))?
    .collect::<Result<Vec<_>>>()
    .map_err(|e| format!("Fehler beim Sammeln der Ergebnisse: {}", e))?;

    Ok(logs)
}

// ============================================================================
// UNDO OPERATIONS
// ============================================================================

/// Macht eine Transaction rückgängig
pub fn undo_transaction(log_id: i64) -> Result<String, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Fehler beim Öffnen der Datenbank: {}", e))?;

    // 1. Transaction Log laden
    let log: TransactionLog = conn.query_row(
        "SELECT id, operation_type, table_name, record_id, old_data, new_data, user_action,
                datetime(created_at, 'localtime') as created_at, can_undo
         FROM transaction_log WHERE id = ?1",
        [log_id],
        |row| {
            Ok(TransactionLog {
                id: row.get(0)?,
                operation_type: row.get(1)?,
                table_name: row.get(2)?,
                record_id: row.get(3)?,
                old_data: row.get(4)?,
                new_data: row.get(5)?,
                user_action: row.get(6)?,
                created_at: row.get(7)?,
                can_undo: row.get(8)?,
            })
        },
    ).map_err(|e| format!("Transaction Log nicht gefunden: {}", e))?;

    if !log.can_undo {
        return Err("Diese Aktion kann nicht rückgängig gemacht werden".to_string());
    }

    // 2. Undo-Operation basierend auf Typ
    match log.operation_type.as_str() {
        "CREATE" => {
            // Bei CREATE: Datensatz löschen
            match log.table_name.as_str() {
                "bookings" => {
                    crate::database::delete_booking(log.record_id)
                        .map_err(|e| format!("Fehler beim Löschen: {}", e))?;
                },
                "guests" => {
                    crate::database::delete_guest(log.record_id)
                        .map_err(|e| format!("Fehler beim Löschen: {}", e))?;
                },
                "rooms" => {
                    crate::database::delete_room(log.record_id)
                        .map_err(|e| format!("Fehler beim Löschen: {}", e))?;
                },
                _ => return Err(format!("Tabelle {} wird nicht unterstützt", log.table_name)),
            }
        },
        "UPDATE" => {
            // Bei UPDATE: Alte Daten wiederherstellen
            if let Some(old_data) = log.old_data {
                match log.table_name.as_str() {
                    "bookings" => {
                        // Parse old booking data and restore it
                        let old_booking: serde_json::Value = serde_json::from_str(&old_data)
                            .map_err(|e| format!("Fehler beim Parsen der alten Buchungsdaten: {}", e))?;

                        // Restore booking using update command with old values
                        // Extract key fields from old_booking JSON
                        let status = old_booking["status"].as_str().unwrap_or("reserviert");
                        let bezahlt = old_booking["bezahlt"].as_bool().unwrap_or(false);
                        let zahlungsmethode = old_booking["zahlungsmethode"].as_str().map(|s| s.to_string());

                        // Update booking to old state
                        conn.execute(
                            "UPDATE bookings SET status = ?1, bezahlt = ?2, zahlungsmethode = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
                            rusqlite::params![status, bezahlt, zahlungsmethode, log.record_id],
                        ).map_err(|e| format!("Fehler beim Wiederherstellen der Buchung: {}", e))?;
                    },
                    "guests" => {
                        // Parse old guest data and restore it
                        let old_guest: serde_json::Value = serde_json::from_str(&old_data)
                            .map_err(|e| format!("Fehler beim Parsen der alten Gast-Daten: {}", e))?;

                        conn.execute(
                            "UPDATE guests SET
                                vorname = ?1, nachname = ?2, email = ?3, telefon = ?4,
                                strasse = ?5, plz = ?6, ort = ?7, dpolg_mitglied = ?8,
                                mitgliedsnummer = ?9, notizen = ?10
                             WHERE id = ?11",
                            rusqlite::params![
                                old_guest["vorname"].as_str().unwrap_or(""),
                                old_guest["nachname"].as_str().unwrap_or(""),
                                old_guest["email"].as_str().unwrap_or(""),
                                old_guest["telefon"].as_str().unwrap_or(""),
                                old_guest["strasse"].as_str(),
                                old_guest["plz"].as_str(),
                                old_guest["ort"].as_str(),
                                old_guest["dpolg_mitglied"].as_bool().unwrap_or(false),
                                old_guest["mitgliedsnummer"].as_str(),
                                old_guest["notizen"].as_str(),
                                log.record_id
                            ],
                        ).map_err(|e| format!("Fehler beim Wiederherstellen des Gastes: {}", e))?;
                    },
                    "rooms" => {
                        // Parse old room data and restore it
                        let old_room: serde_json::Value = serde_json::from_str(&old_data)
                            .map_err(|e| format!("Fehler beim Parsen der alten Zimmer-Daten: {}", e))?;

                        conn.execute(
                            "UPDATE rooms SET
                                name = ?1, gebaeude_typ = ?2, capacity = ?3,
                                price_member = ?4, price_non_member = ?5, ort = ?6,
                                schluesselcode = ?7
                             WHERE id = ?8",
                            rusqlite::params![
                                old_room["name"].as_str().unwrap_or(""),
                                old_room["gebaeude_typ"].as_str().unwrap_or(""),
                                old_room["capacity"].as_i64().unwrap_or(1),
                                old_room["price_member"].as_f64().unwrap_or(0.0),
                                old_room["price_non_member"].as_f64().unwrap_or(0.0),
                                old_room["ort"].as_str().unwrap_or(""),
                                old_room["schluesselcode"].as_str(),
                                log.record_id
                            ],
                        ).map_err(|e| format!("Fehler beim Wiederherstellen des Zimmers: {}", e))?;
                    },
                    _ => return Err(format!("Tabelle {} wird nicht unterstützt", log.table_name)),
                }
            }
        },
        "DELETE" => {
            // Bei DELETE: Datensatz wiederherstellen
            if let Some(old_data) = log.old_data {
                match log.table_name.as_str() {
                    "bookings" => {
                        // Parse old booking data and re-insert it
                        let old_booking: serde_json::Value = serde_json::from_str(&old_data)
                            .map_err(|e| format!("Fehler beim Parsen der alten Buchungsdaten: {}", e))?;

                        // Re-insert the deleted booking with original ID and ALL required fields
                        conn.execute(
                            "INSERT INTO bookings (id, room_id, guest_id, reservierungsnummer, checkin_date, checkout_date, anzahl_gaeste, status, gesamtpreis, bemerkungen, bezahlt, bezahlt_am, zahlungsmethode, anzahl_begleitpersonen, grundpreis, services_preis, rabatt_preis, anzahl_naechte, created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
                            rusqlite::params![
                                log.record_id,
                                old_booking["room_id"].as_i64().unwrap_or(0),
                                old_booking["guest_id"].as_i64().unwrap_or(0),
                                old_booking["reservierungsnummer"].as_str().unwrap_or(""),
                                old_booking["checkin_date"].as_str().unwrap_or(""),
                                old_booking["checkout_date"].as_str().unwrap_or(""),
                                old_booking["anzahl_gaeste"].as_i64().unwrap_or(1),
                                old_booking["status"].as_str().unwrap_or("reserviert"),
                                old_booking["gesamtpreis"].as_f64().unwrap_or(0.0),
                                old_booking["bemerkungen"].as_str(),
                                old_booking["bezahlt"].as_bool().unwrap_or(false),
                                old_booking["bezahlt_am"].as_str(),
                                old_booking["zahlungsmethode"].as_str(),
                                old_booking["anzahl_begleitpersonen"].as_i64().unwrap_or(0),
                                old_booking["grundpreis"].as_f64().unwrap_or(0.0),
                                old_booking["services_preis"].as_f64().unwrap_or(0.0),
                                old_booking["rabatt_preis"].as_f64().unwrap_or(0.0),
                                old_booking["anzahl_naechte"].as_i64().unwrap_or(0),
                                old_booking["created_at"].as_str().unwrap_or("CURRENT_TIMESTAMP"),
                                "CURRENT_TIMESTAMP"
                            ],
                        ).map_err(|e| format!("Fehler beim Wiederherstellen der Buchung: {}", e))?;
                    },
                    "guests" => {
                        // Parse old guest data and re-insert it
                        let old_guest: serde_json::Value = serde_json::from_str(&old_data)
                            .map_err(|e| format!("Fehler beim Parsen der alten Gast-Daten: {}", e))?;

                        conn.execute(
                            "INSERT INTO guests (id, vorname, nachname, email, telefon, strasse, plz, ort, dpolg_mitglied, mitgliedsnummer, notizen, created_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                            rusqlite::params![
                                log.record_id,
                                old_guest["vorname"].as_str().unwrap_or(""),
                                old_guest["nachname"].as_str().unwrap_or(""),
                                old_guest["email"].as_str().unwrap_or(""),
                                old_guest["telefon"].as_str().unwrap_or(""),
                                old_guest["strasse"].as_str(),
                                old_guest["plz"].as_str(),
                                old_guest["ort"].as_str(),
                                old_guest["dpolg_mitglied"].as_bool().unwrap_or(false),
                                old_guest["mitgliedsnummer"].as_str(),
                                old_guest["notizen"].as_str(),
                                old_guest["created_at"].as_str().unwrap_or("CURRENT_TIMESTAMP")
                            ],
                        ).map_err(|e| format!("Fehler beim Wiederherstellen des Gastes: {}", e))?;
                    },
                    "rooms" => {
                        // Parse old room data and re-insert it
                        let old_room: serde_json::Value = serde_json::from_str(&old_data)
                            .map_err(|e| format!("Fehler beim Parsen der alten Zimmer-Daten: {}", e))?;

                        conn.execute(
                            "INSERT INTO rooms (id, name, gebaeude_typ, capacity, price_member, price_non_member, ort, schluesselcode)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                            rusqlite::params![
                                log.record_id,
                                old_room["name"].as_str().unwrap_or(""),
                                old_room["gebaeude_typ"].as_str().unwrap_or(""),
                                old_room["capacity"].as_i64().unwrap_or(1),
                                old_room["price_member"].as_f64().unwrap_or(0.0),
                                old_room["price_non_member"].as_f64().unwrap_or(0.0),
                                old_room["ort"].as_str().unwrap_or(""),
                                old_room["schluesselcode"].as_str()
                            ],
                        ).map_err(|e| format!("Fehler beim Wiederherstellen des Zimmers: {}", e))?;
                    },
                    _ => return Err(format!("Tabelle {} wird nicht unterstützt", log.table_name)),
                }
            }
        },
        _ => return Err(format!("Unbekannter Operation Type: {}", log.operation_type)),
    }

    // 3. Markiere Log als "nicht mehr undo-bar"
    conn.execute(
        "UPDATE transaction_log SET can_undo = 0 WHERE id = ?1",
        [log_id],
    ).map_err(|e| format!("Fehler beim Aktualisieren: {}", e))?;

    Ok(format!("Aktion '{}' erfolgreich rückgängig gemacht", log.user_action))
}

// ============================================================================
// CLEANUP
// ============================================================================

/// Löscht alte Transaction Logs (älter als X Tage)
pub fn cleanup_old_logs(days: i64) -> Result<usize, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Fehler beim Öffnen der Datenbank: {}", e))?;

    let deleted = conn.execute(
        "DELETE FROM transaction_log
         WHERE created_at < datetime('now', '-' || ?1 || ' days')",
        [days],
    ).map_err(|e| format!("Fehler beim Löschen: {}", e))?;

    Ok(deleted)
}
