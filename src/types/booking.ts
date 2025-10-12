export interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  price_member: number;
  price_non_member: number;
  nebensaison_preis: number;
  hauptsaison_preis: number;
  endreinigung: number;
  ort: string;
  schluesselcode?: string;
}

export interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon: string;
  dpolg_mitglied: boolean;
  strasse?: string;
  plz?: string;
  ort?: string;
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
  created_at?: string;
}

export interface BookingWithDetails extends Booking {
  room: Room;
  guest: Guest;
  services: ServiceTemplate[];
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
  is_active: boolean;
  // Emoji & Visualisierung
  emoji?: string;
  color_hex?: string;
  // Putzplan-Integration
  show_in_cleaning_plan: boolean;
  cleaning_plan_position: 'start' | 'end';
  created_at: string;
  updated_at: string;
}

export interface DiscountTemplate {
  id: number;
  name: string;
  description?: string;
  discount_type: 'percent' | 'fixed';
  discount_value: number;
  is_active: boolean;
  // Emoji & Visualisierung
  emoji?: string;
  color_hex?: string;
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