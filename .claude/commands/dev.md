Starte die Desktop-App im Dev-Modus:

1. Prüfe ob Port 1420 belegt ist: `netstat -ano | findstr :1420`
   - Falls belegt: Prozess beenden mit `powershell -Command "Stop-Process -Id <PID> -Force"`
2. Führe `npx tauri dev` im Hintergrund aus (run_in_background: true, timeout: 600000)
3. Warte 10 Sekunden für die Kompilierung
4. Prüfe die Ausgabe und melde dem User den Status (Vite URL + Rust Kompilierung)
