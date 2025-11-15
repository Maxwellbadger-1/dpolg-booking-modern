import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Building2, Plus, Pencil, Trash2, User } from 'lucide-react';
import type { PaymentRecipient } from '../../types/booking';
import PaymentRecipientDialog from './PaymentRecipientDialog';

export default function PaymentRecipientsTab() {
  const [recipients, setRecipients] = useState<PaymentRecipient[]>([]);
  const [loading, setLoading] = useState(true);
  const [showDialog, setShowDialog] = useState(false);
  const [editingRecipient, setEditingRecipient] = useState<PaymentRecipient | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<PaymentRecipient | null>(null);

  useEffect(() => {
    loadRecipients();
  }, []);

  const loadRecipients = async () => {
    setLoading(true);
    try {
      const data = await invoke<PaymentRecipient[]>('get_payment_recipients');
      setRecipients(data);
    } catch (error) {
      console.error('Fehler beim Laden der Rechnungsempfänger:', error);
      alert(`Fehler beim Laden: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleAdd = () => {
    setEditingRecipient(null);
    setShowDialog(true);
  };

  const handleEdit = (recipient: PaymentRecipient) => {
    setEditingRecipient(recipient);
    setShowDialog(true);
  };

  const handleDelete = async (recipient: PaymentRecipient) => {
    try {
      await invoke('delete_payment_recipient_pg', { id: recipient.id });
      // Optimistic Update
      setRecipients(prev => prev.filter(r => r.id !== recipient.id));
      setShowDeleteConfirm(null);
      window.dispatchEvent(new CustomEvent('refresh-data'));
    } catch (error) {
      console.error('Fehler beim Löschen:', error);
      alert(`Fehler beim Löschen: ${error}`);
      // Bei Fehler reload
      loadRecipients();
    }
  };

  const handleDialogClose = (saved: boolean) => {
    setShowDialog(false);
    setEditingRecipient(null);
    if (saved) {
      loadRecipients(); // Reload after save
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-3 bg-blue-600 rounded-lg">
            <Building2 className="w-6 h-6 text-white" />
          </div>
          <div>
            <h2 className="text-2xl font-bold text-white">Rechnungsempfänger</h2>
            <p className="text-slate-400 text-sm">
              Externe Zahlungsempfänger verwalten (z.B. Polizeidienststellen, Behörden)
            </p>
          </div>
        </div>
        <button
          onClick={handleAdd}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          Empfänger hinzufügen
        </button>
      </div>

      {/* Recipient List */}
      <div className="bg-slate-800 rounded-xl p-6 space-y-4">
        {recipients.length === 0 ? (
          <div className="text-center py-12">
            <Building2 className="w-16 h-16 text-slate-600 mx-auto mb-4" />
            <p className="text-slate-400 text-lg mb-2">Keine Rechnungsempfänger vorhanden</p>
            <p className="text-slate-500 text-sm mb-4">
              Erstellen Sie Rechnungsempfänger, die bei Buchungen ausgewählt werden können.
            </p>
            <button
              onClick={handleAdd}
              className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            >
              <Plus className="w-4 h-4" />
              Ersten Empfänger erstellen
            </button>
          </div>
        ) : (
          <div className="space-y-3">
            {recipients.map((recipient) => (
              <div
                key={recipient.id}
                className="bg-slate-700 rounded-lg p-4 hover:bg-slate-600 transition-colors"
              >
                <div className="flex items-start justify-between">
                  <div className="flex gap-4 flex-1">
                    <div className="p-3 bg-slate-800 rounded-lg">
                      <User className="w-5 h-5 text-blue-400" />
                    </div>
                    <div className="flex-1">
                      <h3 className="text-white font-semibold text-lg">{recipient.name}</h3>
                      {recipient.company && (
                        <p className="text-slate-300 text-sm">{recipient.company}</p>
                      )}
                      {(recipient.street || recipient.plz || recipient.city) && (
                        <p className="text-slate-400 text-sm mt-1">
                          {recipient.street && <span>{recipient.street}, </span>}
                          {recipient.plz && <span>{recipient.plz} </span>}
                          {recipient.city && <span>{recipient.city}</span>}
                          {recipient.country && recipient.country !== 'Deutschland' && (
                            <span>, {recipient.country}</span>
                          )}
                        </p>
                      )}
                      {recipient.contact_person && (
                        <p className="text-slate-400 text-sm mt-1">
                          <span className="text-slate-500">Ansprechpartner:</span> {recipient.contact_person}
                        </p>
                      )}
                      {recipient.notes && (
                        <p className="text-slate-500 text-sm mt-2 italic">{recipient.notes}</p>
                      )}
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleEdit(recipient)}
                      className="p-2 hover:bg-slate-500 rounded-lg transition-colors group"
                      title="Bearbeiten"
                    >
                      <Pencil className="w-4 h-4 text-slate-400 group-hover:text-white" />
                    </button>
                    <button
                      onClick={() => setShowDeleteConfirm(recipient)}
                      className="p-2 hover:bg-red-500 rounded-lg transition-colors group"
                      title="Löschen"
                    >
                      <Trash2 className="w-4 h-4 text-slate-400 group-hover:text-white" />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Delete Confirmation Dialog */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-red-500/10 rounded-full">
                <Trash2 className="w-6 h-6 text-red-400" />
              </div>
              <div className="flex-1">
                <h3 className="text-xl font-bold text-white mb-2">Empfänger löschen?</h3>
                <p className="text-slate-300 text-sm">
                  Möchten Sie den Rechnungsempfänger "{showDeleteConfirm.name}" wirklich löschen?
                </p>
                {showDeleteConfirm.company && (
                  <p className="text-slate-400 text-sm mt-1">{showDeleteConfirm.company}</p>
                )}
                <div className="mt-4 p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg">
                  <p className="text-amber-300 text-xs">
                    Hinweis: Empfänger können nicht gelöscht werden, wenn sie in Buchungen verwendet werden.
                  </p>
                </div>
              </div>
            </div>
            <div className="flex gap-3">
              <button
                onClick={() => setShowDeleteConfirm(null)}
                className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
              >
                Abbrechen
              </button>
              <button
                onClick={() => handleDelete(showDeleteConfirm)}
                className="flex-1 px-4 py-3 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors"
              >
                Löschen
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Edit/Add Dialog */}
      <PaymentRecipientDialog
        isOpen={showDialog}
        onClose={handleDialogClose}
        recipient={editingRecipient}
      />
    </div>
  );
}
