/**
 * Date Formatting Utilities - Single Source of Truth
 *
 * Zentrale Stelle für ALLE Datums-Formatierungen.
 * Ersetzt 5+ Duplikate und 22+ scattered toLocaleDateString() Calls.
 *
 * Pattern: Wie priceFormatting.ts - Eine Quelle, konsistent überall
 */

import { format, parseISO, isToday, isTomorrow, isYesterday, formatDistance, differenceInDays } from 'date-fns';
import { de } from 'date-fns/locale';

/**
 * Standard-Format: DD.MM.YYYY
 * Nutze dies als Default für alle Datums-Anzeigen
 *
 * @param dateStr - ISO String (YYYY-MM-DD) oder Date Objekt
 * @returns "28.10.2025"
 */
export function formatDate(dateStr: string | Date): string {
  try {
    const date = typeof dateStr === 'string' ? parseISO(dateStr) : dateStr;
    return format(date, 'dd.MM.yyyy', { locale: de });
  } catch (err) {
    console.error('❌ [formatDate] Invalid date:', dateStr, err);
    return 'Ungültiges Datum';
  }
}

/**
 * Smart Format mit relativen Labels (Heute/Morgen/Gestern)
 * Perfekt für Reminder und aktuelle Buchungen
 *
 * @param dateStr - ISO String oder Date
 * @returns "Heute", "Morgen", "Gestern" oder "28.10.2025"
 */
export function formatDateSmart(dateStr: string | Date): string {
  try {
    const date = typeof dateStr === 'string' ? parseISO(dateStr) : dateStr;

    if (isToday(date)) return 'Heute';
    if (isTomorrow(date)) return 'Morgen';
    if (isYesterday(date)) return 'Gestern';

    return format(date, 'dd.MM.yyyy', { locale: de });
  } catch (err) {
    console.error('❌ [formatDateSmart] Invalid date:', dateStr, err);
    return 'Ungültiges Datum';
  }
}

/**
 * Langes Format: "Montag, 28. Oktober 2025"
 * Nutze für Bestätigungs-Emails und formelle Dokumente
 *
 * @param dateStr - ISO String oder Date
 * @returns "Montag, 28. Oktober 2025"
 */
export function formatDateLong(dateStr: string | Date): string {
  try {
    const date = typeof dateStr === 'string' ? parseISO(dateStr) : dateStr;
    return format(date, 'EEEE, d. MMMM yyyy', { locale: de });
  } catch (err) {
    console.error('❌ [formatDateLong] Invalid date:', dateStr, err);
    return 'Ungültiges Datum';
  }
}

/**
 * Kurzes Format mit Wochentag: "Mo, 28.10.2025"
 * Kompakt aber mit Kontext
 *
 * @param dateStr - ISO String oder Date
 * @returns "Mo, 28.10.2025"
 */
export function formatDateShort(dateStr: string | Date): string {
  try {
    const date = typeof dateStr === 'string' ? parseISO(dateStr) : dateStr;
    return format(date, 'EEE, dd.MM.yyyy', { locale: de });
  } catch (err) {
    console.error('❌ [formatDateShort] Invalid date:', dateStr, err);
    return 'Ungültiges Datum';
  }
}

/**
 * Relativer Zeitstempel: "vor 2 Stunden", "in 3 Tagen"
 * Perfekt für "Erstellt am" oder "Geändert am"
 *
 * @param dateStr - ISO String oder Date
 * @returns "vor 2 Stunden"
 */
export function formatRelative(dateStr: string | Date): string {
  try {
    const date = typeof dateStr === 'string' ? parseISO(dateStr) : dateStr;
    return formatDistance(date, new Date(), {
      addSuffix: true,
      locale: de,
    });
  } catch (err) {
    console.error('❌ [formatRelative] Invalid date:', dateStr, err);
    return 'Ungültiges Datum';
  }
}

/**
 * Format mit Uhrzeit: "28.10.2025, 14:30"
 * Für Timestamps mit Uhrzeit
 *
 * @param dateStr - ISO String mit Zeit oder Date
 * @returns "28.10.2025, 14:30"
 */
export function formatDateTime(dateStr: string | Date): string {
  try {
    const date = typeof dateStr === 'string' ? parseISO(dateStr) : dateStr;
    return format(date, 'dd.MM.yyyy, HH:mm', { locale: de });
  } catch (err) {
    console.error('❌ [formatDateTime] Invalid date:', dateStr, err);
    return 'Ungültiges Datum';
  }
}

/**
 * Nur Uhrzeit: "14:30"
 *
 * @param dateStr - ISO String mit Zeit oder Date
 * @returns "14:30"
 */
export function formatTime(dateStr: string | Date): string {
  try {
    const date = typeof dateStr === 'string' ? parseISO(dateStr) : dateStr;
    return format(date, 'HH:mm', { locale: de });
  } catch (err) {
    console.error('❌ [formatTime] Invalid date:', dateStr, err);
    return '--:--';
  }
}

/**
 * ISO Format für Backend/Datenbank: "2025-10-28"
 * Nutze dies wenn du Datum an Backend schickst
 *
 * @param date - Date Objekt
 * @returns "2025-10-28"
 */
export function formatDateISO(date: Date): string {
  try {
    return format(date, 'yyyy-MM-dd');
  } catch (err) {
    console.error('❌ [formatDateISO] Invalid date:', date, err);
    return '';
  }
}

/**
 * Parse deutsches Datum zu ISO: "28.10.2025" → "2025-10-28"
 * Nutze für User-Input Parsing
 *
 * @param germanDate - "DD.MM.YYYY"
 * @returns "2025-10-28" oder null bei Fehler
 */
export function parseGermanDate(germanDate: string): string | null {
  try {
    const [day, month, year] = germanDate.split('.');
    if (!day || !month || !year) return null;

    const date = new Date(parseInt(year), parseInt(month) - 1, parseInt(day));
    if (isNaN(date.getTime())) return null;

    return formatDateISO(date);
  } catch (err) {
    console.error('❌ [parseGermanDate] Invalid format:', germanDate, err);
    return null;
  }
}

/**
 * Berechne Anzahl Tage zwischen zwei Daten
 * Nutze für Buchungs-Dauer Anzeige
 *
 * @param startStr - Start-Datum (ISO oder Date)
 * @param endStr - End-Datum (ISO oder Date)
 * @returns Anzahl Tage (positiv/negativ)
 */
export function daysBetween(startStr: string | Date, endStr: string | Date): number {
  try {
    const start = typeof startStr === 'string' ? parseISO(startStr) : startStr;
    const end = typeof endStr === 'string' ? parseISO(endStr) : endStr;
    return differenceInDays(end, start);
  } catch (err) {
    console.error('❌ [daysBetween] Invalid dates:', startStr, endStr, err);
    return 0;
  }
}

/**
 * Formatiere Datums-Bereich: "28.10. - 31.10.2025"
 * Smart: Wenn gleiches Jahr/Monat, zeige kompakt
 *
 * @param startStr - Start-Datum
 * @param endStr - End-Datum
 * @returns "28.10. - 31.10.2025" oder "28.10.2025 - 02.11.2025"
 */
export function formatDateRange(startStr: string | Date, endStr: string | Date): string {
  try {
    const start = typeof startStr === 'string' ? parseISO(startStr) : startStr;
    const end = typeof endStr === 'string' ? parseISO(endStr) : endStr;

    const startYear = start.getFullYear();
    const endYear = end.getFullYear();
    const startMonth = start.getMonth();
    const endMonth = end.getMonth();

    // Gleiches Jahr und Monat → Kompakt
    if (startYear === endYear && startMonth === endMonth) {
      return `${format(start, 'dd.', { locale: de })} - ${format(end, 'dd.MM.yyyy', { locale: de })}`;
    }

    // Gleiches Jahr → Monat/Tag kompakt
    if (startYear === endYear) {
      return `${format(start, 'dd.MM.', { locale: de })} - ${format(end, 'dd.MM.yyyy', { locale: de })}`;
    }

    // Verschiedene Jahre → Voll
    return `${formatDate(start)} - ${formatDate(end)}`;
  } catch (err) {
    console.error('❌ [formatDateRange] Invalid dates:', startStr, endStr, err);
    return 'Ungültiger Bereich';
  }
}

/**
 * Validiere ob String ein gültiges Datum ist
 *
 * @param dateStr - Zu prüfender String
 * @returns true wenn gültig
 */
export function isValidDate(dateStr: string): boolean {
  try {
    const date = parseISO(dateStr);
    return !isNaN(date.getTime());
  } catch {
    return false;
  }
}

/**
 * Aktuelles Datum als ISO String
 *
 * @returns "2025-10-28"
 */
export function todayISO(): string {
  return formatDateISO(new Date());
}

/**
 * Legacy Compatibility: Ersetzt toLocaleDateString('de-DE')
 * DEPRECATED: Nutze stattdessen formatDate()
 * Nur für schrittweise Migration
 *
 * @param dateStr - ISO String oder Date
 * @returns "28.10.2025"
 */
export function toGermanDate(dateStr: string | Date): string {
  console.warn('⚠️ [toGermanDate] DEPRECATED - Use formatDate() instead');
  return formatDate(dateStr);
}
