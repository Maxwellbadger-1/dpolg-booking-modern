/**
 * Smart Test Data Generators for DevTools
 *
 * Strategy: Use REAL database data when available, generate minimal test data as fallback
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

export const formatDate = (date: Date): string => {
  return date.toISOString().split('T')[0];
};

export const addDays = (date: Date, days: number): Date => {
  const result = new Date(date);
  result.setDate(result.getDate() + days);
  return result;
};

export const getToday = (): string => formatDate(new Date());
export const getTomorrow = (): string => formatDate(addDays(new Date(), 1));
export const getNextWeek = (): string => formatDate(addDays(new Date(), 7));

// ============================================================================
// TEST DATA CACHE (Fetched once, reused across tests)
// ============================================================================

interface TestDataCache {
  guests: any[];
  rooms: any[];
  bookings: any[];
  serviceTemplates: any[];
  discountTemplates: any[];
  emailTemplates: any[];
  paymentRecipients: any[];
  initialized: boolean;
}

const cache: TestDataCache = {
  guests: [],
  rooms: [],
  bookings: [],
  serviceTemplates: [],
  discountTemplates: [],
  emailTemplates: [],
  paymentRecipients: [],
  initialized: false,
};

/**
 * Initialize cache with real database data
 */
export const initializeTestDataCache = async (): Promise<void> => {
  if (cache.initialized) return;

  try {
    // Fetch all real data in parallel
    const [guests, rooms, bookings, serviceTemplates, discountTemplates, emailTemplates, paymentRecipients] = await Promise.all([
      invoke('get_all_guests_pg').catch(() => []),
      invoke('get_all_rooms_pg').catch(() => []),
      invoke('get_all_bookings_pg').catch(() => []),
      invoke('get_all_service_templates_pg').catch(() => []),
      invoke('get_all_discount_templates_pg').catch(() => []),
      invoke('get_all_email_templates_pg').catch(() => []),
      invoke('get_all_payment_recipients_pg').catch(() => []),
    ]);

    cache.guests = Array.isArray(guests) ? guests : [];
    cache.rooms = Array.isArray(rooms) ? rooms : [];
    cache.bookings = Array.isArray(bookings) ? bookings : [];
    cache.serviceTemplates = Array.isArray(serviceTemplates) ? serviceTemplates : [];
    cache.discountTemplates = Array.isArray(discountTemplates) ? discountTemplates : [];
    cache.emailTemplates = Array.isArray(emailTemplates) ? emailTemplates : [];
    cache.paymentRecipients = Array.isArray(paymentRecipients) ? paymentRecipients : [];
    cache.initialized = true;

    console.log('✅ Test Data Cache initialized:', {
      guests: cache.guests.length,
      rooms: cache.rooms.length,
      bookings: cache.bookings.length,
      serviceTemplates: cache.serviceTemplates.length,
      discountTemplates: cache.discountTemplates.length,
      emailTemplates: cache.emailTemplates.length,
      paymentRecipients: cache.paymentRecipients.length,
    });
  } catch (error) {
    console.error('❌ Failed to initialize test data cache:', error);
    cache.initialized = true; // Mark as initialized anyway to avoid infinite retries
  }
};

/**
 * Refresh cache (call after creating test entities)
 */
export const refreshTestDataCache = async (): Promise<void> => {
  cache.initialized = false;
  await initializeTestDataCache();
};

// ============================================================================
// GUEST TEST DATA
// ============================================================================

export const getTestGuest = () => cache.guests[0] || null;

export const getTestGuestData = () => ({
  vorname: 'DevTools',
  nachname: `Test${Date.now()}`,
  email: `devtools.test.${Date.now()}@example.com`,
  telefon: '+49 000 000000',
  dpolgMitglied: true,
  strasse: 'Test Straße 1',
  plz: '00000',
  ort: 'Teststadt',
  mitgliedsnummer: `DT${Date.now()}`,
  notizen: 'Erstellt via DevTools Comprehensive Testing',
});

// ============================================================================
// ROOM TEST DATA
// ============================================================================

export const getTestRoom = () => cache.rooms[0] || null;

export const getTestRoomData = () => ({
  name: `DT-${Date.now().toString().slice(-4)}`,
  gebaeudeTyp: 'Einzelzimmer',
  capacity: 1,
  priceMember: 50.0,
  priceNonMember: 60.0,
  ort: 'Teststadt',
});

// ============================================================================
// BOOKING TEST DATA
// ============================================================================

export const getTestBooking = () => cache.bookings[0] || null;

export const getTestBookingData = () => {
  const guest = getTestGuest();
  const room = getTestRoom();

  if (!guest || !room) {
    throw new Error('Keine Test-Daten verfügbar! Bitte erst Gast und Zimmer erstellen.');
  }

  // Format: JAHR-XXXXXX (6-stellige Nummer aus Timestamp)
  const year = new Date().getFullYear();
  const uniqueNum = Date.now().toString().slice(-6);

  return {
    guestId: guest.id,
    roomId: room.id,
    reservierungsnummer: `${year}-${uniqueNum}`,
    checkinDate: getToday(),
    checkoutDate: getTomorrow(),
    anzahlGaeste: 1,
    status: 'bestaetigt',
    gesamtpreis: 50.0,
    bemerkungen: 'DevTools Test Booking',
  };
};

// ============================================================================
// SERVICE TEST DATA
// ============================================================================

export const getTestServiceTemplate = () => cache.serviceTemplates[0] || null;

export const getTestServiceData = (bookingId?: number) => {
  const booking = bookingId || getTestBooking()?.id;

  if (!booking) {
    throw new Error('Keine Booking verfügbar! Bitte erst Booking erstellen.');
  }

  return {
    bookingId: booking,
    serviceName: 'DevTools Test Service',
    servicePrice: 10.0,
    templateId: null,
    priceType: 'fixed',
    originalValue: 10.0,
    appliesTo: 'overnight_price',
  };
};

export const getTestServiceTemplateData = () => ({
  serviceName: `DevTools Template ${Date.now()}`,
  priceType: 'fixed',
  originalValue: 15.0,
  appliesTo: 'overnight_price',
});

// ============================================================================
// DISCOUNT TEST DATA
// ============================================================================

export const getTestDiscountTemplate = () => cache.discountTemplates[0] || null;

export const getTestDiscountData = (bookingId?: number) => {
  const booking = bookingId || getTestBooking()?.id;

  if (!booking) {
    throw new Error('Keine Booking verfügbar! Bitte erst Booking erstellen.');
  }

  return {
    bookingId: booking,
    discountName: 'DevTools Test Rabatt',
    discountType: 'percentage',
    discountValue: 10.0,
    appliesTo: 'overnight_price',
    templateId: null,
  };
};

export const getTestDiscountTemplateData = () => ({
  discountName: `DevTools Rabatt ${Date.now()}`,
  discountType: 'percentage',
  discountValue: 10.0,
});

// ============================================================================
// EMAIL TEST DATA
// ============================================================================

export const getTestEmailTemplate = () => cache.emailTemplates[0] || null;

export const getTestEmailTemplateData = () => ({
  name: `DevTools Email ${Date.now()}`,
  subject: 'DevTools Test Email',
  body: '<p>Dies ist eine Test-Email erstellt via DevTools</p>',
  isActive: false, // Don't interfere with production
});

export const getTestEmailLogData = (bookingId?: number, guestId?: number) => {
  const booking = bookingId || getTestBooking()?.id;
  const guest = guestId || getTestGuest()?.id;

  return {
    bookingId: booking || null,
    guestId: guest || null,
    emailType: 'manual',
    recipient: 'devtools.test@example.com',
    subject: 'DevTools Test Email Log',
    body: 'Test email log content',
    status: 'draft',
    sentAt: null,
  };
};

export const getTestEmailConfigData = () => ({
  smtpHost: 'smtp.example.com',
  smtpPort: 587,
  smtpUser: 'devtools@example.com',
  smtpPassword: 'test_password',
  fromEmail: 'devtools@example.com',
  fromName: 'DevTools Test',
  useTls: true,
});

// ============================================================================
// REMINDER TEST DATA
// ============================================================================

export const getTestReminderData = (bookingId?: number) => {
  const booking = bookingId || getTestBooking()?.id;

  return {
    bookingId: booking || null,
    reminderType: 'manual',
    title: 'DevTools Test Reminder',
    description: 'Test reminder erstellt via DevTools',
    dueDate: getTomorrow(),
    priority: 'medium',
    isCompleted: false,
  };
};

// ============================================================================
// ACCOMPANYING GUEST TEST DATA
// ============================================================================

export const getTestAccompanyingGuestData = (bookingId?: number) => {
  const booking = bookingId || getTestBooking()?.id;

  if (!booking) {
    throw new Error('Keine Booking verfügbar!');
  }

  return {
    bookingId: booking,
    firstName: 'DevTools',
    lastName: `Begleitung${Date.now()}`,
    relationship: 'Partner',
  };
};

// ============================================================================
// PAYMENT RECIPIENT TEST DATA
// ============================================================================

export const getTestPaymentRecipient = () => cache.paymentRecipients[0] || null;

export const getTestPaymentRecipientData = () => ({
  name: `DevTools Empfänger ${Date.now()}`,
});

// ============================================================================
// SETTINGS TEST DATA
// ============================================================================

export const getTestCompanySettingsData = () => ({
  companyName: 'DevTools Test Firma',
  companyAddress: 'Test Straße 1, 00000 Teststadt',
  companyPhone: '+49 000 000000',
  companyEmail: 'devtools@example.com',
  taxId: 'DE123456789',
  registrationNumber: 'HRB 12345',
});

export const getTestPricingSettingsData = () => ({
  defaultPricePerNight: 50.0,
  weekendSurcharge: 10.0,
  memberDiscount: 15.0,
  currency: 'EUR',
});

export const getTestPaymentSettingsData = () => ({
  defaultPaymentMethod: 'ueberweisung',
  acceptCash: true,
  acceptCard: true,
  acceptPaypal: false,
  paymentDeadlineDays: 14,
});

export const getTestNotificationSettingsData = () => ({
  enableEmailNotifications: true,
  enableSmsNotifications: false,
  bookingConfirmationEmail: true,
  paymentReminderEmail: true,
  reminderDaysBefore: 3,
});

// ============================================================================
// VALIDATION TEST DATA
// ============================================================================

export const getTestEmailForValidation = () => 'devtools.test@example.com';
export const getTestInvalidEmail = () => 'invalid-email';

export const getTestDateRange = () => ({
  checkin: getToday(),
  checkout: getTomorrow(),
});

export const getTestInvalidDateRange = () => ({
  checkin: getTomorrow(),
  checkout: getToday(), // Invalid: checkout before checkin
});

// ============================================================================
// EXPORT ALL
// ============================================================================

export default {
  // Initialization
  initializeTestDataCache,
  refreshTestDataCache,

  // Helpers
  formatDate,
  addDays,
  getToday,
  getTomorrow,
  getNextWeek,

  // Getters
  getTestGuest,
  getTestRoom,
  getTestBooking,
  getTestServiceTemplate,
  getTestDiscountTemplate,
  getTestEmailTemplate,
  getTestPaymentRecipient,

  // Data Generators
  getTestGuestData,
  getTestRoomData,
  getTestBookingData,
  getTestServiceData,
  getTestServiceTemplateData,
  getTestDiscountData,
  getTestDiscountTemplateData,
  getTestEmailTemplateData,
  getTestEmailLogData,
  getTestEmailConfigData,
  getTestReminderData,
  getTestAccompanyingGuestData,
  getTestPaymentRecipientData,
  getTestCompanySettingsData,
  getTestPricingSettingsData,
  getTestPaymentSettingsData,
  getTestNotificationSettingsData,
  getTestEmailForValidation,
  getTestInvalidEmail,
  getTestDateRange,
  getTestInvalidDateRange,
};
