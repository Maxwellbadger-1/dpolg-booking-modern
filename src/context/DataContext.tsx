import { createContext, useContext, useState, useCallback, useEffect, ReactNode } from 'react';
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
import { syncBooking } from '../hooks/useBookingSync';

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

  const refreshPaymentRecipients = useCallback(async () => {
    try {
      const data = await invoke<PaymentRecipient[]>('get_payment_recipients');
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
      await Promise.all([refreshRooms(), refreshGuests(), refreshBookings(), refreshPaymentRecipients()]);

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
  }, [refreshRooms, refreshGuests, refreshBookings, refreshPaymentRecipients, hasLoadedOnce]);

  // Initial data load on mount
  useEffect(() => {
    refreshAll();
  }, [refreshAll]);

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

      const room = await invoke<Room>('create_room_command', {
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

      const room = await invoke<Room>('update_room_command', {
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

  // Payment Recipient CRUD Operations
  const createPaymentRecipient = useCallback(async (data: any): Promise<PaymentRecipient> => {
    try {
      // 1. Backend Create
      const recipient = await invoke<PaymentRecipient>('create_payment_recipient', data);

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
      await invoke('delete_payment_recipient', { id });

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
      // 1. Backend Create
      const booking = await invoke<Booking>('create_booking_command', data);
      console.log('✅ [DataContext] Buchung erstellt:', booking);
      console.log('📅 [DataContext] checkout_date:', booking.checkout_date);

      // 2. SOFORT zum State hinzufügen (Optimistic Update)
      setBookings(prev => [...prev, booking]);

      // 3. AUTO-SYNC zu Turso (neue Buchung)
      console.log('🔍 [DataContext] Prüfe Auto-Sync Bedingung:', {
        hasCheckoutDate: !!booking.checkout_date,
        checkoutDate: booking.checkout_date
      });

      // ✅ FIX (2025-10-21): Backend macht Auto-Sync automatisch in create_booking_command
      // Frontend-Sync wurde ENTFERNT um Race-Conditions zu vermeiden
      console.log('✅ [DataContext] Auto-Sync wird vom Backend durchgeführt');

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
    // FIX: Normalize camelCase to snake_case for TapeChart compatibility
    const normalizedData = { ...data };
    if (data.checkinDate) {
      normalizedData.checkin_date = data.checkinDate;
    }
    if (data.checkoutDate) {
      normalizedData.checkout_date = data.checkoutDate;
    }

    console.log('🔄 [Optimistic Update] Normalized data:', {
      hasCheckinDate: !!normalizedData.checkinDate,
      hasCheckin_date: !!normalizedData.checkin_date,
      hasCheckoutDate: !!normalizedData.checkoutDate,
      hasCheckout_date: !!normalizedData.checkout_date,
      checkin: normalizedData.checkin_date || normalizedData.checkinDate,
      checkout: normalizedData.checkout_date || normalizedData.checkoutDate
    });

    setBookings(prev => prev.map(b =>
      b.id === id ? { ...b, ...normalizedData } : b
    ));

    try {
      // 3. Backend Update
      const invokePayload = { id, ...sanitizedData };
      console.log('📤 [DataContext] Calling invoke with:');
      console.log('  payment_recipient_id:', invokePayload.payment_recipient_id);
      console.log('  Complete payload:', JSON.stringify(invokePayload, null, 2));

      const booking = await invoke<Booking>('update_booking_command', invokePayload);

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
        console.log('✅ [DataContext] UPDATE Bedingung erfüllt - starte Auto-Sync!');

        if (checkoutChanged) {
          console.log('🔄 [DataContext] Checkout-Datum geändert:', oldBooking.checkout_date, '→', newCheckout);
        }
        if (checkinChanged) {
          console.log('🔄 [DataContext] Checkin-Datum geändert:', oldBooking.checkin_date, '→', newCheckin);
          console.log('   → Sync checkout_date um Priorität zu aktualisieren');
        }

        // Loading Toast
        const syncToast = toast.loading('☁️ Synchronisiere Putzplan...', {
          style: {
            background: '#1e293b',
            color: '#fff',
            borderRadius: '0.75rem',
            padding: '1rem',
          }
        });

        // 🔄 SYNC zu Turso (Mobile App) - Verwende zentralen Sync Helper
        syncBooking({
          bookingId: id,
          checkinDate: newCheckin || '',
          checkoutDate: effectiveNewCheckout || newCheckout,
          oldCheckoutDate: checkoutChanged ? effectiveOldCheckout : undefined
        }).then((result) => {
          console.log('✅ [DataContext] Auto-Sync (UPDATE) erfolgreich:', result);
          toast.success('✅ Putzplan aktualisiert', { id: syncToast });
        }).catch((error) => {
          console.error('❌ [DataContext] Auto-Sync (UPDATE) Fehler:', error);
          toast.error('❌ Putzplan-Sync fehlgeschlagen', { id: syncToast });
        });
      } else {
        console.log('⚠️ [DataContext] Keine UPDATE Auto-Sync - Bedingung nicht erfüllt');
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

      // 4. 🔥 CASCADE DELETE zu Turso (falls Buchung gelöscht wurde)
      // Professionelle Lösung: Wenn Parent-Record gelöscht wird → automatisch alle Child-Records löschen
      // Booking (Parent) → cleaning_tasks (Child)
      if (deletedBooking) {
        console.log('🗑️ [DataContext] Buchung #' + deletedBooking.id + ' gelöscht - CASCADE DELETE zu Turso');

        // Loading Toast
        const syncToast = toast.loading('☁️ Lösche Putzplan-Aufgaben...', {
          style: {
            background: '#1e293b',
            color: '#fff',
            borderRadius: '0.75rem',
            padding: '1rem',
          }
        });

        // Fire-and-forget: CASCADE DELETE für alle Tasks dieser Booking
        invoke('delete_booking_tasks', { bookingId: deletedBooking.id })
          .then((result: string) => {
            console.log('✅ [DataContext] CASCADE DELETE erfolgreich:', result);
            toast.success('✅ Putzplan aktualisiert', { id: syncToast });
          })
          .catch((error: any) => {
            console.error('❌ [DataContext] CASCADE DELETE Fehler:', error);
            toast.error('❌ Putzplan-Sync fehlgeschlagen', { id: syncToast });
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
      await invoke('update_booking_status_command', { bookingId: id, newStatus: status });

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
      await invoke('update_booking_payment_command', {
        bookingId: id,
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
      console.log('🔄 [DataContext] reloadBooking:', id);
      const booking = await invoke<Booking>('get_booking_with_details_by_id_command', { id });
      console.log('✅ [DataContext] Booking reloaded with services:', booking.services);

      // State updaten mit vollständigen Daten (inkl. Services + Emojis)
      setBookings(prev => prev.map(b => b.id === id ? booking : b));
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

  const value: DataContextType = {
    // Data
    rooms,
    guests,
    bookings,
    paymentRecipients,
    loading,

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
