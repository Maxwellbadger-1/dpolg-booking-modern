import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DataProvider } from './context/DataContext';
import TapeChart from './components/TapeChart';
import BookingList from './components/BookingManagement/BookingList';
import GuestList from './components/GuestManagement/GuestList';
import RoomList from './components/RoomManagement/RoomList';
import DevTools from './components/DevTools';
import GuestDialog from './components/GuestManagement/GuestDialog';
import SettingsDialog from './components/Settings/SettingsDialog';
import EmailHistoryView from './components/Email/EmailHistoryView';
import TemplatesManagement from './components/TemplatesManagement/TemplatesManagement';
import BookingDialog from './components/BookingManagement/BookingDialog';
import QuickBookingFAB from './components/QuickBookingFAB';
import DashboardQuickStats from './components/DashboardQuickStats';
import StatisticsView from './components/Statistics/StatisticsView';
import EmailSelectionDialog from './components/BookingManagement/EmailSelectionDialog';
import { Calendar, Hotel, UserPlus, LayoutDashboard, CalendarCheck, Users, Settings, Mail, Briefcase, TrendingUp } from 'lucide-react';

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

type Tab = 'dashboard' | 'bookings' | 'guests' | 'rooms' | 'emails' | 'templates' | 'statistics';

function App() {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [bookings, setBookings] = useState<BookingWithDetails[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showGuestDialog, setShowGuestDialog] = useState(false);
  const [showSettingsDialog, setShowSettingsDialog] = useState(false);
  const [showBookingDialog, setShowBookingDialog] = useState(false);
  const [showEmailDialog, setShowEmailDialog] = useState(false);
  const [selectedBookingId, setSelectedBookingId] = useState<number | undefined>(undefined);
  const [emailBookingId, setEmailBookingId] = useState<number | undefined>(undefined);
  const [prefillData, setPrefillData] = useState<{ roomId?: number; checkinDate?: string; checkoutDate?: string } | undefined>(undefined);
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');

  useEffect(() => {
    loadData();
    updateBookingStatuses(); // Status-Update bei App-Start
  }, []);

  // Auto-Update der Buchungs-Status alle 10 Minuten
  useEffect(() => {
    const interval = setInterval(() => {
      updateBookingStatuses();
    }, 10 * 60 * 1000); // 10 Minuten

    return () => clearInterval(interval);
  }, []);

  const updateBookingStatuses = async () => {
    try {
      console.log('🔄 Updating booking statuses...');
      const changedCount = await invoke<number>('update_booking_statuses_command');
      if (changedCount > 0) {
        console.log(`✅ ${changedCount} Buchungs-Status aktualisiert`);
        // Daten neu laden wenn sich etwas geändert hat
        loadData();
      }
    } catch (error) {
      console.error('❌ Fehler beim Status-Update:', error);
    }
  };

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

  const tabs = [
    { id: 'dashboard' as Tab, label: 'Dashboard', icon: LayoutDashboard },
    { id: 'bookings' as Tab, label: 'Buchungen', icon: CalendarCheck },
    { id: 'guests' as Tab, label: 'Gäste', icon: Users },
    { id: 'rooms' as Tab, label: 'Zimmer', icon: Hotel },
    { id: 'templates' as Tab, label: 'Services & Rabatte', icon: Briefcase },
    { id: 'emails' as Tab, label: 'Email-Verlauf', icon: Mail },
    { id: 'statistics' as Tab, label: 'Statistiken', icon: TrendingUp },
  ];

  return (
    <DataProvider>
      <div className="flex flex-col h-screen bg-gradient-to-br from-slate-50 to-slate-100">
      {/* Header - Compact Single Row */}
      <header className="bg-gradient-to-r from-slate-800 to-slate-900 shadow-2xl border-b border-slate-700">
        <div className="px-6 py-3 flex items-center justify-between">
          {/* Left: Logo + Title + Stats */}
          <div className="flex items-center gap-6">
            <div className="flex items-center gap-3">
              <div className="bg-gradient-to-br from-blue-500 to-blue-600 p-2 rounded-xl shadow-lg">
                <Hotel className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-xl font-bold text-white tracking-tight">
                  DPolG Stiftung Buchungssystem
                </h1>
              </div>
            </div>

            {/* Stats Inline - Clickable */}
            <div className="flex items-center gap-6 border-l border-slate-600 pl-6">
              <button
                onClick={() => setActiveTab('statistics')}
                className="flex items-center gap-2 hover:bg-slate-700/50 rounded-lg px-2 py-1 transition-colors group"
                title="Zu Statistiken navigieren"
              >
                <div className="bg-blue-500/20 p-1.5 rounded-lg group-hover:bg-blue-500/30 transition-colors">
                  <svg className="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                  </svg>
                </div>
                <div className="flex items-baseline gap-1.5">
                  <span className="text-lg font-bold text-white">{rooms.length}</span>
                  <span className="text-xs text-slate-400">Zimmer</span>
                </div>
              </button>

              <button
                onClick={() => setActiveTab('statistics')}
                className="flex items-center gap-2 hover:bg-slate-700/50 rounded-lg px-2 py-1 transition-colors group"
                title="Zu Statistiken navigieren"
              >
                <div className="bg-emerald-500/20 p-1.5 rounded-lg group-hover:bg-emerald-500/30 transition-colors">
                  <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                  </svg>
                </div>
                <div className="flex items-baseline gap-1.5">
                  <span className="text-lg font-bold text-white">{bookings.filter(b => b.status !== 'storniert').length}</span>
                  <span className="text-xs text-slate-400">Aktiv</span>
                </div>
              </button>

              <button
                onClick={() => setActiveTab('statistics')}
                className="flex items-center gap-2 hover:bg-slate-700/50 rounded-lg px-2 py-1 transition-colors group"
                title="Zu Statistiken navigieren"
              >
                <div className="bg-purple-500/20 p-1.5 rounded-lg group-hover:bg-purple-500/30 transition-colors">
                  <svg className="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                  </svg>
                </div>
                <div className="flex items-baseline gap-1.5">
                  <span className="text-lg font-bold text-white">{Math.round((bookings.filter(b => b.status !== 'storniert').length / rooms.length) * 100)}%</span>
                  <span className="text-xs text-slate-400">Auslastung</span>
                </div>
              </button>
            </div>
          </div>

          {/* Right: Actions + Date */}
          <div className="flex items-center gap-3">
            <button
              onClick={() => setShowSettingsDialog(true)}
              className="flex items-center gap-2 p-2.5 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
              title="Einstellungen"
            >
              <Settings className="w-5 h-5" />
            </button>

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

        {/* Tab Navigation */}
        <div className="px-6 pt-3">
          <div className="flex items-center gap-2">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              const isActive = activeTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`
                    flex items-center gap-2 px-4 py-2 rounded-t-lg font-semibold transition-all
                    ${isActive
                      ? 'bg-gradient-to-br from-slate-50 to-slate-100 text-slate-800 shadow-lg'
                      : 'text-slate-400 hover:text-slate-300 hover:bg-slate-700/50'
                    }
                  `}
                >
                  <Icon className="w-4 h-4" />
                  <span className="text-sm">{tab.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 overflow-hidden">
        {activeTab === 'dashboard' && (
          <TapeChart
            onBookingClick={(bookingId) => {
              setSelectedBookingId(bookingId);
              setPrefillData(undefined);
              setShowBookingDialog(true);
            }}
            onCreateBooking={(roomId, startDate, endDate) => {
              console.log('🎨 Create Booking:', { roomId, startDate, endDate });
              setSelectedBookingId(undefined);
              setPrefillData({ roomId, checkinDate: startDate, checkoutDate: endDate });
              setShowBookingDialog(true);
            }}
            onBookingEdit={(bookingId) => {
              setSelectedBookingId(bookingId);
              setPrefillData(undefined);
              setShowBookingDialog(true);
            }}
            onBookingCancel={async (bookingId) => {
              const booking = bookings.find(b => b.id === bookingId);
              if (!booking) return;

              if (confirm(`Buchung ${booking.reservierungsnummer} wirklich stornieren?`)) {
                try {
                  await invoke('update_booking_status_command', {
                    id: bookingId,
                    newStatus: 'storniert'
                  });
                  loadData();
                } catch (error) {
                  alert('Fehler beim Stornieren: ' + error);
                }
              }
            }}
            onSendEmail={(bookingId) => {
              setEmailBookingId(bookingId);
              setShowEmailDialog(true);
            }}
          />
        )}
        {activeTab === 'statistics' && <StatisticsView />}
        {activeTab === 'bookings' && <BookingList />}
        {activeTab === 'guests' && <GuestList />}
        {activeTab === 'rooms' && <RoomList />}
        {activeTab === 'templates' && <TemplatesManagement />}
        {activeTab === 'emails' && <EmailHistoryView />}
      </main>

      {/* Floating Action Button - Always Visible */}
      <QuickBookingFAB onClick={() => setShowBookingDialog(true)} />

      {/* DevTools - nur während Development */}
      <DevTools />

      {/* Guest Dialog */}
      <SettingsDialog
        isOpen={showSettingsDialog}
        onClose={() => setShowSettingsDialog(false)}
      />

      <GuestDialog
        isOpen={showGuestDialog}
        onClose={() => setShowGuestDialog(false)}
        onSuccess={() => {
          console.log('Gast erfolgreich gespeichert');
          setShowGuestDialog(false);
        }}
      />

      {/* Booking Dialog */}
      <BookingDialog
        isOpen={showBookingDialog}
        onClose={() => {
          setShowBookingDialog(false);
          setSelectedBookingId(undefined);
          setPrefillData(undefined);
        }}
        onSuccess={() => {
          console.log('Buchung erfolgreich gespeichert');
          setShowBookingDialog(false);
          setSelectedBookingId(undefined);
          setPrefillData(undefined);
          loadData(); // Reload data after successful booking
        }}
        booking={selectedBookingId ? bookings.find(b => b.id === selectedBookingId) : undefined}
        prefillData={prefillData}
      />

      {/* Email Selection Dialog */}
      {emailBookingId && (() => {
        const booking = bookings.find(b => b.id === emailBookingId);
        return booking ? (
          <EmailSelectionDialog
            bookingId={emailBookingId}
            guestEmail={booking.guest.email}
            isOpen={showEmailDialog}
            onClose={() => {
              setShowEmailDialog(false);
              setEmailBookingId(undefined);
            }}
          />
        ) : null;
      })()}
      </div>
    </DataProvider>
  );
}

export default App;