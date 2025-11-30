// Repository Pattern (Best Practice 2025)
// Each repository handles one entity type

pub mod room_repository;
pub mod guest_repository;
pub mod booking_repository;
pub mod additional_service_repository;
pub mod discount_repository;
pub mod email_log_repository;
pub mod reminder_repository;
pub mod accompanying_guest_repository;
pub mod service_template_repository;
pub mod discount_template_repository;
pub mod payment_recipient_repository;
pub mod company_settings_repository;
pub mod pricing_settings_repository;
pub mod email_config_repository;
pub mod email_template_repository;
pub mod notification_settings_repository;
pub mod payment_settings_repository;
pub mod guest_credit_repository;
pub mod cleaning_task_repository;
pub mod scheduled_email_repository;

pub use room_repository::RoomRepository;
pub use guest_repository::GuestRepository;
pub use booking_repository::BookingRepository;
pub use additional_service_repository::AdditionalServiceRepository;
pub use discount_repository::DiscountRepository;
pub use email_log_repository::EmailLogRepository;
pub use reminder_repository::ReminderRepository;
pub use accompanying_guest_repository::AccompanyingGuestRepository;
pub use service_template_repository::ServiceTemplateRepository;
pub use discount_template_repository::DiscountTemplateRepository;
pub use payment_recipient_repository::PaymentRecipientRepository;
pub use company_settings_repository::CompanySettingsRepository;
pub use pricing_settings_repository::PricingSettingsRepository;
pub use email_config_repository::EmailConfigRepository;
pub use email_template_repository::EmailTemplateRepository;
pub use notification_settings_repository::NotificationSettingsRepository;
pub use payment_settings_repository::PaymentSettingsRepository;
pub use guest_credit_repository::GuestCreditRepository;
pub use cleaning_task_repository::CleaningTaskRepository;
pub use scheduled_email_repository::ScheduledEmailRepository;

// More repositories will be added as needed
