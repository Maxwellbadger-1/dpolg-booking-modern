import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { RefreshCw, Cloud, Calendar } from 'lucide-react';

export default function CleaningSync() {
  const [syncing, setSyncing] = useState(false);
  const [message, setMessage] = useState('');

  const handleSyncToday = async () => {
    setSyncing(true);
    setMessage('');
    try {
      const today = new Date().toISOString().split('T')[0];
      const result = await invoke<string>('sync_cleaning_tasks', { date: today });
      setMessage(result);
    } catch (error) {
      setMessage(`❌ Fehler: ${error}`);
    } finally {
      setSyncing(false);
    }
  };

  const handleSyncWeek = async () => {
    setSyncing(true);
    setMessage('');
    try {
      const result = await invoke<string>('sync_week_ahead');
      setMessage(result);
    } catch (error) {
      setMessage(`❌ Fehler: ${error}`);
    } finally {
      setSyncing(false);
    }
  };

  return (
    <div className="p-6 bg-white rounded-xl shadow-lg">
      <div className="flex items-center gap-3 mb-4">
        <Cloud className="w-6 h-6 text-blue-500" />
        <h2 className="text-xl font-bold text-slate-900">Putzplan Sync</h2>
      </div>

      <p className="text-sm text-slate-600 mb-4">
        Synchronisiere Checkout-Daten zu Supabase für Putzkräfte-Zugriff
      </p>

      <div className="flex gap-3 mb-4">
        <button
          onClick={handleSyncToday}
          disabled={syncing}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-400 text-white font-semibold rounded-lg transition-colors"
        >
          <Calendar className={`w-4 h-4 ${syncing ? 'animate-spin' : ''}`} />
          Heute synchronisieren
        </button>

        <button
          onClick={handleSyncWeek}
          disabled={syncing}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-400 text-white font-semibold rounded-lg transition-colors"
        >
          <RefreshCw className={`w-4 h-4 ${syncing ? 'animate-spin' : ''}`} />
          7 Tage synchronisieren
        </button>
      </div>

      {message && (
        <div className={`p-4 rounded-lg text-sm whitespace-pre-line ${
          message.includes('❌')
            ? 'bg-red-50 text-red-700 border border-red-200'
            : 'bg-green-50 text-green-700 border border-green-200'
        }`}>
          {message}
        </div>
      )}

      <div className="mt-4 p-4 bg-blue-50 rounded-lg border border-blue-200">
        <p className="text-sm text-blue-800 font-semibold mb-2">
          📱 Zugriff für Putzkräfte:
        </p>
        <p className="text-sm text-blue-700">
          Nach der Synchronisation können Putzkräfte über die Mobile-App auf die Daten zugreifen.
        </p>
        <p className="text-xs text-blue-600 mt-2">
          <strong>Wichtig:</strong> Supabase muss konfiguriert sein (siehe Einstellungen)
        </p>
      </div>
    </div>
  );
}
