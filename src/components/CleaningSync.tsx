import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { RefreshCw, Cloud, Calendar, Smartphone, BarChart3, ExternalLink, Monitor, Copy, CheckCircle, Lock, Trash2, FileDown, FolderOpen } from 'lucide-react';

interface CleaningStats {
  today: number;
  tomorrow: number;
  this_week: number;  // Backend sendet snake_case
  total: number;
}

export default function CleaningSync() {
  const [syncing, setSyncing] = useState(false);
  const [cleaning, setCleaning] = useState(false);
  const [message, setMessage] = useState('');
  const [cleanupMessage, setCleanupMessage] = useState('');
  const [lastSync, setLastSync] = useState<Date | null>(null);
  const [stats, setStats] = useState<CleaningStats>({ today: 0, tomorrow: 0, this_week: 0, total: 0 });
  const [previewMode, setPreviewMode] = useState<'mobile' | 'desktop'>('mobile');
  const [passwordCopied, setPasswordCopied] = useState(false);

  // PDF Export State
  const [exporting, setExporting] = useState(false);
  const [exportMessage, setExportMessage] = useState('');
  const [selectedYear, setSelectedYear] = useState(new Date().getFullYear());
  const [selectedMonth, setSelectedMonth] = useState(new Date().getMonth() + 1); // 1-12

  // Hardcoded password (matches mobile app)
  const MOBILE_APP_PASSWORD = 'putzplan2025';

  // Lade Stats aus Turso
  useEffect(() => {
    loadStats();
  }, []);

  const loadStats = async () => {
    console.log('🔍 [CleaningSync] loadStats() aufgerufen');
    try {
      console.log('📡 [CleaningSync] Rufe get_cleaning_stats Command auf...');
      const stats = await invoke<CleaningStats>('get_cleaning_stats');
      console.log('✅ [CleaningSync] Stats erhalten:', stats);
      setStats(stats);
    } catch (error) {
      console.error('❌ [CleaningSync] Fehler beim Laden der Stats:', error);
      // Fallback zu 0, nicht zu Dummy-Daten
      setStats({
        today: 0,
        tomorrow: 0,
        this_week: 0,
        total: 0
      });
    }
  };

  const handleSync3Months = async () => {
    setSyncing(true);
    setMessage('');
    try {
      const result = await invoke<string>('sync_week_ahead');
      setMessage(result);
      setLastSync(new Date());
      // Stats neu laden nach Sync
      await loadStats();
    } catch (error) {
      setMessage(`❌ Fehler: ${error}`);
    } finally {
      setSyncing(false);
    }
  };

  const handleCleanup = async () => {
    if (!confirm('🧹 PUTZPLAN BEREINIGEN\n\nDies löscht ALLE Tasks aus der Cloud und synchronisiert sie neu. \n\nAlle gelöschten Buchungen werden aus dem Putzplan entfernt.\n\nDauer: ca. 30-60 Sekunden\n\nFortfahren?')) {
      return;
    }

    setCleaning(true);
    setCleanupMessage('');
    try {
      const result = await invoke<string>('cleanup_cleaning_tasks');
      setCleanupMessage(result);
      setLastSync(new Date());
      // Stats neu laden nach Cleanup
      await loadStats();
    } catch (error) {
      setCleanupMessage(`❌ Fehler: ${error}`);
    } finally {
      setCleaning(false);
    }
  };

  const openMobileApp = () => {
    window.open('https://dpolg-cleaning-mobile.vercel.app', '_blank');
  };

  const handleCopyPassword = () => {
    navigator.clipboard.writeText(MOBILE_APP_PASSWORD);
    setPasswordCopied(true);
    setTimeout(() => setPasswordCopied(false), 2000);
  };

  const handleExportPDF = async () => {
    setExporting(true);
    setExportMessage('');
    try {
      const pdfPath = await invoke<string>('export_cleaning_timeline_pdf', {
        year: selectedYear,
        month: selectedMonth,
      });
      setExportMessage(`✅ PDF erstellt und geöffnet!`);
    } catch (error) {
      setExportMessage(`❌ Fehler: ${error}`);
    } finally {
      setExporting(false);
    }
  };

  const handleOpenPutzplanFolder = async () => {
    try {
      const folderPath = await invoke<string>('open_putzplan_folder');
      console.log('Putzplan-Ordner geöffnet:', folderPath);
    } catch (error) {
      console.error('Fehler beim Öffnen des Putzplan-Ordners:', error);
    }
  };

  return (
    <div className="h-full overflow-y-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-3 bg-blue-100 rounded-xl">
            <Cloud className="w-6 h-6 text-blue-600" />
          </div>
          <div>
            <h2 className="text-2xl font-bold text-slate-900">Putzplan Sync</h2>
            <p className="text-sm text-slate-600">Cloud-Synchronisation für Mobile-App</p>
          </div>
        </div>

        {lastSync && (
          <div className="text-right">
            <p className="text-xs text-slate-500">Zuletzt synchronisiert</p>
            <p className="text-sm font-semibold text-slate-700">
              {lastSync.toLocaleString('de-DE', {
                day: '2-digit',
                month: '2-digit',
                year: 'numeric',
                hour: '2-digit',
                minute: '2-digit'
              })}
            </p>
          </div>
        )}
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-4 gap-4">
        <div className="p-4 bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl shadow-lg text-white">
          <div className="flex items-center justify-between mb-2">
            <Calendar className="w-5 h-5 opacity-80" />
            <span className="text-3xl font-bold">{stats.today}</span>
          </div>
          <p className="text-sm opacity-90">Heute</p>
        </div>

        <div className="p-4 bg-gradient-to-br from-emerald-500 to-emerald-600 rounded-xl shadow-lg text-white">
          <div className="flex items-center justify-between mb-2">
            <Calendar className="w-5 h-5 opacity-80" />
            <span className="text-3xl font-bold">{stats.tomorrow}</span>
          </div>
          <p className="text-sm opacity-90">Morgen</p>
        </div>

        <div className="p-4 bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl shadow-lg text-white">
          <div className="flex items-center justify-between mb-2">
            <BarChart3 className="w-5 h-5 opacity-80" />
            <span className="text-3xl font-bold">{stats.this_week}</span>
          </div>
          <p className="text-sm opacity-90">Diese Woche</p>
        </div>

        <div className="p-4 bg-gradient-to-br from-slate-600 to-slate-700 rounded-xl shadow-lg text-white">
          <div className="flex items-center justify-between mb-2">
            <Cloud className="w-5 h-5 opacity-80" />
            <span className="text-3xl font-bold">{stats.total}</span>
          </div>
          <p className="text-sm opacity-90">3 Monate</p>
        </div>
      </div>

      {/* Sync Button */}
      <div className="bg-white rounded-xl shadow-lg p-6 border border-slate-200">
        <h3 className="text-lg font-semibold text-slate-900 mb-4">Cloud-Synchronisation</h3>

        <div className="space-y-3">
          <button
            onClick={handleSync3Months}
            disabled={syncing || cleaning || exporting}
            className="w-full flex items-center justify-center gap-3 px-6 py-4 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 disabled:from-slate-400 disabled:to-slate-500 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-xl transform hover:scale-[1.02] active:scale-[0.98]"
          >
            <RefreshCw className={`w-5 h-5 ${syncing ? 'animate-spin' : ''}`} />
            <span>{syncing ? 'Synchronisiere...' : '3 Monate synchronisieren'}</span>
            <span className="text-sm opacity-90">(90 Tage)</span>
          </button>

          <button
            onClick={handleCleanup}
            disabled={syncing || cleaning || exporting}
            className="w-full flex items-center justify-center gap-3 px-6 py-4 bg-gradient-to-r from-amber-600 to-amber-700 hover:from-amber-700 hover:to-amber-800 disabled:from-slate-400 disabled:to-slate-500 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-xl transform hover:scale-[1.02] active:scale-[0.98]"
          >
            <Trash2 className={`w-5 h-5 ${cleaning ? 'animate-pulse' : ''}`} />
            <span>{cleaning ? 'Bereinige...' : 'Putzplan bereinigen'}</span>
            <span className="text-sm opacity-90">(Löscht alte Buchungen)</span>
          </button>
        </div>

        {message && (
          <div className={`mt-4 p-4 rounded-lg text-sm whitespace-pre-line ${
            message.includes('❌')
              ? 'bg-red-50 text-red-700 border border-red-200'
              : 'bg-green-50 text-green-700 border border-green-200'
          }`}>
            {message}
          </div>
        )}

        {cleanupMessage && (
          <div className={`mt-4 p-4 rounded-lg text-sm whitespace-pre-line ${
            cleanupMessage.includes('❌')
              ? 'bg-red-50 text-red-700 border border-red-200'
              : 'bg-green-50 text-green-700 border border-green-200'
          }`}>
            {cleanupMessage}
          </div>
        )}
      </div>

      {/* PDF Export Section */}
      <div className="bg-white rounded-xl shadow-lg p-6 border border-slate-200">
        <h3 className="text-lg font-semibold text-slate-900 mb-4">Timeline PDF Export</h3>

        <div className="space-y-4">
          {/* Monat & Jahr Auswahl */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-semibold text-slate-700 mb-2">Jahr</label>
              <select
                value={selectedYear}
                onChange={(e) => setSelectedYear(parseInt(e.target.value))}
                className="w-full px-4 py-3 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
              >
                {[...Array(3)].map((_, i) => {
                  const year = new Date().getFullYear() + i;
                  return <option key={year} value={year}>{year}</option>;
                })}
              </select>
            </div>

            <div>
              <label className="block text-sm font-semibold text-slate-700 mb-2">Monat</label>
              <select
                value={selectedMonth}
                onChange={(e) => setSelectedMonth(parseInt(e.target.value))}
                className="w-full px-4 py-3 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
              >
                {['Januar', 'Februar', 'März', 'April', 'Mai', 'Juni', 'Juli', 'August', 'September', 'Oktober', 'November', 'Dezember'].map((name, i) => (
                  <option key={i + 1} value={i + 1}>{name}</option>
                ))}
              </select>
            </div>
          </div>

          {/* Export Button */}
          <div className="grid grid-cols-2 gap-3">
            <button
              onClick={handleExportPDF}
              disabled={syncing || cleaning || exporting}
              className="flex items-center justify-center gap-3 px-6 py-4 bg-gradient-to-r from-emerald-600 to-emerald-700 hover:from-emerald-700 hover:to-emerald-800 disabled:from-slate-400 disabled:to-slate-500 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-xl transform hover:scale-[1.02] active:scale-[0.98]"
            >
              <FileDown className={`w-5 h-5 ${exporting ? 'animate-bounce' : ''}`} />
              <span>{exporting ? 'Erstelle PDF...' : 'PDF exportieren'}</span>
            </button>

            <button
              onClick={handleOpenPutzplanFolder}
              disabled={syncing || cleaning || exporting}
              className="flex items-center justify-center gap-3 px-6 py-4 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 disabled:from-slate-400 disabled:to-slate-500 text-white font-semibold rounded-xl transition-all shadow-lg hover:shadow-xl transform hover:scale-[1.02] active:scale-[0.98]"
            >
              <FolderOpen className="w-5 h-5" />
              <span>Ordner öffnen</span>
            </button>
          </div>

          {exportMessage && (
            <div className={`p-4 rounded-lg text-sm ${
              exportMessage.includes('❌')
                ? 'bg-red-50 text-red-700 border border-red-200'
                : 'bg-green-50 text-green-700 border border-green-200'
            }`}>
              {exportMessage}
            </div>
          )}

          <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
            <p className="text-sm text-blue-800">
              📁 PDFs werden im Programmordner gespeichert: <code className="px-2 py-1 bg-white rounded text-xs">putzplan/</code>
            </p>
          </div>
        </div>
      </div>

      {/* Preview Section */}
      <div className="bg-white rounded-xl shadow-lg border border-slate-200 overflow-hidden">
        <div className="p-6 border-b border-slate-200 bg-gradient-to-r from-slate-50 to-white">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              {previewMode === 'mobile' ? (
                <Smartphone className="w-5 h-5 text-slate-700" />
              ) : (
                <Monitor className="w-5 h-5 text-slate-700" />
              )}
              <h3 className="text-lg font-semibold text-slate-900">
                {previewMode === 'mobile' ? 'Mobile App Vorschau' : 'Desktop Vorschau'}
              </h3>
            </div>
            <div className="flex items-center gap-3">
              {/* Toggle Buttons */}
              <div className="flex bg-slate-100 rounded-lg p-1">
                <button
                  onClick={() => setPreviewMode('mobile')}
                  className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
                    previewMode === 'mobile'
                      ? 'bg-white text-blue-600 shadow-sm'
                      : 'text-slate-600 hover:text-slate-900'
                  }`}
                >
                  <Smartphone className="w-4 h-4" />
                  Mobile
                </button>
                <button
                  onClick={() => setPreviewMode('desktop')}
                  className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-all ${
                    previewMode === 'desktop'
                      ? 'bg-white text-blue-600 shadow-sm'
                      : 'text-slate-600 hover:text-slate-900'
                  }`}
                >
                  <Monitor className="w-4 h-4" />
                  Desktop
                </button>
              </div>
              <button
                onClick={openMobileApp}
                className="flex items-center gap-2 px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors"
              >
                <ExternalLink className="w-4 h-4" />
                Öffnen
              </button>
            </div>
          </div>
          <p className="text-sm text-slate-600 mt-1">
            {previewMode === 'mobile'
              ? 'So sehen die Putzkräfte den Putzplan auf ihrem Smartphone'
              : 'So sieht der Putzplan auf einem Desktop-Browser aus'}
          </p>
        </div>

        {/* Preview Content - conditional based on previewMode */}
        {previewMode === 'mobile' ? (
          // Mobile Phone Mockup
          <div className="p-8 bg-gradient-to-br from-slate-100 to-slate-50">
            <div className="max-w-sm mx-auto bg-black rounded-[2.5rem] shadow-2xl p-3">
              {/* Phone Screen */}
              <div className="bg-slate-900 rounded-[2rem] overflow-hidden shadow-inner">
                {/* Notch */}
                <div className="h-7 bg-black rounded-b-3xl mx-auto w-40"></div>

                {/* Screen Content */}
                <div className="h-[600px] overflow-y-auto bg-slate-900">
                  <iframe
                    src="https://dpolg-cleaning-mobile.vercel.app"
                    className="w-full h-full border-0"
                    title="Mobile App Preview"
                  />
                </div>

                {/* Bottom Bar */}
                <div className="h-5 bg-black"></div>
              </div>
            </div>
          </div>
        ) : (
          // Desktop Browser Mockup
          <div className="p-8 bg-gradient-to-br from-slate-100 to-slate-50">
            <div className="max-w-7xl mx-auto">
              {/* Browser Chrome */}
              <div className="bg-slate-200 rounded-t-xl p-3 flex items-center gap-2 border-b border-slate-300">
                <div className="flex gap-2">
                  <div className="w-3 h-3 rounded-full bg-red-500"></div>
                  <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                  <div className="w-3 h-3 rounded-full bg-green-500"></div>
                </div>
                <div className="flex-1 ml-4 bg-white rounded px-3 py-1 text-xs text-slate-600 flex items-center gap-2">
                  <span className="text-slate-400">🔒</span>
                  <span>dpolg-cleaning-mobile.vercel.app</span>
                </div>
              </div>

              {/* Browser Content */}
              <div className="bg-white shadow-2xl rounded-b-xl overflow-hidden">
                <iframe
                  src="https://dpolg-cleaning-mobile.vercel.app"
                  className="w-full h-[700px] border-0"
                  title="Desktop App Preview"
                />
              </div>
            </div>
          </div>
        )}

        {/* QR Code & Info */}
        <div className="p-6 bg-slate-50 border-t border-slate-200">
          <div className="flex items-start gap-4">
            <div className="flex-1">
              <h4 className="font-semibold text-slate-900 mb-2">📱 Zugriff für Putzkräfte</h4>
              <p className="text-sm text-slate-700 mb-3">
                Die Putzkräfte können die App direkt im Browser öffnen - keine Installation nötig!
              </p>
              <div className="flex flex-col gap-2">
                <div className="flex items-center gap-2 text-sm text-slate-600">
                  <div className="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                  <span>URL: <code className="px-2 py-1 bg-white rounded border border-slate-200 text-xs">dpolg-cleaning-mobile.vercel.app</code></span>
                </div>

                {/* Password Display Section */}
                <div className="mt-2 p-4 bg-gradient-to-br from-blue-50 to-blue-100 rounded-lg border border-blue-200">
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <Lock className="w-4 h-4 text-blue-600" />
                        <span className="text-sm font-semibold text-blue-900">Passwort für Mobile App:</span>
                      </div>
                      <div className="flex items-center gap-3">
                        <code className="px-4 py-2 bg-white rounded-lg border-2 border-blue-300 text-blue-900 font-bold text-lg tracking-wider">
                          {MOBILE_APP_PASSWORD}
                        </code>
                        <button
                          onClick={handleCopyPassword}
                          className="flex items-center gap-2 px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors text-sm font-medium"
                        >
                          {passwordCopied ? (
                            <>
                              <CheckCircle className="w-4 h-4" />
                              Kopiert!
                            </>
                          ) : (
                            <>
                              <Copy className="w-4 h-4" />
                              Kopieren
                            </>
                          )}
                        </button>
                      </div>
                      <p className="text-xs text-blue-700 mt-2">
                        🔒 Passwort bleibt während der Browser-Session aktiv
                      </p>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-2 text-sm text-slate-600">
                  <div className="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                  <span>Automatische Updates alle 5 Minuten</span>
                </div>
                <div className="flex items-center gap-2 text-sm text-slate-600">
                  <div className="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                  <span>Funktioniert auf allen Smartphones</span>
                </div>
              </div>
            </div>

            {/* QR Code Placeholder */}
            <div className="p-4 bg-white rounded-lg border-2 border-slate-300 shadow-sm">
              <div className="w-32 h-32 bg-slate-100 rounded flex items-center justify-center text-xs text-slate-500 text-center">
                QR Code<br/>für Mobile-App
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
