import { useState } from 'react';
import { Building2, Save } from 'lucide-react';

export default function PaymentSettingsTab() {
  const [bankName, setBankName] = useState('');
  const [iban, setIban] = useState('');
  const [bic, setBic] = useState('');
  const [accountHolder, setAccountHolder] = useState('');
  const [mwst, setMwst] = useState(19);
  const [paymentDueDays, setPaymentDueDays] = useState(14);
  const [reminderAfterDays, setReminderAfterDays] = useState(14);
  const [loading, setLoading] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setSuccessMessage(null);

    // TODO: Implementiere Backend-Speicherung
    setTimeout(() => {
      setLoading(false);
      setSuccessMessage('Zahlungseinstellungen erfolgreich gespeichert!');
      setTimeout(() => setSuccessMessage(null), 3000);
    }, 500);
  };

  return (
    <form onSubmit={handleSave} className="space-y-6">
      <div>
        <h3 className="text-lg font-bold text-white mb-4">Bankverbindung</h3>
        <div className="space-y-4">
          {/* Bank Name */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              <div className="flex items-center gap-2">
                <Building2 className="w-4 h-4" />
                Bankname *
              </div>
            </label>
            <input
              type="text"
              required
              value={bankName}
              onChange={(e) => setBankName(e.target.value)}
              placeholder="Sparkasse Musterhausen"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* IBAN */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              IBAN *
            </label>
            <input
              type="text"
              required
              value={iban}
              onChange={(e) => setIban(e.target.value)}
              placeholder="DE89 3704 0044 0532 0130 00"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* BIC */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              BIC
            </label>
            <input
              type="text"
              value={bic}
              onChange={(e) => setBic(e.target.value)}
              placeholder="COBADEFFXXX"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Account Holder */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Kontoinhaber *
            </label>
            <input
              type="text"
              required
              value={accountHolder}
              onChange={(e) => setAccountHolder(e.target.value)}
              placeholder="DPolG Stiftung Kreisverband"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </div>
      </div>

      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-lg font-bold text-white mb-4">Steuer & Zahlungsfristen</h3>
        <div className="space-y-4">
          {/* MwSt */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Mehrwertsteuer (%)
            </label>
            <input
              type="number"
              step="0.01"
              value={mwst}
              onChange={(e) => setMwst(parseFloat(e.target.value))}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <p className="text-xs text-slate-400 mt-1">Standard MwSt-Satz für Rechnungen</p>
          </div>

          {/* Payment Due Days */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Zahlungsziel (Tage)
            </label>
            <input
              type="number"
              value={paymentDueDays}
              onChange={(e) => setPaymentDueDays(parseInt(e.target.value))}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <p className="text-xs text-slate-400 mt-1">
              Anzahl Tage bis zur Fälligkeit der Zahlung
            </p>
          </div>

          {/* Reminder After Days */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Zahlungserinnerung nach (Tage)
            </label>
            <input
              type="number"
              value={reminderAfterDays}
              onChange={(e) => setReminderAfterDays(parseInt(e.target.value))}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <p className="text-xs text-slate-400 mt-1">
              Automatische Zahlungserinnerung nach x Tagen
            </p>
          </div>
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
