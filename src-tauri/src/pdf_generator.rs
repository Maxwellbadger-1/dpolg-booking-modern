use printpdf::{
    PdfDocument, PdfDocumentReference, PdfLayerReference, BuiltinFont,
    Mm, Point, Line, Polygon, Color, Rgb,
    path::{PaintMode, WindingOrder},
    ImageRotation,
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
    // LOGO LINKS OBEN (in der Sidebar)
    // ========================================================================
    let logo_max_width = 50.0; // Max. 50mm breit
    // Y-Achse geht von UNTEN nach OBEN!
    // A4 = 297mm hoch, Logo soll bei ~250mm (von unten) sein = ~47mm vom oberen Rand
    let logo_y = 250.0; // Position der UNTEREN Kante des Logos (von unten gemessen)

    println!("🔧 [PDF DEBUG] Sidebar-Breite: {}mm", sidebar_width);
    println!("🔧 [PDF DEBUG] Logo-Y-Position: {}mm (untere Kante, von unten gemessen!)", logo_y);

    if let Some(logo_path) = &company.logo_path {
        if std::path::Path::new(logo_path).exists() {
            println!("📂 [PDF DEBUG] Logo-Pfad existiert: {}", logo_path);
            match load_and_insert_logo(&doc, &current_layer, logo_path, Mm(10.0), Mm(logo_y), logo_max_width) {
                Ok(_) => println!("✅ [PDF DEBUG] Logo erfolgreich im PDF eingefügt"),
                Err(e) => println!("❌ [PDF DEBUG] Logo konnte nicht geladen werden: {}", e),
            }
        } else {
            println!("⚠️ [PDF DEBUG] Logo-Pfad existiert NICHT: {}", logo_path);
        }
    } else {
        println!("⚠️ [PDF DEBUG] Kein Logo-Pfad in Einstellungen gefunden");
    }

    // ========================================================================
    // SIDEBAR CONTENT (Links, weiße Schrift)
    // ========================================================================
    let mut sidebar_y = 235.0; // Unterhalb des Logos

    println!("📝 [PDF DEBUG] Sidebar Content Start bei Y={}mm", sidebar_y);

    current_layer.set_fill_color(color_white.clone());

    // Firmenname mit Umbruch (Deutsch) - linksbündig in Sidebar
    let words: Vec<&str> = company.company_name.split_whitespace().collect();
    println!("📝 [PDF DEBUG] Firmenname hat {} Wörter", words.len());
    for (i, word) in words.iter().enumerate() {
        let y_pos = sidebar_y - (i as f32 * 5.0);
        println!("📝 [PDF DEBUG] Wort '{}' bei Y={}mm", word, y_pos);
        current_layer.use_text(
            *word,  // Dereference &&str to &str
            11.0,
            Mm(10.0), // Linksbündig bei 10mm
            Mm(y_pos),
            &font_bold,
        );
    }
    sidebar_y -= (words.len() as f32 * 5.0) + 12.0;

    // Rechnungsempfänger (Deutsch)
    println!("📝 [PDF DEBUG] 'RECHNUNG AN' bei Y={}mm", sidebar_y);
    current_layer.use_text("RECHNUNG AN", 10.0, Mm(10.0), Mm(sidebar_y), &font_bold);
    sidebar_y -= 6.0;

    current_layer.set_fill_color(color_text_light.clone());
    println!("📝 [PDF DEBUG] Gast-Name '{}' bei Y={}mm", guest_name, sidebar_y);
    current_layer.use_text(guest_name, 8.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 4.0;
    if let Some(addr) = guest_address {
        println!("📝 [PDF DEBUG] Adresse bei Y={}mm", sidebar_y);
        current_layer.use_text(addr, 8.0, Mm(10.0), Mm(sidebar_y), &font_regular);
        sidebar_y -= 4.0;
    }
    if let Some(city) = guest_city {
        println!("📝 [PDF DEBUG] Stadt bei Y={}mm", sidebar_y);
        current_layer.use_text(city, 8.0, Mm(10.0), Mm(sidebar_y), &font_regular);
        sidebar_y -= 4.0;
    }

    // Vielen Dank (Mitte der Sidebar, vertikal zentriert)
    sidebar_y = 150.0;
    println!("📝 [PDF DEBUG] 'Vielen Dank' bei Y={}mm", sidebar_y);
    current_layer.set_fill_color(color_white.clone());
    current_layer.use_text("Vielen Dank", 16.0, Mm(12.0), Mm(sidebar_y), &font_bold);

    // Zahlungsinformationen (unten in Sidebar)
    sidebar_y = 95.0;
    println!("📝 [PDF DEBUG] 'Zahlungsinformationen' Header bei Y={}mm", sidebar_y);
    current_layer.set_fill_color(color_white.clone());
    current_layer.use_text("Zahlungs-", 9.0, Mm(10.0), Mm(sidebar_y), &font_bold);
    sidebar_y -= 4.0;
    current_layer.use_text("informationen", 9.0, Mm(10.0), Mm(sidebar_y), &font_bold);
    sidebar_y -= 8.0;

    current_layer.set_fill_color(color_text_light.clone());
    println!("📝 [PDF DEBUG] IBAN bei Y={}mm", sidebar_y);
    current_layer.use_text("IBAN:", 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&payment.iban, 6.5, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 5.5;

    println!("📝 [PDF DEBUG] Kontoinhaber bei Y={}mm", sidebar_y);
    current_layer.use_text("Kontoinhaber:", 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&payment.account_holder, 6.5, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 5.5;

    println!("📝 [PDF DEBUG] Bank bei Y={}mm", sidebar_y);
    current_layer.use_text("Bank:", 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&payment.bank_name, 6.5, Mm(10.0), Mm(sidebar_y), &font_regular);

    // Kontaktdaten (ganz unten in Sidebar)
    sidebar_y = 30.0;
    println!("📝 [PDF DEBUG] Kontaktdaten bei Y={}mm", sidebar_y);
    current_layer.use_text(&company.phone, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&company.email, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);
    sidebar_y -= 3.5;
    current_layer.use_text(&company.website, 7.0, Mm(10.0), Mm(sidebar_y), &font_regular);

    // ========================================================================
    // RECHTER BEREICH (White Background) - RECHNUNG HEADER
    // ========================================================================
    let content_x = sidebar_width + 15.0; // 15mm Abstand von Sidebar
    let mut content_y = 265.0; // Weiter oben

    println!("📄 [PDF DEBUG] ===== RECHTER BEREICH START =====");
    println!("📄 [PDF DEBUG] Content Start: X={}mm, Y={}mm", content_x, content_y);

    current_layer.set_fill_color(color_text_dark.clone());

    // RECHNUNG (großer Titel, rechtsbündig angeordnet)
    let title_x = content_x + 45.0; // Position für "RECHNUNG"
    println!("📄 [PDF DEBUG] 'RECHNUNG' Titel bei ({}, {})", title_x, content_y);
    current_layer.use_text("RECHNUNG", 26.0, Mm(title_x), Mm(content_y), &font_bold);
    content_y -= 18.0;

    // Rechnungsdetails (rechtsbündig)
    let details_x = content_x + 55.0;
    println!("📄 [PDF DEBUG] 'Rechnungsdetails' bei ({}, {})", details_x, content_y);
    current_layer.use_text("Rechnungsdetails", 10.0, Mm(details_x), Mm(content_y), &font_bold);
    content_y -= 6.0;

    println!("📄 [PDF DEBUG] Rechnungsdatum bei Y={}", content_y);
    current_layer.use_text(
        &format!("Rechnungsdatum: {}", datum_heute),
        8.5,
        Mm(details_x),
        Mm(content_y),
        &font_regular
    );
    content_y -= 4.5;

    println!("📄 [PDF DEBUG] Fälligkeitsdatum bei Y={}", content_y);
    current_layer.use_text(
        &format!("Fälligkeitsdatum: {}", due_date_de),
        8.5,
        Mm(details_x),
        Mm(content_y),
        &font_regular
    );
    content_y -= 4.5;

    println!("📄 [PDF DEBUG] Rechnungsnummer bei Y={}", content_y);
    current_layer.use_text(
        &format!("Rechnungsnr.: #{}", reservierungsnummer),
        8.5,
        Mm(details_x),
        Mm(content_y),
        &font_regular
    );

    content_y -= 18.0;

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
    // ZAHLUNGSBEDINGUNGEN (unten mittig im weißen Bereich)
    // ========================================================================
    content_y = 65.0; // Unterer Bereich
    let terms_x = content_x + 10.0; // Linksbündig im Content-Bereich

    println!("📄 [PDF DEBUG] ===== ZAHLUNGSBEDINGUNGEN =====");
    println!("📄 [PDF DEBUG] Position: ({}, {})", terms_x, content_y);

    current_layer.use_text("ZAHLUNGSBEDINGUNGEN", 9.0, Mm(terms_x), Mm(content_y), &font_bold);
    content_y -= 5.5;

    let terms_text = format!(
        "Der Betrag ist {} Tage nach Erhalt dieser Rechnung zur Zahlung fällig.",
        payment.payment_due_days
    );
    println!("📄 [PDF DEBUG] Terms Text bei Y={}", content_y);
    current_layer.use_text(&terms_text, 7.5, Mm(terms_x), Mm(content_y), &font_regular);
    content_y -= 4.0;

    println!("📄 [PDF DEBUG] Überweisungstext bei Y={}", content_y);
    current_layer.use_text(
        &format!("Bitte unter Angabe der Rechnungsnummer '{}' überweisen.", reservierungsnummer),
        7.5,
        Mm(terms_x),
        Mm(content_y),
        &font_regular
    );

    // Unterschrift (rechts unten, nicht überlappend)
    let signature_x = terms_x + 65.0;
    let signature_y = 50.0;

    println!("📄 [PDF DEBUG] Unterschrift bei ({}, {})", signature_x, signature_y);
    current_layer.use_text("Unterschrift", 9.0, Mm(signature_x), Mm(signature_y + 2.0), &font_bold);
    draw_horizontal_line(
        &current_layer,
        Mm(signature_x),
        Mm(signature_x + 35.0),
        Mm(signature_y),
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
///
/// WICHTIG:
/// - Y-Achse geht von UNTEN nach OBEN (PDF-Koordinatensystem!)
/// - translate_x/translate_y = Position der UNTEREN LINKEN Ecke des Bildes
/// - scale_x/scale_y = Skalierungsfaktor (nicht absolute Größe!)
fn load_and_insert_logo(
    doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    logo_path: &str,
    x: Mm,
    y: Mm,
    max_width_mm: f32,
) -> Result<(), String> {
    println!("🖼️  [LOGO DEBUG] ===== LOGO LADEN START =====");
    println!("🖼️  [LOGO DEBUG] Pfad: {}", logo_path);
    println!("🖼️  [LOGO DEBUG] Position X: {}mm, Y: {}mm", x.0, y.0);
    println!("🖼️  [LOGO DEBUG] Max Breite: {}mm", max_width_mm);

    // Prüfe ob Logo existiert
    if !std::path::Path::new(logo_path).exists() {
        println!("❌ [LOGO DEBUG] Datei nicht gefunden!");
        return Err(format!("Logo-Datei nicht gefunden: {}", logo_path));
    }

    println!("✅ [LOGO DEBUG] Datei existiert");

    // Lade Image mit image crate
    use image::io::Reader as ImageReader;
    let img = ImageReader::open(logo_path)
        .map_err(|e| {
            println!("❌ [LOGO DEBUG] Fehler beim Öffnen: {}", e);
            format!("Fehler beim Öffnen der Bilddatei: {}", e)
        })?
        .decode()
        .map_err(|e| {
            println!("❌ [LOGO DEBUG] Fehler beim Dekodieren: {}", e);
            format!("Fehler beim Dekodieren: {}", e)
        })?;

    let img_width_px = img.width();
    let img_height_px = img.height();
    println!("✅ [LOGO DEBUG] Bild dekodiert: {}x{} Pixel", img_width_px, img_height_px);

    // Konvertiere zu printpdf Image
    use printpdf::Image;
    let pdf_image = Image::from_dynamic_image(&img);
    println!("✅ [LOGO DEBUG] PDF Image-Objekt erstellt");

    // ========================================================================
    // KRITISCH: Dimensionsberechnung
    // ========================================================================
    // Aspect Ratio beibehalten
    let img_aspect = img_width_px as f32 / img_height_px as f32;
    println!("📏 [LOGO DEBUG] Aspect Ratio: {:.2}", img_aspect);

    // Zielgröße berechnen (max_width_mm ist die maximale Breite)
    let target_width_mm = max_width_mm;
    let target_height_mm = target_width_mm / img_aspect;
    println!("📏 [LOGO DEBUG] Zielgröße: {}mm x {}mm", target_width_mm, target_height_mm);

    // ========================================================================
    // KRITISCH: ImageTransform Berechnung
    // ========================================================================
    // DPI für Skalierung (printpdf verwendet 72 DPI standardmäßig)
    let dpi = 72.0;

    // Berechne Pixel → mm Konvertierung
    // 1 Zoll = 25.4mm
    // Bei 72 DPI: 1 Pixel = 25.4/72 mm ≈ 0.3528 mm
    let px_to_mm = 25.4 / dpi;

    // Originalgröße in mm (bei 72 DPI)
    let original_width_mm = img_width_px as f32 * px_to_mm;
    let original_height_mm = img_height_px as f32 * px_to_mm;
    println!("📏 [LOGO DEBUG] Original-Größe bei 72 DPI: {:.2}mm x {:.2}mm", original_width_mm, original_height_mm);

    // Skalierungsfaktor berechnen
    let scale_x = target_width_mm / original_width_mm;
    let scale_y = target_height_mm / original_height_mm;
    println!("📏 [LOGO DEBUG] Skalierungsfaktoren: scale_x={:.4}, scale_y={:.4}", scale_x, scale_y);

    // Position berechnen (untere linke Ecke des Bildes)
    // Y-Position ist die UNTERE Kante des Logos
    let translate_x = x;
    let translate_y = y; // Hier ist bereits die untere Kante
    println!("📍 [LOGO DEBUG] Translate-Position: X={}mm, Y={}mm", translate_x.0, translate_y.0);

    // ========================================================================
    // KRITISCH: Image ins PDF einfügen
    // ========================================================================
    use printpdf::ImageTransform;

    println!("🔧 [LOGO DEBUG] Nutze ImageTransform mit Position & Größe...");

    // WICHTIG: Nutze translate + scale für korrekte Positionierung
    let transform = ImageTransform {
        translate_x: Some(translate_x),
        translate_y: Some(translate_y),
        rotate: None, // Keine Rotation
        scale_x: Some(scale_x),
        scale_y: Some(scale_y),
        dpi: Some(dpi),
    };

    println!("🔧 [LOGO DEBUG] Transform: translate=({}, {}), scale=({:.4}, {:.4}), dpi={}",
        translate_x.0, translate_y.0, scale_x, scale_y, dpi);
    println!("🔧 [LOGO DEBUG] Füge Image zur Layer hinzu...");
    pdf_image.add_to_layer(layer.clone(), transform);

    println!("✅ [LOGO DEBUG] ===== LOGO ERFOLGREICH PLATZIERT =====");
    println!("📦 [LOGO DEBUG] Finale Position: ({:.2}mm, {:.2}mm)", translate_x.0, translate_y.0);
    println!("📦 [LOGO DEBUG] Finale Größe: {:.2}mm x {:.2}mm", target_width_mm, target_height_mm);
    println!("📦 [LOGO DEBUG] Oberkante des Logos bei: {:.2}mm (von unten)", translate_y.0 + target_height_mm);

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
