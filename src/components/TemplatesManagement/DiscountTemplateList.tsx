import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Plus, Pencil, Trash2, Tag, Euro, Percent, CheckCircle, XCircle } from 'lucide-react';
import { DiscountTemplate } from '../../types/booking';
import DiscountTemplateDialog from './DiscountTemplateDialog';
import ConfirmDialog from '../ConfirmDialog';
import { formatDiscountValue, getDiscountIcon } from '../../utils/priceFormatting';

export default function DiscountTemplateList() {
  const [templates, setTemplates] = useState<DiscountTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<DiscountTemplate | undefined>();
  const [deleteDialog, setDeleteDialog] = useState<{ open: boolean; id: number | null }>({
    open: false,
    id: null,
  });

  const loadTemplates = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<DiscountTemplate[]>('get_all_discount_templates_pg');
      setTemplates(data);
    } catch (err) {
      console.error('Fehler beim Laden der Rabatte:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTemplates();
  }, []);

  const handleCreate = () => {
    setSelectedTemplate(undefined);
    setDialogOpen(true);
  };

  const handleEdit = (template: DiscountTemplate) => {
    setSelectedTemplate(template);
    setDialogOpen(true);
  };

  const handleDelete = async () => {
    if (!deleteDialog.id) return;

    try {
      await invoke('delete_discount_template_pg', { id: deleteDialog.id });
      setDeleteDialog({ open: false, id: null });
      loadTemplates();
    } catch (err) {
      console.error('Fehler beim Löschen:', err);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-400">Lädt Rabatte...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-white">Rabatte</h2>
          <p className="text-slate-400 mt-1">Vordefinierte Rabatte für Buchungen</p>
        </div>
        <button
          onClick={handleCreate}
          className="flex items-center gap-2 px-4 py-3 bg-amber-600 hover:bg-amber-700 text-white font-semibold rounded-lg transition-colors"
        >
          <Plus className="w-5 h-5" />
          Neuer Rabatt
        </button>
      </div>

      {error && (
        <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400">
          {error}
        </div>
      )}

      {/* Templates Grid */}
      {templates.length === 0 ? (
        <div className="text-center py-16">
          <Tag className="w-16 h-16 text-slate-600 mx-auto mb-4" />
          <h3 className="text-xl font-semibold text-slate-400 mb-2">
            Keine Rabatte
          </h3>
          <p className="text-slate-500 mb-6">
            Erstelle Rabatte, die du bei Buchungen anwenden kannst
          </p>
          <button
            onClick={handleCreate}
            className="px-6 py-3 bg-amber-600 hover:bg-amber-700 text-white font-semibold rounded-lg transition-colors"
          >
            Ersten Rabatt erstellen
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {templates.map((template) => (
            <div
              key={template.id}
              className="bg-slate-800 rounded-xl p-6 border border-slate-700 hover:border-slate-600 transition-colors"
            >
              {/* Header with Status */}
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center gap-3">
                  <div className="p-3 bg-amber-500/20 rounded-lg">
                    <Tag className="w-5 h-5 text-amber-400" />
                  </div>
                  <div>
                    <h3 className="font-semibold text-white">{template.name}</h3>
                    {template.description && (
                      <p className="text-sm text-slate-400 mt-1">{template.description}</p>
                    )}
                  </div>
                </div>
                <div className="flex items-center gap-1">
                  {template.is_active ? (
                    <CheckCircle className="w-5 h-5 text-emerald-400" title="Aktiv" />
                  ) : (
                    <XCircle className="w-5 h-5 text-slate-500" title="Inaktiv" />
                  )}
                </div>
              </div>

              {/* Discount Value */}
              <div className="flex items-center gap-2 mb-4">
                {getDiscountIcon(template) === 'Percent' ? (
                  <Percent className="w-4 h-4 text-slate-400" />
                ) : (
                  <Euro className="w-4 h-4 text-slate-400" />
                )}
                <span className="text-2xl font-bold text-white">
                  {formatDiscountValue(template)}
                </span>
                <span className="text-sm text-slate-400 ml-auto">
                  {template.discount_type === 'fixed' ? 'Fester Betrag' : 'Prozentual'}
                </span>
              </div>

              {/* Actions */}
              <div className="flex gap-2">
                <button
                  onClick={() => handleEdit(template)}
                  className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors"
                >
                  <Pencil className="w-4 h-4" />
                  Bearbeiten
                </button>
                <button
                  onClick={() => setDeleteDialog({ open: true, id: template.id })}
                  className="flex items-center justify-center gap-2 px-3 py-2 bg-red-600 hover:bg-red-700 text-white font-medium rounded-lg transition-colors"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Dialogs */}
      <DiscountTemplateDialog
        isOpen={dialogOpen}
        onClose={() => {
          setDialogOpen(false);
          setSelectedTemplate(undefined);
        }}
        onSuccess={loadTemplates}
        template={selectedTemplate}
      />

      <ConfirmDialog
        isOpen={deleteDialog.open}
        onClose={() => setDeleteDialog({ open: false, id: null })}
        onConfirm={handleDelete}
        title="Rabatt löschen"
        message="Möchtest du diesen Rabatt wirklich löschen? Diese Aktion kann nicht rückgängig gemacht werden."
        confirmText="Löschen"
        confirmButtonClass="bg-red-600 hover:bg-red-700"
      />
    </div>
  );
}
