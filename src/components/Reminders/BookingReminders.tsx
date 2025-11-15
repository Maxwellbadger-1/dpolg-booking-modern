import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Bell, Plus, CheckCircle, Clock, AlertTriangle, Calendar, Trash2, Edit2, X } from 'lucide-react';
import { format } from 'date-fns';
import { de } from 'date-fns/locale';
import type { Reminder, CreateReminderData, UpdateReminderData } from '../../types/reminder';
import { useGlobalReminderUpdates } from '../../hooks/useGlobalReminderUpdates';

interface BookingRemindersProps {
  bookingId: number;
}

export default function BookingReminders({ bookingId }: BookingRemindersProps) {
  const [reminders, setReminders] = useState<Reminder[]>([]);
  const [loading, setLoading] = useState(false);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [editingReminder, setEditingReminder] = useState<Reminder | null>(null);

  const loadReminders = useCallback(async () => {
    try {
      setLoading(true);
      const data = await invoke<Reminder[]>('get_reminders_for_booking_command', { bookingId });
      setReminders(data);
    } catch (error) {
      console.error('Fehler beim Laden der Erinnerungen:', error);
    } finally {
      setLoading(false);
    }
  }, [bookingId]);

  useEffect(() => {
    loadReminders();
  }, [loadReminders]);

  // Listen to Tauri events from backend
  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen('reminder-updated', (event: any) => {
        const { reminderType, bookingId: eventBookingId } = event.payload;
        if (eventBookingId === bookingId) {
          setReminders(prev => prev.filter(r => r.reminder_type !== reminderType || r.status === 'completed'));
          loadReminders();
        }
      });
      return unlisten;
    };

    let unlistenFn: any = null;
    setupListener().then(fn => { unlistenFn = fn; });

    return () => {
      if (unlistenFn) unlistenFn();
    };
  }, [bookingId, loadReminders]);

  // Subscribe to global reminder updates (survives unmount/remount)
  useGlobalReminderUpdates(bookingId, async (reminderType) => {
    console.log('🔔 [BookingReminders] Update callback triggered', {
      reminderType,
      bookingId,
      timestamp: new Date().toISOString()
    });

    // Optimistic update: Sofort aus UI entfernen
    console.log('⚡ [BookingReminders] Optimistic update: Removing reminder type', reminderType);
    setReminders(prev => {
      console.log('   Before filter - reminders:', prev.length);
      console.log('   Current reminders:', prev.map(r => ({ id: r.id, type: r.reminder_type, status: r.status, title: r.title })));
      const filtered = prev.filter(r => {
        // Keep reminder if: different type OR already completed
        const keep = r.reminder_type !== reminderType || r.status === 'completed';
        if (!keep) {
          console.log('   🗑️ Removing reminder:', { id: r.id, type: r.reminder_type, status: r.status, title: r.title });
        } else {
          console.log('   ✅ Keeping reminder:', { id: r.id, type: r.reminder_type, status: r.status, title: r.title });
        }
        return keep;
      });
      console.log('   After filter - reminders:', filtered.length);
      return filtered;
    });

    // Dann backend reload für Konsistenz
    console.log('🔄 [BookingReminders] Reloading from backend...');
    try {
      setLoading(true);
      const data = await invoke<Reminder[]>('get_reminders_for_booking_command', { bookingId });
      console.log('✅ [BookingReminders] Backend reload complete - reminders:', data.length);
      setReminders(data);
    } catch (error) {
      console.error('❌ [BookingReminders] Fehler beim Laden der Erinnerungen:', error);
    } finally {
      setLoading(false);
    }
  });

  const handleCreateReminder = async (data: CreateReminderData) => {
    try {
      await invoke('create_reminder_command', {
        bookingId: data.booking_id,
        reminderType: data.reminder_type,
        title: data.title,
        description: data.description || null,
        dueDate: data.due_date,
        priority: data.priority,
      });
      await loadReminders();
      setShowCreateDialog(false);
      window.dispatchEvent(new CustomEvent('reminder-updated'));
    } catch (error) {
      console.error('Fehler beim Erstellen der Erinnerung:', error);
      alert(`Fehler beim Erstellen der Erinnerung: ${error}`);
    }
  };

  const handleUpdateReminder = async (id: number, data: UpdateReminderData) => {
    try {
      await invoke('update_reminder_command', {
        id,
        title: data.title,
        description: data.description || null,
        dueDate: data.due_date,
        priority: data.priority,
      });
      await loadReminders();
      setEditingReminder(null);
      window.dispatchEvent(new CustomEvent('reminder-updated'));
    } catch (error) {
      console.error('Fehler beim Aktualisieren der Erinnerung:', error);
      alert(`Fehler beim Aktualisieren der Erinnerung: ${error}`);
    }
  };

  const handleMarkCompleted = async (id: number) => {
    // OPTIMISTIC UPDATE (2025 Best Practice)
    // 1. Sofort Event dispatchen für Badge-Update (KEIN await!)
    window.dispatchEvent(new CustomEvent('reminder-updated'));

    // 2. Optimistisch lokales State updaten
    setReminders(prev => prev.map(r =>
      r.id === id ? { ...r, is_completed: 1 } : r
    ));

    try {
      // 3. Backend Update
      await invoke('mark_reminder_completed_command', { id, completed: true });

      // 4. Final refresh für Konsistenz
      await loadReminders();
    } catch (error) {
      console.error('Fehler beim Markieren der Erinnerung:', error);

      // 5. Rollback bei Fehler
      await loadReminders();
      window.dispatchEvent(new CustomEvent('reminder-updated'));
    }
  };

  const handleMarkUncompleted = async (id: number) => {
    try {
      await invoke('mark_reminder_completed_command', { id, completed: false });
      await loadReminders();
      window.dispatchEvent(new CustomEvent('reminder-updated'));
    } catch (error) {
      console.error('Fehler beim Markieren der Erinnerung:', error);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm('Erinnerung wirklich löschen?')) return;

    try {
      await invoke('delete_reminder_command', { id });
      await loadReminders();
      window.dispatchEvent(new CustomEvent('reminder-updated'));
    } catch (error) {
      console.error('Fehler beim Löschen der Erinnerung:', error);
      alert(`Fehler beim Löschen der Erinnerung: ${error}`);
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high': return 'text-red-600 bg-red-50 border-red-200';
      case 'medium': return 'text-amber-600 bg-amber-50 border-amber-200';
      case 'low': return 'text-blue-600 bg-blue-50 border-blue-200';
      default: return 'text-slate-600 bg-slate-50 border-slate-200';
    }
  };

  const getPriorityIcon = (priority: string) => {
    switch (priority) {
      case 'high': return <AlertTriangle className="w-4 h-4" />;
      case 'medium': return <Clock className="w-4 h-4" />;
      case 'low': return <Calendar className="w-4 h-4" />;
      default: return <Bell className="w-4 h-4" />;
    }
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case 'manual': return 'Manuell';
      case 'auto_incomplete_data': return 'Unvollständige Daten';
      case 'auto_payment': return 'Zahlung ausstehend';
      case 'auto_checkin': return 'Check-in Vorbereitung';
      case 'auto_invoice': return 'Rechnung versenden';
      default: return type;
    }
  };

  const formatDate = (dateStr: string): string => {
    try {
      const date = new Date(dateStr);
      return format(date, 'dd.MM.yyyy', { locale: de });
    } catch {
      return dateStr;
    }
  };

  const incompleteReminders = reminders.filter(r => !r.is_completed);
  const completedReminders = reminders.filter(r => r.is_completed);

  return (
    <div className="border border-slate-200 rounded-lg p-5 bg-gradient-to-br from-purple-50 to-white">
      <div className="flex items-center justify-between mb-4">
        <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800">
          <Bell className="w-5 h-5 text-purple-600" />
          Erinnerungen ({incompleteReminders.length} offen)
        </h3>
        <button
          onClick={() => setShowCreateDialog(true)}
          className="flex items-center gap-2 px-3 py-1.5 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-semibold transition-colors text-sm"
        >
          <Plus className="w-4 h-4" />
          Neue Erinnerung
        </button>
      </div>

      {loading ? (
        <div className="text-center py-8">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600"></div>
          <p className="mt-2 text-sm text-slate-600">Lade Erinnerungen...</p>
        </div>
      ) : reminders.length === 0 ? (
        <div className="text-center py-8 bg-slate-50 rounded-lg border-2 border-dashed border-slate-300">
          <Bell className="w-12 h-12 text-slate-400 mx-auto mb-3" />
          <p className="text-slate-600 font-medium">Noch keine Erinnerungen</p>
          <p className="text-sm text-slate-500 mt-1">
            Klicken Sie auf "Neue Erinnerung" um eine Erinnerung anzulegen
          </p>
        </div>
      ) : (
        <>
          {/* Offene Erinnerungen */}
          {incompleteReminders.length > 0 && (
            <div className="space-y-2 mb-4">
              {incompleteReminders.map((reminder) => (
                <div
                  key={reminder.id}
                  className={`flex items-start gap-3 border rounded-lg p-4 ${getPriorityColor(reminder.priority)}`}
                >
                  <div className="flex-shrink-0 mt-1">
                    {getPriorityIcon(reminder.priority)}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1">
                        <h4 className="font-semibold text-slate-900">{reminder.title}</h4>
                        {reminder.description && (
                          <p className="text-sm text-slate-600 mt-1">{reminder.description}</p>
                        )}
                        <div className="flex items-center gap-3 mt-2 text-xs text-slate-600">
                          <span className="flex items-center gap-1">
                            <Calendar className="w-3 h-3" />
                            {formatDate(reminder.due_date)}
                          </span>
                          <span className="px-2 py-0.5 bg-white rounded-full border">
                            {getTypeLabel(reminder.reminder_type)}
                          </span>
                        </div>
                      </div>
                      <div className="flex items-center gap-1">
                        <button
                          onClick={() => handleMarkCompleted(reminder.id)}
                          className="p-1.5 hover:bg-white rounded transition-colors"
                          title="Als erledigt markieren"
                        >
                          <CheckCircle className="w-4 h-4 text-emerald-600" />
                        </button>
                        <button
                          onClick={() => setEditingReminder(reminder)}
                          className="p-1.5 hover:bg-white rounded transition-colors"
                          title="Bearbeiten"
                        >
                          <Edit2 className="w-4 h-4 text-blue-600" />
                        </button>
                        <button
                          onClick={() => handleDelete(reminder.id)}
                          className="p-1.5 hover:bg-white rounded transition-colors"
                          title="Löschen"
                        >
                          <Trash2 className="w-4 h-4 text-red-600" />
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Erledigte Erinnerungen (collapsed) */}
          {completedReminders.length > 0 && (
            <details className="mt-4">
              <summary className="cursor-pointer text-sm text-slate-600 hover:text-slate-900 font-medium">
                Erledigte Erinnerungen ({completedReminders.length})
              </summary>
              <div className="space-y-2 mt-2">
                {completedReminders.map((reminder) => (
                  <div
                    key={reminder.id}
                    className="flex items-start gap-3 border border-slate-200 rounded-lg p-3 bg-slate-50 opacity-60"
                  >
                    <CheckCircle className="w-4 h-4 text-emerald-600 mt-1" />
                    <div className="flex-1">
                      <h4 className="font-medium text-slate-900 line-through">{reminder.title}</h4>
                      <div className="flex items-center gap-3 mt-1 text-xs text-slate-500">
                        <span>{formatDate(reminder.due_date)}</span>
                        {reminder.completed_at && (
                          <span>Erledigt: {formatDate(reminder.completed_at.split('T')[0])}</span>
                        )}
                      </div>
                    </div>
                    <button
                      onClick={() => handleMarkUncompleted(reminder.id)}
                      className="p-1.5 hover:bg-slate-200 rounded transition-colors"
                      title="Als unerledigt markieren"
                    >
                      <X className="w-4 h-4 text-slate-600" />
                    </button>
                  </div>
                ))}
              </div>
            </details>
          )}
        </>
      )}

      {/* Create/Edit Dialog */}
      {(showCreateDialog || editingReminder) && (
        <ReminderDialog
          reminder={editingReminder}
          bookingId={bookingId}
          onSave={(data) => {
            if (editingReminder) {
              handleUpdateReminder(editingReminder.id, data as UpdateReminderData);
            } else {
              handleCreateReminder(data as CreateReminderData);
            }
          }}
          onClose={() => {
            setShowCreateDialog(false);
            setEditingReminder(null);
          }}
        />
      )}
    </div>
  );
}

interface ReminderDialogProps {
  reminder?: Reminder | null;
  bookingId: number;
  onSave: (data: CreateReminderData | UpdateReminderData) => void;
  onClose: () => void;
}

function ReminderDialog({ reminder, bookingId, onSave, onClose }: ReminderDialogProps) {
  const [title, setTitle] = useState(reminder?.title || '');
  const [description, setDescription] = useState(reminder?.description || '');
  const [dueDate, setDueDate] = useState(reminder?.due_date || new Date().toISOString().split('T')[0]);
  const [priority, setPriority] = useState<'low' | 'medium' | 'high'>(reminder?.priority || 'medium');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) {
      alert('Bitte einen Titel eingeben');
      return;
    }
    if (!dueDate) {
      alert('Bitte ein Datum auswählen');
      return;
    }

    if (reminder) {
      // Update existing reminder
      onSave({
        title: title.trim(),
        description: description.trim() || undefined,
        due_date: dueDate,
        priority,
      });
    } else {
      // Create new reminder
      onSave({
        booking_id: bookingId,
        reminder_type: 'manual',
        title: title.trim(),
        description: description.trim() || undefined,
        due_date: dueDate,
        priority,
      });
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[70]">
      <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-lg w-full mx-4">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-xl font-bold text-white">
            {reminder ? 'Erinnerung bearbeiten' : 'Neue Erinnerung'}
          </h3>
          <button
            onClick={onClose}
            className="p-1 hover:bg-slate-700 rounded-lg transition-colors"
            aria-label="Schließen"
          >
            <X className="w-5 h-5 text-slate-400" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Titel *
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-purple-500"
              placeholder="z.B. Gastdaten nachfragen"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Beschreibung
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-purple-500 resize-none"
              placeholder="Optional: Weitere Details..."
              rows={3}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Fälligkeitsdatum *
            </label>
            <input
              type="date"
              value={dueDate}
              onChange={(e) => setDueDate(e.target.value)}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-purple-500"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Priorität *
            </label>
            <div className="grid grid-cols-3 gap-2">
              <button
                type="button"
                onClick={() => setPriority('low')}
                className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                  priority === 'low'
                    ? 'bg-blue-600 text-white'
                    : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                }`}
              >
                Niedrig
              </button>
              <button
                type="button"
                onClick={() => setPriority('medium')}
                className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                  priority === 'medium'
                    ? 'bg-amber-600 text-white'
                    : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                }`}
              >
                Mittel
              </button>
              <button
                type="button"
                onClick={() => setPriority('high')}
                className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                  priority === 'high'
                    ? 'bg-red-600 text-white'
                    : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                }`}
              >
                Hoch
              </button>
            </div>
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
            >
              Abbrechen
            </button>
            <button
              type="submit"
              className="flex-1 px-4 py-3 bg-purple-600 hover:bg-purple-700 text-white font-semibold rounded-lg transition-colors"
            >
              {reminder ? 'Speichern' : 'Erstellen'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
