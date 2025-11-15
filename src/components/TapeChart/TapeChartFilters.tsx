import { Search, X } from 'lucide-react';
import { useState } from 'react';

interface TapeChartFiltersProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  roomTypeFilter: string;
  onRoomTypeFilterChange: (type: string) => void;
  availableRoomTypes: string[];
}

export default function TapeChartFilters({
  searchQuery,
  onSearchChange,
  roomTypeFilter,
  onRoomTypeFilterChange,
  availableRoomTypes,
}: TapeChartFiltersProps) {

  return (
    <div className="bg-gradient-to-r from-slate-700 to-slate-800 border-b border-slate-600 px-4 py-3">
      <div className="flex items-center gap-3 flex-wrap">
        {/* Search */}
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Suche nach Gast oder Reservierungsnummer..."
            className="w-full pl-10 pr-10 py-2 bg-slate-600/50 border border-slate-500 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
          />
          {searchQuery && (
            <button
              onClick={() => onSearchChange('')}
              className="absolute right-3 top-1/2 -translate-y-1/2 p-1 hover:bg-slate-500 rounded transition-colors"
            >
              <X className="w-3 h-3 text-slate-400" />
            </button>
          )}
        </div>

        {/* Room Type Filter */}
        <div className="relative">
          <select
            value={roomTypeFilter}
            onChange={(e) => onRoomTypeFilterChange(e.target.value)}
            className="appearance-none pl-4 pr-10 py-2 bg-slate-600/50 border border-slate-500 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 cursor-pointer min-w-[160px]"
          >
            <option value="all">Alle Gebäudetypen</option>
            {availableRoomTypes.map((type) => (
              <option key={type} value={type}>
                {type}
              </option>
            ))}
          </select>
          <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none">
            <svg
              className="w-4 h-4 text-slate-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 9l-7 7-7-7"
              />
            </svg>
          </div>
        </div>

        {/* Active Filters Badge */}
        {(searchQuery || roomTypeFilter !== 'all') && (
          <div className="flex items-center gap-2 px-3 py-1.5 bg-blue-500/20 border border-blue-500/50 rounded-lg">
            <div className="text-blue-300 text-xs font-semibold">
              {[
                searchQuery && '🔍 Suche',
                roomTypeFilter !== 'all' && 'Gebäudetyp',
              ]
                .filter(Boolean)
                .join(', ')}
            </div>
            <button
              onClick={() => {
                onSearchChange('');
                onRoomTypeFilterChange('all');
              }}
              className="p-0.5 hover:bg-blue-500/30 rounded transition-colors"
              title="Alle Filter zurücksetzen"
            >
              <X className="w-3 h-3 text-blue-300" />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
