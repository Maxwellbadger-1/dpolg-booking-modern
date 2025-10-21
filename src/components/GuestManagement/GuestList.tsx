import { useState, useMemo, useRef } from 'react';
import { Users, Search, UserPlus, Edit2, Mail, Phone, CheckCircle, X, Trash2, Eye } from 'lucide-react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { useData } from '../../context/DataContext';
import { useDebounce } from '../../hooks/useDebounce';
import GuestDialog from './GuestDialog';
import GuestDetails from './GuestDetails';
import ConfirmDialog from '../ConfirmDialog';
import { SELECT_STYLES, SELECT_BACKGROUND_STYLE } from '../../lib/selectStyles';

interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon: string;
  strasse?: string;
  plz?: string;
  ort?: string;
  dpolg_mitglied: boolean;
  mitgliedsnummer?: string;
  notizen?: string;
}

export default function GuestList() {
  // Get data from global context
  const { guests, loading, deleteGuest } = useData();

  const [searchQuery, setSearchQuery] = useState('');
  const [memberFilter, setMemberFilter] = useState<string>('all');
  const [showDialog, setShowDialog] = useState(false);
  const [selectedGuest, setSelectedGuest] = useState<Guest | undefined>(undefined);
  const [showDetails, setShowDetails] = useState(false);
  const [detailsGuestId, setDetailsGuestId] = useState<number | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [guestToDelete, setGuestToDelete] = useState<{ id: number; name: string } | null>(null);
  const [showErrorDialog, setShowErrorDialog] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string>('');

  // Debounce search query for better performance (300ms delay)
  const debouncedSearchQuery = useDebounce(searchQuery, 300);

  // Ref for virtualization scroll container
  const parentRef = useRef<HTMLDivElement>(null);

  const handleDeleteGuest = (id: number, name: string) => {
    setGuestToDelete({ id, name });
    setShowDeleteConfirm(true);
  };

  const confirmDelete = async () => {
    if (!guestToDelete) return;

    try {
      await deleteGuest(guestToDelete.id);
      setShowDeleteConfirm(false);
      setGuestToDelete(null);
    } catch (error) {
      console.error('Fehler beim Löschen des Gastes:', error);

      // User-friendly error message
      const errorMessage = error instanceof Error ? error.message : String(error);

      if (errorMessage.includes('FOREIGN KEY') || errorMessage.toLowerCase().includes('foreign key constraint')) {
        setErrorMessage(
          `Gast kann nicht gelöscht werden!\n\n` +
          `${guestToDelete.name} hat noch aktive Buchungen.\n\n` +
          `Bitte löschen Sie zuerst alle Buchungen dieses Gastes.`
        );
      } else {
        setErrorMessage(`Fehler beim Löschen des Gastes:\n\n${errorMessage}`);
      }

      setShowDeleteConfirm(false);
      setShowErrorDialog(true);
    }
  };

  const cancelDelete = () => {
    setShowDeleteConfirm(false);
    setGuestToDelete(null);
  };

  const handleViewDetails = (guestId: number) => {
    setDetailsGuestId(guestId);
    setShowDetails(true);
  };

  const handleEdit = (guest: Guest) => {
    setSelectedGuest(guest);
    setShowDialog(true);
  };

  // Memoize filtered guests - only recalculate when guests, debouncedSearchQuery, or memberFilter changes
  const filteredGuests = useMemo(() => {
    return guests.filter(guest => {
      const matchesSearch =
        `${guest.vorname} ${guest.nachname}`.toLowerCase().includes(debouncedSearchQuery.toLowerCase()) ||
        guest.email.toLowerCase().includes(debouncedSearchQuery.toLowerCase()) ||
        guest.telefon?.includes(debouncedSearchQuery);

      const matchesMember =
        memberFilter === 'all' ||
        (memberFilter === 'member' && guest.dpolg_mitglied) ||
        (memberFilter === 'non-member' && !guest.dpolg_mitglied);

      return matchesSearch && matchesMember;
    });
  }, [guests, debouncedSearchQuery, memberFilter]);

  // TanStack Virtual - only render visible items
  const rowVirtualizer = useVirtualizer({
    count: filteredGuests.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 80, // Estimated row height in pixels
    overscan: 5, // Render 5 extra items above/below for smooth scrolling
  });

  if (loading) {
    return (
      <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 p-6 flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
          <p className="text-slate-600">Lade Gäste...</p>
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
            <h2 className="text-2xl font-bold text-slate-800">Gäste</h2>
            <p className="text-sm text-slate-600 mt-1">
              {filteredGuests.length} {filteredGuests.length === 1 ? 'Gast' : 'Gäste'}
            </p>
          </div>
          <button
            onClick={() => {
              setSelectedGuest(undefined);
              setShowDialog(true);
            }}
            className="flex items-center gap-2 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white px-4 py-2 rounded-lg font-semibold shadow-lg transition-all"
          >
            <UserPlus className="w-4 h-4" />
            Neuer Gast
          </button>
        </div>

        {/* Search & Filter */}
        <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-4 mb-6">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-400 w-5 h-5" />
              <input
                type="text"
                placeholder="Suche nach Name, Email oder Telefon..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <select
              value={memberFilter}
              onChange={(e) => setMemberFilter(e.target.value)}
              className={SELECT_STYLES}
              style={SELECT_BACKGROUND_STYLE}
            >
              <option value="all">Alle Gäste</option>
              <option value="member">Nur Mitglieder</option>
              <option value="non-member">Nur Nicht-Mitglieder</option>
            </select>
          </div>
        </div>

        {/* Guests List - Optimized with debouncing and memoization */}
        {filteredGuests.length === 0 ? (
          <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-12">
            <div className="text-center">
              <Users className="w-16 h-16 text-slate-300 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-slate-800 mb-2">Keine Gäste gefunden</h3>
              <p className="text-slate-600">
                {searchQuery || memberFilter !== 'all'
                  ? 'Versuche andere Suchkriterien oder Filter.'
                  : 'Erstelle deinen ersten Gast mit dem Button oben.'
                }
              </p>
            </div>
          </div>
        ) : (
          <div className="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
            {/* Table Header */}
            <div className="bg-slate-50 border-b border-slate-200 grid grid-cols-12 gap-4 px-6 py-3 sticky top-0 z-10">
              <div className="col-span-2 text-xs font-semibold text-slate-600 uppercase tracking-wider">
                Name
              </div>
              <div className="col-span-3 text-xs font-semibold text-slate-600 uppercase tracking-wider">
                Kontakt
              </div>
              <div className="col-span-2 text-xs font-semibold text-slate-600 uppercase tracking-wider">
                Adresse
              </div>
              <div className="col-span-2 text-xs font-semibold text-slate-600 uppercase tracking-wider">
                Mitgliedschaft
              </div>
              <div className="col-span-3 text-xs font-semibold text-slate-600 uppercase tracking-wider text-right">
                Aktionen
              </div>
            </div>

            {/* Virtualized Scrollable Guest List */}
            <div
              ref={parentRef}
              className="h-[600px] overflow-y-auto"
            >
              <div
                style={{
                  height: `${rowVirtualizer.getTotalSize()}px`,
                  width: '100%',
                  position: 'relative',
                }}
              >
                {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                  const guest = filteredGuests[virtualRow.index];
                  return (
                    <div
                      key={guest.id}
                      style={{
                        position: 'absolute',
                        top: 0,
                        left: 0,
                        width: '100%',
                        height: `${virtualRow.size}px`,
                        transform: `translateY(${virtualRow.start}px)`,
                      }}
                      className="grid grid-cols-12 gap-4 px-6 py-4 border-b border-slate-200 hover:bg-slate-50 transition-colors"
                    >
                      {/* Name */}
                      <div className="col-span-2">
                        <div className="text-sm font-semibold text-slate-900">
                          {guest.vorname} {guest.nachname}
                        </div>
                        {guest.notizen && (
                          <div className="text-xs text-slate-500 truncate max-w-xs">
                            {guest.notizen}
                          </div>
                        )}
                      </div>

                      {/* Kontakt */}
                      <div className="col-span-3">
                        <div className="flex flex-col gap-1">
                          <div className="flex items-center gap-1.5 text-sm text-slate-900 truncate">
                            <Mail className="w-3.5 h-3.5 text-slate-400 flex-shrink-0" />
                            <span className="truncate">{guest.email}</span>
                          </div>
                          <div className="flex items-center gap-1.5 text-sm text-slate-600">
                            <Phone className="w-3.5 h-3.5 text-slate-400 flex-shrink-0" />
                            {guest.telefon}
                          </div>
                        </div>
                      </div>

                      {/* Adresse */}
                      <div className="col-span-2">
                        {guest.strasse || guest.plz || guest.ort ? (
                          <div className="text-sm text-slate-900">
                            <div className="truncate">{guest.strasse}</div>
                            <div className="text-xs text-slate-500">
                              {guest.plz} {guest.ort}
                            </div>
                          </div>
                        ) : (
                          <span className="text-sm text-slate-400">Keine Adresse</span>
                        )}
                      </div>

                      {/* Mitgliedschaft */}
                      <div className="col-span-2">
                        {guest.dpolg_mitglied ? (
                          <div>
                            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-100 text-emerald-700 border border-emerald-200">
                              <CheckCircle className="w-3 h-3" />
                              Mitglied
                            </span>
                            {guest.mitgliedsnummer && (
                              <div className="text-xs text-slate-500 mt-1">
                                Nr. {guest.mitgliedsnummer}
                              </div>
                            )}
                          </div>
                        ) : (
                          <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-slate-100 text-slate-700 border border-slate-200">
                            <X className="w-3 h-3" />
                            Kein Mitglied
                          </span>
                        )}
                      </div>

                      {/* Aktionen */}
                      <div className="col-span-3 flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleViewDetails(guest.id)}
                          className="inline-flex items-center gap-1 px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-100 rounded-lg transition-colors"
                        >
                          <Eye className="w-3.5 h-3.5" />
                          Details
                        </button>
                        <button
                          onClick={() => handleEdit(guest)}
                          className="inline-flex items-center gap-1 px-3 py-1.5 text-sm text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                        >
                          <Edit2 className="w-3.5 h-3.5" />
                          Bearbeiten
                        </button>
                        <button
                          onClick={() => handleDeleteGuest(guest.id, `${guest.vorname} ${guest.nachname}`)}
                          className="inline-flex items-center gap-1 px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                          Löschen
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Guest Dialog */}
      <GuestDialog
        isOpen={showDialog}
        onClose={() => setShowDialog(false)}
        onSuccess={() => {
          // Data wird automatisch durch DataContext refresht
          setShowDialog(false);
        }}
        guest={selectedGuest}
      />

      {/* Guest Details */}
      {detailsGuestId && (
        <GuestDetails
          guestId={detailsGuestId}
          isOpen={showDetails}
          onClose={() => {
            setShowDetails(false);
            setDetailsGuestId(null);
          }}
          onEdit={() => {
            const guest = guests.find(g => g.id === detailsGuestId);
            if (guest) {
              setSelectedGuest(guest);
              setShowDetails(false);
              setShowDialog(true);
            }
          }}
        />
      )}

      {/* Delete Confirmation Dialog */}
      <ConfirmDialog
        isOpen={showDeleteConfirm}
        title="Gast löschen"
        message={`Möchten Sie ${guestToDelete?.name} wirklich löschen? Diese Aktion kann nicht rückgängig gemacht werden.`}
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
          setGuestToDelete(null);
        }}
        onCancel={() => {
          setShowErrorDialog(false);
          setGuestToDelete(null);
        }}
        variant="danger"
      />
    </div>
  );
}
