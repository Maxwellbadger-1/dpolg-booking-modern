import { useState, useEffect, useRef } from 'react';
import { Search, Hotel, X, MapPin, Users, CheckCircle } from 'lucide-react';

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

interface SearchableRoomPickerProps {
  rooms: Room[];
  selectedRoomId: number;
  onSelectRoom: (roomId: number) => void;
}

export default function SearchableRoomPicker({
  rooms,
  selectedRoomId,
  onSelectRoom,
}: SearchableRoomPickerProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [isOpen, setIsOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Filter rooms based on search query
  const filteredRooms = rooms.filter((room) => {
    const query = searchQuery.toLowerCase();
    return (
      room.name.toLowerCase().includes(query) ||
      room.ort.toLowerCase().includes(query) ||
      room.gebaeude_typ.toLowerCase().includes(query)
    );
  });

  // Get selected room for display
  const selectedRoom = rooms.find((r) => r.id === selectedRoomId);

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
          prev < filteredRooms.length - 1 ? prev + 1 : prev
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setHighlightedIndex((prev) => (prev > 0 ? prev - 1 : 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredRooms[highlightedIndex]) {
          handleSelectRoom(filteredRooms[highlightedIndex]);
        }
        break;
      case 'Escape':
        e.preventDefault();
        setIsOpen(false);
        setSearchQuery('');
        break;
    }
  };

  const handleSelectRoom = (room: Room) => {
    onSelectRoom(room.id);
    setIsOpen(false);
    setSearchQuery('');
  };

  const handleClear = () => {
    onSelectRoom(0);
    setSearchQuery('');
    inputRef.current?.focus();
  };

  // Highlight matching text
  const highlightText = (text: string, query: string) => {
    if (!query) return text;

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
        <Hotel className="w-4 h-4" />
        Zimmer *
      </label>

      {/* Selected Room Display (if selected) */}
      {selectedRoom && !isOpen ? (
        <div className="w-full px-4 py-3 bg-blue-50 border-2 border-blue-500 rounded-lg flex items-center justify-between group">
          <div className="flex items-center gap-3">
            <CheckCircle className="w-5 h-5 text-blue-600 flex-shrink-0" />
            <div>
              <p className="font-semibold text-slate-900">
                {selectedRoom.name}
                <span className="ml-2 text-xs font-semibold text-blue-600 bg-blue-100 px-2 py-0.5 rounded">
                  {selectedRoom.gebaeude_typ}
                </span>
              </p>
              <div className="flex items-center gap-3 mt-1">
                <p className="text-xs text-slate-600 flex items-center gap-1">
                  <MapPin className="w-3 h-3" />
                  {selectedRoom.ort}
                </p>
                <p className="text-xs text-slate-600 flex items-center gap-1">
                  <Users className="w-3 h-3" />
                  Kapazität: {selectedRoom.capacity}
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
              placeholder="Zimmer suchen (Name oder Ort)..."
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
              {filteredRooms.length > 0 ? (
                <div className="py-2">
                  {filteredRooms.map((room, index) => (
                    <button
                      key={room.id}
                      type="button"
                      onClick={() => handleSelectRoom(room)}
                      className={`w-full px-4 py-3 text-left transition-colors ${
                        index === highlightedIndex
                          ? 'bg-blue-100 border-l-4 border-blue-600'
                          : 'hover:bg-slate-50 border-l-4 border-transparent'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex-1">
                          <p className="font-semibold text-slate-900 mb-1">
                            {highlightText(room.name, searchQuery)}
                            <span className="ml-2 text-xs font-semibold text-blue-600 bg-blue-100 px-2 py-0.5 rounded">
                              {room.gebaeude_typ}
                            </span>
                          </p>
                          <div className="flex items-center gap-3">
                            <p className="text-xs text-slate-600 flex items-center gap-1">
                              <MapPin className="w-3 h-3" />
                              {highlightText(room.ort, searchQuery)}
                            </p>
                            <p className="text-xs text-slate-600 flex items-center gap-1">
                              <Users className="w-3 h-3" />
                              Kapazität: {room.capacity}
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
                    Kein Zimmer gefunden für "{searchQuery}"
                  </p>
                </div>
              ) : (
                <div className="py-8 px-4 text-center">
                  <Search className="w-12 h-12 text-slate-300 mx-auto mb-3" />
                  <p className="text-sm text-slate-600 mb-2">
                    Geben Sie mindestens einen Buchstaben ein
                  </p>
                  <p className="text-xs text-slate-500">
                    Sie können nach Zimmer-Name oder Ort suchen
                  </p>
                </div>
              )}
            </div>
          )}
        </>
      )}

      {/* Keyboard Shortcuts Hint */}
      {isOpen && filteredRooms.length > 0 && (
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
