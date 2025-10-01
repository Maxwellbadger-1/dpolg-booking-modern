import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Mail, Search, CheckCircle, AlertCircle, Clock, RefreshCw, Send } from 'lucide-react';

interface EmailLog {
  id: number;
  booking_id: number;
  guest_id: number;
  template_used: string;
  recipient_email: string;
  subject: string;
  status: string;
  error_message: string | null;
  sent_at: string;
}

interface BookingWithDetails {
  id: number;
  reservierungsnummer: string;
  guest: {
    vorname: string;
    nachname: string;
  };
}

export default function EmailHistoryView() {
  const [emailLogs, setEmailLogs] = useState<EmailLog[]>([]);
  const [filteredLogs, setFilteredLogs] = useState<EmailLog[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState<'all' | 'gesendet' | 'fehler'>('all');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadAllEmailLogs();
  }, []);

  useEffect(() => {
    filterLogs();
  }, [searchQuery, filterStatus, emailLogs]);

  const loadAllEmailLogs = async () => {
    setLoading(true);
    setError(null);
    try {
      // TODO: Backend-Command für alle Email-Logs implementieren
      // Aktuell: Lade Beispieldaten
      const mockData: EmailLog[] = [
        {
          id: 1,
          booking_id: 1,
          guest_id: 1,
          template_used: 'confirmation',
          recipient_email: 'max@mustermann.de',
          subject: 'Buchungsbestätigung - Reservierung RES-2025-001',
          status: 'gesendet',
          error_message: null,
          sent_at: '2025-01-15 14:30:00',
        },
        {
          id: 2,
          booking_id: 1,
          guest_id: 1,
          template_used: 'invoice',
          recipient_email: 'max@mustermann.de',
          subject: 'Rechnung - Reservierung RES-2025-001',
          status: 'gesendet',
          error_message: null,
          sent_at: '2025-01-15 14:30:05',
        },
        {
          id: 3,
          booking_id: 2,
          guest_id: 2,
          template_used: 'confirmation',
          recipient_email: 'anna@beispiel.de',
          subject: 'Buchungsbestätigung - Reservierung RES-2025-002',
          status: 'fehler',
          error_message: 'SMTP Connection timeout',
          sent_at: '2025-01-16 10:15:00',
        },
      ];

      setEmailLogs(mockData);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const filterLogs = () => {
    let filtered = emailLogs;

    // Status-Filter
    if (filterStatus !== 'all') {
      filtered = filtered.filter((log) => log.status === filterStatus);
    }

    // Such-Filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (log) =>
          log.recipient_email.toLowerCase().includes(query) ||
          log.subject.toLowerCase().includes(query) ||
          log.template_used.toLowerCase().includes(query)
      );
    }

    setFilteredLogs(filtered);
  };

  const getTemplateDisplayName = (name: string) => {
    const names: { [key: string]: string } = {
      confirmation: 'Buchungsbestätigung',
      reminder: 'Erinnerung',
      invoice: 'Rechnung',
      payment_reminder: 'Zahlungserinnerung',
      cancellation: 'Stornierungsbestätigung',
    };
    return names[name] || name;
  };

  const formatDateTime = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString('de-DE', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const handleResend = async (log: EmailLog) => {
    // TODO: Implementiere Resend-Funktion
    alert(`Email erneut senden für Buchung #${log.booking_id}`);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-6">
        <p className="text-red-800">Fehler beim Laden der Email-Historie: {error}</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-white flex items-center gap-3">
            <Mail className="w-7 h-7" />
            Email-Verlauf
          </h2>
          <p className="text-sm text-slate-400 mt-1">
            Übersicht aller versendeten Emails
          </p>
        </div>
        <button
          onClick={loadAllEmailLogs}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
        >
          <RefreshCw className="w-4 h-4" />
          Aktualisieren
        </button>
      </div>

      {/* Filters */}
      <div className="bg-slate-800 rounded-xl p-4 border border-slate-700">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Search */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Suchen
            </label>
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-slate-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Email, Betreff oder Template..."
                className="w-full pl-10 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          {/* Status Filter */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Status
            </label>
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value as 'all' | 'gesendet' | 'fehler')}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">Alle ({emailLogs.length})</option>
              <option value="gesendet">
                Gesendet ({emailLogs.filter((l) => l.status === 'gesendet').length})
              </option>
              <option value="fehler">
                Fehler ({emailLogs.filter((l) => l.status === 'fehler').length})
              </option>
            </select>
          </div>
        </div>
      </div>

      {/* Statistics */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl p-6 text-white">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm opacity-90">Gesamt versendet</p>
              <p className="text-3xl font-bold mt-1">{emailLogs.length}</p>
            </div>
            <Mail className="w-12 h-12 opacity-30" />
          </div>
        </div>

        <div className="bg-gradient-to-br from-emerald-500 to-emerald-600 rounded-xl p-6 text-white">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm opacity-90">Erfolgreich</p>
              <p className="text-3xl font-bold mt-1">
                {emailLogs.filter((l) => l.status === 'gesendet').length}
              </p>
            </div>
            <CheckCircle className="w-12 h-12 opacity-30" />
          </div>
        </div>

        <div className="bg-gradient-to-br from-red-500 to-red-600 rounded-xl p-6 text-white">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm opacity-90">Fehlgeschlagen</p>
              <p className="text-3xl font-bold mt-1">
                {emailLogs.filter((l) => l.status === 'fehler').length}
              </p>
            </div>
            <AlertCircle className="w-12 h-12 opacity-30" />
          </div>
        </div>
      </div>

      {/* Email List */}
      <div className="bg-slate-800 rounded-xl border border-slate-700 overflow-hidden">
        {filteredLogs.length === 0 ? (
          <div className="p-12 text-center">
            <Mail className="w-16 h-16 text-slate-600 mx-auto mb-4" />
            <p className="text-slate-400">
              {searchQuery || filterStatus !== 'all'
                ? 'Keine Emails gefunden'
                : 'Noch keine Emails versendet'}
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-slate-900 border-b border-slate-700">
                <tr>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-300 uppercase tracking-wider">
                    Datum & Uhrzeit
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-300 uppercase tracking-wider">
                    Empfänger
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-300 uppercase tracking-wider">
                    Template
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-300 uppercase tracking-wider">
                    Betreff
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-300 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-slate-300 uppercase tracking-wider">
                    Aktionen
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700">
                {filteredLogs.map((log) => (
                  <tr key={log.id} className="hover:bg-slate-700/50 transition-colors">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-2 text-sm text-slate-300">
                        <Clock className="w-4 h-4 text-slate-500" />
                        {formatDateTime(log.sent_at)}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm text-white font-medium">{log.recipient_email}</div>
                    </td>
                    <td className="px-6 py-4">
                      <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-blue-500/20 text-blue-300 border border-blue-500/30">
                        {getTemplateDisplayName(log.template_used)}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm text-slate-300 max-w-md truncate">
                        {log.subject}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      {log.status === 'gesendet' ? (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                          <CheckCircle className="w-3.5 h-3.5" />
                          Gesendet
                        </span>
                      ) : (
                        <div className="space-y-1">
                          <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-red-500/20 text-red-300 border border-red-500/30">
                            <AlertCircle className="w-3.5 h-3.5" />
                            Fehler
                          </span>
                          {log.error_message && (
                            <p className="text-xs text-red-400">{log.error_message}</p>
                          )}
                        </div>
                      )}
                    </td>
                    <td className="px-6 py-4">
                      <button
                        onClick={() => handleResend(log)}
                        className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-xs font-medium transition-colors"
                        title="Erneut senden"
                      >
                        <Send className="w-3.5 h-3.5" />
                        Erneut
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
