import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { Toaster } from 'react-hot-toast';
import toast from 'react-hot-toast';
import type { UnlistenFn } from '@tauri-apps/api/event';
import Lottie from 'lottie-react';
import { DataProvider, useData } from './context/DataContext';
import { OnlineProvider } from './context/OnlineContext';
import { UserProvider } from './context/UserContext';
import OfflineBanner from './components/OfflineBanner';
import TapeChart from './components/TapeChart';
import BookingList from './components/BookingManagement/BookingList';
import GuestList from './components/GuestManagement/GuestList';
import RoomList from './components/RoomManagement/RoomList';
import GuestDialog from './components/GuestManagement/GuestDialog';
import SettingsDialog from './components/Settings/SettingsDialog';
import EmailHistoryView from './components/Email/EmailHistoryView';
import TemplatesManagement from './components/TemplatesManagement/TemplatesManagement';
import BookingSidebar from './components/BookingManagement/BookingSidebar';
import EmailSelectionDialog from './components/BookingManagement/EmailSelectionDialog';
import CancellationConfirmDialog from './components/BookingManagement/CancellationConfirmDialog';
import QuickBookingFAB from './components/QuickBookingFAB';
import StatisticsView from './components/Statistics/StatisticsView';
import UndoRedoButtons from './components/UndoRedoButtons';
import CleaningSync from './components/CleaningSync';
import ErrorBoundary from './components/ErrorBoundary';
import ReminderDropdown from './components/Reminders/ReminderDropdown';
import RemindersView from './components/Reminders/RemindersView';
import DevTools from './components/DevTools/ComprehensiveDevTools';
import { Calendar, Hotel, UserPlus, LayoutDashboard, CalendarCheck, Users, Settings, Mail, Briefcase, TrendingUp, Cloud, Bell } from 'lucide-react';
import loadingAnimation from './loading-animation.json';
import appIcon from './assets/app-icon.png';
import { formatDateShort } from './utils/dateFormatting';
import { runMigrationsIfNeeded } from './auto_migrate';

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
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

type Tab = 'dashboard' | 'bookings' | 'guests' | 'rooms' | 'emails' | 'templates' | 'statistics' | 'cleaning' | 'reminders';

interface DbChangeEvent {
  table: string;
  action: 'INSERT' | 'UPDATE' | 'DELETE';
  id: number;
  timestamp: string;
}

function AppContent() {
  // NORMALIZED STATE: Get guestMap for O(1) lookups
  const { rooms, bookings, loading, refreshAll, updateBookingStatus, guestMap } = useData(); // Use Context directly!
  const [error, setError] = useState<string | null>(null);
  const [showGuestDialog, setShowGuestDialog] = useState(false);
  const [showSettingsDialog, setShowSettingsDialog] = useState(false);
  const [showEmailDialog, setShowEmailDialog] = useState(false);
  const [emailBookingId, setEmailBookingId] = useState<number | undefined>(undefined);
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');
  const [showCancellationConfirm, setShowCancellationConfirm] = useState(false);
  const [bookingToCancel, setBookingToCancel] = useState<{ id: number; reservierungsnummer: string } | undefined>(undefined);

  // Booking Sidebar State
  const [showBookingSidebar, setShowBookingSidebar] = useState(false);
  const [sidebarBookingId, setSidebarBookingId] = useState<number | null>(null);
  const [sidebarMode, setSidebarMode] = useState<'view' | 'edit' | 'create'>('view');
  const [sidebarPrefillData, setSidebarPrefillData] = useState<{ roomId?: number; checkinDate?: string; checkoutDate?: string } | undefined>(undefined);

  // Reminder System State
  const [urgentReminderCount, setUrgentReminderCount] = useState<number>(0);
  const [showReminderDropdown, setShowReminderDropdown] = useState(false);

  useEffect(() => {
    // Run migrations on app startup (idempotent - safe to run multiple times)
    runMigrationsIfNeeded().catch(err => console.error('Migration check failed:', err));

    updateBookingStatuses(); // Status-Update bei App-Start
  }, []);

  // Auto-Update der Buchungs-Status alle 10 Minuten
  useEffect(() => {
    const interval = setInterval(() => {
      updateBookingStatuses();
    }, 10 * 60 * 1000); // 10 Minuten

    return () => clearInterval(interval);
  }, []);

  // Lade urgent reminder count beim Mount (initial load)
  useEffect(() => {
    loadUrgentReminderCount();
  }, []);

  // Real-Time Listener für Reminder-Änderungen (PostgreSQL NOTIFY + reminder-completed Event)
  useEffect(() => {
    console.log('🔧 [DEBUG App] Setting up reminder listeners...');
    let dbChangeUnlisten: UnlistenFn | null = null;
    let reminderCompletedUnlisten: UnlistenFn | null = null;

    const setupListeners = async () => {
      try {
        console.log('🔧 [DEBUG App] Importing @tauri-apps/api/event...');
        const { listen } = await import('@tauri-apps/api/event');
        console.log('🔧 [DEBUG App] listen() imported successfully');

        // Listener 1: PostgreSQL NOTIFY (db-change)
        dbChangeUnlisten = await listen<DbChangeEvent>('db-change', (event) => {
          console.log('🔧 [DEBUG App] db-change event received!', event.payload);
          const { table, action, id } = event.payload;

          // Log ALL events for debugging
          console.log(`🔧 [DEBUG App] Event details: table="${table}", action="${action}", id=${id}`);

          // Nur bei Reminder-Änderungen Badge-Count neu laden
          if (table === 'reminders') {
            console.log(`📡 [Real-Time] ${action} Reminder #${id} → Badge wird aktualisiert`);
            console.log('🔧 [DEBUG App] Calling loadUrgentReminderCount()...');
            loadUrgentReminderCount();
          } else {
            console.log(`🔧 [DEBUG App] Ignoring event for table: ${table}`);
          }
        });
        console.log('✅ [App] Reminder NOTIFY listener (db-change) registriert');

        // Listener 2: Backend Event (reminder-completed) - PHASE 1 FIX
        reminderCompletedUnlisten = await listen('reminder-completed', (event: any) => {
          console.log('✅ [App] reminder-completed Event empfangen:', event.payload);
          loadUrgentReminderCount(); // Sofort Badge neu laden
        });
        console.log('✅ [App] Reminder Event listener (reminder-completed) registriert');
      } catch (e) {
        console.error('❌ [App] Fehler beim Registrieren der Reminder-Listeners:', e);
      }
    };

    setupListeners();

    return () => {
      if (dbChangeUnlisten) dbChangeUnlisten();
      if (reminderCompletedUnlisten) reminderCompletedUnlisten();
    };
  }, []);

  // Auto-Update Check beim App-Start
  useEffect(() => {
    async function checkForUpdates() {
      try {
        console.log('🔍 Checking for updates...');
        const update = await check();

        if (update?.available) {
          console.log(`✅ Update available: ${update.version}`);

          // Zeige Confirmation Dialog
          const shouldUpdate = window.confirm(
            `Update verfügbar: Version ${update.version}\n\n${update.body || 'Neue Version verfügbar'}\n\nJetzt installieren? Die App wird nach der Installation neu gestartet.`
          );

          if (shouldUpdate) {
            toast.loading('Update wird heruntergeladen...', { id: 'update-download' });

            // Download und installiere Update
            let downloaded = 0;
            let contentLength = 0;
            await update.downloadAndInstall((event) => {
              switch (event.event) {
                case 'Started':
                  contentLength = event.data.contentLength || 0;
                  console.log(`Download started (${Math.round(contentLength / 1024 / 1024)} MB)`);
                  break;
                case 'Progress':
                  downloaded += event.data.chunkLength;
                  const percent = contentLength > 0 ? Math.round((downloaded / contentLength) * 100) : 0;
                  toast.loading(`Update wird heruntergeladen... ${percent}%`, { id: 'update-download' });
                  break;
                case 'Finished':
                  toast.success('Update erfolgreich heruntergeladen!', { id: 'update-download' });
                  console.log('Download finished');
                  break;
              }
            });

            console.log('✅ Update installed, restarting app...');
            toast.success('Update installiert! App wird neu gestartet...', { duration: 2000 });

            // App neu starten (nach 2 Sekunden)
            setTimeout(() => {
              relaunch();
            }, 2000);
          }
        } else {
          console.log('✅ App ist auf dem neuesten Stand');
        }
      } catch (error) {
        console.error('❌ Fehler beim Update-Check:', error);
        // Fehler wird nicht dem User angezeigt (nur console)
      }
    }

    // Nur im Production Build nach Updates suchen
    if (import.meta.env.PROD) {
      checkForUpdates();
    }
  }, []);

  // ✅ Event Listener für Toast-Notifications (von Background-Prozessen)
  useEffect(() => {
    const handleShowToast = (event: CustomEvent) => {
      const { id, message, type, duration } = event.detail;

      // ✅ LOADING TOAST - bleibt offen bis update (mit ID)
      if (type === 'loading') {
        toast.loading(message, { id });
        return;
      }

      // ✅ UPDATE EXISTING TOAST (wenn ID vorhanden)
      if (id) {
        switch (type) {
          case 'success':
            toast.success(message, { id, duration: duration || 4000 });
            break;
          case 'error':
            toast.error(message, { id, duration: duration || 5000 });
            break;
          case 'info':
            toast(message, { id, duration: duration || 3000, icon: '📧' });
            break;
          case 'warning':
            toast(message, { id, duration: duration || 4000, icon: '⚠️' });
            break;
          default:
            toast(message, { id, duration: duration || 3000 });
        }
        return;
      }

      // ✅ NEW TOAST (ohne ID - alte Kompatibilität)
      switch (type) {
        case 'success':
          toast.success(message, { duration: duration || 4000 });
          break;
        case 'error':
          toast.error(message, { duration: duration || 5000 });
          break;
        case 'info':
          toast(message, { duration: duration || 3000, icon: '📧' });
          break;
        case 'warning':
          toast(message, { duration: duration || 4000, icon: '⚠️' });
          break;
        default:
          toast(message, { duration: duration || 3000 });
      }
    };

    window.addEventListener('show-toast', handleShowToast as EventListener);
    return () => window.removeEventListener('show-toast', handleShowToast as EventListener);
  }, []);

  const loadUrgentReminderCount = async () => {
    console.log('🔧 [DEBUG App] loadUrgentReminderCount() called');
    try {
      console.log('🔧 [DEBUG App] Invoking get_active_reminders_pg...');
      const reminders = await invoke<any[]>('get_active_reminders_pg');
      console.log('🔧 [DEBUG App] Received reminders:', reminders.length, 'reminders');
      console.log('🔧 [DEBUG App] Old badge count:', urgentReminderCount, '→ New:', reminders.length);
      setUrgentReminderCount(reminders.length);
      console.log('✅ [DEBUG App] Badge count updated to:', reminders.length);
    } catch (error) {
      console.error('❌ [DEBUG App] Fehler beim Laden der dringenden Erinnerungen:', error);
    }
  };

  const updateBookingStatuses = async () => {
    try {
      console.log('🔄 Updating booking statuses...');
      const changedCount = await invoke<number>('update_booking_statuses_pg');
      if (changedCount > 0) {
        console.log(`✅ ${changedCount} Buchungs-Status aktualisiert`);
        // Daten neu laden wenn sich etwas geändert hat
        await refreshAll();
      }
    } catch (error) {
      console.error('❌ Fehler beim Status-Update:', error);
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
      setBookingToCancel(undefined);
    } catch (error) {
      console.error('Fehler beim Stornieren:', error);
      toast.error('Fehler beim Stornieren: ' + error);
    }
  };

  const cancelCancellation = () => {
    setShowCancellationConfirm(false);
    setBookingToCancel(undefined);
  };

  if (loading) {
    return (
      <div className="fixed inset-0 bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 flex flex-col items-center justify-center">
        {/* App Logo - Large */}
        <div className="mb-8">
          <img
            src={appIcon}
            alt="DPolG Buchungssystem Logo"
            className="w-48 h-48 rounded-3xl shadow-[0_20px_60px_-15px_rgba(0,0,0,0.6)]"
          />
        </div>

        {/* Lottie Animation */}
        <div className="w-80 h-80 mb-8">
          <Lottie
            animationData={loadingAnimation}
            loop={true}
            autoplay={true}
          />
        </div>

        {/* Company Branding */}
        <div className="text-center space-y-4">
          <div className="space-y-2">
            <h1 className="text-4xl font-bold text-white tracking-tight">
              Maxflow Software
            </h1>
            <div className="h-1 w-32 bg-gradient-to-r from-blue-500 to-blue-600 rounded-full mx-auto"></div>
          </div>
          <p className="text-xl text-slate-300 font-medium">
            Stiftung der DPolG Buchungssystem
          </p>
          <p className="text-sm text-slate-500 mt-8">
            Lädt...
          </p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-screen bg-gradient-to-br from-slate-800 to-slate-900">
        <div className="text-center max-w-md">
          <div className="text-red-400 mb-4 text-2xl font-bold">⚠️ Fehler beim Laden</div>
          <p className="text-sm text-slate-300 mb-6">{error}</p>
          <button
            onClick={refreshAll}
            className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors shadow-lg"
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
    { id: 'reminders' as Tab, label: 'Erinnerungen', icon: Bell },
    { id: 'templates' as Tab, label: 'Services & Rabatte', icon: Briefcase },
    { id: 'emails' as Tab, label: 'Email-Verlauf', icon: Mail },
    { id: 'statistics' as Tab, label: 'Statistiken', icon: TrendingUp },
    { id: 'cleaning' as Tab, label: 'Putzplan Sync', icon: Cloud },
  ];

  return (
      <div className="flex flex-col h-screen bg-gradient-to-br from-slate-50 to-slate-100">
      {/* Header - Compact Single Row */}
      <header className="bg-gradient-to-r from-slate-800 to-slate-900 shadow-2xl border-b border-slate-700">
        <div className="px-6 py-3 flex items-center justify-between">
          {/* Left: Logo + Title + Stats */}
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-3">
              <img
                src={appIcon}
                alt="DPolG Buchungssystem Logo"
                className="w-14 h-14 rounded-xl shadow-lg"
              />
              <div>
                <h1 className="text-lg font-bold text-white tracking-tight">
                  Stiftung der DPolG Buchungssystem
                </h1>
              </div>
            </div>

            {/* Stats Inline - Clickable */}
            <div className="flex items-center gap-4 border-l border-slate-600 pl-4">
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
                  <span className="text-xs text-slate-400">Gesamt</span>
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
                  <span className="text-lg font-bold text-white">{(() => {
                    const today = new Date().toISOString().split('T')[0];
                    const occupiedRooms = new Set(
                      bookings
                        .filter(b =>
                          b.status !== 'storniert' &&
                          b.checkin_date <= today &&
                          b.checkout_date > today
                        )
                        .map(b => b.room_id)
                    ).size;
                    return Math.round((occupiedRooms / rooms.length) * 100);
                  })()}%</span>
                  <span className="text-xs text-slate-400">Heute</span>
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

            {/* Reminder Bell Icon mit Badge + Dropdown */}
            <div className="relative">
              <button
                onClick={() => setShowReminderDropdown(!showReminderDropdown)}
                className="relative flex items-center gap-2 p-2.5 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
                title="Erinnerungen"
              >
                <Bell className="w-5 h-5" />
                {urgentReminderCount > 0 && (
                  <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center font-bold shadow-lg">
                    {urgentReminderCount > 9 ? '9+' : urgentReminderCount}
                  </span>
                )}
              </button>

              {/* Reminder Dropdown */}
              <ReminderDropdown
                isOpen={showReminderDropdown}
                onClose={() => setShowReminderDropdown(false)}
                onReminderClick={(bookingId) => {
                  if (bookingId) {
                    // Öffne Sidebar in Edit Mode
                    setSidebarBookingId(bookingId);
                    setSidebarMode('edit');
                    setSidebarPrefillData(undefined);
                    setShowBookingSidebar(true);
                    setShowReminderDropdown(false);
                  }
                }}
                onViewAll={() => {
                  setShowReminderDropdown(false);
                  setActiveTab('reminders');
                }}
              />
            </div>

            <button
              onClick={() => setShowGuestDialog(true)}
              className="flex items-center gap-2 btn-primary bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white font-semibold rounded-lg transition-all shadow-lg hover:shadow-xl"
            >
              <UserPlus className="w-4 h-4" />
              Neuer Gast
            </button>

            <div className="flex items-center gap-2 bg-slate-700/50 date-display py-2 rounded-lg backdrop-blur-sm">
              <Calendar className="w-4 h-4 text-blue-400" />
              <span className="text-xs font-semibold text-white">{formatDateShort(new Date())}</span>
            </div>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="px-6 pt-3">
          <div className="flex items-center justify-between gap-2">
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

            {/* Undo/Redo Buttons - Ganz rechts in Tab-Zeile */}
            <UndoRedoButtons />
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 overflow-hidden">
        {activeTab === 'dashboard' && (
          <ErrorBoundary>
            <TapeChart
              onBookingClick={(bookingId) => {
                // Öffne Sidebar in View Mode
                setSidebarBookingId(bookingId);
                setSidebarMode('view');
                setSidebarPrefillData(undefined);
                setShowBookingSidebar(true);
              }}
              onCreateBooking={(roomId, startDate, endDate) => {
                console.log('🎨 Create Booking:', { roomId, startDate, endDate });
                // Öffne Sidebar in Create Mode
                setSidebarBookingId(null);
                setSidebarMode('create');
                setSidebarPrefillData({ roomId, checkinDate: startDate, checkoutDate: endDate });
                setShowBookingSidebar(true);
              }}
              onBookingEdit={(bookingId) => {
                // Öffne Sidebar in Edit Mode
                setSidebarBookingId(bookingId);
                setSidebarMode('edit');
                setSidebarPrefillData(undefined);
                setShowBookingSidebar(true);
              }}
              onBookingCancel={(bookingId) => {
                const booking = bookings.find(b => b.id === bookingId);
                if (!booking) return;

                // Zeige CancellationConfirmDialog
                setBookingToCancel({ id: booking.id, reservierungsnummer: booking.reservierungsnummer });
                setShowCancellationConfirm(true);
              }}
              onSendEmail={(bookingId) => {
                setEmailBookingId(bookingId);
                setShowEmailDialog(true);
              }}
            />
          </ErrorBoundary>
        )}
        {activeTab === 'statistics' && <StatisticsView />}
        {activeTab === 'bookings' && <BookingList />}
        {activeTab === 'guests' && <GuestList />}
        {activeTab === 'rooms' && <RoomList />}
        {activeTab === 'reminders' && (
          <RemindersView
            onNavigateToBooking={(bookingId) => {
              setActiveTab('dashboard');
              // Öffne Sidebar in Edit Mode
              setSidebarBookingId(bookingId);
              setSidebarMode('edit');
              setSidebarPrefillData(undefined);
              setShowBookingSidebar(true);
            }}
          />
        )}
        {activeTab === 'templates' && <TemplatesManagement />}
        {activeTab === 'emails' && <EmailHistoryView />}
        {activeTab === 'cleaning' && <CleaningSync />}
      </main>

      {/* Floating Action Button - Always Visible - Opens Sidebar */}
      <QuickBookingFAB onClick={() => {
        setSidebarBookingId(null);
        setSidebarMode('create');
        setSidebarPrefillData(undefined);
        setShowBookingSidebar(true);
      }} />

      {/* Dialogs */}
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

      {/* Email Selection Dialog */}
      {emailBookingId && (() => {
        const booking = bookings.find(b => b.id === emailBookingId);
        // NORMALIZED STATE: Look up guest from Map
        const guest = booking ? guestMap.get(booking.guest_id) : undefined;
        return booking ? (
          <EmailSelectionDialog
            bookingId={emailBookingId}
            guestEmail={guest?.email || ''}
            isOpen={showEmailDialog}
            onClose={() => {
              setShowEmailDialog(false);
              setEmailBookingId(undefined);
            }}
          />
        ) : null;
      })()}

      {/* Cancellation Confirm Dialog */}
      {bookingToCancel && (
        <CancellationConfirmDialog
          isOpen={showCancellationConfirm}
          reservierungsnummer={bookingToCancel.reservierungsnummer}
          onConfirm={confirmCancellation}
          onCancel={cancelCancellation}
        />
      )}

      {/* Booking Sidebar - Unified View/Edit/Create */}
      <BookingSidebar
        bookingId={sidebarBookingId}
        isOpen={showBookingSidebar}
        onClose={() => {
          setShowBookingSidebar(false);
          setSidebarBookingId(null);
          setSidebarMode('view');
          setSidebarPrefillData(undefined);
        }}
        mode={sidebarMode}
        prefillData={sidebarPrefillData}
      />
      </div>
  );
}

// Wrapper component with DataProvider
function App() {
  // DevTools visibility from localStorage (default: hidden)
  const [showDevTools, setShowDevTools] = useState(() => {
    const saved = localStorage.getItem('devtools-enabled');
    return saved === 'true';
  });

  // Listen for settings changes
  useEffect(() => {
    const handleDevToolsToggle = (e: CustomEvent<{ enabled: boolean }>) => {
      setShowDevTools(e.detail.enabled);
    };
    window.addEventListener('devtools-toggle' as any, handleDevToolsToggle);
    return () => window.removeEventListener('devtools-toggle' as any, handleDevToolsToggle);
  }, []);

  return (
    <UserProvider>
      <OnlineProvider>
        <DataProvider>
          <OfflineBanner />
          <AppContent />
          <Toaster
          position="bottom-right"
          toastOptions={{
            duration: 4000,
            style: {
              background: '#1e293b',
              color: '#fff',
              borderRadius: '0.75rem',
              padding: '1rem',
              boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
            },
            loading: {
              duration: Infinity, // Loading-Toast bleibt sichtbar bis explizit ersetzt
              style: {
                background: '#1e293b',
                color: '#fff',
                borderRadius: '0.75rem',
                padding: '1rem',
                boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
              },
            },
            success: {
              duration: 4000,
              iconTheme: {
                primary: '#10b981',
                secondary: '#fff',
              },
            },
            error: {
              duration: 5000, // Errors länger sichtbar
              iconTheme: {
                primary: '#ef4444',
                secondary: '#fff',
              },
            },
          }}
        />

        {/* DevTools with Pool Stats Monitoring - nur wenn aktiviert */}
        {showDevTools && <DevTools />}
        </DataProvider>
      </OnlineProvider>
    </UserProvider>
  );
}

export default App;