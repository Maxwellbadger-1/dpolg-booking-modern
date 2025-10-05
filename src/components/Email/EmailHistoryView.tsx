import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Mail, Search, CheckCircle, AlertCircle, Clock, RefreshCw, Send, FileText, History, CalendarClock } from 'lucide-react';
import { SELECT_SMALL_STYLES, SELECT_SMALL_BACKGROUND_STYLE } from '../../lib/selectStyles';

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

interface ScheduledEmail {
  booking_id: number;
  reservierungsnummer: string;
  guest_name: string;
  guest_email: string;
  email_type: string;
  scheduled_date: string;
  reason: string;
}

type TabType = 'history' | 'scheduled';

export default function EmailHistoryView() {
  const [activeTab, setActiveTab] = useState<TabType>('history');

  // History Tab State
  const [emailLogs, setEmailLogs] = useState<EmailLog[]>([]);
  const [filteredLogs, setFilteredLogs] = useState<EmailLog[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState<'all' | 'gesendet' | 'fehler'>('all');

  // Scheduled Tab State
  const [scheduledEmails, setScheduledEmails] = useState<ScheduledEmail[]>([]);

  // Common State
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (activeTab === 'history') {
      loadAllEmailLogs();
    } else {
      loadScheduledEmails();
    }
  }, [activeTab]);

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

  const loadScheduledEmails = async () => {
    setLoading(true);
    setError(null);
    try {
      const scheduled = await invoke<ScheduledEmail[]>('get_scheduled_emails');
      setScheduledEmails(scheduled);
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
      confirmation: 'bg-blue-100 text-blue-700 border-blue-200',
      reminder: 'bg-amber-100 text-amber-700 border-amber-200',
      invoice: 'bg-emerald-100 text-emerald-700 border-emerald-200',
      payment_reminder: 'bg-orange-100 text-orange-700 border-orange-200',
      cancellation: 'bg-red-100 text-red-700 border-red-200',
    };
    return colors[name] || 'bg-slate-100 text-slate-700 border-slate-200';
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
      loadAllEmailLogs();
    } catch (err) {
      alert(`Fehler beim Senden: ${err}`);
    }
  };

  const handleRefresh = () => {
    if (activeTab === 'history') {
      loadAllEmailLogs();
    } else {
      loadScheduledEmails();
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
    <div className="h-full flex flex-col p-6 space-y-4 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-3">
          <div className="bg-blue-500/20 p-2 rounded-lg">
            <Mail className="w-6 h-6 text-blue-400" />
          </div>
          <div>
            <h2 className="text-2xl font-bold text-slate-800">Email-Übersicht</h2>
            <p className="text-xs text-slate-600">Versendete und geplante Emails</p>
          </div>
        </div>
        <button
          onClick={handleRefresh}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors text-sm"
        >
          <RefreshCw className="w-4 h-4" />
          Aktualisieren
        </button>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 border-b border-slate-200 flex-shrink-0">
        <button
          onClick={() => setActiveTab('history')}
          className={`flex items-center gap-2 px-4 py-3 font-semibold transition-all border-b-2 ${
            activeTab === 'history'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-slate-600 hover:text-slate-800'
          }`}
        >
          <History className="w-4 h-4" />
          Verlauf
          <span className={`ml-1 px-2 py-0.5 rounded-full text-xs font-bold ${
            activeTab === 'history' ? 'bg-blue-100 text-blue-700' : 'bg-slate-100 text-slate-600'
          }`}>
            {emailLogs.length}
          </span>
        </button>
        <button
          onClick={() => setActiveTab('scheduled')}
          className={`flex items-center gap-2 px-4 py-3 font-semibold transition-all border-b-2 ${
            activeTab === 'scheduled'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-slate-600 hover:text-slate-800'
          }`}
        >
          <CalendarClock className="w-4 h-4" />
          Geplant
          <span className={`ml-1 px-2 py-0.5 rounded-full text-xs font-bold ${
            activeTab === 'scheduled' ? 'bg-blue-100 text-blue-700' : 'bg-slate-100 text-slate-600'
          }`}>
            {scheduledEmails.length}
          </span>
        </button>
      </div>

      {/* Content */}
      {activeTab === 'history' ? (
        <>
          {/* Statistics - Kompakt */}
          <div className="grid grid-cols-3 gap-3 flex-shrink-0">
            <div className="bg-white border border-slate-200 rounded-lg p-3">
              <div className="flex items-center gap-3">
                <div className="bg-blue-500/20 p-2 rounded-lg">
                  <Mail className="w-5 h-5 text-blue-600" />
                </div>
                <div>
                  <p className="text-xs font-medium text-slate-600">Gesamt</p>
                  <p className="text-2xl font-bold text-slate-800">{emailLogs.length}</p>
                </div>
              </div>
            </div>

            <div className="bg-white border border-slate-200 rounded-lg p-3">
              <div className="flex items-center gap-3">
                <div className="bg-emerald-500/20 p-2 rounded-lg">
                  <CheckCircle className="w-5 h-5 text-emerald-600" />
                </div>
                <div>
                  <p className="text-xs font-medium text-slate-600">Erfolgreich</p>
                  <p className="text-2xl font-bold text-emerald-600">
                    {emailLogs.filter((l) => l.status === 'gesendet').length}
                  </p>
                </div>
              </div>
            </div>

            <div className="bg-white border border-slate-200 rounded-lg p-3">
              <div className="flex items-center gap-3">
                <div className="bg-red-500/20 p-2 rounded-lg">
                  <AlertCircle className="w-5 h-5 text-red-600" />
                </div>
                <div>
                  <p className="text-xs font-medium text-slate-600">Fehler</p>
                  <p className="text-2xl font-bold text-red-600">
                    {emailLogs.filter((l) => l.status === 'fehler').length}
                  </p>
                </div>
              </div>
            </div>
          </div>

          {/* Filters - Kompakt */}
          <div className="bg-white border border-slate-200 rounded-lg p-3 flex-shrink-0">
            <div className="grid grid-cols-2 gap-3">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-slate-400" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Email, Betreff oder Template suchen..."
                  className="w-full pl-10 pr-3 py-2 bg-slate-50 border border-slate-200 rounded-lg text-slate-800 placeholder-slate-400 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>

              <select
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value as 'all' | 'gesendet' | 'fehler')}
                className={SELECT_SMALL_STYLES}
                style={SELECT_SMALL_BACKGROUND_STYLE}
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

          {/* Email List - Maximaler Platz */}
          <div className="bg-white border border-slate-200 rounded-lg overflow-hidden flex-1 flex flex-col min-h-0">
            {filteredLogs.length === 0 ? (
              <div className="p-12 text-center flex-1 flex flex-col items-center justify-center">
                <div className="bg-slate-100 w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Mail className="w-8 h-8 text-slate-400" />
                </div>
                <h3 className="text-lg font-semibold text-slate-700 mb-1">
                  {searchQuery || filterStatus !== 'all' ? 'Keine Emails gefunden' : 'Noch keine Emails versendet'}
                </h3>
                <p className="text-sm text-slate-500">
                  {searchQuery || filterStatus !== 'all'
                    ? 'Versuche eine andere Suche oder Filter'
                    : 'Emails werden hier angezeigt, sobald sie versendet wurden'}
                </p>
              </div>
            ) : (
              <div className="overflow-auto flex-1">
                <table className="w-full">
                  <thead className="bg-slate-50 border-b border-slate-200 sticky top-0">
                    <tr>
                      <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                        Datum & Zeit
                      </th>
                      <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                        Empfänger
                      </th>
                      <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                        Template
                      </th>
                      <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                        Betreff
                      </th>
                      <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                        Status
                      </th>
                      <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                        Aktionen
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {filteredLogs.map((log) => (
                      <tr key={log.id} className="hover:bg-slate-50 transition-colors">
                        <td className="px-4 py-3 whitespace-nowrap">
                          <div className="flex items-center gap-2 text-sm text-slate-600">
                            <Clock className="w-4 h-4 text-slate-400" />
                            {formatDateTime(log.sent_at)}
                          </div>
                        </td>
                        <td className="px-4 py-3">
                          <div className="text-sm text-slate-800 font-semibold">{log.recipient_email}</div>
                          <div className="text-xs text-slate-500 mt-0.5">Buchung #{log.booking_id}</div>
                        </td>
                        <td className="px-4 py-3">
                          <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-semibold border ${getTemplateColor(log.template_name)}`}>
                            <FileText className="w-3.5 h-3.5" />
                            {getTemplateDisplayName(log.template_name)}
                          </span>
                        </td>
                        <td className="px-4 py-3">
                          <div className="text-sm text-slate-700 max-w-md truncate">
                            {log.subject}
                          </div>
                        </td>
                        <td className="px-4 py-3">
                          {log.status === 'gesendet' ? (
                            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-semibold bg-emerald-100 text-emerald-700 border border-emerald-200">
                              <CheckCircle className="w-3.5 h-3.5" />
                              Gesendet
                            </span>
                          ) : (
                            <div className="space-y-1">
                              <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-semibold bg-red-100 text-red-700 border border-red-200">
                                <AlertCircle className="w-3.5 h-3.5" />
                                Fehler
                              </span>
                              {log.error_message && (
                                <p className="text-xs text-red-600">{log.error_message}</p>
                              )}
                            </div>
                          )}
                        </td>
                        <td className="px-4 py-3">
                          <button
                            onClick={() => handleResend(log)}
                            className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-xs font-semibold transition-colors"
                            title="Email erneut senden"
                          >
                            <Send className="w-3.5 h-3.5" />
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
        </>
      ) : (
        /* Scheduled Emails Tab */
        <div className="bg-white border border-slate-200 rounded-lg overflow-hidden flex-1 flex flex-col min-h-0">
          {scheduledEmails.length === 0 ? (
            <div className="p-12 text-center flex-1 flex flex-col items-center justify-center">
              <div className="bg-slate-100 w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4">
                <CalendarClock className="w-8 h-8 text-slate-400" />
              </div>
              <h3 className="text-lg font-semibold text-slate-700 mb-1">
                Keine geplanten Emails
              </h3>
              <p className="text-sm text-slate-500">
                Automatische Erinnerungen und Zahlungserinnerungen werden hier angezeigt
              </p>
            </div>
          ) : (
            <div className="overflow-auto flex-1">
              <table className="w-full">
                <thead className="bg-slate-50 border-b border-slate-200 sticky top-0">
                  <tr>
                    <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                      Reservierung
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                      Gast
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                      Email-Typ
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                      Geplant für
                    </th>
                    <th className="px-4 py-3 text-left text-xs font-bold text-slate-700 uppercase tracking-wider">
                      Grund
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {scheduledEmails.map((email, index) => (
                    <tr key={index} className="hover:bg-slate-50 transition-colors">
                      <td className="px-4 py-3 whitespace-nowrap">
                        <div className="text-sm font-semibold text-slate-800">{email.reservierungsnummer}</div>
                        <div className="text-xs text-slate-500 mt-0.5">ID: {email.booking_id}</div>
                      </td>
                      <td className="px-4 py-3">
                        <div className="text-sm text-slate-800 font-semibold">{email.guest_name}</div>
                        <div className="text-xs text-slate-500 mt-0.5">{email.guest_email}</div>
                      </td>
                      <td className="px-4 py-3">
                        <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-semibold border ${getTemplateColor(email.email_type)}`}>
                          <FileText className="w-3.5 h-3.5" />
                          {getTemplateDisplayName(email.email_type)}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2 text-sm text-slate-600">
                          <CalendarClock className="w-4 h-4 text-slate-400" />
                          {email.scheduled_date}
                        </div>
                      </td>
                      <td className="px-4 py-3">
                        <div className="text-sm text-slate-700">{email.reason}</div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
