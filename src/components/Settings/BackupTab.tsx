import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Save, RefreshCw, FolderOpen, RotateCcw, Trash2, CheckCircle, AlertCircle, HardDrive, AlertTriangle, Undo, Clock, Database, FileText, UserCircle } from 'lucide-react';

interface BackupInfo {
  filename: string;
  path: string;
  created_at: string;
  size_bytes: number;
  size_formatted: string;
}

interface BackupSettings {
  auto_backup_enabled: boolean;
  backup_interval: string;
  max_backups: number;
  last_backup_at: string | null;
}

interface TransactionLog {
  id: number;
  operation_type: string;
  table_name: string;
  record_id: number;
  old_data: string | null;
  new_data: string | null;
  user_action: string;
  created_at: string;
  can_undo: boolean;
}

export default function BackupTab() {
  const [settings, setSettings] = useState<BackupSettings>({
    auto_backup_enabled: true,
    backup_interval: 'on_startup',
    max_backups: 10,
    last_backup_at: null,
  });
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [transactions, setTransactions] = useState<TransactionLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingTransactions, setLoadingTransactions] = useState(false);
  const [saving, setSaving] = useState(false);
  const [creating, setCreating] = useState(false);
  const [success, setSuccess] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [restoreDialog, setRestoreDialog] = useState<{ show: boolean; backup: BackupInfo | null }>({ show: false, backup: null });
  const [deleteDialog, setDeleteDialog] = useState<{ show: boolean; backup: BackupInfo | null }>({ show: false, backup: null });
  const [restoring, setRestoring] = useState(false);
  const [undoingId, setUndoingId] = useState<number | null>(null);

  // Load settings and backups
  useEffect(() => {
    loadSettings();
    loadBackups();
    loadTransactions();
  }, []);

  const loadSettings = async () => {
    try {
      const result = await invoke<BackupSettings>('get_backup_settings_command');
      setSettings(result);
    } catch (err) {
      setError(`Fehler beim Laden der Einstellungen: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const loadBackups = async () => {
    try {
      const result = await invoke<BackupInfo[]>('list_backups_command');
      setBackups(result);
    } catch (err) {
      console.error('Fehler beim Laden der Backups:', err);
    }
  };

  const loadTransactions = async () => {
    setLoadingTransactions(true);
    try {
      const result = await invoke<TransactionLog[]>('get_recent_transactions_command', { limit: 20 });
      setTransactions(result);
    } catch (err) {
      console.error('Fehler beim Laden der Transaktionen:', err);
    } finally {
      setLoadingTransactions(false);
    }
  };

  const handleSaveSettings = async () => {
    setSaving(true);
    setError(null);
    setSuccess(null);

    try {
      await invoke('save_backup_settings_command', { settings });
      setSuccess('Einstellungen erfolgreich gespeichert');
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(`Fehler beim Speichern: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  const handleCreateBackup = async () => {
    setCreating(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await invoke<BackupInfo>('create_backup_command');
      setSuccess(`Backup erfolgreich erstellt: ${result.filename}`);
      loadBackups();
      loadSettings(); // Refresh last backup time
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(`Fehler beim Erstellen des Backups: ${err}`);
    } finally {
      setCreating(false);
    }
  };

  const handleRestoreBackup = async () => {
    if (!restoreDialog.backup) return;

    setRestoring(true);
    setError(null);

    try {
      const result = await invoke<string>('restore_backup_command', { backupPath: restoreDialog.backup.path });
      setSuccess(result + ' Bitte starten Sie die Anwendung neu.');
      setRestoreDialog({ show: false, backup: null });
      setTimeout(() => setSuccess(null), 10000);
    } catch (err) {
      setError(`Fehler beim Wiederherstellen: ${err}`);
      setRestoreDialog({ show: false, backup: null });
    } finally {
      setRestoring(false);
    }
  };

  const handleDeleteBackup = async () => {
    if (!deleteDialog.backup) return;

    try {
      await invoke('delete_backup_command', { backupPath: deleteDialog.backup.path });
      setSuccess('Backup erfolgreich gelöscht');
      loadBackups();
      setDeleteDialog({ show: false, backup: null });
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      setError(`Fehler beim Löschen: ${err}`);
      setDeleteDialog({ show: false, backup: null });
    }
  };

  const handleOpenBackupFolder = async () => {
    try {
      await invoke('open_backup_folder_command');
    } catch (err) {
      setError(`Fehler beim Öffnen des Ordners: ${err}`);
    }
  };

  const handleUndoTransaction = async (logId: number) => {
    setUndoingId(logId);
    setError(null);
    setSuccess(null);

    try {
      const result = await invoke<string>('undo_transaction_command', { logId });
      setSuccess(result);
      loadTransactions(); // Refresh transaction list
      setTimeout(() => setSuccess(null), 5000);
    } catch (err) {
      setError(`Fehler beim Rückgängig machen: ${err}`);
    } finally {
      setUndoingId(null);
    }
  };

  const getOperationIcon = (operationType: string) => {
    switch (operationType) {
      case 'CREATE':
        return <FileText className="w-4 h-4 text-emerald-400" />;
      case 'UPDATE':
        return <RefreshCw className="w-4 h-4 text-blue-400" />;
      case 'DELETE':
        return <Trash2 className="w-4 h-4 text-red-400" />;
      default:
        return <Database className="w-4 h-4 text-slate-400" />;
    }
  };

  const getTableIcon = (tableName: string) => {
    switch (tableName) {
      case 'bookings':
        return <Database className="w-3 h-3" />;
      case 'guests':
        return <UserCircle className="w-3 h-3" />;
      case 'rooms':
        return <HardDrive className="w-3 h-3" />;
      default:
        return <FileText className="w-3 h-3" />;
    }
  };

  const getTableDisplayName = (tableName: string) => {
    switch (tableName) {
      case 'bookings':
        return 'Buchung';
      case 'guests':
        return 'Gast';
      case 'rooms':
        return 'Zimmer';
      default:
        return tableName;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <RefreshCw className="w-6 h-6 text-blue-500 animate-spin" />
        <span className="ml-2 text-slate-300">Lade Backup-Einstellungen...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Success/Error Messages */}
      {success && (
        <div className="flex items-center gap-3 p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-lg">
          <CheckCircle className="w-5 h-5 text-emerald-400 flex-shrink-0" />
          <p className="text-emerald-300">{success}</p>
        </div>
      )}

      {error && (
        <div className="flex items-start gap-3 p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
          <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
          <p className="text-red-300">{error}</p>
        </div>
      )}

      {/* Backup Settings */}
      <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700">
        <div className="flex items-center gap-3 mb-6">
          <div className="p-2 bg-blue-500/10 rounded-lg">
            <HardDrive className="w-5 h-5 text-blue-400" />
          </div>
          <h2 className="text-xl font-bold text-white">Backup-Einstellungen</h2>
        </div>

        <div className="space-y-6">
          {/* Auto Backup Toggle */}
          <div className="flex items-center justify-between">
            <div>
              <label className="text-sm font-medium text-slate-300">Automatisches Backup</label>
              <p className="text-xs text-slate-400 mt-1">
                Erstellt automatisch Backups beim App-Start
              </p>
            </div>
            <button
              onClick={() => setSettings({ ...settings, auto_backup_enabled: !settings.auto_backup_enabled })}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                settings.auto_backup_enabled ? 'bg-blue-600' : 'bg-slate-600'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  settings.auto_backup_enabled ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          {/* Backup Interval */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-slate-300">
              Backup-Intervall
            </label>
            <select
              value={settings.backup_interval}
              onChange={(e) => setSettings({ ...settings, backup_interval: e.target.value })}
              className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="on_startup">Beim App-Start</option>
              <option value="daily">Täglich</option>
              <option value="weekly">Wöchentlich</option>
            </select>
            <p className="text-xs text-slate-400">
              Legt fest, wie oft automatisch ein Backup erstellt wird
            </p>
          </div>

          {/* Max Backups */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-slate-300">
              Maximale Anzahl Backups
            </label>
            <input
              type="number"
              min="1"
              max="50"
              value={settings.max_backups}
              onChange={(e) => setSettings({ ...settings, max_backups: parseInt(e.target.value) || 10 })}
              className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p className="text-xs text-slate-400">
              Ältere Backups werden automatisch gelöscht (Standard: 10)
            </p>
          </div>

          {/* Last Backup Info */}
          {settings.last_backup_at && (
            <div className="p-3 bg-slate-700/50 rounded-lg">
              <p className="text-xs text-slate-400">Letztes Backup:</p>
              <p className="text-sm text-white font-medium">{settings.last_backup_at}</p>
            </div>
          )}

          {/* Save Button */}
          <button
            onClick={handleSaveSettings}
            disabled={saving}
            className="w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-800 disabled:text-blue-300 text-white font-semibold rounded-lg transition-colors flex items-center justify-center gap-2"
          >
            {saving ? (
              <>
                <RefreshCw className="w-4 h-4 animate-spin" />
                Speichert...
              </>
            ) : (
              <>
                <Save className="w-4 h-4" />
                Einstellungen speichern
              </>
            )}
          </button>
        </div>
      </div>

      {/* Manual Backup */}
      <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700">
        <h3 className="text-lg font-semibold text-white mb-4">Manuelles Backup</h3>
        <div className="flex gap-3">
          <button
            onClick={handleCreateBackup}
            disabled={creating}
            className="flex-1 px-4 py-3 bg-emerald-600 hover:bg-emerald-700 disabled:bg-emerald-800 disabled:text-emerald-300 text-white font-semibold rounded-lg transition-colors flex items-center justify-center gap-2"
          >
            {creating ? (
              <>
                <RefreshCw className="w-4 h-4 animate-spin" />
                Erstelle Backup...
              </>
            ) : (
              <>
                <HardDrive className="w-4 h-4" />
                Jetzt Backup erstellen
              </>
            )}
          </button>

          <button
            onClick={handleOpenBackupFolder}
            className="px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors flex items-center gap-2"
          >
            <FolderOpen className="w-4 h-4" />
            Backup-Ordner öffnen
          </button>
        </div>
      </div>

      {/* Backup List */}
      <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Verfügbare Backups</h3>
          <button
            onClick={loadBackups}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
            title="Liste aktualisieren"
          >
            <RefreshCw className="w-4 h-4 text-slate-300" />
          </button>
        </div>

        {backups.length === 0 ? (
          <div className="text-center py-8">
            <HardDrive className="w-12 h-12 text-slate-600 mx-auto mb-3" />
            <p className="text-slate-400">Noch keine Backups vorhanden</p>
            <p className="text-sm text-slate-500 mt-1">
              Erstellen Sie Ihr erstes Backup mit dem Button oben
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {backups.map((backup) => (
              <div
                key={backup.path}
                className="flex items-center justify-between p-4 bg-slate-700/30 rounded-lg hover:bg-slate-700/50 transition-colors"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <CheckCircle className="w-4 h-4 text-emerald-400" />
                    <p className="text-sm font-medium text-white">{backup.created_at}</p>
                    <span className="px-2 py-0.5 bg-slate-600 text-xs text-slate-300 rounded">
                      {backup.size_formatted}
                    </span>
                  </div>
                  <p className="text-xs text-slate-400 mt-1 ml-6">{backup.filename}</p>
                </div>

                <div className="flex items-center gap-2">
                  <button
                    onClick={() => setRestoreDialog({ show: true, backup })}
                    className="p-2 hover:bg-blue-600 rounded-lg transition-colors group"
                    title="Wiederherstellen"
                  >
                    <RotateCcw className="w-4 h-4 text-slate-300 group-hover:text-white" />
                  </button>
                  <button
                    onClick={() => setDeleteDialog({ show: true, backup })}
                    className="p-2 hover:bg-red-600 rounded-lg transition-colors group"
                    title="Löschen"
                  >
                    <Trash2 className="w-4 h-4 text-slate-300 group-hover:text-white" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Undo/Redo (Transaction Log) */}
      <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-purple-500/10 rounded-lg">
              <Undo className="w-5 h-5 text-purple-400" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">Rückgängig machen</h2>
              <p className="text-sm text-slate-400">Letzte Änderungen an der Datenbank</p>
            </div>
          </div>
          <button
            onClick={loadTransactions}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
            title="Liste aktualisieren"
          >
            <RefreshCw className={`w-4 h-4 text-slate-300 ${loadingTransactions ? 'animate-spin' : ''}`} />
          </button>
        </div>

        {loadingTransactions ? (
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="w-5 h-5 text-purple-500 animate-spin" />
            <span className="ml-2 text-slate-300">Lade Transaktionen...</span>
          </div>
        ) : transactions.length === 0 ? (
          <div className="text-center py-8">
            <Clock className="w-12 h-12 text-slate-600 mx-auto mb-3" />
            <p className="text-slate-400">Keine Transaktionen vorhanden</p>
            <p className="text-sm text-slate-500 mt-1">
              Änderungen werden automatisch aufgezeichnet
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {transactions.map((transaction) => (
              <div
                key={transaction.id}
                className="flex items-center justify-between p-4 bg-slate-700/30 rounded-lg hover:bg-slate-700/50 transition-colors"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-3">
                    {getOperationIcon(transaction.operation_type)}
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-white">
                          {transaction.user_action}
                        </span>
                        <span className="flex items-center gap-1 px-2 py-0.5 bg-slate-600 text-xs text-slate-300 rounded">
                          {getTableIcon(transaction.table_name)}
                          {getTableDisplayName(transaction.table_name)}
                        </span>
                      </div>
                      <div className="flex items-center gap-2 mt-1">
                        <Clock className="w-3 h-3 text-slate-400" />
                        <p className="text-xs text-slate-400">{transaction.created_at}</p>
                        <span className="text-xs text-slate-500">•</span>
                        <span className="text-xs text-slate-400">ID: {transaction.record_id}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <button
                  onClick={() => handleUndoTransaction(transaction.id)}
                  disabled={!transaction.can_undo || undoingId === transaction.id}
                  className={`p-2 rounded-lg transition-colors flex items-center gap-2 ${
                    transaction.can_undo
                      ? 'hover:bg-purple-600 text-slate-300 hover:text-white'
                      : 'opacity-50 cursor-not-allowed text-slate-500'
                  }`}
                  title={transaction.can_undo ? 'Rückgängig machen' : 'Kann nicht rückgängig gemacht werden'}
                >
                  {undoingId === transaction.id ? (
                    <RefreshCw className="w-4 h-4 animate-spin" />
                  ) : (
                    <Undo className="w-4 h-4" />
                  )}
                </button>
              </div>
            ))}
          </div>
        )}

        <div className="mt-4 p-3 bg-purple-500/10 border border-purple-500/30 rounded-lg">
          <p className="text-xs text-purple-300">
            ℹ️ Nur die letzten 20 Transaktionen werden angezeigt. Bereits rückgängig gemachte Aktionen können nicht erneut rückgängig gemacht werden.
          </p>
        </div>
      </div>

      {/* Info Box */}
      <div className="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4">
        <div className="flex gap-3">
          <AlertCircle className="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" />
          <div className="text-sm text-blue-300">
            <p className="font-semibold mb-1">Wichtige Hinweise:</p>
            <ul className="list-disc list-inside space-y-1 text-blue-300/80">
              <li>Backups werden im App-Datenordner gespeichert</li>
              <li>Nach der Wiederherstellung muss die App neu gestartet werden</li>
              <li>Erstellen Sie regelmäßig externe Backups auf USB/Cloud</li>
              <li>Testen Sie Backups gelegentlich durch Wiederherstellung</li>
            </ul>
          </div>
        </div>
      </div>

      {/* Restore Confirmation Dialog */}
      {restoreDialog.show && restoreDialog.backup && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-amber-500/10 rounded-full">
                <AlertTriangle className="w-6 h-6 text-amber-400" />
              </div>
              <div className="flex-1">
                <h3 className="text-xl font-bold text-white mb-2">Backup wiederherstellen?</h3>
                <p className="text-slate-300 text-sm">
                  Möchten Sie dieses Backup wirklich wiederherstellen?
                </p>
              </div>
            </div>

            <div className="bg-slate-700/30 rounded-lg p-4 mb-6 space-y-2">
              <div>
                <span className="text-xs text-slate-400">Backup:</span>
                <p className="text-sm text-white font-medium">{restoreDialog.backup.filename}</p>
              </div>
              <div>
                <span className="text-xs text-slate-400">Erstellt:</span>
                <p className="text-sm text-white">{restoreDialog.backup.created_at}</p>
              </div>
              <div>
                <span className="text-xs text-slate-400">Größe:</span>
                <p className="text-sm text-white">{restoreDialog.backup.size_formatted}</p>
              </div>
            </div>

            <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4 mb-6">
              <p className="text-red-300 text-sm font-semibold flex items-start gap-2">
                <AlertTriangle className="w-4 h-4 flex-shrink-0 mt-0.5" />
                <span>
                  WARNUNG: Die aktuelle Datenbank wird überschrieben! Bitte starten Sie die Anwendung nach der Wiederherstellung neu.
                </span>
              </p>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => setRestoreDialog({ show: false, backup: null })}
                disabled={restoring}
                className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 disabled:bg-slate-700 disabled:opacity-50 text-white font-semibold rounded-lg transition-colors"
              >
                Abbrechen
              </button>
              <button
                onClick={handleRestoreBackup}
                disabled={restoring}
                className="flex-1 px-4 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-800 text-white font-semibold rounded-lg transition-colors flex items-center justify-center gap-2"
              >
                {restoring ? (
                  <>
                    <RefreshCw className="w-4 h-4 animate-spin" />
                    Stelle wieder her...
                  </>
                ) : (
                  <>
                    <RotateCcw className="w-4 h-4" />
                    Wiederherstellen
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation Dialog */}
      {deleteDialog.show && deleteDialog.backup && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-red-500/10 rounded-full">
                <Trash2 className="w-6 h-6 text-red-400" />
              </div>
              <div className="flex-1">
                <h3 className="text-xl font-bold text-white mb-2">Backup löschen?</h3>
                <p className="text-slate-300 text-sm">
                  Möchten Sie dieses Backup wirklich löschen?
                </p>
              </div>
            </div>

            <div className="bg-slate-700/30 rounded-lg p-4 mb-6 space-y-2">
              <div>
                <span className="text-xs text-slate-400">Backup:</span>
                <p className="text-sm text-white font-medium">{deleteDialog.backup.filename}</p>
              </div>
              <div>
                <span className="text-xs text-slate-400">Erstellt:</span>
                <p className="text-sm text-white">{deleteDialog.backup.created_at}</p>
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => setDeleteDialog({ show: false, backup: null })}
                className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
              >
                Abbrechen
              </button>
              <button
                onClick={handleDeleteBackup}
                className="flex-1 px-4 py-3 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors flex items-center justify-center gap-2"
              >
                <Trash2 className="w-4 h-4" />
                Löschen
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
