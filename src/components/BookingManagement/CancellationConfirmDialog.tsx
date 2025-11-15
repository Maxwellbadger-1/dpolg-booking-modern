import { useState } from 'react';
import { X, Mail, AlertTriangle } from 'lucide-react';

interface CancellationConfirmDialogProps {
  isOpen: boolean;
  reservierungsnummer: string;
  onConfirm: (sendEmail: boolean) => void;
  onCancel: () => void;
}

export default function CancellationConfirmDialog({
  isOpen,
  reservierungsnummer,
  onConfirm,
  onCancel,
}: CancellationConfirmDialogProps) {
  const [sendEmail, setSendEmail] = useState(false);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 max-w-md w-full mx-4">
        {/* Header */}
        <div className="flex items-start justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-amber-500/20 rounded-lg">
              <AlertTriangle className="w-6 h-6 text-amber-500" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">Buchung stornieren</h2>
              <p className="text-sm text-slate-400 mt-1">Reservierung {reservierungsnummer}</p>
            </div>
          </div>
          <button
            onClick={onCancel}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
            aria-label="Schließen"
          >
            <X className="w-5 h-5 text-slate-300" />
          </button>
        </div>

        {/* Content */}
        <div className="mb-6">
          <p className="text-slate-300 mb-4">
            Möchten Sie diese Buchung wirklich stornieren?
          </p>

          {/* Email Checkbox */}
          <label className="flex items-start gap-3 p-4 bg-slate-700/50 rounded-lg cursor-pointer hover:bg-slate-700 transition-colors">
            <input
              type="checkbox"
              checked={sendEmail}
              onChange={(e) => setSendEmail(e.target.checked)}
              className="mt-0.5 w-5 h-5 text-blue-600 bg-slate-600 border-slate-500 rounded focus:ring-2 focus:ring-blue-500 focus:ring-offset-0"
            />
            <div className="flex-1">
              <div className="flex items-center gap-2 text-white font-medium mb-1">
                <Mail className="w-4 h-4" />
                Stornierungsbestätigung per E-Mail senden
              </div>
              <p className="text-sm text-slate-400">
                Der Gast erhält automatisch eine E-Mail mit der Stornierungsbestätigung.
              </p>
            </div>
          </label>
        </div>

        {/* Actions */}
        <div className="flex gap-3">
          <button
            onClick={() => onConfirm(sendEmail)}
            className="flex-1 px-4 py-3 bg-amber-600 hover:bg-amber-700 text-white font-semibold rounded-lg transition-colors"
          >
            Ja, stornieren
          </button>
          <button
            onClick={onCancel}
            className="px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
          >
            Abbrechen
          </button>
        </div>
      </div>
    </div>
  );
}
