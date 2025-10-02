use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use chrono::NaiveDate;

/// Generiert eine moderne PDF-Rechnung für eine Buchung im DPolG Stiftung Design
///
/// # Arguments
/// * `booking_id` - ID der Buchung
/// * `reservierungsnummer` - Reservierungsnummer
/// * `guest_name` - Name des Gastes (Vorname + Nachname)
/// * `guest_address` - Adresse des Gastes (Straße + Hausnummer)
/// * `guest_city` - Stadt und PLZ
/// * `guest_country` - Land
/// * `room_name` - Zimmer-Name
/// * `checkin_date` - Check-in Datum (YYYY-MM-DD)
/// * `checkout_date` - Check-out Datum (YYYY-MM-DD)
/// * `anzahl_gaeste` - Anzahl Gäste
/// * `grundpreis` - Grundpreis in EUR
/// * `services_preis` - Zusatzleistungen in EUR
/// * `rabatt_preis` - Rabatt in EUR (negativ)
/// * `gesamtpreis` - Gesamtpreis in EUR
/// * `output_path` - Pfad wo PDF gespeichert werden soll
///
/// # Returns
/// PathBuf zum generierten PDF
pub fn generate_invoice_pdf(
    _booking_id: i64,
    reservierungsnummer: &str,
    guest_name: &str,
    guest_address: Option<&str>,
    guest_city: Option<&str>,
    guest_country: Option<&str>,
    room_name: &str,
    checkin_date: &str,
    checkout_date: &str,
    anzahl_gaeste: i32,
    grundpreis: f64,
    services_preis: f64,
    rabatt_preis: f64,
    gesamtpreis: f64,
    output_path: &PathBuf,
) -> Result<PathBuf, String> {
    // Datums-Parsing
    let checkin = NaiveDate::parse_from_str(checkin_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid checkin date: {}", e))?;
    let checkout = NaiveDate::parse_from_str(checkout_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid checkout date: {}", e))?;
    let naechte = (checkout - checkin).num_days();

    // Formatiere Daten deutsch
    let checkin_de = checkin.format("%d.%m.%Y").to_string();
    let checkout_de = checkout.format("%d.%m.%Y").to_string();
    let datum_heute = chrono::Local::now().format("%d.%m.%Y").to_string();

    // MwSt.-Berechnung (7% für Übernachtungen)
    let mwst_satz = 7.0;
    let netto = gesamtpreis / (1.0 + mwst_satz / 100.0);
    let mwst_betrag = gesamtpreis - netto;
    let brutto = gesamtpreis;

    // PDF Dokument erstellen (A4)
    let (doc, page1, layer1) = PdfDocument::new(
        "DPolG Stiftung - Rechnung",
        Mm(210.0), // A4 width
        Mm(297.0), // A4 height
        "Layer 1",
    );

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Fonts
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Fehler beim Laden der Schrift: {}", e))?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Fehler beim Laden der Schrift: {}", e))?;

    let mut y_pos = 277.0; // Start position from bottom

    // ========================================================================
    // LOGO BEREICH (oben mittig)
    // ========================================================================
    // TODO: Logo-Grafik hier einfügen wenn verfügbar
    // Momentan Platzhalter mit Text
    current_layer.use_text(
        "DPolG",
        16.0,
        Mm(95.0), // Mittig (210/2 - ~10)
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 6.0;
    current_layer.use_text(
        "Stiftung der Deutschen Polizeigewerkschaft",
        9.0,
        Mm(60.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 15.0;

    // ========================================================================
    // ABSENDER-ZEILE (klein, über Empfängeradresse)
    // ========================================================================
    current_layer.use_text(
        "Stiftung der DPolG, Wackersberger Str. 12, 83661 Lenggries",
        7.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 10.0;

    // ========================================================================
    // EMPFÄNGER-ADRESSE
    // ========================================================================
    current_layer.use_text(
        &format!("Herr/Frau {}", guest_name),
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;

    if let Some(addr) = guest_address {
        current_layer.use_text(addr, 11.0, Mm(20.0), Mm(y_pos), &font_regular);
        y_pos -= 5.0;
    }

    if let Some(city) = guest_city {
        current_layer.use_text(city, 11.0, Mm(20.0), Mm(y_pos), &font_regular);
        y_pos -= 5.0;
    }

    if let Some(country) = guest_country {
        current_layer.use_text(country, 11.0, Mm(20.0), Mm(y_pos), &font_regular);
        y_pos -= 5.0;
    }

    // ========================================================================
    // RECHNUNGSDATUM (rechts oben)
    // ========================================================================
    current_layer.use_text(
        &datum_heute,
        11.0,
        Mm(170.0),
        Mm(y_pos + 15.0), // Auf gleicher Höhe wie Empfänger
        &font_regular,
    );

    y_pos -= 15.0;

    // ========================================================================
    // EINLEITUNGSTEXT
    // ========================================================================
    current_layer.use_text(
        "Vielen Dank für Ihre Buchung in unseren Häusern der Stiftung der DPolG, wofür wir Ihnen folgende",
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;
    current_layer.use_text(
        "Leistungen in Rechnung stellen:",
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 10.0;

    // ========================================================================
    // RECHNUNGSNUMMER
    // ========================================================================
    current_layer.use_text(
        &format!("Rechnung: {}", reservierungsnummer),
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 10.0;

    // ========================================================================
    // LEISTUNGEN-TABELLE (Header)
    // ========================================================================
    // Tabellen-Header
    current_layer.use_text("#", 9.0, Mm(20.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Datum", 9.0, Mm(27.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Anz.", 9.0, Mm(50.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Text", 9.0, Mm(60.0), Mm(y_pos), &font_bold);
    current_layer.use_text("MwSt. %", 9.0, Mm(140.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Preis", 9.0, Mm(160.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Summe", 9.0, Mm(180.0), Mm(y_pos), &font_bold);
    y_pos -= 1.0;

    // Trennlinie (mit Unterstrichen simuliert)
    current_layer.use_text(
        "_____________________________________________________________________________________________",
        8.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;

    // ========================================================================
    // LEISTUNGS-ZEILEN
    // ========================================================================
    // Zeile 1: Zimmer-Übernachtung
    current_layer.use_text("1", 9.0, Mm(20.0), Mm(y_pos), &font_regular);
    current_layer.use_text(
        &format!("{}", checkin_de),
        9.0,
        Mm(27.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("{}", naechte),
        9.0,
        Mm(50.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("Logis {} - {} {} - {}", room_name, guest_name, checkin_de, checkout_de),
        9.0,
        Mm(60.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("{:.2}", mwst_satz),
        9.0,
        Mm(140.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("{:.2} €", grundpreis / naechte as f64),
        9.0,
        Mm(157.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("{:.2} €", grundpreis),
        9.0,
        Mm(175.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;

    // Zeile 2: Zusatzleistungen (falls vorhanden)
    if services_preis > 0.01 {
        current_layer.use_text("2", 9.0, Mm(20.0), Mm(y_pos), &font_regular);
        current_layer.use_text(
            &format!("{}", checkin_de),
            9.0,
            Mm(27.0),
            Mm(y_pos),
            &font_regular,
        );
        current_layer.use_text("1", 9.0, Mm(50.0), Mm(y_pos), &font_regular);
        current_layer.use_text(
            "Zusatzleistungen / Endreinigung",
            9.0,
            Mm(60.0),
            Mm(y_pos),
            &font_regular,
        );
        current_layer.use_text(
            &format!("{:.2}", mwst_satz),
            9.0,
            Mm(140.0),
            Mm(y_pos),
            &font_regular,
        );
        current_layer.use_text(
            &format!("{:.2} €", services_preis),
            9.0,
            Mm(157.0),
            Mm(y_pos),
            &font_regular,
        );
        current_layer.use_text(
            &format!("{:.2} €", services_preis),
            9.0,
            Mm(175.0),
            Mm(y_pos),
            &font_regular,
        );
        y_pos -= 5.0;
    }

    // Zeile 3: Rabatt (falls vorhanden)
    if rabatt_preis.abs() > 0.01 {
        current_layer.use_text("3", 9.0, Mm(20.0), Mm(y_pos), &font_regular);
        current_layer.use_text(
            &format!("{}", checkin_de),
            9.0,
            Mm(27.0),
            Mm(y_pos),
            &font_regular,
        );
        current_layer.use_text("1", 9.0, Mm(50.0), Mm(y_pos), &font_regular);
        current_layer.use_text("Rabatt", 9.0, Mm(60.0), Mm(y_pos), &font_regular);
        current_layer.use_text(
            &format!("{:.2}", mwst_satz),
            9.0,
            Mm(140.0),
            Mm(y_pos),
            &font_regular,
        );
        current_layer.use_text(
            &format!("{:.2} €", rabatt_preis),
            9.0,
            Mm(157.0),
            Mm(y_pos),
            &font_regular,
        );
        current_layer.use_text(
            &format!("{:.2} €", rabatt_preis),
            9.0,
            Mm(175.0),
            Mm(y_pos),
            &font_regular,
        );
        y_pos -= 5.0;
    }

    y_pos -= 5.0;

    // ========================================================================
    // MWST-ZUSAMMENFASSUNG (Tabelle)
    // ========================================================================
    // Header
    current_layer.use_text("MwSt. %", 9.0, Mm(80.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Netto Betrag", 9.0, Mm(110.0), Mm(y_pos), &font_bold);
    current_layer.use_text("MwSt. Betrag", 9.0, Mm(140.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Brutto Betrag", 9.0, Mm(170.0), Mm(y_pos), &font_bold);
    y_pos -= 1.0;

    // Trennlinie
    current_layer.use_text(
        "_____________________________________________________",
        8.0,
        Mm(80.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;

    // Werte
    current_layer.use_text(
        &format!("{:.2}", mwst_satz),
        9.0,
        Mm(85.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("{:.2} €", netto),
        9.0,
        Mm(110.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("{:.2} €", mwst_betrag),
        9.0,
        Mm(140.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("{:.2} €", brutto),
        9.0,
        Mm(170.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 1.0;

    // Trennlinie unten
    current_layer.use_text(
        "_____________________________________________________",
        8.0,
        Mm(80.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;

    // Gesamtsumme
    current_layer.use_text("Gesamtsumme", 9.0, Mm(110.0), Mm(y_pos), &font_regular);
    current_layer.use_text(
        &format!("{:.2} €", netto),
        9.0,
        Mm(110.0),
        Mm(y_pos - 5.0),
        &font_regular,
    );
    y_pos -= 10.0;

    // ========================================================================
    // GESAMTSUMME HERVORGEHOBEN
    // ========================================================================
    current_layer.use_text(
        "Gesamtsumme:",
        12.0,
        Mm(130.0),
        Mm(y_pos),
        &font_bold,
    );
    current_layer.use_text(
        &format!("{:.2} €", gesamtpreis),
        12.0,
        Mm(170.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 5.0;

    current_layer.use_text(
        "Zahlung:",
        12.0,
        Mm(130.0),
        Mm(y_pos),
        &font_regular,
    );
    current_layer.use_text(
        &format!("Debitor: {:.2} €", gesamtpreis),
        12.0,
        Mm(160.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 15.0;

    // ========================================================================
    // ZAHLUNGSHINWEIS
    // ========================================================================
    current_layer.use_text(
        &format!(
            "Der Betrag ist 14 Tage nach Erhalt dieser Rechnung zur Zahlung fällig und bis dahin spätestens unter"
        ),
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;
    current_layer.use_text(
        &format!(
            "Angabe des Verwendungszwecks Rechnung Nr. {} auf folgendes Konto zu überweisen:",
            reservierungsnummer
        ),
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 10.0;

    // ========================================================================
    // BANKVERBINDUNG (hervorgehoben in Box)
    // ========================================================================
    current_layer.use_text(
        "Bitte nutzen Sie ab sofort nur noch diese Kontoverbindung:",
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 8.0;

    current_layer.use_text(
        "IBAN: DE70 7009 0500 0001 9999 90",
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 5.0;
    current_layer.use_text(
        "BIC: GENODEF1S04",
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 5.0;
    current_layer.use_text(
        "Sparda Bank München",
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 10.0;

    // ========================================================================
    // ABSCHLUSSTEXT
    // ========================================================================
    current_layer.use_text(
        "Wir wünschen Ihnen einen schönen Aufenthalt!",
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 10.0;

    current_layer.use_text(
        "Mit freundlichen Grüßen",
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 7.0;

    current_layer.use_text(
        "Ihr Stiftungs-Team",
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 20.0;

    // ========================================================================
    // FOOTER (3-spaltig)
    // ========================================================================
    let footer_y = 30.0;

    // Spalte 1: Adresse
    current_layer.use_text(
        "Stiftung der Deutschen",
        7.0,
        Mm(20.0),
        Mm(footer_y),
        &font_regular,
    );
    current_layer.use_text(
        "Polizeigewerkschaft",
        7.0,
        Mm(20.0),
        Mm(footer_y - 3.0),
        &font_regular,
    );
    current_layer.use_text(
        "Wackersberger Str. 12",
        7.0,
        Mm(20.0),
        Mm(footer_y - 6.0),
        &font_regular,
    );
    current_layer.use_text(
        "83661 Lenggries",
        7.0,
        Mm(20.0),
        Mm(footer_y - 9.0),
        &font_regular,
    );

    // Spalte 2: Kontakt
    current_layer.use_text(
        "Telefon: +49 8042 9725-20",
        7.0,
        Mm(80.0),
        Mm(footer_y),
        &font_regular,
    );
    current_layer.use_text(
        "Fax: +49 8042 9725-22",
        7.0,
        Mm(80.0),
        Mm(footer_y - 3.0),
        &font_regular,
    );
    current_layer.use_text(
        "E-Mail: info@dpolg-stiftung.de",
        7.0,
        Mm(80.0),
        Mm(footer_y - 6.0),
        &font_regular,
    );
    current_layer.use_text(
        "www.dpolg-stiftung.de",
        7.0,
        Mm(80.0),
        Mm(footer_y - 9.0),
        &font_regular,
    );

    // Spalte 3: Bank
    current_layer.use_text(
        "Bank: Sparda Bank München",
        7.0,
        Mm(140.0),
        Mm(footer_y),
        &font_regular,
    );
    current_layer.use_text(
        "IBAN: DE 70 7009 0500 0001 9999 90",
        7.0,
        Mm(140.0),
        Mm(footer_y - 3.0),
        &font_regular,
    );
    current_layer.use_text(
        "BIC: GENODEF1S04",
        7.0,
        Mm(140.0),
        Mm(footer_y - 6.0),
        &font_regular,
    );

    // Spalte 3 (weiter unten): Vorstand
    current_layer.use_text(
        "1. Vorstand: Herr Reinhold Merl",
        7.0,
        Mm(140.0),
        Mm(footer_y - 10.0),
        &font_regular,
    );
    current_layer.use_text(
        "Steuer-Nr.: 141/239/71040",
        7.0,
        Mm(140.0),
        Mm(footer_y - 13.0),
        &font_regular,
    );
    current_layer.use_text(
        "Gerichtsstandort: München",
        7.0,
        Mm(140.0),
        Mm(footer_y - 16.0),
        &font_regular,
    );

    // Seitennummer (unten rechts)
    current_layer.use_text(
        "1/1",
        8.0,
        Mm(190.0),
        Mm(10.0),
        &font_regular,
    );

    // ========================================================================
    // PDF SPEICHERN
    // ========================================================================
    let file = File::create(output_path)
        .map_err(|e| format!("Fehler beim Erstellen der PDF-Datei: {}", e))?;
    let mut writer = BufWriter::new(file);

    doc.save(&mut writer)
        .map_err(|e| format!("Fehler beim Schreiben der PDF: {}", e))?;

    println!("✅ PDF-Rechnung erstellt: {:?}", output_path);

    Ok(output_path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_generate_invoice_pdf() {
        let temp_dir = env::temp_dir();
        let pdf_path = temp_dir.join("test_invoice_modern.pdf");

        let result = generate_invoice_pdf(
            123,
            "25000201",
            "Lothar Bonke",
            Some("Hutstr. 65 a"),
            Some("96450 Coburg"),
            Some("DEUTSCHLAND"),
            "Logis Lengries 3 - Bonke, Lothar",
            "2025-11-03",
            "2025-11-08",
            2,
            350.00,
            55.00,
            0.00,
            405.00,
            &pdf_path,
        );

        assert!(result.is_ok());
        assert!(pdf_path.exists());

        println!("Test-PDF erstellt: {:?}", pdf_path);
        // Cleanup optional - für Inspektion auskommentiert
        // std::fs::remove_file(pdf_path).ok();
    }
}
