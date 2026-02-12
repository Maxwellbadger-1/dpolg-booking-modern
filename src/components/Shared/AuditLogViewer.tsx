import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { History, User, Clock, ChevronDown, ChevronUp, AlertCircle } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { de } from 'date-fns/locale';

interface AuditLog {
  id: number;
  tableName: string;
  recordId: number;
  action: string;
  oldData: string | null;  // JSON string
  newData: string | null;  // JSON string
  userName: string | null;
  changedAt: string;
}

interface AuditLogViewerProps {
  bookingId?: number;
  className?: string;
}

export function AuditLogViewer({ bookingId, className = '' }: AuditLogViewerProps) {
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedLogs, setExpandedLogs] = useState<Set<number>>(new Set());

  useEffect(() => {
    loadAuditLogs();
  }, [bookingId]);

  const loadAuditLogs = async () => {
    setLoading(true);
    setError(null);

    try {
      let result: AuditLog[];

      if (bookingId) {
        result = await invoke('get_booking_audit_log_pg', { bookingId });
      } else {
        result = await invoke('get_audit_log_pg', {
          tableName: null,
          recordId: null,
          limit: 100,
        });
      }

      setLogs(result);
    } catch (err) {
      console.error('Failed to load audit logs:', err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const toggleExpand = (logId: number) => {
    setExpandedLogs((prev) => {
      const next = new Set(prev);
      if (next.has(logId)) {
        next.delete(logId);
      } else {
        next.add(logId);
      }
      return next;
    });
  };

  const getActionBadge = (action: string) => {
    switch (action) {
      case 'INSERT':
        return <span className="px-2 py-0.5 text-xs font-medium bg-green-100 text-green-800 rounded">Erstellt</span>;
      case 'UPDATE':
        return <span className="px-2 py-0.5 text-xs font-medium bg-blue-100 text-blue-800 rounded">Geändert</span>;
      case 'DELETE':
        return <span className="px-2 py-0.5 text-xs font-medium bg-red-100 text-red-800 rounded">Gelöscht</span>;
      default:
        return <span className="px-2 py-0.5 text-xs font-medium bg-gray-100 text-gray-800 rounded">{action}</span>;
    }
  };

  const getChangedFields = (oldDataStr: string | null, newDataStr: string | null): string[] => {
    if (!oldDataStr || !newDataStr) return [];

    try {
      const oldData = JSON.parse(oldDataStr);
      const newData = JSON.parse(newDataStr);
      const changedFields: string[] = [];

      Object.keys(newData).forEach((key) => {
        if (JSON.stringify(oldData[key]) !== JSON.stringify(newData[key])) {
          // Skip technical fields
          if (!['updated_at', 'updated_by', 'id'].includes(key)) {
            changedFields.push(key);
          }
        }
      });

      return changedFields;
    } catch (e) {
      console.error('Failed to parse audit log JSON:', e);
      return [];
    }
  };

  const renderFieldChange = (field: string, oldValue: any, newValue: any) => {
    const formatValue = (val: any) => {
      if (val === null || val === undefined) return <span className="text-gray-400 italic">(leer)</span>;
      if (typeof val === 'boolean') return val ? 'Ja' : 'Nein';
      if (typeof val === 'object') return JSON.stringify(val);
      return String(val);
    };

    return (
      <div key={field} className="py-2 border-b border-gray-100 last:border-0">
        <div className="text-sm font-medium text-gray-700 mb-1">{field}</div>
        <div className="flex items-center gap-2 text-sm">
          <span className="text-red-600 line-through">{formatValue(oldValue)}</span>
          <span className="text-gray-400">→</span>
          <span className="text-green-600 font-medium">{formatValue(newValue)}</span>
        </div>
      </div>
    );
  };

  if (loading) {
    return (
      <div className={`flex items-center justify-center py-8 ${className}`}>
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`bg-red-50 border border-red-200 rounded-lg p-4 ${className}`}>
        <div className="flex items-center gap-2 text-red-800">
          <AlertCircle className="w-5 h-5" />
          <span>Fehler beim Laden der Änderungshistorie: {error}</span>
        </div>
      </div>
    );
  }

  if (logs.length === 0) {
    return (
      <div className={`text-center py-8 text-gray-500 ${className}`}>
        <History className="w-12 h-12 mx-auto mb-3 opacity-30" />
        <p>Keine Änderungen gefunden</p>
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      <div className="flex items-center gap-2 mb-4">
        <History className="w-5 h-5 text-gray-600" />
        <h3 className="text-lg font-semibold text-gray-900">Änderungshistorie</h3>
        <span className="text-sm text-gray-500">({logs.length} Einträge)</span>
      </div>

      <div className="space-y-2">
        {logs.map((log) => {
          const isExpanded = expandedLogs.has(log.id);
          const changedFields = getChangedFields(log.oldData, log.newData);
          const hasDetails = changedFields.length > 0;

          return (
            <div
              key={log.id}
              className="bg-white border border-gray-200 rounded-lg overflow-hidden hover:shadow-md transition-shadow"
            >
              {/* Header */}
              <button
                onClick={() => hasDetails && toggleExpand(log.id)}
                className={`w-full p-4 flex items-start gap-3 text-left ${
                  hasDetails ? 'hover:bg-gray-50 cursor-pointer' : 'cursor-default'
                }`}
              >
                <div className="flex-shrink-0 mt-1">
                  {getActionBadge(log.action)}
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-medium text-gray-900">
                      {`${log.action} on ${log.tableName}`}
                    </span>
                  </div>

                  <div className="flex items-center gap-4 text-sm text-gray-600">
                    {log.userName && (
                      <div className="flex items-center gap-1">
                        <User className="w-3.5 h-3.5" />
                        <span>{log.userName}</span>
                      </div>
                    )}

                    <div className="flex items-center gap-1">
                      <Clock className="w-3.5 h-3.5" />
                      <span>
                        {formatDistanceToNow(new Date(log.changedAt), {
                          addSuffix: true,
                          locale: de,
                        })}
                      </span>
                    </div>

                    {hasDetails && (
                      <span className="text-blue-600">
                        {changedFields.length} Feld(er) geändert
                      </span>
                    )}
                  </div>
                </div>

                {hasDetails && (
                  <div className="flex-shrink-0">
                    {isExpanded ? (
                      <ChevronUp className="w-5 h-5 text-gray-400" />
                    ) : (
                      <ChevronDown className="w-5 h-5 text-gray-400" />
                    )}
                  </div>
                )}
              </button>

              {/* Expanded Details */}
              {isExpanded && hasDetails && (
                <div className="px-4 pb-4 border-t border-gray-100 bg-gray-50">
                  <div className="mt-3 space-y-1">
                    {changedFields.map((field) => {
                      try {
                        const oldData = log.oldData ? JSON.parse(log.oldData) : {};
                        const newData = log.newData ? JSON.parse(log.newData) : {};
                        return renderFieldChange(field, oldData[field], newData[field]);
                      } catch (e) {
                        return null;
                      }
                    })}
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
