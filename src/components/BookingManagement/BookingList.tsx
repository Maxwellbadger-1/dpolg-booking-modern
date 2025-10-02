import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Calendar, Search, Edit2, X, CheckCircle, Circle, Clock, Plus, Trash2, ArrowUpDown, ArrowUp, ArrowDown, Eye, Euro, AlertCircle } from 'lucide-react';
import { format, isWithinInterval, parseISO } from 'date-fns';
import { de } from 'date-fns/locale';
import BookingDialog from './BookingDialog';
import BookingDetails from './BookingDetails';
import ErrorBoundary from '../ErrorBoundary';

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
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
  room: Room;
  guest: Guest;
}

type SortField = 'reservierungsnummer' | 'guest' | 'room' | 'checkin' | 'checkout' | 'status' | 'price';
type SortDirection = 'asc' | 'desc' | null;

export default function BookingList() {
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [roomFilter, setRoomFilter] = useState<string>('all');
  const [dateFrom, setDateFrom] = useState<string>('');
  const [dateTo, setDateTo] = useState<string>('');
  const [sortField, setSortField] = useState<SortField | null>(null);
  const [sortDirection, setSortDirection] = useState<SortDirection>(null);
  const [showDialog, setShowDialog] = useState(false);
  const [selectedBooking, setSelectedBooking] = useState<Booking | undefined>(undefined);
  const [showDetails, setShowDetails] = useState(false);
  const [detailsBookingId, setDetailsBookingId] = useState<number | null>(null);

  useEffect(() => {
    loadBookings();
    loadRooms();
  }, []);

  const loadBookings = async () => {
    try {
      setLoading(true);
      const data = await invoke<Booking[]>('get_all_bookings');
      setBookings(data);
    } catch (error) {
      console.error('Fehler beim Laden der Buchungen:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadRooms = async () => {
    try {
      const data = await invoke<Room[]>('get_all_rooms');
      setRooms(data);
    } catch (error) {
      console.error('Fehler beim Laden der Zimmer:', error);
    }
  };

  const handleDeleteBooking = async (id: number, reservierungsnummer: string) => {
    if (!confirm(`Buchung ${reservierungsnummer} wirklich löschen?`)) {
      return;
    }

    try {
      await invoke('delete_booking_command', { id });
      loadBookings();
    } catch (error) {
      console.error('Fehler beim Löschen der Buchung:', error);
      alert('Fehler beim Löschen der Buchung');
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

  const getStatusBadge = (status: string) => {
    const styles = {
      reserviert: 'bg-blue-100 text-blue-700 border-blue-200',
      bestaetigt: 'bg-emerald-100 text-emerald-700 border-emerald-200',
      eingecheckt: 'bg-purple-100 text-purple-700 border-purple-200',
      ausgecheckt: 'bg-slate-100 text-slate-700 border-slate-200',
      storniert: 'bg-red-100 text-red-700 border-red-200',
    };

    const icons = {
      reserviert: <Circle className="w-3 h-3" />,
      bestaetigt: <CheckCircle className="w-3 h-3" />,
      eingecheckt: <Clock className="w-3 h-3" />,
      ausgecheckt: <CheckCircle className="w-3 h-3" />,
      storniert: <X className="w-3 h-3" />,
    };

    return (
      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold border ${styles[status as keyof typeof styles] || styles.reserviert}`}>
        {icons[status as keyof typeof icons]}
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </span>
    );
  };

  const filteredAndSortedBookings = (() => {
    // Filter
    let filtered = bookings.filter(booking => {
      const matchesSearch =
        booking.reservierungsnummer.toLowerCase().includes(searchQuery.toLowerCase()) ||
        `${booking.guest.vorname} ${booking.guest.nachname}`.toLowerCase().includes(searchQuery.toLowerCase()) ||
        booking.room.name.toLowerCase().includes(searchQuery.toLowerCase());

      const matchesStatus = statusFilter === 'all' || booking.status === statusFilter;
      const matchesRoom = roomFilter === 'all' || booking.room_id.toString() === roomFilter;

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

      return matchesSearch && matchesStatus && matchesRoom && matchesDateRange;
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

  if (loading) {
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
              setSelectedBooking(undefined);
              setShowDialog(true);
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
              className="px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
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
              className="px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
            >
              <option value="all">Alle Zimmer</option>
              {rooms.map(room => (
                <option key={room.id} value={room.id.toString()}>{room.name}</option>
              ))}
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
        {filteredAndSortedBookings.length === 0 ? (
          <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-12">
            <div className="text-center">
              <Calendar className="w-16 h-16 text-slate-300 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-slate-800 mb-2">Keine Buchungen gefunden</h3>
              <p className="text-slate-600">
                {searchQuery || statusFilter !== 'all' || roomFilter !== 'all' || dateFrom || dateTo
                  ? 'Versuche andere Suchkriterien oder Filter.'
                  : 'Erstelle deine erste Buchung mit dem Button oben.'
                }
              </p>
            </div>
          </div>
        ) : (
          <div className="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-slate-50 border-b border-slate-200">
                  <tr>
                    <th
                      className="px-6 py-3 text-left text-xs font-semibold text-slate-600 uppercase tracking-wider cursor-pointer hover:bg-slate-100 transition-colors"
                      onClick={() => handleSort('reservierungsnummer')}
                    >
                      <div className="flex items-center gap-2">
                        Reservierung
                        {getSortIcon('reservierungsnummer')}
                      </div>
                    </th>
                    <th
                      className="px-6 py-3 text-left text-xs font-semibold text-slate-600 uppercase tracking-wider cursor-pointer hover:bg-slate-100 transition-colors"
                      onClick={() => handleSort('guest')}
                    >
                      <div className="flex items-center gap-2">
                        Gast
                        {getSortIcon('guest')}
                      </div>
                    </th>
                    <th
                      className="px-6 py-3 text-left text-xs font-semibold text-slate-600 uppercase tracking-wider cursor-pointer hover:bg-slate-100 transition-colors"
                      onClick={() => handleSort('room')}
                    >
                      <div className="flex items-center gap-2">
                        Zimmer
                        {getSortIcon('room')}
                      </div>
                    </th>
                    <th
                      className="px-6 py-3 text-left text-xs font-semibold text-slate-600 uppercase tracking-wider cursor-pointer hover:bg-slate-100 transition-colors"
                      onClick={() => handleSort('checkin')}
                    >
                      <div className="flex items-center gap-2">
                        Check-in
                        {getSortIcon('checkin')}
                      </div>
                    </th>
                    <th
                      className="px-6 py-3 text-left text-xs font-semibold text-slate-600 uppercase tracking-wider cursor-pointer hover:bg-slate-100 transition-colors"
                      onClick={() => handleSort('checkout')}
                    >
                      <div className="flex items-center gap-2">
                        Check-out
                        {getSortIcon('checkout')}
                      </div>
                    </th>
                    <th
                      className="px-6 py-3 text-left text-xs font-semibold text-slate-600 uppercase tracking-wider cursor-pointer hover:bg-slate-100 transition-colors"
                      onClick={() => handleSort('status')}
                    >
                      <div className="flex items-center gap-2">
                        Status
                        {getSortIcon('status')}
                      </div>
                    </th>
                    <th
                      className="px-6 py-3 text-right text-xs font-semibold text-slate-600 uppercase tracking-wider cursor-pointer hover:bg-slate-100 transition-colors"
                      onClick={() => handleSort('price')}
                    >
                      <div className="flex items-center justify-end gap-2">
                        Preis
                        {getSortIcon('price')}
                      </div>
                    </th>
                    <th className="px-6 py-3 text-center text-xs font-semibold text-slate-600 uppercase tracking-wider">
                      Bezahlt
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-semibold text-slate-600 uppercase tracking-wider">
                      Aktionen
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-200">
                  {filteredAndSortedBookings.map((booking) => (
                    <tr key={booking.id} className="hover:bg-slate-50 transition-colors">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="text-sm font-semibold text-slate-900">{booking.reservierungsnummer}</div>
                        <div className="text-xs text-slate-500">{booking.anzahl_gaeste} Gäste</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="text-sm font-medium text-slate-900">
                          {booking.guest.vorname} {booking.guest.nachname}
                        </div>
                        <div className="text-xs text-slate-500">{booking.guest.email}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="text-sm font-medium text-slate-900">{booking.room.name}</div>
                        <div className="text-xs text-slate-500">{booking.room.ort}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="text-sm text-slate-900">
                          {format(new Date(booking.checkin_date), 'dd.MM.yyyy', { locale: de })}
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="text-sm text-slate-900">
                          {format(new Date(booking.checkout_date), 'dd.MM.yyyy', { locale: de })}
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        {getStatusBadge(booking.status)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right">
                        <div className="text-sm font-semibold text-slate-900">
                          {booking.gesamtpreis.toFixed(2)} €
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-center">
                        {booking.bezahlt ? (
                          <div className="inline-flex flex-col items-center">
                            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-100 text-emerald-800 border border-emerald-200">
                              <CheckCircle className="w-3.5 h-3.5" />
                              Bezahlt
                            </span>
                            {booking.bezahlt_am && (
                              <span className="text-xs text-slate-500 mt-1">
                                {format(new Date(booking.bezahlt_am), 'dd.MM.yyyy', { locale: de })}
                              </span>
                            )}
                          </div>
                        ) : (
                          <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-amber-100 text-amber-800 border border-amber-200">
                            <AlertCircle className="w-3.5 h-3.5" />
                            Offen
                          </span>
                        )}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right">
                        <div className="flex items-center justify-end gap-2">
                          <button
                            onClick={() => {
                              setDetailsBookingId(booking.id);
                              setShowDetails(true);
                            }}
                            className="inline-flex items-center gap-1 px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-100 rounded-lg transition-colors"
                          >
                            <Eye className="w-3.5 h-3.5" />
                            Details
                          </button>
                          <button
                            onClick={() => {
                              setSelectedBooking(booking);
                              setShowDialog(true);
                            }}
                            className="inline-flex items-center gap-1 px-3 py-1.5 text-sm text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                          >
                            <Edit2 className="w-3.5 h-3.5" />
                            Bearbeiten
                          </button>
                          <button
                            onClick={() => handleDeleteBooking(booking.id, booking.reservierungsnummer)}
                            className="inline-flex items-center gap-1 px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                            Löschen
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>

      {/* Booking Dialog */}
      <BookingDialog
        isOpen={showDialog}
        onClose={() => setShowDialog(false)}
        onSuccess={() => {
          loadBookings();
          setShowDialog(false);
        }}
        booking={selectedBooking}
      />

      {/* Booking Details */}
      {detailsBookingId && (
        <ErrorBoundary>
          <BookingDetails
            bookingId={detailsBookingId}
            isOpen={showDetails}
            onClose={() => {
              setShowDetails(false);
              setDetailsBookingId(null);
            }}
            onEdit={() => {
              const booking = bookings.find(b => b.id === detailsBookingId);
              if (booking) {
                setSelectedBooking(booking);
                setShowDetails(false);
                setShowDialog(true);
              }
            }}
          />
        </ErrorBoundary>
      )}
    </div>
  );
}
