import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Server, User, Lock, Shield, Mail, CheckCircle, AlertCircle, HelpCircle, Eye, EyeOff } from 'lucide-react';
import { emailProviders, EmailProvider } from '../../lib/emailProviders';
import ProviderHelpDialog from './ProviderHelpDialog';

interface EmailConfig {
  id: number;
  smtp_server: string;
  smtp_port: number;
  smtp_username: string;
  smtp_password: string;
  from_email: string;
  from_name: string;
  use_tls: boolean;
  created_at: string;
  updated_at: string;
}

export default function EmailConfigTab() {
  const [smtpServer, setSmtpServer] = useState('');
  const [smtpPort, setSmtpPort] = useState(587);
  const [smtpUsername, setSmtpUsername] = useState('');
  const [smtpPassword, setSmtpPassword] = useState('');
  const [fromEmail, setFromEmail] = useState('');
  const [fromName, setFromName] = useState('');
  const [useTls, setUseTls] = useState(true);
  const [showPassword, setShowPassword] = useState(false);
  const [hasExistingPassword, setHasExistingPassword] = useState(false);
  const [testRecipient, setTestRecipient] = useState('');
  const [loading, setLoading] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [selectedProvider, setSelectedProvider] = useState<EmailProvider | null>(null);
  const [showProviderDialog, setShowProviderDialog] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const config = await invoke<EmailConfig>('get_email_config_command');
      setSmtpServer(config.smtp_server);
      setSmtpPort(config.smtp_port);
      setSmtpUsername(config.smtp_username);
      // Tracke ob ein Passwort existiert, zeige es aber nicht im Feld an
      if (config.smtp_password && config.smtp_password.length > 0) {
        setHasExistingPassword(true);
        setSmtpPassword(''); // Feld bleibt leer
      }
      setFromEmail(config.from_email);
      setFromName(config.from_name);
      setUseTls(config.use_tls);
    } catch (error) {
      console.log('Keine Email-Konfiguration gefunden');
      setHasExistingPassword(false);
    }
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccessMessage(null);

    try {
      // Backend behandelt leeres Passwort korrekt:
      // - Bei Update: behält altes Passwort
      // - Bei Insert: gibt Fehler zurück
      await invoke('save_email_config_command', {
        smtpServer,
        smtpPort,
        smtpUsername,
        smtpPassword: smtpPassword, // Sende was im Feld steht (leer oder neu)
        fromEmail,
        fromName,
        useTls,
      });

      setSuccessMessage('Email-Konfiguration erfolgreich gespeichert!');
      setTimeout(() => setSuccessMessage(null), 3000);

      // Neu laden um aktuellen Status zu zeigen
      await loadConfig();
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  };

  const handleProviderClick = (provider: EmailProvider) => {
    setSelectedProvider(provider);
    setShowProviderDialog(true);
  };

  const handleApplyProvider = (provider: EmailProvider) => {
    setSmtpServer(provider.smtp_server);
    setSmtpPort(provider.smtp_port);
    setUseTls(provider.use_tls);
    setSuccessMessage(`${provider.name} Einstellungen wurden übernommen. Bitte geben Sie noch Ihren Benutzernamen und Passwort ein.`);
    setTimeout(() => setSuccessMessage(null), 5000);
  };

  const handleTest = async () => {
    if (!testRecipient) {
      alert('Bitte geben Sie eine Test-Email-Adresse ein');
      return;
    }

    // Wenn kein Passwort eingegeben wurde und keine Config existiert
    if (!hasExistingPassword && smtpPassword.length === 0) {
      alert('Bitte geben Sie erst ein Passwort ein');
      return;
    }

    setTesting(true);
    setTestResult(null);
    setError(null);

    try {
      const result = await invoke<string>('test_email_connection_command', {
        smtpServer,
        smtpPort,
        smtpUsername,
        smtpPassword: smtpPassword, // Backend lädt gespeichertes PW wenn leer
        fromEmail,
        fromName,
        testRecipient,
      });

      setTestResult({ success: true, message: result });
    } catch (error) {
      setTestResult({
        success: false,
        message: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setTesting(false);
    }
  };

  // Detect active provider
  const activeProvider = emailProviders.find(p => p.smtp_server === smtpServer);

  return (
    <form onSubmit={handleSave} className="space-y-6">
      {/* Email Provider Selection - Top */}
      <div>
        <div className="flex items-center gap-2 mb-3">
          <HelpCircle className="w-5 h-5 text-blue-400" />
          <h3 className="text-sm font-bold text-white">Beliebte Email-Anbieter</h3>
        </div>
        <p className="text-xs text-slate-400 mb-4">
          Klicken Sie auf einen Anbieter für eine Schritt-für-Schritt-Anleitung
        </p>
        <div className="flex gap-3 overflow-x-auto pb-2 pt-2">
          {emailProviders.map((provider) => {
            const isActive = activeProvider?.id === provider.id;
            return (
              <button
                key={provider.id}
                type="button"
                onClick={() => handleProviderClick(provider)}
                className={`flex-shrink-0 w-36 p-4 rounded-lg transition-all relative ${
                  isActive
                    ? 'bg-emerald-900/30 border-2 border-emerald-500 shadow-lg shadow-emerald-500/20'
                    : 'bg-slate-700 border border-slate-600 hover:border-blue-500'
                } group`}
              >
                {isActive && (
                  <div className="absolute -top-1 -right-1 bg-emerald-500 rounded-full p-1">
                    <CheckCircle className="w-4 h-4 text-white" />
                  </div>
                )}
                <div className="bg-white rounded-md p-2 mb-3 flex items-center justify-center h-12">
                  <img src={provider.logo} alt={provider.name} className="max-h-8 w-auto" />
                </div>
                <p className={`text-xs font-medium text-center mb-1 ${
                  isActive ? 'text-emerald-300' : 'text-slate-300 group-hover:text-white'
                }`}>
                  {provider.name}
                </p>
                {isActive && (
                  <p className="text-[10px] text-emerald-400 text-center font-semibold">
                    ✓ Aktiv
                  </p>
                )}
              </button>
            );
          })}
        </div>
      </div>

      {/* SMTP Server */}
      <div>
        <label className="block text-sm font-medium text-slate-300 mb-2">
          <div className="flex items-center gap-2">
            <Server className="w-4 h-4" />
            SMTP Server *
          </div>
        </label>
        <input
          type="text"
          required
          value={smtpServer}
          onChange={(e) => setSmtpServer(e.target.value)}
          placeholder="smtp.gmail.com"
          className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* SMTP Port */}
      <div>
        <label className="block text-sm font-medium text-slate-300 mb-2">
          SMTP Port *
        </label>
        <input
          type="number"
          required
          value={smtpPort}
          onChange={(e) => setSmtpPort(parseInt(e.target.value))}
          placeholder="587"
          className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <p className="text-xs text-slate-400 mt-1">Standard: 587 (TLS) oder 465 (SSL)</p>
      </div>

      {/* Username */}
      <div>
        <label className="block text-sm font-medium text-slate-300 mb-2">
          <div className="flex items-center gap-2">
            <User className="w-4 h-4" />
            Benutzername *
          </div>
        </label>
        <input
          type="text"
          required
          value={smtpUsername}
          onChange={(e) => setSmtpUsername(e.target.value)}
          placeholder="ihr-email@beispiel.de"
          className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* Password */}
      <div>
        <label className="block text-sm font-medium text-slate-300 mb-2">
          <div className="flex items-center gap-2">
            <Lock className="w-4 h-4" />
            Passwort *
            {hasExistingPassword && smtpPassword.length === 0 && (
              <span className="text-xs text-emerald-400 flex items-center gap-1">
                <CheckCircle className="w-3 h-3" />
                Gespeichert
              </span>
            )}
          </div>
        </label>
        <div className="relative">
          <input
            type={showPassword ? "text" : "password"}
            required={!hasExistingPassword}
            value={smtpPassword}
            onChange={(e) => setSmtpPassword(e.target.value)}
            placeholder={hasExistingPassword ? "Leer lassen um bestehendes Passwort zu behalten" : "••••••••"}
            className="w-full px-4 py-3 pr-12 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white transition-colors"
          >
            {showPassword ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
          </button>
        </div>
        {hasExistingPassword && (
          <p className="text-xs text-slate-400 mt-1">
            Nur ausfüllen wenn Sie das Passwort ändern möchten
          </p>
        )}
      </div>

      {/* From Email */}
      <div>
        <label className="block text-sm font-medium text-slate-300 mb-2">
          Absender Email *
        </label>
        <input
          type="email"
          required
          value={fromEmail}
          onChange={(e) => setFromEmail(e.target.value)}
          placeholder="noreply@beispiel.de"
          className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* From Name */}
      <div>
        <label className="block text-sm font-medium text-slate-300 mb-2">
          Absender Name *
        </label>
        <input
          type="text"
          required
          value={fromName}
          onChange={(e) => setFromName(e.target.value)}
          placeholder="DPolG Stiftung Buchungssystem"
          className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* TLS */}
      <div className="flex items-center gap-3">
        <input
          type="checkbox"
          id="use_tls"
          checked={useTls}
          onChange={(e) => setUseTls(e.target.checked)}
          className="w-4 h-4 text-blue-600 border-slate-600 rounded focus:ring-blue-500"
        />
        <label htmlFor="use_tls" className="text-sm font-medium text-slate-300 flex items-center gap-2">
          <Shield className="w-4 h-4" />
          TLS verwenden (empfohlen)
        </label>
      </div>

      {/* Test Email Section */}
      <div className="border-t border-slate-700 pt-6 mt-6">
        <h3 className="text-sm font-bold text-white mb-3">Verbindung testen</h3>
        <div className="space-y-3">
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Test-Empfänger
            </label>
            <input
              type="email"
              value={testRecipient}
              onChange={(e) => setTestRecipient(e.target.value)}
              placeholder="test@beispiel.de"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <button
            type="button"
            onClick={handleTest}
            disabled={testing}
            className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 disabled:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center justify-center gap-2"
          >
            {testing ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                Sende Test-Email...
              </>
            ) : (
              <>
                <Mail className="w-4 h-4" />
                Test-Email senden
              </>
            )}
          </button>

          {/* Test Result */}
          {testResult && (
            <div
              className={`p-4 rounded-lg flex items-start gap-3 ${
                testResult.success
                  ? 'bg-emerald-500/10 border border-emerald-500/30'
                  : 'bg-red-500/10 border border-red-500/30'
              }`}
            >
              {testResult.success ? (
                <CheckCircle className="w-5 h-5 text-emerald-400 flex-shrink-0 mt-0.5" />
              ) : (
                <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
              )}
              <div>
                <p className={`text-sm font-semibold ${testResult.success ? 'text-emerald-300' : 'text-red-300'}`}>
                  {testResult.success ? 'Erfolgreich!' : 'Fehler'}
                </p>
                <p className={`text-sm mt-1 ${testResult.success ? 'text-emerald-400' : 'text-red-400'}`}>
                  {testResult.message}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4">
          <p className="text-sm text-red-300">{error}</p>
        </div>
      )}

      {/* Success Display */}
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
            'Speichern'
          )}
        </button>
      </div>

      {/* Provider Help Dialog */}
      {selectedProvider && (
        <ProviderHelpDialog
          provider={selectedProvider}
          isOpen={showProviderDialog}
          onClose={() => setShowProviderDialog(false)}
          onApply={handleApplyProvider}
        />
      )}
    </form>
  );
}
