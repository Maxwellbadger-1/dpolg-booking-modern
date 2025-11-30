use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

// ============================================================================
// ROOM MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: i32,
    pub name: String,
    pub gebaeude_typ: String,
    pub capacity: i32,
    pub price_member: f64,
    pub price_non_member: f64,
    pub nebensaison_preis: Option<f64>,
    pub hauptsaison_preis: Option<f64>,
    pub endreinigung: Option<f64>,
    pub ort: String,
    pub schluesselcode: Option<String>,
    pub street_address: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub notizen: Option<String>,
}

impl From<Row> for Room {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            gebaeude_typ: row.get("gebaeude_typ"),
            capacity: row.get("capacity"),
            price_member: row.get("price_member"),
            price_non_member: row.get("price_non_member"),
            nebensaison_preis: row.get("nebensaison_preis"),
            hauptsaison_preis: row.get("hauptsaison_preis"),
            endreinigung: row.get("endreinigung"),
            ort: row.get("ort"),
            schluesselcode: row.get("schluesselcode"),
            street_address: row.get("street_address"),
            postal_code: row.get("postal_code"),
            city: row.get("city"),
            notizen: row.get("notizen"),
        }
    }
}

// ============================================================================
// GUEST MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guest {
    pub id: i32,
    pub vorname: String,
    pub nachname: String,
    pub email: String,
    pub telefon: Option<String>,
    pub dpolg_mitglied: bool,
    pub strasse: Option<String>,
    pub plz: Option<String>,
    pub ort: Option<String>,
    pub mitgliedsnummer: Option<String>,
    pub notizen: Option<String>,
    pub beruf: Option<String>,
    pub bundesland: Option<String>,
    pub dienststelle: Option<String>,
    pub created_at: Option<String>,
    // Extended fields
    pub anrede: Option<String>,
    pub geschlecht: Option<String>,
    pub land: Option<String>,
    pub telefon_geschaeftlich: Option<String>,
    pub telefon_privat: Option<String>,
    pub telefon_mobil: Option<String>,
    pub fax: Option<String>,
    pub geburtsdatum: Option<String>,
    pub geburtsort: Option<String>,
    pub sprache: Option<String>,
    pub nationalitaet: Option<String>,
    pub identifikationsnummer: Option<String>,
    pub debitorenkonto: Option<String>,
    pub kennzeichen: Option<String>,
    pub rechnungs_email: Option<String>,
    pub marketing_einwilligung: Option<bool>,
    pub leitweg_id: Option<String>,
    pub kostenstelle: Option<String>,
    pub tags: Option<String>,
    pub automail: Option<bool>,
    pub automail_sprache: Option<String>,
}

impl From<Row> for Guest {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            vorname: row.get("vorname"),
            nachname: row.get("nachname"),
            email: row.get("email"),
            telefon: row.get("telefon"),
            dpolg_mitglied: row.get("dpolg_mitglied"),
            strasse: row.get("strasse"),
            plz: row.get("plz"),
            ort: row.get("ort"),
            mitgliedsnummer: row.get("mitgliedsnummer"),
            notizen: row.get("notizen"),
            beruf: row.get("beruf"),
            bundesland: row.get("bundesland"),
            dienststelle: row.get("dienststelle"),
            created_at: row.get("created_at"),
            anrede: row.get("anrede"),
            geschlecht: row.get("geschlecht"),
            land: row.get("land"),
            telefon_geschaeftlich: row.get("telefon_geschaeftlich"),
            telefon_privat: row.get("telefon_privat"),
            telefon_mobil: row.get("telefon_mobil"),
            fax: row.get("fax"),
            geburtsdatum: row.get("geburtsdatum"),
            geburtsort: row.get("geburtsort"),
            sprache: row.get("sprache"),
            nationalitaet: row.get("nationalitaet"),
            identifikationsnummer: row.get("identifikationsnummer"),
            debitorenkonto: row.get("debitorenkonto"),
            kennzeichen: row.get("kennzeichen"),
            rechnungs_email: row.get("rechnungs_email"),
            marketing_einwilligung: row.get("marketing_einwilligung"),
            leitweg_id: row.get("leitweg_id"),
            kostenstelle: row.get("kostenstelle"),
            tags: row.get("tags"),
            automail: row.get("automail"),
            automail_sprache: row.get("automail_sprache"),
        }
    }
}

// ============================================================================
// BOOKING MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    pub id: i32,
    pub room_id: i32,
    pub guest_id: i32,
    pub reservierungsnummer: String,
    pub checkin_date: String,
    pub checkout_date: String,
    pub anzahl_gaeste: i32,
    pub status: String,
    pub gesamtpreis: f64,
    pub bemerkungen: Option<String>,
    pub created_at: Option<String>,
    pub anzahl_begleitpersonen: Option<i32>,
    pub grundpreis: Option<f64>,
    pub services_preis: Option<f64>,
    pub rabatt_preis: Option<f64>,
    pub anzahl_naechte: Option<i32>,
    pub updated_at: Option<String>,
    pub bezahlt: Option<bool>,
    pub bezahlt_am: Option<String>,
    pub zahlungsmethode: Option<String>,
    pub mahnung_gesendet_am: Option<String>,
    pub rechnung_versendet_am: Option<String>,
    pub rechnung_versendet_an: Option<String>,
    pub ist_stiftungsfall: Option<bool>,
    pub payment_recipient_id: Option<i32>,
    pub putzplan_checkout_date: Option<String>,
    pub ist_dpolg_mitglied: Option<bool>,
    pub credit_used: Option<f64>,
    // Optional services and discounts for TapeChart emoji display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<ServiceEmoji>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discounts: Option<Vec<DiscountEmoji>>,
}

impl From<Row> for Booking {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            room_id: row.get("room_id"),
            guest_id: row.get("guest_id"),
            reservierungsnummer: row.get("reservierungsnummer"),
            checkin_date: row.get("checkin_date"),
            checkout_date: row.get("checkout_date"),
            anzahl_gaeste: row.get("anzahl_gaeste"),
            status: row.get("status"),
            gesamtpreis: row.get("gesamtpreis"),
            bemerkungen: row.get("bemerkungen"),
            created_at: row.get("created_at"),
            anzahl_begleitpersonen: row.get("anzahl_begleitpersonen"),
            grundpreis: row.get("grundpreis"),
            services_preis: row.get("services_preis"),
            rabatt_preis: row.get("rabatt_preis"),
            anzahl_naechte: row.get("anzahl_naechte"),
            updated_at: row.get("updated_at"),
            bezahlt: row.get("bezahlt"),
            bezahlt_am: row.get("bezahlt_am"),
            zahlungsmethode: row.get("zahlungsmethode"),
            mahnung_gesendet_am: row.get("mahnung_gesendet_am"),
            rechnung_versendet_am: row.get("rechnung_versendet_am"),
            rechnung_versendet_an: row.get("rechnung_versendet_an"),
            ist_stiftungsfall: row.get("ist_stiftungsfall"),
            payment_recipient_id: row.get("payment_recipient_id"),
            putzplan_checkout_date: row.get("putzplan_checkout_date"),
            ist_dpolg_mitglied: row.get("ist_dpolg_mitglied"),
            credit_used: row.try_get("credit_used").ok().flatten(),
            // Initialize with None - will be populated by get_all_bookings_pg
            services: None,
            discounts: None,
        }
    }
}

/// Booking with Services and Discounts for TapeChart Emoji display
/// Lighter than BookingWithDetails (no room/guest objects)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingWithEmojis {
    // Flatten Booking fields directly (for TypeScript compatibility)
    #[serde(flatten)]
    pub booking: Booking,
    // Services and Discounts with emojis for TapeChart
    pub services: Vec<ServiceEmoji>,
    pub discounts: Vec<DiscountEmoji>,
}

/// Lightweight service struct for emoji display in TapeChart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEmoji {
    pub id: i64,
    pub name: String,
    pub emoji: Option<String>,
    pub color_hex: Option<String>,
    pub cleaning_plan_position: Option<String>,
}

/// Lightweight discount struct for emoji display in TapeChart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountEmoji {
    pub id: i64,
    pub name: String,
    pub emoji: Option<String>,
    pub color_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingWithDetails {
    // Flatten Booking fields directly (for TypeScript compatibility)
    #[serde(flatten)]
    pub booking: Booking,
    // Nested objects (required by TypeScript interface)
    pub room: Option<Room>,
    pub guest: Option<Guest>,
    // Services and Discounts (required by TypeScript interface)
    pub services: Vec<AdditionalService>,
    pub discounts: Vec<Discount>,
    // Legacy fields (kept for backwards compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guest_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guest_email: Option<String>,
}

// ============================================================================
// ADDITIONAL SERVICE MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalService {
    pub id: i64,
    pub booking_id: i64,
    pub service_name: String,
    pub service_price: f32,
    pub template_id: Option<i32>,
    pub price_type: String,
    pub original_value: f32,
    pub applies_to: String,
    pub emoji: Option<String>,
}

impl From<Row> for AdditionalService {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            booking_id: row.get("booking_id"),
            service_name: row.get("service_name"),
            service_price: row.get("service_price"),
            template_id: row.get("template_id"),
            price_type: row.get("price_type"),
            original_value: row.get("original_value"),
            applies_to: row.get("applies_to"),
            emoji: row.get("emoji"),
        }
    }
}

// ============================================================================
// DISCOUNT MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discount {
    pub id: i64,
    pub booking_id: i64,
    pub discount_name: String,
    pub discount_type: String,
    pub discount_value: f32,
    pub emoji: Option<String>,
}

impl From<Row> for Discount {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            booking_id: row.get("booking_id"),
            discount_name: row.get("discount_name"),
            discount_type: row.get("discount_type"),
            discount_value: row.get("discount_value"),
            emoji: row.get("emoji"),
        }
    }
}

// ============================================================================
// EMAIL LOG MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLog {
    pub id: i32,
    pub booking_id: Option<i32>,
    pub guest_id: i32,
    pub template_name: String,
    pub recipient_email: String,
    pub subject: String,
    pub status: String,
    pub error_message: Option<String>,
    pub sent_at: Option<String>,
}

impl From<Row> for EmailLog {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            booking_id: row.get("booking_id"),
            guest_id: row.get("guest_id"),
            template_name: row.get("template_name"),
            recipient_email: row.get("recipient_email"),
            subject: row.get("subject"),
            status: row.get("status"),
            error_message: row.get("error_message"),
            sent_at: row.get("sent_at"),
        }
    }
}

// More models can be added here...

// ============================================================================
// REMINDER MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: i32,
    pub booking_id: Option<i32>,
    pub reminder_type: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: String,
    pub priority: String,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub is_snoozed: bool,
    pub snoozed_until: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Row> for Reminder {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            booking_id: row.get("booking_id"),
            reminder_type: row.get("reminder_type"),
            title: row.get("title"),
            description: row.get("description"),
            due_date: row.get("due_date"),
            priority: row.get("priority"),
            is_completed: row.get("is_completed"),
            completed_at: row.get("completed_at"),
            is_snoozed: row.get("is_snoozed"),  // Now BOOLEAN in PostgreSQL
            snoozed_until: row.get("snoozed_until"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// AccompanyingGuest Model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccompanyingGuest {
    pub id: i64,
    pub booking_id: i64,
    #[serde(rename = "vorname", alias = "firstName", alias = "first_name")]
    pub vorname: String,
    #[serde(rename = "nachname", alias = "lastName", alias = "last_name")]
    pub nachname: String,
    #[serde(rename = "geburtsdatum", alias = "birthDate", alias = "birth_date")]
    pub geburtsdatum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub companion_id: Option<i64>,
}

impl From<Row> for AccompanyingGuest {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            booking_id: row.get("booking_id"),
            vorname: row.get("vorname"),
            nachname: row.get("nachname"),
            geburtsdatum: row.get("geburtsdatum"),
            companion_id: None, // Not stored in accompanying_guests table
        }
    }
}

// ServiceTemplate Model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceTemplate {
    pub id: i32,
    #[serde(rename = "name")]
    pub service_name: String,
    pub description: Option<String>,
    pub price_type: String,
    #[serde(rename = "price")]
    pub original_value: f64,
    pub applies_to: String,
    pub is_active: bool,
    pub emoji: Option<String>,
    pub show_in_cleaning_plan: bool,
    pub cleaning_plan_position: String,
    pub created_at: Option<String>,
}

impl From<Row> for ServiceTemplate {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            service_name: row.get("name"),
            description: row.get("description"),
            price_type: row.get("price_type"),
            original_value: row.get("price"),
            applies_to: row.get("applies_to"),
            is_active: row.get("is_active"),
            emoji: row.get("emoji"),
            show_in_cleaning_plan: row.get::<_, Option<bool>>("show_in_cleaning_plan").unwrap_or(false),
            cleaning_plan_position: row.get::<_, Option<String>>("cleaning_plan_position").unwrap_or_else(|| "start".to_string()),
            created_at: row.get("created_at"),
        }
    }
}

// DiscountTemplate Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountTemplate {
    pub id: i32,
    #[serde(rename = "name")]
    pub discount_name: String,
    pub discount_type: String,
    pub discount_value: f64,
    pub is_active: bool,
    pub emoji: Option<String>,
    pub created_at: Option<String>,
}

impl From<Row> for DiscountTemplate {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            discount_name: row.get("name"),
            discount_type: row.get("discount_type"),
            discount_value: row.get("discount_value"),
            is_active: row.get("is_active"),
            emoji: row.get("emoji"),
            created_at: row.get("created_at"),
        }
    }
}

// PaymentRecipient Model (externe Zahlungsempfänger für Rechnung)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRecipient {
    pub id: i32,
    pub name: String,
    pub company: Option<String>,
    pub street: Option<String>,
    pub plz: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub contact_person: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Row> for PaymentRecipient {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            company: row.get("company"),
            street: row.get("street"),
            plz: row.get("plz"),
            city: row.get("city"),
            country: row.get("country"),
            contact_person: row.get("contact_person"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// CompanySettings Model (Singleton) - Matches PostgreSQL table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanySettings {
    pub id: i32,
    pub company_name: String,
    pub street_address: Option<String>,
    pub plz: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub tax_id: Option<String>,
    pub ceo_name: Option<String>,
    pub registry_court: Option<String>,
    pub logo_path: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Row> for CompanySettings {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            company_name: row.get("company_name"),
            street_address: row.get("street_address"),
            plz: row.get("plz"),
            city: row.get("city"),
            country: row.get("country"),
            phone: row.get("phone"),
            fax: row.get("fax"),
            email: row.get("email"),
            website: row.get("website"),
            tax_id: row.get("tax_id"),
            ceo_name: row.get("ceo_name"),
            registry_court: row.get("registry_court"),
            logo_path: row.get("logo_path"),
            updated_at: row.get("updated_at"),
        }
    }
}

// PricingSettings Model (Singleton) - Matches PostgreSQL table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingSettings {
    pub id: i32,
    pub hauptsaison_aktiv: Option<bool>,
    pub hauptsaison_start: Option<String>,
    pub hauptsaison_ende: Option<String>,
    pub mitglieder_rabatt_aktiv: Option<bool>,
    pub mitglieder_rabatt_prozent: Option<f64>,
    pub rabatt_basis: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Row> for PricingSettings {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            hauptsaison_aktiv: row.get("hauptsaison_aktiv"),
            hauptsaison_start: row.get("hauptsaison_start"),
            hauptsaison_ende: row.get("hauptsaison_ende"),
            mitglieder_rabatt_aktiv: row.get("mitglieder_rabatt_aktiv"),
            mitglieder_rabatt_prozent: row.get("mitglieder_rabatt_prozent"),
            rabatt_basis: row.get("rabatt_basis"),
            updated_at: row.get("updated_at"),
        }
    }
}

// EmailConfig Model (Singleton) - Matches PostgreSQL table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailConfig {
    pub id: i32,
    pub smtp_server: Option<String>,
    pub smtp_port: Option<i32>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub from_email: Option<String>,
    pub from_name: Option<String>,
    pub use_tls: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Row> for EmailConfig {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            smtp_server: row.get("smtp_server"),
            smtp_port: row.get("smtp_port"),
            smtp_username: row.get("smtp_username"),
            smtp_password: row.get("smtp_password"),
            from_email: row.get("from_email"),
            from_name: row.get("from_name"),
            use_tls: row.get("use_tls"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// EmailTemplate Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: i32,
    pub template_name: String,
    pub subject: String,
    pub body: String,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<Row> for EmailTemplate {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            template_name: row.get("template_name"),
            subject: row.get("subject"),
            body: row.get("body"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// NotificationSettings Model (Singleton) - Matches PostgreSQL table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub id: i32,
    pub checkin_reminders_enabled: Option<bool>,
    pub payment_reminders_enabled: Option<bool>,
    pub payment_reminder_after_days: Option<i32>,
    pub payment_reminder_repeat_days: Option<i32>,
    pub scheduler_interval_hours: Option<i32>,
    pub updated_at: Option<String>,
}

impl From<Row> for NotificationSettings {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            checkin_reminders_enabled: row.get("checkin_reminders_enabled"),
            payment_reminders_enabled: row.get("payment_reminders_enabled"),
            payment_reminder_after_days: row.get("payment_reminder_after_days"),
            payment_reminder_repeat_days: row.get("payment_reminder_repeat_days"),
            scheduler_interval_hours: row.get("scheduler_interval_hours"),
            updated_at: row.get("updated_at"),
        }
    }
}

// PaymentSettings Model (Singleton) - Matches PostgreSQL table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSettings {
    pub id: i32,
    pub bank_name: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub account_holder: Option<String>,
    pub mwst_rate: Option<f64>,
    pub payment_due_days: Option<i32>,
    pub reminder_after_days: Option<i32>,
    pub payment_text: Option<String>,
    pub updated_at: Option<String>,
    pub dpolg_rabatt: Option<f64>,
}

impl From<Row> for PaymentSettings {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            bank_name: row.get("bank_name"),
            iban: row.get("iban"),
            bic: row.get("bic"),
            account_holder: row.get("account_holder"),
            mwst_rate: row.get("mwst_rate"),
            payment_due_days: row.get("payment_due_days"),
            reminder_after_days: row.get("reminder_after_days"),
            payment_text: row.get("payment_text"),
            updated_at: row.get("updated_at"),
            dpolg_rabatt: row.get("dpolg_rabatt"),
        }
    }
}
