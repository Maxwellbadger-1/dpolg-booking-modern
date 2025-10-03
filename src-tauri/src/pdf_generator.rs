use printpdf::*;
use std::path::PathBuf;
use chrono::NaiveDate;

use crate::database::{get_company_settings, get_payment_settings};

/// Generiert eine moderne PDF-Rechnung mit dunkler Sidebar und Logo oben mittig
///
/// **REFACTORED für printpdf 0.8:**
/// - Verwendet Op:: Pattern für alle Operationen
/// - Logo via RawImage::decode_from_bytes + Op::UseXobject
/// - Seiten mit PdfPage::new() + with_pages()
pub fn generate_invoice_pdf(
    _booking_id: i64,
    reservierungsnummer: &str,
    guest_name: &str,
    guest_address: Option<&str>,
    guest_city: Option<&str>,
    _guest_country: Option<&str>,
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
    let mut doc = PdfDocument::new("DPolG Stiftung - Rechnung");

    // ========================================================================
    // LOGO LADEN (wenn vorhanden)
    // ========================================================================
    let logo_image_id = if let Some(logo_path) = &company.logo_path {
        if std::path::Path::new(logo_path).exists() {
            println!("📂 [PDF DEBUG] Logo-Pfad existiert: {}", logo_path);
            match load_logo_as_raw_image(logo_path) {
                Ok(raw_image) => {
                    let img_id = doc.add_image(&raw_image);
                    println!("✅ [PDF DEBUG] Logo erfolgreich als RawImage geladen");
                    Some(img_id)
                }
                Err(e) => {
                    println!("❌ [PDF DEBUG] Logo konnte nicht geladen werden: {}", e);
                    None
                }
            }
        } else {
            println!("⚠️ [PDF DEBUG] Logo-Pfad existiert NICHT: {}", logo_path);
            None
        }
    } else {
        println!("⚠️ [PDF DEBUG] Kein Logo-Pfad in Einstellungen gefunden");
        None
    };

    // ========================================================================
    // PAGE OPERATIONS ZUSAMMENSTELLEN
    // ========================================================================
    let mut ops: Vec<Op> = Vec::new();
    println!("🔧 [PDF DEBUG] ===== OPERATIONS START =====");

    // ========================================================================
    // FARB-DEFINITIONEN (Dark Sidebar Design)
    // ========================================================================
    let color_dark_bg = Rgb { r: 60.0/255.0, g: 60.0/255.0, b: 60.0/255.0, icc_profile: None }; // #3C3C3C Dunkelgrau
    let color_white = Rgb { r: 1.0, g: 1.0, b: 1.0, icc_profile: None };
    let color_text_dark = Rgb { r: 31.0/255.0, g: 41.0/255.0, b: 55.0/255.0, icc_profile: None }; // #1F2937
    let color_text_light = Rgb { r: 0.8, g: 0.8, b: 0.8, icc_profile: None }; // Hellgrau für Sidebar
    let color_table_header = Rgb { r: 0.95, g: 0.95, b: 0.95, icc_profile: None }; // #F3F3F3 Hellgrau
    let color_border = Rgb { r: 0.85, g: 0.85, b: 0.85, icc_profile: None };

    // ========================================================================
    // DUNKLE SIDEBAR LINKS (1/3 der Breite = 70mm)
    // ========================================================================
    let sidebar_width = 70.0;

    // Sidebar-Hintergrund als gefülltes Polygon
    // Helper: Konvertierung Mm -> Pt (1mm = 2.834645669pt bei 72 DPI)
    let mm_to_pt = |mm: f32| Pt(mm * 2.834645669);

    ops.push(Op::SetFillColor { col: Color::Rgb(color_dark_bg.clone()) });
    ops.push(Op::SetOutlineColor { col: Color::Rgb(color_dark_bg.clone()) });
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    LinePoint { p: Point { x: mm_to_pt(0.0), y: mm_to_pt(0.0) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(sidebar_width), y: mm_to_pt(0.0) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(sidebar_width), y: mm_to_pt(297.0) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(0.0), y: mm_to_pt(297.0) }, bezier: false },
                ],
            }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    });
    println!("✅ [PDF DEBUG] Sidebar-Rechteck hinzugefügt, ops.len() = {}", ops.len());

    // ========================================================================
    // LAYOUT-VARIABLEN (für gesamtes PDF)
    // ========================================================================
    let _content_x = sidebar_width + 15.0;  // Linke Kante des Hauptbereichs
    let content_y_start = 265.0;
    let table_x = 35.0;
    let table_width = 210.0 - table_x - 10.0;
    let table_height = 60.0;
    let col_widths = [15.0, 65.0, 30.0, 20.0, 30.0]; // NR, BESCHREIBUNG, EINZELPREIS, MENGE, GESAMT

    // ========================================================================
    // TABELLEN-HINTERGRÜNDE (Zeichnen BEVOR Text kommt!)
    // ========================================================================
    // Diese müssen ZUERST gezeichnet werden, damit der Text darüber erscheint
    // WICHTIG: Tabelle muss UNTER "Rechnungsdetails" starten!
    // content_y_start = 265mm (RECHNUNG Titel)
    // Nach RECHNUNG: -18mm
    // Nach "Rechnungsdetails": -6mm
    // Nach 3 Zeilen Details (Datum, Fällig, Nummer): -3*4.5mm = -13.5mm
    // Gesamt: 265 - 18 - 6 - 13.5 = 227.5mm
    let table_y = 227.0; // Tabelle startet hier (Oberkante)

    // Weißer Hintergrund für gesamte Tabelle
    ops.push(Op::SetFillColor { col: Color::Rgb(color_white.clone()) });
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    LinePoint { p: Point { x: mm_to_pt(table_x), y: mm_to_pt(table_y - table_height) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(table_x + table_width), y: mm_to_pt(table_y - table_height) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(table_x + table_width), y: mm_to_pt(table_y) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(table_x), y: mm_to_pt(table_y) }, bezier: false },
                ],
            }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    });

    // Tabellen-Header Hintergrund (grau)
    ops.push(Op::SetFillColor { col: Color::Rgb(color_table_header.clone()) });
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    LinePoint { p: Point { x: mm_to_pt(table_x), y: mm_to_pt(table_y - 7.0) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(table_x + table_width), y: mm_to_pt(table_y - 7.0) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(table_x + table_width), y: mm_to_pt(table_y) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(table_x), y: mm_to_pt(table_y) }, bezier: false },
                ],
            }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    });
    println!("✅ [PDF DEBUG] Tabellen-Hintergründe hinzugefügt, ops.len() = {}", ops.len());

    // ========================================================================
    // LOGO EINFÜGEN (in der Sidebar, oben links mit weißem Hintergrund)
    // ========================================================================
    if let Some(logo_id) = logo_image_id {
        // Logo-Dimensionen und Position
        // Sidebar ist 70mm breit, Logo soll max 50mm breit sein, zentriert in Sidebar
        let logo_max_width = 50.0; // mm
        let logo_x = (sidebar_width - logo_max_width) / 2.0; // Zentriert in Sidebar
        let logo_y = 270.0; // Höher platzieren (von unten gemessen) - Force recompile

        // Weißer Hintergrund für Logo (Rechteck)
        let logo_bg_padding = 2.0; // mm Padding um Logo
        ops.push(Op::SetFillColor { col: Color::Rgb(color_white.clone()) });
        ops.push(Op::DrawPolygon {
            polygon: Polygon {
                rings: vec![PolygonRing {
                    points: vec![
                        LinePoint { p: Point { x: mm_to_pt(logo_x - logo_bg_padding), y: mm_to_pt(logo_y - logo_bg_padding) }, bezier: false },
                        LinePoint { p: Point { x: mm_to_pt(logo_x + logo_max_width + logo_bg_padding), y: mm_to_pt(logo_y - logo_bg_padding) }, bezier: false },
                        LinePoint { p: Point { x: mm_to_pt(logo_x + logo_max_width + logo_bg_padding), y: mm_to_pt(logo_y + 20.0 + logo_bg_padding) }, bezier: false },
                        LinePoint { p: Point { x: mm_to_pt(logo_x - logo_bg_padding), y: mm_to_pt(logo_y + 20.0 + logo_bg_padding) }, bezier: false },
                    ],
                }],
                mode: PaintMode::Fill,
                winding_order: WindingOrder::NonZero,
            },
        });
        println!("✅ [LOGO DEBUG] Weißer Hintergrund für Logo hinzugefügt");

        println!("🖼️  [LOGO DEBUG] Logo wird platziert bei X={}mm, Y={}mm", logo_x, logo_y);
        println!("🖼️  [LOGO DEBUG] Logo-Position in Pt: X={}, Y={}", logo_x * 2.834645669, logo_y * 2.834645669);

        // Transform für Logo (Position + Skalierung)
        // Logo auf max 50mm Breite skalieren
        let mut logo_transform = XObjectTransform::default();
        logo_transform.translate_x = Some(Pt(logo_x * 2.834645669));
        logo_transform.translate_y = Some(Pt(logo_y * 2.834645669));
        logo_transform.scale_x = Some(0.2); // Kleiner skalieren (20% statt 50%)
        logo_transform.scale_y = Some(0.2);
        logo_transform.dpi = Some(72.0);

        println!("🖼️  [LOGO DEBUG] XObjectTransform: translate_x={:?}, translate_y={:?}, scale_x={:?}, scale_y={:?}",
                 logo_transform.translate_x, logo_transform.translate_y, logo_transform.scale_x, logo_transform.scale_y);

        ops.push(Op::UseXobject {
            id: logo_id,
            transform: logo_transform,
        });

        println!("✅ [LOGO DEBUG] Logo mit Op::UseXobject hinzugefügt, ops.len() = {}", ops.len());
    } else {
        println!("⚠️ [LOGO DEBUG] Logo-ID ist None, überspringe Logo-Platzierung");
    }

    // ========================================================================
    // SIDEBAR CONTENT (Text, weiße Schrift)
    // ========================================================================
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    println!("📝 [TEXT DEBUG] Sidebar Text Section START, ops.len() = {}", ops.len());

    let mut sidebar_y = 235.0; // Unterhalb des Logos

    // Firmenname (mehrzeilig)
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(11.0), font: BuiltinFont::HelveticaBold });
    ops.push(Op::SetLineHeight { lh: Pt(5.0) });
    ops.push(Op::SetFillColor { col: Color::Rgb(color_white.clone()) });

    let words: Vec<&str> = company.company_name.split_whitespace().collect();
    let words_count = words.len();
    for word in words {
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(word.to_string())],
            font: BuiltinFont::HelveticaBold,
        });
        ops.push(Op::AddLineBreak);
    }

    sidebar_y -= (words_count as f32 * 5.0) + 12.0;

    // Rechnungsempfänger
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(10.0), font: BuiltinFont::HelveticaBold });
    println!("📝 [TEXT DEBUG] Writing 'RECHNUNG AN' at Y={}mm, font=HelveticaBold, size=10pt", sidebar_y);
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("RECHNUNG AN".to_string())],
        font: BuiltinFont::HelveticaBold,
    });
    sidebar_y -= 6.0;

    ops.push(Op::SetFillColor { col: Color::Rgb(color_text_light.clone()) });
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(8.0), font: BuiltinFont::Helvetica });
    println!("📝 [TEXT DEBUG] Writing guest name '{}' at Y={}mm, font=Helvetica, size=8pt", guest_name, sidebar_y);
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(guest_name.to_string())],
        font: BuiltinFont::Helvetica,
    });
    sidebar_y -= 4.0;

    if let Some(addr) = guest_address {
        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(addr.to_string())],
            font: BuiltinFont::Helvetica,
        });
        sidebar_y -= 4.0;
    }

    if let Some(city) = guest_city {
        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
        ops.push(Op::WriteTextBuiltinFont {
            items: vec![TextItem::Text(city.to_string())],
            font: BuiltinFont::Helvetica,
        });
    }

    // "Vielen Dank" (Mitte der Sidebar)
    sidebar_y = 150.0;
    ops.push(Op::SetFillColor { col: Color::Rgb(color_white.clone()) });
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(12.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(16.0), font: BuiltinFont::HelveticaBold });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("Vielen Dank".to_string())],
        font: BuiltinFont::HelveticaBold,
    });

    // Zahlungsinformationen (unten in Sidebar)
    sidebar_y = 95.0;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(9.0), font: BuiltinFont::HelveticaBold });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("Zahlungs-".to_string())],
        font: BuiltinFont::HelveticaBold,
    });
    sidebar_y -= 4.0;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("informationen".to_string())],
        font: BuiltinFont::HelveticaBold,
    });
    sidebar_y -= 8.0;

    ops.push(Op::SetFillColor { col: Color::Rgb(color_text_light.clone()) });
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(7.0), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("IBAN:".to_string())],
        font: BuiltinFont::Helvetica,
    });
    sidebar_y -= 3.5;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(6.5), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(payment.iban.clone())],
        font: BuiltinFont::Helvetica,
    });
    sidebar_y -= 5.5;

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(7.0), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("Kontoinhaber:".to_string())],
        font: BuiltinFont::Helvetica,
    });
    sidebar_y -= 3.5;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(6.5), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(payment.account_holder.clone())],
        font: BuiltinFont::Helvetica,
    });
    sidebar_y -= 5.5;

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(7.0), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("Bank:".to_string())],
        font: BuiltinFont::Helvetica,
    });
    sidebar_y -= 3.5;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(6.5), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(payment.bank_name.clone())],
        font: BuiltinFont::Helvetica,
    });

    // Kontaktdaten (ganz unten in Sidebar)
    sidebar_y = 30.0;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(7.0), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(company.phone.clone())],
        font: BuiltinFont::Helvetica,
    });
    sidebar_y -= 3.5;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(company.email.clone())],
        font: BuiltinFont::Helvetica,
    });
    sidebar_y -= 3.5;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(10.0), y: mm_to_pt(sidebar_y) } });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(company.website.clone())],
        font: BuiltinFont::Helvetica,
    });

    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);
    println!("📝 [TEXT DEBUG] Sidebar Text Section END, ops.len() = {}", ops.len());

    // ========================================================================
    // RECHTER BEREICH (White Background) - RECHNUNG HEADER
    // ========================================================================
    let content_x = sidebar_width + 15.0;
    let mut content_y = 265.0;

    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFillColor { col: Color::Rgb(color_text_dark.clone()) });
    println!("📝 [TEXT DEBUG] Rechnung Header Section START, ops.len() = {}", ops.len());

    // "RECHNUNG" Titel
    let title_x = content_x + 45.0;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(title_x), y: mm_to_pt(content_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(26.0), font: BuiltinFont::HelveticaBold });
    println!("📝 [TEXT DEBUG] Writing 'RECHNUNG' at X={}mm, Y={}mm, size=26pt", title_x, content_y);
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("RECHNUNG".to_string())],
        font: BuiltinFont::HelveticaBold,
    });
    content_y -= 18.0;

    // Rechnungsdetails
    let details_x = content_x + 55.0;
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(details_x), y: mm_to_pt(content_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(10.0), font: BuiltinFont::HelveticaBold });
    println!("📝 [TEXT DEBUG] Writing 'Rechnungsdetails' at X={}mm, Y={}mm, size=10pt", details_x, content_y);
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("Rechnungsdetails".to_string())],
        font: BuiltinFont::HelveticaBold,
    });
    content_y -= 6.0;

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(details_x), y: mm_to_pt(content_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(8.5), font: BuiltinFont::Helvetica });
    println!("📝 [TEXT DEBUG] Writing 'Rechnungsdatum: {}' at Y={}mm, size=8.5pt", datum_heute, content_y);
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(format!("Rechnungsdatum: {}", datum_heute))],
        font: BuiltinFont::Helvetica,
    });
    content_y -= 4.5;

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(details_x), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(format!("Fälligkeitsdatum: {}", due_date_de))],
        font: BuiltinFont::Helvetica,
    });
    content_y -= 4.5;

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(details_x), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(format!("Rechnungsnr.: #{}", reservierungsnummer))],
        font: BuiltinFont::Helvetica,
    });

    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    content_y -= 18.0;

    // ========================================================================
    // TABELLE TEXT (Hintergründe wurden bereits oben gezeichnet)
    // ========================================================================
    // col_widths wurde bereits am Anfang definiert

    // Header Text (Tabellen-Hintergründe wurden bereits am Anfang gezeichnet!)
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFillColor { col: Color::Rgb(color_text_dark.clone()) });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(9.0), font: BuiltinFont::HelveticaBold });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + 2.0), y: mm_to_pt(content_y - 5.0) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("NR".to_string())], font: BuiltinFont::HelveticaBold });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + 2.0), y: mm_to_pt(content_y - 5.0) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("LEISTUNGSBESCHREIBUNG".to_string())], font: BuiltinFont::HelveticaBold });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + 2.0), y: mm_to_pt(content_y - 5.0) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("EINZELPREIS".to_string())], font: BuiltinFont::HelveticaBold });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + col_widths[2] + 2.0), y: mm_to_pt(content_y - 5.0) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("MENGE".to_string())], font: BuiltinFont::HelveticaBold });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + col_widths[2] + col_widths[3] + 2.0), y: mm_to_pt(content_y - 5.0) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("GESAMT".to_string())], font: BuiltinFont::HelveticaBold });

    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    content_y -= 10.0;

    // ========================================================================
    // TABELLEN-ZEILEN
    // ========================================================================
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(9.0), font: BuiltinFont::Helvetica });
    ops.push(Op::SetFillColor { col: Color::Rgb(color_text_dark.clone()) });

    let mut row_num = 1;

    // Zeile 1: Übernachtung
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + 2.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format!("{:02}", row_num))], font: BuiltinFont::Helvetica });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + 2.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(format!("{} ({} - {})", room_name, checkin_de, checkout_de))],
        font: BuiltinFont::Helvetica
    });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + 2.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(grundpreis / naechte as f64))], font: BuiltinFont::Helvetica });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + col_widths[2] + 2.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format!("{}", naechte))], font: BuiltinFont::Helvetica });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + col_widths[2] + col_widths[3] + 2.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(grundpreis))], font: BuiltinFont::Helvetica });

    content_y -= 6.0;
    row_num += 1;

    // Zeile 2: Zusatzleistungen
    if services_preis > 0.01 {
        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format!("{:02}", row_num))], font: BuiltinFont::Helvetica });

        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("Zusatzleistungen".to_string())], font: BuiltinFont::Helvetica });

        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(services_preis))], font: BuiltinFont::Helvetica });

        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + col_widths[2] + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("1".to_string())], font: BuiltinFont::Helvetica });

        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + col_widths[2] + col_widths[3] + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(services_preis))], font: BuiltinFont::Helvetica });

        content_y -= 6.0;
        row_num += 1;
    }

    // Zeile 3: Rabatt
    if rabatt_preis.abs() > 0.01 {
        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format!("{:02}", row_num))], font: BuiltinFont::Helvetica });

        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("Rabatt".to_string())], font: BuiltinFont::Helvetica });

        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(rabatt_preis))], font: BuiltinFont::Helvetica });

        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + col_widths[2] + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("1".to_string())], font: BuiltinFont::Helvetica });

        ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(table_x + col_widths[0] + col_widths[1] + col_widths[2] + col_widths[3] + 2.0), y: mm_to_pt(content_y) } });
        ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(rabatt_preis))], font: BuiltinFont::Helvetica });

        content_y -= 6.0;
    }

    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    // Trennlinie
    ops.push(Op::SetOutlineColor { col: Color::Rgb(color_border.clone()) });
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint { p: Point { x: mm_to_pt(table_x), y: mm_to_pt(content_y) }, bezier: false },
                LinePoint { p: Point { x: mm_to_pt(table_x + table_width), y: mm_to_pt(content_y) }, bezier: false },
            ],
            is_closed: false,
        },
    });

    content_y -= 8.0;

    // ========================================================================
    // SUMMEN-BLOCK (grauer Balken)
    // ========================================================================
    let summary_x = table_x - 25.0;
    let summary_width = table_width + 25.0;
    let summary_height = 18.0;

    ops.push(Op::SetFillColor { col: Color::Rgb(color_table_header.clone()) });
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    LinePoint { p: Point { x: mm_to_pt(summary_x), y: mm_to_pt(content_y - summary_height + 2.0) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(summary_x + summary_width), y: mm_to_pt(content_y - summary_height + 2.0) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(summary_x + summary_width), y: mm_to_pt(content_y) }, bezier: false },
                    LinePoint { p: Point { x: mm_to_pt(summary_x), y: mm_to_pt(content_y) }, bezier: false },
                ],
            }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    });

    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFillColor { col: Color::Rgb(color_text_dark.clone()) });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(9.0), font: BuiltinFont::Helvetica });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(summary_x + 2.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("Zwischensumme".to_string())], font: BuiltinFont::Helvetica });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(summary_x + 30.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(netto))], font: BuiltinFont::Helvetica });

    content_y -= 5.0;

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(summary_x + 2.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format!("MwSt. {}%", mwst_satz as i32))], font: BuiltinFont::Helvetica });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(summary_x + 30.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(mwst_betrag))], font: BuiltinFont::Helvetica });

    content_y -= 7.0;

    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(11.0), font: BuiltinFont::HelveticaBold });
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(summary_x + 2.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("Gesamtbetrag".to_string())], font: BuiltinFont::HelveticaBold });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(summary_x + 30.0), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text(format_price(gesamtpreis))], font: BuiltinFont::HelveticaBold });

    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    // ========================================================================
    // ZAHLUNGSBEDINGUNGEN (unten mittig im weißen Bereich)
    // ========================================================================
    content_y = 65.0;
    let terms_x = content_x + 10.0;

    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFillColor { col: Color::Rgb(color_text_dark.clone()) });

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(terms_x), y: mm_to_pt(content_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(9.0), font: BuiltinFont::HelveticaBold });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("ZAHLUNGSBEDINGUNGEN".to_string())], font: BuiltinFont::HelveticaBold });

    content_y -= 5.5;

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(terms_x), y: mm_to_pt(content_y) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(7.5), font: BuiltinFont::Helvetica });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(format!(
            "Der Betrag ist {} Tage nach Erhalt dieser Rechnung zur Zahlung fällig.",
            payment.payment_due_days
        ))],
        font: BuiltinFont::Helvetica
    });

    content_y -= 4.0;

    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(terms_x), y: mm_to_pt(content_y) } });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(format!("Bitte unter Angabe der Rechnungsnummer '{}' überweisen.", reservierungsnummer))],
        font: BuiltinFont::Helvetica
    });

    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    // Unterschrift (rechts unten)
    let signature_x = terms_x + 65.0;
    let signature_y = 50.0;

    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor { pos: Point { x: mm_to_pt(signature_x), y: mm_to_pt(signature_y + 2.0) } });
    ops.push(Op::SetFontSizeBuiltinFont { size: Pt(9.0), font: BuiltinFont::HelveticaBold });
    ops.push(Op::WriteTextBuiltinFont { items: vec![TextItem::Text("Unterschrift".to_string())], font: BuiltinFont::HelveticaBold });
    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    ops.push(Op::SetOutlineColor { col: Color::Rgb(color_border.clone()) });
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint { p: Point { x: mm_to_pt(signature_x), y: mm_to_pt(signature_y) }, bezier: false },
                LinePoint { p: Point { x: mm_to_pt(signature_x + 35.0), y: mm_to_pt(signature_y) }, bezier: false },
            ],
            is_closed: false,
        },
    });

    // ========================================================================
    // PDF SPEICHERN
    // ========================================================================
    // ========================================================================
    // ANALYSE: Zähle wie viele Text-Ops tatsächlich im PDF sind (VOR dem Move!)
    // ========================================================================
    let ops_count = ops.len();
    let text_ops_count = ops.iter().filter(|op| matches!(op, Op::WriteTextBuiltinFont { .. })).count();
    let font_size_ops = ops.iter().filter(|op| matches!(op, Op::SetFontSizeBuiltinFont { .. })).count();
    let cursor_ops = ops.iter().filter(|op| matches!(op, Op::SetTextCursor { .. })).count();
    let polygon_ops = ops.iter().filter(|op| matches!(op, Op::DrawPolygon { .. })).count();
    let start_text_sections = ops.iter().filter(|op| matches!(op, Op::StartTextSection)).count();
    let end_text_sections = ops.iter().filter(|op| matches!(op, Op::EndTextSection)).count();

    println!("🔧 [PDF DEBUG] ===== OPERATIONS GESAMT: {} =====", ops_count);
    println!("📊 [FINAL DEBUG] Ops-Analyse:");
    println!("   - Gesamt Ops: {}", ops_count);
    println!("   - WriteTextBuiltinFont: {}", text_ops_count);
    println!("   - SetFontSizeBuiltinFont: {}", font_size_ops);
    println!("   - SetTextCursor: {}", cursor_ops);
    println!("   - DrawPolygon: {}", polygon_ops);
    println!("   - StartTextSection: {}", start_text_sections);
    println!("   - EndTextSection: {}", end_text_sections);
    println!("🔧 [PDF DEBUG] Erstelle PdfPage mit {} Operationen", ops_count);

    let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);

    println!("🔧 [PDF DEBUG] PdfPage erstellt, speichere mit with_pages()");

    let pdf_bytes = doc
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    println!("🔧 [PDF DEBUG] PDF Bytes generiert: {} bytes", pdf_bytes.len());

    std::fs::write(output_path, pdf_bytes)
        .map_err(|e| format!("Fehler beim Schreiben der PDF-Datei: {}", e))?;

    println!("✅ Moderne PDF-Rechnung mit Sidebar erstellt (printpdf 0.8): {:?}", output_path);
    println!("📋 CRITICAL INFO: {} WriteTextBuiltinFont ops wurden hinzugefügt, aber nur wenige erscheinen im PDF!", text_ops_count);

    Ok(output_path.clone())
}

// ============================================================================
// HILFSFUNKTIONEN
// ============================================================================

/// Lädt ein PNG/JPG Logo als RawImage (printpdf 0.8 API)
/// WICHTIG: Benötigt printpdf features = ["png", "jpeg"]
fn load_logo_as_raw_image(logo_path: &str) -> Result<RawImage, String> {
    println!("🖼️  [LOGO DEBUG] ===== LOGO LADEN START =====");
    println!("🖼️  [LOGO DEBUG] Pfad: {}", logo_path);

    // Prüfe ob Logo existiert
    if !std::path::Path::new(logo_path).exists() {
        println!("❌ [LOGO DEBUG] Datei nicht gefunden!");
        return Err(format!("Logo-Datei nicht gefunden: {}", logo_path));
    }

    println!("✅ [LOGO DEBUG] Datei existiert");

    // Lade Image-Bytes
    let image_bytes = std::fs::read(logo_path)
        .map_err(|e| format!("Fehler beim Lesen der Logo-Datei: {}", e))?;

    println!("✅ [LOGO DEBUG] {} Bytes gelesen", image_bytes.len());

    // Dekodiere mit printpdf (mit aktivierten png/jpeg features)
    let raw_image = RawImage::decode_from_bytes(&image_bytes, &mut Vec::new())
        .map_err(|e| format!("Fehler beim Dekodieren der Bilddatei: {}", e))?;

    println!("✅ [LOGO DEBUG] RawImage erfolgreich dekodiert");

    Ok(raw_image)
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
        let pdf_path = temp_dir.join("test_invoice_sidebar_v08.pdf");

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

        println!("Test-PDF mit printpdf 0.8 erstellt: {:?}", pdf_path);
    }
}
