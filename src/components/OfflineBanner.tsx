import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { WifiOff, Wifi } from 'lucide-react';

export default function OfflineBanner() {
  const [isOnline, setIsOnline] = useState(true);
  const [checking, setChecking] = useState(false);

  const checkConnection = async () => {
    setChecking(true);
    try {
      const connected = await invoke<boolean>('check_db_connection');
      setIsOnline(connected);
    } catch (error) {
      console.error('Connection check failed:', error);
      setIsOnline(false);
    } finally {
      setChecking(false);
    }
  };

  useEffect(() => {
    // Initial check
    checkConnection();

    // Check every 30 seconds
    const interval = setInterval(checkConnection, 30000);

    return () => clearInterval(interval);
  }, []);

  if (isOnline) {
    return null; // Don't show banner when online
  }

  return (
    <div className="fixed top-0 left-0 right-0 bg-red-600 text-white px-4 py-3 z-[9999] shadow-lg">
      <div className="flex items-center justify-between max-w-7xl mx-auto">
        <div className="flex items-center gap-3">
          <WifiOff className="w-6 h-6" />
          <div>
            <div className="font-bold text-lg">
              ⚠️ KEINE VERBINDUNG ZUR DATENBANK
            </div>
            <div className="text-sm">
              Nur-Lese-Modus aktiv. Daten sind möglicherweise nicht aktuell. Bearbeiten/Erstellen nicht möglich.
            </div>
          </div>
        </div>
        <button
          onClick={checkConnection}
          disabled={checking}
          className="px-4 py-2 bg-white text-red-600 rounded font-semibold hover:bg-gray-100 disabled:opacity-50 flex items-center gap-2"
        >
          <Wifi className="w-4 h-4" />
          {checking ? 'Prüfe...' : 'Erneut prüfen'}
        </button>
      </div>
    </div>
  );
}
