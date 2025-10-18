import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Briefcase, Euro, FileText, Smile, ClipboardList } from 'lucide-react';
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
        is_active: template.is_active,
        emoji: template.emoji || '',
        show_in_cleaning_plan: template.show_in_cleaning_plan,
        cleaning_plan_position: template.cleaning_plan_position,
      });
    } else {
      setFormData({
        name: '',
        description: '',
        price: 0,
        is_active: true,
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
        await invoke('update_service_template_command', {
          id: template.id,
          name: formData.name,
          description: formData.description || null,
          price: formData.price,
          isActive: formData.is_active,
          emoji: formData.emoji || null,
          showInCleaningPlan: formData.show_in_cleaning_plan,
          cleaningPlanPosition: formData.cleaning_plan_position,
        });
      } else {
        // Create new template
        await invoke('create_service_template_command', {
          name: formData.name,
          description: formData.description || null,
          price: formData.price,
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

          {/* Price */}
          <div className="space-y-2">
            <label htmlFor="price" className="block text-sm font-medium text-slate-300">
              Preis *
            </label>
            <div className="relative">
              <Euro className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-400" />
              <input
                id="price"
                type="number"
                step="0.01"
                min="0"
                required
                value={formData.price}
                onChange={(e) => setFormData({ ...formData, price: parseFloat(e.target.value) || 0 })}
                className="w-full pl-12 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
                placeholder="0.00"
              />
            </div>
          </div>

          {/* Emoji */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-slate-300">
              Emoji (optional)
            </label>
            <div className="relative">
              <button
                type="button"
                onClick={() => setShowEmojiPicker(!showEmojiPicker)}
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white hover:bg-slate-600 transition-colors flex items-center justify-between"
              >
                <div className="flex items-center gap-2">
                  <Smile className="w-5 h-5 text-slate-400" />
                  <span className="text-sm">
                    {formData.emoji ? (
                      <span className="text-2xl">{formData.emoji}</span>
                    ) : (
                      'Emoji auswählen...'
                    )}
                  </span>
                </div>
                {formData.emoji && (
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      setFormData({ ...formData, emoji: '' });
                    }}
                    className="text-red-400 hover:text-red-300 p-1 rounded hover:bg-red-500/20 transition-colors"
                    title="Emoji entfernen"
                  >
                    <X className="w-4 h-4" />
                  </button>
                )}
              </button>

              {/* Emoji Picker Dropdown */}
              {showEmojiPicker && (
                <div
                  ref={emojiPickerRef}
                  className="absolute top-full mt-2 z-50 shadow-2xl rounded-lg overflow-hidden"
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
