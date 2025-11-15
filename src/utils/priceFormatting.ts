/**
 * Utility Functions für konsistente Preis-Formatierung
 * Single Source of Truth für alle Preis-Anzeigen
 */

import { ServiceTemplate } from '../types/booking';

export interface PriceableService {
  price: number;
  price_type: 'fixed' | 'percent';
  original_value?: number;
  applies_to?: 'overnight_price' | 'total_price';
  service_price?: number;
}

/**
 * Formatiert Service-Preis für Anzeige (Template-Ansicht)
 * @returns "19.0 %" oder "15.00 €"
 */
export function formatServicePrice(service: PriceableService): string {
  if (service.price_type === 'percent') {
    return `${service.price.toFixed(1)} %`;
  }
  return `${service.price.toFixed(2)} €`;
}

/**
 * Formatiert Service-Preis mit Symbol-Präfix
 * @returns "% 19.0" oder "€ 15.00"
 */
export function formatServicePriceWithSymbol(service: PriceableService): { symbol: string; value: string; suffix: string } {
  if (service.price_type === 'percent') {
    return {
      symbol: '%',
      value: service.price.toFixed(1),
      suffix: '%',
    };
  }
  return {
    symbol: '€',
    value: service.price.toFixed(2),
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
  if (service.price_type === 'percent') {
    const percentValue = service.original_value || service.price;

    if (service.applies_to === 'overnight_price') {
      // Prozent vom Grundpreis
      return grundpreis * (percentValue / 100);
    } else {
      // Prozent vom Gesamtpreis (Grundpreis + bisherige Services)
      return (grundpreis + currentServicesTotal) * (percentValue / 100);
    }
  }

  // Festbetrag
  return service.service_price || service.price;
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
  if (service.price_type === 'percent') {
    if (service.applies_to === 'overnight_price') {
      const price = calculateServicePrice(service, grundpreis, currentServicesTotal);
      return `${price.toFixed(2)} €`;
    } else {
      // total_price: Zu komplex für Frontend, Backend berechnet
      return '(berechnet)';
    }
  }

  return `${(service.service_price || service.price).toFixed(2)} €`;
}

/**
 * Gibt eine lesbare Beschreibung des Preis-Typs zurück
 * @returns "19.0% vom Grundpreis" oder "15.00 €"
 */
export function getServicePriceDescription(service: PriceableService): string {
  if (service.price_type === 'percent') {
    const percentValue = (service.original_value || service.price).toFixed(1);
    const basis = service.applies_to === 'overnight_price'
      ? 'vom Grundpreis'
      : 'vom Gesamtpreis';
    return `${percentValue}% ${basis}`;
  }

  return `${(service.service_price || service.price).toFixed(2)} €`;
}

/**
 * Prüft ob ein Service prozentual ist
 */
export function isPercentageService(service: PriceableService): boolean {
  return service.price_type === 'percent';
}

/**
 * Gibt das passende Icon für den Service-Typ zurück
 * @returns 'Euro' oder 'Percent' (Lucide React Icon namen)
 */
export function getServicePriceIcon(service: PriceableService): 'Euro' | 'Percent' {
  return service.price_type === 'percent' ? 'Percent' : 'Euro';
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
