// ============================================================================
// DATABASE VIEWS MODULE
// ============================================================================
// Dieses Modul stellt Wrapper-Funktionen bereit, die die neuen Views nutzen
// während die alten Struct-Definitionen beibehalten werden für Kompatibilität
// ============================================================================

use rusqlite::{Connection, Result, params};
use crate::database::{Room, Guest, Booking, BookingWithDetails, PaymentRecipient};

/// Helper function to map view columns back to original struct fields
/// Dies ermöglicht es, die Views zu nutzen ohne alle Structs zu ändern
pub fn map_room_from_view(row: &rusqlite::Row) -> Result<Room> {
    Ok(Room {
        id: row.get(0)?,
        name: row.get(1)?,
        gebaeude_typ: row.get(2)?,  // building_type -> gebaeude_typ
        capacity: row.get(3)?,
        price_member: row.get(4)?,
        price_non_member: row.get(5)?,
        nebensaison_preis: row.get(6)?,  // off_season_price -> nebensaison_preis
        hauptsaison_preis: row.get(7)?,  // main_season_price -> hauptsaison_preis
        endreinigung: row.get(8)?,  // cleaning_fee -> endreinigung
        ort: row.get(9)?,  // location -> ort
        schluesselcode: row.get(10)?,  // key_code -> schluesselcode
        street_address: row.get(11)?,
        postal_code: row.get(12)?,
        city: row.get(13)?,
        notizen: row.get(14)?,  // notes -> notizen
    })
}

/// Get all rooms using the view
pub fn get_all_rooms_via_view(conn: &Connection) -> Result<Vec<Room>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, building_type, capacity, price_member, price_non_member,
                off_season_price, main_season_price, cleaning_fee, location,
                key_code, street_address, postal_code, city, notes
         FROM v_rooms
         ORDER BY name"
    )?;

    let rooms = stmt
        .query_map([], map_room_from_view)?
        .collect::<Result<Vec<_>>>()?;

    Ok(rooms)
}

/// Helper function to map guest from view
pub fn map_guest_from_view(row: &rusqlite::Row) -> Result<Guest> {
    Ok(Guest {
        id: row.get(0)?,
        vorname: row.get(1)?,  // first_name -> vorname
        nachname: row.get(2)?,  // last_name -> nachname
        email: row.get(3)?,
        telefon: row.get(4)?,  // phone -> telefon
        dpolg_mitglied: row.get(5)?,  // is_dpolg_member -> dpolg_mitglied
        strasse: row.get(6)?,  // street -> strasse
        plz: row.get(7)?,  // postal_code -> plz
        ort: row.get(8)?,  // city -> ort
        mitgliedsnummer: row.get(9)?,  // member_number -> mitgliedsnummer
        notizen: row.get(10)?,  // notes -> notizen
        beruf: row.get(11)?,  // occupation -> beruf
        bundesland: row.get(12)?,  // state -> bundesland
        dienststelle: row.get(13)?,  // department -> dienststelle
        created_at: row.get(14)?,
        // Erweiterte Felder
        anrede: row.get(15)?,  // salutation -> anrede
        geschlecht: row.get(16)?,  // gender -> geschlecht
        land: row.get(17)?,  // country -> land
        telefon_geschaeftlich: row.get(18)?,  // phone_business -> telefon_geschaeftlich
        telefon_privat: row.get(19)?,  // phone_private -> telefon_privat
        telefon_mobil: row.get(20)?,  // phone_mobile -> telefon_mobil
        fax: row.get(21)?,
        geburtsdatum: row.get(22)?,  // date_of_birth -> geburtsdatum
        geburtsort: row.get(23)?,  // place_of_birth -> geburtsort
        sprache: row.get(24)?,  // language -> sprache
        nationalitaet: row.get(25)?,  // nationality -> nationalitaet
        identifikationsnummer: row.get(26)?,  // identification_number -> identifikationsnummer
        debitorenkonto: row.get(27)?,  // debtor_account -> debitorenkonto
        kennzeichen: row.get(28)?,  // license_plate -> kennzeichen
        rechnungs_email: row.get(29)?,  // invoice_email -> rechnungs_email
        marketing_einwilligung: row.get(30)?,  // marketing_consent -> marketing_einwilligung
        leitweg_id: row.get(31)?,  // routing_id -> leitweg_id
        kostenstelle: row.get(32)?,  // cost_center -> kostenstelle
        tags: row.get(33)?,
        automail: row.get(34)?,
        automail_sprache: row.get(35)?,  // automail_language -> automail_sprache
    })
}

/// Get all guests using the view
pub fn get_all_guests_via_view(conn: &Connection) -> Result<Vec<Guest>> {
    let mut stmt = conn.prepare(
        "SELECT id, first_name, last_name, email, phone, is_dpolg_member,
                street, postal_code, city, member_number, notes,
                occupation, state, department, created_at,
                salutation, gender, country, phone_business, phone_private,
                phone_mobile, fax, date_of_birth, place_of_birth, language,
                nationality, identification_number, debtor_account, license_plate,
                invoice_email, marketing_consent, routing_id, cost_center,
                tags, automail, automail_language
         FROM v_guests
         ORDER BY last_name, first_name"
    )?;

    let guests = stmt
        .query_map([], map_guest_from_view)?
        .collect::<Result<Vec<_>>>()?;

    Ok(guests)
}

/// Helper function to map booking from view
pub fn map_booking_from_view(row: &rusqlite::Row) -> Result<Booking> {
    Ok(Booking {
        id: row.get(0)?,
        room_id: row.get(1)?,
        guest_id: row.get(2)?,
        reservierungsnummer: row.get(3)?,  // reservation_number -> reservierungsnummer
        checkin_date: row.get(4)?,
        checkout_date: row.get(5)?,
        anzahl_gaeste: row.get(6)?,  // guest_count -> anzahl_gaeste
        status: row.get(7)?,
        gesamtpreis: row.get(8)?,  // total_price -> gesamtpreis
        bemerkungen: row.get(9)?,  // remarks -> bemerkungen
        created_at: row.get(10)?,
        anzahl_begleitpersonen: row.get(11)?,  // companion_count -> anzahl_begleitpersonen
        grundpreis: row.get(12)?,  // base_price -> grundpreis
        services_preis: row.get(13)?,  // services_price -> services_preis
        rabatt_preis: row.get(14)?,  // discount_amount -> rabatt_preis
        anzahl_naechte: row.get(15)?,  // nights_count -> anzahl_naechte
        updated_at: row.get(16)?,
        bezahlt: row.get(17)?,  // is_paid -> bezahlt
        bezahlt_am: row.get(18)?,  // paid_at -> bezahlt_am
        zahlungsmethode: row.get(19)?,  // payment_method -> zahlungsmethode
        mahnung_gesendet_am: row.get(20)?,  // reminder_sent_at -> mahnung_gesendet_am
        rechnung_versendet_am: row.get(21)?,  // invoice_sent_at -> rechnung_versendet_am
        rechnung_versendet_an: row.get(22)?,  // invoice_sent_to -> rechnung_versendet_an
        ist_stiftungsfall: row.get(23)?,  // is_foundation_case -> ist_stiftungsfall
        payment_recipient_id: row.get(24)?,
        putzplan_checkout_date: row.get(25)?,  // cleaning_checkout_date -> putzplan_checkout_date
        ist_dpolg_mitglied: row.get(26)?,  // is_dpolg_member -> ist_dpolg_mitglied
    })
}

/// Get all bookings using the view
pub fn get_all_bookings_via_view(conn: &Connection) -> Result<Vec<Booking>> {
    let mut stmt = conn.prepare(
        "SELECT id, room_id, guest_id, reservation_number, checkin_date, checkout_date,
                guest_count, status, total_price, remarks, created_at,
                companion_count, base_price, services_price, discount_amount,
                nights_count, updated_at, is_paid, paid_at, payment_method,
                reminder_sent_at, invoice_sent_at, invoice_sent_to,
                is_foundation_case, payment_recipient_id, cleaning_checkout_date,
                is_dpolg_member
         FROM v_bookings
         ORDER BY checkin_date DESC"
    )?;

    let bookings = stmt
        .query_map([], map_booking_from_view)?
        .collect::<Result<Vec<_>>>()?;

    Ok(bookings)
}

/// Get bookings with details using views
pub fn get_bookings_with_details_via_view(conn: &Connection) -> Result<Vec<BookingWithDetails>> {
    let query = "
        SELECT
            b.id, b.room_id, b.guest_id, b.reservation_number,
            b.checkin_date, b.checkout_date, b.guest_count,
            b.companion_count, b.status,
            b.base_price, b.services_price, b.discount_amount,
            b.total_price, b.nights_count, b.remarks,
            b.is_paid, b.paid_at, b.payment_method,
            b.reminder_sent_at, b.invoice_sent_at, b.invoice_sent_to,
            b.is_foundation_case, b.payment_recipient_id,
            b.cleaning_checkout_date, b.is_dpolg_member,

            -- Room data
            r.id, r.name, r.building_type, r.capacity,
            r.price_member, r.price_non_member,
            r.off_season_price, r.main_season_price,
            r.cleaning_fee, r.location, r.key_code,
            r.street_address, r.postal_code, r.city, r.notes,

            -- Guest data
            g.id, g.first_name, g.last_name, g.email, g.phone,
            g.is_dpolg_member, g.street, g.postal_code, g.city
        FROM v_bookings b
        LEFT JOIN v_rooms r ON b.room_id = r.id
        LEFT JOIN v_guests g ON b.guest_id = g.id
        ORDER BY b.checkin_date DESC
    ";

    let mut stmt = conn.prepare(query)?;
    let bookings = stmt
        .query_map([], |row| {
            // Map booking fields
            let booking = Booking {
                id: row.get(0)?,
                room_id: row.get(1)?,
                guest_id: row.get(2)?,
                reservierungsnummer: row.get(3)?,
                checkin_date: row.get(4)?,
                checkout_date: row.get(5)?,
                anzahl_gaeste: row.get(6)?,
                anzahl_begleitpersonen: row.get(7)?,
                status: row.get(8)?,
                grundpreis: row.get(9)?,
                services_preis: row.get(10)?,
                rabatt_preis: row.get(11)?,
                gesamtpreis: row.get(12)?,
                anzahl_naechte: row.get(13)?,
                bemerkungen: row.get(14)?,
                bezahlt: row.get(15)?,
                bezahlt_am: row.get(16)?,
                zahlungsmethode: row.get(17)?,
                mahnung_gesendet_am: row.get(18)?,
                rechnung_versendet_am: row.get(19)?,
                rechnung_versendet_an: row.get(20)?,
                ist_stiftungsfall: row.get(21)?,
                payment_recipient_id: row.get(22)?,
                putzplan_checkout_date: row.get(23)?,
                ist_dpolg_mitglied: row.get(24)?,
                created_at: String::new(),
                updated_at: None,
            };

            // Map room fields
            let room = Room {
                id: row.get(25)?,
                name: row.get(26)?,
                gebaeude_typ: row.get(27)?,
                capacity: row.get(28)?,
                price_member: row.get(29)?,
                price_non_member: row.get(30)?,
                nebensaison_preis: row.get(31)?,
                hauptsaison_preis: row.get(32)?,
                endreinigung: row.get(33)?,
                ort: row.get(34)?,
                schluesselcode: row.get(35)?,
                street_address: row.get(36)?,
                postal_code: row.get(37)?,
                city: row.get(38)?,
                notizen: row.get(39)?,
            };

            // Map guest fields (simplified)
            let guest = Guest {
                id: row.get(40)?,
                vorname: row.get(41)?,
                nachname: row.get(42)?,
                email: row.get(43)?,
                telefon: row.get(44)?,
                dpolg_mitglied: row.get(45)?,
                strasse: row.get(46)?,
                plz: row.get(47)?,
                ort: row.get(48)?,
                // Rest with defaults
                mitgliedsnummer: None,
                notizen: None,
                beruf: None,
                bundesland: None,
                dienststelle: None,
                created_at: None,
                anrede: None,
                geschlecht: None,
                land: None,
                telefon_geschaeftlich: None,
                telefon_privat: None,
                telefon_mobil: None,
                fax: None,
                geburtsdatum: None,
                geburtsort: None,
                sprache: None,
                nationalitaet: None,
                identifikationsnummer: None,
                debitorenkonto: None,
                kennzeichen: None,
                rechnungs_email: None,
                marketing_einwilligung: None,
                leitweg_id: None,
                kostenstelle: None,
                tags: None,
                automail: None,
                automail_sprache: None,
            };

            // Create BookingWithDetails
            Ok(BookingWithDetails {
                id: booking.id,
                room_id: booking.room_id,
                guest_id: booking.guest_id,
                reservierungsnummer: booking.reservierungsnummer.clone(),
                checkin_date: booking.checkin_date.clone(),
                checkout_date: booking.checkout_date.clone(),
                anzahl_gaeste: booking.anzahl_gaeste,
                anzahl_begleitpersonen: booking.anzahl_begleitpersonen,
                status: booking.status.clone(),
                grundpreis: booking.grundpreis,
                services_preis: booking.services_preis,
                rabatt_preis: booking.rabatt_preis,
                gesamtpreis: booking.gesamtpreis,
                anzahl_naechte: booking.anzahl_naechte,
                bemerkungen: booking.bemerkungen.clone(),
                bezahlt: booking.bezahlt,
                bezahlt_am: booking.bezahlt_am.clone(),
                zahlungsmethode: booking.zahlungsmethode.clone(),
                mahnung_gesendet_am: booking.mahnung_gesendet_am.clone(),
                rechnung_versendet_am: booking.rechnung_versendet_am.clone(),
                rechnung_versendet_an: booking.rechnung_versendet_an.clone(),
                ist_stiftungsfall: booking.ist_stiftungsfall,
                payment_recipient_id: booking.payment_recipient_id,
                putzplan_checkout_date: booking.putzplan_checkout_date.clone(),
                ist_dpolg_mitglied: booking.ist_dpolg_mitglied,
                room,
                guest,
                services: Vec::new(),  // Would need separate query
                discounts: Vec::new(),  // Would need separate query
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(bookings)
}

/// Search function using views
pub fn search_bookings_via_view(
    conn: &Connection,
    search_term: &str,
) -> Result<Vec<BookingWithDetails>> {
    let query = "
        SELECT DISTINCT
            b.id, b.room_id, b.guest_id, b.reservation_number,
            b.checkin_date, b.checkout_date, b.guest_count,
            b.companion_count, b.status,
            b.base_price, b.services_price, b.discount_amount,
            b.total_price, b.nights_count, b.remarks,
            b.is_paid, b.paid_at, b.payment_method,
            b.reminder_sent_at, b.invoice_sent_at, b.invoice_sent_to,
            b.is_foundation_case, b.payment_recipient_id,
            b.cleaning_checkout_date, b.is_dpolg_member,

            r.id, r.name, r.building_type, r.capacity,
            r.price_member, r.price_non_member,
            r.off_season_price, r.main_season_price,
            r.cleaning_fee, r.location, r.key_code,
            r.street_address, r.postal_code, r.city, r.notes,

            g.id, g.first_name, g.last_name, g.email, g.phone,
            g.is_dpolg_member, g.street, g.postal_code, g.city
        FROM v_bookings b
        LEFT JOIN v_rooms r ON b.room_id = r.id
        LEFT JOIN v_guests g ON b.guest_id = g.id
        WHERE
            b.reservation_number LIKE ?1 OR
            g.first_name LIKE ?1 OR
            g.last_name LIKE ?1 OR
            g.email LIKE ?1 OR
            r.name LIKE ?1
        ORDER BY b.checkin_date DESC
    ";

    let search_pattern = format!("%{}%", search_term);

    let mut stmt = conn.prepare(query)?;
    let bookings = stmt
        .query_map([&search_pattern], |row| {
            // Same mapping as get_bookings_with_details_via_view
            // ... (code omitted for brevity, same as above)
            Ok(BookingWithDetails {
                // ... same mapping
                id: row.get(0)?,
                room_id: row.get(1)?,
                guest_id: row.get(2)?,
                reservierungsnummer: row.get(3)?,
                checkin_date: row.get(4)?,
                checkout_date: row.get(5)?,
                anzahl_gaeste: row.get(6)?,
                anzahl_begleitpersonen: row.get(7)?,
                status: row.get(8)?,
                grundpreis: row.get(9)?,
                services_preis: row.get(10)?,
                rabatt_preis: row.get(11)?,
                gesamtpreis: row.get(12)?,
                anzahl_naechte: row.get(13)?,
                bemerkungen: row.get(14)?,
                bezahlt: row.get(15)?,
                bezahlt_am: row.get(16)?,
                zahlungsmethode: row.get(17)?,
                mahnung_gesendet_am: row.get(18)?,
                rechnung_versendet_am: row.get(19)?,
                rechnung_versendet_an: row.get(20)?,
                ist_stiftungsfall: row.get(21)?,
                payment_recipient_id: row.get(22)?,
                putzplan_checkout_date: row.get(23)?,
                ist_dpolg_mitglied: row.get(24)?,
                room: Room {
                    id: row.get(25)?,
                    name: row.get(26)?,
                    gebaeude_typ: row.get(27)?,
                    capacity: row.get(28)?,
                    price_member: row.get(29)?,
                    price_non_member: row.get(30)?,
                    nebensaison_preis: row.get(31)?,
                    hauptsaison_preis: row.get(32)?,
                    endreinigung: row.get(33)?,
                    ort: row.get(34)?,
                    schluesselcode: row.get(35)?,
                    street_address: row.get(36)?,
                    postal_code: row.get(37)?,
                    city: row.get(38)?,
                    notizen: row.get(39)?,
                },
                guest: Guest {
                    id: row.get(40)?,
                    vorname: row.get(41)?,
                    nachname: row.get(42)?,
                    email: row.get(43)?,
                    telefon: row.get(44)?,
                    dpolg_mitglied: row.get(45)?,
                    strasse: row.get(46)?,
                    plz: row.get(47)?,
                    ort: row.get(48)?,
                    mitgliedsnummer: None,
                    notizen: None,
                    beruf: None,
                    bundesland: None,
                    dienststelle: None,
                    created_at: None,
                    anrede: None,
                    geschlecht: None,
                    land: None,
                    telefon_geschaeftlich: None,
                    telefon_privat: None,
                    telefon_mobil: None,
                    fax: None,
                    geburtsdatum: None,
                    geburtsort: None,
                    sprache: None,
                    nationalitaet: None,
                    identifikationsnummer: None,
                    debitorenkonto: None,
                    kennzeichen: None,
                    rechnungs_email: None,
                    marketing_einwilligung: None,
                    leitweg_id: None,
                    kostenstelle: None,
                    tags: None,
                    automail: None,
                    automail_sprache: None,
                },
                services: Vec::new(),
                discounts: Vec::new(),
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(bookings)
}

/// Helper to check if views exist
pub fn check_views_exist(conn: &Connection) -> Result<bool> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name LIKE 'v_%'",
        [],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

/// Initialize all views if they don't exist
pub fn init_views(conn: &Connection) -> Result<()> {
    // This would execute the SQL from database_cleanup_phase1.sql
    // For now, we assume the views are created via SQL file

    if !check_views_exist(conn)? {
        println!("⚠️  Views not found. Please run database_cleanup_phase1.sql");
    } else {
        println!("✅ Database views initialized");
    }

    Ok(())
}