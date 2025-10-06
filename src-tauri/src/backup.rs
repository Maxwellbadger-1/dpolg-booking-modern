use rusqlite::{Connection, Result};
use crate::database::get_db_path;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Local;
use serde::{Serialize, Deserialize};

// ============================================================================
// BACKUP STRUCTURES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupInfo {
    pub filename: String,
    pub path: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub size_formatted: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupSettings {
    pub auto_backup_enabled: bool,
    pub backup_interval: String, // "daily", "weekly", "on_startup"
    pub max_backups: i32,
    pub last_backup_at: Option<String>,
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            auto_backup_enabled: true,
            backup_interval: "on_startup".to_string(),
            max_backups: 10,
            last_backup_at: None,
        }
    }
}

// ============================================================================
// BACKUP DIRECTORY MANAGEMENT
// ============================================================================

/// Gibt den Backup-Ordner-Pfad zurück
pub fn get_backup_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Fehler beim Ermitteln des App-Datenordners: {}", e))?;

    let backup_dir = app_data_dir.join("backups");

    // Backup-Ordner erstellen falls nicht vorhanden
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Fehler beim Erstellen des Backup-Ordners: {}", e))?;
    }

    Ok(backup_dir)
}

// ============================================================================
// BACKUP CREATION
// ============================================================================

/// Erstellt ein Backup der Datenbank
pub fn create_backup(app: &tauri::AppHandle) -> Result<BackupInfo, String> {
    create_backup_internal(app, "manual")
}

/// Erstellt ein Auto-Backup vor kritischen Operationen
pub fn create_auto_backup(app: &tauri::AppHandle, reason: &str) -> Result<BackupInfo, String> {
    create_backup_internal(app, reason)
}

/// Interne Backup-Erstellung (gemeinsame Logik)
fn create_backup_internal(app: &tauri::AppHandle, backup_type: &str) -> Result<BackupInfo, String> {
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  BACKUP CREATION ({})                           │", backup_type.to_uppercase());
    println!("└─────────────────────────────────────────────────────┘");

    // 1. Backup-Ordner ermitteln
    let backup_dir = get_backup_dir(app)?;
    println!("📁 Backup-Ordner: {}", backup_dir.display());

    // 2. Datenbank-Pfad ermitteln
    let db_path = get_db_path();
    println!("💾 Datenbank: {}", db_path.display());

    if !db_path.exists() {
        return Err("Datenbank-Datei nicht gefunden".to_string());
    }

    // 3. Backup-Dateinamen mit Timestamp und Typ generieren
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let backup_filename = if backup_type == "manual" {
        format!("booking_system_backup_{}.db", timestamp)
    } else {
        format!("booking_system_auto_{}_{}.db", backup_type, timestamp)
    };
    let backup_path = backup_dir.join(&backup_filename);

    println!("🔄 Erstelle Backup: {}", backup_filename);

    // 4. Datenbank kopieren
    fs::copy(&db_path, &backup_path)
        .map_err(|e| format!("Fehler beim Kopieren der Datenbank: {}", e))?;

    // 5. Backup-Info erstellen
    let metadata = fs::metadata(&backup_path)
        .map_err(|e| format!("Fehler beim Lesen der Backup-Metadaten: {}", e))?;

    let size_bytes = metadata.len();
    let size_formatted = format_file_size(size_bytes);

    let backup_info = BackupInfo {
        filename: backup_filename,
        path: backup_path.to_string_lossy().to_string(),
        created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        size_bytes,
        size_formatted,
    };

    println!("✅ Backup erfolgreich erstellt: {} ({})", backup_info.filename, backup_info.size_formatted);

    // 6. Alte Backups aufräumen (max 20 für Auto-Backups)
    cleanup_old_backups(app)?;

    // 7. Letztes Backup-Datum speichern
    update_last_backup_time()?;

    println!("└─────────────────────────────────────────────────────┘");

    Ok(backup_info)
}

// ============================================================================
// BACKUP LISTING
// ============================================================================

/// Liste alle verfügbaren Backups
pub fn list_backups(app: &tauri::AppHandle) -> Result<Vec<BackupInfo>, String> {
    let backup_dir = get_backup_dir(app)?;

    if !backup_dir.exists() {
        return Ok(vec![]);
    }

    let mut backups = Vec::new();

    let entries = fs::read_dir(&backup_dir)
        .map_err(|e| format!("Fehler beim Lesen des Backup-Ordners: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Fehler beim Lesen des Eintrags: {}", e))?;
        let path = entry.path();

        // Nur .db Dateien mit "backup" im Namen
        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("db")
            && path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.contains("backup"))
        {
            let metadata = fs::metadata(&path)
                .map_err(|e| format!("Fehler beim Lesen der Metadaten: {}", e))?;

            let filename = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let size_bytes = metadata.len();
            let size_formatted = format_file_size(size_bytes);

            // Datum aus Metadaten extrahieren
            let created_at = metadata.modified()
                .ok()
                .and_then(|time| {
                    use std::time::SystemTime;
                    let datetime: chrono::DateTime<Local> = time.into();
                    Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
                })
                .unwrap_or_else(|| "Unbekannt".to_string());

            backups.push(BackupInfo {
                filename,
                path: path.to_string_lossy().to_string(),
                created_at,
                size_bytes,
                size_formatted,
            });
        }
    }

    // Sortiere nach Datum (neueste zuerst)
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(backups)
}

// ============================================================================
// BACKUP RESTORATION
// ============================================================================

/// Stellt ein Backup wieder her
pub fn restore_backup(backup_path: String) -> Result<String, String> {
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  BACKUP RESTORATION                                 │");
    println!("└─────────────────────────────────────────────────────┘");

    let backup_path = Path::new(&backup_path);

    // 1. Prüfen ob Backup existiert
    if !backup_path.exists() {
        return Err("Backup-Datei nicht gefunden".to_string());
    }

    println!("📁 Backup: {}", backup_path.display());

    // 2. Aktuellen DB-Pfad ermitteln
    let db_path = get_db_path();
    println!("💾 Ziel-Datenbank: {}", db_path.display());

    // 3. Aktuelle Datenbank als Sicherheitskopie umbenennen (falls Wiederherstellung fehlschlägt)
    let temp_backup = db_path.with_extension("db.before_restore");
    if db_path.exists() {
        fs::copy(&db_path, &temp_backup)
            .map_err(|e| format!("Fehler beim Erstellen der Sicherheitskopie: {}", e))?;
        println!("🔒 Sicherheitskopie erstellt");
    }

    // 4. Backup wiederherstellen (überschreibt aktuelle DB)
    match fs::copy(&backup_path, &db_path) {
        Ok(_) => {
            println!("✅ Backup erfolgreich wiederhergestellt");

            // Sicherheitskopie löschen
            if temp_backup.exists() {
                let _ = fs::remove_file(&temp_backup);
            }

            println!("└─────────────────────────────────────────────────────┘");
            Ok("Backup erfolgreich wiederhergestellt. Bitte starten Sie die Anwendung neu.".to_string())
        }
        Err(e) => {
            // Bei Fehler: Sicherheitskopie wiederherstellen
            if temp_backup.exists() {
                let _ = fs::copy(&temp_backup, &db_path);
                let _ = fs::remove_file(&temp_backup);
            }
            Err(format!("Fehler beim Wiederherstellen: {}", e))
        }
    }
}

// ============================================================================
// BACKUP DELETION
// ============================================================================

/// Löscht ein spezifisches Backup
pub fn delete_backup(backup_path: String) -> Result<String, String> {
    let path = Path::new(&backup_path);

    if !path.exists() {
        return Err("Backup nicht gefunden".to_string());
    }

    // Sicherheitsprüfung: Nur Dateien im Backup-Ordner mit "backup" im Namen
    let filename = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if !filename.contains("backup") {
        return Err("Sicherheitsfehler: Datei ist kein Backup".to_string());
    }

    fs::remove_file(path)
        .map_err(|e| format!("Fehler beim Löschen: {}", e))?;

    Ok("Backup erfolgreich gelöscht".to_string())
}

// ============================================================================
// BACKUP CLEANUP
// ============================================================================

/// Löscht alte Backups (behält nur max_backups neueste)
fn cleanup_old_backups(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = get_backup_settings()?;
    let backups = list_backups(app)?;

    if backups.len() > settings.max_backups as usize {
        println!("🧹 Räume alte Backups auf (max: {})", settings.max_backups);

        // Lösche älteste Backups
        for backup in backups.iter().skip(settings.max_backups as usize) {
            println!("   🗑️  Lösche: {}", backup.filename);
            let _ = fs::remove_file(&backup.path);
        }
    }

    Ok(())
}

// ============================================================================
// BACKUP SETTINGS
// ============================================================================

/// Lädt Backup-Einstellungen
pub fn get_backup_settings() -> Result<BackupSettings, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    // Prüfe ob backup_settings Tabelle existiert
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='backup_settings'",
        [],
        |row| row.get(0),
    ).unwrap_or(false);

    if !table_exists {
        // Tabelle erstellen
        conn.execute(
            "CREATE TABLE backup_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                auto_backup_enabled BOOLEAN NOT NULL DEFAULT 1,
                backup_interval TEXT NOT NULL DEFAULT 'on_startup',
                max_backups INTEGER NOT NULL DEFAULT 10,
                last_backup_at TEXT
            )",
            [],
        ).map_err(|e| format!("Fehler beim Erstellen der Tabelle: {}", e))?;

        // Default-Einstellungen speichern
        let default = BackupSettings::default();
        conn.execute(
            "INSERT INTO backup_settings (id, auto_backup_enabled, backup_interval, max_backups) VALUES (1, ?1, ?2, ?3)",
            [
                &default.auto_backup_enabled as &dyn rusqlite::ToSql,
                &default.backup_interval,
                &default.max_backups,
            ],
        ).map_err(|e| format!("Fehler beim Einfügen der Default-Einstellungen: {}", e))?;

        return Ok(default);
    }

    // Einstellungen laden
    let settings = conn.query_row(
        "SELECT auto_backup_enabled, backup_interval, max_backups, last_backup_at FROM backup_settings WHERE id = 1",
        [],
        |row| {
            Ok(BackupSettings {
                auto_backup_enabled: row.get(0)?,
                backup_interval: row.get(1)?,
                max_backups: row.get(2)?,
                last_backup_at: row.get(3)?,
            })
        },
    ).map_err(|e| format!("Fehler beim Laden: {}", e))?;

    Ok(settings)
}

/// Speichert Backup-Einstellungen
pub fn save_backup_settings(settings: BackupSettings) -> Result<BackupSettings, String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    conn.execute(
        "UPDATE backup_settings SET
            auto_backup_enabled = ?1,
            backup_interval = ?2,
            max_backups = ?3
         WHERE id = 1",
        [
            &settings.auto_backup_enabled as &dyn rusqlite::ToSql,
            &settings.backup_interval,
            &settings.max_backups,
        ],
    ).map_err(|e| format!("Fehler beim Speichern: {}", e))?;

    Ok(settings)
}

/// Aktualisiert das "Letztes Backup"-Datum
fn update_last_backup_time() -> Result<(), String> {
    let conn = Connection::open(get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "UPDATE backup_settings SET last_backup_at = ?1 WHERE id = 1",
        [&now],
    ).map_err(|e| format!("Fehler beim Update: {}", e))?;

    Ok(())
}

// ============================================================================
// HELPERS
// ============================================================================

/// Formatiert Dateigröße in menschenlesbares Format
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} Bytes", bytes)
    }
}
