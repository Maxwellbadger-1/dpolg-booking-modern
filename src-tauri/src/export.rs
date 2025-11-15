use rusqlite::Connection;
use crate::database;

/// Generates a CSV string from bookings with all details
pub fn export_bookings_to_csv(conn: &Connection) -> Result<String, String> {
    let bookings = database::get_bookings_with_details_from_conn(conn)?;

    let mut csv = String::new();

    // Header
    csv.push_str("Reservierungsnummer,Status,Check-in,Check-out,Anzahl Nächte,");
    csv.push_str("Gast Vorname,Gast Nachname,Gast Email,Gast Telefon,DPolG Stiftung Mitglied,");
    csv.push_str("Zimmer,Gebäude,Ort,Kapazität,Anzahl Gäste,Anzahl Begleitpersonen,");
    csv.push_str("Grundpreis,Services,Rabatt,Gesamtpreis,Bemerkungen\n");

    // Data rows
    for booking in bookings {
        csv.push_str(&escape_csv_field(&booking.reservierungsnummer));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.status));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.checkin_date));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.checkout_date));
        csv.push(',');
        csv.push_str(&booking.anzahl_naechte.to_string());
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.guest_vorname));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.guest_nachname));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.guest_email));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.guest_telefon));
        csv.push(',');
        csv.push_str(if booking.guest_dpolg_mitglied { "Ja" } else { "Nein" });
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.room_name));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.room_gebaeude_typ));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.room_ort));
        csv.push(',');
        csv.push_str(&booking.room_capacity.to_string());
        csv.push(',');
        csv.push_str(&booking.anzahl_gaeste.to_string());
        csv.push(',');
        csv.push_str(&booking.anzahl_begleitpersonen.to_string());
        csv.push(',');
        csv.push_str(&format!("{:.2}", booking.grundpreis));
        csv.push(',');
        csv.push_str(&format!("{:.2}", booking.services_preis));
        csv.push(',');
        csv.push_str(&format!("{:.2}", booking.rabatt_preis));
        csv.push(',');
        csv.push_str(&format!("{:.2}", booking.gesamtpreis));
        csv.push(',');
        csv.push_str(&escape_csv_field(&booking.bemerkungen.unwrap_or_default()));
        csv.push('\n');
    }

    Ok(csv)
}

/// Generates a CSV string from guests
pub fn export_guests_to_csv(conn: &Connection) -> Result<String, String> {
    let guests = database::get_all_guests_from_conn(conn)?;

    let mut csv = String::new();

    // Header
    csv.push_str("ID,Vorname,Nachname,Email,Telefon,DPolG Stiftung Mitglied,");
    csv.push_str("Straße,PLZ,Ort,Mitgliedsnummer,Beruf,Bundesland,Dienststelle,Notizen,Erstellt am\n");

    // Data rows
    for guest in guests {
        csv.push_str(&guest.id.to_string());
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.vorname));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.nachname));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.email));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.telefon));
        csv.push(',');
        csv.push_str(if guest.dpolg_mitglied { "Ja" } else { "Nein" });
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.strasse.unwrap_or_default()));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.plz.unwrap_or_default()));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.ort.unwrap_or_default()));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.mitgliedsnummer.unwrap_or_default()));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.beruf.unwrap_or_default()));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.bundesland.unwrap_or_default()));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.dienststelle.unwrap_or_default()));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.notizen.unwrap_or_default()));
        csv.push(',');
        csv.push_str(&escape_csv_field(&guest.created_at));
        csv.push('\n');
    }

    Ok(csv)
}

/// Generates a CSV string from rooms
pub fn export_rooms_to_csv(conn: &Connection) -> Result<String, String> {
    let rooms = database::get_rooms_from_conn(conn)?;

    let mut csv = String::new();

    // Header
    csv.push_str("ID,Name,Gebäude/Typ,Kapazität,Preis Mitglieder,Preis Nicht-Mitglieder,");
    csv.push_str("Ort,Schlüsselcode\n");

    // Data rows
    for room in rooms {
        csv.push_str(&room.id.to_string());
        csv.push(',');
        csv.push_str(&escape_csv_field(&room.name));
        csv.push(',');
        csv.push_str(&escape_csv_field(&room.gebaeude_typ));
        csv.push(',');
        csv.push_str(&room.capacity.to_string());
        csv.push(',');
        csv.push_str(&format!("{:.2}", room.price_member));
        csv.push(',');
        csv.push_str(&format!("{:.2}", room.price_non_member));
        csv.push(',');
        csv.push_str(&escape_csv_field(&room.ort));
        csv.push(',');
        csv.push_str(&escape_csv_field(&room.schluesselcode.unwrap_or_default()));
        csv.push('\n');
    }

    Ok(csv)
}

/// Escapes a field for CSV format
/// - Wraps in quotes if contains comma, newline, or quote
/// - Doubles any internal quotes
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('\n') || field.contains('"') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_field_simple() {
        assert_eq!(escape_csv_field("test"), "test");
    }

    #[test]
    fn test_escape_csv_field_with_comma() {
        assert_eq!(escape_csv_field("test,value"), "\"test,value\"");
    }

    #[test]
    fn test_escape_csv_field_with_quote() {
        assert_eq!(escape_csv_field("test\"value"), "\"test\"\"value\"");
    }

    #[test]
    fn test_escape_csv_field_with_newline() {
        assert_eq!(escape_csv_field("test\nvalue"), "\"test\nvalue\"");
    }

    #[test]
    fn test_escape_csv_field_complex() {
        assert_eq!(
            escape_csv_field("test, \"quoted\", value\nnewline"),
            "\"test, \"\"quoted\"\", value\nnewline\""
        );
    }
}
