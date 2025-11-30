// Helper Functions for TapeChart

export interface BookingChangeData {
  bookingId: number;
  reservierungsnummer: string;
  guestName: string;
  roomName: string;
  oldData: {
    checkin_date: string;
    checkout_date: string;
    room_id: number;
    gesamtpreis: number;
  };
  newData: {
    checkin_date: string;
    checkout_date: string;
    room_id: number;
    gesamtpreis: number;
  };
}

// Filter bookings based on search query, status, and room type
// NORMALIZED STATE: Now accepts guestMap and roomMap for O(1) lookups
export function filterBookings(
  bookings: any[],
  searchQuery: string,
  statusFilter: string,
  roomTypeFilter: string,
  guestMap?: Map<number, any>,
  roomMap?: Map<number, any>
): any[] {
  return bookings.filter((booking) => {
    // Search filter (guest name or reservation number)
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      // NORMALIZED STATE: Look up guest from Map
      const guest = guestMap?.get(booking.guest_id);
      const guestName = guest ? `${guest.vorname} ${guest.nachname}`.toLowerCase() : '';
      const reservierung = booking.reservierungsnummer.toLowerCase();

      if (!guestName.includes(query) && !reservierung.includes(query)) {
        return false;
      }
    }

    // Status filter - Professional PMS Pattern:
    // Exclude 'storniert' bookings by default (even when showing 'all')
    // Only show them when explicitly selected via 'storniert' filter
    if (statusFilter === 'all') {
      // When showing "all", exclude cancelled bookings
      if (booking.status === 'storniert') {
        return false;
      }
    } else if (statusFilter === 'stiftungsfall') {
      // Special case: 'stiftungsfall' is NOT a status, it's a boolean flag
      // Show only bookings where ist_stiftungsfall === true (regardless of status)
      if (!booking.ist_stiftungsfall) {
        return false;
      }
    } else if (booking.status !== statusFilter) {
      // When specific status is selected, only show that status
      return false;
    }

    // Room type filter
    // NORMALIZED STATE: Look up room from Map
    if (roomTypeFilter !== 'all') {
      const room = roomMap?.get(booking.room_id);
      if (room?.gebaeude_typ !== roomTypeFilter) {
        return false;
      }
    }

    return true;
  });
}

// Get unique room types from rooms
export function getUniqueRoomTypes(rooms: any[]): string[] {
  const types = new Set(rooms.map((r) => r.gebaeude_typ));
  return Array.from(types).sort();
}

// Calculate price for booking (placeholder - should use backend logic)
export async function calculateBookingPrice(
  roomId: number,
  checkin: string,
  checkout: string,
  guestId: number
): Promise<number> {
  // This would call the backend pricing calculation
  // For now, return a placeholder
  return 100;
}
