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
  }, [refreshAll]);

  // Room CRUD Operations
  const createRoom = useCallback(async (data: any): Promise<Room> => {
    const room = await invoke<Room>('create_room_command', data);
    await refreshRooms();
    return room;
  }, [refreshRooms]);

  const updateRoom = useCallback(async (id: number, data: any): Promise<Room> => {
    const room = await invoke<Room>('update_room_command', { id, ...data });
    await refreshRooms();
    return room;
  }, [refreshRooms]);

  const deleteRoom = useCallback(async (id: number): Promise<void> => {
    await invoke('delete_room_command', { id });
    await refreshRooms();
    await refreshBookings(); // Buchungen könnten betroffen sein
  }, [refreshRooms, refreshBookings]);

  // Guest CRUD Operations
  const createGuest = useCallback(async (data: any): Promise<Guest> => {
    const guest = await invoke<Guest>('create_guest_command', data);
    await refreshGuests();
    return guest;
  }, [refreshGuests]);

  const updateGuest = useCallback(async (id: number, data: any): Promise<Guest> => {
    const guest = await invoke<Guest>('update_guest_command', { id, ...data });
    await refreshGuests();
    return guest;
  }, [refreshGuests]);

  const deleteGuest = useCallback(async (id: number): Promise<void> => {
    await invoke('delete_guest_command', { id });
    await refreshGuests();
    await refreshBookings(); // Buchungen könnten betroffen sein
  }, [refreshGuests, refreshBookings]);

  // Booking CRUD Operations
  const createBooking = useCallback(async (data: any): Promise<Booking> => {
    const booking = await invoke<Booking>('create_booking_command', data);
    await refreshBookings();
    return booking;
  }, [refreshBookings]);

  const updateBooking = useCallback(async (id: number, data: any): Promise<Booking> => {
    const booking = await invoke<Booking>('update_booking_command', { id, ...data });
    await refreshBookings();
    return booking;
  }, [refreshBookings]);

  const deleteBooking = useCallback(async (id: number): Promise<void> => {
    await invoke('delete_booking_command', { id });
    await refreshBookings();
  }, [refreshBookings]);

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
