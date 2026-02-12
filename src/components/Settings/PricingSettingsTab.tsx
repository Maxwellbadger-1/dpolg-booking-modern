import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DollarSign, Calendar, Percent, Target } from 'lucide-react';

interface PricingSettings {
  id: number;
  hauptsaisonAktiv: boolean | null;
  hauptsaisonStart: string | null;  // MM-DD Format
  hauptsaisonEnde: string | null;   // MM-DD Format
  mitgliederRabattAktiv: boolean | null;
  mitgliederRabattProzent: number | null;
  rabattBasis: string | null;
  updatedAt: string | null;
}

export default function PricingSettingsTab() {
  const [settings, setSettings] = useState<PricingSettings>({
    id: 1,
    hauptsaisonAktiv: true,
    hauptsaisonStart: '06-01',
    hauptsaisonEnde: '08-31',
    mitgliederRabattAktiv: true,
    mitgliederRabattProzent: 15.0,
    rabattBasis: 'zimmerpreis',
    updatedAt: '',
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const result = await invoke<PricingSettings>('get_pricing_settings_pg');
      setSettings(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Fehler beim Laden der Einstellungen');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      setError(null);
      await invoke('update_pricing_settings_pg', { settings });
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Fehler beim Speichern');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mb-4"></div>
          <p className="text-slate-300">Lade Preiseinstellungen...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-blue-500/20 rounded-lg">
          <DollarSign className="w-5 h-5 text-blue-400" />
        </div>
        <div>
          <h3 className="text-lg font-semibold text-white">Preiseinstellungen</h3>
          <p className="text-sm text-slate-400">
            Konfiguriere Saisonzeiten und Mitgliederrabatte
          </p>
        </div>
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-500/10 border border-red-500/50 rounded-lg p-4">
          <p className="text-red-400 text-sm">{error}</p>
        </div>
      )}

      {/* Success Message */}
      {success && (
        <div className="bg-emerald-500/10 border border-emerald-500/50 rounded-lg p-4">
          <p className="text-emerald-400 text-sm">✅ Einstellungen erfolgreich gespeichert!</p>
        </div>
      )}

      {/* Hauptsaison Settings */}
      <div className="bg-slate-700/50 rounded-lg p-6 space-y-4">
        <div className="flex items-center gap-3 mb-4">
          <Calendar className="w-5 h-5 text-blue-400" />
          <h4 className="text-base font-semibold text-white">Saisonzeiten</h4>
        </div>

        {/* Hauptsaison Aktivierung */}
        <div className="flex items-center justify-between">
          <div>
            <label className="text-sm font-medium text-slate-300">
              Hauptsaison aktivieren
            </label>
            <p className="text-xs text-slate-400 mt-1">
              Wenn deaktiviert, wird immer der Nebensaison-Preis verwendet
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={settings.hauptsaisonAktiv}
              onChange={(e) =>
                setSettings({ ...settings, hauptsaisonAktiv: e.target.checked })
              }
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-slate-600 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
          </label>
        </div>

        {/* Hauptsaison Zeitraum */}
        {settings.hauptsaisonAktiv && (
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Start (MM-TT)
              </label>
              <input
                type="text"
                value={settings.hauptsaisonStart}
                onChange={(e) =>
                  setSettings({ ...settings, hauptsaisonStart: e.target.value })
                }
                placeholder="06-01"
                className="w-full px-4 py-2 bg-slate-600 border border-slate-500 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <p className="text-xs text-slate-400 mt-1">Format: MM-TT (z.B. 06-01 für 1. Juni)</p>
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Ende (MM-TT)
              </label>
              <input
                type="text"
                value={settings.hauptsaisonEnde}
                onChange={(e) =>
                  setSettings({ ...settings, hauptsaisonEnde: e.target.value })
                }
                placeholder="08-31"
                className="w-full px-4 py-2 bg-slate-600 border border-slate-500 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <p className="text-xs text-slate-400 mt-1">Format: MM-TT (z.B. 08-31 für 31. August)</p>
            </div>
          </div>
        )}
      </div>

      {/* Mitgliederrabatt Settings */}
      <div className="bg-slate-700/50 rounded-lg p-6 space-y-4">
        <div className="flex items-center gap-3 mb-4">
          <Percent className="w-5 h-5 text-emerald-400" />
          <h4 className="text-base font-semibold text-white">Mitgliederrabatt (DPolG)</h4>
        </div>

        {/* Rabatt Aktivierung */}
        <div className="flex items-center justify-between">
          <div>
            <label className="text-sm font-medium text-slate-300">
              Rabatt aktivieren
            </label>
            <p className="text-xs text-slate-400 mt-1">
              Automatischer Rabatt für DPolG-Mitglieder
            </p>
          </div>
          <label className="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={settings.mitgliederRabattAktiv}
              onChange={(e) =>
                setSettings({ ...settings, mitgliederRabattAktiv: e.target.checked })
              }
              className="sr-only peer"
            />
            <div className="w-11 h-6 bg-slate-600 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-emerald-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
          </label>
        </div>

        {/* Rabatt Prozentsatz */}
        {settings.mitgliederRabattAktiv && (
          <>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Rabatt-Prozentsatz
              </label>
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  value={settings.mitgliederRabattProzent}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      mitgliederRabattProzent: parseFloat(e.target.value) || 0,
                    })
                  }
                  min="0"
                  max="100"
                  step="0.1"
                  className="flex-1 px-4 py-2 bg-slate-600 border border-slate-500 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-emerald-500"
                />
                <span className="text-slate-300 font-semibold">%</span>
              </div>
              <p className="text-xs text-slate-400 mt-1">
                Standard: 15% (empfohlen für DPolG-Mitglieder)
              </p>
            </div>

            {/* Rabatt-Basis */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Rabatt gilt für
              </label>
              <p className="text-xs text-slate-400 mb-3">
                Diese Einstellung gilt für <strong>alle prozentualen Rabatte</strong> (manuell und DPolG-Auto-Rabatt).
                Hinweis: Services haben ein eigenes "Anwenden auf" Feld pro Service.
              </p>
              <div className="space-y-3">
                <label className="flex items-start gap-3 p-3 bg-slate-600/50 rounded-lg cursor-pointer hover:bg-slate-600 transition-colors">
                  <input
                    type="radio"
                    name="rabattBasis"
                    value="zimmerpreis"
                    checked={settings.rabattBasis === 'zimmerpreis'}
                    onChange={(e) =>
                      setSettings({
                        ...settings,
                        rabattBasis: e.target.value as 'zimmerpreis' | 'gesamtpreis',
                      })
                    }
                    className="mt-1 w-4 h-4 text-emerald-500 focus:ring-emerald-500"
                  />
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <Target className="w-4 h-4 text-emerald-400" />
                      <span className="text-sm font-medium text-white">Nur Zimmerpreis</span>
                    </div>
                    <p className="text-xs text-slate-400 mt-1">
                      Rabatt wird nur auf den Zimmerpreis angewendet (ohne Services wie Endreinigung)
                    </p>
                  </div>
                </label>

                <label className="flex items-start gap-3 p-3 bg-slate-600/50 rounded-lg cursor-pointer hover:bg-slate-600 transition-colors">
                  <input
                    type="radio"
                    name="rabattBasis"
                    value="gesamtpreis"
                    checked={settings.rabattBasis === 'gesamtpreis'}
                    onChange={(e) =>
                      setSettings({
                        ...settings,
                        rabattBasis: e.target.value as 'zimmerpreis' | 'gesamtpreis',
                      })
                    }
                    className="mt-1 w-4 h-4 text-emerald-500 focus:ring-emerald-500"
                  />
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <DollarSign className="w-4 h-4 text-emerald-400" />
                      <span className="text-sm font-medium text-white">Gesamtpreis</span>
                    </div>
                    <p className="text-xs text-slate-400 mt-1">
                      Rabatt wird auf Zimmerpreis + Services angewendet
                    </p>
                  </div>
                </label>
              </div>
            </div>
          </>
        )}
      </div>

      {/* Save Button */}
      <div className="flex justify-end pt-4">
        <button
          onClick={handleSave}
          disabled={saving}
          className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors disabled:bg-slate-600 disabled:cursor-not-allowed flex items-center gap-2"
        >
          {saving ? (
            <>
              <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              Speichert...
            </>
          ) : (
            'Einstellungen speichern'
          )}
        </button>
      </div>
    </div>
  );
}
