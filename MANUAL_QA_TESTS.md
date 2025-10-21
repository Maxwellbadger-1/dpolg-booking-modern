# 🧪 Manuelle QA Tests - Produktionsfreigabe
**Datum:** 2025-10-20
**Version:** Pre-Production
**Durchgeführt von:** _____________

---

## ✅ Checkliste - Basis-Funktionalität

### 1. Buchung erstellen
- [ ] Neuen Gast anlegen
  - [ ] Vorname, Nachname, Email eingeben
  - [ ] Adresse ausfüllen
  - [ ] Speichern erfolgreich
- [ ] Buchung anlegen
  - [ ] Gast auswählen
  - [ ] Zimmer auswählen
  - [ ] Check-in Datum wählen
  - [ ] Check-out Datum wählen
  - [ ] **DPolG-Mitglied Checkbox aktivieren**
  - [ ] Services hinzufügen (optional)
  - [ ] **Preis korrekt berechnet (mit 15% Rabatt auf Zimmerpreis bei Mitgliedern)?**
  - [ ] Speichern erfolgreich

**Notizen:**
```
Erwartetes Verhalten:
- Nebensaison: 3 Nächte à 45€ = 135€
- DPolG-Rabatt: 15% von 135€ = 20,25€
- Endreinigung: 20€
- Gesamtpreis: 134,75€
```

---

### 2. Buchung bearbeiten
- [ ] Existierende Buchung aus Liste öffnen
- [ ] Check-in/Check-out Datum ändern
- [ ] Services hinzufügen/entfernen
- [ ] DPolG-Mitglied Status ändern
- [ ] Preis wird live aktualisiert
- [ ] Speichern erfolgreich
- [ ] Änderungen im TapeChart sichtbar

---

### 3. Buchung löschen
- [ ] Buchung auswählen
- [ ] Löschen-Button klicken
- [ ] Bestätigungsdialog erscheint
- [ ] Löschen bestätigen
- [ ] Buchung verschwindet aus TapeChart
- [ ] Buchung verschwindet aus Buchungsliste

---

### 4. TapeChart Visualisierung
- [ ] TapeChart wird korrekt geladen
- [ ] Alle Buchungen werden angezeigt
- [ ] Farben korrekt:
  - [ ] Blau = Belegt
  - [ ] Grün = Check-in
  - [ ] Rot = Check-out
- [ ] **Drag & Drop funktioniert**
  - [ ] Buchung verschieben (Datum ändern)
  - [ ] Buchung auf anderes Zimmer verschieben
  - [ ] Buchung verlängern/verkürzen (Resize)
- [ ] Tooltips zeigen korrekte Informationen

---

### 5. PDF Export - Rechnung
- [ ] Buchung auswählen
- [ ] "Rechnung generieren" klicken
- [ ] PDF wird erstellt und angezeigt
- [ ] PDF-Inhalt korrekt:
  - [ ] Gastnamen
  - [ ] Zimmername
  - [ ] Check-in/Check-out Datum
  - [ ] Anzahl Nächte
  - [ ] Preisberechnung:
    - [ ] Grundpreis (Nächte × Zimmerpreis)
    - [ ] Services aufgelistet
    - [ ] DPolG-Rabatt (falls Mitglied)
    - [ ] Endreinigung
    - [ ] Gesamtpreis
  - [ ] Rechnungsnummer
  - [ ] Datum

---

### 6. PDF Export - Buchungsbestätigung
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

---

### 7. PDF Export - Putzplan ⭐ NEU
- [ ] Menü: "Putzplan" öffnen
- [ ] Monat auswählen
- [ ] "Putzplan exportieren" klicken
- [ ] PDF wird erstellt und im Programmordner gespeichert
- [ ] PDF öffnet sich automatisch
- [ ] PDF-Inhalt korrekt:
  - [ ] Zeitraum im Header angezeigt
  - [ ] Statistiken (Zimmer gesamt, Aktive Buchungen, Abreisen)
  - [ ] TapeChart Visualisierung:
    - [ ] Alle Zimmer aufgelistet
    - [ ] Check-in Tage: Blau mit Emojis (nur Emojis, kein Name!)
    - [ ] Belegte Tage: Blau mit Gastname
    - [ ] Check-out Tage: Rot mit Emojis (nur Emojis, kein "ABREISE ✓"!)
    - [ ] Emojis groß und erkennbar (16px)
    - [ ] Gastnamen passen vollständig in Zellen (Zeilenumbruch)
  - [ ] Legende vorhanden

**Bekannte Putzplan-Fixes:**
- ✅ Blaue Tage zwischen Check-in und Check-out werden angezeigt
- ✅ Zimmer 4 wird korrekt angezeigt (auch wenn Check-in/out außerhalb Monat)
- ✅ Nur Emojis in Check-in/out Zellen (kein Text mehr)
- ✅ Emojis 16px groß

---

### 8. Preisberechnung - Hauptsaison vs Nebensaison
- [ ] Buchung in **Nebensaison** erstellen (Jan-Mai, Sep-Dez)
  - [ ] Nebensaison-Preis wird verwendet
- [ ] Buchung in **Hauptsaison** erstellen (Jun-Aug)
  - [ ] Hauptsaison-Preis wird verwendet
- [ ] Preiseinstellungen öffnen
  - [ ] Hauptsaison Start/Ende konfigurieren
  - [ ] Hauptsaison aktivieren/deaktivieren
  - [ ] Änderungen werden gespeichert
  - [ ] Neue Buchungen verwenden aktualisierte Settings

---

### 9. DPolG-Rabatt Berechnung ⚠️ WICHTIG
- [ ] **Rabatt wird korrekt angewendet:**
  - [ ] Mitglied-Checkbox aktiviert
  - [ ] 15% Rabatt auf **Zimmerpreis** (NICHT Gesamtpreis!)
  - [ ] Services werden NICHT rabattiert
  - [ ] Endreinigung wird NICHT rabattiert
- [ ] **Rabatt wird in Rechnung angezeigt:**
  - [ ] Position "DPolG-Mitglieder Rabatt (15%)" vorhanden
  - [ ] Rabattbetrag korrekt
- [ ] **Rabatt-Konfiguration:**
  - [ ] Preiseinstellungen → DPolG-Rabatt aktivieren/deaktivieren
  - [ ] Rabatt-Prozentsatz änderbar
  - [ ] Rabatt-Basis: "Nur Zimmerpreis" ausgewählt

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

### 10. Gästeverwaltung
- [ ] Gästeliste öffnen
- [ ] Gast bearbeiten
  - [ ] Name ändern
  - [ ] Adresse ändern
  - [ ] Email ändern
  - [ ] Speichern erfolgreich
- [ ] Gast suchen/filtern
- [ ] Gast löschen (falls keine Buchungen vorhanden)

---

### 11. Zimmer-Verwaltung
- [ ] Zimmerliste öffnen
- [ ] Zimmer bearbeiten
  - [ ] Nebensaison-Preis ändern
  - [ ] Hauptsaison-Preis ändern
  - [ ] Endreinigung-Preis ändern
  - [ ] Speichern erfolgreich
- [ ] Neues Zimmer anlegen
- [ ] Zimmer deaktivieren (falls keine aktiven Buchungen)

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

### 13. Performance & Stabilität
- [ ] App startet in < 2 Sekunden
- [ ] Keine Console-Errors im Browser DevTools
- [ ] Keine Memory Leaks (länger als 5 Minuten verwenden)
- [ ] TapeChart scrollt smooth
- [ ] Drag & Drop ist flüssig (keine Verzögerung)
- [ ] PDF-Generierung < 3 Sekunden

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

### Bug 2: _______________________
**Beschreibung:**
**Status:**
**Priorität:**
**Schritte zur Reproduktion:**

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
