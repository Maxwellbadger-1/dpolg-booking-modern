/**
 * Utility Functions für konsistente Preis-Formatierung
 * Single Source of Truth für alle Preis-Anzeigen
 */

import { ServiceTemplate } from '../types/booking';

export interface PriceableService {
  price: number;
  // Unterstützt beide Namenskonventionen (camelCase vom Backend, snake_case legacy)
  price_type?: 'fixed' | 'percent';
  priceType?: 'fixed' | 'percent';
  original_value?: number;
  applies_to?: 'overnight_price' | 'total_price';
  appliesTo?: 'overnight_price' | 'total_price';
  service_price?: number;
}

// Helper um priceType unabhängig von der Namenskonvention zu bekommen
function getPriceType(service: PriceableService): 'fixed' | 'percent' {
  return service.priceType || service.price_type || 'fixed';
}

// Helper um appliesTo unabhängig von der Namenskonvention zu bekommen
function getAppliesTo(service: PriceableService): 'overnight_price' | 'total_price' {
  return service.appliesTo || service.applies_to || 'overnight_price';
}

/**
 * Formatiert Service-Preis für Anzeige (Template-Ansicht)
 * @returns "19.0 %" oder "15.00 €"
 */
export function formatServicePrice(service: PriceableService): string {
  const priceValue = service.price ?? service.original_value ?? 0;
  if (getPriceType(service) === 'percent') {
    return `${priceValue.toFixed(1)} %`;
  }
  return `${priceValue.toFixed(2)} €`;
}

/**
 * Formatiert Service-Preis mit Symbol-Präfix
 * @returns "% 19.0" oder "€ 15.00"
 */
export function formatServicePriceWithSymbol(service: PriceableService): { symbol: string; value: string; suffix: string } {
  const priceValue = service.price ?? service.original_value ?? 0;
  if (getPriceType(service) === 'percent') {
    return {
      symbol: '%',
      value: priceValue.toFixed(1),
      suffix: '%',
    };
  }
  return {
    symbol: '€',
    value: priceValue.toFixed(2),
    suffix: '€',
  };
}

/**
 * Berechnet den tatsächlichen Preis eines Services in einer Buchung
 * Für prozentuale Services: Berechnet % vom Grundpreis/Gesamtpreis
 * Für Festbeträge: Gibt service_price zurück
 */
export function calculateServicePrice(
  service: PriceableService,
  grundpreis: number,
  currentServicesTotal: number = 0
): number {
  if (getPriceType(service) === 'percent') {
    const percentValue = service.price ?? service.original_value ?? 0;

    if (getAppliesTo(service) === 'overnight_price') {
      // Prozent vom Grundpreis
      return grundpreis * (percentValue / 100);
    } else {
      // Prozent vom Gesamtpreis (Grundpreis + bisherige Services)
      return (grundpreis + currentServicesTotal) * (percentValue / 100);
    }
  }

  // Festbetrag
  return service.service_price ?? service.price ?? service.original_value ?? 0;
}

/**
 * Formatiert berechneten Service-Preis für Anzeige in Buchung
 * @returns "28.50 €" oder "(berechnet)"
 */
export function formatCalculatedServicePrice(
  service: PriceableService,
  grundpreis: number,
  currentServicesTotal: number = 0
): string {
  if (getPriceType(service) === 'percent') {
    if (getAppliesTo(service) === 'overnight_price') {
      const price = calculateServicePrice(service, grundpreis, currentServicesTotal);
      return `${price.toFixed(2)} €`;
    } else {
      // total_price: Zu komplex für Frontend, Backend berechnet
      return '(berechnet)';
    }
  }

  const priceValue = service.service_price ?? service.price ?? service.original_value ?? 0;
  return `${priceValue.toFixed(2)} €`;
}

/**
 * Gibt eine lesbare Beschreibung des Preis-Typs zurück
 * @returns "19.0% vom Grundpreis" oder "15.00 €"
 */
export function getServicePriceDescription(service: PriceableService): string {
  if (getPriceType(service) === 'percent') {
    const percentValue = (service.price ?? service.original_value ?? 0).toFixed(1);
    const basis = getAppliesTo(service) === 'overnight_price'
      ? 'vom Grundpreis'
      : 'vom Gesamtpreis';
    return `${percentValue}% ${basis}`;
  }

  const priceValue = service.service_price ?? service.price ?? service.original_value ?? 0;
  return `${priceValue.toFixed(2)} €`;
}

/**
 * Prüft ob ein Service prozentual ist
 */
export function isPercentageService(service: PriceableService): boolean {
  return getPriceType(service) === 'percent';
}

/**
 * Gibt das passende Icon für den Service-Typ zurück
 * @returns 'Euro' oder 'Percent' (Lucide React Icon namen)
 */
export function getServicePriceIcon(service: PriceableService): 'Euro' | 'Percent' {
  return getPriceType(service) === 'percent' ? 'Percent' : 'Euro';
}

/**
 * DISCOUNT UTILITIES
 */

export interface PriceableDiscount {
  discount_value: number;
  discount_type: 'fixed' | 'percent';
  applies_to?: 'overnight_price' | 'total_price';
}

/**
 * Formatiert Rabatt für Anzeige (Template-Ansicht)
 * @returns "10.0 %" oder "50.00 €"
 */
export function formatDiscountValue(discount: PriceableDiscount): string {
  if (discount.discount_type === 'percent') {
    return `${discount.discount_value.toFixed(1)} %`;
  }
  return `${discount.discount_value.toFixed(2)} €`;
}

/**
 * Gibt das passende Icon für den Rabatt-Typ zurück
 * @returns 'Euro' oder 'Percent' (Lucide React Icon namen)
 */
export function getDiscountIcon(discount: PriceableDiscount): 'Euro' | 'Percent' {
  return discount.discount_type === 'percent' ? 'Percent' : 'Euro';
}
