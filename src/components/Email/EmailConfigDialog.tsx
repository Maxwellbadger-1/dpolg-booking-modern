import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Mail, Server, Lock, User, Shield, CheckCircle, AlertCircle } from 'lucide-react';

interface EmailConfigDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

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

export default function EmailConfigDialog({ isOpen, onClose }: EmailConfigDialogProps) {
  const [smtpServer, setSmtpServer] = useState('');
  const [smtpPort, setSmtpPort] = useState(587);
  const [smtpUsername, setSmtpUsername] = useState('');
  const [smtpPassword, setSmtpPassword] = useState('');
  const [fromEmail, setFromEmail] = useState('');
  const [fromName, setFromName] = useState('');
  const [useTls, setUseTls] = useState(true);
  const [testRecipient, setTestRecipient] = useState('');
  const [loading, setLoading] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadConfig();
    }
  }, [isOpen]);

  const loadConfig = async () => {
    try {
      const config = await invoke<EmailConfig>('get_email_config_command');
      setSmtpServer(config.smtp_server);
      setSmtpPort(config.smtp_port);
      setSmtpUsername(config.smtp_username);
      setSmtpPassword(config.smtp_password);
      setFromEmail(config.from_email);
      setFromName(config.from_name);
      setUseTls(config.use_tls);
    } catch (error) {
      // Konfiguration noch nicht vorhanden - ist ok
      console.log('Keine Email-Konfiguration gefunden (wird beim ersten Speichern erstellt)');
    }
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      await invoke('save_email_config_command', {
        smtpServer,
        smtpPort,
        smtpUsername,
        smtpPassword,
        fromEmail,
        fromName,
        useTls,
      });

      alert('Email-Konfiguration erfolgreich gespeichert!');
      onClose();
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  };

  const handleTest = async () => {
    if (!testRecipient) {
      alert('Bitte geben Sie eine Test-Email-Adresse ein');
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
        smtpPassword,
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

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-500 to-blue-600 px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="bg-white/20 p-2 rounded-lg">
              <Mail className="w-6 h-6 text-white" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">Email-Konfiguration</h2>
              <p className="text-sm text-blue-100">SMTP-Einstellungen verwalten</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="text-white/80 hover:text-white transition-colors p-1 rounded-lg hover:bg-white/10"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Content */}
        <form onSubmit={handleSave} className="p-6 overflow-y-auto max-h-[calc(90vh-180px)]">
          <div className="space-y-4">
            {/* SMTP Server */}
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
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
                className="w-full px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* SMTP Port */}
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                SMTP Port *
              </label>
              <input
                type="number"
                required
                value={smtpPort}
                onChange={(e) => setSmtpPort(parseInt(e.target.value))}
                placeholder="587"
                className="w-full px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <p className="text-xs text-slate-500 mt-1">Standard: 587 (TLS) oder 465 (SSL)</p>
            </div>

            {/* Username */}
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
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
                className="w-full px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Password */}
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                <div className="flex items-center gap-2">
                  <Lock className="w-4 h-4" />
                  Passwort *
                </div>
              </label>
              <input
                type="password"
                required
                value={smtpPassword}
                onChange={(e) => setSmtpPassword(e.target.value)}
                placeholder="••••••••"
                className="w-full px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* From Email */}
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                Absender Email *
              </label>
              <input
                type="email"
                required
                value={fromEmail}
                onChange={(e) => setFromEmail(e.target.value)}
                placeholder="noreply@beispiel.de"
                className="w-full px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* From Name */}
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                Absender Name *
              </label>
              <input
                type="text"
                required
                value={fromName}
                onChange={(e) => setFromName(e.target.value)}
                placeholder="DPolG Stiftung Buchungssystem"
                className="w-full px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* TLS */}
            <div className="flex items-center gap-3">
              <input
                type="checkbox"
                id="use_tls"
                checked={useTls}
                onChange={(e) => setUseTls(e.target.checked)}
                className="w-4 h-4 text-blue-600 border-slate-300 rounded focus:ring-blue-500"
              />
              <label htmlFor="use_tls" className="text-sm font-medium text-slate-700 flex items-center gap-2">
                <Shield className="w-4 h-4" />
                TLS verwenden (empfohlen)
              </label>
            </div>

            {/* Test Email Section */}
            <div className="border-t border-slate-200 pt-4 mt-6">
              <h3 className="text-sm font-bold text-slate-800 mb-3">Verbindung testen</h3>
              <div className="space-y-3">
                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-2">
                    Test-Empfänger
                  </label>
                  <input
                    type="email"
                    value={testRecipient}
                    onChange={(e) => setTestRecipient(e.target.value)}
                    placeholder="test@beispiel.de"
                    className="w-full px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <button
                  type="button"
                  onClick={handleTest}
                  disabled={testing}
                  className="w-full px-4 py-3 bg-purple-600 hover:bg-purple-700 disabled:bg-slate-400 text-white rounded-lg font-semibold transition-colors flex items-center justify-center gap-2"
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
                        ? 'bg-emerald-50 border border-emerald-200'
                        : 'bg-red-50 border border-red-200'
                    }`}
                  >
                    {testResult.success ? (
                      <CheckCircle className="w-5 h-5 text-emerald-600 flex-shrink-0 mt-0.5" />
                    ) : (
                      <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
                    )}
                    <div>
                      <p
                        className={`text-sm font-semibold ${
                          testResult.success ? 'text-emerald-800' : 'text-red-800'
                        }`}
                      >
                        {testResult.success ? 'Erfolgreich!' : 'Fehler'}
                      </p>
                      <p
                        className={`text-sm mt-1 ${
                          testResult.success ? 'text-emerald-700' : 'text-red-700'
                        }`}
                      >
                        {testResult.message}
                      </p>
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Error Display */}
            {error && (
              <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                <p className="text-sm text-red-800">{error}</p>
              </div>
            )}
          </div>
        </form>

        {/* Footer */}
        <div className="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="px-6 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-lg font-semibold transition-colors"
          >
            Abbrechen
          </button>
          <button
            onClick={handleSave}
            disabled={loading}
            className="px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-400 text-white rounded-lg font-semibold transition-colors flex items-center gap-2"
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
      </div>
    </div>
  );
}
