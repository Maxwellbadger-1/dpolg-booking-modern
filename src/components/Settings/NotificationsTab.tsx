import { useState } from 'react';
import { Bell, Save } from 'lucide-react';

export default function NotificationsTab() {
  const [autoConfirmation, setAutoConfirmation] = useState(true);
  const [autoInvoice, setAutoInvoice] = useState(true);
  const [autoReminder, setAutoReminder] = useState(true);
  const [autoCancellation, setAutoCancellation] = useState(true);
  const [reminderDays, setReminderDays] = useState(14);
  const [loading, setLoading] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setSuccessMessage(null);

    // TODO: Implementiere Backend-Speicherung
    setTimeout(() => {
      setLoading(false);
      setSuccessMessage('Benachrichtigungseinstellungen erfolgreich gespeichert!');
      setTimeout(() => setSuccessMessage(null), 3000);
    }, 500);
  };

  return (
    <form onSubmit={handleSave} className="space-y-6">
      <div>
        <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
          <Bell className="w-5 h-5" />
          Automatische Email-Benachrichtigungen
        </h3>
        <p className="text-sm text-slate-400 mb-6">
          Legen Sie fest, welche Emails automatisch versendet werden sollen.
        </p>

        <div className="space-y-6">
          {/* Auto Confirmation */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="auto_confirmation"
              checked={autoConfirmation}
              onChange={(e) => setAutoConfirmation(e.target.checked)}
              className="mt-1 w-4 h-4 text-blue-600 border-slate-600 rounded focus:ring-blue-500"
            />
            <div className="flex-1">
              <label htmlFor="auto_confirmation" className="block text-sm font-semibold text-white cursor-pointer">
                Buchungsbestätigung bei Erstellung
              </label>
              <p className="text-xs text-slate-400 mt-1">
                Sendet automatisch eine Bestätigungs-Email wenn eine neue Buchung erstellt wird.
              </p>
            </div>
          </div>

          {/* Auto Invoice */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="auto_invoice"
              checked={autoInvoice}
              onChange={(e) => setAutoInvoice(e.target.checked)}
              className="mt-1 w-4 h-4 text-blue-600 border-slate-600 rounded focus:ring-blue-500"
            />
            <div className="flex-1">
              <label htmlFor="auto_invoice" className="block text-sm font-semibold text-white cursor-pointer">
                Rechnung mit Buchungsbestätigung senden
              </label>
              <p className="text-xs text-slate-400 mt-1">
                Hängt die Rechnung direkt an die Buchungsbestätigung an (empfohlen).
              </p>
            </div>
          </div>

          {/* Auto Reminder */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="auto_reminder"
              checked={autoReminder}
              onChange={(e) => setAutoReminder(e.target.checked)}
              className="mt-1 w-4 h-4 text-blue-600 border-slate-600 rounded focus:ring-blue-500"
            />
            <div className="flex-1">
              <label htmlFor="auto_reminder" className="block text-sm font-semibold text-white cursor-pointer">
                Automatische Zahlungserinnerung
              </label>
              <p className="text-xs text-slate-400 mt-1 mb-3">
                Sendet automatisch eine Zahlungserinnerung an Gäste mit unbezahlten Buchungen.
              </p>
              {autoReminder && (
                <div>
                  <label className="block text-xs font-medium text-slate-300 mb-2">
                    Erinnerung nach (Tage)
                  </label>
                  <input
                    type="number"
                    value={reminderDays}
                    onChange={(e) => setReminderDays(parseInt(e.target.value))}
                    min="1"
                    max="90"
                    className="w-32 px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <p className="text-xs text-slate-400 mt-1">
                    Standard: 14 Tage nach Buchung
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Auto Cancellation */}
          <div className="flex items-start gap-4 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
            <input
              type="checkbox"
              id="auto_cancellation"
              checked={autoCancellation}
              onChange={(e) => setAutoCancellation(e.target.checked)}
              className="mt-1 w-4 h-4 text-blue-600 border-slate-600 rounded focus:ring-blue-500"
            />
            <div className="flex-1">
              <label htmlFor="auto_cancellation" className="block text-sm font-semibold text-white cursor-pointer">
                Stornierungsbestätigung bei Stornierung
              </label>
              <p className="text-xs text-slate-400 mt-1">
                Sendet automatisch eine Bestätigung wenn eine Buchung storniert wird.
              </p>
            </div>
          </div>
        </div>
      </div>

      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-sm font-bold text-white mb-3">Hinweis</h3>
        <div className="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4">
          <p className="text-sm text-blue-300">
            Stellen Sie sicher, dass die Email-Konfiguration korrekt eingerichtet ist,
            damit automatische Benachrichtigungen versendet werden können.
          </p>
        </div>
      </div>

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
          disabled={loading}
          className="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center gap-2"
        >
          {loading ? (
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
