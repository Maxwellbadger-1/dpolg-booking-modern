import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import TapeChart from './components/TapeChart';
import DevTools from './components/DevTools';
import GuestDialog from './components/GuestManagement/GuestDialog';
import { Calendar, Hotel, UserPlus } from 'lucide-react';

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
  bemerkungen?: string;
  room: Room;
  guest: Guest;
}

function App() {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [bookings, setBookings] = useState<BookingWithDetails[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showGuestDialog, setShowGuestDialog] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      console.log('Loading rooms...');
      const roomsData = await invoke<Room[]>('get_all_rooms');
      console.log('Rooms loaded:', roomsData);

      console.log('Loading bookings...');
      const bookingsData = await invoke<BookingWithDetails[]>('get_all_bookings');
      console.log('Bookings loaded:', bookingsData);

      setRooms(roomsData);
      setBookings(bookingsData);
      setError(null);
    } catch (err) {
      console.error('FULL ERROR:', err);
      console.error('Error type:', typeof err);
      console.error('Error object:', JSON.stringify(err, null, 2));
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-50">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          <p className="mt-4 text-gray-600">Lade Daten...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-50">
        <div className="text-center max-w-md">
          <div className="text-red-500 mb-4">⚠️ Fehler beim Laden</div>
          <p className="text-sm text-gray-600">{error}</p>
          <button
            onClick={loadData}
            className="mt-4 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
          >
            Erneut versuchen
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen bg-gradient-to-br from-slate-50 to-slate-100">
      {/* Header - Compact Single Row */}
      <header className="bg-gradient-to-r from-slate-800 to-slate-900 shadow-2xl border-b-2 border-slate-700">
        <div className="px-6 py-3 flex items-center justify-between">
          {/* Left: Logo + Title + Stats */}
          <div className="flex items-center gap-6">
            <div className="flex items-center gap-3">
              <div className="bg-gradient-to-br from-blue-500 to-blue-600 p-2 rounded-xl shadow-lg">
                <Hotel className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-xl font-bold text-white tracking-tight">
                  DPolG Buchungssystem
                </h1>
              </div>
            </div>

            {/* Stats Inline */}
            <div className="flex items-center gap-6 border-l border-slate-600 pl-6">
              <div className="flex items-center gap-2">
                <div className="bg-blue-500/20 p-1.5 rounded-lg">
                  <svg className="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                  </svg>
                </div>
                <div className="flex items-baseline gap-1.5">
                  <span className="text-lg font-bold text-white">{rooms.length}</span>
                  <span className="text-xs text-slate-400">Zimmer</span>
                </div>
              </div>

              <div className="flex items-center gap-2">
                <div className="bg-emerald-500/20 p-1.5 rounded-lg">
                  <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                  </svg>
                </div>
                <div className="flex items-baseline gap-1.5">
                  <span className="text-lg font-bold text-white">{bookings.filter(b => b.status !== 'storniert').length}</span>
                  <span className="text-xs text-slate-400">Aktiv</span>
                </div>
              </div>

              <div className="flex items-center gap-2">
                <div className="bg-purple-500/20 p-1.5 rounded-lg">
                  <svg className="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                  </svg>
                </div>
                <div className="flex items-baseline gap-1.5">
                  <span className="text-lg font-bold text-white">{Math.round((bookings.filter(b => b.status !== 'storniert').length / rooms.length) * 100)}%</span>
                  <span className="text-xs text-slate-400">Auslastung</span>
                </div>
              </div>
            </div>
          </div>

          {/* Right: Actions + Date */}
          <div className="flex items-center gap-3">
            <button
              onClick={() => setShowGuestDialog(true)}
              className="flex items-center gap-2 btn-primary bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white font-semibold rounded-lg transition-all shadow-lg hover:shadow-xl"
            >
              <UserPlus className="w-4 h-4" />
              Neuer Gast
            </button>

            <div className="flex items-center gap-2 bg-slate-700/50 date-display py-2 rounded-lg backdrop-blur-sm">
              <Calendar className="w-4 h-4 text-blue-400" />
              <span className="text-xs font-semibold text-white">{new Date().toLocaleDateString('de-DE', {
                weekday: 'short',
                day: '2-digit',
                month: 'short',
                year: 'numeric'
              })}</span>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 overflow-hidden">
        <TapeChart rooms={rooms} bookings={bookings} />
      </main>

      {/* DevTools - nur während Development */}
      <DevTools />

      {/* Guest Dialog */}
      <GuestDialog
        isOpen={showGuestDialog}
        onClose={() => setShowGuestDialog(false)}
        onSuccess={() => {
          console.log('Gast erfolgreich gespeichert');
          setShowGuestDialog(false);
        }}
      />
    </div>
  );
}

export default App;