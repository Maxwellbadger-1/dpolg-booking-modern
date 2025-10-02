use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use chrono::NaiveDate;

/// Generiert eine PDF-Rechnung für eine Buchung
///
/// # Arguments
/// * `booking_id` - ID der Buchung
/// * `reservierungsnummer` - Reservierungsnummer
/// * `guest_name` - Name des Gastes (Vorname + Nachname)
/// * `room_name` - Zimmer-Name
/// * `checkin_date` - Check-in Datum
/// * `checkout_date` - Check-out Datum
/// * `anzahl_gaeste` - Anzahl Gäste
/// * `gesamtpreis` - Gesamtpreis in EUR
/// * `output_path` - Pfad wo PDF gespeichert werden soll
///
/// # Returns
/// PathBuf zum generierten PDF
pub fn generate_invoice_pdf(
    _booking_id: i64,
    reservierungsnummer: &str,
    guest_name: &str,
    room_name: &str,
    checkin_date: &str,
    checkout_date: &str,
    anzahl_gaeste: i32,
    gesamtpreis: f64,
    output_path: &PathBuf,
) -> Result<PathBuf, String> {
    // Berechne Anzahl Nächte
    let checkin = NaiveDate::parse_from_str(checkin_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid checkin date: {}", e))?;
    let checkout = NaiveDate::parse_from_str(checkout_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid checkout date: {}", e))?;
    let naechte = (checkout - checkin).num_days();

    // PDF Dokument erstellen (A4)
    let (doc, page1, layer1) = PdfDocument::new(
        "DPolG Stiftung - Rechnung",
        Mm(210.0), // A4 width
        Mm(297.0), // A4 height
        "Layer 1",
    );

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Font laden (Built-in Helvetica)
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Fehler beim Laden der Schrift: {}", e))?;
    let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Fehler beim Laden der Schrift: {}", e))?;

    // Positions-Helper
    let mut y_pos = 270.0; // Start oben (mm from bottom)

    // === HEADER ===
    current_layer.use_text(
        "DPolG Stiftung",
        24.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 10.0;

    current_layer.use_text(
        "Rechnung / Invoice",
        18.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 15.0;

    // === RECHNUNGS-INFO ===
    current_layer.use_text(
        &format!("Reservierungsnummer: {}", reservierungsnummer),
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 7.0;

    let datum_heute = chrono::Local::now().format("%d.%m.%Y").to_string();
    current_layer.use_text(
        &format!("Rechnungsdatum: {}", datum_heute),
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 15.0;

    // === GAST-INFO ===
    current_layer.use_text(
        "Rechnungsempfänger:",
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 7.0;

    current_layer.use_text(
        guest_name,
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 20.0;

    // === LEISTUNGEN ===
    current_layer.use_text(
        "Leistungen:",
        14.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 10.0;

    // Table Headers
    current_layer.use_text("Beschreibung", 11.0, Mm(20.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Menge", 11.0, Mm(120.0), Mm(y_pos), &font_bold);
    current_layer.use_text("Preis", 11.0, Mm(160.0), Mm(y_pos), &font_bold);
    y_pos -= 7.0;

    // Trennlinie (simuliert mit Unterstrichen)
    current_layer.use_text(
        "____________________________________________________________________________",
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;

    // Zimmer-Buchung
    let beschreibung = format!(
        "Zimmer: {} ({} - {})",
        room_name,
        checkin,
        checkout
    );
    current_layer.use_text(&beschreibung, 11.0, Mm(20.0), Mm(y_pos), &font_regular);
    current_layer.use_text(
        &format!("{} Nächte", naechte),
        11.0,
        Mm(120.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 7.0;

    // Anzahl Gäste
    current_layer.use_text(
        &format!("Anzahl Gäste: {}", anzahl_gaeste),
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 15.0;

    // === SUMME ===
    current_layer.use_text(
        "____________________________________________________________________________",
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 7.0;

    current_layer.use_text(
        "Gesamtbetrag:",
        12.0,
        Mm(120.0),
        Mm(y_pos),
        &font_bold,
    );
    current_layer.use_text(
        &format!("{:.2} €", gesamtpreis),
        12.0,
        Mm(160.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 15.0;

    // === ZAHLUNGSHINWEIS ===
    current_layer.use_text(
        "Zahlungshinweis:",
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 7.0;

    current_layer.use_text(
        "Bitte überweisen Sie den Gesamtbetrag innerhalb von 7 Tagen auf folgendes Konto:",
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 7.0;

    current_layer.use_text(
        "IBAN: DE12 3456 7890 1234 5678 90",
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 5.0;

    current_layer.use_text(
        &format!("Verwendungszweck: {}", reservierungsnummer),
        10.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );
    y_pos -= 15.0;

    // === FOOTER ===
    current_layer.use_text(
        "Vielen Dank für Ihre Buchung!",
        11.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 7.0;

    current_layer.use_text(
        "DPolG Stiftung • Musterstraße 123 • 12345 Musterstadt",
        9.0,
        Mm(20.0),
        Mm(y_pos),
        &font_regular,
    );

    // PDF speichern
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
        let pdf_path = temp_dir.join("test_invoice.pdf");

        let result = generate_invoice_pdf(
            123,
            "RES-2025-001",
            "Max Mustermann",
            "Zimmer 101",
            "2025-01-10",
            "2025-01-15",
            2,
            450.00,
            &pdf_path,
        );

        assert!(result.is_ok());
        assert!(pdf_path.exists());

        // Cleanup
        std::fs::remove_file(pdf_path).ok();
    }
}
