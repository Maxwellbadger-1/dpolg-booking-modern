use rusqlite::{Connection, params};
use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Starting CSV import...");

    // CSV-Datei Pfad
    let csv_path = "/Users/maximilianfegg/Desktop/Sicherungskopie DPolG Buchungssystem.nosynch/Claude Code/contacts_export (4).csv";

    // Datenbank-Verbindung
    let db_path = "/Users/maximilianfegg/Desktop/Sicherungskopie DPolG Buchungssystem.nosynch/Claude Code/dpolg-booking-modern/src-tauri/booking_system.db";
    let conn = Connection::open(db_path)?;

    println!("✅ Database connection established");

    // CSV-Reader erstellen
    let file = File::open(csv_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut success_count = 0;
    let mut error_count = 0;
    let mut skip_count = 0;

    // Iteriere über alle Zeilen
    for (index, result) in rdr.records().enumerate() {
        let record = result?;

        // Extrahiere Felder
        let first_name = record.get(2).unwrap_or("").trim();
        let last_name = record.get(3).unwrap_or("").trim();
        let email = record.get(12).unwrap_or("").trim();

        // Skip wenn Vorname oder Nachname leer
        if first_name.is_empty() || last_name.is_empty() {
            skip_count += 1;
            println!("⏭️  Row {}: Skipping (empty name)", index + 2);
            continue;
        }

        // Default Email wenn leer
        let email = if email.is_empty() {
            format!("{}{}@noemail.local",
                first_name.to_lowercase().replace(" ", ""),
                last_name.to_lowercase().replace(" ", "")
            )
        } else {
            email.to_string()
        };

        // Telefon-Priorität: mobile > private > business
        let phone_mobile = record.get(10).unwrap_or("").trim();
        let phone_private = record.get(9).unwrap_or("").trim();
        let phone_business = record.get(8).unwrap_or("").trim();

        let telefon = if !phone_mobile.is_empty() {
            Some(phone_mobile.to_string())
        } else if !phone_private.is_empty() {
            Some(phone_private.to_string())
        } else if !phone_business.is_empty() {
            Some(phone_business.to_string())
        } else {
            None
        };

        // Kategorie → dpolg_mitglied
        let kategorie = record.get(29).unwrap_or("").trim();
        let dpolg_mitglied = kategorie.contains("SF") || kategorie.contains("PSF");

        // Andere Felder
        let strasse = record.get(4).unwrap_or("").trim();
        let plz = record.get(6).unwrap_or("").trim();
        let ort = record.get(5).unwrap_or("").trim();
        let notizen = record.get(22).unwrap_or("").trim();

        // Erweiterte Felder
        let anrede = record.get(1).unwrap_or("").trim();
        let geschlecht = record.get(0).unwrap_or("").trim();
        let land = record.get(7).unwrap_or("").trim();
        let fax = record.get(11).unwrap_or("").trim();
        let geburtsdatum = record.get(15).unwrap_or("").trim();
        let geburtsort = record.get(16).unwrap_or("").trim();
        let sprache = record.get(17).unwrap_or("").trim();
        let nationalitaet = record.get(18).unwrap_or("").trim();
        let bundesland = record.get(19).unwrap_or("").trim();
        let identifikationsnummer = record.get(20).unwrap_or("").trim();
        let debitorenkonto = record.get(21).unwrap_or("").trim();
        let kennzeichen = record.get(23).unwrap_or("").trim();
        let rechnungs_email = record.get(24).unwrap_or("").trim();
        let marketing_einwilligung_str = record.get(25).unwrap_or("").trim();
        let marketing_einwilligung = if marketing_einwilligung_str.to_lowercase() == "true" || marketing_einwilligung_str == "1" {
            Some(true)
        } else if marketing_einwilligung_str.to_lowercase() == "false" || marketing_einwilligung_str == "0" {
            Some(false)
        } else {
            None
        };
        let leitweg_id = record.get(26).unwrap_or("").trim();
        let kostenstelle = record.get(27).unwrap_or("").trim();
        let tags = record.get(28).unwrap_or("").trim();

        // Insert in Datenbank
        let result = conn.execute(
            "INSERT INTO guests (
                vorname, nachname, email, telefon, dpolg_mitglied,
                strasse, plz, ort, notizen,
                anrede, geschlecht, land,
                telefon_geschaeftlich, telefon_privat, telefon_mobil, fax,
                geburtsdatum, geburtsort, sprache, nationalitaet, bundesland,
                identifikationsnummer, debitorenkonto, kennzeichen,
                rechnungs_email, marketing_einwilligung,
                leitweg_id, kostenstelle, tags
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                ?10, ?11, ?12,
                ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20, ?21,
                ?22, ?23, ?24,
                ?25, ?26,
                ?27, ?28, ?29
            )",
            params![
                first_name,
                last_name,
                email,
                telefon,
                if dpolg_mitglied { 1 } else { 0 },
                if strasse.is_empty() { None } else { Some(strasse) },
                if plz.is_empty() { None } else { Some(plz) },
                if ort.is_empty() { None } else { Some(ort) },
                if notizen.is_empty() { None } else { Some(notizen) },
                if anrede.is_empty() { None } else { Some(anrede) },
                if geschlecht.is_empty() { None } else { Some(geschlecht) },
                if land.is_empty() { None } else { Some(land) },
                if phone_business.is_empty() { None } else { Some(phone_business) },
                if phone_private.is_empty() { None } else { Some(phone_private) },
                if phone_mobile.is_empty() { None } else { Some(phone_mobile) },
                if fax.is_empty() { None } else { Some(fax) },
                if geburtsdatum.is_empty() { None } else { Some(geburtsdatum) },
                if geburtsort.is_empty() { None } else { Some(geburtsort) },
                if sprache.is_empty() { None } else { Some(sprache) },
                if nationalitaet.is_empty() { None } else { Some(nationalitaet) },
                if bundesland.is_empty() { None } else { Some(bundesland) },
                if identifikationsnummer.is_empty() { None } else { Some(identifikationsnummer) },
                if debitorenkonto.is_empty() { None } else { Some(debitorenkonto) },
                if kennzeichen.is_empty() { None } else { Some(kennzeichen) },
                if rechnungs_email.is_empty() { None } else { Some(rechnungs_email) },
                marketing_einwilligung.map(|b| if b { 1 } else { 0 }),
                if leitweg_id.is_empty() { None } else { Some(leitweg_id) },
                if kostenstelle.is_empty() { None } else { Some(kostenstelle) },
                if tags.is_empty() { None } else { Some(tags) },
            ]
        );

        match result {
            Ok(_) => {
                success_count += 1;
                if success_count % 100 == 0 {
                    println!("✅ Imported {} guests...", success_count);
                }
            }
            Err(e) => {
                error_count += 1;
                println!("❌ Row {}: Error importing {} {} - {}",
                    index + 2, first_name, last_name, e);
            }
        }
    }

    println!("\n📊 Import Summary:");
    println!("   ✅ Success: {} guests", success_count);
    println!("   ⏭️  Skipped: {} rows (empty names)", skip_count);
    println!("   ❌ Errors:  {} rows", error_count);
    println!("   📋 Total:   {} rows processed", success_count + skip_count + error_count);

    Ok(())
}
