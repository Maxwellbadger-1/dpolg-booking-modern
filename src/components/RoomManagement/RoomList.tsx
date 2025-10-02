import { useState, useEffect } from 'react';
import { Hotel, Search, Plus, Edit2, MapPin, Users, Euro, Trash2 } from 'lucide-react';
import { useData } from '../../context/DataContext';
import RoomDialog from './RoomDialog';
import ConfirmDialog from '../ConfirmDialog';

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  price_member: number;
  price_non_member: number;
  ort: string;
  schluesselcode?: string;
}

export default function RoomList() {
  // Get data from global context
  const { rooms, loading, deleteRoom } = useData();

  const [searchQuery, setSearchQuery] = useState('');
  const [locationFilter, setLocationFilter] = useState<string>('all');
  const [showDialog, setShowDialog] = useState(false);
  const [selectedRoom, setSelectedRoom] = useState<Room | undefined>(undefined);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [roomToDelete, setRoomToDelete] = useState<{ id: number; name: string } | null>(null);
  const [showErrorDialog, setShowErrorDialog] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string>('');

  const handleDeleteRoom = (id: number, name: string) => {
    setRoomToDelete({ id, name });
    setShowDeleteConfirm(true);
  };

  const confirmDelete = async () => {
    if (!roomToDelete) return;

    try {
      await deleteRoom(roomToDelete.id);
      setShowDeleteConfirm(false);
      setRoomToDelete(null);
    } catch (error) {
      console.error('Fehler beim Löschen des Zimmers:', error);

      // User-friendly error message
      const errMsg = error instanceof Error ? error.message : String(error);
      if (errMsg.includes('FOREIGN KEY') || errMsg.toLowerCase().includes('foreign key constraint')) {
        setErrorMessage(
          `Zimmer kann nicht gelöscht werden!\n\n` +
          `${roomToDelete.name} hat noch aktive Buchungen.\n\n` +
          `Bitte löschen Sie zuerst alle Buchungen für dieses Zimmer.`
        );
      } else {
        setErrorMessage(`Fehler beim Löschen des Zimmers:\n\n${errMsg}`);
      }

      setShowDeleteConfirm(false);
      setShowErrorDialog(true);
      // roomToDelete wird nach Schließen des Error Dialogs zurückgesetzt
    }
  };

  const cancelDelete = () => {
    setShowDeleteConfirm(false);
    setRoomToDelete(null);
  };

  const uniqueLocations = Array.from(new Set(rooms.map(r => r.ort))).sort();

  const filteredRooms = rooms.filter(room => {
    const matchesSearch =
      room.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      room.gebaeude_typ.toLowerCase().includes(searchQuery.toLowerCase()) ||
      room.ort.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesLocation = locationFilter === 'all' || room.ort === locationFilter;

    return matchesSearch && matchesLocation;
  });

  if (loading) {
    return (
      <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 p-6 flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
          <p className="text-slate-600">Lade Zimmer...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 p-6 overflow-auto">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-2xl font-bold text-slate-800">Zimmer</h2>
            <p className="text-sm text-slate-600 mt-1">
              {filteredRooms.length} {filteredRooms.length === 1 ? 'Zimmer' : 'Zimmer'}
            </p>
          </div>
          <button
            onClick={() => {
              setSelectedRoom(undefined);
              setShowDialog(true);
            }}
            className="flex items-center gap-2 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white px-4 py-2 rounded-lg font-semibold shadow-lg transition-all"
          >
            <Plus className="w-4 h-4" />
            Neues Zimmer
          </button>
        </div>

        {/* Search & Filter */}
        <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-4 mb-6">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-400 w-5 h-5" />
              <input
                type="text"
                placeholder="Suche nach Zimmername, Gebäude oder Ort..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <select
              value={locationFilter}
              onChange={(e) => setLocationFilter(e.target.value)}
              className="px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
            >
              <option value="all">Alle Orte</option>
              {uniqueLocations.map(location => (
                <option key={location} value={location}>{location}</option>
              ))}
            </select>
          </div>
        </div>

        {/* Rooms Grid */}
        {filteredRooms.length === 0 ? (
          <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-12">
            <div className="text-center">
              <Hotel className="w-16 h-16 text-slate-300 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-slate-800 mb-2">Keine Zimmer gefunden</h3>
              <p className="text-slate-600">
                {searchQuery || locationFilter !== 'all'
                  ? 'Versuche andere Suchkriterien oder Filter.'
                  : 'Erstelle dein erstes Zimmer mit dem Button oben.'
                }
              </p>
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredRooms.map((room) => (
              <div
                key={room.id}
                className="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden hover:shadow-lg transition-all group"
              >
                {/* Card Header */}
                <div className="bg-gradient-to-r from-blue-500 to-blue-600 px-6 py-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="bg-white/20 p-2 rounded-lg">
                        <Hotel className="w-5 h-5 text-white" />
                      </div>
                      <div>
                        <h3 className="text-lg font-bold text-white">{room.name}</h3>
                        <p className="text-sm text-blue-100">{room.gebaeude_typ}</p>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Card Body */}
                <div className="p-6 space-y-4">
                  {/* Location */}
                  <div className="flex items-center gap-2 text-sm text-slate-600">
                    <MapPin className="w-4 h-4 text-slate-400" />
                    <span>{room.ort}</span>
                  </div>

                  {/* Capacity */}
                  <div className="flex items-center gap-2 text-sm text-slate-600">
                    <Users className="w-4 h-4 text-slate-400" />
                    <span>Kapazität: {room.capacity} Personen</span>
                  </div>

                  {/* Pricing */}
                  <div className="pt-4 border-t border-slate-200 space-y-2">
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-slate-600">Mitglieder</span>
                      <span className="font-semibold text-emerald-600 flex items-center gap-1">
                        <Euro className="w-3.5 h-3.5" />
                        {room.price_member.toFixed(2)}
                      </span>
                    </div>
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-slate-600">Nicht-Mitglieder</span>
                      <span className="font-semibold text-slate-900 flex items-center gap-1">
                        <Euro className="w-3.5 h-3.5" />
                        {room.price_non_member.toFixed(2)}
                      </span>
                    </div>
                  </div>

                  {/* Key Code */}
                  {room.schluesselcode && (
                    <div className="pt-3 border-t border-slate-200">
                      <div className="text-xs text-slate-500">Schlüsselcode</div>
                      <div className="text-sm font-mono font-semibold text-slate-900 mt-1">
                        {room.schluesselcode}
                      </div>
                    </div>
                  )}
                </div>

                {/* Card Footer */}
                <div className="px-6 py-3 bg-slate-50 border-t border-slate-200 flex gap-2">
                  <button
                    onClick={() => {
                      setSelectedRoom(room);
                      setShowDialog(true);
                    }}
                    className="flex-1 flex items-center justify-center gap-2 px-4 py-2 text-sm text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                  >
                    <Edit2 className="w-3.5 h-3.5" />
                    Bearbeiten
                  </button>
                  <button
                    onClick={() => handleDeleteRoom(room.id, room.name)}
                    className="flex-1 flex items-center justify-center gap-2 px-4 py-2 text-sm text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                    Löschen
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Room Dialog */}
      <RoomDialog
        isOpen={showDialog}
        onClose={() => setShowDialog(false)}
        onSuccess={() => {
          // Data wird automatisch durch DataContext refresht
          setShowDialog(false);
        }}
        room={selectedRoom}
      />

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        isOpen={showDeleteConfirm}
        title="Zimmer löschen"
        message={`Möchten Sie ${roomToDelete?.name} wirklich löschen? Diese Aktion kann nicht rückgängig gemacht werden.`}
        confirmLabel="Ja, löschen"
        cancelLabel="Abbrechen"
        onConfirm={confirmDelete}
        onCancel={cancelDelete}
        variant="danger"
      />

      {/* Error Dialog */}
      <ConfirmDialog
        isOpen={showErrorDialog}
        title="Fehler"
        message={errorMessage}
        confirmLabel="OK"
        cancelLabel=""
        onConfirm={() => {
          setShowErrorDialog(false);
          setRoomToDelete(null);
        }}
        onCancel={() => {
          setShowErrorDialog(false);
          setRoomToDelete(null);
        }}
        variant="danger"
      />
    </div>
  );
}
