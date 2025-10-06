import { useState, useEffect } from 'react';
import { Bell, Save, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface NotificationSettings {
  id: number;
  checkin_reminders_enabled: boolean;
  payment_reminders_enabled: boolean;
  payment_reminder_after_days: number;
  payment_reminder_repeat_days: number;
  scheduler_interval_hours: number;
  updated_at: string;
}

export default function NotificationsTab() {
  const [settings, setSettings] = useState<NotificationSettings>({
    id: 1,
    checkin_reminders_enabled: true,
    payment_reminders_enabled: true,
    payment_reminder_after_days: 14,
    payment_reminder_repeat_days: 14,
    scheduler_interval_hours: 1,
    updated_at: '',
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Load settings on mount
  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const data = await invoke<NotificationSettings>('get_notification_settings_command');
      setSettings(data);
      setError(null);
    } catch (err) {
      console.error('Fehler beim Laden der Einstellungen:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setSuccessMessage(null);
    setError(null);

    try {
      const updated = await invoke<NotificationSettings>('save_notification_settings_command', { settings });
      setSettings(updated);
      setSuccessMessage('Benachrichtigungseinstellungen erfolgreich gespeichert!');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      console.error('Fehler beim Speichern:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        <span className="ml-3 text-slate-300">Lade Einstellungen...</span>
      </div>
    );
  }

  return (
    <form onSubmit={handleSave} className="space-y-6">
      <div>
        <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
          <Bell className="w-5 h-5" />
          Automatische Email-Benachrichtigungen
        </h3>
        <p className="text-sm text-slate-400 mb-6">
          Konfigurieren Sie den Email-Scheduler und legen Sie fest, welche Erinnerungen automatisch versendet werden.
        </p>

        <div className="space-y-6">
          {/* Check-in Reminders */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="checkin_reminders"
              checked={settings.checkin_reminders_enabled}
              onChange={(e) => setSettings({ ...settings, checkin_reminders_enabled: e.target.checked })}
              className="mt-1 w-4 h-4 text-blue-600 border-slate-600 rounded focus:ring-blue-500"
            />
            <div className="flex-1">
              <label htmlFor="checkin_reminders" className="block text-sm font-semibold text-white cursor-pointer">
                Check-in Erinnerung (1 Tag vorher)
              </label>
              <p className="text-xs text-slate-400 mt-1">
                Sendet automatisch eine Erinnerung 1 Tag vor dem Check-in an alle Gäste mit bestätigten Buchungen.
              </p>
            </div>
          </div>

          {/* Payment Reminders */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="payment_reminders"
              checked={settings.payment_reminders_enabled}
              onChange={(e) => setSettings({ ...settings, payment_reminders_enabled: e.target.checked })}
              className="mt-1 w-4 h-4 text-blue-600 border-slate-600 rounded focus:ring-blue-500"
            />
            <div className="flex-1">
              <label htmlFor="payment_reminders" className="block text-sm font-semibold text-white cursor-pointer">
                Zahlungserinnerungen
              </label>
              <p className="text-xs text-slate-400 mt-1 mb-3">
                Sendet automatisch Zahlungserinnerungen an Gäste mit unbezahlten Buchungen.
              </p>
              {settings.payment_reminders_enabled && (
                <div className="grid grid-cols-2 gap-4 mt-3">
                  <div>
                    <label className="block text-xs font-medium text-slate-300 mb-2">
                      Erste Erinnerung nach (Tage)
                    </label>
                    <input
                      type="number"
                      value={settings.payment_reminder_after_days}
                      onChange={(e) => setSettings({ ...settings, payment_reminder_after_days: parseInt(e.target.value) || 14 })}
                      min="1"
                      max="90"
                      className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <p className="text-xs text-slate-400 mt-1">
                      Nach wie vielen Tagen seit Buchungserstellung
                    </p>
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-slate-300 mb-2">
                      Wiederholung nach (Tage)
                    </label>
                    <input
                      type="number"
                      value={settings.payment_reminder_repeat_days}
                      onChange={(e) => setSettings({ ...settings, payment_reminder_repeat_days: parseInt(e.target.value) || 14 })}
                      min="1"
                      max="90"
                      className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <p className="text-xs text-slate-400 mt-1">
                      Nach wie vielen Tagen erneut mahnen
                    </p>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Scheduler Interval */}
          <div className="p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <div className="flex items-start gap-4">
              <RefreshCw className="w-5 h-5 text-blue-400 mt-1" />
              <div className="flex-1">
                <label className="block text-sm font-semibold text-white mb-2">
                  Scheduler-Intervall (Stunden)
                </label>
                <p className="text-xs text-slate-400 mb-3">
                  Wie oft soll der Email-Scheduler nach anstehenden Emails prüfen?
                </p>
                <div className="flex items-center gap-3">
                  <input
                    type="number"
                    value={settings.scheduler_interval_hours}
                    onChange={(e) => setSettings({ ...settings, scheduler_interval_hours: parseInt(e.target.value) || 1 })}
                    min="1"
                    max="24"
                    className="w-32 px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <span className="text-sm text-slate-300">Stunde(n)</span>
                </div>
                <p className="text-xs text-slate-400 mt-2">
                  ⚠️ Empfohlen: 1 Stunde (für Tests). In Production: 24 Stunden.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Info Box */}
      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-sm font-bold text-white mb-3">Wichtige Hinweise</h3>
        <div className="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4 space-y-2">
          <p className="text-sm text-blue-300">
            • Stellen Sie sicher, dass die Email-Konfiguration korrekt eingerichtet ist.
          </p>
          <p className="text-sm text-blue-300">
            • Der Scheduler läuft automatisch im Hintergrund und prüft regelmäßig nach anstehenden Emails.
          </p>
          <p className="text-sm text-blue-300">
            • Bereits versendete Erinnerungen werden nicht erneut versendet (Duplikatsschutz).
          </p>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4">
          <p className="text-sm text-red-300">{error}</p>
        </div>
      )}

      {/* Success Message */}
      {successMessage && (
        <div className="bg-emerald-500/10 border border-emerald-500/30 rounded-lg p-4">
          <p className="text-sm text-emerald-300">{successMessage}</p>
        </div>
      )}

      {/* Save Button */}
      <div className="flex justify-end pt-4">
        <button
          type="submit"
          disabled={saving}
          className="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center gap-2"
        >
          {saving ? (
            <>
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
              Speichert...
            </>
          ) : (
            <>
              <Save className="w-4 h-4" />
              Speichern
            </>
          )}
        </button>
      </div>
    </form>
  );
}
