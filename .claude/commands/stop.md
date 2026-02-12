# Stoppe die App und gebe alle Ports frei

Beende alle laufenden Dev-Prozesse und gebe Port 1420 frei:

1. **Finde Prozesse auf Port 1420:**
   ```bash
   netstat -ano | findstr ":1420"
   ```
   - Extrahiere alle PIDs die Port 1420 verwenden

2. **Beende alle Prozesse:**
   - Für jeden gefundenen PID: `taskkill //PID <PID> //F`
   - Beende Tauri App: `tasklist | grep -i "dpolg-booking-modern.exe"` → `taskkill //PID <PID> //F`
   - Beende auch alle node.exe Prozesse im Projekt-Verzeichnis

3. **Stoppe laufende Background-Tasks:**
   - Prüfe aktive Tasks mit `TaskOutput`
   - Stoppe alle Tauri-dev-bezogenen Tasks mit `TaskStop`

4. **Verifiziere:**
   ```bash
   netstat -ano | findstr ":1420"
   ```
   - Sollte leer sein (keine Ausgabe)

5. **Bestätige dem User:**
   - ✅ Port 1420 freigegeben
   - ✅ Alle Dev-Prozesse gestoppt
   - ✅ Background-Tasks beendet

**Hinweis:** Dieser Command beendet nur Dev-Prozesse, keine System-Prozesse.
