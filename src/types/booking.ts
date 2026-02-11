export interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  nebensaison_preis: number;
  hauptsaison_preis: number;
  endreinigung: number;
  ort: string;
  schluesselcode?: string;
  streetAddress?: string;
  postalCode?: string;
  city?: string;
}

export interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon?: string; // NOW OPTIONAL!
  dpolg_mitglied: boolean;
  strasse?: string;
  plz?: string;
  ort?: string;
  mitgliedsnummer?: string;
  notizen?: string;
  beruf?: string;
  bundesland?: string;
  dienststelle?: string;
  created_at?: string;
  // NEW: 21 additional fields from CSV import
  anrede?: string; // "Herr", "Frau"
  geschlecht?: string; // "männlich", "weiblich", "divers"
  land?: string; // Land (z.B. "Deutschland")
  telefon_geschaeftlich?: string;
  telefon_privat?: string;
  telefon_mobil?: string;
  fax?: string;
  geburtsdatum?: string; // ISO 8601 format
  geburtsort?: string;
  sprache?: string; // z.B. "Deutsch", "Englisch"
  nationalitaet?: string;
  identifikationsnummer?: string; // Ausweisnummer o.Ä.
  debitorenkonto?: string;
  kennzeichen?: string; // Auto-Kennzeichen
  rechnungs_email?: string; // Alternative Email für Rechnungen
  marketing_einwilligung?: boolean;
  leitweg_id?: string; // E-Rechnungs-Leitweg-ID
  kostenstelle?: string;
  tags?: string; // Komma-separierte Tags
}

export interface Booking {
  id: number;
  room_id: number;
  guest_id: number;
  reservierungsnummer: string;
  checkin_date: string;
  checkout_date: string;
  anzahl_gaeste: number;
  anzahl_begleitpersonen: number;
  status: 'reserviert' | 'bestaetigt' | 'eingecheckt' | 'ausgecheckt' | 'storniert';
  grundpreis: number;
  services_preis: number;
  rabatt_preis: number;
  gesamtpreis: number;
  anzahl_naechte: number;
  bemerkungen?: string;
  bezahlt: boolean;
  bezahlt_am?: string | null;
  zahlungsmethode?: string | null;
  mahnung_gesendet_am?: string | null;
  rechnung_versendet_am?: string | null;
  rechnung_versendet_an?: string | null;
  ist_stiftungsfall: boolean;
  payment_recipient_id?: number | null;
  // Putzplan: Alternatives Checkout-Datum (optional, falls abweichend)
  putzplan_checkout_date?: string | null;
  created_at?: string;
  // DPolG Mitglied (für Rabattberechnung)
  ist_dpolg_mitglied?: boolean;
  // Gast-Guthaben (für Rechnungen)
  credit_used?: number | null;
  // Services und Discounts für TapeChart Emoji-Anzeige
  services?: AdditionalService[];
  discounts?: DiscountTemplate[];
}

// Additional Service (gebuchter Service mit berechneten Werten)
export interface AdditionalService {
  id: number;
  booking_id: number;
  service_name: string;
  service_price: number;      // Berechneter finaler Preis
  created_at: string;
  template_id: number | null;
  // Neue Felder für prozentuale Services:
  price_type: 'fixed' | 'percent';
  original_value: number;      // Festbetrag ODER Prozentsatz
  applies_to: 'overnight_price' | 'total_price';
  // Emoji vom Template
  emoji?: string;
}

export interface BookingWithDetails extends Booking {
  room: Room;
  guest: Guest;
  services: AdditionalService[];
  discounts: DiscountTemplate[];
}

export interface TapeChartEvent {
  id: number;
  resourceId: number;
  startDate: Date;
  endDate: Date;
  name: string;
  booking: BookingWithDetails;
  status: Booking['status'];
}

export interface ServiceTemplate {
  id: number;
  name: string;
  description?: string;
  price: number;
  isActive: boolean;
  // Preis-Typ: Festbetrag oder Prozent
  priceType: 'fixed' | 'percent';
  // Worauf bezieht sich der Prozent-Preis? (nur relevant bei priceType='percent')
  appliesTo: 'overnight_price' | 'total_price';
  // Emoji
  emoji?: string;
  // Putzplan-Integration
  showInCleaningPlan: boolean;
  cleaningPlanPosition: 'start' | 'end';
  createdAt?: string;
  updatedAt?: string;
}

export interface DiscountTemplate {
  id: number;
  name: string;
  description?: string;
  discount_type: 'percent' | 'fixed';
  discount_value: number;
  is_active: boolean;
  // Emoji
  emoji?: string;
  // Putzplan-Integration
  show_in_cleaning_plan: boolean;
  cleaning_plan_position: 'start' | 'end';
  // Worauf bezieht sich der Rabatt?
  applies_to: 'overnight_price' | 'total_price';
  created_at: string;
  updated_at: string;
}

// Begleitpersonen (Booking-spezifisch)
export interface AccompanyingGuest {
  id: number;
  booking_id: number;
  vorname: string;
  nachname: string;
  geburtsdatum?: string;
  companion_id?: number; // Referenz zu GuestCompanion (wenn aus Pool)
}

// Permanenter Pool von Begleitpersonen pro Gast
export interface GuestCompanion {
  id: number;
  guest_id: number;
  vorname: string;
  nachname: string;
  geburtsdatum?: string;
  beziehung?: string; // "Ehepartner", "Kind", "Freund", etc.
  notizen?: string;
  created_at: string;
}

// Rechnungsempfänger (externe Zahlungsempfänger für Buchungen)
export interface PaymentRecipient {
  id: number;
  name: string;
  company?: string;
  street?: string;
  plz?: string;
  city?: string;
  country: string;
  contact_person?: string;
  notes?: string;
  created_at: string;
  updated_at?: string;
}