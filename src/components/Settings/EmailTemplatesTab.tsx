import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FileText, Save, RotateCcw } from 'lucide-react';

interface EmailTemplate {
  id: number;
  template_name: string;
  subject: string;
  body: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export default function EmailTemplatesTab() {
  const [templates, setTemplates] = useState<EmailTemplate[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<EmailTemplate | null>(null);
  const [subject, setSubject] = useState('');
  const [body, setBody] = useState('');
  const [description, setDescription] = useState('');
  const [loading, setLoading] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    try {
      const result = await invoke<EmailTemplate[]>('get_all_email_templates_pg');
      console.log('═══════════════════════════════════════');
      console.log('📧 Loaded Email Templates:', JSON.stringify(result, null, 2));
      console.log('📊 Count:', result.length);
      console.log('═══════════════════════════════════════');
      setTemplates(result);
      if (result.length > 0 && !selectedTemplate) {
        selectTemplate(result[0]);
      }
    } catch (error) {
      console.error('❌ Error loading templates:', error);
      setError(error instanceof Error ? error.message : String(error));
    }
  };

  const selectTemplate = (template: EmailTemplate) => {
    setSelectedTemplate(template);
    setSubject(template.subject);
    setBody(template.body);
    setDescription(template.description || '');
    setSuccessMessage(null);
    setError(null);
  };

  const handleSave = async () => {
    if (!selectedTemplate) return;

    setLoading(true);
    setError(null);
    setSuccessMessage(null);

    try {
      await invoke('update_template_command', {
        id: selectedTemplate.id,
        subject,
        body,
        description: description || null,
      });

      setSuccessMessage('Template erfolgreich gespeichert!');
      setTimeout(() => setSuccessMessage(null), 3000);
      await loadTemplates();
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    if (selectedTemplate) {
      setSubject(selectedTemplate.subject);
      setBody(selectedTemplate.body);
      setDescription(selectedTemplate.description || '');
    }
  };

  const getTemplateDisplayName = (name: string) => {
    const names: { [key: string]: string } = {
      confirmation: 'Buchungsbestätigung',
      reminder: 'Erinnerung',
      invoice: 'Rechnung',
      payment_reminder: 'Zahlungserinnerung',
      cancellation: 'Stornierungsbestätigung',
    };
    return names[name] || name;
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
      {/* Template List */}
      <div className="space-y-2">
        <h3 className="text-sm font-bold text-white mb-3">Email-Templates</h3>
        {templates.map((template) => (
          <button
            key={template.id}
            onClick={() => selectTemplate(template)}
            className={`w-full text-left p-4 rounded-lg transition-colors ${
              selectedTemplate?.id === template.id
                ? 'bg-blue-600 text-white'
                : 'bg-slate-700 hover:bg-slate-600 text-slate-300'
            }`}
          >
            <div className="flex items-start gap-3">
              <FileText className="w-5 h-5 flex-shrink-0 mt-0.5" />
              <div>
                <p className="font-semibold">{getTemplateDisplayName(template.template_name)}</p>
                {template.description && (
                  <p className="text-xs mt-1 opacity-80">{template.description}</p>
                )}
              </div>
            </div>
          </button>
        ))}
      </div>

      {/* Template Editor */}
      <div className="md:col-span-2 space-y-4">
        {selectedTemplate ? (
          <>
            <div>
              <h3 className="text-lg font-bold text-white mb-2">
                {getTemplateDisplayName(selectedTemplate.template_name)}
              </h3>
              <p className="text-sm text-slate-400">
                Template bearbeiten und speichern
              </p>
            </div>

            {/* Description */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Beschreibung
              </label>
              <input
                type="text"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Kurze Beschreibung des Templates"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Subject */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Betreff *
              </label>
              <input
                type="text"
                required
                value={subject}
                onChange={(e) => setSubject(e.target.value)}
                placeholder="Email-Betreff"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Body */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Email-Text *
              </label>
              <textarea
                required
                value={body}
                onChange={(e) => setBody(e.target.value)}
                rows={12}
                placeholder="Email-Inhalt mit Platzhaltern"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
              />
            </div>

            {/* Placeholders Info - Erweitert und gruppiert */}
            <div className="bg-slate-700/50 border border-slate-600 rounded-lg p-4">
              <p className="text-xs font-bold text-slate-300 mb-3">📌 Verfügbare Platzhalter:</p>

              {/* Gast-Daten */}
              <div className="mb-3">
                <p className="text-xs font-semibold text-emerald-400 mb-1.5">👤 Gast-Daten:</p>
                <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-400 ml-2">
                  <div><code className="text-blue-400">{'{gast_vorname}'}</code> - Vorname</div>
                  <div><code className="text-blue-400">{'{gast_nachname}'}</code> - Nachname</div>
                  <div><code className="text-blue-400">{'{gast_email}'}</code> - Email-Adresse</div>
                  <div><code className="text-blue-400">{'{gast_telefon}'}</code> - Telefonnummer</div>
                  <div><code className="text-blue-400">{'{gast_strasse}'}</code> - Straße <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{gast_plz}'}</code> - PLZ <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{gast_ort}'}</code> - Ort <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{gast_land}'}</code> - Land <span className="text-orange-400">NEU</span></div>
                </div>
              </div>

              {/* Buchungs-Daten */}
              <div className="mb-3">
                <p className="text-xs font-semibold text-amber-400 mb-1.5">📅 Buchungs-Daten:</p>
                <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-400 ml-2">
                  <div><code className="text-blue-400">{'{reservierungsnummer}'}</code> - Reservierungsnr.</div>
                  <div><code className="text-blue-400">{'{checkin_date}'}</code> - Check-in Datum</div>
                  <div><code className="text-blue-400">{'{checkout_date}'}</code> - Check-out Datum</div>
                  <div><code className="text-blue-400">{'{anzahl_gaeste}'}</code> - Anzahl Gäste</div>
                  <div><code className="text-blue-400">{'{anzahl_naechte}'}</code> - Anzahl Nächte</div>
                  <div><code className="text-blue-400">{'{buchung_status}'}</code> - Status <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{bezahlt_status}'}</code> - Bezahlt/Offen <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{erstellt_am}'}</code> - Buchungsdatum <span className="text-orange-400">NEU</span></div>
                </div>
              </div>

              {/* Mitreisende */}
              <div className="mb-3">
                <p className="text-xs font-semibold text-cyan-400 mb-1.5">👥 Mitreisende/Begleitpersonen:</p>
                <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-400 ml-2">
                  <div><code className="text-blue-400">{'{anzahl_mitreisende}'}</code> - Anzahl <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{mitreisende_namen}'}</code> - Namen (Komma) <span className="text-orange-400">NEU</span></div>
                  <div className="col-span-2"><code className="text-blue-400">{'{mitreisende_liste}'}</code> - Formatierte Liste <span className="text-orange-400">NEU</span></div>
                </div>
              </div>

              {/* Zimmer-Daten */}
              <div className="mb-3">
                <p className="text-xs font-semibold text-purple-400 mb-1.5">🏠 Zimmer-Daten:</p>
                <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-400 ml-2">
                  <div><code className="text-blue-400">{'{zimmer_name}'}</code> - Zimmer-Name</div>
                  <div><code className="text-blue-400">{'{zimmer_ort}'}</code> - Ort</div>
                  <div><code className="text-blue-400">{'{zimmer_typ}'}</code> - Gebäude-Typ</div>
                  <div><code className="text-blue-400">{'{schluesselcode}'}</code> - Schlüsselcode</div>
                  <div><code className="text-blue-400">{'{zimmer_strasse}'}</code> - Straße & Hausnr. <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{zimmer_plz}'}</code> - PLZ <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{zimmer_stadt}'}</code> - Stadt <span className="text-orange-400">NEU</span></div>
                  <div className="col-span-2"><code className="text-blue-400">{'{zimmer_adresse}'}</code> - Vollständige Adresse <span className="text-orange-400">NEU</span></div>
                </div>
              </div>

              {/* Preis-Daten */}
              <div className="mb-3">
                <p className="text-xs font-semibold text-green-400 mb-1.5">💰 Preis & Zahlung:</p>
                <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-400 ml-2">
                  <div><code className="text-blue-400">{'{grundpreis}'}</code> - Grundpreis</div>
                  <div><code className="text-blue-400">{'{services_preis}'}</code> - Zusatzleistungen</div>
                  <div><code className="text-blue-400">{'{rabatt_preis}'}</code> - Rabatt</div>
                  <div><code className="text-blue-400">{'{gesamtpreis}'}</code> - Gesamtpreis</div>
                  <div><code className="text-blue-400">{'{offener_betrag}'}</code> - Offener Betrag <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{zahlungsziel_tage}'}</code> - Zahlungsfrist (Tage) <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{zahlungsziel_datum}'}</code> - Zahlungsdatum <span className="text-orange-400">NEU</span></div>
                </div>
              </div>

              {/* Services */}
              <div className="mb-3">
                <p className="text-xs font-semibold text-indigo-400 mb-1.5">🛎️ Zusatzleistungen:</p>
                <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-400 ml-2">
                  <div><code className="text-blue-400">{'{services_liste}'}</code> - Einfache Liste <span className="text-orange-400">NEU</span></div>
                  <div><code className="text-blue-400">{'{services_details}'}</code> - Liste mit Preisen <span className="text-orange-400">NEU</span></div>
                </div>
              </div>

              {/* Firma-Daten */}
              <div className="mb-3">
                <p className="text-xs font-semibold text-blue-400 mb-1.5">🏢 Firmen-Daten:</p>
                <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-400 ml-2">
                  <div><code className="text-blue-400">{'{firma_name}'}</code> - Firmenname</div>
                  <div><code className="text-blue-400">{'{firma_adresse}'}</code> - Adresse</div>
                  <div><code className="text-blue-400">{'{firma_plz}'}</code> - PLZ</div>
                  <div><code className="text-blue-400">{'{firma_ort}'}</code> - Ort</div>
                  <div><code className="text-blue-400">{'{firma_telefon}'}</code> - Telefon</div>
                  <div><code className="text-blue-400">{'{firma_email}'}</code> - Email</div>
                  <div><code className="text-blue-400">{'{firma_website}'}</code> - Website</div>
                  <div><code className="text-blue-400">{'{firma_steuernummer}'}</code> - Steuernr.</div>
                  <div><code className="text-blue-400">{'{firma_iban}'}</code> - IBAN <span className="text-orange-400">✓</span></div>
                  <div><code className="text-blue-400">{'{firma_bic}'}</code> - BIC <span className="text-orange-400">✓</span></div>
                  <div><code className="text-blue-400">{'{firma_kontoinhaber}'}</code> - Kontoinhaber <span className="text-orange-400">NEU</span></div>
                </div>
              </div>

              {/* Datum-Platzhalter */}
              <div>
                <p className="text-xs font-semibold text-pink-400 mb-1.5">📆 Datum-Platzhalter:</p>
                <div className="grid grid-cols-2 gap-1.5 text-xs text-slate-400 ml-2">
                  <div><code className="text-blue-400">{'{heute}'}</code> - Heutiges Datum</div>
                  <div><code className="text-blue-400">{'{jetzt}'}</code> - Datum + Uhrzeit</div>
                </div>
              </div>
            </div>

            {/* Success/Error Messages */}
            {successMessage && (
              <div className="bg-emerald-500/10 border border-emerald-500/30 rounded-lg p-4">
                <p className="text-sm text-emerald-300">{successMessage}</p>
              </div>
            )}

            {error && (
              <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4">
                <p className="text-sm text-red-300">{error}</p>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex justify-end gap-3 pt-4">
              <button
                type="button"
                onClick={handleReset}
                className="px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center gap-2"
              >
                <RotateCcw className="w-4 h-4" />
                Zurücksetzen
              </button>
              <button
                type="button"
                onClick={handleSave}
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
          </>
        ) : (
          <div className="flex items-center justify-center h-64">
            <p className="text-slate-400">Wählen Sie ein Template aus der Liste</p>
          </div>
        )}
      </div>
    </div>
  );
}
