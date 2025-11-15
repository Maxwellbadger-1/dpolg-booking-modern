import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Plus, Trash2, Edit2, Users, AlertCircle, Check } from 'lucide-react';
import type { GuestCompanion, AccompanyingGuest } from '../../types/booking';
import { formatDate } from '../../utils/dateFormatting';

interface CompanionSelectorProps {
  guestId: number; // Hauptgast
  bookingId?: number; // Optional: Wenn Buchung bereits existiert
  roomCapacity?: number; // Zimmerkapazität (für Validierung)
  onCompanionsChange: (companions: AccompanyingGuest[]) => void;
}

export default function CompanionSelector({ guestId, bookingId, roomCapacity, onCompanionsChange }: CompanionSelectorProps) {
  const [companions, setCompanions] = useState<GuestCompanion[]>([]);
  const [selectedCompanionIds, setSelectedCompanionIds] = useState<Set<number>>(new Set());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);

  // Berechne maximale Anzahl Begleitpersonen (Zimmerkapazität - 1 Hauptgast)
  const maxCompanions = roomCapacity ? roomCapacity - 1 : undefined;
  const isAtCapacity = maxCompanions !== undefined && selectedCompanionIds.size >= maxCompanions;

  // Neue Begleitperson Formular
  const [newCompanion, setNewCompanion] = useState({
    vorname: '',
    nachname: '',
    geburtsdatum: '',
    beziehung: '',
    notizen: '',
  });

  // Pool von Begleitpersonen laden
  useEffect(() => {
    loadCompanions();
  }, [guestId]);

  // Wenn Buchung bereits existiert, lade die bereits zugewiesenen Begleitpersonen
  useEffect(() => {
    if (bookingId) {
      loadBookingCompanions();
    }
  }, [bookingId]);

  const loadCompanions = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<GuestCompanion[]>('get_guest_companions_command', { guestId });
      setCompanions(result);
    } catch (err) {
      console.error('Fehler beim Laden der Begleitpersonen:', err);
      setError(err instanceof Error ? err.message : 'Fehler beim Laden');
    } finally {
      setLoading(false);
    }
  };

  const loadBookingCompanions = async () => {
    if (!bookingId) return;
    try {
      const result = await invoke<AccompanyingGuest[]>('get_booking_accompanying_guests_command', { bookingId });
      // Markiere Begleitpersonen, die bereits zur Buchung gehören
      const companionIds = new Set(result.filter(ag => ag.companion_id).map(ag => ag.companion_id!));
      setSelectedCompanionIds(companionIds);
    } catch (err) {
      console.error('Fehler beim Laden der Buchungs-Begleitpersonen:', err);
    }
  };

  const handleAddCompanion = async () => {
    if (!newCompanion.vorname.trim() || !newCompanion.nachname.trim()) {
      setError('Vorname und Nachname sind Pflichtfelder');
      return;
    }

    // Kapazitätsprüfung
    if (isAtCapacity) {
      setError(`Maximale Kapazität erreicht! Das Zimmer bietet Platz für maximal ${roomCapacity} Personen (1 Hauptgast + ${maxCompanions} Begleitpersonen).`);
      return;
    }

    try {
      setError(null);
      const created = await invoke<GuestCompanion>('create_guest_companion_command', {
        guestId,
        vorname: newCompanion.vorname.trim(),
        nachname: newCompanion.nachname.trim(),
        geburtsdatum: newCompanion.geburtsdatum || null,
        beziehung: newCompanion.beziehung || null,
        notizen: newCompanion.notizen || null,
      });

      // Füge zur Liste hinzu und markiere als ausgewählt
      setCompanions([...companions, created]);
      setSelectedCompanionIds(new Set([...selectedCompanionIds, created.id]));

      // Formular zurücksetzen
      setNewCompanion({
        vorname: '',
        nachname: '',
        geburtsdatum: '',
        beziehung: '',
        notizen: '',
      });
      setShowAddForm(false);

      // Aktualisiere Parent-Komponente
      updateParent(new Set([...selectedCompanionIds, created.id]));
    } catch (err) {
      console.error('Fehler beim Erstellen der Begleitperson:', err);
      setError(err instanceof Error ? err.message : 'Fehler beim Erstellen');
    }
  };

  const handleDeleteCompanion = async (id: number) => {
    if (!confirm('Begleitperson wirklich aus dem Pool löschen?')) return;

    try {
      await invoke('delete_guest_companion_command', { id });
      setCompanions(companions.filter(c => c.id !== id));
      selectedCompanionIds.delete(id);
      setSelectedCompanionIds(new Set(selectedCompanionIds));
      updateParent(selectedCompanionIds);
    } catch (err) {
      console.error('Fehler beim Löschen:', err);
      setError(err instanceof Error ? err.message : 'Fehler beim Löschen');
    }
  };

  const handleToggleCompanion = (companionId: number) => {
    const newSelected = new Set(selectedCompanionIds);
    if (newSelected.has(companionId)) {
      // Abwählen ist immer erlaubt
      newSelected.delete(companionId);
    } else {
      // Hinzufügen nur erlaubt wenn Kapazität nicht erreicht
      if (isAtCapacity) {
        setError(`Maximale Kapazität erreicht! Das Zimmer bietet Platz für maximal ${roomCapacity} Personen (1 Hauptgast + ${maxCompanions} Begleitpersonen).`);
        return;
      }
      newSelected.add(companionId);
    }
    setError(null); // Clear error on successful action
    setSelectedCompanionIds(newSelected);
    updateParent(newSelected);
  };

  // Benachrichtige Parent-Komponente über Änderungen
  const updateParent = (selected: Set<number>) => {
    const selectedCompanions: AccompanyingGuest[] = companions
      .filter(c => selected.has(c.id))
      .map(c => ({
        id: 0, // Wird beim Speichern gesetzt
        booking_id: bookingId || 0,
        vorname: c.vorname,
        nachname: c.nachname,
        geburtsdatum: c.geburtsdatum,
        companion_id: c.id,
      }));
    onCompanionsChange(selectedCompanions);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Users className="w-5 h-5 text-slate-400" />
          <h3 className="text-lg font-semibold text-slate-900">Begleitpersonen</h3>
          {selectedCompanionIds.size > 0 && (
            <span className="px-2 py-1 bg-blue-100 text-blue-700 text-sm font-medium rounded-full">
              {selectedCompanionIds.size} ausgewählt
            </span>
          )}
          {roomCapacity && (
            <span className={`px-2 py-1 text-xs font-medium rounded-full ${
              isAtCapacity
                ? 'bg-red-100 text-red-700'
                : selectedCompanionIds.size > 0
                ? 'bg-amber-100 text-amber-700'
                : 'bg-slate-100 text-slate-600'
            }`}>
              {selectedCompanionIds.size} / {maxCompanions} Plätze
            </span>
          )}
        </div>
        <button
          type="button"
          onClick={() => setShowAddForm(!showAddForm)}
          disabled={isAtCapacity}
          className="flex items-center gap-2 px-4 py-2 bg-emerald-500 hover:bg-emerald-600 disabled:bg-slate-300 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
          title={isAtCapacity ? 'Maximale Kapazität erreicht' : 'Neue Begleitperson hinzufügen'}
        >
          <Plus className="w-4 h-4" />
          Neue Person
        </button>
      </div>

      {/* Error Message */}
      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
          <AlertCircle className="w-4 h-4 flex-shrink-0" />
          {error}
        </div>
      )}

      {/* Add Companion Form */}
      {showAddForm && (
        <div className="p-6 bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl border border-slate-200 space-y-4">
          <h4 className="text-sm font-semibold text-slate-700 mb-3">Neue Begleitperson hinzufügen</h4>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Vorname *
              </label>
              <input
                type="text"
                value={newCompanion.vorname}
                onChange={(e) => setNewCompanion({ ...newCompanion, vorname: e.target.value })}
                className="w-full px-4 py-2 bg-white border border-slate-300 rounded-lg text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Max"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Nachname *
              </label>
              <input
                type="text"
                value={newCompanion.nachname}
                onChange={(e) => setNewCompanion({ ...newCompanion, nachname: e.target.value })}
                className="w-full px-4 py-2 bg-white border border-slate-300 rounded-lg text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Mustermann"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Geburtsdatum
              </label>
              <input
                type="date"
                value={newCompanion.geburtsdatum}
                onChange={(e) => setNewCompanion({ ...newCompanion, geburtsdatum: e.target.value })}
                className="w-full px-4 py-2 bg-white border border-slate-300 rounded-lg text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">
                Beziehung
              </label>
              <select
                value={newCompanion.beziehung}
                onChange={(e) => setNewCompanion({ ...newCompanion, beziehung: e.target.value })}
                className="w-full px-5 py-3.5 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                style={{
                  backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='%23475569' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E")`,
                  backgroundRepeat: 'no-repeat',
                  backgroundPosition: 'right 0.75rem center',
                  backgroundSize: '1.5rem',
                  paddingRight: '3rem'
                }}
              >
                <option value="">Bitte wählen...</option>
                <option value="Ehepartner">Ehepartner/in</option>
                <option value="Lebensgefährte">Lebensgefährte/in</option>
                <option value="Kind">Kind</option>
                <option value="Elternteil">Elternteil</option>
                <option value="Geschwister">Geschwister</option>
                <option value="Freund">Freund/in</option>
                <option value="Kollege">Kollege/in</option>
                <option value="Sonstige">Sonstige</option>
              </select>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">
              Notizen
            </label>
            <textarea
              value={newCompanion.notizen}
              onChange={(e) => setNewCompanion({ ...newCompanion, notizen: e.target.value })}
              className="w-full px-4 py-2 bg-white border border-slate-300 rounded-lg text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
              rows={2}
              placeholder="Optionale Notizen..."
            />
          </div>

          <div className="flex gap-3">
            <button
              type="button"
              onClick={handleAddCompanion}
              className="flex items-center gap-2 px-4 py-2 bg-emerald-500 hover:bg-emerald-600 text-white rounded-lg transition-colors"
            >
              <Check className="w-4 h-4" />
              Speichern
            </button>
            <button
              type="button"
              onClick={() => {
                setShowAddForm(false);
                setNewCompanion({ vorname: '', nachname: '', geburtsdatum: '', beziehung: '', notizen: '' });
              }}
              className="px-4 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-lg transition-colors"
            >
              Abbrechen
            </button>
          </div>
        </div>
      )}

      {/* Companions List */}
      {companions.length === 0 ? (
        <div className="p-8 text-center bg-slate-50 rounded-lg border border-slate-200">
          <Users className="w-12 h-12 text-slate-300 mx-auto mb-3" />
          <p className="text-slate-500 mb-2">Noch keine Begleitpersonen gespeichert</p>
          <p className="text-sm text-slate-400">Fügen Sie häufig mitreisende Personen hinzu für schnellere Buchungen</p>
        </div>
      ) : (
        <div className="grid gap-3">
          {companions.map((companion) => {
            const isSelected = selectedCompanionIds.has(companion.id);
            const isDisabled = !isSelected && isAtCapacity;

            return (
              <div
                key={companion.id}
                className={`p-4 rounded-lg border-2 transition-all ${
                  isDisabled
                    ? 'bg-slate-50 border-slate-200 opacity-50 cursor-not-allowed'
                    : isSelected
                    ? 'bg-blue-50 border-blue-300 cursor-pointer'
                    : 'bg-white border-slate-200 hover:border-slate-300 cursor-pointer'
                }`}
                onClick={() => !isDisabled && handleToggleCompanion(companion.id)}
                title={isDisabled ? 'Maximale Kapazität erreicht' : undefined}
              >
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-3 flex-1">
                  {/* Checkbox */}
                  <div className={`mt-1 w-5 h-5 rounded border-2 flex items-center justify-center transition-colors ${
                    selectedCompanionIds.has(companion.id)
                      ? 'bg-blue-500 border-blue-500'
                      : 'bg-white border-slate-300'
                  }`}>
                    {selectedCompanionIds.has(companion.id) && (
                      <Check className="w-3 h-3 text-white" />
                    )}
                  </div>

                  {/* Info */}
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <h4 className="font-semibold text-slate-900">
                        {companion.vorname} {companion.nachname}
                      </h4>
                      {companion.beziehung && (
                        <span className="px-2 py-0.5 bg-slate-100 text-slate-600 text-xs rounded-full">
                          {companion.beziehung}
                        </span>
                      )}
                    </div>
                    {companion.geburtsdatum && (
                      <p className="text-sm text-slate-500 mt-1">
                        Geboren: {formatDate(companion.geburtsdatum)}
                      </p>
                    )}
                    {companion.notizen && (
                      <p className="text-sm text-slate-600 mt-2 italic">
                        {companion.notizen}
                      </p>
                    )}
                  </div>
                </div>

                {/* Actions */}
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDeleteCompanion(companion.id);
                  }}
                  className="p-2 hover:bg-red-50 text-red-500 rounded-lg transition-colors"
                  title="Aus Pool löschen"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
            );
          })}
        </div>
      )}

      {/* Info Text */}
      <div className="text-xs text-slate-500 bg-slate-50 p-3 rounded-lg border border-slate-200">
        <strong>Hinweis:</strong> Hier gespeicherte Begleitpersonen stehen für zukünftige Buchungen zur Verfügung.
        Wählen Sie die Personen aus, die an dieser Buchung teilnehmen sollen.
      </div>
    </div>
  );
}
