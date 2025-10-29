import { Calendar, Plus, TrendingUp, Home, Users } from 'lucide-react';
import { formatDateLong } from '../utils/dateFormatting';

interface DashboardQuickStatsProps {
  onCreateBooking: () => void;
  stats?: {
    todayCheckins: number;
    todayCheckouts: number;
    availableRooms: number;
    occupancyRate: number;
  };
}

export default function DashboardQuickStats({
  onCreateBooking,
  stats = {
    todayCheckins: 0,
    todayCheckouts: 0,
    availableRooms: 0,
    occupancyRate: 0,
  },
}: DashboardQuickStatsProps) {
  const today = formatDateLong(new Date());

  return (
    <div className="bg-gradient-to-br from-blue-600 to-blue-700 rounded-2xl shadow-xl p-6 text-white">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <Calendar className="w-5 h-5 text-blue-200" />
            <p className="text-sm font-medium text-blue-100">Heute</p>
          </div>
          <h2 className="text-lg font-bold capitalize">{today}</h2>
        </div>
        <button
          onClick={onCreateBooking}
          className="flex items-center gap-2 px-4 py-2.5 bg-white hover:bg-blue-50 text-blue-700 font-semibold rounded-lg transition-all shadow-lg hover:shadow-xl hover:scale-105 active:scale-95"
        >
          <Plus className="w-5 h-5" />
          Neue Buchung
        </button>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-4 gap-4">
        {/* Check-ins Today */}
        <div className="bg-white/10 backdrop-blur-sm rounded-lg p-4 border border-white/20">
          <div className="flex items-center gap-2 mb-2">
            <div className="bg-emerald-500/20 p-1.5 rounded">
              <Users className="w-4 h-4 text-emerald-200" />
            </div>
            <p className="text-xs font-medium text-blue-100">Check-ins</p>
          </div>
          <p className="text-3xl font-bold">{stats.todayCheckins}</p>
        </div>

        {/* Check-outs Today */}
        <div className="bg-white/10 backdrop-blur-sm rounded-lg p-4 border border-white/20">
          <div className="flex items-center gap-2 mb-2">
            <div className="bg-amber-500/20 p-1.5 rounded">
              <Users className="w-4 h-4 text-amber-200" />
            </div>
            <p className="text-xs font-medium text-blue-100">Check-outs</p>
          </div>
          <p className="text-3xl font-bold">{stats.todayCheckouts}</p>
        </div>

        {/* Available Rooms */}
        <div className="bg-white/10 backdrop-blur-sm rounded-lg p-4 border border-white/20">
          <div className="flex items-center gap-2 mb-2">
            <div className="bg-purple-500/20 p-1.5 rounded">
              <Home className="w-4 h-4 text-purple-200" />
            </div>
            <p className="text-xs font-medium text-blue-100">Freie Zimmer</p>
          </div>
          <p className="text-3xl font-bold">{stats.availableRooms}</p>
        </div>

        {/* Occupancy Rate */}
        <div className="bg-white/10 backdrop-blur-sm rounded-lg p-4 border border-white/20">
          <div className="flex items-center gap-2 mb-2">
            <div className="bg-rose-500/20 p-1.5 rounded">
              <TrendingUp className="w-4 h-4 text-rose-200" />
            </div>
            <p className="text-xs font-medium text-blue-100">Auslastung</p>
          </div>
          <p className="text-3xl font-bold">{stats.occupancyRate}%</p>
        </div>
      </div>

      {/* Quick Info */}
      <div className="mt-4 pt-4 border-t border-white/20">
        <p className="text-xs text-blue-100">
          💡 Tipp: Nutze den{' '}
          <Plus className="inline w-3 h-3" />
          {' '}-Button rechts unten für schnelle Buchungen
        </p>
      </div>
    </div>
  );
}
