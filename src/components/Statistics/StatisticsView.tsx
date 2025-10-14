import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { TrendingUp, Home, Users, Calendar, DollarSign, Clock, Award, UserCheck, Repeat, XCircle, CalendarRange, Heart, Briefcase, Moon } from 'lucide-react';

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  price_member: number;
  price_non_member: number;
  ort: string;
}

interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon: string;
  dpolg_mitglied: boolean;
}

interface BookingWithDetails {
  id: number;
  room_id: number;
  guest_id: number;
  reservierungsnummer: string;
  checkin_date: string;
  checkout_date: string;
  anzahl_gaeste: number;
  anzahl_naechte: number;
  status: string;
  gesamtpreis: number;
  ist_stiftungsfall: boolean;
  room: Room;
  guest: Guest;
}

type DateRange = 'thisMonth' | 'lastMonth' | 'thisYear' | 'lastYear' | 'allTime' | 'customYear' | 'customRange';

interface StatCardProps {
  title: string;
  value: string | number;
  icon: React.ElementType;
  iconColor: string;
  trend?: string;
}

function StatCard({ title, value, icon: Icon, iconColor, trend }: StatCardProps) {
  return (
    <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-xl p-6 border border-slate-700 hover:border-slate-600 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div className={`${iconColor} p-3 rounded-lg`}>
          <Icon className="w-6 h-6 text-white" />
        </div>
        {trend && (
          <span className="text-xs text-emerald-400 font-semibold">
            {trend}
          </span>
        )}
      </div>
      <h3 className="text-sm font-medium text-slate-400 mb-1">{title}</h3>
      <p className="text-3xl font-bold text-white">{value}</p>
    </div>
  );
}

export default function StatisticsView() {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [bookings, setBookings] = useState<BookingWithDetails[]>([]);
  const [guests, setGuests] = useState<Guest[]>([]);
  const [loading, setLoading] = useState(true);
  const [dateRange, setDateRange] = useState<DateRange>('allTime');
  const [selectedYear, setSelectedYear] = useState<number>(new Date().getFullYear());
  const [customStartDate, setCustomStartDate] = useState<string>('');
  const [customEndDate, setCustomEndDate] = useState<string>('');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [roomsData, bookingsData, guestsData] = await Promise.all([
        invoke<Room[]>('get_all_rooms'),
        invoke<BookingWithDetails[]>('get_all_bookings'),
        invoke<Guest[]>('get_all_guests_command'),
      ]);

      console.log('═══════════════════════════════════════');
      console.log('📊 STATISTICS DEBUG - Raw Data Loaded:');
      console.log('  Rooms:', roomsData.length, roomsData);
      console.log('  Bookings:', bookingsData.length, bookingsData);
      console.log('  Guests:', guestsData.length, guestsData);
      console.log('═══════════════════════════════════════');

      setRooms(roomsData);
      setBookings(bookingsData);
      setGuests(guestsData);
    } catch (error) {
      console.error('Fehler beim Laden der Daten:', error);
    } finally {
      setLoading(false);
    }
  };

  // Get available years from bookings
  const getAvailableYears = (): number[] => {
    const years = new Set<number>();
    bookings.forEach(booking => {
      const checkinYear = new Date(booking.checkin_date).getFullYear();
      const checkoutYear = new Date(booking.checkout_date).getFullYear();
      years.add(checkinYear);
      years.add(checkoutYear);
    });
    return Array.from(years).sort((a, b) => b - a); // Neueste zuerst
  };

  // Date range filter
  const getDateRangeBounds = (): { start: Date; end: Date } => {
    const now = new Date();
    const start = new Date();
    const end = new Date();

    switch (dateRange) {
      case 'thisMonth':
        start.setDate(1);
        start.setHours(0, 0, 0, 0);
        end.setMonth(end.getMonth() + 1);
        end.setDate(0);
        end.setHours(23, 59, 59, 999);
        break;
      case 'lastMonth':
        start.setMonth(start.getMonth() - 1);
        start.setDate(1);
        start.setHours(0, 0, 0, 0);
        end.setDate(0);
        end.setHours(23, 59, 59, 999);
        break;
      case 'thisYear':
        start.setMonth(0);
        start.setDate(1);
        start.setHours(0, 0, 0, 0);
        end.setMonth(11);
        end.setDate(31);
        end.setHours(23, 59, 59, 999);
        break;
      case 'lastYear':
        start.setFullYear(start.getFullYear() - 1);
        start.setMonth(0);
        start.setDate(1);
        start.setHours(0, 0, 0, 0);
        end.setFullYear(end.getFullYear() - 1);
        end.setMonth(11);
        end.setDate(31);
        end.setHours(23, 59, 59, 999);
        break;
      case 'customYear':
        start.setFullYear(selectedYear);
        start.setMonth(0);
        start.setDate(1);
        start.setHours(0, 0, 0, 0);
        end.setFullYear(selectedYear);
        end.setMonth(11);
        end.setDate(31);
        end.setHours(23, 59, 59, 999);
        break;
      case 'customRange':
        if (customStartDate && customEndDate) {
          const customStart = new Date(customStartDate);
          const customEnd = new Date(customEndDate);
          customStart.setHours(0, 0, 0, 0);
          customEnd.setHours(23, 59, 59, 999);
          return { start: customStart, end: customEnd };
        }
        // Fallback wenn keine Daten gesetzt
        start.setFullYear(2000);
        end.setFullYear(2100);
        break;
      case 'allTime':
        start.setFullYear(2000);
        end.setFullYear(2100);
        break;
    }

    return { start, end };
  };

  const isBookingInRange = (booking: BookingWithDetails): boolean => {
    const { start, end } = getDateRangeBounds();
    const checkin = new Date(booking.checkin_date);
    const checkout = new Date(booking.checkout_date);

    // Buchung ist im Zeitraum wenn Check-in oder Check-out im Bereich liegt
    // ODER wenn die Buchung den gesamten Zeitraum überspannt
    return (
      (checkin >= start && checkin <= end) ||
      (checkout >= start && checkout <= end) ||
      (checkin <= start && checkout >= end)
    );
  };

  // Filter bookings by date range
  const filteredBookings = bookings.filter(isBookingInRange);

  console.log('═══════════════════════════════════════');
  console.log('📊 STATISTICS DEBUG - After Filtering:');
  console.log('  dateRange:', dateRange);
  console.log('  Date bounds:', getDateRangeBounds());
  console.log('  Total bookings:', bookings.length);
  console.log('  Filtered bookings:', filteredBookings.length);
  console.log('  Filtered booking details:', filteredBookings);
  console.log('═══════════════════════════════════════');

  // Filter guests who have bookings in the selected range
  const guestsInRange = guests.filter(guest =>
    filteredBookings.some(b => b.guest_id === guest.id)
  );

  // Helper functions for calculations
  const calculateNights = (checkin: string, checkout: string): number => {
    const start = new Date(checkin);
    const end = new Date(checkout);
    return Math.ceil((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24));
  };

  // Calculate nights within the date range
  const calculateNightsInRange = (booking: BookingWithDetails): number => {
    const { start, end } = getDateRangeBounds();
    const checkin = new Date(booking.checkin_date);
    const checkout = new Date(booking.checkout_date);

    // Overlap berechnen
    const effectiveCheckin = checkin > start ? checkin : start;
    const effectiveCheckout = checkout < end ? checkout : end;

    if (effectiveCheckin >= effectiveCheckout) return 0;

    return Math.ceil((effectiveCheckout.getTime() - effectiveCheckin.getTime()) / (1000 * 60 * 60 * 24));
  };

  // Room Statistics
  const activeBookings = filteredBookings.filter(b => b.status !== 'storniert');

  console.log('═══════════════════════════════════════');
  console.log('📊 STATISTICS DEBUG - Active Bookings:');
  console.log('  Active bookings (nicht storniert):', activeBookings.length);
  console.log('  Active booking statuses:', activeBookings.map(b => b.status));
  console.log('═══════════════════════════════════════');

  // Auslastung: (Gebuchte Zimmernächte / Verfügbare Zimmernächte) × 100
  const { start, end } = getDateRangeBounds();
  const daysInRange = Math.ceil((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24));
  const availableRoomNights = rooms.length * daysInRange;

  const bookedRoomNights = activeBookings.reduce((sum, b) => {
    const nights = calculateNightsInRange(b);
    console.log(`  Booking ${b.reservierungsnummer}: ${nights} nights in range`);
    return sum + nights;
  }, 0);

  console.log('═══════════════════════════════════════');
  console.log('📊 STATISTICS DEBUG - Occupancy:');
  console.log('  Days in range:', daysInRange);
  console.log('  Available room nights:', availableRoomNights);
  console.log('  Booked room nights:', bookedRoomNights);
  console.log('  Occupancy rate:', Math.round((bookedRoomNights / availableRoomNights) * 100), '%');
  console.log('═══════════════════════════════════════');

  const occupancyRate = availableRoomNights > 0
    ? Math.round((bookedRoomNights / availableRoomNights) * 100)
    : 0;

  // ADR (Average Daily Rate) - Durchschnittlicher Tagespreis
  const totalRevenue = activeBookings.reduce((sum, b) => sum + b.gesamtpreis, 0);
  const totalNights = activeBookings.reduce((sum, b) => {
    return sum + calculateNightsInRange(b);
  }, 0);
  const adr = totalNights > 0 ? totalRevenue / totalNights : 0;

  console.log('═══════════════════════════════════════');
  console.log('📊 STATISTICS DEBUG - Revenue:');
  console.log('  Total revenue:', totalRevenue);
  console.log('  Total nights:', totalNights);
  console.log('  ADR:', adr);
  console.log('═══════════════════════════════════════');

  // RevPAR (Revenue Per Available Room) - Umsatz pro verfügbarem Zimmer
  const revpar = availableRoomNights > 0 ? totalRevenue / rooms.length / daysInRange : 0;

  // Most booked room in this period
  const roomBookingCount = activeBookings.reduce((acc, b) => {
    acc[b.room_id] = (acc[b.room_id] || 0) + 1;
    return acc;
  }, {} as Record<number, number>);
  const mostBookedRoomId = Object.entries(roomBookingCount)
    .sort(([, a], [, b]) => b - a)[0]?.[0];
  const mostBookedRoom = rooms.find(r => r.id === Number(mostBookedRoomId));

  // Room by type statistics
  const roomsByType = rooms.reduce((acc, room) => {
    acc[room.gebaeude_typ] = (acc[room.gebaeude_typ] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  // Guest Statistics (only for guests with bookings in range)
  const memberGuests = guestsInRange.filter(g => g.dpolg_mitglied).length;
  const nonMemberGuests = guestsInRange.filter(g => !g.dpolg_mitglied).length;
  const memberPercentage = guestsInRange.length > 0
    ? Math.round((memberGuests / guestsInRange.length) * 100)
    : 0;
  const nonMemberPercentage = guestsInRange.length > 0
    ? Math.round((nonMemberGuests / guestsInRange.length) * 100)
    : 0;

  // Average length of stay
  const avgLengthOfStay = activeBookings.length > 0
    ? totalNights / activeBookings.length
    : 0;

  // Return guest rate (guests who have booked more than once OVERALL, not just in this period)
  const returnGuests = guestsInRange.filter(guest => {
    // Count ALL bookings for this guest (across all time, not just in range)
    const totalGuestBookings = bookings.filter(b =>
      b.guest_id === guest.id &&
      b.status !== 'storniert'
    ).length;
    return totalGuestBookings > 1; // Has booked more than once overall
  }).length;

  const returnGuestRate = guestsInRange.length > 0
    ? Math.round((returnGuests / guestsInRange.length) * 100)
    : 0;

  // Cancelled bookings in this period
  const cancelledBookings = filteredBookings.filter(b => b.status === 'storniert').length;
  const cancelRate = filteredBookings.length > 0
    ? Math.round((cancelledBookings / filteredBookings.length) * 100)
    : 0;

  // Booking Categories - Stiftungsfälle vs Normal Urlauber
  const stiftungsfaelle = activeBookings.filter(b => b.ist_stiftungsfall).length;
  const normalUrlauber = activeBookings.filter(b => !b.ist_stiftungsfall).length;

  // Übernachtungszahlen (Nights stayed)
  const gesamtNaechte = activeBookings.reduce((sum, b) => sum + b.anzahl_naechte, 0);
  const naechteNormalUrlauber = activeBookings
    .filter(b => !b.ist_stiftungsfall)
    .reduce((sum, b) => sum + b.anzahl_naechte, 0);
  const naechteStiftungsfaelle = activeBookings
    .filter(b => b.ist_stiftungsfall)
    .reduce((sum, b) => sum + b.anzahl_naechte, 0);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          <p className="mt-4 text-slate-400">Lade Statistiken...</p>
        </div>
      </div>
    );
  }

  const dateRangeLabels: Record<Exclude<DateRange, 'customYear' | 'customRange'>, string> = {
    thisMonth: 'Dieser Monat',
    lastMonth: 'Letzter Monat',
    thisYear: 'Dieses Jahr',
    lastYear: 'Letztes Jahr',
    allTime: 'Gesamter Zeitraum',
  };

  const availableYears = getAvailableYears();

  return (
    <div className="h-full overflow-y-auto bg-gradient-to-br from-slate-800 to-slate-900 p-6">
      {/* Date Range Selector */}
      <div className="mb-6 bg-slate-900/50 backdrop-blur-sm rounded-xl p-5 border border-slate-700/50 shadow-xl">
        <div className="flex items-start gap-6">
          {/* Icon */}
          <div className="mt-2">
            <CalendarRange className="w-5 h-5 text-blue-400" />
          </div>

          <div className="flex-1 space-y-4">
            {/* Schnellauswahl */}
            <div>
              <label className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2 block">Zeitraum</label>
              <div className="flex gap-2 flex-wrap">
                {(Object.keys(dateRangeLabels) as Array<keyof typeof dateRangeLabels>).map((range) => (
                  <button
                    key={range}
                    onClick={() => setDateRange(range)}
                    className={`px-3 py-1.5 rounded-lg font-medium text-sm transition-all ${
                      dateRange === range
                        ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/30'
                        : 'bg-slate-800 text-slate-300 hover:bg-slate-700 border border-slate-700'
                    }`}
                  >
                    {dateRangeLabels[range]}
                  </button>
                ))}
              </div>
            </div>

            {/* Jahr & Individuell - Nebeneinander */}
            <div className="grid grid-cols-2 gap-4">
              {/* Jahr-Auswahl */}
              <div>
                <label className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2 block">Jahr</label>
                <select
                  value={selectedYear}
                  onChange={(e) => {
                    setSelectedYear(parseInt(e.target.value));
                    setDateRange('customYear');
                  }}
                  className={`w-full px-4 py-2 rounded-lg text-sm font-medium appearance-none cursor-pointer focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all ${
                    dateRange === 'customYear'
                      ? 'bg-blue-600 border-2 border-blue-500 text-white shadow-lg shadow-blue-500/30'
                      : 'bg-slate-800 border border-slate-700 text-slate-200 hover:bg-slate-750 hover:border-slate-600'
                  }`}
                  style={{
                    backgroundImage: dateRange === 'customYear'
                      ? `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='20' height='20' viewBox='0 0 24 24' fill='none' stroke='%23ffffff' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E")`
                      : `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='20' height='20' viewBox='0 0 24 24' fill='none' stroke='%2394a3b8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E")`,
                    backgroundRepeat: 'no-repeat',
                    backgroundPosition: 'right 0.75rem center',
                    backgroundSize: '1.25rem',
                    paddingRight: '2.5rem'
                  }}
                >
                  {availableYears.map(year => (
                    <option key={year} value={year}>{year}</option>
                  ))}
                </select>
              </div>

              {/* Individueller Zeitraum */}
              <div>
                <label className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2 block">Individuell</label>
                <div className={`flex items-center gap-2 px-3 py-2 rounded-lg transition-all ${
                  dateRange === 'customRange'
                    ? 'bg-blue-600 border-2 border-blue-500 shadow-lg shadow-blue-500/30'
                    : 'bg-slate-800 border border-slate-700'
                }`}>
                  <input
                    type="date"
                    value={customStartDate}
                    onChange={(e) => {
                      setCustomStartDate(e.target.value);
                      if (e.target.value && customEndDate) {
                        setDateRange('customRange');
                      }
                    }}
                    className={`flex-1 px-2 py-1 rounded text-xs font-medium bg-transparent border-none focus:outline-none ${
                      dateRange === 'customRange' ? 'text-white placeholder-blue-200' : 'text-slate-200 placeholder-slate-500'
                    }`}
                    placeholder="Von"
                  />
                  <span className={dateRange === 'customRange' ? 'text-blue-200' : 'text-slate-600'}>—</span>
                  <input
                    type="date"
                    value={customEndDate}
                    onChange={(e) => {
                      setCustomEndDate(e.target.value);
                      if (customStartDate && e.target.value) {
                        setDateRange('customRange');
                      }
                    }}
                    className={`flex-1 px-2 py-1 rounded text-xs font-medium bg-transparent border-none focus:outline-none ${
                      dateRange === 'customRange' ? 'text-white placeholder-blue-200' : 'text-slate-200 placeholder-slate-500'
                    }`}
                    placeholder="Bis"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Overview Section */}
      <div className="mb-8">
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
          <TrendingUp className="w-6 h-6 text-blue-400" />
          Übersicht
        </h2>
        <div className="grid grid-cols-4 gap-4">
          <StatCard
            title="Gesamtumsatz"
            value={`${totalRevenue.toLocaleString('de-DE', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} €`}
            icon={DollarSign}
            iconColor="bg-emerald-500"
          />
          <StatCard
            title="Auslastung"
            value={`${occupancyRate}%`}
            icon={TrendingUp}
            iconColor="bg-purple-500"
          />
          <StatCard
            title="ADR (Ø Tagespreis)"
            value={`${adr.toLocaleString('de-DE', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} €`}
            icon={DollarSign}
            iconColor="bg-blue-500"
          />
          <StatCard
            title="RevPAR"
            value={`${revpar.toLocaleString('de-DE', { minimumFractionDigits: 2, maximumFractionDigits: 2 })} €`}
            icon={DollarSign}
            iconColor="bg-indigo-500"
          />
        </div>
      </div>

      {/* Booking Categories Section */}
      <div className="mb-8">
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
          <Calendar className="w-6 h-6 text-blue-400" />
          Buchungs-Kategorien
        </h2>

        <div className="grid grid-cols-3 gap-4 mb-6">
          <StatCard
            title="Stiftungsfälle"
            value={stiftungsfaelle}
            icon={Heart}
            iconColor="bg-pink-500"
          />
          <StatCard
            title="Normal Urlauber"
            value={normalUrlauber}
            icon={Briefcase}
            iconColor="bg-blue-500"
          />
          <StatCard
            title="Gesamt Übernachtungen"
            value={`${gesamtNaechte.toLocaleString('de-DE')} Nächte`}
            icon={Moon}
            iconColor="bg-indigo-500"
          />
        </div>

        {/* Übernachtungszahlen Breakdown */}
        <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-xl p-6 border border-slate-700">
          <h3 className="text-lg font-bold text-white mb-4">Übernachtungen nach Kategorie</h3>
          <div className="grid grid-cols-3 gap-4">
            <div className="bg-slate-700/50 rounded-lg p-6">
              <div className="flex items-center gap-3 mb-2">
                <Moon className="w-5 h-5 text-indigo-400" />
                <p className="text-sm text-slate-400 font-medium">Gesamt</p>
              </div>
              <p className="text-4xl font-bold text-white">{gesamtNaechte.toLocaleString('de-DE')}</p>
              <p className="text-sm text-slate-400 mt-1">Nächte insgesamt</p>
            </div>
            <div className="bg-blue-500/20 rounded-lg p-6 border border-blue-500/30">
              <div className="flex items-center gap-3 mb-2">
                <Briefcase className="w-5 h-5 text-blue-400" />
                <p className="text-sm text-blue-300 font-medium">Normal Urlauber</p>
              </div>
              <p className="text-4xl font-bold text-white">{naechteNormalUrlauber.toLocaleString('de-DE')}</p>
              <p className="text-sm text-blue-300 mt-1">{normalUrlauber} Buchungen</p>
            </div>
            <div className="bg-pink-500/20 rounded-lg p-6 border border-pink-500/30">
              <div className="flex items-center gap-3 mb-2">
                <Heart className="w-5 h-5 text-pink-400" />
                <p className="text-sm text-pink-300 font-medium">Stiftungsfälle</p>
              </div>
              <p className="text-4xl font-bold text-white">{naechteStiftungsfaelle.toLocaleString('de-DE')}</p>
              <p className="text-sm text-pink-300 mt-1">{stiftungsfaelle} Buchungen</p>
            </div>
          </div>
        </div>
      </div>

      {/* Room Statistics Section */}
      <div className="mb-8">
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
          <Home className="w-6 h-6 text-blue-400" />
          Zimmer-Statistiken
        </h2>

        <div className="grid grid-cols-4 gap-4 mb-6">
          <StatCard
            title="Gesamtzahl Zimmer"
            value={rooms.length}
            icon={Home}
            iconColor="bg-blue-500"
          />
          <StatCard
            title="Aktive Buchungen"
            value={activeBookings.length}
            icon={Calendar}
            iconColor="bg-emerald-500"
          />
          <StatCard
            title="Meistgebuchtes Zimmer"
            value={mostBookedRoom ? mostBookedRoom.name : 'N/A'}
            icon={Award}
            iconColor="bg-amber-500"
          />
          <StatCard
            title="Ø Aufenthaltsdauer"
            value={`${avgLengthOfStay.toFixed(1)} Nächte`}
            icon={Clock}
            iconColor="bg-purple-500"
          />
        </div>

        {/* Room Types Breakdown */}
        <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-xl p-6 border border-slate-700">
          <h3 className="text-lg font-bold text-white mb-4">Zimmer nach Typ</h3>
          <div className="grid grid-cols-3 gap-4">
            {Object.entries(roomsByType).map(([type, count]) => (
              <div key={type} className="bg-slate-700/50 rounded-lg p-4">
                <p className="text-sm text-slate-400 mb-1">{type}</p>
                <p className="text-2xl font-bold text-white">{count} Zimmer</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Guest Statistics Section */}
      <div className="mb-8">
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
          <Users className="w-6 h-6 text-blue-400" />
          Gäste-Statistiken
        </h2>

        <div className="grid grid-cols-4 gap-4 mb-6">
          <StatCard
            title="Gäste im Zeitraum"
            value={guestsInRange.length}
            icon={Users}
            iconColor="bg-blue-500"
          />
          <StatCard
            title="DPolG Mitglieder"
            value={`${memberPercentage}%`}
            icon={UserCheck}
            iconColor="bg-emerald-500"
          />
          <StatCard
            title="Wiederkehrende Gäste"
            value={`${returnGuestRate}%`}
            icon={Repeat}
            iconColor="bg-purple-500"
          />
          <StatCard
            title="Stornierungsrate"
            value={`${cancelRate}%`}
            icon={XCircle}
            iconColor="bg-red-500"
          />
        </div>

        {/* Member vs Non-Member Breakdown */}
        <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-xl p-6 border border-slate-700">
          <h3 className="text-lg font-bold text-white mb-4">Gäste-Segmentierung</h3>
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-emerald-500/20 rounded-lg p-6 border border-emerald-500/30">
              <div className="flex items-center gap-3 mb-2">
                <UserCheck className="w-5 h-5 text-emerald-400" />
                <p className="text-sm text-emerald-300 font-medium">DPolG Mitglieder</p>
              </div>
              <p className="text-4xl font-bold text-white">{memberGuests}</p>
              <p className="text-sm text-emerald-300 mt-1">{memberPercentage}% aller Gäste</p>
            </div>
            <div className="bg-blue-500/20 rounded-lg p-6 border border-blue-500/30">
              <div className="flex items-center gap-3 mb-2">
                <Users className="w-5 h-5 text-blue-400" />
                <p className="text-sm text-blue-300 font-medium">Nicht-Mitglieder</p>
              </div>
              <p className="text-4xl font-bold text-white">{nonMemberGuests}</p>
              <p className="text-sm text-blue-300 mt-1">{nonMemberPercentage}% aller Gäste</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
