import { useState, useEffect, useRef } from 'react';
import { Search, UserPlus, X, Mail, Phone, CheckCircle } from 'lucide-react';

interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon: string;
  dpolg_mitglied: boolean;
}

interface SearchableGuestPickerProps {
  guests: Guest[];
  selectedGuestId: number;
  onSelectGuest: (guestId: number) => void;
  onCreateNew?: () => void;
}

export default function SearchableGuestPicker({
  guests,
  selectedGuestId,
  onSelectGuest,
  onCreateNew,
}: SearchableGuestPickerProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [isOpen, setIsOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Filter guests based on search query
  const filteredGuests = guests.filter((guest) => {
    const query = searchQuery.toLowerCase();
    return (
      guest.vorname.toLowerCase().includes(query) ||
      guest.nachname.toLowerCase().includes(query) ||
      guest.email.toLowerCase().includes(query) ||
      guest.telefon?.toLowerCase().includes(query)
    );
  });

  // Get selected guest for display
  const selectedGuest = guests.find((g) => g.id === selectedGuestId);

  // Reset highlighted index when filtered results change
  useEffect(() => {
    setHighlightedIndex(0);
  }, [searchQuery]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        inputRef.current &&
        !inputRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isOpen) {
      if (e.key === 'Enter' || e.key === 'ArrowDown') {
        setIsOpen(true);
        e.preventDefault();
      }
      return;
    }

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setHighlightedIndex((prev) =>
          prev < filteredGuests.length - 1 ? prev + 1 : prev
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setHighlightedIndex((prev) => (prev > 0 ? prev - 1 : 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredGuests[highlightedIndex]) {
          handleSelectGuest(filteredGuests[highlightedIndex]);
        }
        break;
      case 'Escape':
        e.preventDefault();
        setIsOpen(false);
        setSearchQuery('');
        break;
    }
  };

  const handleSelectGuest = (guest: Guest) => {
    onSelectGuest(guest.id);
    setIsOpen(false);
    setSearchQuery('');
  };

  const handleClear = () => {
    onSelectGuest(0);
    setSearchQuery('');
    inputRef.current?.focus();
  };

  const handleCreateNew = () => {
    setIsOpen(false);
    setSearchQuery('');
    onCreateNew?.();
  };

  // Highlight matching text
  const highlightText = (text: string | null | undefined, query: string) => {
    if (!text || !query) return text || '';

    const parts = text.split(new RegExp(`(${query})`, 'gi'));
    return (
      <>
        {parts.map((part, i) =>
          part.toLowerCase() === query.toLowerCase() ? (
            <mark key={i} className="bg-blue-200 text-blue-900 font-semibold">
              {part}
            </mark>
          ) : (
            <span key={i}>{part}</span>
          )
        )}
      </>
    );
  };

  return (
    <div className="relative">
      {/* Label */}
      <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
        <UserPlus className="w-4 h-4" />
        Gast *
      </label>

      {/* Selected Guest Display (if selected) */}
      {selectedGuest && !isOpen ? (
        <div className="w-full px-4 py-3 bg-emerald-50 border-2 border-emerald-500 rounded-lg flex items-center justify-between group">
          <div className="flex items-center gap-3">
            <CheckCircle className="w-5 h-5 text-emerald-600 flex-shrink-0" />
            <div>
              <p className="font-semibold text-slate-900">
                {selectedGuest.vorname} {selectedGuest.nachname}
                {selectedGuest.dpolg_mitglied && (
                  <span className="ml-2 text-xs font-semibold text-emerald-600 bg-emerald-100 px-2 py-0.5 rounded">
                    Mitglied
                  </span>
                )}
              </p>
              <div className="flex items-center gap-3 mt-1">
                <p className="text-xs text-slate-600 flex items-center gap-1">
                  <Mail className="w-3 h-3" />
                  {selectedGuest.email}
                </p>
                <p className="text-xs text-slate-600 flex items-center gap-1">
                  <Phone className="w-3 h-3" />
                  {selectedGuest.telefon}
                </p>
              </div>
            </div>
          </div>
          <button
            type="button"
            onClick={handleClear}
            className="opacity-0 group-hover:opacity-100 p-2 hover:bg-red-100 rounded-lg transition-all"
            title="Auswahl aufheben"
          >
            <X className="w-4 h-4 text-red-600" />
          </button>
        </div>
      ) : (
        <>
          {/* Search Input */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-slate-400" />
            <input
              ref={inputRef}
              type="text"
              value={searchQuery}
              onChange={(e) => {
                setSearchQuery(e.target.value);
                setIsOpen(true);
              }}
              onFocus={() => setIsOpen(true)}
              onKeyDown={handleKeyDown}
              placeholder="Gast suchen (Name, Email oder Telefon)..."
              className="w-full pl-10 pr-4 py-3 border-2 border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-slate-900 placeholder-slate-400"
              autoComplete="off"
            />
          </div>

          {/* Dropdown Results */}
          {isOpen && (
            <div
              ref={dropdownRef}
              className="absolute z-50 w-full mt-2 bg-white border-2 border-slate-300 rounded-lg shadow-2xl max-h-80 overflow-y-auto"
            >
              {filteredGuests.length > 0 ? (
                <div className="py-2">
                  {filteredGuests.map((guest, index) => (
                    <button
                      key={guest.id}
                      type="button"
                      onClick={() => handleSelectGuest(guest)}
                      className={`w-full px-4 py-3 text-left transition-colors ${
                        index === highlightedIndex
                          ? 'bg-blue-100 border-l-4 border-blue-600'
                          : 'hover:bg-slate-50 border-l-4 border-transparent'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex-1">
                          <p className="font-semibold text-slate-900 mb-1">
                            {highlightText(
                              `${guest.vorname} ${guest.nachname}`,
                              searchQuery
                            )}
                            {guest.dpolg_mitglied && (
                              <span className="ml-2 text-xs font-semibold text-emerald-600 bg-emerald-100 px-2 py-0.5 rounded">
                                Mitglied
                              </span>
                            )}
                          </p>
                          <div className="flex items-center gap-3">
                            <p className="text-xs text-slate-600 flex items-center gap-1">
                              <Mail className="w-3 h-3" />
                              {highlightText(guest.email, searchQuery)}
                            </p>
                            <p className="text-xs text-slate-600 flex items-center gap-1">
                              <Phone className="w-3 h-3" />
                              {highlightText(guest.telefon, searchQuery)}
                            </p>
                          </div>
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              ) : searchQuery ? (
                <div className="py-8 px-4 text-center">
                  <p className="text-sm text-slate-600 mb-3">
                    Kein Gast gefunden für "{searchQuery}"
                  </p>
                  {onCreateNew && (
                    <button
                      type="button"
                      onClick={handleCreateNew}
                      className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <UserPlus className="w-4 h-4" />
                      Neuen Gast anlegen
                    </button>
                  )}
                </div>
              ) : (
                <div className="py-8 px-4 text-center">
                  <Search className="w-12 h-12 text-slate-300 mx-auto mb-3" />
                  <p className="text-sm text-slate-600 mb-2">
                    Geben Sie mindestens einen Buchstaben ein
                  </p>
                  <p className="text-xs text-slate-500">
                    Sie können nach Name, Email oder Telefon suchen
                  </p>
                  {onCreateNew && (
                    <button
                      type="button"
                      onClick={handleCreateNew}
                      className="mt-4 inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <UserPlus className="w-4 h-4" />
                      Neuen Gast anlegen
                    </button>
                  )}
                </div>
              )}
            </div>
          )}
        </>
      )}

      {/* Keyboard Shortcuts Hint */}
      {isOpen && filteredGuests.length > 0 && (
        <div className="mt-2 px-3 py-2 bg-slate-50 border border-slate-200 rounded-lg">
          <p className="text-xs text-slate-600">
            <kbd className="px-1.5 py-0.5 bg-white border border-slate-300 rounded text-xs font-mono">
              ↑
            </kbd>{' '}
            <kbd className="px-1.5 py-0.5 bg-white border border-slate-300 rounded text-xs font-mono">
              ↓
            </kbd>{' '}
            zum Navigieren •{' '}
            <kbd className="px-1.5 py-0.5 bg-white border border-slate-300 rounded text-xs font-mono">
              Enter
            </kbd>{' '}
            zum Auswählen •{' '}
            <kbd className="px-1.5 py-0.5 bg-white border border-slate-300 rounded text-xs font-mono">
              Esc
            </kbd>{' '}
            zum Schließen
          </p>
        </div>
      )}
    </div>
  );
}
