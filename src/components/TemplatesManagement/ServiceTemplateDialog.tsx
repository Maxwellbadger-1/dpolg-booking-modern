import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Briefcase, Euro, FileText, Smile, ClipboardList, Percent } from 'lucide-react';
import { ServiceTemplate } from '../../types/booking';
import data from '@emoji-mart/data';
import Picker from '@emoji-mart/react';

interface ServiceTemplateDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  template?: ServiceTemplate;
}

export default function ServiceTemplateDialog({
  isOpen,
  onClose,
  onSuccess,
  template
}: ServiceTemplateDialogProps) {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    price: 0,
    is_active: true,
    price_type: 'fixed' as 'fixed' | 'percent',
    applies_to: 'overnight_price' as 'overnight_price' | 'total_price',
    emoji: '',
    show_in_cleaning_plan: false,
    cleaning_plan_position: 'start' as 'start' | 'end',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const emojiPickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (template) {
      setFormData({
        name: template.name,
        description: template.description || '',
        price: template.price,
        is_active: template.isActive,
        price_type: template.priceType,
        applies_to: template.appliesTo,
        emoji: template.emoji || '',
        show_in_cleaning_plan: template.showInCleaningPlan,
        cleaning_plan_position: template.cleaningPlanPosition,
      });
    } else {
      setFormData({
        name: '',
        description: '',
        price: 0,
        is_active: true,
        price_type: 'fixed',
        applies_to: 'overnight_price',
        emoji: '',
        show_in_cleaning_plan: false,
        cleaning_plan_position: 'start',
      });
    }
    setError(null);
  }, [template, isOpen]);

  // Close emoji picker when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (emojiPickerRef.current && !emojiPickerRef.current.contains(event.target as Node)) {
        setShowEmojiPicker(false);
      }
    };

    if (showEmojiPicker) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showEmojiPicker]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (template?.id) {
        // Update existing template
        await invoke('update_service_template_pg', {
          id: template.id,
          name: formData.name,
          description: formData.description || null,
          price: formData.price,
          isActive: formData.is_active,
          priceType: formData.price_type,
          appliesTo: formData.applies_to,
          emoji: formData.emoji || null,
          showInCleaningPlan: formData.show_in_cleaning_plan,
          cleaningPlanPosition: formData.cleaning_plan_position,
        });
      } else {
        // Create new template
        await invoke('create_service_template_pg', {
          name: formData.name,
          description: formData.description || null,
          price: formData.price,
          priceType: formData.price_type,
          appliesTo: formData.applies_to,
          emoji: formData.emoji || null,
          showInCleaningPlan: formData.show_in_cleaning_plan,
          cleaningPlanPosition: formData.cleaning_plan_position,
        });
      }
      onSuccess();
      onClose();
    } catch (err) {
      console.error('Fehler beim Speichern der Zusatzleistung:', err);
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
            <div className="p-3 bg-emerald-500/20 rounded-xl">
              <Briefcase className="w-6 h-6 text-emerald-400" />
            </div>
            <h2 className="text-2xl font-bold text-white">
              {template ? 'Zusatzleistung bearbeiten' : 'Neue Zusatzleistung'}
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
              <Briefcase className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
              <input
                id="name"
                type="text"
                required
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full pl-12 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
                placeholder="z.B. Frühstück pro Person"
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
                className="w-full pl-12 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent resize-none"
                placeholder="Zusätzliche Informationen..."
              />
            </div>
          </div>

          {/* Price Type */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-slate-300">
              Preis-Typ *
            </label>
            <div className="grid grid-cols-2 gap-4">
              <button
                type="button"
                onClick={() => setFormData({ ...formData, price_type: 'fixed' })}
                className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border-2 transition-all ${
                  formData.price_type === 'fixed'
                    ? 'border-emerald-500 bg-emerald-500/20 text-emerald-400'
                    : 'border-slate-600 bg-slate-700 text-slate-400 hover:border-slate-500'
                }`}
              >
                <Euro className="w-5 h-5" />
                <span className="font-medium">Fester Betrag</span>
              </button>
              <button
                type="button"
                onClick={() => setFormData({ ...formData, price_type: 'percent' })}
                className={`flex items-center justify-center gap-2 px-4 py-3 rounded-lg border-2 transition-all ${
                  formData.price_type === 'percent'
                    ? 'border-emerald-500 bg-emerald-500/20 text-emerald-400'
                    : 'border-slate-600 bg-slate-700 text-slate-400 hover:border-slate-500'
                }`}
              >
                <Percent className="w-5 h-5" />
                <span className="font-medium">Prozentual</span>
              </button>
            </div>
          </div>

          {/* Price */}
          <div className="space-y-2">
            <label htmlFor="price" className="block text-sm font-medium text-slate-300">
              {formData.price_type === 'fixed' ? 'Preis-Betrag' : 'Preis-Prozentsatz'} *
            </label>
            <div className="relative">
              {formData.price_type === 'fixed' ? (
                <Euro className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
              ) : (
                <Percent className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
              )}
              <input
                id="price"
                type="number"
                step={formData.price_type === 'fixed' ? '0.01' : '1'}
                min="0"
                max={formData.price_type === 'percent' ? '100' : undefined}
                required
                value={formData.price}
                onChange={(e) => setFormData({ ...formData, price: parseFloat(e.target.value) || 0 })}
                className="w-full pl-12 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
                placeholder={formData.price_type === 'fixed' ? '0.00' : '0'}
              />
            </div>
          </div>

          {/* Applies To (nur bei Prozent) */}
          {formData.price_type === 'percent' && (
            <div className="space-y-2">
              <label htmlFor="applies_to" className="block text-sm font-medium text-slate-300">
                Worauf bezieht sich der Preis? *
              </label>
              <select
                id="applies_to"
                value={formData.applies_to}
                onChange={(e) => setFormData({ ...formData, applies_to: e.target.value as 'overnight_price' | 'total_price' })}
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
              >
                <option value="overnight_price">Übernachtungspreis</option>
                <option value="total_price">Gesamtpreis</option>
              </select>
            </div>
          )}

          {/* Emoji */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-slate-300">
              Emoji (optional)
            </label>
            <div className="relative">
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setShowEmojiPicker(!showEmojiPicker)}
                  className="flex-1 px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white hover:bg-slate-600 transition-colors flex items-center gap-2"
                >
                  <Smile className="w-5 h-5 text-slate-400" />
                  <span className="text-sm">
                    {formData.emoji ? (
                      <span className="text-2xl">{formData.emoji}</span>
                    ) : (
                      'Emoji auswählen...'
                    )}
                  </span>
                </button>
                {formData.emoji && (
                  <button
                    type="button"
                    onClick={() => setFormData({ ...formData, emoji: '' })}
                    className="p-3 bg-slate-700 border border-slate-600 rounded-lg text-red-400 hover:text-red-300 hover:bg-red-500/20 transition-colors"
                    title="Emoji entfernen"
                  >
                    <X className="w-4 h-4" />
                  </button>
                )}
              </div>

              {/* Emoji Picker Dropdown */}
              {showEmojiPicker && (
                <div
                  ref={emojiPickerRef}
                  className="absolute left-0 top-full mt-2 z-[100] shadow-2xl rounded-lg overflow-hidden"
                >
                  <Picker
                    data={data}
                    onEmojiSelect={(emoji: any) => {
                      setFormData({ ...formData, emoji: emoji.native });
                      setShowEmojiPicker(false);
                    }}
                    theme="dark"
                    previewPosition="none"
                    searchPosition="sticky"
                    locale="de"
                    perLine={8}
                    maxFrequentRows={2}
                  />
                </div>
              )}
            </div>
          </div>

          {/* Putzplan-Integration */}
          <div className="space-y-4 p-4 bg-slate-800/50 rounded-lg border border-slate-700">
            <div className="flex items-center gap-3">
              <ClipboardList className="w-5 h-5 text-emerald-400" />
              <h3 className="text-sm font-semibold text-white">Putzplan-Integration</h3>
            </div>

            <div className="flex items-center gap-3">
              <input
                id="show_in_cleaning_plan"
                type="checkbox"
                checked={formData.show_in_cleaning_plan}
                onChange={(e) => setFormData({ ...formData, show_in_cleaning_plan: e.target.checked })}
                className="w-5 h-5 rounded border-slate-600 bg-slate-700 text-emerald-500 focus:ring-2 focus:ring-emerald-500 focus:ring-offset-0"
              />
              <label htmlFor="show_in_cleaning_plan" className="text-sm font-medium text-slate-300">
                Im Putzplan anzeigen
              </label>
            </div>

            {formData.show_in_cleaning_plan && (
              <div className="space-y-2 pl-8">
                <label htmlFor="position" className="block text-sm font-medium text-slate-300">
                  Position im Putzplan
                </label>
                <select
                  id="position"
                  value={formData.cleaning_plan_position}
                  onChange={(e) => setFormData({ ...formData, cleaning_plan_position: e.target.value as 'start' | 'end' })}
                  className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
                >
                  <option value="start">Anreise (Check-in Tag)</option>
                  <option value="end">Abreise (Check-out Tag)</option>
                </select>
              </div>
            )}
          </div>

          {/* Active Toggle */}
          {template && (
            <div className="flex items-center gap-3">
              <input
                id="is_active"
                type="checkbox"
                checked={formData.is_active}
                onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })}
                className="w-5 h-5 rounded border-slate-600 bg-slate-700 text-emerald-500 focus:ring-2 focus:ring-emerald-500 focus:ring-offset-0"
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
              className="flex-1 px-4 py-3 bg-emerald-600 hover:bg-emerald-700 text-white font-semibold rounded-lg transition-colors disabled:bg-emerald-800 disabled:cursor-not-allowed"
            >
              {loading ? 'Speichert...' : 'Speichern'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
