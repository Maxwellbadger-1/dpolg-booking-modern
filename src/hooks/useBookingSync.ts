/**
 * useBookingSync Hook - Single Source of Truth for Booking Sync
 *
 * Eliminiert duplizierte sync_affected_dates() Aufrufe (10+ Duplikate).
 * Inspired by professional codebases (Notion, Linear, etc.)
 *
 * Pattern: Ein Hook für alle Sync-Operationen
 *
 * Usage:
 * ```typescript
 * const { syncBookingDates } = useBookingSync();
 *
 * // Nach Service/Discount Änderungen
 * await syncBookingDates({
 *   bookingId: 81,
 *   checkinDate: '2025-10-28',
 *   checkoutDate: '2025-10-30'
 * });
 * ```
 */

import { invoke } from '@tauri-apps/api/core';
import { useCallback } from 'react';

export interface SyncBookingDatesOptions {
  bookingId: number;
  checkinDate: string;
  checkoutDate: string;
  oldCheckoutDate?: string; // Optional: For date range changes
}

export interface UseBookingSyncReturn {
  syncBookingDates: (options: SyncBookingDatesOptions) => Promise<string>;
  syncBookingDatesQuiet: (options: SyncBookingDatesOptions) => void;
}

/**
 * Custom hook for syncing booking changes to Turso (Mobile App).
 *
 * Consolidates the duplicated sync_affected_dates() pattern that appears
 * 10+ times across BookingSidebar, BookingDetails, and DataContext.
 *
 * @returns Sync control object with syncBookingDates function
 */
export function useBookingSync(): UseBookingSyncReturn {
  /**
   * Synchronizes booking changes to Turso database.
   * Used after Service/Discount/Date changes to update mobile app emojis.
   *
   * @param options - Booking sync parameters
   * @returns Promise with sync result message
   * @throws Error if sync fails
   */
  const syncBookingDates = useCallback(async (options: SyncBookingDatesOptions): Promise<string> => {
    const { bookingId, checkinDate, checkoutDate, oldCheckoutDate } = options;

    console.log('🔄 [useBookingSync] Sync zu Turso:', {
      bookingId,
      checkinDate,
      checkoutDate,
      oldCheckoutDate,
    });

    try {
      // WICHTIG: Tauri konvertiert automatisch camelCase → snake_case
      // Frontend: bookingId, checkinDate, oldCheckout, newCheckout
      // Backend: booking_id, checkin_date, old_checkout, new_checkout
      const result = await invoke<string>('sync_affected_dates', {
        bookingId,
        checkinDate,
        oldCheckout: oldCheckoutDate || checkoutDate,
        newCheckout: checkoutDate,
      });

      console.log('✅ [useBookingSync] Sync erfolgreich:', result);
      return result;
    } catch (error) {
      console.error('❌ [useBookingSync] Sync Fehler:', error);
      throw error;
    }
  }, []);

  /**
   * "Fire and forget" version of syncBookingDates.
   * Triggers sync but doesn't wait for completion or handle errors explicitly.
   *
   * Use this when:
   * - You don't need to wait for sync to complete
   * - Sync errors shouldn't block the UI
   * - Optimistic UI updates are more important than sync confirmation
   *
   * @param options - Booking sync parameters
   */
  const syncBookingDatesQuiet = useCallback((options: SyncBookingDatesOptions): void => {
    syncBookingDates(options).catch((error) => {
      // Error already logged in syncBookingDates
      console.warn('⚠️  [useBookingSync] Quiet sync failed (non-blocking):', error);
    });
  }, [syncBookingDates]);

  return {
    syncBookingDates,
    syncBookingDatesQuiet,
  };
}

/**
 * Helper function for components that can't use hooks.
 * Direct function call version of useBookingSync.
 *
 * Usage:
 * ```typescript
 * await syncBooking({
 *   bookingId: 81,
 *   checkinDate: '2025-10-28',
 *   checkoutDate: '2025-10-30'
 * });
 * ```
 */
export async function syncBooking(options: SyncBookingDatesOptions): Promise<string> {
  const { bookingId, checkinDate, checkoutDate, oldCheckoutDate } = options;

  console.log('🔄 [syncBooking] Sync zu Turso:', {
    bookingId,
    checkinDate,
    checkoutDate,
    oldCheckoutDate,
  });

  try {
    const result = await invoke<string>('sync_affected_dates', {
      bookingId,
      checkinDate,
      oldCheckout: oldCheckoutDate || checkoutDate,
      newCheckout: checkoutDate,
    });

    console.log('✅ [syncBooking] Sync erfolgreich:', result);
    return result;
  } catch (error) {
    console.error('❌ [syncBooking] Sync Fehler:', error);
    throw error;
  }
}
