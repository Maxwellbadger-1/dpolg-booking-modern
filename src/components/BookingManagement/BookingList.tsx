import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Calendar, Search, Edit2, X, CheckCircle, Circle, Clock, Plus, Trash2, ArrowUpDown, ArrowUp, ArrowDown, Eye, Euro, AlertCircle, MoreVertical } from 'lucide-react';
import { format, isWithinInterval, parseISO } from 'date-fns';
import { de } from 'date-fns/locale';
import { useVirtualizer } from '@tanstack/react-virtual';
import BookingSidebar from './BookingSidebar';
import ErrorBoundary from '../ErrorBoundary';
import ConfirmDialog from '../ConfirmDialog';
import CancellationConfirmDialog from './CancellationConfirmDialog';
import StatusDropdown from './StatusDropdown';
import PaymentDropdown from './PaymentDropdown';
import InvoiceDropdown from './InvoiceDropdown';
import PortalDropdown from '../PortalDropdown';
import { useData } from '../../context/DataContext';
import { SELECT_STYLES, SELECT_BACKGROUND_STYLE } from '../../lib/selectStyles';

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  ort: string;
}

interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  dpolg_mitglied: boolean;
}

interface Booking {
  id: number;
  room_id: number;
  guest_id: number;
  reservierungsnummer: string;
  checkin_date: string;
  checkout_date: string;
  anzahl_gaeste: number;
  status: string;
  gesamtpreis: number;
  bemerkungen?: string;
  bezahlt: boolean;
  bezahlt_am?: string | null;
  zahlungsmethode?: string | null;
  mahnung_gesendet_am?: string | null;
  rechnung_versendet_am?: string | null;
  rechnung_versendet_an?: string | null;
  ist_stiftungsfall: boolean;
  room: Room;
  guest: Guest;
}

type SortField = 'reservierungsnummer' | 'guest' | 'room' | 'checkin' | 'checkout' | 'status' | 'price';
type SortDirection = 'asc' | 'desc' | null;

export default function BookingList() {
  const { bookings, rooms, loading: contextLoading, deleteBooking, refreshAll, updateBookingStatus, updateBookingPayment, markInvoiceSent } = useData();
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [roomFilter, setRoomFilter] = useState<string>('all');
  const [paymentFilter, setPaymentFilter] = useState<string>('all'); // NEW: Filter für Bezahlt/Offen
  const [stiftungsfallFilter, setStiftungsfallFilter] = useState<string>('all'); // NEW: Filter für Stiftungsfälle
  const [dateFrom, setDateFrom] = useState<string>('');
  const [dateTo, setDateTo] = useState<string>('');
  const [sortField, setSortField] = useState<SortField | null>(null);
  const [sortDirection, setSortDirection] = useState<SortDirection>(null);

  // Sidebar State
  const [showSidebar, setShowSidebar] = useState(false);
  const [sidebarBookingId, setSidebarBookingId] = useState<number | null>(null);
  const [sidebarMode, setSidebarMode] = useState<'view' | 'edit' | 'create'>('view');
  const [sidebarPrefillData, setSidebarPrefillData] = useState<{ roomId?: number; checkinDate?: string; checkoutDate?: string } | undefined>(undefined);

  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [bookingToDelete, setBookingToDelete] = useState<{ id: number; reservierungsnummer: string } | null>(null);
  const [showCancellationConfirm, setShowCancellationConfirm] = useState(false);
  const [bookingToCancel, setBookingToCancel] = useState<{ id: number; reservierungsnummer: string } | null>(null);
  const [sendCancellationEmail, setSendCancellationEmail] = useState(false);

  // Ref for virtualization scroll container
  const parentRef = useRef<HTMLDivElement>(null);

  // Data is loaded automatically via DataContext - no need for manual loading!

  const handleDeleteBooking = (id: number, reservierungsnummer: string) => {
    setBookingToDelete({ id, reservierungsnummer });
    setShowDeleteConfirm(true);
  };

  const confirmDelete = async (sendEmail: boolean) => {
    if (!bookingToDelete) return;

    try {
      // Löschen
      await deleteBooking(bookingToDelete.id);

      // Optional: Stornierungsbestätigung senden
      if (sendEmail) {
        try {
          await invoke('send_cancellation_email_command', { bookingId: bookingToDelete.id });
          console.log('Stornierungsbestätigung gesendet');
        } catch (emailError) {
          console.error('Fehler beim Senden der Stornierungsbestätigung:', emailError);
          // Nicht kritisch - Buchung ist bereits gelöscht
        }
      }

      await refreshAll(); // Auto-refresh via Context
      setShowDeleteConfirm(false);
      setBookingToDelete(null);
      setSendCancellationEmail(false);
    } catch (error) {
      console.error('Fehler beim Löschen:', error);
      alert('Fehler beim Löschen: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  const cancelDelete = () => {
    setShowDeleteConfirm(false);
    setBookingToDelete(null);
    setSendCancellationEmail(false);
  };

  const handleStatusChange = async (bookingId: number, newStatus: string) => {
    // Bei "storniert": Zeige Cancellation Dialog
    if (newStatus === 'storniert') {
      const booking = bookings.find(b => b.id === bookingId);
      if (booking) {
        setBookingToCancel({ id: booking.id, reservierungsnummer: booking.reservierungsnummer });
        setShowCancellationConfirm(true);
      }
      return;
    }

    // Andere Status: Direkt ändern mit Optimistic Update
    try {
      await updateBookingStatus(bookingId, newStatus);
    } catch (error) {
      console.error('Fehler beim Ändern des Status:', error);
      alert('Fehler beim Ändern des Status: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  const confirmCancellation = async (sendEmail: boolean) => {
    if (!bookingToCancel) return;

    try {
      // Status auf "storniert" setzen mit Optimistic Update
      await updateBookingStatus(bookingToCancel.id, 'storniert');

      // Optional: Stornierungsbestätigung senden
      if (sendEmail) {
        try {
          await invoke('send_cancellation_email_command', { bookingId: bookingToCancel.id });
          console.log('Stornierungsbestätigung gesendet');
        } catch (emailError) {
          console.error('Fehler beim Senden der Stornierungsbestätigung:', emailError);
        }
      }

      setShowCancellationConfirm(false);
      setBookingToCancel(null);
    } catch (error) {
      console.error('Fehler beim Stornieren:', error);
      alert('Fehler beim Stornieren: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  const cancelCancellation = () => {
    setShowCancellationConfirm(false);
    setBookingToCancel(null);
  };

  const handlePaymentChange = async (bookingId: number, isPaid: boolean, zahlungsmethode?: string, paymentDate?: string) => {
    try {
      await updateBookingPayment(bookingId, isPaid, zahlungsmethode, paymentDate);
    } catch (error) {
      console.error('Fehler beim Ändern des Bezahlt-Status:', error);
      alert('Fehler beim Ändern des Bezahlt-Status: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  const handleInvoiceStatusChange = async (bookingId: number, emailAddress: string) => {
    try {
      await markInvoiceSent(bookingId, emailAddress);
    } catch (error) {
      console.error('Fehler beim Markieren der Rechnung:', error);
      // Toast wird bereits in DataContext angezeigt
    }
  };

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      // Toggle direction: asc -> desc -> null -> asc
      if (sortDirection === 'asc') {
        setSortDirection('desc');
      } else if (sortDirection === 'desc') {
        setSortDirection(null);
        setSortField(null);
      }
    } else {
      setSortField(field);
      setSortDirection('asc');
    }
  };

  const getSortIcon = (field: SortField) => {
    if (sortField !== field) {
      return <ArrowUpDown className="w-3.5 h-3.5 text-slate-400" />;
    }
    if (sortDirection === 'asc') {
      return <ArrowUp className="w-3.5 h-3.5 text-blue-600" />;
    }
    return <ArrowDown className="w-3.5 h-3.5 text-blue-600" />;
  };

  const filteredAndSortedBookings = (() => {
    // Filter
    let filtered = bookings.filter(booking => {
      const matchesSearch =
        booking.reservierungsnummer.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (booking.guest ? `${booking.guest.vorname} ${booking.guest.nachname}`.toLowerCase().includes(searchQuery.toLowerCase()) : false) ||
        (booking.room ? booking.room.name.toLowerCase().includes(searchQuery.toLowerCase()) : false);

      const matchesStatus = statusFilter === 'all' || booking.status === statusFilter;
      const matchesRoom = roomFilter === 'all' || booking.room_id.toString() === roomFilter;
      const matchesPayment = paymentFilter === 'all' ||
        (paymentFilter === 'bezahlt' && booking.bezahlt) ||
        (paymentFilter === 'offen' && !booking.bezahlt);
      const matchesStiftungsfall = stiftungsfallFilter === 'all' ||
        (stiftungsfallFilter === 'stiftungsfall' && booking.ist_stiftungsfall) ||
        (stiftungsfallFilter === 'normal' && !booking.ist_stiftungsfall);

      // Date range filter
      let matchesDateRange = true;
      if (dateFrom || dateTo) {
        const checkinDate = parseISO(booking.checkin_date);
        const checkoutDate = parseISO(booking.checkout_date);

        if (dateFrom && dateTo) {
          const from = parseISO(dateFrom);
          const to = parseISO(dateTo);
          matchesDateRange = isWithinInterval(checkinDate, { start: from, end: to }) ||
                            isWithinInterval(checkoutDate, { start: from, end: to }) ||
                            (checkinDate <= from && checkoutDate >= to);
        } else if (dateFrom) {
          const from = parseISO(dateFrom);
          matchesDateRange = checkoutDate >= from;
        } else if (dateTo) {
          const to = parseISO(dateTo);
          matchesDateRange = checkinDate <= to;
        }
      }

      return matchesSearch && matchesStatus && matchesRoom && matchesPayment && matchesStiftungsfall && matchesDateRange;
    });

    // Sort
    if (sortField && sortDirection) {
      filtered = [...filtered].sort((a, b) => {
        let aValue: any;
        let bValue: any;

        switch (sortField) {
          case 'reservierungsnummer':
            aValue = a.reservierungsnummer;
            bValue = b.reservierungsnummer;
            break;
          case 'guest':
            aValue = `${a.guest.vorname} ${a.guest.nachname}`;
            bValue = `${b.guest.vorname} ${b.guest.nachname}`;
            break;
          case 'room':
            aValue = a.room.name;
            bValue = b.room.name;
            break;
          case 'checkin':
            aValue = new Date(a.checkin_date).getTime();
            bValue = new Date(b.checkin_date).getTime();
            break;
          case 'checkout':
            aValue = new Date(a.checkout_date).getTime();
            bValue = new Date(b.checkout_date).getTime();
            break;
          case 'status':
            aValue = a.status;
            bValue = b.status;
            break;
          case 'price':
            aValue = a.gesamtpreis;
            bValue = b.gesamtpreis;
            break;
          default:
            return 0;
        }

        if (aValue < bValue) return sortDirection === 'asc' ? -1 : 1;
        if (aValue > bValue) return sortDirection === 'asc' ? 1 : -1;
        return 0;
      });
    }

    return filtered;
  })();

  // TanStack Virtual - only render visible items
  const rowVirtualizer = useVirtualizer({
    count: filteredAndSortedBookings.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 120, // Estimated row height in pixels (taller due to complex content)
    overscan: 3, // Render 3 extra items above/below for smooth scrolling
  });

  if (contextLoading) {
    return (
      <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 p-6 flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
          <p className="text-slate-600">Lade Buchungen...</p>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 p-6 overflow-auto">
        <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-2xl font-bold text-slate-800">Buchungen</h2>
            <p className="text-sm text-slate-600 mt-1">
              {filteredAndSortedBookings.length} {filteredAndSortedBookings.length === 1 ? 'Buchung' : 'Buchungen'}
            </p>
          </div>
          <button
            onClick={() => {
              // Öffne Sidebar in Create Mode
              setSidebarBookingId(null);
              setSidebarMode('create');
              setSidebarPrefillData(undefined);
              setShowSidebar(true);
            }}
            className="flex items-center gap-2 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white px-4 py-2 rounded-lg font-semibold shadow-lg transition-all"
          >
            <Plus className="w-4 h-4" />
            Neue Buchung
          </button>
        </div>

        {/* Search & Filter */}
        <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-4 mb-6 space-y-4">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-400 w-5 h-5" />
              <input
                type="text"
                placeholder="Suche nach Reservierungsnummer, Gast oder Zimmer..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
              className={SELECT_STYLES}
              style={SELECT_BACKGROUND_STYLE}
            >
              <option value="all">Alle Status</option>
              <option value="reserviert">Reserviert</option>
              <option value="bestaetigt">Bestätigt</option>
              <option value="eingecheckt">Eingecheckt</option>
              <option value="ausgecheckt">Ausgecheckt</option>
              <option value="storniert">Storniert</option>
            </select>
            <select
              value={roomFilter}
              onChange={(e) => setRoomFilter(e.target.value)}
              className={SELECT_STYLES}
              style={SELECT_BACKGROUND_STYLE}
            >
              <option value="all">Alle Zimmer</option>
              {rooms.map(room => (
                <option key={room.id} value={room.id.toString()}>
                  {room.name} - {room.gebaeude_typ} • {room.capacity}P • {room.ort}
                </option>
              ))}
            </select>
            <select
              value={paymentFilter}
              onChange={(e) => setPaymentFilter(e.target.value)}
              className={SELECT_STYLES}
              style={SELECT_BACKGROUND_STYLE}
            >
              <option value="all">Alle Zahlungen</option>
              <option value="bezahlt">Bezahlt</option>
              <option value="offen">Offen</option>
            </select>
            <select
              value={stiftungsfallFilter}
              onChange={(e) => setStiftungsfallFilter(e.target.value)}
              className={SELECT_STYLES}
              style={SELECT_BACKGROUND_STYLE}
            >
              <option value="all">Alle Buchungen</option>
              <option value="stiftungsfall">Nur Stiftungsfälle</option>
              <option value="normal">Nur Normale</option>
            </select>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <label className="text-sm text-slate-600 font-medium whitespace-nowrap">Zeitraum:</label>
              <input
                type="date"
                value={dateFrom}
                onChange={(e) => setDateFrom(e.target.value)}
                className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Von"
              />
              <span className="text-slate-400">bis</span>
              <input
                type="date"
                value={dateTo}
                onChange={(e) => setDateTo(e.target.value)}
                className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Bis"
              />
            </div>
            {(dateFrom || dateTo) && (
              <button
                onClick={() => {
                  setDateFrom('');
                  setDateTo('');
                }}
                className="px-3 py-2 text-sm text-slate-600 hover:bg-slate-100 rounded-lg transition-colors"
              >
                Zurücksetzen
              </button>
            )}
          </div>
        </div>

        {/* Bookings Table */}
        {filteredAndSortedBookings.length === 0 && (
          <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-12">
            <div className="text-center">
              <Calendar className="w-16 h-16 text-slate-300 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-slate-800 mb-2">Keine Buchungen gefunden</h3>
              <p className="text-slate-600">
                {searchQuery || statusFilter !== 'all' || roomFilter !== 'all' || paymentFilter !== 'all' || stiftungsfallFilter !== 'all' || dateFrom || dateTo
                  ? 'Versuche andere Suchkriterien oder Filter.'
                  : 'Erstelle deine erste Buchung mit dem Button oben.'
                }
              </p>
            </div>
          </div>
        )}

        {filteredAndSortedBookings.length > 0 && (
          <div className="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
            {/* Scroll Container with Virtualization */}
            <div
              ref={parentRef}
              className="overflow-auto"
              style={{ height: 'calc(100vh - 420px)' }}
            >
              <div className="min-w-[1500px]">
                {/* Table Header - Sticky Top with border-bottom for separation */}
                <div className="bg-slate-50 border-b-2 border-slate-300 grid px-6 py-4 sticky top-0 z-30" style={{ gridTemplateColumns: '150px 200px 120px 110px 110px 140px 100px 120px 120px 120px 200px' }}>
                  {/* Sticky Left Column - Reservierung */}
                  <div
                    className="sticky left-0 z-20 bg-slate-50 text-xs font-semibold text-slate-700 uppercase tracking-wider cursor-pointer hover:bg-slate-200 transition-colors px-2 py-1 -ml-6 pl-6 pr-4 text-center"
                    onClick={() => handleSort('reservierungsnummer')}
                  >
                    <div className="flex items-center justify-center gap-2">
                      Reservierung
                      {getSortIcon('reservierungsnummer')}
                    </div>
                  </div>
                  <div
                    className="text-xs font-semibold text-slate-700 uppercase tracking-wider cursor-pointer hover:bg-slate-200 transition-colors px-2 py-1 text-center"
                    onClick={() => handleSort('guest')}
                  >
                    <div className="flex items-center justify-center gap-2">
                      Gast
                      {getSortIcon('guest')}
                    </div>
                  </div>
                  <div
                    className="text-xs font-semibold text-slate-700 uppercase tracking-wider cursor-pointer hover:bg-slate-200 transition-colors px-2 py-1 text-center"
                    onClick={() => handleSort('room')}
                  >
                    <div className="flex items-center justify-center gap-2">
                      Zimmer
                      {getSortIcon('room')}
                    </div>
                  </div>
                  <div
                    className="text-xs font-semibold text-slate-700 uppercase tracking-wider cursor-pointer hover:bg-slate-200 transition-colors px-2 py-1 text-center"
                    onClick={() => handleSort('checkin')}
                  >
                    <div className="flex items-center justify-center gap-2">
                      Check-in
                      {getSortIcon('checkin')}
                    </div>
                  </div>
                  <div
                    className="text-xs font-semibold text-slate-700 uppercase tracking-wider cursor-pointer hover:bg-slate-200 transition-colors px-2 py-1 text-center"
                    onClick={() => handleSort('checkout')}
                  >
                    <div className="flex items-center justify-center gap-2">
                      Check-out
                      {getSortIcon('checkout')}
                    </div>
                  </div>
                  <div
                    className="text-xs font-semibold text-slate-700 uppercase tracking-wider cursor-pointer hover:bg-slate-200 transition-colors px-2 py-1 text-center"
                    onClick={() => handleSort('status')}
                  >
                    <div className="flex items-center justify-center gap-2">
                      Status
                      {getSortIcon('status')}
                    </div>
                  </div>
                  <div
                    className="text-xs font-semibold text-slate-700 uppercase tracking-wider cursor-pointer hover:bg-slate-200 transition-colors px-2 py-1 text-center"
                    onClick={() => handleSort('price')}
                  >
                    <div className="flex items-center justify-center gap-2">
                      Preis
                      {getSortIcon('price')}
                    </div>
                  </div>
                  <div className="text-xs font-semibold text-slate-700 uppercase tracking-wider text-center px-2 py-1">
                    Bezahlt
                  </div>
                  <div className="text-xs font-semibold text-slate-700 uppercase tracking-wider text-center px-2 py-1">
                    Rechnung
                  </div>
                  <div className="text-xs font-semibold text-slate-700 uppercase tracking-wider text-center px-2 py-1">
                    Stiftung
                  </div>
                  <div className="text-xs font-semibold text-slate-700 uppercase tracking-wider text-center px-2 py-1">
                    Aktionen
                  </div>
                </div>

                {/* Virtualized Scrollable Booking List */}
                <div
                  style={{
                    height: `${rowVirtualizer.getTotalSize()}px`,
                    width: '100%',
                    position: 'relative',
                  }}
                >
                  {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                    const booking = filteredAndSortedBookings[virtualRow.index];
                    const isEven = virtualRow.index % 2 === 0;
                    return (
                      <div
                        key={booking.id}
                        style={{
                          position: 'absolute',
                          top: 0,
                          left: 0,
                          width: '100%',
                          height: `${virtualRow.size}px`,
                          transform: `translateY(${virtualRow.start}px)`,
                          gridTemplateColumns: '150px 200px 120px 110px 110px 140px 100px 120px 120px 120px 200px',
                        }}
                        className={`group grid px-6 py-4 border-b border-slate-200 transition-colors hover:bg-blue-50 ${
                          isEven ? 'bg-white' : 'bg-slate-50'
                        }`}
                      >
                        {/* Reservierung - Sticky Left */}
                        <div className="sticky left-0 z-10 bg-inherit -ml-6 pl-6 pr-4 text-center">
                          <div className="text-sm font-semibold text-slate-900">{booking.reservierungsnummer}</div>
                          <div className="text-xs text-slate-500">{booking.anzahl_gaeste} Gäste</div>
                        </div>

                        {/* Gast */}
                        <div className="px-2 text-center">
                          <div className="text-sm font-medium text-slate-900">
                            {booking.guest ? `${booking.guest.vorname} ${booking.guest.nachname}` : 'Unbekannt'}
                          </div>
                          <div className="text-xs text-slate-500 truncate">{booking.guest?.email || '-'}</div>
                        </div>

                        {/* Zimmer */}
                        <div className="px-2 text-center">
                          <div className="text-sm font-medium text-slate-900">{booking.room?.name || 'Unbekannt'}</div>
                          <div className="text-xs text-slate-500">{booking.room?.ort || '-'}</div>
                        </div>

                        {/* Check-in */}
                        <div className="px-2 text-center">
                          <div className="text-sm text-slate-900">
                            {format(new Date(booking.checkin_date), 'dd.MM.yyyy', { locale: de })}
                          </div>
                        </div>

                        {/* Check-out */}
                        <div className="px-2 text-center">
                          <div className="text-sm text-slate-900">
                            {format(new Date(booking.checkout_date), 'dd.MM.yyyy', { locale: de })}
                          </div>
                        </div>

                        {/* Status */}
                        <div className="px-2 text-center">
                          <StatusDropdown
                            currentStatus={booking.status}
                            bookingId={booking.id}
                            onStatusChange={(newStatus) => handleStatusChange(booking.id, newStatus)}
                          />
                        </div>

                        {/* Preis */}
                        <div className="px-2 text-center">
                          <div className="text-sm font-semibold text-slate-900">
                            {booking.gesamtpreis.toFixed(2)} €
                          </div>
                        </div>

                        {/* Bezahlt */}
                        <div className="px-2 text-center">
                          <div className="inline-flex flex-col items-center">
                            <PaymentDropdown
                              isPaid={booking.bezahlt}
                              bookingId={booking.id}
                              onPaymentChange={(isPaid, zahlungsmethode, paymentDate) => handlePaymentChange(booking.id, isPaid, zahlungsmethode, paymentDate)}
                            />
                            {booking.bezahlt && booking.bezahlt_am && (
                              <span className="text-xs text-slate-500 mt-1">
                                {format(new Date(booking.bezahlt_am), 'dd.MM.yyyy', { locale: de })}
                              </span>
                            )}
                          </div>
                        </div>

                        {/* Rechnung */}
                        <div className="px-2 text-center">
                          <InvoiceDropdown
                            invoiceSentAt={booking.rechnung_versendet_am}
                            invoiceSentTo={booking.rechnung_versendet_an}
                            bookingId={booking.id}
                            guestEmail={booking.guest?.email || ''}
                            onInvoiceStatusChange={handleInvoiceStatusChange}
                          />
                        </div>

                        {/* Stiftungsfall */}
                        <div className="px-2 text-center">
                          {booking.ist_stiftungsfall && (
                            <span className="inline-flex items-center gap-1.5 px-2 py-1 bg-gradient-to-r from-amber-100 to-orange-100 text-amber-800 text-xs font-bold rounded-full border-2 border-amber-300 shadow-sm">
                              <AlertCircle className="w-3 h-3" />
                            </span>
                          )}
                        </div>

                        {/* Aktionen */}
                        <div className="px-2 text-center">
                          <div className="inline-flex items-center gap-2">
                            {/* Details anzeigen */}
                            <button
                              onClick={() => {
                                setSidebarBookingId(booking.id);
                                setSidebarMode('view');
                                setSidebarPrefillData(undefined);
                                setShowSidebar(true);
                              }}
                              title="Details anzeigen"
                              className="inline-flex items-center justify-center w-8 h-8 text-blue-600 hover:bg-blue-100 rounded-lg transition-colors"
                            >
                              <Eye className="w-4 h-4" />
                            </button>

                            {/* Bearbeiten */}
                            <button
                              onClick={() => {
                                setSidebarBookingId(booking.id);
                                setSidebarMode('edit');
                                setSidebarPrefillData(undefined);
                                setShowSidebar(true);
                              }}
                              title="Bearbeiten"
                              className="inline-flex items-center justify-center w-8 h-8 text-slate-600 hover:bg-slate-200 rounded-lg transition-colors"
                            >
                              <Edit2 className="w-4 h-4" />
                            </button>

                            {/* Löschen */}
                            <button
                              onClick={() => handleDeleteBooking(booking.id, booking.reservierungsnummer)}
                              title="Löschen"
                              className="inline-flex items-center justify-center w-8 h-8 text-red-600 hover:bg-red-100 rounded-lg transition-colors"
                            >
                              <Trash2 className="w-4 h-4" />
                            </button>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>
        )}
        </div>
      </div>

      <BookingSidebar
        bookingId={sidebarBookingId}
        isOpen={showSidebar}
        onClose={() => {
          setShowSidebar(false);
          setSidebarBookingId(null);
          setSidebarMode('view');
          setSidebarPrefillData(undefined);
        }}
        mode={sidebarMode}
        prefillData={sidebarPrefillData}
      />

      {/* Delete Confirmation Dialog with Email Option */}
      {showDeleteConfirm && bookingToDelete && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 max-w-md w-full mx-4">
            <div className="flex items-start justify-between mb-6">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-red-500/20 rounded-lg">
                  <Trash2 className="w-6 h-6 text-red-500" />
                </div>
                <div>
                  <h2 className="text-xl font-bold text-white">Buchung löschen</h2>
                  <p className="text-sm text-slate-400 mt-1">Reservierung {bookingToDelete.reservierungsnummer}</p>
                </div>
              </div>
              <button
                onClick={cancelDelete}
                className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
                aria-label="Schließen"
              >
                <X className="w-5 h-5 text-slate-300" />
              </button>
            </div>

            <div className="mb-6">
              <p className="text-slate-300 mb-4">
                Möchten Sie diese Buchung wirklich löschen? Diese Aktion kann nicht rückgängig gemacht werden.
              </p>

              <label className="flex items-start gap-3 p-4 bg-slate-700/50 rounded-lg cursor-pointer hover:bg-slate-700 transition-colors">
                <input
                  type="checkbox"
                  checked={sendCancellationEmail}
                  onChange={(e) => setSendCancellationEmail(e.target.checked)}
                  className="mt-0.5 w-5 h-5 text-blue-600 bg-slate-600 border-slate-500 rounded focus:ring-2 focus:ring-blue-500 focus:ring-offset-0"
                />
                <div className="flex-1">
                  <div className="flex items-center gap-2 text-white font-medium mb-1">
                    <CheckCircle className="w-4 h-4" />
                    Stornierungsbestätigung per E-Mail senden
                  </div>
                  <p className="text-sm text-slate-400">
                    Der Gast erhält automatisch eine E-Mail mit der Stornierungsbestätigung.
                  </p>
                </div>
              </label>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => confirmDelete(sendCancellationEmail)}
                className="flex-1 px-4 py-3 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors"
              >
                Ja, löschen
              </button>
              <button
                onClick={cancelDelete}
                className="px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
              >
                Abbrechen
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Cancellation Confirmation Dialog */}
      <CancellationConfirmDialog
        isOpen={showCancellationConfirm}
        bookingNumber={bookingToCancel?.reservierungsnummer || ''}
        onConfirm={confirmCancellation}
        onCancel={cancelCancellation}
      />
    </>
  );
}
