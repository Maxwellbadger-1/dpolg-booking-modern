import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Tag, Euro, Percent, FileText } from 'lucide-react';
import { DiscountTemplate } from '../../types/booking';

interface DiscountTemplateDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  template?: DiscountTemplate;
}

export default function DiscountTemplateDialog({
  isOpen,
  onClose,
  onSuccess,
  template
}: DiscountTemplateDialogProps) {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    discount_type: 'fixed' as 'percent' | 'fixed',
    discount_value: 0,
    is_active: true,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (template) {
      setFormData({
        name: template.name,
        description: template.description || '',
        discount_type: template.discount_type as 'percent' | 'fixed',
        discount_value: template.discount_value,
        is_active: template.is_active,
      });
    } else {
      setFormData({
        name: '',
        description: '',
        discount_type: 'fixed',
        discount_value: 0,
        is_active: true,
      });
    }
    setError(null);
  }, [template, isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (template?.id) {
        // Update existing template
        await invoke('update_discount_template_command', {
          id: template.id,
          name: formData.name,
          description: formData.description || null,
          discountType: formData.discount_type,
          discountValue: formData.discount_value,
          isActive: formData.is_active,
        });
      } else {
        // Create new template
        await invoke('create_discount_template_command', {
          name: formData.name,
          description: formData.description || null,
          discountType: formData.discount_type,
          discountValue: formData.discount_value,
        });
      }
      onSuccess();
      onClose();
    } catch (err) {
      console.error('Fehler beim Speichern des Rabatts:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="px-8 py-6 border-b border-slate-700 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-amber-500/20 rounded-xl">
              <Tag className="w-6 h-6 text-amber-400" />
            </div>
            <h2 className="text-2xl font-bold text-white">
              {template ? 'Rabatt bearbeiten' : 'Neuer Rabatt'}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-slate-300" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-8 space-y-6 overflow-y-auto max-h-[calc(90vh-180px)]">
          {error && (
            <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400">
              {error}
            </div>
          )}

          {/* Name */}
          <div className="space-y-2">
            <label htmlFor="name" className="block text-sm font-medium text-slate-300">
              Bezeichnung *
            </label>
            <div className="relative">
              <Tag className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
              <input
                id="name"
                type="text"
                required
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full pl-12 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent"
                placeholder="z.B. Frühbucher-Rabatt"
              />
            </div>
          </div>

          {/* Description */}
          <div className="space-y-2">
            <label htmlFor="description" className="block text-sm font-medium text-slate-300">
              Beschreibung (optional)
            </label>
            <div className="relative">
              <FileText className="absolute left-4 top-4 w-5 h-5 text-slate-400" />
              <textarea
                id="description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={3}
                className="w-full pl-12 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent resize-none"
                placeholder="Zusätzliche Informationen..."
              />
            </div>
          </div>

          {/* Discount Type */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-slate-300">
              Rabatt-Typ *
            </label>
            <div className="grid grid-cols-2 gap-4">
              <button
                type="button"
                onClick={() => setFormData({ ...formData, discount_type: 'fixed' })}
                className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border-2 transition-all ${
                  formData.discount_type === 'fixed'
                    ? 'border-amber-500 bg-amber-500/20 text-amber-400'
                    : 'border-slate-600 bg-slate-700 text-slate-400 hover:border-slate-500'
                }`}
              >
                <Euro className="w-5 h-5" />
                <span className="font-medium">Fester Betrag</span>
              </button>
              <button
                type="button"
                onClick={() => setFormData({ ...formData, discount_type: 'percent' })}
                className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border-2 transition-all ${
                  formData.discount_type === 'percent'
                    ? 'border-amber-500 bg-amber-500/20 text-amber-400'
                    : 'border-slate-600 bg-slate-700 text-slate-400 hover:border-slate-500'
                }`}
              >
                <Percent className="w-5 h-5" />
                <span className="font-medium">Prozentual</span>
              </button>
            </div>
          </div>

          {/* Discount Value */}
          <div className="space-y-2">
            <label htmlFor="discount_value" className="block text-sm font-medium text-slate-300">
              {formData.discount_type === 'fixed' ? 'Rabatt-Betrag' : 'Rabatt-Prozentsatz'} *
            </label>
            <div className="relative">
              {formData.discount_type === 'fixed' ? (
                <Euro className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
              ) : (
                <Percent className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
              )}
              <input
                id="discount_value"
                type="number"
                step={formData.discount_type === 'fixed' ? '0.01' : '1'}
                min="0"
                max={formData.discount_type === 'percent' ? '100' : undefined}
                required
                value={formData.discount_value}
                onChange={(e) => setFormData({ ...formData, discount_value: parseFloat(e.target.value) || 0 })}
                className="w-full pl-12 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent"
                placeholder={formData.discount_type === 'fixed' ? '0.00' : '0'}
              />
            </div>
          </div>

          {/* Active Toggle */}
          {template && (
            <div className="flex items-center gap-3">
              <input
                id="is_active"
                type="checkbox"
                checked={formData.is_active}
                onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })}
                className="w-5 h-5 rounded border-slate-600 bg-slate-700 text-amber-500 focus:ring-2 focus:ring-amber-500 focus:ring-offset-0"
              />
              <label htmlFor="is_active" className="text-sm font-medium text-slate-300">
                Aktiv (verfügbar bei Buchungen)
              </label>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex gap-3 pt-6 border-t border-slate-700">
            <button
              type="button"
              onClick={onClose}
              disabled={loading}
              className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors disabled:bg-slate-800 disabled:cursor-not-allowed"
            >
              Abbrechen
            </button>
            <button
              type="submit"
              disabled={loading}
              className="flex-1 px-4 py-3 bg-amber-600 hover:bg-amber-700 text-white font-semibold rounded-lg transition-colors disabled:bg-amber-800 disabled:cursor-not-allowed"
            >
              {loading ? 'Speichert...' : 'Speichern'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
