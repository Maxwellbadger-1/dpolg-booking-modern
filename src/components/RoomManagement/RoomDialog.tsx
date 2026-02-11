import { useState, useEffect } from 'react';
import { X, Hotel, MapPin, Users, Euro, Key } from 'lucide-react';
import { useData } from '../../context/DataContext';

interface Room {
  id?: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  nebensaison_preis: number;
  hauptsaison_preis: number;
  endreinigung: number;
  ort: string;
  schluesselcode?: string;
  streetAddress?: string;
  postalCode?: string;
  city?: string;
  notizen?: string;
}

interface RoomDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  room?: Room;
}

export default function RoomDialog({ isOpen, onClose, onSuccess, room }: RoomDialogProps) {
  const { createRoom, updateRoom } = useData();
  const [formData, setFormData] = useState<Room>({
    name: '',
    gebaeude_typ: '',
    capacity: 1,
    nebensaison_preis: 0,
    hauptsaison_preis: 0,
    endreinigung: 0,
    ort: '',
    schluesselcode: '',
    streetAddress: '',
    postalCode: '',
    city: '',
    notizen: '',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (room) {
      setFormData(room);
    } else {
      setFormData({
        name: '',
        gebaeude_typ: '',
        capacity: 1,
        nebensaison_preis: 0,
        hauptsaison_preis: 0,
        endreinigung: 0,
        ort: '',
        schluesselcode: '',
        streetAddress: '',
        postalCode: '',
        city: '',
        notizen: '',
      });
    }
    setError(null);
  }, [room, isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      const roomData = {
        name: formData.name,
        gebaeudeTyp: formData.gebaeude_typ,
        capacity: formData.capacity,
        nebensaisonPreis: formData.nebensaison_preis,
        hauptsaisonPreis: formData.hauptsaison_preis,
        endreinigung: formData.endreinigung,
        ort: formData.ort,
        schluesselcode: formData.schluesselcode || null,
        streetAddress: formData.streetAddress || null,
        postalCode: formData.postalCode || null,
        city: formData.city || null,
        notizen: formData.notizen || null,
      };

      if (room?.id) {
        // Update existing room (Optimistic Update via DataContext)
        await updateRoom(room.id, roomData);
      } else {
        // Create new room (Optimistic Update via DataContext)
        await createRoom(roomData);
      }

      onSuccess();
      onClose();
    } catch (err) {
      console.error('Fehler beim Speichern des Zimmers:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-500 to-blue-600 px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="bg-white/20 p-2 rounded-lg">
              <Hotel className="w-6 h-6 text-white" />
            </div>
            <h2 className="text-xl font-bold text-white">
              {room ? 'Zimmer bearbeiten' : 'Neues Zimmer'}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="text-white/80 hover:text-white transition-colors p-1 rounded-lg hover:bg-white/10"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 overflow-y-auto max-h-[calc(90vh-200px)]">
          {error && (
            <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-700">{error}</p>
            </div>
          )}

          <div className="space-y-6">
            {/* Zimmer Name */}
            <div>
              <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                <Hotel className="w-4 h-4" />
                Zimmer Name *
              </label>
              <input
                type="text"
                required
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="z.B. Zimmer 101"
              />
            </div>

            {/* Gebäude Typ & Ort */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-semibold text-slate-700 mb-2">
                  Gebäude/Typ *
                </label>
                <input
                  type="text"
                  required
                  value={formData.gebaeude_typ}
                  onChange={(e) => setFormData({ ...formData, gebaeude_typ: e.target.value })}
                  className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="z.B. Hauptgebäude"
                />
              </div>
              <div>
                <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                  <MapPin className="w-4 h-4" />
                  Ort *
                </label>
                <input
                  type="text"
                  required
                  value={formData.ort}
                  onChange={(e) => setFormData({ ...formData, ort: e.target.value })}
                  className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="z.B. Berlin"
                />
              </div>
            </div>

            {/* Adresse (optional) */}
            <div className="border border-slate-200 rounded-lg p-4 bg-slate-50">
              <h3 className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-4">
                <MapPin className="w-4 h-4" />
                Adresse (optional)
              </h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-slate-600 mb-2">
                    Straße & Hausnummer
                  </label>
                  <input
                    type="text"
                    value={formData.streetAddress || ''}
                    onChange={(e) => setFormData({ ...formData, streetAddress: e.target.value })}
                    className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    placeholder="z.B. Hauptstraße 123"
                  />
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-slate-600 mb-2">
                      PLZ
                    </label>
                    <input
                      type="text"
                      value={formData.postalCode || ''}
                      onChange={(e) => setFormData({ ...formData, postalCode: e.target.value })}
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder="z.B. 10115"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-slate-600 mb-2">
                      Stadt
                    </label>
                    <input
                      type="text"
                      value={formData.city || ''}
                      onChange={(e) => setFormData({ ...formData, city: e.target.value })}
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder="z.B. Berlin"
                    />
                  </div>
                </div>
                <p className="text-xs text-slate-500">
                  Diese Adresse kann in E-Mail-Vorlagen verwendet werden
                </p>
              </div>
            </div>

            {/* Capacity */}
            <div>
              <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                <Users className="w-4 h-4" />
                Kapazität (Personen) *
              </label>
              <input
                type="number"
                required
                min="1"
                max="20"
                value={formData.capacity}
                onChange={(e) => setFormData({ ...formData, capacity: parseInt(e.target.value) || 1 })}
                className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Preise - Preisliste 2025 */}
            <div className="border border-slate-200 rounded-lg p-4 bg-slate-50">
              <h3 className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-4">
                <Euro className="w-4 h-4" />
                Preise pro Nacht (Preisliste 2025)
              </h3>

              {/* Saisonpreise */}
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-slate-600 mb-2">
                      Nebensaison *
                    </label>
                    <div className="relative">
                      <input
                        type="number"
                        required
                        min="0"
                        step="0.01"
                        value={formData.nebensaison_preis}
                        onChange={(e) => setFormData({ ...formData, nebensaison_preis: parseFloat(e.target.value) || 0 })}
                        className="w-full px-4 py-2 pr-8 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        placeholder="0.00"
                      />
                      <span className="absolute right-3 top-1/2 transform -translate-y-1/2 text-slate-400 text-sm">€</span>
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm text-slate-600 mb-2">
                      Hauptsaison *
                    </label>
                    <div className="relative">
                      <input
                        type="number"
                        required
                        min="0"
                        step="0.01"
                        value={formData.hauptsaison_preis}
                        onChange={(e) => setFormData({ ...formData, hauptsaison_preis: parseFloat(e.target.value) || 0 })}
                        className="w-full px-4 py-2 pr-8 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        placeholder="0.00"
                      />
                      <span className="absolute right-3 top-1/2 transform -translate-y-1/2 text-slate-400 text-sm">€</span>
                    </div>
                  </div>
                </div>

                {/* Endreinigung */}
                <div>
                  <label className="block text-sm text-slate-600 mb-2">
                    Endreinigung (einmalig) *
                  </label>
                  <div className="relative">
                    <input
                      type="number"
                      required
                      min="0"
                      step="0.01"
                      value={formData.endreinigung}
                      onChange={(e) => setFormData({ ...formData, endreinigung: parseFloat(e.target.value) || 0 })}
                      className="w-full px-4 py-2 pr-8 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder="0.00"
                    />
                    <span className="absolute right-3 top-1/2 transform -translate-y-1/2 text-slate-400 text-sm">€</span>
                  </div>
                  <p className="text-xs text-slate-500 mt-1">Wird automatisch zu jeder Buchung hinzugefügt</p>
                </div>

                {/* Info Box */}
                <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
                  <p className="text-xs text-blue-700">
                    <strong>Hauptsaison:</strong> 01.06-15.09.2025 & 22.12-28.02.2026<br />
                    <strong>DPolG-Rabatt:</strong> 15% automatisch für Mitglieder
                  </p>
                </div>
              </div>
            </div>

            {/* Schlüsselcode */}
            <div>
              <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                <Key className="w-4 h-4" />
                Schlüsselcode (optional)
              </label>
              <input
                type="text"
                value={formData.schluesselcode || ''}
                onChange={(e) => setFormData({ ...formData, schluesselcode: e.target.value })}
                className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                placeholder="z.B. 1234"
              />
            </div>

            {/* Notizen */}
            <div>
              <label className="block text-sm font-semibold text-slate-700 mb-2">
                Notizen (optional)
              </label>
              <textarea
                value={formData.notizen || ''}
                onChange={(e) => setFormData({ ...formData, notizen: e.target.value })}
                rows={4}
                className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                placeholder="Zusätzliche Informationen zum Zimmer..."
              />
            </div>
          </div>
        </form>

        {/* Footer */}
        <div className="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-slate-700 hover:bg-slate-200 rounded-lg transition-colors font-semibold"
          >
            Abbrechen
          </button>
          <button
            onClick={handleSubmit}
            disabled={loading}
            className="px-6 py-2 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white rounded-lg font-semibold shadow-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? 'Speichere...' : room ? 'Aktualisieren' : 'Erstellen'}
          </button>
        </div>
      </div>
    </div>
  );
}
