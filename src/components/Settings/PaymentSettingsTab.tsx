import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Building2, Save } from 'lucide-react';

interface PaymentSettings {
  id: number;
  bank_name: string;
  iban: string;
  bic: string;
  account_holder: string;
  mwst_rate: number;
  payment_due_days: number;
  reminder_after_days: number;
  payment_text?: string;
  updated_at: string;
}

export default function PaymentSettingsTab() {
  const [settings, setSettings] = useState<PaymentSettings | null>(null);
  const [bankName, setBankName] = useState('');
  const [iban, setIban] = useState('');
  const [bic, setBic] = useState('');
  const [accountHolder, setAccountHolder] = useState('');
  const [mwstRate, setMwstRate] = useState(7);
  const [paymentDueDays, setPaymentDueDays] = useState(14);
  const [reminderAfterDays, setReminderAfterDays] = useState(14);
  const [paymentText, setPaymentText] = useState('');
  const [loading, setLoading] = useState(false);
  const [loadingData, setLoadingData] = useState(true);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    setLoadingData(true);
    try {
      const data = await invoke<PaymentSettings>('get_payment_settings_command');
      setSettings(data);

      // Formular befüllen
      setBankName(data.bank_name);
      setIban(data.iban);
      setBic(data.bic);
      setAccountHolder(data.account_holder);
      setMwstRate(data.mwst_rate);
      setPaymentDueDays(data.payment_due_days);
      setReminderAfterDays(data.reminder_after_days);
      setPaymentText(data.payment_text || '');
    } catch (error) {
      console.error('Fehler beim Laden der Zahlungseinstellungen:', error);
      alert(`Fehler beim Laden: ${error}`);
    } finally {
      setLoadingData(false);
    }
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setSuccessMessage(null);

    try {
      const updatedSettings: PaymentSettings = {
        id: settings?.id || 1,
        bank_name: bankName,
        iban,
        bic,
        account_holder: accountHolder,
        mwst_rate: mwstRate,
        payment_due_days: paymentDueDays,
        reminder_after_days: reminderAfterDays,
        payment_text: paymentText || undefined,
        updated_at: new Date().toISOString(),
      };

      const result = await invoke<PaymentSettings>('save_payment_settings_command', {
        settings: updatedSettings,
      });

      setSettings(result);
      setSuccessMessage('Zahlungseinstellungen erfolgreich gespeichert!');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (error) {
      console.error('Fehler beim Speichern:', error);
      alert(`Fehler beim Speichern: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  if (loadingData) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

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
              placeholder="Sparda Bank München"
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
              placeholder="DE70 7009 0500 0001 9999 90"
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
              placeholder="GENODEF1S04"
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
              placeholder="Stiftung der Deutschen Polizeigewerkschaft"
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
              value={mwstRate}
              onChange={(e) => setMwstRate(parseFloat(e.target.value))}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <p className="text-xs text-slate-400 mt-1">Standard MwSt-Satz für Rechnungen (7% für Übernachtungen)</p>
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
