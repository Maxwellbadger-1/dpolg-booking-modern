import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { check } from '@tauri-apps/plugin-updater';
import { getVersion } from '@tauri-apps/api/app';
import { relaunch } from '@tauri-apps/plugin-process';
import { Building2, Save, Upload, XCircle, Download, CheckCircle, Info, Code } from 'lucide-react';

interface CompanySettings {
  id: number;
  companyName: string;
  streetAddress: string | null;
  plz: string | null;
  city: string | null;
  country: string | null;
  phone: string | null;
  fax: string | null;
  email: string | null;
  website: string | null;
  taxId: string | null;
  ceoName: string | null;
  registryCourt: string | null;
  logoPath: string | null;
  logoData: string | null;
  logoMimeType: string | null;
  updatedAt: string | null;
}

interface LogoUploadResult {
  logoData: string;
  logoMimeType: string;
  fileName: string;
}

export default function GeneralSettingsTab() {
  const [settings, setSettings] = useState<CompanySettings | null>(null);
  const [companyName, setCompanyName] = useState('');
  const [address, setAddress] = useState('');
  const [plz, setPlz] = useState('');
  const [city, setCity] = useState('');
  const [country, setCountry] = useState('Deutschland');
  const [phone, setPhone] = useState('');
  const [fax, setFax] = useState('');
  const [email, setEmail] = useState('');
  const [website, setWebsite] = useState('');
  const [taxId, setTaxId] = useState('');
  const [ceoName, setCeoName] = useState('');
  const [registryCourt, setRegistryCourt] = useState('');
  const [logoPath, setLogoPath] = useState('');
  const [logoData, setLogoData] = useState<string | null>(null);
  const [logoMimeType, setLogoMimeType] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [loadingData, setLoadingData] = useState(true);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [updateStatus, setUpdateStatus] = useState<'idle' | 'checking' | 'available' | 'current' | 'error'>('idle');
  const [updateVersion, setUpdateVersion] = useState<string>('');
  const [currentVersion, setCurrentVersion] = useState<string>('...');
  const [devToolsEnabled, setDevToolsEnabled] = useState(() => {
    return localStorage.getItem('devtools-enabled') === 'true';
  });

  useEffect(() => {
    loadSettings();
    loadCurrentVersion();
  }, []);

  const loadSettings = async () => {
    setLoadingData(true);
    try {
      const data = await invoke<CompanySettings>('get_company_settings_pg');
      setSettings(data);

      // Formular befüllen
      setCompanyName(data.companyName);
      setAddress(data.streetAddress || '');
      setPlz(data.plz || '');
      setCity(data.city || '');
      setCountry(data.country || 'Deutschland');
      setPhone(data.phone || '');
      setFax(data.fax || '');
      setEmail(data.email || '');
      setWebsite(data.website || '');
      setTaxId(data.taxId || '');
      setCeoName(data.ceoName || '');
      setRegistryCourt(data.registryCourt || '');
      setLogoPath(data.logoPath || '');
      setLogoData(data.logoData || null);
      setLogoMimeType(data.logoMimeType || null);
    } catch (error) {
      console.error('Fehler beim Laden der Einstellungen:', error);
      alert(`Fehler beim Laden: ${error}`);
    } finally {
      setLoadingData(false);
    }
  };

  const loadCurrentVersion = async () => {
    try {
      const version = await getVersion();
      setCurrentVersion(version);
    } catch (error) {
      console.error('Fehler beim Laden der Version:', error);
      setCurrentVersion('Unbekannt');
    }
  };

  const handleLogoUpload = async () => {
    try {
      const file = await open({
        filters: [{
          name: 'Bilder',
          extensions: ['png', 'jpg', 'jpeg']
        }],
        multiple: false,
        directory: false,
      });

      if (file) {
        // Backend liest Datei und gibt Base64-Daten zurück
        const result = await invoke<LogoUploadResult>('upload_logo_command', {
          sourcePath: file
        });
        setLogoData(result.logoData);
        setLogoMimeType(result.logoMimeType);
        setLogoPath('');
        setSuccessMessage('Logo erfolgreich hochgeladen! Bitte speichern.');
        setTimeout(() => setSuccessMessage(null), 3000);
      }
    } catch (error) {
      console.error('Fehler beim Hochladen:', error);
      alert(`Fehler beim Hochladen: ${error}`);
    }
  };

  const handleLogoDelete = () => {
    setLogoPath('');
    setLogoData(null);
    setLogoMimeType(null);
    setSuccessMessage('Logo entfernt. Bitte speichern Sie die Änderungen.');
    setTimeout(() => setSuccessMessage(null), 3000);
  };

  const checkForUpdate = async () => {
    setCheckingUpdate(true);
    setUpdateStatus('checking');
    setSuccessMessage('🔍 Suche nach Updates...');

    try {
      console.log('🔍 Manueller Update-Check gestartet...');

      // Check if we're in development mode
      const isDev = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';
      if (isDev) {
        console.log('⚠️ Update-Check im Development-Mode übersprungen');
        setUpdateStatus('current');
        setSuccessMessage(`ℹ️ Update-Check ist nur in Production Builds verfügbar (aktuelle Version: ${currentVersion})`);
        setTimeout(() => {
          setSuccessMessage(null);
          setUpdateStatus('idle');
        }, 6000);
        setCheckingUpdate(false);
        return;
      }

      const update = await check();

      if (update?.available) {
        console.log(`✅ Update verfügbar: ${update.version}`);
        setUpdateStatus('available');
        setUpdateVersion(update.version);
        setSuccessMessage(null); // Clear checking message

        // Zeige Update-Dialog
        const shouldUpdate = window.confirm(
          `Update verfügbar: Version ${update.version}\n\n${update.body || 'Neue Version verfügbar'}\n\nJetzt installieren? Die App wird nach der Installation neu gestartet.`
        );

        if (shouldUpdate) {
          setSuccessMessage('⬇️ Update wird heruntergeladen...');

          // Download und Installation
          let downloaded = 0;
          let contentLength = 0;
          await update.downloadAndInstall((event) => {
            switch (event.event) {
              case 'Started':
                contentLength = event.data.contentLength || 0;
                const sizeMB = Math.round(contentLength / 1024 / 1024);
                console.log(`Download gestartet (${sizeMB} MB)`);
                setSuccessMessage(`⬇️ Lade Update herunter (${sizeMB} MB)...`);
                break;
              case 'Progress':
                downloaded += event.data.chunkLength;
                const percent = contentLength > 0 ? Math.round((downloaded / contentLength) * 100) : 0;
                setSuccessMessage(`⬇️ Lade Update herunter... ${percent}%`);
                break;
              case 'Finished':
                setSuccessMessage('✅ Update installiert! App wird neu gestartet...');
                console.log('Update installiert');
                break;
            }
          });

          // App neu starten
          setTimeout(async () => {
            await relaunch();
          }, 2000);
        } else {
          // User hat Update abgelehnt
          setUpdateStatus('idle');
          setSuccessMessage(null);
        }
      } else {
        console.log('✅ App ist auf dem neuesten Stand');
        setUpdateStatus('current');
        setSuccessMessage(`✅ App ist bereits aktuell (Version ${currentVersion})`);
        setTimeout(() => {
          setSuccessMessage(null);
          setUpdateStatus('idle');
        }, 5000); // 5 Sekunden statt 3
      }
    } catch (error) {
      console.error('❌ Fehler beim Update-Check:', error);
      setUpdateStatus('error');
      const errorMessage = error instanceof Error ? error.message : String(error);
      setSuccessMessage(`❌ Fehler beim Update-Check: ${errorMessage}`);
      setTimeout(() => {
        setSuccessMessage(null);
        setUpdateStatus('idle');
      }, 8000); // 8 Sekunden für Error-Messages
    } finally {
      setCheckingUpdate(false);
    }
  };

  const toggleDevTools = (enabled: boolean) => {
    setDevToolsEnabled(enabled);
    localStorage.setItem('devtools-enabled', enabled.toString());
    // Dispatch event to notify App.tsx
    window.dispatchEvent(new CustomEvent('devtools-toggle', { detail: { enabled } }));
    setSuccessMessage(enabled ? '🛠️ DevTools aktiviert' : '🛠️ DevTools deaktiviert');
    setTimeout(() => setSuccessMessage(null), 3000);
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setSuccessMessage(null);

    try {
      const updatedSettings: CompanySettings = {
        id: settings?.id || 1,
        companyName: companyName,
        streetAddress: address || null,
        plz: plz || null,
        city: city || null,
        country: country || null,
        phone: phone || null,
        fax: fax || null,
        email: email || null,
        website: website || null,
        taxId: taxId || null,
        ceoName: ceoName || null,
        registryCourt: registryCourt || null,
        logoPath: logoPath || null,
        logoData: logoData || null,
        logoMimeType: logoMimeType || null,
        updatedAt: null,
      };

      const result = await invoke<CompanySettings>('update_company_settings_pg', {
        settings: updatedSettings,
      });

      setSettings(result);
      setSuccessMessage('Allgemeine Einstellungen erfolgreich gespeichert!');
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
      {/* 🚀 SOFTWARE UPDATES - TOP PRIORITY */}
      <div>
        <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
          <Download className="w-5 h-5 text-blue-400" />
          Software-Updates
        </h3>
        <div className="space-y-4">
          <div className="bg-gradient-to-r from-blue-600/10 to-blue-500/10 border border-blue-500/30 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-slate-300">
                  Aktuelle Version: <span className="text-white font-semibold">{currentVersion}</span>
                </p>
                <p className="text-xs text-slate-400 mt-1">
                  Automatische Update-Prüfung beim App-Start
                </p>
              </div>
              <button
                type="button"
                onClick={checkForUpdate}
                disabled={checkingUpdate}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center gap-2"
              >
                {checkingUpdate ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    Prüfe...
                  </>
                ) : (
                  <>
                    <Download className="w-4 h-4" />
                    Nach Updates suchen
                  </>
                )}
              </button>
            </div>

            {/* Update Status Messages */}
            {updateStatus === 'checking' && (
              <div className="mt-4 flex items-center gap-2 text-blue-400 bg-blue-900/20 p-3 rounded-lg border border-blue-700">
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-400"></div>
                <span className="text-sm font-medium">Prüfe auf Updates...</span>
              </div>
            )}

            {updateStatus === 'current' && (
              <div className="mt-4 flex items-center gap-2 text-emerald-400 bg-emerald-900/20 p-3 rounded-lg border border-emerald-700">
                <CheckCircle className="w-4 h-4" />
                <span className="text-sm font-medium">App ist auf dem neuesten Stand (v{currentVersion})</span>
              </div>
            )}

            {updateStatus === 'available' && (
              <div className="mt-4 flex items-center gap-2 text-amber-400 bg-amber-900/20 p-3 rounded-lg border border-amber-700">
                <Info className="w-4 h-4" />
                <span className="text-sm font-medium">Update verfügbar: Version {updateVersion}</span>
              </div>
            )}

            {updateStatus === 'error' && (
              <div className="mt-4 flex items-center gap-2 text-red-400 bg-red-900/20 p-3 rounded-lg border border-red-700">
                <XCircle className="w-4 h-4" />
                <span className="text-sm font-medium">Fehler beim Update-Check</span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* ENTWICKLER-TOOLS */}
      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-lg font-bold text-white mb-4 flex items-center gap-2">
          <Code className="w-5 h-5 text-purple-400" />
          Entwickler-Tools
        </h3>
        <div className="space-y-4">
          <div className="bg-gradient-to-r from-purple-600/10 to-purple-500/10 border border-purple-500/30 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-slate-300">
                  DevTools anzeigen
                </p>
                <p className="text-xs text-slate-400 mt-1">
                  Zeigt Entwickler-Werkzeuge für Diagnose und Tests
                </p>
              </div>
              <button
                type="button"
                onClick={() => toggleDevTools(!devToolsEnabled)}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                  devToolsEnabled ? 'bg-purple-600' : 'bg-slate-600'
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                    devToolsEnabled ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>
            {devToolsEnabled && (
              <div className="mt-4 flex items-center gap-2 text-purple-400 bg-purple-900/20 p-3 rounded-lg border border-purple-700">
                <Code className="w-4 h-4" />
                <span className="text-sm font-medium">DevTools sind aktiviert - Button erscheint unten rechts</span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* UNTERKUNFTSDATEN */}
      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-lg font-bold text-white mb-4">Unterkunftsdaten</h3>
        <div className="space-y-4">
          {/* Company Name */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              <div className="flex items-center gap-2">
                <Building2 className="w-4 h-4" />
                Name der Unterkunft *
              </div>
            </label>
            <input
              type="text"
              required
              value={companyName}
              onChange={(e) => setCompanyName(e.target.value)}
              placeholder="Stiftung der Deutschen Polizeigewerkschaft"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Address */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Straße & Hausnummer *
            </label>
            <input
              type="text"
              required
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              placeholder="Wackersberger Str. 12"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* PLZ & Ort */}
          <div className="grid grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                PLZ *
              </label>
              <input
                type="text"
                required
                value={plz}
                onChange={(e) => setPlz(e.target.value)}
                placeholder="83661"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Ort *
              </label>
              <input
                type="text"
                required
                value={city}
                onChange={(e) => setCity(e.target.value)}
                placeholder="Lenggries"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Land *
              </label>
              <input
                type="text"
                required
                value={country}
                onChange={(e) => setCountry(e.target.value)}
                placeholder="Deutschland"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          {/* Telefon & Fax */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Telefon *
              </label>
              <input
                type="tel"
                required
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                placeholder="+49 8042 9725-20"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Fax
              </label>
              <input
                type="tel"
                value={fax}
                onChange={(e) => setFax(e.target.value)}
                placeholder="+49 8042 9725-22"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          {/* Email */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Email *
            </label>
            <input
              type="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="info@dpolg-stiftung.de"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Website */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Website *
            </label>
            <input
              type="url"
              required
              value={website}
              onChange={(e) => setWebsite(e.target.value)}
              placeholder="www.dpolg-stiftung.de"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </div>
      </div>

      {/* Rechtliche Daten */}
      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-lg font-bold text-white mb-4">Rechtliche Angaben</h3>
        <div className="space-y-4">
          {/* Tax ID */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Steuernummer *
            </label>
            <input
              type="text"
              required
              value={taxId}
              onChange={(e) => setTaxId(e.target.value)}
              placeholder="141/239/71040"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* CEO Name */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Vorstand / Geschäftsführer *
            </label>
            <input
              type="text"
              required
              value={ceoName}
              onChange={(e) => setCeoName(e.target.value)}
              placeholder="Herr Reinhold Merl"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Registry Court */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Gerichtsstandort *
            </label>
            <input
              type="text"
              required
              value={registryCourt}
              onChange={(e) => setRegistryCourt(e.target.value)}
              placeholder="München"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </div>
      </div>


      {/* Logo Upload */}
      <div className="border-t border-slate-700 pt-6">
        <h3 className="text-lg font-bold text-white mb-4">Logo (für Rechnungen)</h3>
        <div className="space-y-4">
          {/* Logo-Vorschau */}
          {logoData && logoMimeType && (
            <div className="bg-slate-700 rounded-lg p-4 border border-slate-600">
              <p className="text-sm text-slate-300 mb-2">Aktuelles Logo:</p>
              <div className="bg-white p-4 rounded">
                <img
                  src={`data:${logoMimeType};base64,${logoData}`}
                  alt="Firmen-Logo"
                  className="max-h-32 w-auto object-contain mx-auto"
                />
              </div>
            </div>
          )}

          {/* Upload-Button */}
          <button
            type="button"
            onClick={handleLogoUpload}
            className="flex items-center gap-2 px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-colors"
          >
            <Upload className="w-5 h-5" />
            {logoData ? 'Logo ändern' : 'Logo hochladen'}
          </button>

          <p className="text-xs text-slate-400">
            Unterstützte Formate: PNG, JPG, JPEG (max. 2 MB). Das Logo wird oben mittig auf Rechnungen angezeigt.
          </p>

          {/* Logo löschen */}
          {logoData && (
            <button
              type="button"
              onClick={handleLogoDelete}
              className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-semibold transition-colors text-sm"
            >
              <XCircle className="w-4 h-4" />
              Logo entfernen
            </button>
          )}
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
