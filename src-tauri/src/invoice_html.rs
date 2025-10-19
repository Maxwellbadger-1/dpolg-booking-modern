use crate::database::{
    get_booking_with_details_by_id, get_booking_credit_usage, get_company_settings,
    get_payment_settings, BookingWithDetails, CompanySettings, PaymentSettings,
};
use crate::payment_recipients::get_payment_recipient;
use chrono::NaiveDate;
use qrcode::QrCode;
use image::Luma;
use base64::{Engine as _, engine::general_purpose};

/// Generiert HTML-Rechnung für eine Buchung
#[tauri::command]
pub fn generate_invoice_html(booking_id: i64) -> Result<String, String> {
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  INVOICE HTML GENERATOR                             │");
    println!("│  Booking ID: {}                                    │", booking_id);
    println!("└─────────────────────────────────────────────────────┘");

    // 1. Lade Template
    let template = include_str!("../../invoice_modern_template.html");

    // 2. Lade Buchungsdaten
    let booking = get_booking_with_details_by_id(booking_id)
        .map_err(|e| format!("Fehler beim Laden der Buchung: {}", e))?;

    // 3. Lade Settings
    let company = get_company_settings()
        .map_err(|e| format!("Fehler beim Laden der Firmeneinstellungen: {}", e))?;
    let payment = get_payment_settings()
        .map_err(|e| format!("Fehler beim Laden der Zahlungseinstellungen: {}", e))?;

    // 4. Generiere HTML
    let html = generate_html_from_booking(template, &booking, &company, &payment)?;

    println!("✅ Invoice HTML generated successfully");
    Ok(html)
}

/// Generiert EPC QR-Code für SEPA-Zahlungen
/// Format: European Payment Council Standard
fn generate_payment_qr_code(
    iban: &str,
    bic: &str,
    account_holder: &str,
    amount: f64,
    reference: &str,
) -> Result<String, String> {
    // EPC QR Code Format (European Payment Council)
    // https://www.europeanpaymentscouncil.eu/document-library/guidance-documents/quick-response-code-guidelines-enable-data-capture-initiation
    let epc_data = format!(
        "BCD\n002\n1\nSCT\n{}\n{}\n{}\nEUR{:.2}\n\n\n{}",
        bic,
        account_holder,
        iban,
        amount,
        reference
    );

    println!("🔧 [QR] Generating EPC QR Code:");
    println!("   IBAN: {}", iban);
    println!("   BIC: {}", bic);
    println!("   Amount: {:.2} EUR", amount);
    println!("   Reference: {}", reference);

    // Generiere QR Code
    let qr_code = QrCode::new(epc_data.as_bytes())
        .map_err(|e| format!("Fehler beim Erstellen des QR-Codes: {}", e))?;

    // Rendere als PNG Bild (5px pro Modul für bessere Lesbarkeit)
    let image = qr_code.render::<Luma<u8>>()
        .min_dimensions(200, 200)
        .max_dimensions(300, 300)
        .build();

    // Konvertiere zu PNG Bytes
    let mut png_bytes = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Fehler beim Kodieren des PNG: {}", e))?;

    // Encode als Base64
    let base64_image = general_purpose::STANDARD.encode(&png_bytes);

    println!("✅ [QR] QR Code generated successfully ({} bytes)", png_bytes.len());

    Ok(format!("data:image/png;base64,{}", base64_image))
}

fn generate_html_from_booking(
    template: &str,
    booking: &BookingWithDetails,
    company: &CompanySettings,
    payment: &PaymentSettings,
) -> Result<String, String> {
    let mut html = template.to_string();

    // ============================================================================
    // HEADER - Company Info
    // ============================================================================
    html = html.replace("{{COMPANY_NAME}}", &company.company_name);
    html = html.replace("{{STREET_ADDRESS}}", &company.street_address);
    html = html.replace("{{PLZ}}", &company.plz);
    html = html.replace("{{CITY}}", &company.city);
    html = html.replace("{{COUNTRY}}", &company.country);
    html = html.replace("{{PHONE}}", &company.phone);
    html = html.replace("{{EMAIL}}", &company.email);
    html = html.replace("{{TAX_ID}}", &company.tax_id);

    // Logo (wenn vorhanden, sonst Platzhalter)
    let logo_html = if let Some(ref logo_path) = company.logo_path {
        if !logo_path.is_empty() {
            format!(r#"<img src="{}" alt="Company Logo" style="max-width: 100%; max-height: 100%; object-fit: contain;" />"#, logo_path)
        } else {
            format!(r#"<div class="logo-placeholder">[LOGO]<br>{}</div>"#, &company.company_name)
        }
    } else {
        format!(r#"<div class="logo-placeholder">[LOGO]<br>{}</div>"#, &company.company_name)
    };
    html = html.replace("{{LOGO_HTML}}", &logo_html);

    // ============================================================================
    // HEADER - Invoice Number
    // ============================================================================
    let invoice_number = format!("#{}-{:04}",
        chrono::Local::now().format("%Y"),
        booking.id
    );
    html = html.replace("{{INVOICE_NUMBER}}", &invoice_number);

    // ============================================================================
    // RECIPIENTS - Gast
    // ============================================================================
    let guest_name = format!("{} {}", booking.guest.vorname, booking.guest.nachname);
    html = html.replace("{{GUEST_NAME}}", &guest_name);

    let guest_address = format!(
        "{}<br>{} {}",
        booking.guest.strasse.as_ref().unwrap_or(&"".to_string()),
        booking.guest.plz.as_ref().unwrap_or(&"".to_string()),
        booking.guest.ort.as_ref().unwrap_or(&"".to_string())
    );
    html = html.replace("{{GUEST_ADDRESS}}", &guest_address);

    // Rechnungsempfänger (optional) - wenn vorhanden
    println!("🔍 [INVOICE] payment_recipient_id: {:?}", booking.payment_recipient_id);

    let invoice_recipient_card = if let Some(recipient_id) = booking.payment_recipient_id {
        println!("✅ [INVOICE] payment_recipient_id is Some({}), calling get_payment_recipient", recipient_id);

        match get_payment_recipient(recipient_id) {
            Ok(Some(recipient)) => {
                println!("✅ [INVOICE] get_payment_recipient SUCCESS: {} ({})", recipient.name, recipient.city.as_ref().unwrap_or(&"".to_string()));

                let recipient_address = format!(
                    "{}{}{} {}{}",
                    recipient.street.as_ref().map(|s| format!("{}<br>", s)).unwrap_or_default(),
                    recipient.plz.as_ref().map(|p| format!("{} ", p)).unwrap_or_default(),
                    recipient.city.as_ref().unwrap_or(&"".to_string()),
                    if recipient.country != "Deutschland" { format!("<br>{}", recipient.country) } else { "".to_string() },
                    if let Some(ref contact) = recipient.contact_person {
                        format!("<br><small style='color: #64748b;'>Ansprechpartner: {}</small>", contact)
                    } else {
                        "".to_string()
                    }
                );

                let card_html = format!(
                    r#"<div class="recipient-card" style="padding: 8px; background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); border-radius: 8px; border: 1px solid #1d4ed8; display: flex; flex-direction: column; justify-content: center; min-height: 85px;">
                        <div style="font-size: 8px; color: rgba(255,255,255,0.8); text-transform: uppercase; font-weight: 600; letter-spacing: 0.05em; margin-bottom: 5px;">⚠️ EXTERNE RECHNUNG</div>
                        <div style="font-size: 11px; font-weight: bold; color: white; margin-bottom: 3px;">{}{}</div>
                        <div style="font-size: 9px; color: rgba(255,255,255,0.9); line-height: 1.3;">{}</div>
                        <div style="margin-top: 6px; padding-top: 6px; border-top: 1px solid rgba(255,255,255,0.2); font-size: 8px; color: rgba(255,255,255,0.7); line-height: 1.3;">
                            Diese Rechnung wird an den oben angegebenen externen Empfänger adressiert.
                        </div>
                    </div>"#,
                    recipient.name,
                    recipient.company.as_ref().map(|c| format!("<br><span style='font-size: 10px; font-weight: normal;'>{}</span>", c)).unwrap_or_default(),
                    recipient_address
                );

                println!("📦 [INVOICE] Generated card HTML ({} bytes)", card_html.len());
                card_html
            },
            Ok(None) => {
                println!("⚠️ [INVOICE] get_payment_recipient returned Ok(None) - recipient not found!");
                "".to_string()
            },
            Err(e) => {
                println!("❌ [INVOICE] get_payment_recipient ERROR: {}", e);
                "".to_string()
            }
        }
    } else {
        println!("ℹ️ [INVOICE] payment_recipient_id is None - no external recipient");
        "".to_string()
    };

    println!("🔄 [INVOICE] Replacing {{{{INVOICE_RECIPIENT_CARD}}}} with content ({} bytes)", invoice_recipient_card.len());
    html = html.replace("{{INVOICE_RECIPIENT_CARD}}", &invoice_recipient_card);
    println!("✅ [INVOICE] Replacement complete");

    // ============================================================================
    // RECIPIENTS - Booking Details
    // ============================================================================
    html = html.replace("{{ROOM_NAME}}", &format!("Zimmer {} - {}", booking.room.name, booking.room.gebaeude_typ));
    html = html.replace("{{CHECKIN_DATE}}", &format_german_date(&booking.checkin_date));
    html = html.replace("{{CHECKOUT_DATE}}", &format_german_date(&booking.checkout_date));

    let nights = calculate_nights(&booking.checkin_date, &booking.checkout_date);
    html = html.replace("{{NIGHTS_COUNT}}", &nights.to_string());

    let guest_count = booking.anzahl_gaeste;
    let guest_label = if guest_count == 1 { "Gast" } else { "Gäste" };
    html = html.replace("{{GUEST_COUNT}}", &guest_count.to_string());
    html = html.replace("{{GUEST_COUNT_LABEL}}", guest_label);

    // ============================================================================
    // META - Dates & Booking Number
    // ============================================================================
    let invoice_date = chrono::Local::now().format("%d.%m.%Y").to_string();
    html = html.replace("{{INVOICE_DATE}}", &invoice_date);

    let stay_period = format!(
        "{} - {}",
        format_german_date(&booking.checkin_date),
        format_german_date(&booking.checkout_date)
    );
    html = html.replace("{{STAY_PERIOD}}", &stay_period);

    // Fälligkeitsdatum
    let due_date = chrono::Local::now()
        .checked_add_signed(chrono::Duration::days(payment.payment_due_days as i64))
        .unwrap()
        .format("%d.%m.%Y")
        .to_string();
    html = html.replace("{{DUE_DATE}}", &due_date);

    html = html.replace("{{BOOKING_NUMBER}}", &booking.reservierungsnummer);

    // Mitgliedsnummer (optional)
    let membership_meta = if let Some(ref membership_id) = booking.guest.mitgliedsnummer {
        if !membership_id.is_empty() {
            format!(
                r#"<div class="meta-item">
                    <div class="meta-label">Mitgliedsnummer</div>
                    <div class="meta-value">{}</div>
                </div>"#,
                membership_id
            )
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    html = html.replace("{{MEMBERSHIP_META}}", &membership_meta);

    // ============================================================================
    // SERVICES TABLE
    // ============================================================================
    let service_rows = generate_service_rows(booking, nights)?;
    html = html.replace("{{SERVICE_ROWS}}", &service_rows);

    // ============================================================================
    // TOTALS - Calculate prices
    // ============================================================================
    let subtotal = booking.grundpreis + booking.services_preis;
    let tax_7_base = booking.grundpreis; // Übernachtung = 7%
    let tax_19_base = booking.services_preis; // Services = 19%

    let tax_7 = tax_7_base * 0.07;
    let tax_19 = tax_19_base * 0.19;

    let total_before_discount = subtotal + tax_7 + tax_19;

    // Rabatte
    let discount_total = booking.discounts.iter()
        .map(|d| {
            if d.discount_type == "percent" {
                subtotal * (d.discount_value / 100.0)
            } else {
                d.discount_value
            }
        })
        .sum::<f64>();

    // 💰 Gast-Guthaben (Verrechnetes Guthaben für diese Buchung)
    let credit_used = get_booking_credit_usage(booking.id).unwrap_or(0.0);
    println!("💰 [INVOICE] Credit used for booking {}: {:.2} €", booking.id, credit_used);

    let grand_total = total_before_discount - discount_total - credit_used;

    html = html.replace("{{SUBTOTAL}}", &format_currency(subtotal));

    // Tax Rows
    let mut tax_rows = String::new();
    if tax_7 > 0.0 {
        tax_rows.push_str(&format!(
            r#"<div class="total-row tax">
                <span class="total-label">MwSt. 7% (Übernachtung)</span>
                <span>{}</span>
            </div>"#,
            format_currency(tax_7)
        ));
    }
    if tax_19 > 0.0 {
        tax_rows.push_str(&format!(
            r#"<div class="total-row tax">
                <span class="total-label">MwSt. 19% (Zusatzleistungen)</span>
                <span>{}</span>
            </div>"#,
            format_currency(tax_19)
        ));
    }
    html = html.replace("{{TAX_ROWS}}", &tax_rows);

    // Discount Rows
    let mut discount_rows = booking.discounts.iter()
        .map(|d| {
            let amount = if d.discount_type == "percent" {
                subtotal * (d.discount_value / 100.0)
            } else {
                d.discount_value
            };
            format!(
                r#"<div class="total-row" style="color: var(--success); font-size: 13px;">
                    <span class="total-label">{}</span>
                    <span>- {}</span>
                </div>"#,
                d.name,
                format_currency(amount)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 💰 Credit Row (wenn Guthaben verrechnet wurde)
    if credit_used > 0.0 {
        let credit_row = format!(
            r#"<div class="total-row" style="color: #10b981; font-size: 13px; font-weight: 600;">
                <span class="total-label">💰 Verrechnetes Gast-Guthaben</span>
                <span>- {}</span>
            </div>"#,
            format_currency(credit_used)
        );
        if !discount_rows.is_empty() {
            discount_rows.push_str("\n");
        }
        discount_rows.push_str(&credit_row);
    }

    html = html.replace("{{DISCOUNT_ROWS}}", &discount_rows);

    html = html.replace("{{GRAND_TOTAL}}", &format_currency(grand_total));

    // ============================================================================
    // FOOTER - Payment Info
    // ============================================================================
    html = html.replace("{{ACCOUNT_HOLDER}}", &payment.account_holder);
    html = html.replace("{{BANK_NAME}}", &payment.bank_name);
    html = html.replace("{{IBAN}}", &payment.iban);
    html = html.replace("{{BIC}}", &payment.bic);

    let payment_reference = format!("{} / {}", invoice_number, booking.reservierungsnummer);
    html = html.replace("{{PAYMENT_REFERENCE}}", &payment_reference);

    // ============================================================================
    // QR CODE - Generate EPC QR Code für SEPA-Zahlungen
    // ============================================================================
    let qr_code_data_url = generate_payment_qr_code(
        &payment.iban,
        &payment.bic,
        &payment.account_holder,
        grand_total,
        &payment_reference,
    )?;

    // Ersetze [QR CODE] Platzhalter mit echtem QR-Code Bild (zentriert im Rahmen)
    let qr_code_html = format!(
        r#"<img src="{}" alt="QR Code für Zahlung" style="width: 100%; height: 100%; object-fit: contain;" />"#,
        qr_code_data_url
    );
    html = html.replace("[QR CODE]<br>\n                        für Zahlung", &qr_code_html);

    Ok(html)
}

fn generate_service_rows(booking: &BookingWithDetails, nights: i32) -> Result<String, String> {
    let mut rows = Vec::new();
    let mut pos = 1;

    // 1. Zimmerpreis (Übernachtung)
    let room_price_per_night = booking.grundpreis / nights as f64;
    rows.push(format!(
        r#"<tr>
            <td>{:02}</td>
            <td>
                <div class="item-description">Übernachtung {}</div>
                <div class="item-details">Zimmer {}, {} Personen</div>
            </td>
            <td>{} Nächte</td>
            <td>{}</td>
            <td>{}</td>
        </tr>"#,
        pos,
        booking.room.gebaeude_typ,
        booking.room.name,
        booking.anzahl_gaeste,
        nights,
        format_currency(room_price_per_night),
        format_currency(booking.grundpreis)
    ));
    pos += 1;

    // 2. Services (Zusatzleistungen)
    for service in &booking.services {
        // Aktuell werden alle Services als Pauschale behandelt (TODO: is_per_night Feature hinzufügen)
        let quantity_label = "1 Pauschal".to_string();
        let total_price = service.price;

        rows.push(format!(
            r#"<tr>
                <td>{:02}</td>
                <td>
                    <div class="item-description">{}</div>
                    <div class="item-details">{}</div>
                </td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
            pos,
            service.name,
            service.description.as_ref().unwrap_or(&"".to_string()),
            quantity_label,
            format_currency(service.price),
            format_currency(total_price)
        ));
        pos += 1;
    }

    Ok(rows.join("\n"))
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn format_german_date(date_str: &str) -> String {
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        date.format("%d.%m.%Y").to_string()
    } else {
        date_str.to_string()
    }
}

fn calculate_nights(checkin: &str, checkout: &str) -> i32 {
    if let (Ok(date_in), Ok(date_out)) = (
        NaiveDate::parse_from_str(checkin, "%Y-%m-%d"),
        NaiveDate::parse_from_str(checkout, "%Y-%m-%d")
    ) {
        (date_out - date_in).num_days() as i32
    } else {
        1
    }
}

fn format_currency(amount: f64) -> String {
    format!("{:.2} €", amount).replace(".", ",")
}
