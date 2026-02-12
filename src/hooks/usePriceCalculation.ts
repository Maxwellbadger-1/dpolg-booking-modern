/**
 * usePriceCalculation Hook
 *
 * Single Source of Truth für ALLE Preisberechnungen.
 * Dieser Hook ruft das Backend auf und cached die Ergebnisse.
 *
 * WICHTIG: Frontend berechnet NIEMALS selbst!
 * Alle Berechnungen erfolgen im Backend (pricing.rs)
 *
 * Usage:
 * ```typescript
 * const { priceBreakdown, loading, error } = usePriceCalculation({
 *   roomId: 1,
 *   checkin: '2025-01-01',
 *   checkout: '2025-01-03',
 *   isMember: true,
 *   services: [{
 *     name: 'Frühbucher',
 *     priceType: 'percent',
 *     originalValue: 10,
 *     appliesTo: 'overnight_price'
 *   }],
 *   discounts: []
 * });
 * ```
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TypeScript Interfaces (match Rust structs)
// ============================================================================

export interface ServiceInput {
  name: string;
  priceType: 'fixed' | 'percent';
  originalValue: number;
  appliesTo: 'overnight_price' | 'total_price';
}

export interface DiscountInput {
  name: string;
  discountType: 'fixed' | 'percent';
  value: number;
}

export interface ServiceCalculation {
  name: string;
  priceType: string;
  originalValue: number;
  appliesTo: string;
  calculatedPrice: number;
  baseAmount?: number;
}

export interface DiscountCalculation {
  name: string;
  discountType: string;
  originalValue: number;
  calculatedAmount: number;
  baseAmount?: number;
}

export interface FullPriceBreakdown {
  basePrice: number;
  nights: number;
  pricePerNight: number;
  isHauptsaison: boolean;
  hauptsaisonNights: number;
  nebensaisonNights: number;
  services: ServiceCalculation[];
  servicesTotal: number;
  discounts: DiscountCalculation[];
  discountsTotal: number;
  subtotal: number;
  total: number;
  calculationTimestamp: string;
}

export interface PriceCalculationInput {
  roomId: number;
  checkin: string;
  checkout: string;
  isMember: boolean;
  services: ServiceInput[];
  discounts: DiscountInput[];
}

// ============================================================================
// Main Hook
// ============================================================================

export function usePriceCalculation(input: PriceCalculationInput | null) {
  const [priceBreakdown, setPriceBreakdown] = useState<FullPriceBreakdown | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Cache für Performance (verhindert unnötige Backend-Calls)
  const cacheKey = input ? JSON.stringify(input) : null;
  const cache = useRef<Map<string, FullPriceBreakdown>>(new Map());

  const calculate = useCallback(async () => {
    if (!input) {
      setPriceBreakdown(null);
      return;
    }

    // Validierung: Mindestdaten müssen vorhanden sein
    if (!input.roomId || !input.checkin || !input.checkout) {
      setPriceBreakdown(null);
      return;
    }

    // Validiere Datums-Strings vor Backend-Call
    if (!input.checkin || !input.checkout ||
        input.checkin === '' || input.checkout === '') {
      setPriceBreakdown(null);
      return;
    }

    // Validiere Datum-Format
    try {
      const checkinDate = new Date(input.checkin);
      const checkoutDate = new Date(input.checkout);
      if (isNaN(checkinDate.getTime()) || isNaN(checkoutDate.getTime())) {
        setPriceBreakdown(null);
        return;
      }
    } catch {
      setPriceBreakdown(null);
      return;
    }

    // Cache-Check
    if (cacheKey) {
      const cached = cache.current.get(cacheKey);
      if (cached) {
        setPriceBreakdown(cached);
        return;
      }
    }

    setLoading(true);
    setError(null);

    try {
      // DEBUG: Log what we're sending to backend
      console.log('═══════════════════════════════════════════════════════════════');
      console.log('🔍 usePriceCalculation - SENDING TO BACKEND');
      console.log('═══════════════════════════════════════════════════════════════');
      console.log('📤 roomId:', input.roomId);
      console.log('📤 checkin:', input.checkin);
      console.log('📤 checkout:', input.checkout);
      console.log('📤 isMember:', input.isMember);
      console.log('📤 services:', input.services.length, 'items');
      input.services.forEach((s, i) => {
        console.log(`   [${i}] name='${s.name}', originalValue=${s.originalValue}, priceType='${s.priceType}', appliesTo='${s.appliesTo}'`);
      });
      console.log('📤 discounts:', input.discounts.length, 'items');
      input.discounts.forEach((d, i) => {
        console.log(`   [${i}] name='${d.name}', value=${d.value}, discountType='${d.discountType}'`);
      });
      console.log('═══════════════════════════════════════════════════════════════');

      // EINE Stelle für Backend-Call - Single Source of Truth!
      const result = await invoke<FullPriceBreakdown>(
        'calculate_full_booking_price_pg',
        {
          roomId: input.roomId,
          checkin: input.checkin,
          checkout: input.checkout,
          isMember: input.isMember,
          services: input.services.length > 0 ? input.services : undefined,
          discounts: input.discounts.length > 0 ? input.discounts : undefined,
        }
      );

      console.log('✅ Backend Result:', result);

      // Cache speichern
      if (cacheKey) {
        cache.current.set(cacheKey, result);
      }

      setPriceBreakdown(result);
    } catch (err) {
      console.error('❌ Preisberechnung fehlgeschlagen:', err);
      setError(String(err));
      setPriceBreakdown(null);
    } finally {
      setLoading(false);
    }
  }, [cacheKey, input]);

  // Auto-Berechnung bei Input-Änderung
  useEffect(() => {
    calculate();
  }, [calculate]);

  // Event-Listener für globale Updates
  useEffect(() => {
    const handlePriceUpdate = () => {
      console.log('🔄 [usePriceCalculation] Global price recalculation triggered');
      cache.current.clear(); // Cache invalidieren
      calculate();
    };

    window.addEventListener('price-recalculation-needed', handlePriceUpdate);
    return () => window.removeEventListener('price-recalculation-needed', handlePriceUpdate);
  }, [calculate]);

  return {
    priceBreakdown,
    loading,
    error,
    recalculate: calculate,
  };
}

// ============================================================================
// Helper: Trigger global recalculation
// ============================================================================

/**
 * Trigger eine globale Preis-Neuberechnung.
 * Alle usePriceCalculation() Hooks invalidieren ihren Cache und berechnen neu.
 *
 * Use-Cases:
 * - Nach Änderung an Pricing Settings
 * - Nach Änderung an Zimmer-Preisen
 * - Nach Hinzufügen/Löschen von Service-Templates
 */
export function triggerGlobalPriceRecalculation() {
  console.log('🔄 [triggerGlobalPriceRecalculation] Dispatching global event');
  window.dispatchEvent(new CustomEvent('price-recalculation-needed'));
}
