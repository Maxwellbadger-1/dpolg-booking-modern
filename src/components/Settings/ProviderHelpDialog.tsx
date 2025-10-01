import { X, CheckCircle, ExternalLink } from 'lucide-react';
import { EmailProvider } from '../../lib/emailProviders';

interface ProviderHelpDialogProps {
  provider: EmailProvider;
  isOpen: boolean;
  onClose: () => void;
  onApply: (provider: EmailProvider) => void;
}

export default function ProviderHelpDialog({ provider, isOpen, onClose, onApply }: ProviderHelpDialogProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[60]">
      <div className="bg-slate-800 rounded-2xl shadow-2xl p-6 max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto border border-slate-700">
        {/* Header */}
        <div className="flex items-start justify-between mb-6">
          <div className="flex items-center gap-4">
            <div className="bg-white rounded-lg p-3 shadow-md">
              <img src={provider.logo} alt={provider.name} className="h-10 w-auto" />
            </div>
            <div>
              <h2 className="text-2xl font-bold text-white">{provider.name}</h2>
              <p className="text-sm text-slate-400">SMTP Konfiguration</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-slate-300" />
          </button>
        </div>

        {/* Description */}
        <div className="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4 mb-6">
          <p className="text-sm text-blue-300">{provider.description}</p>
        </div>

        {/* SMTP Settings */}
        <div className="bg-slate-700/50 rounded-lg p-5 mb-6">
          <h3 className="text-sm font-bold text-white mb-3">📋 SMTP-Einstellungen</h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-xs text-slate-400">SMTP Server</p>
              <p className="text-sm font-mono text-white mt-1">{provider.smtp_server}</p>
            </div>
            <div>
              <p className="text-xs text-slate-400">Port</p>
              <p className="text-sm font-mono text-white mt-1">{provider.smtp_port}</p>
            </div>
            <div>
              <p className="text-xs text-slate-400">TLS/SSL</p>
              <p className="text-sm font-semibold text-emerald-400 mt-1">
                {provider.use_tls ? 'Aktiviert' : 'Deaktiviert'}
              </p>
            </div>
            <div>
              <p className="text-xs text-slate-400">Authentifizierung</p>
              <p className="text-sm font-semibold text-emerald-400 mt-1">Erforderlich</p>
            </div>
          </div>
        </div>

        {/* Step by Step Guide */}
        <div className="mb-6">
          <h3 className="text-sm font-bold text-white mb-4">🔧 Schritt-für-Schritt Anleitung</h3>
          <ol className="space-y-3">
            {provider.steps.map((step, index) => (
              <li key={index} className="flex gap-3">
                <div className="flex-shrink-0 w-7 h-7 bg-blue-600 rounded-full flex items-center justify-center text-white text-sm font-bold">
                  {index + 1}
                </div>
                <p className="text-sm text-slate-300 pt-1">{step}</p>
              </li>
            ))}
          </ol>
        </div>

        {/* Help Link */}
        {provider.helpUrl && (
          <div className="bg-slate-700/30 border border-slate-600 rounded-lg p-4 mb-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-semibold text-white">Weitere Hilfe benötigt?</p>
                <p className="text-xs text-slate-400 mt-1">Offizielle Dokumentation des Anbieters</p>
              </div>
              <a
                href={provider.helpUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-2 px-4 py-2 bg-slate-600 hover:bg-slate-500 text-white rounded-lg text-sm font-medium transition-colors"
              >
                Hilfe öffnen
                <ExternalLink className="w-4 h-4" />
              </a>
            </div>
          </div>
        )}

        {/* Footer */}
        <div className="flex justify-end gap-3 pt-4 border-t border-slate-700">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-semibold transition-colors"
          >
            Schließen
          </button>
          <button
            onClick={() => {
              onApply(provider);
              onClose();
            }}
            className="flex items-center gap-2 px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-colors"
          >
            <CheckCircle className="w-4 h-4" />
            Einstellungen übernehmen
          </button>
        </div>
      </div>
    </div>
  );
}
