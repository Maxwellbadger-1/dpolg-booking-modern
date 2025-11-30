/**
 * useBatchPriceCalculation Hook
 *
 * Berechnet Preise für mehrere Buchungen auf einmal (Batch-Operation).
 * Verwendet für Listen-Ansichten: BookingList, GuestDetails, StatisticsView, TapeChart
 *
 * WICHTIG: Dieser Hook ist effizienter als mehrfache Einzelaufrufe von usePriceCalculation!
 *
 * Usage:
 * ```typescript
 * const { priceMap, loading, error, refetch } = useBatchPriceCalculation(bookingIds);
 *
 * // Get price for specific booking
 * const price = priceMap.get(bookingId);
 * console.log(price?.total, price?.nights);
 * ```
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TypeScript Interfaces
// ============================================================================

export interface BookingPriceResult {
  bookingId: number;
  nights: number;
  basePrice: number;
  servicesTotal: number;
  discountsTotal: number;
  total: number;
}

// ============================================================================
// Main Hook
// ============================================================================

export function useBatchPriceCalculation(bookingIds: number[]) {
  const [priceMap, setPriceMap] = useState<Map<number, BookingPriceResult>>(new Map());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Cache für Performance
  const lastIdsRef = useRef<string>('');

  const fetchPrices = useCallback(async () => {
    if (!bookingIds || bookingIds.length === 0) {
      setPriceMap(new Map());
      return;
    }

    // Check if IDs changed
    const idsKey = bookingIds.sort((a, b) => a - b).join(',');
    if (idsKey === lastIdsRef.current && priceMap.size > 0) {
      return; // Already loaded
    }

    setLoading(true);
    setError(null);

    try {
      console.log('🔄 [useBatchPriceCalculation] Fetching prices for', bookingIds.length, 'bookings');

      const results = await invoke<BookingPriceResult[]>(
        'calculate_batch_booking_prices_pg',
        { bookingIds }
      );

      // Convert array to Map for O(1) lookup
      const newMap = new Map<number, BookingPriceResult>();
      results.forEach(result => {
        newMap.set(result.bookingId, result);
      });

      setPriceMap(newMap);
      lastIdsRef.current = idsKey;

      console.log('✅ [useBatchPriceCalculation] Loaded prices for', results.length, 'bookings');
    } catch (err) {
      console.error('❌ [useBatchPriceCalculation] Error:', err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [bookingIds]);

  // Auto-fetch when IDs change
  useEffect(() => {
    fetchPrices();
  }, [fetchPrices]);

  // Listen for global price recalculation events
  useEffect(() => {
    const handlePriceUpdate = () => {
      console.log('🔄 [useBatchPriceCalculation] Global recalculation triggered');
      lastIdsRef.current = ''; // Clear cache
      fetchPrices();
    };

    window.addEventListener('price-recalculation-needed', handlePriceUpdate);
    return () => window.removeEventListener('price-recalculation-needed', handlePriceUpdate);
  }, [fetchPrices]);

  return {
    priceMap,
    loading,
    error,
    refetch: fetchPrices,
  };
}

// ============================================================================
// Helper: Get price for single booking from map
// ============================================================================

/**
 * Helper function to get price data for a booking, with fallback defaults
 */
export function getBookingPrice(
  priceMap: Map<number, BookingPriceResult>,
  bookingId: number
): BookingPriceResult {
  return priceMap.get(bookingId) || {
    bookingId,
    nights: 0,
    basePrice: 0,
    servicesTotal: 0,
    discountsTotal: 0,
    total: 0,
  };
}
