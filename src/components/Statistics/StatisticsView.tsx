import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { TrendingUp, Home, Users, Calendar, DollarSign, Clock, Award, UserCheck, Repeat, XCircle } from 'lucide-react';

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
  status: string;
  gesamtpreis: number;
  room: Room;
  guest: Guest;
}

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

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [roomsData, bookingsData, guestsData] = await Promise.all([
        invoke<Room[]>('get_all_rooms'),
        invoke<BookingWithDetails[]>('get_all_bookings'),
        invoke<Guest[]>('get_all_guests'),
      ]);
      setRooms(roomsData);
      setBookings(bookingsData);
      setGuests(guestsData);
    } catch (error) {
      console.error('Fehler beim Laden der Daten:', error);
    } finally {
      setLoading(false);
    }
  };

  // Helper functions for calculations
  const calculateNights = (checkin: string, checkout: string): number => {
    const start = new Date(checkin);
    const end = new Date(checkout);
    return Math.ceil((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24));
  };

  // Room Statistics
  const activeBookings = bookings.filter(b => b.status !== 'storniert');
  const occupancyRate = rooms.length > 0
    ? Math.round((activeBookings.length / rooms.length) * 100)
    : 0;

  // ADR (Average Daily Rate) - Durchschnittlicher Tagespreis
  const totalRevenue = activeBookings.reduce((sum, b) => sum + b.gesamtpreis, 0);
  const totalNights = activeBookings.reduce((sum, b) => {
    return sum + calculateNights(b.checkin_date, b.checkout_date);
  }, 0);
  const adr = totalNights > 0 ? totalRevenue / totalNights : 0;

  // RevPAR (Revenue Per Available Room)
  const revpar = rooms.length > 0 ? totalRevenue / rooms.length : 0;

  // Most booked room
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

  // Guest Statistics
  const memberGuests = guests.filter(g => g.dpolg_mitglied).length;
  const nonMemberGuests = guests.filter(g => !g.dpolg_mitglied).length;
  const memberPercentage = guests.length > 0
    ? Math.round((memberGuests / guests.length) * 100)
    : 0;

  // Average length of stay
  const avgLengthOfStay = activeBookings.length > 0
    ? totalNights / activeBookings.length
    : 0;

  // Return guest rate (guests with more than 1 booking)
  const guestBookingCount = activeBookings.reduce((acc, b) => {
    acc[b.guest_id] = (acc[b.guest_id] || 0) + 1;
    return acc;
  }, {} as Record<number, number>);
  const returnGuests = Object.values(guestBookingCount).filter(count => count > 1).length;
  const returnGuestRate = guests.length > 0
    ? Math.round((returnGuests / guests.length) * 100)
    : 0;

  // Cancelled bookings (No-show rate approximation)
  const cancelledBookings = bookings.filter(b => b.status === 'storniert').length;
  const cancelRate = bookings.length > 0
    ? Math.round((cancelledBookings / bookings.length) * 100)
    : 0;

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

  return (
    <div className="h-full overflow-y-auto bg-gradient-to-br from-slate-800 to-slate-900 p-6">
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
            title="Gesamtzahl Gäste"
            value={guests.length}
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
              <p className="text-sm text-blue-300 mt-1">{100 - memberPercentage}% aller Gäste</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
