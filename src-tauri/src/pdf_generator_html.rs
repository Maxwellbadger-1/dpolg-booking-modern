use crate::database::BookingWithDetails;
use chrono::Local;
use tauri::AppHandle;
use tera::{Tera, Context};
use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::types::PrintToPdfOptions;
use base64::Engine;

#[tauri::command]
pub fn generate_invoice_pdf_html(
    app: AppHandle,
    booking: BookingWithDetails,
) -> Result<String, String> {
    use tauri::Manager;

    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  HTML→PDF GENERATOR                                 │");
    println!("└─────────────────────────────────────────────────────┘");

    println!("📁 Getting app data directory...");
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| {
            eprintln!("❌ Failed to get app_data_dir: {}", e);
            format!("Fehler app_data_dir: {}", e)
        })?;
    println!("✅ App data dir: {:?}", app_data_dir);

    let invoices_dir = app_data_dir.join("invoices");
    println!("📁 Creating invoices directory: {:?}", invoices_dir);
    std::fs::create_dir_all(&invoices_dir)
        .map_err(|e| {
            eprintln!("❌ Failed to create directory: {}", e);
            format!("Fehler create_dir: {}", e)
        })?;
    println!("✅ Directory ready");

    // 1. Tera Template Engine initialisieren
    println!("📝 Initializing Tera template engine...");
    let current_dir = std::env::current_dir()
        .map_err(|e| {
            eprintln!("❌ Failed to get current_dir: {}", e);
            format!("Fehler current_dir: {}", e)
        })?;
    println!("   Current dir: {:?}", current_dir);

    // Check if we're already in src-tauri directory
    let template_path = if current_dir.ends_with("src-tauri") {
        current_dir.join("templates/**/*.html")
    } else {
        current_dir.join("src-tauri/templates/**/*.html")
    };
    println!("   Template path: {:?}", template_path);

    let tera = Tera::new(template_path.to_str().unwrap())
        .map_err(|e| {
            eprintln!("❌ Tera initialization failed: {}", e);
            format!("Tera Fehler: {}", e)
        })?;
    println!("✅ Tera initialized");

    // 2. Context mit Booking-Daten befüllen
    let mut context = Context::new();
    context.insert("booking", &booking);
    context.insert("guest", &booking.guest);
    context.insert("room", &booking.room);

    // Berechnete Werte
    let price_per_night = if booking.anzahl_naechte > 0 {
        booking.grundpreis / booking.anzahl_naechte as f64
    } else {
        0.0
    };
    context.insert("price_per_night", &format!("{:.2}", price_per_night));

    let subtotal = booking.grundpreis + booking.services_preis;
    context.insert("subtotal", &format!("{:.2}", subtotal));

    // Datum formatieren
    let invoice_date = Local::now().format("%d.%m.%Y").to_string();
    let due_date = (Local::now() + chrono::Duration::days(14)).format("%d.%m.%Y").to_string();
    context.insert("invoice_date", &invoice_date);
    context.insert("due_date", &due_date);

    // Logo als Base64 einbetten
    println!("🖼️ Loading logo...");

    // Versuche zuerst Logo aus Company Settings zu laden
    let logo_base64 = match crate::database::get_company_settings() {
        Ok(settings) if settings.logo_path.is_some() => {
            let logo_path_str = settings.logo_path.unwrap();
            let logo_path = std::path::Path::new(&logo_path_str);
            println!("   Trying company settings logo: {:?}", logo_path);

            if logo_path.exists() {
                match std::fs::read(&logo_path) {
                    Ok(logo_bytes) => {
                        let encoded = base64::engine::general_purpose::STANDARD.encode(&logo_bytes);
                        println!("✅ Company logo loaded ({} bytes)", logo_bytes.len());
                        format!("data:image/png;base64,{}", encoded)
                    }
                    Err(e) => {
                        eprintln!("⚠️ Could not read company logo: {}", e);
                        String::new()
                    }
                }
            } else {
                eprintln!("⚠️ Company logo not found at: {:?}", logo_path);
                String::new()
            }
        }
        _ => {
            // Fallback: Standard-Logo aus icons/icon.png
            println!("   No company logo configured, trying default logo...");
            let logo_path = current_dir.join("icons/icon.png");

            if logo_path.exists() {
                match std::fs::read(&logo_path) {
                    Ok(logo_bytes) => {
                        let encoded = base64::engine::general_purpose::STANDARD.encode(&logo_bytes);
                        println!("✅ Default logo loaded ({} bytes)", logo_bytes.len());
                        format!("data:image/png;base64,{}", encoded)
                    }
                    Err(e) => {
                        eprintln!("⚠️ Could not read default logo: {}", e);
                        String::new()
                    }
                }
            } else {
                eprintln!("⚠️ No logo found");
                String::new()
            }
        }
    };

    context.insert("logo_base64", &logo_base64);

    // 3. Template rendern
    let html = tera.render("invoice.html", &context)
        .map_err(|e| format!("Template render Fehler: {}", e))?;

    // 4. HTML in temp file schreiben
    let temp_html_path = invoices_dir.join(format!("temp_invoice_{}.html", booking.reservierungsnummer));
    std::fs::write(&temp_html_path, &html)
        .map_err(|e| format!("Fehler HTML schreiben: {}", e))?;

    // 5. Headless Chrome starten
    println!("🌐 Starting headless Chrome...");
    let launch_options = LaunchOptions::default_builder()
        .headless(true)
        .build()
        .map_err(|e| {
            eprintln!("❌ Chrome launch options error: {}", e);
            format!("Chrome launch options: {}", e)
        })?;

    let browser = Browser::new(launch_options)
        .map_err(|e| {
            eprintln!("❌ Chrome start error: {}", e);
            format!("Chrome starten Fehler: {}", e)
        })?;
    println!("✅ Chrome started");

    let tab = browser.new_tab()
        .map_err(|e| format!("Chrome tab Fehler: {}", e))?;

    // 6. HTML laden
    let file_url = format!("file://{}", temp_html_path.display());
    tab.navigate_to(&file_url)
        .map_err(|e| format!("Chrome navigate Fehler: {}", e))?;

    tab.wait_until_navigated()
        .map_err(|e| format!("Chrome wait Fehler: {}", e))?;

    // Kurze Wartezeit für Font-Rendering
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 7. PDF generieren
    let pdf_data = tab.print_to_pdf(Some(PrintToPdfOptions {
        landscape: Some(false),
        display_header_footer: Some(false),
        print_background: Some(true),
        scale: Some(1.0),
        paper_width: Some(8.27), // A4 width in inches (210mm)
        paper_height: Some(11.69), // A4 height in inches (297mm)
        margin_top: Some(0.0),
        margin_bottom: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: Some(false),
        transfer_mode: None,
        generate_document_outline: Some(false),
        generate_tagged_pdf: Some(false),
    }))
    .map_err(|e| format!("PDF generieren Fehler: {}", e))?;

    // 8. PDF speichern
    let file_name = format!("Rechnung_{}.pdf", booking.reservierungsnummer);
    let output_path = invoices_dir.join(&file_name);

    std::fs::write(&output_path, pdf_data)
        .map_err(|e| format!("PDF speichern Fehler: {}", e))?;

    // 9. Temp HTML löschen
    std::fs::remove_file(&temp_html_path).ok();

    println!("✅ PDF mit HTML→PDF erstellt: {:?}", output_path);
    Ok(output_path.to_string_lossy().to_string())
}
