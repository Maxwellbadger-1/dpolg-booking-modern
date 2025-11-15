use crate::database::BookingWithDetails;
use tauri::AppHandle;
use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::types::PrintToPdfOptions;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// ⚡ PERFORMANCE FIX: Browser-Pool - Chrome-Instanz wird einmalig gestartet und wiederverwendet
// Vorher: 20+ Sekunden (jedes Mal neuer Browser-Start)
// Nachher: < 1 Sekunde (Browser bleibt im Speicher)
static BROWSER_POOL: Lazy<Mutex<Option<Browser>>> = Lazy::new(|| {
    println!("🚀 [BROWSER POOL] Initializing browser pool...");
    Mutex::new(None)
});

fn get_or_create_browser() -> Result<Browser, String> {
    let mut pool = BROWSER_POOL.lock().map_err(|e| format!("Browser pool lock error: {}", e))?;

    // Prüfe ob Browser bereits existiert und noch läuft
    if let Some(ref browser) = *pool {
        println!("♻️  [BROWSER POOL] Reusing existing browser instance");
        // Clone ist günstig, da Browser intern Arc verwendet
        return Ok(browser.clone());
    }

    // Browser noch nicht gestartet - erstmalig initialisieren
    println!("🌐 [BROWSER POOL] Starting new browser instance (first time)...");

    // 🏗️ CI-KOMPATIBILITÄT: --no-sandbox Flag für GitHub Actions
    // Chrome braucht --no-sandbox in containerisierten CI-Umgebungen
    let is_ci = std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("RUST_CI").is_ok();

    if is_ci {
        println!("🏗️  [CI MODE] Running in CI environment, using --no-sandbox");
    }

    let launch_options = LaunchOptions::default_builder()
        .headless(true)
        .sandbox(!is_ci)  // Sandbox OFF in CI, ON in Production
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

    println!("✅ [BROWSER POOL] Browser started and cached");
    *pool = Some(browser.clone());
    Ok(browser)
}

#[tauri::command]
pub fn generate_invoice_pdf_html(
    app: AppHandle,
    booking: BookingWithDetails,
) -> Result<String, String> {
    use tauri::Manager;

    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  HTML→PDF GENERATOR (Browser Pool Optimized)        │");
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

    // 1. Neues HTML-Template System verwenden
    println!("📝 Generating invoice HTML with new template system...");
    let html = crate::invoice_html::generate_invoice_html(booking.id)
        .map_err(|e| {
            eprintln!("❌ Invoice HTML generation failed: {}", e);
            format!("HTML-Generierung Fehler: {}", e)
        })?;
    println!("✅ Invoice HTML generated successfully");

    // 4. HTML in temp file schreiben
    let temp_html_path = invoices_dir.join(format!("temp_invoice_{}.html", booking.reservierungsnummer));
    std::fs::write(&temp_html_path, &html)
        .map_err(|e| format!("Fehler HTML schreiben: {}", e))?;

    // 5. ⚡ Browser aus Pool holen (wiederverwendet = SCHNELL!)
    println!("♻️  Getting browser from pool...");
    let browser = get_or_create_browser()?;
    println!("✅ Browser ready");

    // Versuche Tab zu erstellen - wenn Verbindung geschlossen, Browser neu starten
    let tab = match browser.new_tab() {
        Ok(t) => t,
        Err(_e) => {
            println!("⚠️  Browser connection closed, resetting pool...");
            // Browser-Pool zurücksetzen
            let mut pool = BROWSER_POOL.lock().map_err(|e| format!("Browser pool lock error: {}", e))?;
            *pool = None;
            drop(pool);

            // Neuen Browser starten
            println!("🔄 Restarting browser...");
            let new_browser = get_or_create_browser()?;
            new_browser.new_tab()
                .map_err(|e| format!("Chrome tab Fehler nach Neustart: {}", e))?
        }
    };

    // 6. HTML laden
    let file_url = format!("file://{}", temp_html_path.display());
    tab.navigate_to(&file_url)
        .map_err(|e| format!("Chrome navigate Fehler: {}", e))?;

    tab.wait_until_navigated()
        .map_err(|e| format!("Chrome wait Fehler: {}", e))?;

    // Reduzierte Wartezeit für Font-Rendering (von 500ms auf 200ms)
    std::thread::sleep(std::time::Duration::from_millis(200));

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
