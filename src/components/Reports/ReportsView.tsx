import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { BarChart3, TrendingUp, Euro, Calendar, Home, Users, Clock } from 'lucide-react';
import { format, startOfMonth, endOfMonth, subMonths } from 'date-fns';
import { de } from 'date-fns/locale';

interface ReportStats {
  total_bookings: number;
  active_bookings: number;
  total_revenue: number;
  total_nights: number;
  average_price_per_night: number;
  occupancy_rate: number;
}

interface RoomOccupancy {
  room_id: number;
  room_name: string;
  total_bookings: number;
  total_nights: number;
  total_revenue: number;
  occupancy_rate: number;
}

export default function ReportsView() {
  const [stats, setStats] = useState<ReportStats | null>(null);
  const [roomOccupancy, setRoomOccupancy] = useState<RoomOccupancy[]>([]);
  const [loading, setLoading] = useState(true);
  const [startDate, setStartDate] = useState(format(startOfMonth(new Date()), 'yyyy-MM-dd'));
  const [endDate, setEndDate] = useState(format(endOfMonth(new Date()), 'yyyy-MM-dd'));

  useEffect(() => {
    loadReports();
  }, [startDate, endDate]);

  const loadReports = async () => {
    try {
      setLoading(true);
      const [statsData, occupancyData] = await Promise.all([
        invoke<ReportStats>('get_report_stats_command', { startDate, endDate }),
        invoke<RoomOccupancy[]>('get_room_occupancy_command', { startDate, endDate }),
      ]);
      setStats(statsData);
      setRoomOccupancy(occupancyData);
    } catch (error) {
      console.error('Fehler beim Laden der Reports:', error);
    } finally {
      setLoading(false);
    }
  };

  const setCurrentMonth = () => {
    setStartDate(format(startOfMonth(new Date()), 'yyyy-MM-dd'));
    setEndDate(format(endOfMonth(new Date()), 'yyyy-MM-dd'));
  };

  const setLastMonth = () => {
    const lastMonth = subMonths(new Date(), 1);
    setStartDate(format(startOfMonth(lastMonth), 'yyyy-MM-dd'));
    setEndDate(format(endOfMonth(lastMonth), 'yyyy-MM-dd'));
  };

  if (loading) {
    return (
      <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 p-6 flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
          <p className="text-slate-600">Lade Reports...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 p-6 overflow-auto">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-slate-800">Reports & Statistiken</h2>
            <p className="text-sm text-slate-600 mt-1">
              Zeitraum: {format(new Date(startDate), 'dd.MM.yyyy', { locale: de })} - {format(new Date(endDate), 'dd.MM.yyyy', { locale: de })}
            </p>
          </div>
        </div>

        {/* Date Filter */}
        <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-4">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <label className="text-sm text-slate-600 font-medium">Von:</label>
              <input
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div className="flex items-center gap-2">
              <label className="text-sm text-slate-600 font-medium">Bis:</label>
              <input
                type="date"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <button
              onClick={setCurrentMonth}
              className="px-4 py-2 bg-blue-100 hover:bg-blue-200 text-blue-700 rounded-lg font-semibold transition-colors text-sm"
            >
              Aktueller Monat
            </button>
            <button
              onClick={setLastMonth}
              className="px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg font-semibold transition-colors text-sm"
            >
              Letzter Monat
            </button>
          </div>
        </div>

        {/* Stats Cards */}
        {stats && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <div className="bg-blue-100 p-3 rounded-lg">
                  <Calendar className="w-6 h-6 text-blue-600" />
                </div>
                <div className="text-right">
                  <p className="text-sm text-slate-600">Buchungen Gesamt</p>
                  <p className="text-3xl font-bold text-slate-900">{stats.total_bookings}</p>
                </div>
              </div>
              <div className="flex items-center gap-2 text-sm text-emerald-600">
                <Users className="w-4 h-4" />
                <span>{stats.active_bookings} aktiv</span>
              </div>
            </div>

            <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <div className="bg-emerald-100 p-3 rounded-lg">
                  <Euro className="w-6 h-6 text-emerald-600" />
                </div>
                <div className="text-right">
                  <p className="text-sm text-slate-600">Umsatz Gesamt</p>
                  <p className="text-3xl font-bold text-slate-900">{stats.total_revenue.toFixed(0)} €</p>
                </div>
              </div>
              <div className="text-sm text-slate-600">
                Ø {stats.average_price_per_night.toFixed(2)} € / Nacht
              </div>
            </div>

            <div className="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <div className="bg-purple-100 p-3 rounded-lg">
                  <TrendingUp className="w-6 h-6 text-purple-600" />
                </div>
                <div className="text-right">
                  <p className="text-sm text-slate-600">Auslastung</p>
                  <p className="text-3xl font-bold text-slate-900">{stats.occupancy_rate.toFixed(1)}%</p>
                </div>
              </div>
              <div className="flex items-center gap-2 text-sm text-slate-600">
                <Clock className="w-4 h-4" />
                <span>{stats.total_nights} Nächte</span>
              </div>
            </div>
          </div>
        )}

        {/* Room Occupancy Table */}
        <div className="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
          <div className="px-6 py-4 border-b border-slate-200">
            <div className="flex items-center gap-2">
              <Home className="w-5 h-5 text-blue-600" />
              <h3 className="text-lg font-bold text-slate-800">Belegung pro Zimmer</h3>
            </div>
          </div>

          {roomOccupancy.length === 0 ? (
            <div className="text-center py-12">
              <BarChart3 className="w-16 h-16 text-slate-300 mx-auto mb-4" />
              <p className="text-slate-600">Keine Daten für den gewählten Zeitraum verfügbar</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-slate-50 border-b border-slate-200">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-semibold text-slate-600 uppercase tracking-wider">
                      Zimmer
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-semibold text-slate-600 uppercase tracking-wider">
                      Buchungen
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-semibold text-slate-600 uppercase tracking-wider">
                      Nächte
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-semibold text-slate-600 uppercase tracking-wider">
                      Umsatz
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-semibold text-slate-600 uppercase tracking-wider">
                      Auslastung
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-200">
                  {roomOccupancy.map((room) => (
                    <tr key={room.room_id} className="hover:bg-slate-50 transition-colors">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="font-semibold text-slate-900">{room.room_name}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right">
                        <div className="text-sm text-slate-900">{room.total_bookings}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right">
                        <div className="text-sm text-slate-900">{room.total_nights}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right">
                        <div className="text-sm font-semibold text-slate-900">
                          {room.total_revenue.toFixed(2)} €
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right">
                        <div className="flex items-center justify-end gap-2">
                          <div className="w-24 bg-slate-200 rounded-full h-2">
                            <div
                              className={`h-2 rounded-full ${
                                room.occupancy_rate >= 80
                                  ? 'bg-emerald-500'
                                  : room.occupancy_rate >= 50
                                  ? 'bg-blue-500'
                                  : 'bg-slate-400'
                              }`}
                              style={{ width: `${Math.min(100, room.occupancy_rate)}%` }}
                            ></div>
                          </div>
                          <span className="text-sm font-semibold text-slate-900 w-12">
                            {room.occupancy_rate.toFixed(0)}%
                          </span>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
