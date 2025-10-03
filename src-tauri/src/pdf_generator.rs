use printpdf::{
    PdfDocument, PdfDocumentReference, PdfLayerReference, BuiltinFont,
    Mm, Point, Line, Polygon, Color, Rgb,
    path::{PaintMode, WindingOrder},
};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use chrono::NaiveDate;

use crate::database::{get_company_settings, get_payment_settings};

/// Generiert eine moderne PDF-Rechnung mit dunkler Sidebar und Logo oben mittig
///
/// Design basiert auf moderner Invoice-Vorlage mit:
/// - Dunkler Sidebar links (ca. 1/3 der Breite)
/// - Logo oben mittig
/// - Übersichtliche Tabelle mit klarer Struktur
/// - Payment Info in der Sidebar
/// - Terms & Conditions unten rechts
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
    _anzahl_gaeste: i32,
    grundpreis: f64,
    services_preis: f64,
    rabatt_preis: f64,
    gesamtpreis: f64,
    output_path: &PathBuf,
) -> Result<PathBuf, String> {
    // ========================================================================
    // DATEN LADEN AUS EINSTELLUNGEN
    // ========================================================================
    let company = get_company_settings()
        .map_err(|e| format!("Fehler beim Laden der Firmeneinstellungen: {}", e))?;

    let payment = get_payment_settings()
        .map_err(|e| format!("Fehler beim Laden der Zahlungseinstellungen: {}", e))?;

    // ========================================================================
    // DATUMS-VERARBEITUNG
    // ========================================================================
    let checkin = NaiveDate::parse_from_str(checkin_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid checkin date: {}", e))?;
    let checkout = NaiveDate::parse_from_str(checkout_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid checkout date: {}", e))?;
    let naechte = (checkout - checkin).num_days();

    // Formatiere Daten deutsch
    let checkin_de = checkin.format("%d.%m.%Y").to_string();
    let checkout_de = checkout.format("%d.%m.%Y").to_string();
    let datum_heute = chrono::Local::now().format("%d.%m.%Y").to_string();

    // Zahlungsziel berechnen
    let due_date = chrono::Local::now()
        .naive_local()
        .date()
        + chrono::Duration::days(payment.payment_due_days as i64);
    let due_date_de = due_date.format("%d.%m.%Y").to_string();

    // ========================================================================
    // MWST-BERECHNUNG
    // ========================================================================
    let mwst_satz = payment.mwst_rate;
    let netto = gesamtpreis / (1.0 + mwst_satz / 100.0);
    let mwst_betrag = gesamtpreis - netto;

    // ========================================================================
    // PDF DOKUMENT ERSTELLEN (A4)
    // ========================================================================
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

    // ========================================================================
    // FARB-DEFINITIONEN (Dark Sidebar Design)
    // ========================================================================
    let color_dark_bg = Color::Rgb(Rgb::new(60.0/255.0, 60.0/255.0, 60.0/255.0, None)); // #3C3C3C Dunkelgrau
    let color_white = Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None));
    let color_text_dark = Color::Rgb(Rgb::new(31.0/255.0, 41.0/255.0, 55.0/255.0, None)); // #1F2937
    let color_text_light = Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None)); // Hellgrau für Sidebar
    let color_table_header = Color::Rgb(Rgb::new(0.95, 0.95, 0.95, None)); // #F3F3F3 Hellgrau
    let color_border = Color::Rgb(Rgb::new(0.85, 0.85, 0.85, None));

    // ========================================================================
    // DUNKLE SIDEBAR LINKS (1/3 der Breite = 70mm)
    // ========================================================================
    let sidebar_width = 70.0;

    // Sidebar-Hintergrund
    draw_filled_rect(&current_layer, Mm(0.0), Mm(0.0), Mm(sidebar_width), Mm(297.0), color_dark_bg.clone());

    // ========================================================================
    // LOGO LINKS OBEN (zentriert in der Sidebar-Breite)
    // ========================================================================
    let logo_y = 270.0; // Ganz oben
    let logo_x = sidebar_width / 2.0; // Zentriert in Sidebar (35mm von links)

    if let Some(logo_path) = &company.logo_path {
        if std::path::Path::new(logo_path).exists() {
            match load_and_insert_logo(&doc, &current_layer, logo_path, Mm(logo_x), Mm(logo_y), 30.0) {
                Ok(_) => println!("✅ Logo erfolgreich im PDF eingefügt"),
                Err(e) => println!("⚠️ Logo konnte nicht geladen werden: {}", e),
            }
        }
    }

    // ========================================================================
    // SIDEBAR CONTENT (Links, weiße Schrift)
    // ========================================================================
    let mut sidebar_y = 240.0;

    current_layer.set_fill_color(color_white.clone());

    // Firmenname mit Umbruch (Deutsch) - zentriert in Sidebar
    let words: Vec<&str> = company.company_name.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        current_layer.use_text(
            word,
            11.0,
            Mm(sidebar_width / 2.0 - (word.len() as f32 * 1.5)), // Annähernd zentriert
            Mm(sidebar_y - (i as f32 * 4.5)),
            &font_bold,
        );
    }
    sidebar_y -= (words.len() as f32 * 4.5) + 10.0;

    // Rechnungsempfänger (Deutsch)
    current_layer.use_text("RECHNUNG AN", 10.0, Mm(10.0), Mm(sidebar_y), &font_bold);
    sidebar_y -= 5.0;

    current_layer.set_fill_color(color_text_light.clone());
    current_layer.use_text(guest_name, 8.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 4.0;
    if let Some(addr) = guest_address {
        current_layer.use_text(addr, 8.0, Mm(10.0), Mm(sidebar_y), &font_regular);
        sidebar_y -= 4.0;
    }
    if let Some(city) = guest_city {
        current_layer.use_text(city, 8.0, Mm(10.0), Mm(sidebar_y), &font_regular);
        sidebar_y -= 4.0;
    }

    // Vielen Dank (Mitte)
    sidebar_y = 150.0;
    current_layer.set_fill_color(color_white.clone());
    current_layer.use_text("Vielen Dank", 18.0, Mm(10.0), Mm(sidebar_y), &font_bold);

    // Zahlungsinformationen (unten in Sidebar mit Umbruch)
    sidebar_y = 90.0;
    current_layer.set_fill_color(color_white.clone());
    current_layer.use_text("Zahlungs-", 10.0, Mm(10.0), Mm(sidebar_y), &font_bold);
    sidebar_y -= 4.5;
    current_layer.use_text("informationen", 10.0, Mm(10.0), Mm(sidebar_y), &font_bold);
    sidebar_y -= 7.0;

    current_layer.set_fill_color(color_text_light.clone());
    current_layer.use_text("IBAN:", 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&payment.iban, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 5.0;

    current_layer.use_text("Kontoinhaber:", 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&payment.account_holder, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 5.0;

    current_layer.use_text("Bank:", 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&payment.bank_name, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);

    // Kontaktdaten (ganz unten)
    sidebar_y = 40.0;
    current_layer.use_text(&company.phone, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&company.email, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&company.website, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);

    // ========================================================================
    // RECHTER BEREICH (White Background) - RECHNUNG HEADER
    // ========================================================================
    let content_x = sidebar_width + 10.0; // 10mm Abstand von Sidebar
    let mut content_y = 250.0;

    current_layer.set_fill_color(color_text_dark.clone());

    // RECHNUNG (großer Titel)
    current_layer.use_text("RECHNUNG", 28.0, Mm(content_x + 55.0), Mm(content_y), &font_bold);
    content_y -= 15.0;

    // Rechnungsdetails
    current_layer.use_text("Rechnungsdetails", 11.0, Mm(content_x + 70.0), Mm(content_y), &font_bold);
    content_y -= 6.0;

    current_layer.use_text(
        &format!("Rechnungsdatum: {}", datum_heute),
        9.0,
        Mm(content_x + 70.0),
        Mm(content_y),
        &font_regular
    );
    content_y -= 4.5;
    current_layer.use_text(
        &format!("Fälligkeitsdatum: {}", due_date_de),
        9.0,
        Mm(content_x + 70.0),
        Mm(content_y),
        &font_regular
    );
    content_y -= 4.5;
    current_layer.use_text(
        &format!("Rechnungsnummer: #{}", reservierungsnummer),
        9.0,
        Mm(content_x + 70.0),
        Mm(content_y),
        &font_regular
    );

    content_y -= 15.0;

    // ========================================================================
    // TABELLE (überlappt Sidebar, weißer Hintergrund)
    // ========================================================================
    let table_x = 35.0; // Beginnt über der Sidebar (sidebar_width = 70mm)
    let table_width = 210.0 - table_x - 10.0; // Bis zum rechten Rand
    let col_widths = [15.0, 65.0, 30.0, 20.0, 30.0]; // NR, BESCHREIBUNG, EINZELPREIS, MENGE, GESAMT

    // Weißer Hintergrund für gesamte Tabelle
    let table_height = 60.0; // Angepasst an Anzahl der Zeilen
    draw_filled_rect(
        &current_layer,
        Mm(table_x),
        Mm(content_y - table_height),
        Mm(table_width),
        Mm(table_height),
        color_white.clone()
    );

    // Tabellen-Header Hintergrund (grau, über weißem Hintergrund)
    draw_filled_rect(
        &current_layer,
        Mm(table_x),
        Mm(content_y - 7.0),
        Mm(table_width),
        Mm(7.0),
        color_table_header.clone()
    );

    // Header Text (Deutsch)
    current_layer.set_fill_color(color_text_dark.clone());
    current_layer.use_text("NR", 9.0, Mm(table_x + 2.0), Mm(content_y - 5.0), &font_bold);
    current_layer.use_text("LEISTUNGSBESCHREIBUNG", 9.0, Mm(table_x + col_widths[0] + 2.0), Mm(content_y - 5.0), &font_bold);
    current_layer.use_text("EINZELPREIS", 9.0, Mm(table_x + col_widths[0] + col_widths[1] + 2.0), Mm(content_y - 5.0), &font_bold);
    current_layer.use_text("MENGE", 9.0, Mm(table_x + col_widths[0] + col_widths[1] + col_widths[2] + 2.0), Mm(content_y - 5.0), &font_bold);
    current_layer.use_text("GESAMT", 9.0, Mm(table_x + col_widths[0] + col_widths[1] + col_widths[2] + col_widths[3] + 2.0), Mm(content_y - 5.0), &font_bold);

    content_y -= 10.0;

    // ========================================================================
    // TABELLEN-ZEILEN
    // ========================================================================
    let mut row_num = 1;

    // Zeile 1: Übernachtung
    current_layer.use_text(&format!("{:02}", row_num), 9.0, Mm(table_x + 2.0), Mm(content_y), &font_regular);
    current_layer.use_text(
        &format!("{} ({} - {})", room_name, checkin_de, checkout_de),
        9.0,
        Mm(table_x + col_widths[0] + 2.0),
        Mm(content_y),
        &font_regular
    );
    current_layer.use_text(
        &format_price(grundpreis / naechte as f64),
        9.0,
        Mm(table_x + col_widths[0] + col_widths[1] + 2.0),
        Mm(content_y),
        &font_regular
    );
    current_layer.use_text(
        &format!("{}", naechte),
        9.0,
        Mm(table_x + col_widths[0] + col_widths[1] + col_widths[2] + 2.0),
        Mm(content_y),
        &font_regular
    );
    current_layer.use_text(
        &format_price(grundpreis),
        9.0,
        Mm(table_x + col_widths[0] + col_widths[1] + col_widths[2] + col_widths[3] + 2.0),
        Mm(content_y),
        &font_regular
    );
    content_y -= 6.0;
    row_num += 1;

    // Zeile 2: Zusatzleistungen (falls vorhanden)
    if services_preis > 0.01 {
        current_layer.use_text(&format!("{:02}", row_num), 9.0, Mm(table_x + 2.0), Mm(content_y), &font_regular);
        current_layer.use_text("Zusatzleistungen", 9.0, Mm(table_x + col_widths[0] + 2.0), Mm(content_y), &font_regular);
        current_layer.use_text(&format_price(services_preis), 9.0, Mm(table_x + col_widths[0] + col_widths[1] + 2.0), Mm(content_y), &font_regular);
        current_layer.use_text("1", 9.0, Mm(table_x + col_widths[0] + col_widths[1] + col_widths[2] + 2.0), Mm(content_y), &font_regular);
        current_layer.use_text(&format_price(services_preis), 9.0, Mm(table_x + col_widths[0] + col_widths[1] + col_widths[2] + col_widths[3] + 2.0), Mm(content_y), &font_regular);
        content_y -= 6.0;
        row_num += 1;
    }

    // Zeile 3: Rabatt (falls vorhanden)
    if rabatt_preis.abs() > 0.01 {
        current_layer.use_text(&format!("{:02}", row_num), 9.0, Mm(table_x + 2.0), Mm(content_y), &font_regular);
        current_layer.use_text("Rabatt", 9.0, Mm(table_x + col_widths[0] + 2.0), Mm(content_y), &font_regular);
        current_layer.use_text(&format_price(rabatt_preis), 9.0, Mm(table_x + col_widths[0] + col_widths[1] + 2.0), Mm(content_y), &font_regular);
        current_layer.use_text("1", 9.0, Mm(table_x + col_widths[0] + col_widths[1] + col_widths[2] + 2.0), Mm(content_y), &font_regular);
        current_layer.use_text(&format_price(rabatt_preis), 9.0, Mm(table_x + col_widths[0] + col_widths[1] + col_widths[2] + col_widths[3] + 2.0), Mm(content_y), &font_regular);
        content_y -= 6.0;
    }

    // Trennlinie
    draw_horizontal_line(
        &current_layer,
        Mm(table_x),
        Mm(table_x + table_width),
        Mm(content_y),
        color_border.clone()
    );
    content_y -= 8.0;

    // ========================================================================
    // SUMMEN-BLOCK (grauer Balken bis zur/hinter Sidebar)
    // ========================================================================
    let summary_x = table_x - 25.0; // Beginnt näher an Sidebar (oder dahinter)
    let summary_width = table_width + 25.0; // Erweitert bis/über Sidebar
    let summary_height = 18.0;

    // Grauer Hintergrund für Summen-Block (überspannt bis zur Sidebar)
    draw_filled_rect(
        &current_layer,
        Mm(summary_x),
        Mm(content_y - summary_height + 2.0),
        Mm(summary_width),
        Mm(summary_height),
        color_table_header.clone()
    );

    current_layer.set_fill_color(color_text_dark.clone());

    current_layer.use_text("Zwischensumme", 9.0, Mm(summary_x + 2.0), Mm(content_y), &font_regular);
    current_layer.use_text(&format_price(netto), 9.0, Mm(summary_x + 30.0), Mm(content_y), &font_regular);
    content_y -= 5.0;

    current_layer.use_text(&format!("MwSt. {}%", mwst_satz as i32), 9.0, Mm(summary_x + 2.0), Mm(content_y), &font_regular);
    current_layer.use_text(&format_price(mwst_betrag), 9.0, Mm(summary_x + 30.0), Mm(content_y), &font_regular);
    content_y -= 7.0;

    current_layer.use_text("Gesamtbetrag", 11.0, Mm(summary_x + 2.0), Mm(content_y), &font_bold);
    current_layer.use_text(&format_price(gesamtpreis), 11.0, Mm(summary_x + 30.0), Mm(content_y), &font_bold);
    content_y -= 15.0;

    // ========================================================================
    // ZAHLUNGSBEDINGUNGEN (mittig unten im weißen Bereich, linksbündig)
    // ========================================================================
    content_y = 60.0; // Unterer Bereich
    let terms_x = 105.0 - 35.0; // Mittig, aber linksbündig (105 ist Mitte von 210mm)

    current_layer.use_text("ZAHLUNGSBEDINGUNGEN", 10.0, Mm(terms_x), Mm(content_y), &font_bold);
    content_y -= 5.0;

    let terms_text = format!(
        "Der Betrag ist {} Tage nach Erhalt dieser Rechnung zur Zahlung fällig.",
        payment.payment_due_days
    );
    current_layer.use_text(&terms_text, 8.0, Mm(terms_x), Mm(content_y), &font_regular);
    content_y -= 3.5;
    current_layer.use_text(
        &format!("Bitte unter Angabe der Rechnungsnummer '{}' überweisen.", reservierungsnummer),
        8.0,
        Mm(terms_x),
        Mm(content_y),
        &font_regular
    );

    content_y -= 10.0;

    // Unterschrift (rechts davon)
    current_layer.use_text("Unterschrift", 10.0, Mm(terms_x + 70.0), Mm(content_y + 10.0), &font_bold);
    draw_horizontal_line(
        &current_layer,
        Mm(terms_x + 70.0),
        Mm(terms_x + 110.0),
        Mm(content_y + 7.0),
        color_border.clone()
    );

    // ========================================================================
    // PDF SPEICHERN
    // ========================================================================
    let file = File::create(output_path)
        .map_err(|e| format!("Fehler beim Erstellen der PDF-Datei: {}", e))?;
    let mut writer = BufWriter::new(file);

    doc.save(&mut writer)
        .map_err(|e| format!("Fehler beim Schreiben der PDF: {}", e))?;

    println!("✅ Moderne PDF-Rechnung mit Sidebar erstellt: {:?}", output_path);

    Ok(output_path.clone())
}

// ============================================================================
// HILFSFUNKTIONEN FÜR GRAFIK-ELEMENTE
// ============================================================================

/// Lädt ein PNG/JPG Logo und fügt es ins PDF ein
fn load_and_insert_logo(
    doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    logo_path: &str,
    x: Mm,
    y: Mm,
    max_height_mm: f32,
) -> Result<(), String> {
    println!("🖼️  [LOGO PDF] Versuche Logo zu laden: {}", logo_path);

    // Prüfe ob Logo existiert
    if !std::path::Path::new(logo_path).exists() {
        println!("❌ [LOGO PDF] Datei nicht gefunden: {}", logo_path);
        return Err(format!("Logo-Datei nicht gefunden: {}", logo_path));
    }

    println!("✅ [LOGO PDF] Datei existiert");

    // Lade Image mit image crate
    use image::io::Reader as ImageReader;
    let img = ImageReader::open(logo_path)
        .map_err(|e| format!("Fehler beim Öffnen der Bilddatei: {}", e))?
        .decode()
        .map_err(|e| format!("Fehler beim Dekodieren: {}", e))?;

    println!("✅ [LOGO PDF] Bild dekodiert: {}x{}", img.width(), img.height());

    // Konvertiere zu printpdf Image
    use printpdf::Image;
    let pdf_image = Image::from_dynamic_image(&img);

    println!("✅ [LOGO PDF] PDF Image erstellt");

    // Berechne Dimensionen (max_height_mm beibehalten, Breite proportional)
    let img_aspect = img.width() as f32 / img.height() as f32;
    let height_mm = max_height_mm;
    let width_mm = height_mm * img_aspect;

    println!("✅ [LOGO PDF] Platziere Logo: {}mm x {}mm bei ({}, {})", width_mm, height_mm, x.0, y.0);

    // Füge Image zur Layer hinzu mit Transform
    use printpdf::ImageTransform;

    pdf_image.add_to_layer(
        layer.clone(),
        ImageTransform {
            translate_x: Some(x),
            translate_y: Some(Mm(y.0 - height_mm)), // Von oben nach unten
            scale_x: Some(width_mm / (img.width() as f32 / 300.0 * 25.4)),
            scale_y: Some(height_mm / (img.height() as f32 / 300.0 * 25.4)),
            dpi: Some(300.0),
            ..Default::default()
        }
    );

    println!("✅ [LOGO PDF] Logo erfolgreich im PDF platziert");
    Ok(())
}

/// Zeichnet ein gefülltes Rechteck (Hintergrund)
fn draw_filled_rect(layer: &PdfLayerReference, x: Mm, y: Mm, width: Mm, height: Mm, color: Color) {
    layer.set_fill_color(color.clone());
    layer.set_outline_color(color);
    layer.set_outline_thickness(0.0);

    let points = vec![
        (Point::new(x, y), false),
        (Point::new(Mm(x.0 + width.0), y), false),
        (Point::new(Mm(x.0 + width.0), Mm(y.0 + height.0)), false),
        (Point::new(x, Mm(y.0 + height.0)), false),
    ];

    let polygon = Polygon {
        rings: vec![points],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    };

    layer.add_polygon(polygon);
}

/// Zeichnet eine horizontale Linie
fn draw_horizontal_line(layer: &PdfLayerReference, x_start: Mm, x_end: Mm, y: Mm, color: Color) {
    layer.set_outline_color(color);
    layer.set_outline_thickness(0.5);

    let points = vec![
        (Point::new(x_start, y), false),
        (Point::new(x_end, y), false),
    ];

    let line = Line {
        points,
        is_closed: false,
    };

    layer.add_line(line);
}

/// Formatiert Preis in deutschem Format "XXX,XX €"
fn format_price(price: f64) -> String {
    format!("{:.2} €", price).replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_generate_invoice_pdf_sidebar_design() {
        let temp_dir = env::temp_dir();
        let pdf_path = temp_dir.join("test_invoice_sidebar.pdf");

        let result = generate_invoice_pdf(
            123,
            "25000201",
            "Max Mustermann",
            Some("Teststraße 123"),
            Some("12345 Teststadt"),
            Some("DEUTSCHLAND"),
            "Zimmer 101",
            "2025-01-10",
            "2025-01-15",
            2,
            500.00,
            50.00,
            0.00,
            550.00,
            &pdf_path,
        );

        assert!(result.is_ok());
        assert!(pdf_path.exists());

        println!("Test-PDF mit Sidebar-Design erstellt: {:?}", pdf_path);
    }
}
