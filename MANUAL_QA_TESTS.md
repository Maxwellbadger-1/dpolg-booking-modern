# 🧪 Manuelle QA Tests - Produktionsfreigabe
**Datum:** 2025-10-20
**Version:** Pre-Production
**Durchgeführt von:** _____________

---

## ✅ Checkliste - Basis-Funktionalität

### 1. Buchung erstellen ✅
- [x] Neuen Gast anlegen
  - [x] Vorname, Nachname, Email eingeben
  - [x] Adresse ausfüllen
  - [x] Speichern erfolgreich
- [x] Buchung anlegen
  - [x] Gast auswählen
  - [x] Zimmer auswählen
  - [x] Check-in Datum wählen
  - [x] Check-out Datum wählen
  - [x] **DPolG-Mitglied Checkbox aktivieren**
  - [x] Services hinzufügen (optional)
  - [x] **Preis korrekt berechnet (mit 15% Rabatt auf Zimmerpreis bei Mitgliedern)?**
  - [x] Speichern erfolgreich

**Testergebnis:** ✅ BESTANDEN (Buchung 78, 79 erstellt - DPolG-Rabatt korrekt)

**Notizen:**
```
Erwartetes Verhalten:
- Nebensaison: 3 Nächte à 45€ = 135€
- DPolG-Rabatt: 15% von 135€ = 20,25€
- Endreinigung: 20€
- Gesamtpreis: 134,75€
```

---

### 2. Buchung bearbeiten ✅
- [x] Existierende Buchung aus Liste öffnen
- [x] Check-in/Check-out Datum ändern
- [x] Services hinzufügen/entfernen
- [x] DPolG-Mitglied Status ändern
- [x] Preis wird live aktualisiert
- [x] Speichern erfolgreich
- [x] Änderungen im TapeChart sichtbar (Optimistic Update)

**Testergebnis:** ✅ BESTANDEN - Optimistic Update funktioniert, Sidebar schließt automatisch

---

### 3. Buchung löschen ✅
- [x] Buchung auswählen
- [x] Löschen-Button klicken
- [x] Bestätigungsdialog erscheint
- [x] Löschen bestätigen
- [x] Buchung verschwindet aus TapeChart (Optimistic Update)
- [x] Buchung verschwindet aus Buchungsliste

**Testergebnis:** ✅ BESTANDEN - Löschen funktioniert mit Optimistic Update

---

### 4. TapeChart Visualisierung ✅
- [x] TapeChart wird korrekt geladen
- [x] Alle Buchungen werden angezeigt
- [x] Farben korrekt:
  - [x] Blau = Belegt
  - [x] Grün = Check-in
  - [x] Rot = Check-out
- [x] **Drag & Drop funktioniert**
  - [x] Buchung verschieben (Datum ändern)
  - [x] Buchung auf anderes Zimmer verschieben
  - [x] Buchung verlängern/verkürzen (Resize)
- [x] Tooltips zeigen korrekte Informationen

**Testergebnis:** ✅ BESTANDEN - TapeChart Visualisierung und Drag&Drop funktionieren einwandfrei

---

### 5. PDF Export - Rechnung ✅
- [x] Buchung auswählen
- [x] "Rechnung generieren" klicken
- [x] PDF wird erstellt und angezeigt
- [x] PDF-Inhalt korrekt:
  - [x] Gastnamen
  - [x] Zimmername
  - [x] Check-in/Check-out Datum
  - [x] Anzahl Nächte
  - [x] Preisberechnung:
    - [x] Grundpreis (Nächte × Zimmerpreis)
    - [x] Services aufgelistet
    - [x] DPolG-Rabatt (falls Mitglied) - **WIRD JETZT KORREKT ABGEZOGEN**
    - [x] Endreinigung
    - [x] Gesamtpreis korrekt
  - [x] Rechnungsnummer
  - [x] Datum

**Testergebnis:** ✅ BESTANDEN - Bug behoben: Endbetrag subtrahiert jetzt korrekt den DPolG-Rabatt

**Beispiel-Berechnung:**
```
Zwischensumme:    212,00 €
MwSt. 7%:           7,14 €
MwSt. 19%:         20,90 €
DPolG-Rabatt:     -15,30 €
========================
Endbetrag:        224,74 € ✅
```

---

### 6. PDF Export - Buchungsbestätigung ⏭️
- [ ] Buchung auswählen
- [ ] "Bestätigung generieren" klicken
- [ ] PDF wird erstellt und angezeigt
- [ ] PDF-Inhalt korrekt:
  - [ ] Gastnamen
  - [ ] Zimmername
  - [ ] Check-in/Check-out Datum
  - [ ] Begleitpersonen (falls vorhanden)
  - [ ] Services aufgelistet
  - [ ] Gesamtpreis

**Testergebnis:** ⏭️ ÜBERSPRUNGEN - Feature nicht implementiert

---

### 7. PDF Export - Putzplan ✅
- [x] Menü: "Putzplan" öffnen
- [x] Monat auswählen
- [x] "Putzplan exportieren" klicken
- [x] PDF wird erstellt und im Programmordner gespeichert
- [x] PDF öffnet sich automatisch
- [x] PDF-Inhalt korrekt:
  - [x] Zeitraum im Header angezeigt
  - [x] Statistiken (Zimmer gesamt, Aktive Buchungen, Abreisen)
  - [x] TapeChart Visualisierung:
    - [x] Alle Zimmer aufgelistet
    - [x] Check-in Tage: Blau mit Emojis (nur Emojis, kein Name!)
    - [x] Belegte Tage: Blau mit Gastname + **Personenanzahl am ersten Tag**
    - [x] Check-out Tage: Rot mit Emojis (nur Emojis, kein "ABREISE ✓"!)
    - [x] Emojis groß und erkennbar (16px)
    - [x] Gastnamen passen vollständig in Zellen (Zeilenumbruch)
  - [x] Legende vorhanden

**Testergebnis:** ✅ BESTANDEN

**Putzplan-Fixes (2025-10-21):**
- ✅ Blaue Tage zwischen Check-in und Check-out werden angezeigt
- ✅ Zimmer 4 wird korrekt angezeigt (auch wenn Check-in/out außerhalb Monat)
- ✅ Nur Emojis in Check-in/out Zellen (kein Text mehr)
- ✅ Emojis 16px groß
- ✅ **Personenanzahl wird IMMER am ersten Tag angezeigt** (auch ohne Services)
- ✅ **Buchungen bleiben im PDF nach Checkout-Änderung** (sync_affected_dates Fix)
- ✅ **Mobile App: Check-IN Tasks nur bei Services** (Turso-Filter)

---

### 8. Preisberechnung - Hauptsaison vs Nebensaison ✅
- [x] Buchung in **Nebensaison** erstellen (Jan-Mai, Sep-Dez)
  - [x] Nebensaison-Preis wird verwendet
- [x] Buchung in **Hauptsaison** erstellen (Jun-Aug)
  - [x] Hauptsaison-Preis wird verwendet
- [x] Preiseinstellungen öffnen
  - [x] Hauptsaison Start/Ende konfigurieren
  - [x] Hauptsaison aktivieren/deaktivieren
  - [x] Änderungen werden gespeichert
  - [x] Neue Buchungen verwenden aktualisierte Settings

**Testergebnis:** ✅ BESTANDEN - Saisonale Preisberechnung funktioniert korrekt

---

### 9. DPolG-Rabatt Berechnung ✅
- [x] **Rabatt wird korrekt angewendet:**
  - [x] Mitglied-Checkbox aktiviert
  - [x] 15% Rabatt auf **Zimmerpreis** (NICHT Gesamtpreis!)
  - [x] Services werden NICHT rabattiert
  - [x] Endreinigung wird NICHT rabattiert
- [x] **Rabatt wird in Rechnung angezeigt:**
  - [x] Position "DPolG-Mitglieder Rabatt (15%)" vorhanden
  - [x] Rabattbetrag korrekt
- [x] **Rabatt-Konfiguration:**
  - [x] Preiseinstellungen → DPolG-Rabatt aktivieren/deaktivieren
  - [x] Rabatt-Prozentsatz änderbar
  - [x] Rabatt-Basis: "Nur Zimmerpreis" ausgewählt

**Testergebnis:** ✅ BESTANDEN - Bug behoben: Rabatt persistiert jetzt bei Service-Änderungen

**Erwartetes Beispiel:**
```
Zimmerpreis: 3 Nächte × 45€ = 135,00€
Endreinigung:                  20,00€
Services (Frühstück):          15,00€
-----------------------------------
Zwischensumme:                170,00€
DPolG-Rabatt (15% von 135€): - 20,25€
===================================
Gesamtpreis:                  149,75€
```

---

### 10. Gästeverwaltung ✅
- [x] Gästeliste öffnen
- [x] Gast bearbeiten
  - [x] Name ändern
  - [x] Adresse ändern
  - [x] Email ändern
  - [x] Speichern erfolgreich
- [x] Gast suchen/filtern
- [x] Gast löschen (falls keine Buchungen vorhanden)

**Testergebnis:** ✅ BESTANDEN - Gästeverwaltung funktioniert einwandfrei

---

### 11. Zimmer-Verwaltung ✅
- [x] Zimmerliste öffnen
- [x] Zimmer bearbeiten
  - [x] Nebensaison-Preis ändern
  - [x] Hauptsaison-Preis ändern
  - [x] Endreinigung-Preis ändern
  - [x] Speichern erfolgreich
- [x] Neues Zimmer anlegen
- [x] Zimmer deaktivieren (falls keine aktiven Buchungen)

**Testergebnis:** ✅ BESTANDEN - Zimmerverwaltung funktioniert einwandfrei

---

### 12. Email-Versand (optional)
- [ ] Email-Einstellungen konfiguriert
- [ ] Rechnung per Email versenden
  - [ ] Email-Dialog öffnet sich
  - [ ] Empfänger korrekt vorausgefüllt
  - [ ] Betreff korrekt
  - [ ] PDF als Anhang vorhanden
  - [ ] Email erfolgreich versendet
- [ ] Bestätigung per Email versenden

---

### 13. Performance & Stabilität ✅
- [x] App startet in < 2 Sekunden
- [x] Keine Console-Errors im Browser DevTools
- [x] Keine Memory Leaks (länger als 5 Minuten verwenden)
- [x] TapeChart scrollt smooth
- [x] Drag & Drop ist flüssig (keine Verzögerung)
- [x] PDF-Generierung < 3 Sekunden

**Testergebnis:** ✅ BESTANDEN - Performance-Optimierungen erfolgreich (Browser-Pool für PDF-Generierung)

**Performance-Fixes (2025-10-21):**
- ✅ **Browser-Pool implementiert:** Chrome-Instanz wird wiederverwendet
  - Erste Rechnung: ~15-20 Sek (Chrome-Start)
  - Weitere Rechnungen: < 2 Sek ⚡
- ✅ **MwSt.-Berechnung korrigiert:** Brutto-Preise (MwSt. bereits enthalten)
- ✅ **Rechnung überarbeitet:** Korrekte Kontaktdaten, 15% Rabatt-Hinweis

---

### 14. Edge Cases & Error Handling
- [ ] Ungültige Daten eingeben
  - [ ] Check-out vor Check-in → Fehlermeldung
  - [ ] Überlappende Buchungen → Fehlermeldung
  - [ ] Leere Pflichtfelder → Validation
- [ ] Sehr lange Namen (>100 Zeichen)
- [ ] Sonderzeichen in Namen (ä, ö, ü, ß)
- [ ] 0 Nächte → Fehlermeldung
- [ ] Negative Preise → Validation

---

## 🐛 Gefundene Bugs

### Bug 1: DPolG-Rabatt wird nicht in Buchungsdetails/Rechnung angezeigt ⚠️
**Beschreibung:** User meldet "dpolg mitglied wird nicht mehr mit einberechnet - während der buchung wirds mit einberechnet aber wenn ich auf buchungsdetails gehe wirds in der preisaufschlüsselung nicht angezeigt und auch in der rechnung nicht mehr"
**Status:** ✅ BEHOBEN (Commit: 67beb24)
**Priorität:** ~~🔴 CRITICAL~~ → ✅ BEHOBEN
**Schritte zur Reproduktion:**
1. Neue Buchung mit DPolG-Mitglied erstellen
2. Rabatt wird initial berechnet und angezeigt
3. Buchung speichern
4. Buchungsdetails öffnen → Rabatt fehlt in Preisaufschlüsselung
5. Rechnung generieren → Rabatt fehlt im PDF

**Root Cause:** `ist_dpolg_mitglied` Feld existierte nicht in der Datenbank. Rabatt wurde nur in-memory berechnet aber nie persistiert.

**Lösung:**
- Datenbank-Schema erweitert: `ist_dpolg_mitglied` Spalte zu bookings-Tabelle hinzugefügt
- Backend aktualisiert: Alle CRUD-Operationen speichern/lesen jetzt den Member-Status
- Frontend aktualisiert: BookingSidebar übergibt `istDpolgMitglied` von Gast-Daten
- **Behoben in Commit:** `67beb24` - fix(booking): Persist DPolG member status to database

---

### Bug 2: DPolG-Rabatt wird auf 0€ gesetzt beim Hinzufügen/Entfernen von Services ⚠️
**Beschreibung:** Wenn ein Service zu einer Buchung mit DPolG-Mitglied hinzugefügt oder entfernt wird, wird der DPolG-Rabatt (15%) auf 0€ zurückgesetzt.
**Status:** ✅ BEHOBEN (Commit: 70d9d45)
**Priorität:** ~~🔴 CRITICAL~~ → ✅ BEHOBEN
**Schritte zur Reproduktion:**
1. Buchung mit DPolG-Mitglied erstellen (z.B. Booking 78, 79)
2. Rabatt wird korrekt berechnet und gespeichert (z.B. 17,85€)
3. Service hinzufügen (z.B. Endreinigung)
4. Rabatt springt auf 0,00€ ❌
5. Gesamtpreis ist falsch (zu hoch)

**Root Cause:**
- `link_service_template_to_booking()` in database.rs:3084-3092 hat nur Discount-Templates berücksichtigt
- DPolG-Rabatt ist KEIN Template, sondern wird basierend auf `ist_dpolg_mitglied` berechnet
- Bei Preisrekalkulation wurde DPolG-Rabatt komplett ignoriert

**Lösung:**
- database.rs:3097-3117: DPolG-Rabatt wird jetzt zusätzlich zu Templates berechnet
- database.rs:3179-3199: Gleicher Fix für `link_discount_template_to_booking()`
- Rabatt-Berechnung prüft `ist_dpolg_mitglied` Flag und holt Settings aus DB
- **Behoben in Commit:** `70d9d45` - fix(booking): DPolG-Rabatt wird jetzt bei Service-Änderungen persistiert

---

## 📊 Test-Ergebnis

### Zusammenfassung:
- **Tests durchgeführt:** ____ / 60+
- **Tests bestanden:** ____
- **Tests fehlgeschlagen:** ____
- **Kritische Bugs:** ____
- **Kleinere Bugs:** ____

### Produktionsfreigabe:
- [ ] **JA** - Alle Tests bestanden, App ist bereit für Production
- [ ] **NEIN** - Bugs müssen behoben werden

---

**Unterschrift Tester:** _____________________
**Datum:** _____________________
