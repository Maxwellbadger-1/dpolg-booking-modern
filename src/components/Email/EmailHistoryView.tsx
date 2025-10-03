import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Mail, Search, CheckCircle, AlertCircle, Clock, RefreshCw, Send, FileText } from 'lucide-react';

interface EmailLog {
  id: number;
  booking_id: number;
  guest_id: number;
  template_name: string;
  recipient_email: string;
  subject: string;
  status: string;
  error_message: string | null;
  sent_at: string;
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
      const logs = await invoke<EmailLog[]>('get_all_email_logs_command');
      setEmailLogs(logs);
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
          log.template_name.toLowerCase().includes(query)
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

  const getTemplateColor = (name: string) => {
    const colors: { [key: string]: string } = {
      confirmation: 'bg-blue-500/20 text-blue-300 border-blue-500/30',
      reminder: 'bg-amber-500/20 text-amber-300 border-amber-500/30',
      invoice: 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30',
      payment_reminder: 'bg-orange-500/20 text-orange-300 border-orange-500/30',
      cancellation: 'bg-red-500/20 text-red-300 border-red-500/30',
    };
    return colors[name] || 'bg-slate-500/20 text-slate-300 border-slate-500/30';
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
    if (!confirm('Email wirklich erneut senden?')) return;

    try {
      let commandName = '';
      switch (log.template_name) {
        case 'confirmation':
          commandName = 'send_confirmation_email_command';
          break;
        case 'reminder':
          commandName = 'send_reminder_email_command';
          break;
        case 'invoice':
          commandName = 'send_invoice_email_command';
          break;
        case 'payment_reminder':
          commandName = 'send_payment_reminder_email_command';
          break;
        case 'cancellation':
          commandName = 'send_cancellation_email_command';
          break;
        default:
          alert('Unbekannter Template-Typ');
          return;
      }

      await invoke(commandName, { bookingId: log.booking_id });
      alert('Email erfolgreich versendet!');
      loadAllEmailLogs(); // Reload to show new entry
    } catch (err) {
      alert(`Fehler beim Senden: ${err}`);
    }
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
      <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-6">
        <div className="flex items-center gap-3">
          <AlertCircle className="w-6 h-6 text-red-400" />
          <div>
            <h3 className="text-lg font-semibold text-red-300">Fehler beim Laden</h3>
            <p className="text-red-400 mt-1">{error}</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header mit verbesserter Optik */}
      <div className="flex items-center justify-between bg-gradient-to-r from-slate-800 to-slate-900 rounded-xl p-6 border border-slate-700">
        <div>
          <h2 className="text-3xl font-bold text-white flex items-center gap-3">
            <Mail className="w-8 h-8 text-blue-400" />
            Email-Verlauf
          </h2>
          <p className="text-sm text-slate-400 mt-2">
            Übersicht aller versendeten Emails
          </p>
        </div>
        <button
          onClick={loadAllEmailLogs}
          className="flex items-center gap-2 px-5 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-all hover:scale-105 active:scale-95"
        >
          <RefreshCw className="w-5 h-5" />
          Aktualisieren
        </button>
      </div>

      {/* Statistics - verbesserte Lesbarkeit */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-gradient-to-br from-slate-800 to-slate-900 border border-slate-700 rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-slate-400 uppercase tracking-wide">Gesamt versendet</p>
              <p className="text-4xl font-bold text-white mt-2">{emailLogs.length}</p>
            </div>
            <div className="bg-blue-500/20 p-4 rounded-xl">
              <Mail className="w-8 h-8 text-blue-400" />
            </div>
          </div>
        </div>

        <div className="bg-gradient-to-br from-slate-800 to-slate-900 border border-slate-700 rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-slate-400 uppercase tracking-wide">Erfolgreich</p>
              <p className="text-4xl font-bold text-emerald-400 mt-2">
                {emailLogs.filter((l) => l.status === 'gesendet').length}
              </p>
            </div>
            <div className="bg-emerald-500/20 p-4 rounded-xl">
              <CheckCircle className="w-8 h-8 text-emerald-400" />
            </div>
          </div>
        </div>

        <div className="bg-gradient-to-br from-slate-800 to-slate-900 border border-slate-700 rounded-xl p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-slate-400 uppercase tracking-wide">Fehlgeschlagen</p>
              <p className="text-4xl font-bold text-red-400 mt-2">
                {emailLogs.filter((l) => l.status === 'fehler').length}
              </p>
            </div>
            <div className="bg-red-500/20 p-4 rounded-xl">
              <AlertCircle className="w-8 h-8 text-red-400" />
            </div>
          </div>
        </div>
      </div>

      {/* Filters - verbesserte Lesbarkeit */}
      <div className="bg-slate-800 rounded-xl p-6 border border-slate-700">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Search */}
          <div>
            <label className="block text-sm font-semibold text-slate-300 mb-3">
              Suchen
            </label>
            <div className="relative">
              <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-slate-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Email, Betreff oder Template suchen..."
                className="w-full pl-12 pr-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
              />
            </div>
          </div>

          {/* Status Filter */}
          <div>
            <label className="block text-sm font-semibold text-slate-300 mb-3">
              Status filtern
            </label>
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value as 'all' | 'gesendet' | 'fehler')}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            >
              <option value="all">Alle anzeigen ({emailLogs.length})</option>
              <option value="gesendet">
                Nur Erfolgreiche ({emailLogs.filter((l) => l.status === 'gesendet').length})
              </option>
              <option value="fehler">
                Nur Fehler ({emailLogs.filter((l) => l.status === 'fehler').length})
              </option>
            </select>
          </div>
        </div>
      </div>

      {/* Email List - verbesserte Tabelle */}
      <div className="bg-slate-800 rounded-xl border border-slate-700 overflow-hidden">
        {filteredLogs.length === 0 ? (
          <div className="p-16 text-center">
            <div className="bg-slate-700/50 w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-6">
              <Mail className="w-10 h-10 text-slate-500" />
            </div>
            <h3 className="text-xl font-semibold text-slate-300 mb-2">
              {searchQuery || filterStatus !== 'all' ? 'Keine Emails gefunden' : 'Noch keine Emails versendet'}
            </h3>
            <p className="text-slate-400">
              {searchQuery || filterStatus !== 'all'
                ? 'Versuche eine andere Suche oder Filter'
                : 'Emails werden hier angezeigt, sobald sie versendet wurden'}
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-slate-900 border-b border-slate-700">
                <tr>
                  <th className="px-6 py-4 text-left text-xs font-bold text-slate-300 uppercase tracking-wider">
                    Datum & Zeit
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-bold text-slate-300 uppercase tracking-wider">
                    Empfänger
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-bold text-slate-300 uppercase tracking-wider">
                    Template
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-bold text-slate-300 uppercase tracking-wider">
                    Betreff
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-bold text-slate-300 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-bold text-slate-300 uppercase tracking-wider">
                    Aktionen
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700">
                {filteredLogs.map((log) => (
                  <tr key={log.id} className="hover:bg-slate-700/30 transition-colors">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-2 text-sm font-medium text-slate-300">
                        <Clock className="w-4 h-4 text-slate-500" />
                        {formatDateTime(log.sent_at)}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm text-white font-semibold">{log.recipient_email}</div>
                      <div className="text-xs text-slate-400 mt-1">Buchung #{log.booking_id}</div>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border ${getTemplateColor(log.template_name)}`}>
                        <FileText className="w-3.5 h-3.5" />
                        {getTemplateDisplayName(log.template_name)}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm text-slate-200 max-w-md truncate font-medium">
                        {log.subject}
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      {log.status === 'gesendet' ? (
                        <span className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                          <CheckCircle className="w-4 h-4" />
                          Gesendet
                        </span>
                      ) : (
                        <div className="space-y-1">
                          <span className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-red-500/20 text-red-300 border border-red-500/30">
                            <AlertCircle className="w-4 h-4" />
                            Fehler
                          </span>
                          {log.error_message && (
                            <p className="text-xs text-red-400 font-medium">{log.error_message}</p>
                          )}
                        </div>
                      )}
                    </td>
                    <td className="px-6 py-4">
                      <button
                        onClick={() => handleResend(log)}
                        className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-semibold transition-all hover:scale-105 active:scale-95"
                        title="Email erneut senden"
                      >
                        <Send className="w-4 h-4" />
                        Erneut senden
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
