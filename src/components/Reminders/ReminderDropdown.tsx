import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, CheckCircle, Clock, AlertTriangle, Calendar, ChevronRight } from 'lucide-react';
import type { Reminder } from '../../types/reminder';
import { formatDateSmart } from '../../utils/dateFormatting';

interface ReminderDropdownProps {
  isOpen: boolean;
  onClose: () => void;
  onReminderClick?: (bookingId: number | null) => void;
  onViewAll?: () => void;
}

export default function ReminderDropdown({ isOpen, onClose, onReminderClick, onViewAll }: ReminderDropdownProps) {
  const [urgentReminders, setUrgentReminders] = useState<Reminder[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadUrgentReminders();
    }
  }, [isOpen]);

  const loadUrgentReminders = async () => {
    setLoading(true);
    try {
      const reminders = await invoke<Reminder[]>('get_urgent_reminders_command');
      // Nur die nächsten 5 anzeigen
      setUrgentReminders(reminders.slice(0, 5));
    } catch (error) {
      console.error('Fehler beim Laden der Erinnerungen:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleMarkCompleted = async (id: number, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke('mark_reminder_completed_command', { id, completed: true });
      // Reload reminders
      await loadUrgentReminders();
      // Trigger refresh für Badge count
      window.dispatchEvent(new CustomEvent('reminder-updated'));
    } catch (error) {
      console.error('Fehler beim Markieren der Erinnerung:', error);
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high': return 'text-red-400 bg-red-500/10';
      case 'medium': return 'text-amber-400 bg-amber-500/10';
      case 'low': return 'text-blue-400 bg-blue-500/10';
      default: return 'text-slate-400 bg-slate-500/10';
    }
  };

  const getPriorityIcon = (priority: string) => {
    switch (priority) {
      case 'high': return <AlertTriangle className="w-4 h-4" />;
      case 'medium': return <Clock className="w-4 h-4" />;
      case 'low': return <Calendar className="w-4 h-4" />;
      default: return <Clock className="w-4 h-4" />;
    }
  };


  if (!isOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div className="fixed inset-0 z-[90]" onClick={onClose} />

      {/* Dropdown */}
      <div className="absolute top-full right-0 mt-2 w-96 bg-gradient-to-br from-slate-800 to-slate-900 rounded-xl shadow-2xl border border-slate-700 z-[100]">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-slate-700">
          <h3 className="text-lg font-bold text-white">Dringende Erinnerungen</h3>
          <button
            onClick={onClose}
            className="p-1 hover:bg-slate-700 rounded-lg transition-colors"
            aria-label="Schließen"
          >
            <X className="w-5 h-5 text-slate-400" />
          </button>
        </div>

        {/* Content */}
        <div className="max-h-96 overflow-y-auto">
          {loading ? (
            <div className="p-8 text-center">
              <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
              <p className="mt-2 text-sm text-slate-400">Lade Erinnerungen...</p>
            </div>
          ) : urgentReminders.length === 0 ? (
            <div className="p-8 text-center">
              <CheckCircle className="w-12 h-12 text-emerald-400 mx-auto mb-3" />
              <p className="text-slate-300 font-medium">Keine dringenden Erinnerungen</p>
              <p className="text-sm text-slate-400 mt-1">Alles erledigt! 🎉</p>
            </div>
          ) : (
            <div className="divide-y divide-slate-700">
              {urgentReminders.map((reminder) => (
                <div
                  key={reminder.id}
                  className="p-4 hover:bg-slate-700/30 transition-colors cursor-pointer group"
                  onClick={() => onReminderClick && onReminderClick(reminder.booking_id)}
                >
                  <div className="flex items-start gap-3">
                    {/* Priority Icon */}
                    <div className={`p-2 rounded-lg ${getPriorityColor(reminder.priority)}`}>
                      {getPriorityIcon(reminder.priority)}
                    </div>

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between gap-2">
                        <h4 className="text-sm font-semibold text-white group-hover:text-blue-400 transition-colors">
                          {reminder.title}
                        </h4>
                        <span className={`text-xs font-medium px-2 py-0.5 rounded-full whitespace-nowrap ${
                          reminder.due_date < new Date().toISOString().split('T')[0]
                            ? 'bg-red-500/20 text-red-300'
                            : 'bg-blue-500/20 text-blue-300'
                        }`}>
                          {formatDateSmart(reminder.due_date)}
                        </span>
                      </div>

                      {reminder.description && (
                        <p className="text-xs text-slate-400 mt-1 line-clamp-2">
                          {reminder.description}
                        </p>
                      )}

                      {/* Actions */}
                      <div className="flex items-center gap-2 mt-2">
                        <button
                          onClick={(e) => handleMarkCompleted(reminder.id, e)}
                          className="text-xs text-emerald-400 hover:text-emerald-300 font-medium flex items-center gap-1"
                        >
                          <CheckCircle className="w-3 h-3" />
                          Erledigt
                        </button>
                        {reminder.booking_id && (
                          <span className="text-xs text-slate-500">•</span>
                        )}
                        {reminder.booking_id && (
                          <span className="text-xs text-blue-400">Buchung #{reminder.booking_id}</span>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        {urgentReminders.length > 0 && (
          <div className="p-3 border-t border-slate-700">
            <button
              onClick={() => {
                onClose();
                onViewAll && onViewAll();
              }}
              className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors flex items-center justify-center gap-2"
            >
              Alle Erinnerungen anzeigen
              <ChevronRight className="w-4 h-4" />
            </button>
          </div>
        )}
      </div>
    </>
  );
}
