// Wrapper für neuen HTML-PDF Generator
// Behält alte Command-Names für Kompatibilität mit Frontend

use crate::database::{get_booking_with_details_by_id, BookingWithDetails};
use crate::pdf_generator_html::generate_invoice_pdf_html;
use tauri::AppHandle;
use std::path::PathBuf;

#[derive(serde::Serialize)]
pub struct InvoicePdf {
    pub filename: String,
    pub path: String,
    pub size: u64,
}

#[tauri::command]
pub fn generate_invoice_pdf_command(
    app: AppHandle,
    booking_id: i64,
) -> Result<String, String> {
    println!("════════════════════════════════════════════════════════");
    println!("🔵 PDF GENERATION STARTED");
    println!("📋 Booking ID: {}", booking_id);
    println!("════════════════════════════════════════════════════════");

    // Booking-Daten laden
    println!("🔍 Loading booking data...");
    let booking = get_booking_with_details_by_id(booking_id)
        .map_err(|e| {
            eprintln!("❌ ERROR loading booking: {}", e);
            e.to_string()
        })?;

    println!("✅ Booking loaded: {}", booking.reservierungsnummer);
    println!("   Guest: {} {}", booking.guest.vorname, booking.guest.nachname);
    println!("   Room: {}", booking.room.name);
    println!("   Dates: {} - {}", booking.checkin_date, booking.checkout_date);

    // Neuen HTML-PDF Generator aufrufen
    println!("🚀 Calling HTML-PDF generator...");
    let result = generate_invoice_pdf_html(app, booking);

    match &result {
        Ok(path) => {
            println!("✅ PDF SUCCESSFULLY GENERATED!");
            println!("📄 Path: {}", path);
            println!("════════════════════════════════════════════════════════");
        }
        Err(e) => {
            eprintln!("❌ PDF GENERATION FAILED!");
            eprintln!("Error: {}", e);
            eprintln!("════════════════════════════════════════════════════════");
        }
    }

    result
}

#[tauri::command]
pub fn get_invoice_pdfs_for_booking_command(
    app: AppHandle,
    booking_id: i64,
) -> Result<Vec<InvoicePdf>, String> {
    use tauri::Manager;

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Fehler: {}", e))?;

    let invoices_dir = app_data_dir.join("invoices");

    if !invoices_dir.exists() {
        return Ok(vec![]);
    }

    let booking = get_booking_with_details_by_id(booking_id)
        .map_err(|e| e.to_string())?;

    let pattern = format!("Rechnung_{}", booking.reservierungsnummer);

    let mut pdfs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&invoices_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with(&pattern) && filename_str.ends_with(".pdf") {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        pdfs.push(InvoicePdf {
                            filename: filename_str.to_string(),
                            path: path.to_string_lossy().to_string(),
                            size: metadata.len(),
                        });
                    }
                }
            }
        }
    }

    Ok(pdfs)
}

#[tauri::command]
pub fn open_invoices_folder_command(app: AppHandle) -> Result<(), String> {
    use tauri::Manager;

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Fehler: {}", e))?;

    let invoices_dir = app_data_dir.join("invoices");

    std::fs::create_dir_all(&invoices_dir)
        .map_err(|e| format!("Fehler: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&invoices_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&invoices_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&invoices_dir)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_pdf_file_command(file_path: String) -> Result<(), String> {
    println!("📂 Opening PDF file: {}", file_path);

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", &file_path])
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    }

    println!("✅ PDF file opened successfully");
    Ok(())
}

// NEUE FUNKTION: PDF generieren UND per Email versenden
#[tauri::command]
pub async fn generate_and_send_invoice_command(
    app: AppHandle,
    booking_id: i64,
) -> Result<String, String> {
    println!("════════════════════════════════════════════════════════");
    println!("📧 PDF GENERATION & EMAIL STARTED");
    println!("📋 Booking ID: {}", booking_id);
    println!("════════════════════════════════════════════════════════");

    // 1. PDF generieren
    println!("📄 Step 1: Generating PDF...");
    let pdf_path = generate_invoice_pdf_command(app, booking_id)?;
    println!("✅ PDF generated: {}", pdf_path);

    // 2. Email mit PDF-Anhang versenden
    println!("📧 Step 2: Sending email with PDF attachment...");
    use std::path::PathBuf;
    let pdf_pathbuf = PathBuf::from(pdf_path);
    crate::email::send_invoice_email_with_pdf(booking_id, pdf_pathbuf).await?;
    println!("✅ Email sent successfully!");

    println!("════════════════════════════════════════════════════════");
    Ok("Rechnung erfolgreich erstellt und per Email versendet".to_string())
}
