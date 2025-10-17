import { useState, useEffect } from 'react';
import { X, Hotel, MapPin, Users, Euro, Key } from 'lucide-react';
import { useData } from '../../context/DataContext';

interface Room {
  id?: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  price_member: number;
  price_non_member: number;
  nebensaison_preis: number;
  hauptsaison_preis: number;
  endreinigung: number;
  ort: string;
  schluesselcode?: string;
  streetAddress?: string;
  postalCode?: string;
  city?: string;
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
    price_member: 0,        // Deprecated - kept for backward compatibility
    price_non_member: 0,     // Deprecated - kept for backward compatibility
    nebensaison_preis: 0,
    hauptsaison_preis: 0,
    endreinigung: 0,
    ort: '',
    schluesselcode: '',
    streetAddress: '',
    postalCode: '',
    city: '',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [debugInfo, setDebugInfo] = useState<any>(null);

  useEffect(() => {
    if (room) {
      setFormData(room);
    } else {
      setFormData({
        name: '',
        gebaeude_typ: '',
        capacity: 1,
        price_member: 0,
        price_non_member: 0,
        nebensaison_preis: 0,
        hauptsaison_preis: 0,
        endreinigung: 0,
        ort: '',
        schluesselcode: '',
        streetAddress: '',
        postalCode: '',
        city: '',
      });
    }
    setError(null);
  }, [room, isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      // Capture debug info BEFORE sending
      const debugData = {
        step: 'START',
        formData: JSON.parse(JSON.stringify(formData)),
        formDataAddressFields: {
          streetAddress: formData.streetAddress,
          streetAddressType: typeof formData.streetAddress,
          postalCode: formData.postalCode,
          postalCodeType: typeof formData.postalCode,
          city: formData.city,
          cityType: typeof formData.city,
        }
      };
      setDebugInfo(debugData);

      console.log('═══════════════════════════════════════');
      console.log('🔍 [RoomDialog] handleSubmit START');
      console.log('  📦 formData RAW:', JSON.stringify(formData, null, 2));
      console.log('  📍 formData.streetAddress:', formData.streetAddress, '(type:', typeof formData.streetAddress, ')');
      console.log('  📍 formData.postalCode:', formData.postalCode, '(type:', typeof formData.postalCode, ')');
      console.log('  📍 formData.city:', formData.city, '(type:', typeof formData.city, ')');

      const roomData = {
        name: formData.name,
        gebaeudeTyp: formData.gebaeude_typ,
        capacity: formData.capacity,
        priceMember: formData.price_member,
        priceNonMember: formData.price_non_member,
        nebensaisonPreis: formData.nebensaison_preis,
        hauptsaisonPreis: formData.hauptsaison_preis,
        endreinigung: formData.endreinigung,
        ort: formData.ort,
        schluesselcode: formData.schluesselcode || null,
        streetAddress: formData.streetAddress || null,
        postalCode: formData.postalCode || null,
        city: formData.city || null,
      };

      // Update debug info with roomData
      setDebugInfo({
        ...debugData,
        step: 'PREPARED',
        roomData: JSON.parse(JSON.stringify(roomData)),
        roomDataAddressFields: {
          streetAddress: roomData.streetAddress,
          streetAddressType: typeof roomData.streetAddress,
          postalCode: roomData.postalCode,
          postalCodeType: typeof roomData.postalCode,
          city: roomData.city,
          cityType: typeof roomData.city,
        }
      });

      console.log('  📤 roomData PREPARED:', JSON.stringify(roomData, null, 2));
      console.log('  📍 roomData.streetAddress:', roomData.streetAddress, '(type:', typeof roomData.streetAddress, ')');
      console.log('  📍 roomData.postalCode:', roomData.postalCode, '(type:', typeof roomData.postalCode, ')');
      console.log('  📍 roomData.city:', roomData.city, '(type:', typeof roomData.city, ')');

      let result;
      if (room?.id) {
        // Update existing room (Optimistic Update via DataContext)
        console.log('  🔄 UPDATE MODE - calling updateRoom with id:', room.id);
        setDebugInfo(prev => ({ ...prev, step: 'SENDING', mode: 'UPDATE', id: room.id }));
        result = await updateRoom(room.id, roomData);
        console.log('  ✅ updateRoom returned successfully');
      } else {
        // Create new room (Optimistic Update via DataContext)
        console.log('  ➕ CREATE MODE - calling createRoom');
        setDebugInfo(prev => ({ ...prev, step: 'SENDING', mode: 'CREATE' }));
        result = await createRoom(roomData);
        console.log('  ✅ createRoom returned successfully');
      }

      // Show result from backend
      setDebugInfo(prev => ({
        ...prev,
        step: 'SUCCESS',
        backendResult: JSON.parse(JSON.stringify(result)),
        backendAddressFields: {
          street_address: result.street_address,
          postal_code: result.postal_code,
          city: result.city,
        }
      }));

      console.log('═══════════════════════════════════════');

      // Wait 3 seconds so user can see debug info
      await new Promise(resolve => setTimeout(resolve, 3000));

      onSuccess();
      onClose();
      setDebugInfo(null);
    } catch (err) {
      console.error('❌ [RoomDialog] Fehler beim Speichern des Zimmers:', err);
      setDebugInfo(prev => ({
        ...prev,
        step: 'ERROR',
        error: err instanceof Error ? err.message : String(err)
      }));
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
          </div>
        </form>

        {/* DEBUG PANEL - VISIBLE IN UI */}
        {debugInfo && (
          <div className="mx-6 mb-6 p-6 bg-yellow-50 border-2 border-yellow-500 rounded-xl">
            <h3 className="text-lg font-bold text-yellow-900 mb-4 flex items-center gap-2">
              🐛 DEBUG INFO - {debugInfo.step}
            </h3>

            <div className="space-y-4 text-sm">
              {/* Step Indicator */}
              <div className="flex gap-2 mb-4">
                <span className={`px-3 py-1 rounded-full text-xs font-bold ${debugInfo.step === 'START' ? 'bg-blue-500 text-white' : 'bg-gray-300'}`}>
                  START
                </span>
                <span className={`px-3 py-1 rounded-full text-xs font-bold ${debugInfo.step === 'PREPARED' ? 'bg-blue-500 text-white' : 'bg-gray-300'}`}>
                  PREPARED
                </span>
                <span className={`px-3 py-1 rounded-full text-xs font-bold ${debugInfo.step === 'SENDING' ? 'bg-blue-500 text-white' : 'bg-gray-300'}`}>
                  SENDING
                </span>
                <span className={`px-3 py-1 rounded-full text-xs font-bold ${debugInfo.step === 'SUCCESS' ? 'bg-green-500 text-white' : debugInfo.step === 'ERROR' ? 'bg-red-500 text-white' : 'bg-gray-300'}`}>
                  {debugInfo.step === 'ERROR' ? 'ERROR' : 'SUCCESS'}
                </span>
              </div>

              {/* FormData Address Fields */}
              <div className="bg-white p-4 rounded-lg border border-yellow-400">
                <h4 className="font-bold text-yellow-900 mb-2">📥 formData Address Fields:</h4>
                <div className="font-mono text-xs space-y-1">
                  <div>streetAddress: <span className="text-blue-600">"{debugInfo.formDataAddressFields?.streetAddress}"</span> (type: {debugInfo.formDataAddressFields?.streetAddressType})</div>
                  <div>postalCode: <span className="text-blue-600">"{debugInfo.formDataAddressFields?.postalCode}"</span> (type: {debugInfo.formDataAddressFields?.postalCodeType})</div>
                  <div>city: <span className="text-blue-600">"{debugInfo.formDataAddressFields?.city}"</span> (type: {debugInfo.formDataAddressFields?.cityType})</div>
                </div>
              </div>

              {/* RoomData Address Fields */}
              {debugInfo.roomDataAddressFields && (
                <div className="bg-white p-4 rounded-lg border border-yellow-400">
                  <h4 className="font-bold text-yellow-900 mb-2">📤 roomData Address Fields (sent to backend):</h4>
                  <div className="font-mono text-xs space-y-1">
                    <div>streetAddress: <span className="text-purple-600">"{debugInfo.roomDataAddressFields.streetAddress}"</span> (type: {debugInfo.roomDataAddressFields.streetAddressType})</div>
                    <div>postalCode: <span className="text-purple-600">"{debugInfo.roomDataAddressFields.postalCode}"</span> (type: {debugInfo.roomDataAddressFields.postalCodeType})</div>
                    <div>city: <span className="text-purple-600">"{debugInfo.roomDataAddressFields.city}"</span> (type: {debugInfo.roomDataAddressFields.cityType})</div>
                  </div>
                </div>
              )}

              {/* Backend Result */}
              {debugInfo.backendAddressFields && (
                <div className="bg-white p-4 rounded-lg border border-green-500">
                  <h4 className="font-bold text-green-900 mb-2">✅ Backend Returned (snake_case):</h4>
                  <div className="font-mono text-xs space-y-1">
                    <div>street_address: <span className="text-green-600">"{debugInfo.backendAddressFields.street_address}"</span></div>
                    <div>postal_code: <span className="text-green-600">"{debugInfo.backendAddressFields.postal_code}"</span></div>
                    <div>city: <span className="text-green-600">"{debugInfo.backendAddressFields.city}"</span></div>
                  </div>
                </div>
              )}

              {/* Mode Info */}
              {debugInfo.mode && (
                <div className="bg-blue-100 p-3 rounded-lg">
                  <div className="font-bold">Mode: {debugInfo.mode}</div>
                  {debugInfo.id && <div className="text-xs">Room ID: {debugInfo.id}</div>}
                </div>
              )}

              {/* Error */}
              {debugInfo.error && (
                <div className="bg-red-100 border border-red-400 p-3 rounded-lg">
                  <div className="font-bold text-red-900">❌ Error:</div>
                  <div className="text-xs text-red-700">{debugInfo.error}</div>
                </div>
              )}
            </div>
          </div>
        )}

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
