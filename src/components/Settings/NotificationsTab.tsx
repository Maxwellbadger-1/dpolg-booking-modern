import { useState, useEffect } from 'react';
import { Bell, Save, RefreshCw, AlertCircle, PlayCircle, Mail, Bug } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { ReminderSettings } from '../../types/reminder';

interface NotificationSettings {
  id: number;
  checkinRemindersEnabled: boolean | null;
  paymentRemindersEnabled: boolean | null;
  paymentReminderAfterDays: number | null;
  paymentReminderRepeatDays: number | null;
  schedulerIntervalHours: number | null;
  updatedAt: string | null;
}

interface ScheduledEmail {
  booking_id: number;
  reservierungsnummer: string;
  guest_name: string;
  guest_email: string;
  email_type: string;
  scheduled_date: string;
  reason: string;
}

export default function NotificationsTab() {
  const [settings, setSettings] = useState<NotificationSettings>({
    id: 1,
    checkinRemindersEnabled: true,
    paymentRemindersEnabled: true,
    paymentReminderAfterDays: 14,
    paymentReminderRepeatDays: 14,
    schedulerIntervalHours: 1,
    updatedAt: null,
  });
  const [reminderSettings, setReminderSettings] = useState<ReminderSettings>({
    id: 1,
    auto_reminder_incomplete_data: true,
    auto_reminder_payment: true,
    auto_reminder_checkin: true,
    auto_reminder_invoice: true,
    updatedAt: null,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);
  const [scheduledEmails, setScheduledEmails] = useState<ScheduledEmail[]>([]);
  const [showScheduled, setShowScheduled] = useState(false);

  // Load settings on mount
  useEffect(() => {
    loadSettings();
    loadReminderSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const data = await invoke<NotificationSettings>('get_notification_settings_pg');
      setSettings(data);
      setError(null);
    } catch (err) {
      console.error('Fehler beim Laden der Einstellungen:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const loadReminderSettings = async () => {
    try {
      const data = await invoke<ReminderSettings>('get_reminder_settings_command');
      setReminderSettings(data);
    } catch (err) {
      console.error('Fehler beim Laden der Reminder-Einstellungen:', err);
    }
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setSuccessMessage(null);
    setError(null);

    try {
      // Save email notification settings
      const updated = await invoke<NotificationSettings>('update_notification_settings_pg', { settings });
      setSettings(updated);

      // Save reminder settings
      const updatedReminders = await invoke<ReminderSettings>('save_reminder_settings_command', { settings: reminderSettings });
      setReminderSettings(updatedReminders);

      setSuccessMessage('Einstellungen erfolgreich gespeichert!');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      console.error('Fehler beim Speichern:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleTestScheduler = async () => {
    setTesting(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const result = await invoke<string>('trigger_email_check');
      setSuccessMessage(`✅ ${result}`);
      setTimeout(() => setSuccessMessage(null), 5000);
    } catch (err) {
      console.error('Fehler beim Testen:', err);
      setError(`Fehler: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setTesting(false);
    }
  };

  const handleShowScheduled = async () => {
    setTesting(true);
    setError(null);

    try {
      const emails = await invoke<ScheduledEmail[]>('get_scheduled_emails');
      setScheduledEmails(emails);
      setShowScheduled(true);
      console.log('📧 Scheduled emails:', emails);
    } catch (err) {
      console.error('Fehler beim Laden geplanter Emails:', err);
      setError(`Fehler: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setTesting(false);
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
              checked={settings.checkinRemindersEnabled}
              onChange={(e) => setSettings({ ...settings, checkinRemindersEnabled: e.target.checked })}
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
              checked={settings.paymentRemindersEnabled}
              onChange={(e) => setSettings({ ...settings, paymentRemindersEnabled: e.target.checked })}
              className="mt-1 w-4 h-4 text-blue-600 border-slate-600 rounded focus:ring-blue-500"
            />
            <div className="flex-1">
              <label htmlFor="payment_reminders" className="block text-sm font-semibold text-white cursor-pointer">
                Zahlungserinnerungen
              </label>
              <p className="text-xs text-slate-400 mt-1 mb-3">
                Sendet automatisch Zahlungserinnerungen an Gäste mit unbezahlten Buchungen.
              </p>
              {settings.paymentRemindersEnabled && (
                <div className="grid grid-cols-2 gap-4 mt-3">
                  <div>
                    <label className="block text-xs font-medium text-slate-300 mb-2">
                      Erste Erinnerung nach (Tage)
                    </label>
                    <input
                      type="number"
                      value={settings.paymentReminderAfterDays}
                      onChange={(e) => setSettings({ ...settings, paymentReminderAfterDays: parseInt(e.target.value) || 14 })}
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
                      value={settings.paymentReminderRepeatDays}
                      onChange={(e) => setSettings({ ...settings, paymentReminderRepeatDays: parseInt(e.target.value) || 14 })}
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
                    value={settings.schedulerIntervalHours}
                    onChange={(e) => setSettings({ ...settings, schedulerIntervalHours: parseInt(e.target.value) || 1 })}
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

      {/* Auto-Reminder Settings */}
      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
          <AlertCircle className="w-5 h-5" />
          Automatische Erinnerungen (Reminder-System)
        </h3>
        <p className="text-sm text-slate-400 mb-6">
          Legen Sie fest, welche Erinnerungen automatisch im System erstellt werden sollen.
        </p>

        <div className="space-y-4">
          {/* Auto-Reminder: Incomplete Data */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="auto_reminder_incomplete_data"
              checked={reminderSettings.auto_reminder_incomplete_data}
              onChange={(e) => setReminderSettings({ ...reminderSettings, auto_reminder_incomplete_data: e.target.checked })}
              className="mt-1 w-4 h-4 text-purple-600 border-slate-600 rounded focus:ring-purple-500"
            />
            <div className="flex-1">
              <label htmlFor="auto_reminder_incomplete_data" className="block text-sm font-semibold text-white cursor-pointer">
                Unvollständige Gästdaten
              </label>
              <p className="text-xs text-slate-400 mt-1">
                Erstellt automatisch eine Erinnerung, wenn wichtige Gästdaten fehlen (z.B. Adresse, Telefon).
              </p>
            </div>
          </div>

          {/* Auto-Reminder: Payment */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="auto_reminder_payment"
              checked={reminderSettings.auto_reminder_payment}
              onChange={(e) => setReminderSettings({ ...reminderSettings, auto_reminder_payment: e.target.checked })}
              className="mt-1 w-4 h-4 text-purple-600 border-slate-600 rounded focus:ring-purple-500"
            />
            <div className="flex-1">
              <label htmlFor="auto_reminder_payment" className="block text-sm font-semibold text-white cursor-pointer">
                Zahlung ausstehend
              </label>
              <p className="text-xs text-slate-400 mt-1">
                Erstellt eine Erinnerung 7 Tage vor Check-in, wenn die Buchung noch nicht bezahlt wurde.
              </p>
            </div>
          </div>

          {/* Auto-Reminder: Check-in */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="auto_reminder_checkin"
              checked={reminderSettings.auto_reminder_checkin}
              onChange={(e) => setReminderSettings({ ...reminderSettings, auto_reminder_checkin: e.target.checked })}
              className="mt-1 w-4 h-4 text-purple-600 border-slate-600 rounded focus:ring-purple-500"
            />
            <div className="flex-1">
              <label htmlFor="auto_reminder_checkin" className="block text-sm font-semibold text-white cursor-pointer">
                Check-in Vorbereitung
              </label>
              <p className="text-xs text-slate-400 mt-1">
                Erstellt eine Erinnerung 1 Tag vor Check-in zur Vorbereitung (Schlüssel, Zimmer prüfen, etc.).
              </p>
            </div>
          </div>

          {/* Auto-Reminder: Invoice */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="auto_reminder_invoice"
              checked={reminderSettings.auto_reminder_invoice}
              onChange={(e) => setReminderSettings({ ...reminderSettings, auto_reminder_invoice: e.target.checked })}
              className="mt-1 w-4 h-4 text-purple-600 border-slate-600 rounded focus:ring-purple-500"
            />
            <div className="flex-1">
              <label htmlFor="auto_reminder_invoice" className="block text-sm font-semibold text-white cursor-pointer">
                Rechnung versenden
              </label>
              <p className="text-xs text-slate-400 mt-1">
                Erstellt eine Erinnerung nach Buchungsbestätigung, die Rechnung zu versenden.
              </p>
            </div>
          </div>
        </div>

        <div className="bg-purple-500/10 border border-purple-500/30 rounded-lg p-4 mt-6">
          <p className="text-sm text-purple-300">
            <strong>Hinweis:</strong> Diese Einstellungen steuern, welche Erinnerungen <strong>automatisch</strong> im System erstellt werden.
            Sie können jederzeit zusätzlich manuelle Erinnerungen für einzelne Buchungen erstellen.
          </p>
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

      {/* Debug/Testing Panel */}
      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-sm font-bold text-white mb-3 flex items-center gap-2">
          <Bug className="w-5 h-5 text-amber-400" />
          Testing & Debug
        </h3>
        <p className="text-sm text-slate-400 mb-4">
          Testen Sie das Reminder-System und sehen Sie, welche Emails geplant sind.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Test Scheduler Button */}
          <button
            type="button"
            onClick={handleTestScheduler}
            disabled={testing}
            className="px-4 py-3 bg-amber-600 hover:bg-amber-700 disabled:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center justify-center gap-2"
          >
            {testing ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                Prüfe...
              </>
            ) : (
              <>
                <PlayCircle className="w-4 h-4" />
                Jetzt prüfen (manuell)
              </>
            )}
          </button>

          {/* Show Scheduled Emails Button */}
          <button
            type="button"
            onClick={handleShowScheduled}
            disabled={testing}
            className="px-4 py-3 bg-purple-600 hover:bg-purple-700 disabled:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center justify-center gap-2"
          >
            {testing ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                Lade...
              </>
            ) : (
              <>
                <Mail className="w-4 h-4" />
                Geplante Emails anzeigen
              </>
            )}
          </button>
        </div>

        {/* Scheduled Emails List */}
        {showScheduled && (
          <div className="mt-4 bg-slate-700/50 rounded-lg border border-slate-600 p-4">
            <div className="flex items-center justify-between mb-3">
              <h4 className="text-sm font-bold text-white">
                Geplante automatische Emails ({scheduledEmails.length})
              </h4>
              <button
                type="button"
                onClick={() => setShowScheduled(false)}
                className="text-xs text-slate-400 hover:text-white transition-colors"
              >
                Schließen
              </button>
            </div>

            {scheduledEmails.length === 0 ? (
              <p className="text-sm text-slate-400">
                ✅ Keine geplanten Emails gefunden. Alle Erinnerungen wurden bereits versendet oder es gibt keine anstehenden Buchungen.
              </p>
            ) : (
              <div className="space-y-2">
                {scheduledEmails.map((email, idx) => (
                  <div key={idx} className="bg-slate-800/50 rounded p-3 border border-slate-600">
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex-1">
                        <div className="flex items-center gap-2 mb-1">
                          <span className={`inline-flex px-2 py-0.5 text-xs font-semibold rounded ${
                            email.email_type === 'Erinnerung'
                              ? 'bg-blue-500/20 text-blue-300'
                              : 'bg-amber-500/20 text-amber-300'
                          }`}>
                            {email.email_type}
                          </span>
                          <span className="text-xs text-slate-400">
                            {email.reservierungsnummer}
                          </span>
                        </div>
                        <p className="text-sm text-white font-medium">
                          {email.guest_name} ({email.guest_email})
                        </p>
                        <p className="text-xs text-slate-400 mt-1">
                          {email.reason}
                        </p>
                      </div>
                      <div className="text-right">
                        <p className="text-xs text-slate-300 font-semibold">
                          {email.scheduled_date}
                        </p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
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
