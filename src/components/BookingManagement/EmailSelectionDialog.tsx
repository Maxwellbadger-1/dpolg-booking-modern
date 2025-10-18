import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Mail, FileText, Bell, CreditCard, X, CheckCircle, AlertCircle, Loader2 } from 'lucide-react';

interface EmailSelectionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  bookingId: number;
  guestEmail: string;
  istStiftungsfall?: boolean;
}

interface EmailOption {
  id: string;
  name: string;
  description: string;
  icon: React.ElementType;
  command: string;
  color: string;
}

export default function EmailSelectionDialog({ isOpen, onClose, bookingId, guestEmail, istStiftungsfall = false }: EmailSelectionDialogProps) {
  const [selectedEmails, setSelectedEmails] = useState<string[]>([]);
  const [sending, setSending] = useState(false);
  const [results, setResults] = useState<{ id: string; success: boolean; message: string }[]>([]);
  const [showResults, setShowResults] = useState(false);

  const emailOptions: EmailOption[] = [
    {
      id: 'confirmation',
      name: 'Buchungsbestätigung',
      description: 'Bestätigt die Reservierung mit allen Details',
      icon: Mail,
      command: 'send_confirmation_email_command',
      color: 'blue',
    },
    {
      id: 'invoice',
      name: 'Rechnung (mit PDF)',
      description: 'Rechnung als PDF-Anhang versenden',
      icon: FileText,
      command: 'generate_and_send_invoice_command',
      color: 'emerald',
    },
    {
      id: 'reminder',
      name: 'Erinnerung',
      description: 'Erinnerung an bevorstehenden Aufenthalt',
      icon: Bell,
      command: 'send_reminder_email_command',
      color: 'amber',
    },
    {
      id: 'payment',
      name: 'Zahlungserinnerung',
      description: 'Zahlungserinnerung mit offenen Beträgen',
      icon: CreditCard,
      command: 'send_payment_reminder_email_command',
      color: 'purple',
    },
  ];

  const toggleEmail = (id: string) => {
    setSelectedEmails(prev =>
      prev.includes(id) ? prev.filter(e => e !== id) : [...prev, id]
    );
  };

  const handleSend = async () => {
    if (selectedEmails.length === 0) {
      onClose();
      return;
    }

    // ✅ INSTANT CLOSE - Dialog schließt sofort, Emails im Hintergrund
    const emailsToSend = [...selectedEmails];
    const emailCount = emailsToSend.length;

    // Close dialog immediately
    handleClose();

    // ✅ LOADING TOAST - bleibt offen bis fertig
    const toastId = `email-batch-${bookingId}`;
    const toastMessage = emailCount === 1
      ? '📧 Email wird versendet...'
      : `📧 ${emailCount} Emails werden versendet...`;

    window.dispatchEvent(new CustomEvent('show-toast', {
      detail: {
        id: toastId,
        message: toastMessage,
        type: 'loading'
      }
    }));

    // ✅ ASYNC BACKGROUND PROCESS - Emails versenden ohne UI zu blockieren
    const sendEmailsInBackground = async () => {
      let successCount = 0;
      let failCount = 0;
      let hasInvoiceEmail = false;

      for (const emailId of emailsToSend) {
        const option = emailOptions.find(o => o.id === emailId);
        if (!option) continue;

        try {
          await invoke(option.command, { bookingId });
          successCount++;

          // Track if invoice email was sent
          if (emailId === 'invoice') {
            hasInvoiceEmail = true;
          }

          // Show individual success toast (optional - can be removed if too many toasts)
          // window.dispatchEvent(new CustomEvent('show-toast', {
          //   detail: {
          //     message: `✅ ${option.name} versendet`,
          //     type: 'success',
          //     duration: 2000
          //   }
          // }));

        } catch (error) {
          failCount++;
          const errorMsg = error instanceof Error ? error.message : String(error);

          // Show error toast for failed emails
          window.dispatchEvent(new CustomEvent('show-toast', {
            detail: {
              message: `❌ ${option.name}: ${errorMsg}`,
              type: 'error',
              duration: 5000
            }
          }));
        }
      }

      // ✅ UPDATE LOADING TOAST mit Final Result
      if (successCount > 0 && failCount === 0) {
        const summaryMessage = successCount === 1
          ? '✅ Email erfolgreich versendet'
          : `✅ ${successCount} Emails erfolgreich versendet`;

        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId, // ← Gleiche ID = Toast wird updated!
            message: summaryMessage,
            type: 'success',
            duration: 3000
          }
        }));
      } else if (successCount > 0 && failCount > 0) {
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId, // ← Gleiche ID = Toast wird updated!
            message: `⚠️ ${successCount} erfolgreich, ${failCount} fehlgeschlagen`,
            type: 'warning',
            duration: 4000
          }
        }));
      } else if (failCount > 0) {
        // Alle fehlgeschlagen
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId, // ← Gleiche ID = Toast wird updated!
            message: `❌ ${failCount} Email${failCount > 1 ? 's' : ''} fehlgeschlagen`,
            type: 'error',
            duration: 5000
          }
        }));
      }

      // Trigger data refresh if invoice email was successfully sent
      // This ensures the rechnung_versendet_am field and emoji are updated in the UI
      if (hasInvoiceEmail) {
        window.dispatchEvent(new CustomEvent('refresh-invoice-status', {
          detail: { bookingId }
        }));
      }
    };

    // Start background process (no await!)
    sendEmailsInBackground();
  };

  const handleClose = () => {
    setSelectedEmails([]);
    setResults([]);
    setShowResults(false);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl max-w-2xl w-full border border-slate-700">
        {/* Header */}
        <div className="px-6 py-4 border-b border-slate-700 flex items-center justify-between">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2">
              <Mail className="w-6 h-6 text-blue-400" />
              Emails versenden
            </h2>
            <p className="text-sm text-slate-400 mt-1">
              An: {guestEmail}
            </p>
          </div>
          <button
            onClick={handleClose}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors text-slate-400 hover:text-white"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          {!showResults ? (
            <>
              <p className="text-slate-300 mb-4">
                Wählen Sie aus, welche Emails versendet werden sollen:
              </p>

              {/* Email Options */}
              <div className="space-y-3 mb-6">
                {emailOptions.map((option) => {
                  const Icon = option.icon;
                  const isSelected = selectedEmails.includes(option.id);

                  return (
                    <button
                      key={option.id}
                      onClick={() => toggleEmail(option.id)}
                      disabled={sending || (istStiftungsfall && option.id === 'invoice')}
                      className={`w-full p-4 rounded-lg border-2 transition-all text-left ${
                        isSelected
                          ? `bg-${option.color}-500/20 border-${option.color}-500`
                          : 'bg-slate-700/30 border-slate-600 hover:border-slate-500'
                      } ${(sending || (istStiftungsfall && option.id === 'invoice')) ? 'opacity-50 cursor-not-allowed' : ''}`}
                    >
                      <div className="flex items-start gap-3">
                        <div className={`p-2 rounded-lg ${
                          isSelected ? `bg-${option.color}-500/30` : 'bg-slate-600/50'
                        }`}>
                          <Icon className={`w-5 h-5 ${
                            isSelected ? `text-${option.color}-400` : 'text-slate-400'
                          }`} />
                        </div>
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <h3 className={`font-semibold ${
                              isSelected ? 'text-white' : 'text-slate-300'
                            }`}>
                              {option.name}
                            </h3>
                            {isSelected && (
                              <CheckCircle className={`w-4 h-4 text-${option.color}-400`} />
                            )}
                          </div>
                          <p className="text-sm text-slate-400">
                            {option.description}
                          </p>
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>

              {/* Info Box */}
              <div className="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4 mb-6">
                <p className="text-sm text-blue-300">
                  💡 <strong>Tipp:</strong> Buchungsbestätigung und Rechnung sind separate Emails.
                  Sie können auch später über die Buchungsdetails weitere Emails versenden.
                </p>
              </div>
            </>
          ) : (
            /* Results */
            <div className="space-y-3 mb-6">
              <h3 className="text-lg font-semibold text-white mb-3">Versand-Ergebnisse:</h3>
              {results.map((result) => {
                const option = emailOptions.find(o => o.id === result.id);
                return (
                  <div
                    key={result.id}
                    className={`p-4 rounded-lg border ${
                      result.success
                        ? 'bg-emerald-500/10 border-emerald-500/30'
                        : 'bg-red-500/10 border-red-500/30'
                    }`}
                  >
                    <div className="flex items-start gap-3">
                      {result.success ? (
                        <CheckCircle className="w-5 h-5 text-emerald-400 flex-shrink-0 mt-0.5" />
                      ) : (
                        <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
                      )}
                      <div>
                        <p className={`font-semibold ${
                          result.success ? 'text-emerald-300' : 'text-red-300'
                        }`}>
                          {option?.name}
                        </p>
                        <p className={`text-sm mt-1 ${
                          result.success ? 'text-emerald-400' : 'text-red-400'
                        }`}>
                          {result.message}
                        </p>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-slate-700 flex justify-between">
          <button
            onClick={handleClose}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-semibold transition-colors"
          >
            {showResults ? 'Schließen' : 'Überspringen'}
          </button>
          {!showResults && (
            <button
              onClick={handleSend}
              disabled={sending || selectedEmails.length === 0}
              className="px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center gap-2"
            >
              {sending ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Sende {selectedEmails.length} Email{selectedEmails.length !== 1 ? 's' : ''}...
                </>
              ) : (
                <>
                  <Mail className="w-4 h-4" />
                  {selectedEmails.length > 0
                    ? `${selectedEmails.length} Email${selectedEmails.length !== 1 ? 's' : ''} senden`
                    : 'Email auswählen'}
                </>
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
