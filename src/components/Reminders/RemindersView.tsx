import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Bell, Plus, CheckCircle, Clock, AlertTriangle, Calendar, Trash2, Edit2, Filter, ExternalLink } from 'lucide-react';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { Reminder, CreateReminderData, UpdateReminderData } from '../../types/reminder';
import { formatDate } from '../../utils/dateFormatting';
import FilterDatePicker from '../BookingManagement/FilterDatePicker';

interface RemindersViewProps {
  onNavigateToBooking?: (bookingId: number) => void;
}

export default function RemindersView({ onNavigateToBooking }: RemindersViewProps) {
  const [allReminders, setAllReminders] = useState<Reminder[]>([]);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState<'all' | 'open' | 'completed'>('open');
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [editingReminder, setEditingReminder] = useState<Reminder | null>(null);
  const [deleteDialogReminder, setDeleteDialogReminder] = useState<Reminder | null>(null);

  useEffect(() => {
    loadReminders();
  }, []);

  const loadReminders = async () => {
    try {
      setLoading(true);
      // Lade IMMER alle Reminders (inkl. completed), Filter nur für Anzeige
      const data = await invoke<Reminder[]>('get_all_reminders_pg', { includeCompleted: true });
      setAllReminders(data);
    } catch (error) {
      console.error('Fehler beim Laden der Erinnerungen:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateReminder = async (data: CreateReminderData) => {
    try {
      await invoke('create_reminder_pg', {
        bookingId: data.booking_id,
        reminderType: data.reminder_type,
        title: data.title,
        description: data.description || null,
        dueDate: data.due_date,
        priority: data.priority,
      });
      await loadReminders();
      setShowCreateDialog(false);
      // Badge-Count wird automatisch via PostgreSQL NOTIFY aktualisiert
    } catch (error) {
      console.error('Fehler beim Erstellen der Erinnerung:', error);
      alert(`Fehler beim Erstellen der Erinnerung: ${error}`);
    }
  };

  const handleUpdateReminder = async (id: number, data: UpdateReminderData) => {
    try {
      await invoke('update_reminder_pg', {
        id,
        title: data.title,
        description: data.description || null,
        dueDate: data.due_date,
        priority: data.priority,
      });
      await loadReminders();
      setEditingReminder(null);
      // Badge-Count wird automatisch via PostgreSQL NOTIFY aktualisiert
    } catch (error) {
      console.error('Fehler beim Aktualisieren der Erinnerung:', error);
      alert(`Fehler beim Aktualisieren der Erinnerung: ${error}`);
    }
  };

  const handleMarkCompleted = async (id: number, completed: boolean) => {
    console.log('✅ [RemindersView] handleMarkCompleted called', { id, completed });

    // OPTIMISTIC UPDATE (2025 Best Practice)
    // 1. Optimistisch lokales State updaten
    console.log('⚡ [RemindersView] Optimistic update - setting allReminders');
    setAllReminders(prev => prev.map(r =>
      r.id === id ? { ...r, is_completed: completed ? 1 : 0 } : r
    ));

    try {
      // 2. Backend Update (triggert PostgreSQL NOTIFY → Badge-Count Update automatisch)
      console.log('📤 [RemindersView] Calling backend complete_reminder_pg');
      await invoke('complete_reminder_pg', { id, completed });
      console.log('✅ [RemindersView] Backend update successful');

      // 3. Final refresh für Konsistenz
      console.log('🔄 [RemindersView] Refreshing reminders from backend');
      await loadReminders();
    } catch (error) {
      console.error('❌ [RemindersView] Fehler beim Markieren der Erinnerung:', error);

      // 4. Rollback bei Fehler
      await loadReminders();
    }
  };

  const handleDelete = async (reminder: Reminder) => {
    // Zeige Custom Delete Dialog
    setDeleteDialogReminder(reminder);
  };

  const confirmDelete = async () => {
    if (!deleteDialogReminder) return;

    try {
      await invoke('delete_reminder_pg', { id: deleteDialogReminder.id });
      await loadReminders();
      setDeleteDialogReminder(null);
      // Badge-Count wird automatisch via PostgreSQL NOTIFY aktualisiert
    } catch (error) {
      console.error('Fehler beim Löschen der Erinnerung:', error);
      alert(`Fehler beim Löschen der Erinnerung: ${error}`);
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high': return 'border-red-200 bg-red-50';
      case 'medium': return 'border-amber-200 bg-amber-50';
      case 'low': return 'border-blue-200 bg-blue-50';
      default: return 'border-slate-200 bg-slate-50';
    }
  };

  const getPriorityBadgeColor = (priority: string) => {
    switch (priority) {
      case 'high': return 'bg-red-100 text-red-700';
      case 'medium': return 'bg-amber-100 text-amber-700';
      case 'low': return 'bg-blue-100 text-blue-700';
      default: return 'bg-slate-100 text-slate-700';
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


  const isOverdue = (dateStr: string): boolean => {
    const date = new Date(dateStr);
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    return date < today;
  };

  // Berechne Anzahlen IMMER von ALLEN Reminders (nicht gefiltert)
  const openReminders = allReminders.filter(r => !r.is_completed);
  const completedReminders = allReminders.filter(r => r.is_completed);

  // Filtere für Anzeige basierend auf aktivem Tab
  const displayedReminders = filter === 'open'
    ? openReminders
    : filter === 'completed'
    ? completedReminders
    : allReminders;

  // 🚀 Virtual Scrolling Setup (Performance-Optimierung)
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: displayedReminders.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 180, // Geschätzte Höhe pro Reminder-Card in px
    overscan: 5, // Rendere 5 Items außerhalb des sichtbaren Bereichs
  });

  return (
    <div className="h-full flex flex-col bg-gradient-to-br from-slate-50 to-slate-100">
      {/* Header */}
      <div className="bg-white border-b border-slate-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="bg-gradient-to-br from-purple-500 to-purple-600 p-3 rounded-xl shadow-lg">
              <Bell className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-slate-800">Erinnerungen</h1>
              <p className="text-sm text-slate-600">
                {openReminders.length} offen · {completedReminders.length} erledigt
              </p>
            </div>
          </div>
          <button
            onClick={() => setShowCreateDialog(true)}
            className="flex items-center gap-2 px-4 py-3 bg-purple-600 hover:bg-purple-700 text-white rounded-lg font-semibold transition-colors shadow-lg hover:shadow-xl"
          >
            <Plus className="w-5 h-5" />
            Neue Erinnerung
          </button>
        </div>

        {/* Filters */}
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-slate-500" />
          <button
            onClick={() => setFilter('open')}
            className={`px-4 py-2 rounded-lg font-medium transition-colors ${
              filter === 'open'
                ? 'bg-purple-100 text-purple-700 border border-purple-200'
                : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
            }`}
          >
            Offen ({openReminders.length})
          </button>
          <button
            onClick={() => setFilter('completed')}
            className={`px-4 py-2 rounded-lg font-medium transition-colors ${
              filter === 'completed'
                ? 'bg-purple-100 text-purple-700 border border-purple-200'
                : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
            }`}
          >
            Erledigt ({completedReminders.length})
          </button>
          <button
            onClick={() => setFilter('all')}
            className={`px-4 py-2 rounded-lg font-medium transition-colors ${
              filter === 'all'
                ? 'bg-purple-100 text-purple-700 border border-purple-200'
                : 'bg-slate-100 text-slate-600 hover:bg-slate-200'
            }`}
          >
            Alle ({allReminders.length})
          </button>
        </div>
      </div>

      {/* Content */}
      <div ref={parentRef} className="flex-1 overflow-y-auto p-6">
        {loading ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-center">
              <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
              <p className="mt-4 text-slate-600">Lade Erinnerungen...</p>
            </div>
          </div>
        ) : displayedReminders.length === 0 ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-center">
              <Bell className="w-16 h-16 text-slate-300 mx-auto mb-4" />
              <p className="text-lg font-medium text-slate-600 mb-2">
                {filter === 'open' && 'Keine offenen Erinnerungen'}
                {filter === 'completed' && 'Keine erledigten Erinnerungen'}
                {filter === 'all' && 'Noch keine Erinnerungen'}
              </p>
              <p className="text-sm text-slate-500">
                Klicken Sie auf "Neue Erinnerung" um eine Erinnerung anzulegen
              </p>
            </div>
          </div>
        ) : (
          <div className="max-w-4xl mx-auto">
            {/* Virtual Scrolling Container */}
            <div
              style={{
                height: `${virtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {virtualizer.getVirtualItems().map((virtualItem) => {
                const reminder = displayedReminders[virtualItem.index];
                return (
                  <div
                    key={reminder.id}
                    data-index={virtualItem.index}
                    ref={virtualizer.measureElement}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: '100%',
                      transform: `translateY(${virtualItem.start}px)`,
                      paddingBottom: '16px', // gap-4
                    }}
                  >
                    <div
                      className={`border-2 rounded-xl p-5 transition-all ${
                        reminder.is_completed
                          ? 'bg-slate-50 border-slate-200 opacity-60'
                          : getPriorityColor(reminder.priority)
                      }`}
                    >
                <div className="flex items-start gap-4">
                  {/* Priority Icon */}
                  <div className="flex-shrink-0 mt-1">
                    {reminder.is_completed ? (
                      <CheckCircle className="w-6 h-6 text-emerald-600" />
                    ) : (
                      <div className={`p-2 rounded-lg ${getPriorityBadgeColor(reminder.priority)}`}>
                        {getPriorityIcon(reminder.priority)}
                      </div>
                    )}
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex-1">
                        <h3 className={`text-lg font-bold text-slate-900 mb-1 ${reminder.is_completed ? 'line-through' : ''}`}>
                          {reminder.title}
                        </h3>
                        {reminder.description && (
                          <p className="text-sm text-slate-600 mb-3">{reminder.description}</p>
                        )}
                        <div className="flex items-center gap-3 flex-wrap">
                          <span className={`flex items-center gap-1 text-sm font-medium px-3 py-1 rounded-full ${
                            isOverdue(reminder.due_date) && !reminder.is_completed
                              ? 'bg-red-100 text-red-700'
                              : 'bg-slate-100 text-slate-700'
                          }`}>
                            <Calendar className="w-3.5 h-3.5" />
                            {formatDate(reminder.due_date)}
                            {isOverdue(reminder.due_date) && !reminder.is_completed && ' (Überfällig)'}
                          </span>
                          <span className="text-xs px-2 py-1 bg-white border border-slate-200 rounded-full">
                            {getTypeLabel(reminder.reminder_type)}
                          </span>
                          <span className={`text-xs px-2 py-1 rounded-full font-medium ${getPriorityBadgeColor(reminder.priority)}`}>
                            {reminder.priority === 'high' && 'Hoch'}
                            {reminder.priority === 'medium' && 'Mittel'}
                            {reminder.priority === 'low' && 'Niedrig'}
                          </span>
                          {reminder.booking_id && (
                            <button
                              onClick={() => onNavigateToBooking && onNavigateToBooking(reminder.booking_id!)}
                              className="flex items-center gap-1 text-xs text-blue-600 hover:text-blue-700 font-medium"
                            >
                              <ExternalLink className="w-3.5 h-3.5" />
                              Buchung #{reminder.booking_id}
                            </button>
                          )}
                        </div>
                        {reminder.is_completed && reminder.completed_at && (
                          <p className="text-xs text-slate-500 mt-2">
                            Erledigt am {formatDate(reminder.completed_at.split('T')[0])}
                          </p>
                        )}
                      </div>

                      {/* Actions */}
                      <div className="flex items-center gap-1">
                        {!reminder.is_completed ? (
                          <>
                            <button
                              onClick={() => handleMarkCompleted(reminder.id, true)}
                              className="p-2 hover:bg-white rounded-lg transition-colors"
                              title="Als erledigt markieren"
                            >
                              <CheckCircle className="w-5 h-5 text-emerald-600" />
                            </button>
                            <button
                              onClick={() => setEditingReminder(reminder)}
                              className="p-2 hover:bg-white rounded-lg transition-colors"
                              title="Bearbeiten"
                            >
                              <Edit2 className="w-5 h-5 text-blue-600" />
                            </button>
                            <button
                              onClick={() => handleDelete(reminder)}
                              className="p-2 hover:bg-white rounded-lg transition-colors"
                              title="Löschen"
                            >
                              <Trash2 className="w-5 h-5 text-red-600" />
                            </button>
                          </>
                        ) : (
                          <>
                            <button
                              onClick={() => handleMarkCompleted(reminder.id, false)}
                              className="p-2 hover:bg-white rounded-lg transition-colors"
                              title="Als unerledigt markieren"
                            >
                              <Clock className="w-5 h-5 text-slate-600" />
                            </button>
                            <button
                              onClick={() => handleDelete(reminder)}
                              className="p-2 hover:bg-white rounded-lg transition-colors"
                              title="Löschen"
                            >
                              <Trash2 className="w-5 h-5 text-red-600" />
                            </button>
                          </>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>

      {/* Create/Edit Dialog */}
      {(showCreateDialog || editingReminder) && (
        <ReminderDialog
          reminder={editingReminder}
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

      {/* Delete Confirmation Dialog */}
      {deleteDialogReminder && (
        <DeleteConfirmDialog
          reminder={deleteDialogReminder}
          onConfirm={confirmDelete}
          onCancel={() => setDeleteDialogReminder(null)}
        />
      )}
    </div>
  );
}

interface ReminderDialogProps {
  reminder?: Reminder | null;
  onSave: (data: CreateReminderData | UpdateReminderData) => void;
  onClose: () => void;
}

function ReminderDialog({ reminder, onSave, onClose }: ReminderDialogProps) {
  const [title, setTitle] = useState(reminder?.title || '');
  const [description, setDescription] = useState(reminder?.description || '');
  const [dueDate, setDueDate] = useState(reminder?.due_date || new Date().toISOString().split('T')[0]);
  const [priority, setPriority] = useState<'low' | 'medium' | 'high'>(reminder?.priority || 'medium');
  const [bookingId, setBookingId] = useState<string>(reminder?.booking_id?.toString() || '');

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
        booking_id: bookingId ? parseInt(bookingId) : null,
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
            <Bell className="w-5 h-5 text-slate-400" />
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

          {!reminder && (
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Buchungs-ID (optional)
              </label>
              <input
                type="number"
                value={bookingId}
                onChange={(e) => setBookingId(e.target.value)}
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-purple-500"
                placeholder="z.B. 42"
              />
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Fälligkeitsdatum *
            </label>
            <FilterDatePicker
              value={dueDate}
              onChange={setDueDate}
              placeholder="Datum auswählen"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-purple-500 cursor-pointer"
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

interface DeleteConfirmDialogProps {
  reminder: Reminder;
  onConfirm: () => void;
  onCancel: () => void;
}

function DeleteConfirmDialog({ reminder, onConfirm, onCancel }: DeleteConfirmDialogProps) {
  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[70]">
      <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4">
        {/* Header */}
        <div className="flex items-start gap-4 mb-6">
          <div className="p-3 bg-red-500/10 rounded-full flex-shrink-0">
            <AlertTriangle className="w-6 h-6 text-red-400" />
          </div>
          <div className="flex-1">
            <h3 className="text-xl font-bold text-white mb-2">Erinnerung löschen?</h3>
            <p className="text-slate-300 text-sm">
              Diese Aktion kann nicht rückgängig gemacht werden.
            </p>
          </div>
        </div>

        {/* Reminder Details */}
        <div className="mb-6 p-4 bg-slate-700/50 rounded-lg border border-slate-600">
          <p className="text-white font-semibold mb-1">{reminder.title}</p>
          {reminder.description && (
            <p className="text-slate-300 text-sm mb-2">{reminder.description}</p>
          )}
          <div className="flex items-center gap-2 text-xs text-slate-400">
            <Calendar className="w-3 h-3" />
            <span>{formatDate(reminder.due_date)}</span>
            <span>•</span>
            <span className="capitalize">{reminder.priority} Priorität</span>
          </div>
        </div>

        {/* Buttons */}
        <div className="flex gap-3">
          <button
            onClick={onCancel}
            className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
          >
            Abbrechen
          </button>
          <button
            onClick={onConfirm}
            className="flex-1 px-4 py-3 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors"
          >
            Löschen
          </button>
        </div>
      </div>
    </div>
  );
}
