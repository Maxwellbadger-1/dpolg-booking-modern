# ⚠️ KRITISCHE FEATURES CHECKLIST

**VOR JEDEM RELEASE DIESE FEATURES MANUELL TESTEN!**

---

## 🎯 CORE FEATURES (MUST WORK!)

### Tapechart
- [ ] **Drag & Drop** - Buchungen verschieben funktioniert
- [ ] **Resize** - Buchungen verlängern/verkürzen funktioniert
- [ ] **Create by Drag** - Neue Buchung durch Ziehen erstellen
- [ ] **Heute-Spalte** - Grüne Markierung sichtbar
- [ ] **Monatswechsel** - Navigation zwischen Monaten funktioniert
- [ ] **Zoom/Density** - Compact/Comfortable/Spacious Modi wechseln

### Buchungsverwaltung
- [ ] **Neue Buchung** - Dialog öffnet, alle Felder funktionieren
- [ ] **Buchung bearbeiten** - Sidebar öffnet, Änderungen speichern
- [ ] **Buchung löschen** - Löschen mit Bestätigung funktioniert
- [ ] **Status ändern** - Pending/Confirmed/Checked-in/Checked-out
- [ ] **Email senden** - Bestätigungs-Email verschicken
- [ ] **PDF exportieren** - Buchungsbestätigung als PDF

### Gästeverwaltung
- [ ] **Gast erstellen** - Neuen Gast anlegen
- [ ] **Gast bearbeiten** - Gast-Daten ändern
- [ ] **Gast suchen** - Suche funktioniert
- [ ] **Gast löschen** - Mit Warnung wenn Buchungen vorhanden

### Zimmerverwaltung
- [ ] **Zimmer erstellen** - Neues Zimmer anlegen
- [ ] **Zimmer bearbeiten** - Zimmerdaten ändern
- [ ] **Zimmerstatus** - Verfügbar/Blockiert wechseln

---

## 🔧 IMPORTANT FEATURES

### Einstellungen
- [ ] **Preise** - Preise anpassen und speichern
- [ ] **Email-Konfiguration** - SMTP-Einstellungen testen
- [ ] **Templates** - Email-Templates bearbeiten
- [ ] **Backup** - Manuelles Backup erstellen
- [ ] **Software-Update** - Update-Check funktioniert

### Putzplan Sync
- [ ] **Mobile App iframe** - Vorschau lädt korrekt (Desktop & Mobile)
- [ ] **3 Monate synchronisieren** - Sync ohne Fehler
- [ ] **Putzplan bereinigen** - Cleanup funktioniert
- [ ] **PDF Export** - Timeline PDF erstellen
- [ ] **Stats anzeigen** - Heute/Morgen/Woche/Total korrekt

### Erinnerungen
- [ ] **Dropdown öffnet** - Glocken-Icon zeigt Erinnerungen
- [ ] **Erinnerung erstellen** - Neue Erinnerung anlegen
- [ ] **Als erledigt markieren** - Status-Toggle funktioniert
- [ ] **Alle anzeigen** - Erinnerungen-View öffnet

---

## 🎨 UI/UX FEATURES

### Design System
- [ ] **Scrollbars** - Modern, 8px, overlay-style
- [ ] **Dropdowns** - Nicht von Tapechart überdeckt (z-index)
- [ ] **Modals** - Öffnen/Schließen smooth
- [ ] **Loading States** - Spinner während async Operations
- [ ] **Toast Notifications** - Erfolgs-/Fehler-Meldungen

### Responsiveness
- [ ] **Tabellen skalieren** - Höhe passt sich an Viewport an
- [ ] **Logo Größe** - Richtig dimensioniert
- [ ] **Mobile Preview** - Tapechart funktioniert auch klein

---

## 🌐 MOBILE APP (Vercel)

### dpolg-cleaning-mobile.vercel.app
- [ ] **App lädt** - Keine Fehler in Console
- [ ] **Passwort funktioniert** - `putzplan2025` erlaubt Zugriff
- [ ] **Emojis anzeigen** - 🐕 Hund, andere Service-Icons sichtbar
- [ ] **Tasks filtern** - Heute/Woche/Timeline Filter
- [ ] **Task als erledigt** - Toggle funktioniert
- [ ] **Auto-Refresh** - Alle 5 Minuten neue Daten

---

## 🔒 CRITICAL BUG CHECKS

### Bekannte Probleme (IMMER prüfen!)
- [ ] **JSX Syntax** - Keine Adjacent Elements ohne Wrapper
- [ ] **Tauri Invoke** - IMMER camelCase im Frontend
- [ ] **z-index** - Dropdowns über Tapechart (z-[100])
- [ ] **iframe CSP** - Vercel-Domain erlaubt in tauri.conf.json
- [ ] **Emoji Spalten** - emojis_start/emojis_end in Turso
- [ ] **Date Formats** - DD.MM.YYYY (nicht MM/DD/YYYY)

---

## ✅ VOR RELEASE AUSFÜHREN

```bash
# 1. TypeScript Check
npm run type-check

# 2. Build Check
npm run build

# 3. Tauri Build (Test)
npm run tauri build

# 4. Git Status Clean
git status

# 5. Version Bump
# package.json, tauri.conf.json, latest.json

# 6. Commit & Tag
git add .
git commit -m "chore: Release v1.X.X"
git tag v1.X.X
git push && git push --tags
```

---

## 📝 NACH RELEASE

- [ ] **GitHub Release** - Erstellen mit Changelog
- [ ] **Vercel Deploy** - Mobile App neu deployen (wenn geändert)
- [ ] **Update testen** - Von vorheriger Version zu neuer
- [ ] **Changelog updaten** - CHANGELOG.md aktualisieren
- [ ] **ROADMAP markieren** - Erledigte Tasks abhaken

---

**Zuletzt getestet:** _[Datum eintragen]_
**Release Version:** _[Version eintragen]_
**Tester:** _[Name eintragen]_
