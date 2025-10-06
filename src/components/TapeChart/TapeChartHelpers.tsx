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
export function filterBookings(
  bookings: any[],
  searchQuery: string,
  statusFilter: string,
  roomTypeFilter: string
): any[] {
  return bookings.filter((booking) => {
    // Search filter (guest name or reservation number)
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      const guestName = `${booking.guest.vorname} ${booking.guest.nachname}`.toLowerCase();
      const reservierung = booking.reservierungsnummer.toLowerCase();

      if (!guestName.includes(query) && !reservierung.includes(query)) {
        return false;
      }
    }

    // Status filter
    if (statusFilter !== 'all' && booking.status !== statusFilter) {
      return false;
    }

    // Room type filter
    if (roomTypeFilter !== 'all' && booking.room.gebaeude_typ !== roomTypeFilter) {
      return false;
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
