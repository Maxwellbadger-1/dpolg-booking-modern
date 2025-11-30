import { createContext, useContext, useState, useCallback, useEffect, useMemo, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import toast from 'react-hot-toast';
import {
  commandManager,
  UpdateBookingStatusCommand,
  UpdateBookingPaymentCommand,
  CreateBookingCommand,
  UpdateBookingCommand,
  DeleteBookingCommand,
  CreateGuestCommand,
  UpdateGuestCommand,
  DeleteGuestCommand,
  CreateRoomCommand,
  UpdateRoomCommand,
  DeleteRoomCommand
} from '../lib/commandManager';

// Import Types from centralized location
import type { Room, Guest, BookingWithDetails as Booking, PaymentRecipient } from '../types/booking';

// Context Type
interface DataContextType {
  // Data
  rooms: Room[];
  guests: Guest[];
  bookings: Booking[];
  paymentRecipients: PaymentRecipient[];
  loading: boolean;

  // Normalized State Lookups (2025 Best Practice - O(1) Performance)
  guestMap: Map<number, Guest>;
  roomMap: Map<number, Room>;

  // Refresh Functions
  refreshRooms: () => Promise<void>;
  refreshGuests: () => Promise<void>;
  refreshBookings: () => Promise<void>;
  refreshPaymentRecipients: () => Promise<void>;
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

  createPaymentRecipient: (data: any) => Promise<PaymentRecipient>;
  updatePaymentRecipient: (id: number, data: any) => Promise<PaymentRecipient>;
  deletePaymentRecipient: (id: number) => Promise<void>;

  // Optimistic Updates
  updateBookingStatus: (id: number, status: string) => Promise<void>;
  updateBookingPayment: (id: number, isPaid: boolean, zahlungsmethode?: string, paymentDate?: string) => Promise<void>;
  markInvoiceSent: (id: number, emailAddress: string) => Promise<void>;
  reloadBooking: (id: number) => Promise<void>;
}

// Context
const DataContext = createContext<DataContextType | undefined>(undefined);

// Provider Component
export function DataProvider({ children }: { children: ReactNode }) {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [guests, setGuests] = useState<Guest[]>([]);
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [paymentRecipients, setPaymentRecipients] = useState<PaymentRecipient[]>([]);
  const [loading, setLoading] = useState(true); // Initial Load = true (für Splash Screen)
  const [hasLoadedOnce, setHasLoadedOnce] = useState(false); // Track ob Initial Load fertig

  // Refresh Functions
  const refreshRooms = useCallback(async () => {
    try {
      const data = await invoke<Room[]>('get_all_rooms_pg');
      setRooms(data);
    } catch (error) {
      console.error('Fehler beim Laden der Zimmer:', error);
      throw error;
    }
  }, []);

  const refreshGuests = useCallback(async () => {
    try {
      const data = await invoke<Guest[]>('get_all_guests_pg');
      setGuests(data);
    } catch (error) {
      console.error('Fehler beim Laden der Gäste:', error);
      throw error;
    }
  }, []);

  const refreshBookings = useCallback(async () => {
    try {
      const bookingsData = await invoke<Booking[]>('get_all_bookings_pg');

      // NORMALIZED STATE: No enrichment! Bookings only contain IDs
      // Components use guestMap/roomMap for O(1) lookups
      setBookings(bookingsData);
    } catch (error) {
      console.error('Fehler beim Laden der Buchungen:', error);
      throw error;
    }
  }, []);

  const refreshPaymentRecipients = useCallback(async () => {
    try {
      const data = await invoke<PaymentRecipient[]>('get_all_payment_recipients_pg');
      setPaymentRecipients(data);
    } catch (error) {
      console.error('Fehler beim Laden der Rechnungsempfänger:', error);
      throw error;
    }
  }, []);

  const refreshAll = useCallback(async () => {
    // NUR beim Initial Load (erster Aufruf) loading = true setzen
    if (!hasLoadedOnce) {
      setLoading(true);
    }

    try {
      // Load all data in parallel
      const [roomsData, guestsData, bookingsData] = await Promise.all([
        invoke<Room[]>('get_all_rooms_pg'),
        invoke<Guest[]>('get_all_guests_pg'),
        invoke<Booking[]>('get_all_bookings_pg'),
      ]);

      // Also load payment recipients
      await refreshPaymentRecipients();

      setRooms(roomsData);
      setGuests(guestsData);

      // NORMALIZED STATE: No enrichment! Bookings only contain IDs
      // Components use guestMap/roomMap for O(1) lookups
      setBookings(bookingsData);

      // Nach Initial Load: Warte 3 Sekunden für Splash Screen, dann deaktivieren
      if (!hasLoadedOnce) {
        setTimeout(() => {
          setLoading(false);
          setHasLoadedOnce(true);
        }, 3000); // 3 Sekunden Splash Screen
      }
    } catch (error) {
      // Bei Fehler: Sofort deaktivieren
      setLoading(false);
      if (!hasLoadedOnce) {
        setHasLoadedOnce(true);
      }
      throw error;
    }
  }, [refreshPaymentRecipients, hasLoadedOnce]);

  // Initial data load on mount
  useEffect(() => {
    refreshAll();
  }, [refreshAll]);

  // Real-Time Polling - Check for updates every 3 seconds
  useEffect(() => {
    let lastTimestamp = new Date().toISOString();
    let pollInterval: NodeJS.Timeout;

    const pollForUpdates = async () => {
      try {
        const response = await invoke<{
          bookings: Booking[];
          guests: Guest[];
          rooms: Room[];
          timestamp: string;
        }>('get_updates_since', { sinceTimestamp: lastTimestamp });

        // Update timestamp for next poll
        lastTimestamp = response.timestamp;

        // Merge updates into existing state (if any updates exist)
        if (response.bookings.length > 0) {
          console.log(`📊 Real-Time: ${response.bookings.length} booking(s) updated`);
          setBookings(prev => {
            const updatedIds = new Set(response.bookings.map(b => b.id));
            // Remove old versions and add updated ones
            const filtered = prev.filter(b => !updatedIds.has(b.id));
            return [...filtered, ...response.bookings];
          });
        }

        if (response.guests.length > 0) {
          console.log(`📊 Real-Time: ${response.guests.length} guest(s) updated`);
          setGuests(prev => {
            const updatedIds = new Set(response.guests.map(g => g.id));
            const filtered = prev.filter(g => !updatedIds.has(g.id));
            return [...filtered, ...response.guests];
          });
        }

        if (response.rooms.length > 0) {
          console.log(`📊 Real-Time: ${response.rooms.length} room(s) updated`);
          setRooms(prev => {
            const updatedIds = new Set(response.rooms.map(r => r.id));
            const filtered = prev.filter(r => !updatedIds.has(r.id));
            return [...filtered, ...response.rooms];
          });
        }
      } catch (error) {
        console.error('Real-Time polling error:', error);
        // Continue polling even on errors
      }
    };

    // Wait for initial load to complete before starting polling
    if (hasLoadedOnce) {
      pollInterval = setInterval(pollForUpdates, 3000); // Poll every 3 seconds
    }

    return () => {
      if (pollInterval) {
        clearInterval(pollInterval);
      }
    };
  }, [hasLoadedOnce]);

  // Room CRUD Operations
  const createRoom = useCallback(async (data: any): Promise<Room> => {
    console.log('🔍 [DataContext.createRoom] START');
    console.log('  📥 Incoming data:', JSON.stringify(data, null, 2));

    try {
      // 1. Backend Create - EINZELNE PARAMETER übergeben (nicht als Object!)
      console.log('  📤 Calling create_room_command with individual parameters:');
      console.log('    - name:', data.name);
      console.log('    - gebaeudeTyp:', data.gebaeudeTyp);
      console.log('    - capacity:', data.capacity);
      console.log('    - priceMember:', data.priceMember);
      console.log('    - priceNonMember:', data.priceNonMember);
      console.log('    - nebensaisonPreis:', data.nebensaisonPreis);
      console.log('    - hauptsaisonPreis:', data.hauptsaisonPreis);
      console.log('    - endreinigung:', data.endreinigung);
      console.log('    - ort:', data.ort);
      console.log('    - schluesselcode:', data.schluesselcode);
      console.log('    - streetAddress:', data.streetAddress);
      console.log('    - postalCode:', data.postalCode);
      console.log('    - city:', data.city);

      const room = await invoke<Room>('create_room_pg', {
        name: data.name,
        gebaeudeTyp: data.gebaeudeTyp,
        capacity: data.capacity,
        priceMember: data.priceMember,
        priceNonMember: data.priceNonMember,
        nebensaisonPreis: data.nebensaisonPreis,
        hauptsaisonPreis: data.hauptsaisonPreis,
        endreinigung: data.endreinigung,
        ort: data.ort,
        schluesselcode: data.schluesselcode || null,
        streetAddress: data.streetAddress || null,
        postalCode: data.postalCode || null,
        city: data.city || null,
        notizen: data.notizen || null,
      });

      console.log('  ✅ Backend returned:', JSON.stringify(room, null, 2));

      // 2. SOFORT zum State hinzufügen (Optimistic Update)
      setRooms(prev => [...prev, room]);

      // 3. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return room;
    } catch (error) {
      console.error('  ❌ [DataContext.createRoom] Error:', error);
      // Kein Rollback nötig - Room wurde noch nicht hinzugefügt
      throw error;
    }
  }, []);

  const updateRoom = useCallback(async (id: number, data: any): Promise<Room> => {
    console.log('🔍 [DataContext.updateRoom] START');
    console.log('  📥 Incoming data:', JSON.stringify(data, null, 2));

    // 1. Backup für Rollback
    const oldRoom = rooms.find(r => r.id === id);

    // 2. SOFORT im UI ändern (Optimistic Update)
    setRooms(prev => prev.map(r =>
      r.id === id ? { ...r, ...data } : r
    ));

    try {
      // 3. Backend Update - EINZELNE PARAMETER übergeben (nicht als Object!)
      // Tauri braucht JEDEN Parameter separat, sonst werden sie nicht richtig gemappt!
      console.log('  📤 Calling update_room_command with individual parameters:');
      console.log('    - id:', id);
      console.log('    - name:', data.name);
      console.log('    - gebaeudeTyp:', data.gebaeudeTyp);
      console.log('    - capacity:', data.capacity);
      console.log('    - priceMember:', data.priceMember);
      console.log('    - priceNonMember:', data.priceNonMember);
      console.log('    - nebensaisonPreis:', data.nebensaisonPreis);
      console.log('    - hauptsaisonPreis:', data.hauptsaisonPreis);
      console.log('    - endreinigung:', data.endreinigung);
      console.log('    - ort:', data.ort);
      console.log('    - schluesselcode:', data.schluesselcode);
      console.log('    - streetAddress:', data.streetAddress);
      console.log('    - postalCode:', data.postalCode);
      console.log('    - city:', data.city);

      const room = await invoke<Room>('update_room_pg', {
        id,
        name: data.name,
        gebaeudeTyp: data.gebaeudeTyp,
        capacity: data.capacity,
        priceMember: data.priceMember,
        priceNonMember: data.priceNonMember,
        nebensaisonPreis: data.nebensaisonPreis,
        hauptsaisonPreis: data.hauptsaisonPreis,
        endreinigung: data.endreinigung,
        ort: data.ort,
        schluesselcode: data.schluesselcode || null,
        streetAddress: data.streetAddress || null,
        postalCode: data.postalCode || null,
        city: data.city || null,
        notizen: data.notizen || null,
      });

      console.log('  ✅ Backend returned:', JSON.stringify(room, null, 2));

      // 4. State mit Backend-Response aktualisieren (korrekte snake_case Keys!)
      setRooms(prev => prev.map(r =>
        r.id === id ? room : r
      ));

      // 5. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return room;
    } catch (error) {
      console.error('  ❌ [DataContext.updateRoom] Error:', error);
      // 6. Rollback bei Fehler
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
      await invoke('delete_room_pg', { id });

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
      const guest = await invoke<Guest>('create_guest_pg', data);

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
      const guest = await invoke<Guest>('update_guest_pg', { id, ...data });

      // 4. State mit Backend-Response aktualisieren (korrekte snake_case Keys!)
      setGuests(prev => prev.map(g =>
        g.id === id ? guest : g
      ));

      // 5. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return guest;
    } catch (error) {
      // 6. Rollback bei Fehler
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
      await invoke('delete_guest_pg', { id });

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

  // Payment Recipient CRUD Operations
  const createPaymentRecipient = useCallback(async (data: any): Promise<PaymentRecipient> => {
    try {
      // 1. Backend Create
      const recipient = await invoke<PaymentRecipient>('create_payment_recipient_pg', data);

      // 2. SOFORT zum State hinzufügen (Optimistic Update)
      setPaymentRecipients(prev => [...prev, recipient]);

      // 3. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return recipient;
    } catch (error) {
      // Kein Rollback nötig - Recipient wurde noch nicht hinzugefügt
      throw error;
    }
  }, []);

  const updatePaymentRecipient = useCallback(async (id: number, data: any): Promise<PaymentRecipient> => {
    // 1. Backup für Rollback
    const oldRecipient = paymentRecipients.find(r => r.id === id);

    // 2. SOFORT im UI ändern (Optimistic Update)
    setPaymentRecipients(prev => prev.map(r =>
      r.id === id ? { ...r, ...data } : r
    ));

    try {
      // 3. Backend Update
      const recipient = await invoke<PaymentRecipient>('update_payment_recipient', { id, ...data });

      // 4. State mit Backend-Response aktualisieren (korrekte snake_case Keys!)
      setPaymentRecipients(prev => prev.map(r =>
        r.id === id ? recipient : r
      ));

      // 5. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return recipient;
    } catch (error) {
      // 6. Rollback bei Fehler
      if (oldRecipient) {
        setPaymentRecipients(prev => prev.map(r =>
          r.id === id ? oldRecipient : r
        ));
      }
      throw error;
    }
  }, [paymentRecipients]);

  const deletePaymentRecipient = useCallback(async (id: number): Promise<void> => {
    // 1. Backup für Rollback
    const deletedRecipient = paymentRecipients.find(r => r.id === id);

    // 2. SOFORT aus UI entfernen (Optimistic Update)
    setPaymentRecipients(prev => prev.filter(r => r.id !== id));

    try {
      // 3. Backend Delete
      await invoke('delete_payment_recipient_pg', { id });

      // 4. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));
    } catch (error) {
      // 5. Rollback - Recipient wiederherstellen
      if (deletedRecipient) {
        setPaymentRecipients(prev => [...prev, deletedRecipient]);
      }
      throw error;
    }
  }, [paymentRecipients]);

  // Booking CRUD Operations
  const createBooking = useCallback(async (data: any): Promise<Booking> => {
    console.log('🔍 [DataContext] createBooking aufgerufen mit:', data);
    try {
      // 1. Backend Create (Auto-Sync zu Turso erfolgt im Backend)
      const booking = await invoke<Booking>('create_booking_pg', data);
      console.log('✅ [DataContext] Buchung erstellt:', booking);
      console.log('📅 [DataContext] checkout_date:', booking.checkout_date);

      // 2. SOFORT zum State hinzufügen (Optimistic Update)
      setBookings(prev => [...prev, booking]);

      // 3. Toast-Benachrichtigung für Auto-Sync
      if (booking.checkout_date) {
        toast.success('✅ Buchung erstellt – Putzplan automatisch aktualisiert', {
          duration: 3000,
        });
      }

      // 4. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return booking;
    } catch (error) {
      console.error('❌ [DataContext] Fehler beim Erstellen der Buchung:', error);
      // Kein Rollback nötig - Booking wurde noch nicht hinzugefügt
      throw error;
    }
  }, []);

  const updateBooking = useCallback(async (id: number, data: any): Promise<Booking> => {
    console.log('🔍 [DataContext.updateBooking] START');

    // FIX: Support beide camelCase und snake_case (Backwards-Compatibility)
    // Neue Komponenten senden: paymentRecipientId (camelCase)
    // Alte Komponenten senden: payment_recipient_id (snake_case)
    const paymentRecipientIdValue = data.paymentRecipientId ?? data.payment_recipient_id;
    console.log('  📦 Incoming paymentRecipientId:', data.paymentRecipientId, 'type:', typeof data.paymentRecipientId);
    console.log('  📦 Incoming payment_recipient_id:', data.payment_recipient_id, 'type:', typeof data.payment_recipient_id);
    console.log('  ✅ Using value:', paymentRecipientIdValue, 'type:', typeof paymentRecipientIdValue);

    // FIX: Konvertiere undefined zu null (Serde/JSON.stringify Issue)
    // Problem: JSON.stringify überspringt undefined properties → Rust bekommt Feld nicht!
    // Lösung: Explizit undefined → null konvertieren
    const sanitizedData = { ...data };

    // WICHTIG: Tauri erwartet paymentRecipientId (camelCase), konvertiert automatisch zu payment_recipient_id (snake_case)
    sanitizedData.paymentRecipientId = paymentRecipientIdValue === undefined ? null : paymentRecipientIdValue;

    // Entferne den alten snake_case Key (falls vorhanden) um Konsistenz zu gewährleisten
    delete sanitizedData.payment_recipient_id;

    console.log('  ✅ [SANITIZED] paymentRecipientId:', sanitizedData.paymentRecipientId, 'type:', typeof sanitizedData.paymentRecipientId);

    // 1. Backup für Rollback
    const oldBooking = bookings.find(b => b.id === id);

    // 2. SOFORT im UI ändern (Optimistic Update)
    // Normalize ALL camelCase to snake_case for TapeChart compatibility
    const normalizedData = { ...data };
    if (data.checkinDate) {
      normalizedData.checkin_date = data.checkinDate;
    }
    if (data.checkoutDate) {
      normalizedData.checkout_date = data.checkoutDate;
    }
    if (data.roomId) {
      normalizedData.room_id = data.roomId;
    }
    if (data.guestId) {
      normalizedData.guest_id = data.guestId;
    }
    if (data.putzplanCheckoutDate) {
      normalizedData.putzplan_checkout_date = data.putzplanCheckoutDate;
    }

    setBookings(prev => prev.map(b => {
      if (b.id === id) {
        // WICHTIG: Services und Discounts beibehalten wenn nicht explizit im Update enthalten
        // Das verhindert, dass reloadBooking-Updates durch updateBooking überschrieben werden
        const updatedBooking = { ...b, ...normalizedData };
        if (!normalizedData.services && b.services) {
          updatedBooking.services = b.services;
        }
        if (!normalizedData.discounts && b.discounts) {
          updatedBooking.discounts = b.discounts;
        }
        return updatedBooking;
      }
      return b;
    }));

    try {
      // 3. Backend Update with Optimistic Locking (2025 Best Practice)
      // FIX: Wenn der Aufrufer ein aktuelles expectedUpdatedAt mitgibt, verwende das
      // statt des veralteten lokalen State (wichtig für BookingSidebar die frisch vom Server lädt)
      const expectedTimestamp = data.expectedUpdatedAt ?? oldBooking?.updated_at;

      const invokePayload = {
        id,
        ...sanitizedData,
        // WICHTIG: reservierungsnummer vom oldBooking holen (wird nicht im updatePayload gesendet)
        reservierungsnummer: oldBooking?.reservierungsnummer,
        expectedUpdatedAt: expectedTimestamp,  // ← OPTIMISTIC LOCKING: Version check
      };

      // Entferne expectedUpdatedAt aus sanitizedData um Konflikte zu vermeiden
      delete invokePayload.expectedUpdatedAt;
      invokePayload.expectedUpdatedAt = expectedTimestamp;
      console.log('📤 [DataContext] Calling invoke with:');
      console.log('  payment_recipient_id:', invokePayload.payment_recipient_id);
      console.log('  reservierungsnummer:', invokePayload.reservierungsnummer);
      console.log('  expectedUpdatedAt:', invokePayload.expectedUpdatedAt, '(Optimistic Locking)');
      console.log('  Complete payload:', JSON.stringify(invokePayload, null, 2));

      const booking = await invoke<Booking>('update_booking_pg', invokePayload);

      // WICHTIG: Server-Response (mit neuem updated_at) in State übernehmen
      // Das verhindert Optimistic Locking Konflikte bei nachfolgenden Updates
      // ABER: Backend-Response enthält keine Services/Discounts, daher beibehalten!
      setBookings(prev => prev.map(b => {
        if (b.id === id) {
          return {
            ...booking,
            services: b.services || booking.services,  // Behalte bestehende Services
            discounts: b.discounts || booking.discounts  // Behalte bestehende Discounts
          };
        }
        return b;
      }));

      // 4. AUTO-SYNC zu Turso (falls checkout_date ODER checkin_date geändert wurde)
      // FIX: Support both camelCase and snake_case (Backwards-Compatibility)
      const newCheckout = data.checkoutDate ?? data.checkout_date;
      const newCheckin = data.checkinDate ?? data.checkin_date;

      // FIX (2025-10-24): Berücksichtige alternatives Putzplan-Checkout-Datum
      // Das EFFEKTIVE Checkout-Datum ist: putzplan_checkout_date WENN gesetzt, SONST checkout_date
      const newPutzplanCheckout = data.putzplanCheckoutDate ?? data.putzplan_checkout_date;
      const effectiveNewCheckout = newPutzplanCheckout || newCheckout;

      const oldPutzplanCheckout = oldBooking?.putzplan_checkout_date;
      const effectiveOldCheckout = oldPutzplanCheckout || oldBooking?.checkout_date;

      const checkoutChanged = oldBooking && (effectiveNewCheckout || newCheckout) && effectiveOldCheckout !== effectiveNewCheckout;
      const checkinChanged = oldBooking && newCheckin && oldBooking.checkin_date !== newCheckin;

      console.log('🔍 [DataContext] Prüfe UPDATE Auto-Sync:', {
        hasOldBooking: !!oldBooking,
        checkoutChanged,
        checkinChanged,
        oldCheckout: oldBooking?.checkout_date,
        newCheckout,
        oldCheckin: oldBooking?.checkin_date,
        newCheckin,
        dataKeys: Object.keys(data)
      });

      if (checkoutChanged || checkinChanged) {
        console.log('✅ [DataContext] UPDATE Bedingung erfüllt - Backend macht Auto-Sync!');

        if (checkoutChanged) {
          console.log('🔄 [DataContext] Checkout-Datum geändert:', oldBooking.checkout_date, '→', newCheckout);
        }
        if (checkinChanged) {
          console.log('🔄 [DataContext] Checkin-Datum geändert:', oldBooking.checkin_date, '→', newCheckin);
        }

        // Backend (update_booking_pg) hat bereits automatisch synchronisiert
        toast.success('✅ Buchung aktualisiert – Putzplan automatisch synchronisiert', {
          duration: 3000,
        });
      } else {
        console.log('⚠️ [DataContext] Keine datum-relevanten Änderungen');
      }

      // 5. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));

      return booking;
    } catch (error) {
      // 6. Rollback bei Fehler
      if (oldBooking) {
        setBookings(prev => prev.map(b =>
          b.id === id ? oldBooking : b
        ));
      }

      // 7. Optimistic Locking Conflict Detection (2025 Best Practice)
      const errorMsg = String(error);
      if (errorMsg.includes('was modified by another user') || errorMsg.includes('Conflict:')) {
        console.error('⚠️ [DataContext] CONFLICT DETECTED:', errorMsg);
        toast.error('⚠️ Dieser Datensatz wurde von einem anderen Benutzer geändert. Bitte aktualisieren Sie die Seite und versuchen Sie es erneut.', {
          duration: 6000,  // Länger anzeigen bei Konflikt
          style: {
            background: '#dc2626',  // Rot für Konflikt
            color: '#fff',
            borderRadius: '0.75rem',
            padding: '1rem',
          }
        });
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
      await invoke('delete_booking_pg', { id });

      // 4. Toast-Benachrichtigung (Backend hat bereits automatisch Tasks gelöscht)
      if (deletedBooking) {
        console.log('🗑️ [DataContext] Buchung #' + deletedBooking.id + ' gelöscht - Backend macht CASCADE DELETE');
        toast.success('✅ Buchung gelöscht – Putzplan automatisch bereinigt', {
          duration: 3000,
        });
      }

      // 5. Event für Undo-Button
      window.dispatchEvent(new CustomEvent('refresh-data'));
    } catch (error) {
      // 6. Rollback - Booking wiederherstellen
      if (deletedBooking) {
        setBookings(prev => [...prev, deletedBooking]);
      }
      throw error;
    }
  }, [bookings]);

  // Command Pattern: Update Booking Status (INSTANT UNDO/REDO!)
  const updateBookingStatus = useCallback(async (id: number, status: string): Promise<void> => {
    const oldBooking = bookings.find(b => b.id === id);
    if (!oldBooking) return;

    // Create and execute command (INSTANT UI update!)
    const command = new UpdateBookingStatusCommand(
      id,
      oldBooking.status,
      status,
      setBookings
    );
    commandManager.executeCommand(command);

    try {
      // Backend sync (fire-and-forget, runs in background)
      await invoke('update_booking_status_pg', { id: id, status: status });

      // 🔥 2-SCHRITT SYNC zu Turso (Status-Änderung, z.B. Stornierung)
      // Bei Status-Änderung (reserviert → storniert oder umgekehrt) müssen Tasks neu berechnet werden
      // Schritt 1: DELETE alte Tasks (CASCADE)
      // Schritt 2: Full Sync erstellt neue Tasks basierend auf neuem Status
      if (oldBooking) {
        console.log('🔄 [DataContext] Status-Änderung: "' + oldBooking.status + '" → "' + status + '" - 2-Schritt Sync');

        // Loading Toast
        const syncToast = toast.loading('☁️ Synchronisiere Putzplan...', {
          style: {
            background: '#1e293b',
            color: '#fff',
            borderRadius: '0.75rem',
            padding: '1rem',
          }
        });

        // Schritt 1: DELETE alle Tasks dieser Booking
        invoke('delete_booking_tasks', { bookingId: id })
          .then(() => {
            console.log('✅ [DataContext] Booking Tasks gelöscht, starte Full Sync...');
            // Schritt 2: Full Sync (erstellt neue Tasks falls Buchung aktiv)
            return invoke('sync_week_ahead');
          })
          .then((result: string) => {
            console.log('✅ [DataContext] Vollständiger Sync erfolgreich:', result);
            toast.success('✅ Putzplan aktualisiert', { id: syncToast });
          })
          .catch((error: any) => {
            console.error('❌ [DataContext] Sync fehlgeschlagen:', error);
            toast.error('❌ Putzplan-Sync fehlgeschlagen', { id: syncToast });
          });
      }
    } catch (error) {
      // On error: Undo the command (instant rollback!)
      commandManager.undo();
      console.error('Fehler beim Aktualisieren des Buchungsstatus:', error);
      throw error;
    }
  }, [bookings]);

  // Command Pattern: Update Booking Payment (INSTANT UNDO/REDO!)
  const updateBookingPayment = useCallback(async (id: number, isPaid: boolean, zahlungsmethode?: string, paymentDate?: string): Promise<void> => {
    const oldBooking = bookings.find(b => b.id === id);
    if (!oldBooking) return;

    // Verwende das übergebene Datum, oder falls nicht vorhanden, heutiges Datum
    const newPaidAt = isPaid ? (paymentDate || new Date().toISOString().split('T')[0]) : null;
    const newMethod = isPaid ? (zahlungsmethode || 'Überweisung') : null;

    // Create and execute command (INSTANT UI update!)
    const command = new UpdateBookingPaymentCommand(
      id,
      oldBooking.bezahlt,
      isPaid,
      oldBooking.bezahlt_am || null,
      newPaidAt,
      oldBooking.zahlungsmethode || null,
      newMethod,
      setBookings
    );
    commandManager.executeCommand(command);

    try {
      // Backend sync (fire-and-forget, runs in background)
      await invoke('update_booking_payment_pg', {
        id: id,
        bezahlt: isPaid,
        zahlungsmethode: newMethod,
        bezahltAm: newPaidAt  // NEU: Datum an Backend senden
      });

      // ✅ REMINDER BADGE UPDATE: Trigger Bell-Icon Count Refresh (only when marking as paid)
      if (isPaid) {
        window.dispatchEvent(new CustomEvent('reminder-updated'));
      }
    } catch (error) {
      // On error: Undo the command (instant rollback!)
      commandManager.undo();
      console.error('Fehler beim Aktualisieren des Zahlungsstatus:', error);
      throw error;
    }
  }, [bookings]);

  // Mark Invoice as Sent (Optimistic Update)
  const markInvoiceSent = useCallback(async (id: number, emailAddress: string): Promise<void> => {
    const oldBooking = bookings.find(b => b.id === id);
    if (!oldBooking) return;

    const sentAt = new Date().toISOString();

    // SOFORT im UI ändern (Optimistic Update)
    setBookings(prev => prev.map(b =>
      b.id === id ? {
        ...b,
        rechnung_versendet_am: sentAt,
        rechnung_versendet_an: emailAddress
      } : b
    ));

    try {
      // Backend sync
      await invoke('mark_invoice_sent_command', {
        bookingId: id,
        emailAddress
      });

      toast.success('✅ Rechnung als versendet markiert');
    } catch (error) {
      // Rollback bei Fehler
      setBookings(prev => prev.map(b =>
        b.id === id ? oldBooking : b
      ));
      console.error('Fehler beim Markieren der Rechnung:', error);
      toast.error('❌ Fehler beim Markieren der Rechnung');
      throw error;
    }
  }, [bookings]);

  // Reload Single Booking (für Optimistic Updates nach Service/Discount-Linking)
  const reloadBooking = useCallback(async (id: number): Promise<void> => {
    try {
      console.log('🔄 [DataContext] reloadBooking START:', id);
      console.log('📊 [DataContext] Current bookings count:', bookings.length);

      const booking = await invoke<Booking>('get_booking_with_details_by_id_pg', { id });
      console.log('✅ [DataContext] Booking reloaded from backend:', {
        id: booking.id,
        servicesCount: booking.services?.length || 0,
        services: booking.services?.map(s => ({ name: s.name, emoji: s.emoji }))
      });

      // State updaten mit vollständigen Daten (inkl. Services + Emojis)
      setBookings(prev => {
        console.log('🔄 [DataContext] Updating bookings state...');
        const oldBooking = prev.find(b => b.id === id);
        console.log('📝 [DataContext] Old booking services:', oldBooking?.services?.map(s => ({ name: s.name, emoji: s.emoji })));
        console.log('📝 [DataContext] New booking services:', booking.services?.map(s => ({ name: s.name, emoji: s.emoji })));

        const newState = prev.map(b => b.id === id ? booking : b);
        console.log('✅ [DataContext] State updated, new bookings count:', newState.length);
        return newState;
      });

      console.log('✅ [DataContext] reloadBooking COMPLETE');
    } catch (error) {
      console.error('❌ [DataContext] Fehler beim Neuladen der Buchung:', error);
      throw error;
    }
  }, []);

  // Event Listener: Refresh invoice status after email send
  useEffect(() => {
    const handleInvoiceStatusRefresh = (event: CustomEvent) => {
      const { bookingId } = event.detail;
      console.log('📧 [DataContext] Invoice email sent, reloading booking:', bookingId);
      reloadBooking(bookingId).catch(error => {
        console.error('❌ [DataContext] Failed to reload booking after invoice send:', error);
      });
    };

    window.addEventListener('refresh-invoice-status', handleInvoiceStatusRefresh as EventListener);

    return () => {
      window.removeEventListener('refresh-invoice-status', handleInvoiceStatusRefresh as EventListener);
    };
  }, [reloadBooking]);

  // Event Listener: Sync Mobile App after Undo operation
  useEffect(() => {
    const handleUndoExecuted = (event: CustomEvent) => {
      const { commandType, description } = event.detail || {};
      console.log('🔄 [DataContext] Undo executed - syncing mobile app:', { commandType, description });

      // Loading Toast
      const syncToast = toast.loading('☁️ Synchronisiere Putzplan nach Undo...', {
        style: {
          background: '#1e293b',
          color: '#fff',
          borderRadius: '0.75rem',
          padding: '1rem',
        }
      });

      // Vollständiger Sync (nächste 7 Tage)
      invoke('sync_week_ahead')
        .then((result: string) => {
          console.log('✅ [DataContext] Mobile App Sync nach Undo erfolgreich:', result);
          toast.success('✅ Putzplan aktualisiert', { id: syncToast });
        })
        .catch((error: any) => {
          console.error('❌ [DataContext] Mobile App Sync nach Undo fehlgeschlagen:', error);
          toast.error('❌ Putzplan-Sync fehlgeschlagen', { id: syncToast });
        });
    };

    window.addEventListener('undo-executed', handleUndoExecuted as EventListener);

    return () => {
      window.removeEventListener('undo-executed', handleUndoExecuted as EventListener);
    };
  }, []);

  // Event Listener: Sync Mobile App after Redo operation
  useEffect(() => {
    const handleRedoExecuted = (event: CustomEvent) => {
      const { commandType, description } = event.detail || {};
      console.log('🔄 [DataContext] Redo executed - syncing mobile app:', { commandType, description });

      // Loading Toast
      const syncToast = toast.loading('☁️ Synchronisiere Putzplan nach Wiederherstellen...', {
        style: {
          background: '#1e293b',
          color: '#fff',
          borderRadius: '0.75rem',
          padding: '1rem',
        }
      });

      // Vollständiger Sync (nächste 7 Tage)
      invoke('sync_week_ahead')
        .then((result: string) => {
          console.log('✅ [DataContext] Mobile App Sync nach Redo erfolgreich:', result);
          toast.success('✅ Putzplan aktualisiert', { id: syncToast });
        })
        .catch((error: any) => {
          console.error('❌ [DataContext] Mobile App Sync nach Redo fehlgeschlagen:', error);
          toast.error('❌ Putzplan-Sync fehlgeschlagen', { id: syncToast });
        });
    };

    window.addEventListener('redo-executed', handleRedoExecuted as EventListener);

    return () => {
      window.removeEventListener('redo-executed', handleRedoExecuted as EventListener);
    };
  }, []);

  // NORMALIZED STATE MAPS (2025 Best Practice - O(1) Performance)
  // These maps are used by components to look up guests/rooms by ID
  // Instead of enriching bookings with nested objects, components use these maps
  const guestMap = useMemo(() => new Map(guests.map(g => [g.id, g])), [guests]);
  const roomMap = useMemo(() => new Map(rooms.map(r => [r.id, r])), [rooms]);

  const value: DataContextType = {
    // Data
    rooms,
    guests,
    bookings,
    paymentRecipients,
    loading,

    // Normalized State Lookups (O(1) Performance)
    guestMap,
    roomMap,

    // Refresh
    refreshRooms,
    refreshGuests,
    refreshBookings,
    refreshPaymentRecipients,
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
    createPaymentRecipient,
    updatePaymentRecipient,
    deletePaymentRecipient,

    // Optimistic Updates
    updateBookingStatus,
    updateBookingPayment,
    markInvoiceSent,
    reloadBooking,
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
