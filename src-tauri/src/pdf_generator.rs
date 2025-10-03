use crate::database::BookingWithDetails;
use chrono::Local;
use tauri::AppHandle;
use pdf_writer::{Pdf, Ref, Content, Str, Name, Filter, Rect, Finish};
use pdf_writer::types::ColorSpaceOperand;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::Write;

const MM_TO_PT: f32 = 2.834645669;
const PAGE_WIDTH_PT: f32 = 595.0;
const PAGE_HEIGHT_PT: f32 = 842.0;
const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;

// Layout EXAKT wie HTML-Template
const GRAY_STRIP_MM: f32 = 21.2; // 60px konstant für Details/Table/Totals
const CONTENT_MARGIN_LEFT_MM: f32 = 35.3; // 100px padding-left
const SIDEBAR_WIDTH_PCT: f32 = 0.40; // Nur Header/Footer: 40%

fn mm_to_pt(mm: f32) -> f32 { mm * MM_TO_PT }
fn y_from_top(y_mm: f32) -> f32 { PAGE_HEIGHT_MM - y_mm }

fn write_text(content: &mut Content, text: &str, x_mm: f32, y_mm_top: f32, font_size: f32, font: Name, r: f32, g: f32, b: f32) {
    content.set_fill_color_space(ColorSpaceOperand::Named(Name(b"DeviceRGB")));
    content.set_fill_color([r, g, b]);
    content.set_font(font, font_size);
    content.set_text_matrix([1.0, 0.0, 0.0, 1.0, mm_to_pt(x_mm), mm_to_pt(y_from_top(y_mm_top))]);
    content.show(Str(text.as_bytes()));
}

fn text_black(content: &mut Content, text: &str, x_mm: f32, y_mm_top: f32, font_size: f32, font: Name) {
    write_text(content, text, x_mm, y_mm_top, font_size, font, 0.0, 0.0, 0.0);
}

fn text_white(content: &mut Content, text: &str, x_mm: f32, y_mm_top: f32, font_size: f32, font: Name) {
    write_text(content, text, x_mm, y_mm_top, font_size, font, 1.0, 1.0, 1.0);
}

fn draw_rect(content: &mut Content, x_mm: f32, y_mm_top: f32, width_mm: f32, height_mm: f32, r: f32, g: f32, b: f32) {
    let y_bottom = y_from_top(y_mm_top + height_mm);
    content.set_fill_color_space(ColorSpaceOperand::Named(Name(b"DeviceRGB")));
    content.set_fill_color([r, g, b]);
    content.rect(mm_to_pt(x_mm), mm_to_pt(y_bottom), mm_to_pt(width_mm), mm_to_pt(height_mm));
    content.fill_nonzero();
}

fn draw_line(content: &mut Content, x1_mm: f32, y1_mm_top: f32, x2_mm: f32, y2_mm_top: f32, r: f32, g: f32, b: f32, width_pt: f32) {
    content.set_stroke_color_space(ColorSpaceOperand::Named(Name(b"DeviceRGB")));
    content.set_stroke_color([r, g, b]);
    content.set_line_width(width_pt);
    content.move_to(mm_to_pt(x1_mm), mm_to_pt(y_from_top(y1_mm_top)));
    content.line_to(mm_to_pt(x2_mm), mm_to_pt(y_from_top(y2_mm_top)));
    content.stroke();
}

#[tauri::command]
pub fn generate_invoice_pdf(
    app: AppHandle,
    booking: BookingWithDetails,
) -> Result<String, String> {
    use tauri::Manager;

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Fehler: {}", e))?;

    let invoices_dir = app_data_dir.join("invoices");
    std::fs::create_dir_all(&invoices_dir)
        .map_err(|e| format!("Fehler: {}", e))?;

    let file_name = format!("Rechnung_{}.pdf", booking.reservierungsnummer);
    let output_path = invoices_dir.join(&file_name);

    let logo_path = app_data_dir.join("logos").join("dpolg_logo.png");
    let logo_exists = logo_path.exists();

    let mut pdf = Pdf::new();

    let catalog_id = Ref::new(1);
    let pages_id = Ref::new(2);
    let page_id = Ref::new(3);
    let content_id = Ref::new(4);
    let helvetica_id = Ref::new(5);
    let helvetica_bold_id = Ref::new(6);
    let mut next_ref = 7;

    let image_id = if logo_exists {
        let img_id = Ref::new(next_ref);
        next_ref += 1;

        let image_data = std::fs::read(&logo_path)
            .map_err(|e| format!("Logo-Fehler: {}", e))?;
        let dynamic_image = image::load_from_memory(&image_data)
            .map_err(|e| format!("Logo dekodieren: {}", e))?;
        let rgb_image = dynamic_image.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&rgb_image.into_raw())
            .map_err(|e| format!("Komprimierung: {}", e))?;
        let compressed = encoder.finish()
            .map_err(|e| format!("Komprimierung: {}", e))?;

        let mut image_obj = pdf.image_xobject(img_id, &compressed);
        image_obj.filter(Filter::FlateDecode);
        image_obj.width(width as i32);
        image_obj.height(height as i32);
        image_obj.color_space().device_rgb();
        image_obj.bits_per_component(8);
        image_obj.finish();

        Some(img_id)
    } else {
        None
    };

    pdf.type1_font(helvetica_id).base_font(Name(b"Helvetica"));
    pdf.type1_font(helvetica_bold_id).base_font(Name(b"Helvetica-Bold"));

    pdf.catalog(catalog_id).pages(pages_id);
    pdf.pages(pages_id).kids([page_id]).count(1);

    {
        let mut page = pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, PAGE_WIDTH_PT, PAGE_HEIGHT_PT));
        page.parent(pages_id);
        page.contents(content_id);

        let mut resources = page.resources();
        let mut fonts = resources.fonts();
        fonts.pair(Name(b"F1"), helvetica_id);
        fonts.pair(Name(b"F2"), helvetica_bold_id);
        fonts.finish();

        if image_id.is_some() {
            resources.x_objects().pair(Name(b"Im1"), image_id.unwrap());
        }
    }

    let mut content = Content::new();

    // Farben aus HTML
    const DARK_GRAY: [f32; 3] = [0.227, 0.227, 0.227]; // #3a3a3a
    const LIGHT_GRAY_BG: [f32; 3] = [0.910, 0.910, 0.910]; // #e8e8e8
    const LIGHT_BG: [f32; 3] = [0.980, 0.980, 0.980]; // #fafafa
    const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

    // HEADER - 40% Sidebar (dunkel) + 60% weiß
    let header_height = 70.0;
    let sidebar_width = PAGE_WIDTH_MM * SIDEBAR_WIDTH_PCT; // 40% = 84mm

    draw_rect(&mut content, 0.0, 0.0, sidebar_width, header_height, DARK_GRAY[0], DARK_GRAY[1], DARK_GRAY[2]);
    draw_rect(&mut content, sidebar_width, 0.0, PAGE_WIDTH_MM - sidebar_width, header_height, WHITE[0], WHITE[1], WHITE[2]);

    content.begin_text();

    // Logo (mit weißem Hintergrund über Sidebar)
    if image_id.is_some() {
        content.end_text();
        draw_rect(&mut content, 5.0, 5.0, sidebar_width - 10.0, 30.0, WHITE[0], WHITE[1], WHITE[2]);

        let logo_width_mm = 60.0;
        let logo_height_mm = 30.0;
        let logo_x = (sidebar_width - logo_width_mm) / 2.0;
        let logo_y_top = 10.0;

        content.save_state();
        content.transform([
            mm_to_pt(logo_width_mm), 0.0, 0.0,
            mm_to_pt(logo_height_mm),
            mm_to_pt(logo_x),
            mm_to_pt(y_from_top(logo_y_top + logo_height_mm))
        ]);
        content.x_object(Name(b"Im1"));
        content.restore_state();
        content.begin_text();
    }

    // RECHNUNG AN
    text_white(&mut content, "RECHNUNG AN", 10.0, 42.0, 9.0, Name(b"F2"));

    let guest_name = format!("{} {}", booking.guest.vorname, booking.guest.nachname);
    text_white(&mut content, &guest_name, 10.0, 50.0, 11.0, Name(b"F2"));

    if let Some(strasse) = &booking.guest.strasse {
        text_white(&mut content, strasse, 10.0, 58.0, 9.0, Name(b"F1"));
    }
    let ort = format!("{} {}",
        booking.guest.plz.as_deref().unwrap_or(""),
        booking.guest.ort.as_deref().unwrap_or("")
    ).trim().to_string();
    if !ort.is_empty() {
        text_white(&mut content, &ort, 10.0, 64.0, 9.0, Name(b"F1"));
    }

    // RECHNUNG Titel (rechts, 48px = 32pt)
    text_black(&mut content, "RECHNUNG", sidebar_width + 20.0, 30.0, 32.0, Name(b"F2"));

    content.end_text();

    // INVOICE DETAILS - 60px grauer Streifen + Rest #fafafa
    let details_y = header_height;
    let details_height = 28.0;

    draw_rect(&mut content, 0.0, details_y, GRAY_STRIP_MM, details_height, DARK_GRAY[0], DARK_GRAY[1], DARK_GRAY[2]);
    draw_rect(&mut content, GRAY_STRIP_MM, details_y, PAGE_WIDTH_MM - GRAY_STRIP_MM, details_height, LIGHT_BG[0], LIGHT_BG[1], LIGHT_BG[2]);

    content.begin_text();

    text_black(&mut content, "Rechnungsdetails", CONTENT_MARGIN_LEFT_MM, details_y + 6.0, 14.0, Name(b"F2"));

    let rechnungsdatum = Local::now().format("%d.%m.%Y").to_string();
    let faellig_am = (Local::now() + chrono::Duration::days(14)).format("%d.%m.%Y").to_string();

    let meta_y = details_y + 17.0;
    let meta_x = CONTENT_MARGIN_LEFT_MM;

    text_black(&mut content, &format!("Rechnungsdatum: {}", rechnungsdatum), meta_x, meta_y, 9.5, Name(b"F1"));
    text_black(&mut content, &format!("Faelligkeitsdatum: {}", faellig_am), meta_x + 65.0, meta_y, 9.5, Name(b"F1"));
    text_black(&mut content, &format!("Rechnungsnr.: {}", booking.reservierungsnummer), meta_x + 142.0, meta_y, 9.5, Name(b"F1"));

    content.end_text();

    draw_line(&mut content, GRAY_STRIP_MM, details_y + details_height - 0.5, PAGE_WIDTH_MM, details_y + details_height - 0.5, 0.88, 0.88, 0.88, 0.5);

    // TABLE SECTION - 60px grauer Streifen + weiß
    let table_y = details_y + details_height;
    let table_header_height = 12.0;
    let row_height = 15.0;

    // Grauer Streifen für gesamte Table Section
    let table_total_height = 90.0;
    draw_rect(&mut content, 0.0, table_y, GRAY_STRIP_MM, table_total_height, DARK_GRAY[0], DARK_GRAY[1], DARK_GRAY[2]);

    // Table content Bereich weiß
    draw_rect(&mut content, GRAY_STRIP_MM, table_y, PAGE_WIDTH_MM - GRAY_STRIP_MM, table_total_height, WHITE[0], WHITE[1], WHITE[2]);

    // Table Header (hellgrau)
    let table_x = CONTENT_MARGIN_LEFT_MM;
    let table_width = PAGE_WIDTH_MM - CONTENT_MARGIN_LEFT_MM - 12.0;
    draw_rect(&mut content, table_x, table_y, table_width, table_header_height, LIGHT_GRAY_BG[0], LIGHT_GRAY_BG[1], LIGHT_GRAY_BG[2]);

    content.begin_text();

    // Spalten (HTML: 10%, 45%, 15%, 10%, 20%)
    let col_widths = [table_width * 0.10, table_width * 0.45, table_width * 0.15, table_width * 0.10, table_width * 0.20];
    let headers = ["NR", "PRODUKTBESCHREIBUNG", "EINZELPREIS", "MENGE", "GESAMT"];

    let header_y = table_y + 7.5;
    let mut x = table_x + 3.0;
    for (i, header) in headers.iter().enumerate() {
        text_black(&mut content, header, x, header_y, 8.0, Name(b"F2"));
        x += col_widths[i];
    }

    draw_line(&mut content, table_x, table_y + table_header_height, table_x + table_width, table_y + table_header_height, 0.82, 0.82, 0.82, 0.8);

    // Tabellenzeile 1
    let row1_y = table_y + table_header_height + 5.0;
    x = table_x + 3.0;

    text_black(&mut content, "01", x, row1_y, 9.5, Name(b"F1"));
    x += col_widths[0];

    let beschreibung = format!("Unterkunft {} vom {} bis {}",
        booking.room.name,
        booking.checkin_date,
        booking.checkout_date
    );
    text_black(&mut content, &beschreibung, x, row1_y, 9.0, Name(b"F1"));
    x += col_widths[1];

    let preis_pro_nacht = booking.grundpreis / booking.anzahl_naechte as f64;
    text_black(&mut content, &format!("{:.2} EUR", preis_pro_nacht), x, row1_y, 9.5, Name(b"F1"));
    x += col_widths[2];

    text_black(&mut content, &format!("{}", booking.anzahl_naechte), x + 5.0, row1_y, 9.5, Name(b"F1"));
    x += col_widths[3];

    text_black(&mut content, &format!("{:.2} EUR", booking.grundpreis), x, row1_y, 9.5, Name(b"F1"));

    content.end_text();

    draw_line(&mut content, table_x, row1_y + row_height - 5.0, table_x + table_width, row1_y + row_height - 5.0, 0.88, 0.88, 0.88, 0.5);

    // TOTALS SECTION - innerhalb Table Wrapper (grau/weiß Hintergrund schon da)
    let totals_y = row1_y + row_height + 3.0;
    draw_rect(&mut content, GRAY_STRIP_MM, totals_y, PAGE_WIDTH_MM - GRAY_STRIP_MM, 40.0, 0.96, 0.96, 0.96);

    content.begin_text();

    // Summen rechtsbündig (300px = 106mm)
    let totals_width = 106.0;
    let totals_x_end = PAGE_WIDTH_MM - 15.0;
    let totals_x_label = totals_x_end - totals_width;
    let totals_x_value = totals_x_end - 35.0;

    let mut current_y = totals_y + 10.0;

    text_black(&mut content, "Zwischensumme", totals_x_label, current_y, 9.5, Name(b"F1"));
    text_black(&mut content, &format!("{:.2} EUR", booking.grundpreis + booking.services_preis), totals_x_value, current_y, 9.5, Name(b"F1"));

    if booking.rabatt_preis > 0.0 {
        current_y += 8.0;
        text_black(&mut content, "Rabatt", totals_x_label, current_y, 9.5, Name(b"F1"));
        text_black(&mut content, &format!("-{:.2} EUR", booking.rabatt_preis), totals_x_value, current_y, 9.5, Name(b"F1"));
    }

    current_y += 13.0;
    content.end_text();
    draw_line(&mut content, totals_x_label, current_y - 3.0, totals_x_end, current_y - 3.0, 0.2, 0.2, 0.2, 1.8);
    content.begin_text();

    text_black(&mut content, "Gesamtsumme", totals_x_label, current_y, 12.0, Name(b"F2"));
    text_black(&mut content, &format!("{:.2} EUR", booking.gesamtpreis), totals_x_value, current_y, 12.0, Name(b"F2"));

    content.end_text();

    // FOOTER - 40% Sidebar + 60% weiß (wie Header)
    let footer_y = PAGE_HEIGHT_MM - 85.0;
    let footer_height = 85.0;

    draw_rect(&mut content, 0.0, footer_y, sidebar_width, footer_height, DARK_GRAY[0], DARK_GRAY[1], DARK_GRAY[2]);
    draw_rect(&mut content, sidebar_width, footer_y, PAGE_WIDTH_MM - sidebar_width, footer_height, WHITE[0], WHITE[1], WHITE[2]);

    content.begin_text();

    text_white(&mut content, "Vielen Dank", 10.0, footer_y + 10.0, 18.0, Name(b"F2"));

    text_white(&mut content, "ZAHLUNGSINFORMATIONEN", 10.0, footer_y + 32.0, 9.0, Name(b"F2"));
    text_white(&mut content, "IBAN: DE70 7009 0500 0001 9999 90", 10.0, footer_y + 41.0, 8.5, Name(b"F1"));
    text_white(&mut content, "BIC: GENODEF1S04", 10.0, footer_y + 48.0, 8.5, Name(b"F1"));
    text_white(&mut content, "Bank: Sparda Bank Muenchen", 10.0, footer_y + 55.0, 8.5, Name(b"F1"));

    text_white(&mut content, "+49 8042 9725-20", 10.0, footer_y + 65.0, 8.5, Name(b"F1"));
    text_white(&mut content, "info@dpolg-stiftung.de", 10.0, footer_y + 72.0, 8.5, Name(b"F1"));
    text_white(&mut content, "www.dpolg-stiftung.de", 10.0, footer_y + 79.0, 8.5, Name(b"F1"));

    text_black(&mut content, "GESCHAEFTSBEDINGUNGEN", sidebar_width + 10.0, footer_y + 10.0, 9.5, Name(b"F2"));
    text_black(&mut content, "Zahlbar innerhalb von 14 Tagen ohne Abzug.", sidebar_width + 10.0, footer_y + 22.0, 8.5, Name(b"F1"));

    content.end_text();

    pdf.stream(content_id, &content.finish());
    let pdf_bytes = pdf.finish();
    std::fs::write(&output_path, pdf_bytes)
        .map_err(|e| format!("Fehler beim Speichern: {}", e))?;

    println!("✅ PDF erstellt: {:?}", output_path);
    Ok(output_path.to_string_lossy().to_string())
}
