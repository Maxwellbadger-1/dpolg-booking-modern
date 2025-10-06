import { createContext, useContext, useState, useCallback, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Types
interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  price_member: number;
  price_non_member: number;
  ort: string;
  schluesselcode?: string;
}

interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon: string;
  strasse?: string;
  plz?: string;
  ort?: string;
  dpolg_mitglied: boolean;
  mitgliedsnummer?: string;
  notizen?: string;
}

interface Booking {
  id: number;
  room_id: number;
  guest_id: number;
  reservierungsnummer: string;
  checkin_date: string;
  checkout_date: string;
  status: string;
  gesamtpreis: number;
  bemerkungen?: string;
  bezahlt: boolean;
  bezahlt_am?: string | null;
  zahlungsmethode?: string | null;
  room: Room;
  guest: Guest;
}

// Context Type
interface DataContextType {
  // Data
  rooms: Room[];
  guests: Guest[];
  bookings: Booking[];
  loading: boolean;

  // Refresh Functions
  refreshRooms: () => Promise<void>;
  refreshGuests: () => Promise<void>;
  refreshBookings: () => Promise<void>;
  refreshAll: () => Promise<void>;

  // CRUD Operations (trigger auto-refresh)
  createRoom: (data: any) => Promise<Room>;
  updateRoom: (id: number, data: any) => Promise<Room>;
  deleteRoom: (id: number) => Promise<void>;

  createGuest: (data: any) => Promise<Guest>;
  updateGuest: (id: number, data: any) => Promise<Guest>;
  deleteGuest: (id: number) => Promise<void>;

  createBooking: (data: any) => Promise<Booking>;
  updateBooking: (id: number, data: any) => Promise<Booking>;
  deleteBooking: (id: number) => Promise<void>;

  // Optimistic Updates
  updateBookingStatus: (id: number, status: string) => Promise<void>;
  updateBookingPayment: (id: number, isPaid: boolean, zahlungsmethode?: string) => Promise<void>;
}

// Context
const DataContext = createContext<DataContextType | undefined>(undefined);

// Provider Component
export function DataProvider({ children }: { children: ReactNode }) {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [guests, setGuests] = useState<Guest[]>([]);
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [loading, setLoading] = useState(false);

  // Refresh Functions
  const refreshRooms = useCallback(async () => {
    try {
      const data = await invoke<Room[]>('get_all_rooms');
      setRooms(data);
    } catch (error) {
      console.error('Fehler beim Laden der Zimmer:', error);
      throw error;
    }
  }, []);

  const refreshGuests = useCallback(async () => {
    try {
      const data = await invoke<Guest[]>('get_all_guests_command');
      setGuests(data);
    } catch (error) {
      console.error('Fehler beim Laden der Gäste:', error);
      throw error;
    }
  }, []);

  const refreshBookings = useCallback(async () => {
    try {
      const data = await invoke<Booking[]>('get_all_bookings');
      setBookings(data);
    } catch (error) {
      console.error('Fehler beim Laden der Buchungen:', error);
      throw error;
    }
  }, []);

  const refreshAll = useCallback(async () => {
    setLoading(true);
    try {
      await Promise.all([refreshRooms(), refreshGuests(), refreshBookings()]);
    } finally {
      setLoading(false);
    }
  }, [refreshRooms, refreshGuests, refreshBookings]);

  // Initial data load on mount
  useEffect(() => {
    refreshAll();

    // Listen ONLY for UNDO operations (not regular mutations!)
    const handleUndoRefresh = () => {
      console.log('🔄 Global data refresh triggered by UNDO operation');
      refreshAll();
    };

    window.addEventListener('undo-executed', handleUndoRefresh);
    return () => window.removeEventListener('undo-executed', handleUndoRefresh);
  }, [refreshAll]);

  // Room CRUD Operations
  const createRoom = useCallback(async (data: any): Promise<Room> => {
    try {
      // 1. Backend Create
      const room = await invoke<Room>('create_room_command', data);

      // 2. SOFORT zum State hinzufügen (Optimistic Update)
      setRooms(prev => [...prev, room]);

      // 3. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return room;
    } catch (error) {
      // Kein Rollback nötig - Room wurde noch nicht hinzugefügt
      throw error;
    }
  }, []);

  const updateRoom = useCallback(async (id: number, data: any): Promise<Room> => {
    // 1. Backup für Rollback
    const oldRoom = rooms.find(r => r.id === id);

    // 2. SOFORT im UI ändern (Optimistic Update)
    setRooms(prev => prev.map(r =>
      r.id === id ? { ...r, ...data } : r
    ));

    try {
      // 3. Backend Update
      const room = await invoke<Room>('update_room_command', { id, ...data });

      // 4. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return room;
    } catch (error) {
      // 5. Rollback bei Fehler
      if (oldRoom) {
        setRooms(prev => prev.map(r =>
          r.id === id ? oldRoom : r
        ));
      }
      throw error;
    }
  }, [rooms]);

  const deleteRoom = useCallback(async (id: number): Promise<void> => {
    // 1. Backup für Rollback
    const deletedRoom = rooms.find(r => r.id === id);

    // 2. SOFORT aus UI entfernen (Optimistic Update)
    setRooms(prev => prev.filter(r => r.id !== id));

    try {
      // 3. Backend Delete
      await invoke('delete_room_command', { id });

      // 4. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));
    } catch (error) {
      // 5. Rollback - Room wiederherstellen
      if (deletedRoom) {
        setRooms(prev => [...prev, deletedRoom]);
      }
      throw error;
    }
  }, [rooms]);

  // Guest CRUD Operations
  const createGuest = useCallback(async (data: any): Promise<Guest> => {
    try {
      // 1. Backend Create
      const guest = await invoke<Guest>('create_guest_command', data);

      // 2. SOFORT zum State hinzufügen (Optimistic Update)
      setGuests(prev => [...prev, guest]);

      // 3. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return guest;
    } catch (error) {
      // Kein Rollback nötig - Guest wurde noch nicht hinzugefügt
      throw error;
    }
  }, []);

  const updateGuest = useCallback(async (id: number, data: any): Promise<Guest> => {
    // 1. Backup für Rollback
    const oldGuest = guests.find(g => g.id === id);

    // 2. SOFORT im UI ändern (Optimistic Update)
    setGuests(prev => prev.map(g =>
      g.id === id ? { ...g, ...data } : g
    ));

    try {
      // 3. Backend Update
      const guest = await invoke<Guest>('update_guest_command', { id, ...data });

      // 4. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return guest;
    } catch (error) {
      // 5. Rollback bei Fehler
      if (oldGuest) {
        setGuests(prev => prev.map(g =>
          g.id === id ? oldGuest : g
        ));
      }
      throw error;
    }
  }, [guests]);

  const deleteGuest = useCallback(async (id: number): Promise<void> => {
    // 1. Backup für Rollback
    const deletedGuest = guests.find(g => g.id === id);

    // 2. SOFORT aus UI entfernen (Optimistic Update)
    setGuests(prev => prev.filter(g => g.id !== id));

    try {
      // 3. Backend Delete
      await invoke('delete_guest_command', { id });

      // 4. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));
    } catch (error) {
      // 5. Rollback - Guest wiederherstellen
      if (deletedGuest) {
        setGuests(prev => [...prev, deletedGuest]);
      }
      throw error;
    }
  }, [guests]);

  // Booking CRUD Operations
  const createBooking = useCallback(async (data: any): Promise<Booking> => {
    console.log('🔍 [DataContext] createBooking aufgerufen mit:', data);
    try {
      // 1. Backend Create
      const booking = await invoke<Booking>('create_booking_command', data);
      console.log('✅ [DataContext] Buchung erstellt:', booking);

      // 2. SOFORT zum State hinzufügen (Optimistic Update)
      setBookings(prev => [...prev, booking]);

      // 3. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return booking;
    } catch (error) {
      console.error('❌ [DataContext] Fehler beim Erstellen der Buchung:', error);
      // Kein Rollback nötig - Booking wurde noch nicht hinzugefügt
      throw error;
    }
  }, []);

  const updateBooking = useCallback(async (id: number, data: any): Promise<Booking> => {
    // 1. Backup für Rollback
    const oldBooking = bookings.find(b => b.id === id);

    // 2. SOFORT im UI ändern (Optimistic Update)
    setBookings(prev => prev.map(b =>
      b.id === id ? { ...b, ...data } : b
    ));

    try {
      // 3. Backend Update
      const booking = await invoke<Booking>('update_booking_command', { id, ...data });

      // 4. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return booking;
    } catch (error) {
      // 5. Rollback bei Fehler
      if (oldBooking) {
        setBookings(prev => prev.map(b =>
          b.id === id ? oldBooking : b
        ));
      }
      throw error;
    }
  }, [bookings]);

  const deleteBooking = useCallback(async (id: number): Promise<void> => {
    // 1. Backup für Rollback
    const deletedBooking = bookings.find(b => b.id === id);

    // 2. SOFORT aus UI entfernen (Optimistic Update)
    setBookings(prev => prev.filter(b => b.id !== id));

    try {
      // 3. Backend Delete
      await invoke('delete_booking_command', { id });

      // 4. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));
    } catch (error) {
      // 5. Rollback - Booking wiederherstellen
      if (deletedBooking) {
        setBookings(prev => [...prev, deletedBooking]);
      }
      throw error;
    }
  }, [bookings]);

  // Optimistic Update für Booking Status
  const updateBookingStatus = useCallback(async (id: number, status: string): Promise<void> => {
    // 1. Alte Buchung sichern für Rollback
    const oldBooking = bookings.find(b => b.id === id);

    // 2. SOFORT im UI ändern (Optimistic Update)
    setBookings(prev => prev.map(b =>
      b.id === id ? { ...b, status } : b
    ));

    try {
      // 3. Backend Update
      await invoke('update_booking_status_command', { bookingId: id, newStatus: status });

      // 4. Trigger refresh-data event for Undo button (OHNE refreshBookings!)
      window.dispatchEvent(new CustomEvent('refresh-data'));
    } catch (error) {
      // 5. Rollback bei Fehler - alte Buchung wiederherstellen
      if (oldBooking) {
        setBookings(prev => prev.map(b =>
          b.id === id ? oldBooking : b
        ));
      }
      console.error('Fehler beim Aktualisieren des Buchungsstatus:', error);
      throw error;
    }
  }, [bookings]);

  // Optimistic Update für Booking Payment
  const updateBookingPayment = useCallback(async (id: number, isPaid: boolean, zahlungsmethode?: string): Promise<void> => {
    console.log('🔍 [DataContext] updateBookingPayment called:', { id, isPaid, zahlungsmethode });

    // 1. Alte Buchung sichern für Rollback
    const oldBooking = bookings.find(b => b.id === id);

    // 2. SOFORT im UI ändern (Optimistic Update)
    setBookings(prev => prev.map(b =>
      b.id === id ? {
        ...b,
        bezahlt: isPaid,
        bezahlt_am: isPaid ? new Date().toISOString().split('T')[0] : null,
        zahlungsmethode: isPaid ? (zahlungsmethode || 'Überweisung') : null
      } : b
    ));

    try {
      // 3. Backend Update mit Zahlungsmethode
      console.log('🔍 [DataContext] Calling invoke update_booking_payment_command...');
      await invoke('update_booking_payment_command', {
        bookingId: id,
        bezahlt: isPaid,
        zahlungsmethode: isPaid ? zahlungsmethode : null
      });
      console.log('✅ [DataContext] Backend update successful');

      // 4. Trigger refresh-data event for Undo button (OHNE refreshBookings!)
      window.dispatchEvent(new CustomEvent('refresh-data'));
    } catch (error) {
      // 5. Rollback bei Fehler
      if (oldBooking) {
        setBookings(prev => prev.map(b =>
          b.id === id ? oldBooking : b
        ));
      }
      console.error('Fehler beim Aktualisieren des Zahlungsstatus:', error);
      throw error;
    }
  }, [bookings]);

  const value: DataContextType = {
    // Data
    rooms,
    guests,
    bookings,
    loading,

    // Refresh
    refreshRooms,
    refreshGuests,
    refreshBookings,
    refreshAll,

    // CRUD
    createRoom,
    updateRoom,
    deleteRoom,
    createGuest,
    updateGuest,
    deleteGuest,
    createBooking,
    updateBooking,
    deleteBooking,

    // Optimistic Updates
    updateBookingStatus,
    updateBookingPayment,
  };

  return <DataContext.Provider value={value}>{children}</DataContext.Provider>;
}

// Custom Hook
export function useData() {
  const context = useContext(DataContext);
  if (context === undefined) {
    throw new Error('useData must be used within a DataProvider');
  }
  return context;
}
