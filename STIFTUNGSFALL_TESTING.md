# 🧪 Stiftungsfall Feature - Test-Checkliste

## Übersicht
Das Stiftungsfall-Feature ermöglicht es, Buchungen als "Stiftungsfall" zu markieren. Diese Buchungen erhalten:
- ✅ **Keine automatischen Emails** (aber manuelle PDF-Generierung möglich)
- ✅ **Orange/Amber Farbe** im TapeChart (statt grün/blau)
- ✅ **Visuelles Badge** in der BookingList
- ✅ **Eigener Filter** in der BookingList

---

## 🎯 Test-Szenarien

### ✅ Test 1: Neue Buchung als Stiftungsfall erstellen

**Schritte:**
1. Navigation: "Buchungen" → Button "Neue Buchung"
2. Formular ausfüllen:
   - Zimmer auswählen
   - Gast auswählen
   - Check-in & Check-out Datum
   - Anzahl Gäste
3. **WICHTIG**: Checkbox "Stiftungsfall" aktivieren
   - ⚠️ Amber Warning-Box sollte erscheinen
4. Button "Speichern" klicken

**Erwartetes Verhalten:**
- ✅ Buchung wird gespeichert
- ✅ **KEIN Email-Dialog** erscheint (normalerweise würde EmailSelectionDialog öffnen)
- ✅ Direkter Rücksprung zur BookingList
- ✅ Success-Toast: "Buchung erfolgreich erstellt"

**Fehlverhalten:**
- ❌ Email-Dialog öffnet sich → BUG!
- ❌ Buchung wird nicht gespeichert → BUG!

---

### ✅ Test 2: Stiftungsfall im TapeChart (Orange Farbe)

**Schritte:**
1. Navigation: "TapeChart" öffnen
2. Stiftungsfall-Buchung im Chart suchen
3. Farbe des Buchungsbalkens prüfen

**Erwartetes Verhalten:**
- ✅ Balken hat **Orange/Amber Gradient** (from-amber-500 to-orange-600)
- ✅ Nicht grün (bestätigt) oder blau (eingecheckt)
- ✅ Hover zeigt Gast-Name + Reservierungsnummer

**Fehlverhalten:**
- ❌ Balken ist grün/blau statt orange → BUG!
- ❌ Balken fehlt komplett → BUG!

---

### ✅ Test 3: BookingList - Badge & Filter

**Schritte:**
1. Navigation: "Buchungen" Liste öffnen
2. Spalte "Stiftungsfall" suchen (zwischen "Rechnung" und "Aktionen")
3. Stiftungsfall-Buchung finden

**Erwartetes Verhalten:**
- ✅ **Amber Badge** mit Text "Stiftungsfall" und AlertCircle Icon
- ✅ Badge-Design: Amber Gradient mit Border
- ✅ Normale Buchungen haben **kein Badge** (Zelle leer)

**Filter-Test:**
4. Dropdown "Alle Buchungen" öffnen (oben rechts)
5. Option "Nur Stiftungsfälle" wählen

**Erwartetes Verhalten:**
- ✅ Liste zeigt **NUR Stiftungsfall-Buchungen**
- ✅ Normale Buchungen verschwinden

6. Option "Nur Normale" wählen

**Erwartetes Verhalten:**
- ✅ Liste zeigt **NUR normale Buchungen**
- ✅ Stiftungsfall-Buchungen verschwinden

7. Option "Alle Buchungen" wählen

**Erwartetes Verhalten:**
- ✅ Liste zeigt wieder **alle Buchungen**

**Fehlverhalten:**
- ❌ Badge fehlt → BUG!
- ❌ Filter funktioniert nicht → BUG!
- ❌ Spalte "Stiftungsfall" fehlt → BUG!

---

### ✅ Test 4: BookingDetails - Email-Buttons disabled

**Schritte:**
1. Stiftungsfall-Buchung in Liste finden
2. Button "Details" klicken
3. Email-Buttons prüfen (rechte Sidebar):
   - "Bestätigung"
   - "Erinnerung"
   - "Rechnung"

**Erwartetes Verhalten:**
- ✅ **Alle 3 Buttons sind disabled** (grau, nicht klickbar)
- ✅ Hover zeigt Tooltip: "Stiftungsfall: Kein automatischer Email-Versand"
- ✅ **Amber Warning-Box** am Anfang des Details-Dialogs:
   - Header: "🟠 Stiftungsfall"
   - Text: "Diese Buchung ist als Stiftungsfall markiert..."

**PDF-Test:**
4. Unter "Rechnung" → Button "PDF erstellen"

**Erwartetes Verhalten:**
- ✅ PDF-Generierung **funktioniert** (trotz disabled Email-Button)
- ✅ PDF wird heruntergeladen

**Fehlverhalten:**
- ❌ Email-Buttons sind klickbar → BUG!
- ❌ Kein Tooltip → BUG!
- ❌ Warning-Box fehlt → BUG!
- ❌ PDF-Generierung schlägt fehl → BUG!

---

### ✅ Test 5: Buchung bearbeiten (Toggle Stiftungsfall)

**Schritte:**
1. Normale Buchung (NICHT Stiftungsfall) auswählen
2. Button "Bearbeiten" klicken
3. **Checkbox "Stiftungsfall" aktivieren**
4. Speichern

**Erwartetes Verhalten:**
- ✅ Buchung wird aktualisiert
- ✅ TapeChart: Balken wird **orange**
- ✅ BookingList: **Badge erscheint**
- ✅ BookingDetails: Email-Buttons werden **disabled**

**Rück-Toggle:**
5. Buchung erneut bearbeiten
6. **Checkbox "Stiftungsfall" deaktivieren**
7. Speichern

**Erwartetes Verhalten:**
- ✅ Buchung wird aktualisiert
- ✅ TapeChart: Balken wird **grün/blau** (Status-Farbe)
- ✅ BookingList: **Badge verschwindet**
- ✅ BookingDetails: Email-Buttons werden **aktiviert**

**Fehlverhalten:**
- ❌ Update schlägt fehl → BUG!
- ❌ Farbe ändert sich nicht → BUG!
- ❌ Badge bleibt/verschwindet nicht → BUG!

---

### ✅ Test 6: Edge Cases

#### 6.1 Email-Dialog bei normaler Buchung
**Schritte:**
1. Neue Buchung **OHNE** Stiftungsfall-Checkbox erstellen
2. Speichern

**Erwartetes Verhalten:**
- ✅ EmailSelectionDialog **öffnet sich**
- ✅ Checkboxen für Email-Optionen anzeigbar

#### 6.2 Stiftungsfall in Kombination mit Status
**Schritte:**
1. Stiftungsfall-Buchung erstellen mit Status "Bestätigt"
2. TapeChart prüfen

**Erwartetes Verhalten:**
- ✅ Farbe ist **ORANGE** (nicht grün)
- ✅ Stiftungsfall hat **Priorität** über Status-Farbe

#### 6.3 Sortierung & Suche
**Schritte:**
1. BookingList: Nach Stiftungsfall-Reservierungsnummer suchen
2. Status-Filter auf "Bestätigt" setzen + Stiftungsfall-Filter auf "Nur Stiftungsfälle"

**Erwartetes Verhalten:**
- ✅ Suche funktioniert
- ✅ Beide Filter kombinierbar
- ✅ Stiftungsfall-Buchung wird gefunden (wenn Status passt)

---

## 🐛 Bekannte Einschränkungen

### ⚠️ Keine automatischen Emails
- Stiftungsfälle erhalten **NIE** automatische Emails
- PDFs können aber **manuell** erstellt und heruntergeladen werden

### ⚠️ Preis-Berechnung
- Stiftungsfälle haben normale Preisberechnung
- Falls spezielle Preis-Logik gewünscht: Separat implementieren

---

## 📊 Test-Ergebnis Tracking

### Durchgeführt:
- [ ] Test 1: Neue Buchung als Stiftungsfall
- [ ] Test 2: TapeChart Orange Farbe
- [ ] Test 3: BookingList Badge & Filter
- [ ] Test 4: BookingDetails Email-Buttons disabled
- [ ] Test 5: Buchung bearbeiten (Toggle)
- [ ] Test 6.1: Email-Dialog bei normaler Buchung
- [ ] Test 6.2: Stiftungsfall + Status-Kombination
- [ ] Test 6.3: Sortierung & Suche

### Bugs gefunden:
*(Hier Bugs dokumentieren)*

---

## 🔍 Code-Locations (für Debugging)

### BookingDialog
- **Datei**: `src/components/BookingManagement/BookingDialog.tsx`
- **Checkbox**: Zeile ~250-270
- **Email-Logic**: Zeile 711-725
- **Payload**: Zeile 639 (Update) & 670 (Create)

### TapeChart
- **Datei**: `src/components/TapeChart.tsx`
- **Farb-Definition**: Zeile 92-97 (STATUS_COLORS.stiftungsfall)
- **Farb-Auswahl**: Zeile 164-167 (Priority-Check)

### BookingList
- **Datei**: `src/components/BookingManagement/BookingList.tsx`
- **Filter-State**: Zeile 63
- **Filter-Logic**: Zeile 225-227
- **Filter-Dropdown**: Zeile 382-391
- **Badge**: Zeile 585-592

### BookingDetails
- **Datei**: `src/components/BookingManagement/BookingDetails.tsx`
- **Warning-Box**: Zeile 410-427
- **Email-Buttons disabled**: Zeile 758-786

---

## ✅ Abnahme-Kriterien

Feature ist **DONE** wenn:
1. ✅ Alle 8 Tests bestanden
2. ✅ Keine kritischen Bugs
3. ✅ Orange Farbe konsistent im TapeChart
4. ✅ Badge korrekt angezeigt
5. ✅ Filter funktioniert
6. ✅ Email-Buttons disabled bei Stiftungsfall
7. ✅ Toggle zwischen Stiftungsfall/Normal funktioniert

---

**Letzte Aktualisierung**: 2025-10-10
**Status**: ✅ Implementierung abgeschlossen, Testing ausstehend
