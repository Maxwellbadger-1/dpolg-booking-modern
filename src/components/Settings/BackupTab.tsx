import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Save, RefreshCw, FolderOpen, RotateCcw, Trash2, CheckCircle, AlertCircle, HardDrive, AlertTriangle } from 'lucide-react';

// Korrekte Interfaces passend zum Rust Backend
interface BackupEntry {
  name: string;
  size: number;
  createdAt: string;
}

interface BackupSettings {
  autoBackupEnabled: boolean;
  backupIntervalDays: number;
  backupPath: string;
}

export default function BackupTab() {
  const [settings, setSettings] = useState<BackupSettings>({
    autoBackupEnabled: false,
    backupIntervalDays: 7,
    backupPath: 'backups',
  });
  const [backups, setBackups] = useState<BackupEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [creating, setCreating] = useState(false);
  const [success, setSuccess] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [restoreDialog, setRestoreDialog] = useState<{ show: boolean; backup: BackupEntry | null }>({ show: false, backup: null });
  const [deleteDialog, setDeleteDialog] = useState<{ show: boolean; backup: BackupEntry | null }>({ show: false, backup: null });
  const [restoring, setRestoring] = useState(false);

  useEffect(() => {
    loadSettings();
    loadBackups();
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
      const result = await invoke<BackupEntry[]>('list_backups_command');
      setBackups(result);
    } catch (err) {
      console.error('Fehler beim Laden der Backups:', err);
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
      const filename = await invoke<string>('create_backup_command');
      setSuccess(`Backup erfolgreich erstellt: ${filename}`);
      loadBackups();
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
      await invoke('restore_backup_command', { backupName: restoreDialog.backup.name });
      setSuccess('Backup erfolgreich wiederhergestellt! Bitte starten Sie die Anwendung neu.');
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
      await invoke('delete_backup_command', { backupName: deleteDialog.backup.name });
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

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
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
                Erstellt automatisch Backups in konfigurierten Intervallen
              </p>
            </div>
            <button
              onClick={() => setSettings({ ...settings, autoBackupEnabled: !settings.autoBackupEnabled })}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                settings.autoBackupEnabled ? 'bg-blue-600' : 'bg-slate-600'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  settings.autoBackupEnabled ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          {/* Backup Interval */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-slate-300">
              Backup-Intervall (Tage)
            </label>
            <input
              type="number"
              min="1"
              max="30"
              value={settings.backupIntervalDays}
              onChange={(e) => setSettings({ ...settings, backupIntervalDays: parseInt(e.target.value) || 7 })}
              className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p className="text-xs text-slate-400">
              Legt fest, nach wie vielen Tagen automatisch ein Backup erstellt wird
            </p>
          </div>

          {/* Backup Path */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-slate-300">
              Backup-Pfad
            </label>
            <input
              type="text"
              value={settings.backupPath}
              onChange={(e) => setSettings({ ...settings, backupPath: e.target.value })}
              className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <p className="text-xs text-slate-400">
              Ordner, in dem Backups gespeichert werden (relativ zum App-Ordner)
            </p>
          </div>

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
                key={backup.name}
                className="flex items-center justify-between p-4 bg-slate-700/30 rounded-lg hover:bg-slate-700/50 transition-colors"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <CheckCircle className="w-4 h-4 text-emerald-400" />
                    <p className="text-sm font-medium text-white">{backup.createdAt}</p>
                    <span className="px-2 py-0.5 bg-slate-600 text-xs text-slate-300 rounded">
                      {formatBytes(backup.size)}
                    </span>
                  </div>
                  <p className="text-xs text-slate-400 mt-1 ml-6">{backup.name}</p>
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

      {/* Info Box */}
      <div className="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4">
        <div className="flex gap-3">
          <AlertCircle className="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" />
          <div className="text-sm text-blue-300">
            <p className="font-semibold mb-1">Wichtige Hinweise:</p>
            <ul className="list-disc list-inside space-y-1 text-blue-300/80">
              <li>Backups werden im konfigurierten Ordner gespeichert</li>
              <li>Nach der Wiederherstellung muss die App neu gestartet werden</li>
              <li>Erstellen Sie regelmäßig externe Backups auf USB/Cloud</li>
              <li>pg_dump und psql müssen auf dem System installiert sein</li>
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
                <p className="text-sm text-white font-medium">{restoreDialog.backup.name}</p>
              </div>
              <div>
                <span className="text-xs text-slate-400">Erstellt:</span>
                <p className="text-sm text-white">{restoreDialog.backup.createdAt}</p>
              </div>
              <div>
                <span className="text-xs text-slate-400">Größe:</span>
                <p className="text-sm text-white">{formatBytes(restoreDialog.backup.size)}</p>
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
                <p className="text-sm text-white font-medium">{deleteDialog.backup.name}</p>
              </div>
              <div>
                <span className="text-xs text-slate-400">Erstellt:</span>
                <p className="text-sm text-white">{deleteDialog.backup.createdAt}</p>
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
