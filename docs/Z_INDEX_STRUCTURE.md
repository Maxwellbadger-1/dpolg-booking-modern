# Z-Index Structure - Buchungstabelle

**Erstellt am:** 2025-10-20
**Problem:** Dropdowns wurden von sticky Elementen verdeckt

---

## ✅ Korrigierte Z-Index Hierarchie

```
z-[200] → Alle Dropdowns (HÖCHSTE PRIORITÄT)
   ├─ StatusDropdown
   ├─ PaymentDropdown (beide Menüs)
   ├─ InvoiceDropdown
   └─ Action Menu (Bearbeiten/Löschen)

z-20    → Sticky Header + Sticky Column Intersection
   └─ Header-Zelle "Reservierung" (sticky left + sticky top)

z-10    → Sticky Header (nur top)
   └─ Alle anderen Header-Zellen

z-5     → Sticky Column (nur left, Body-Rows)
   └─ Body-Zelle "Reservierung" (sticky left)

z-0/auto → Normale Zellen
   └─ Alle anderen Body-Zellen
```

---

## 🎨 Farben & Styling

### Header
- **Hintergrund:** `bg-slate-50` (leichtes Grau)
- **Border:** `border-b-2 border-slate-300`
- **Hover:** `hover:bg-slate-200` (für sticky column)

### Body Rows
- **Gerade Zeilen:** `bg-white`
- **Ungerade Zeilen:** `bg-slate-50`
- **Hover:** `group-hover:bg-blue-50` (beide)

### Sticky Column (Reservierung)
- **Opaque Background:** Gleicher Hintergrund wie Row (white oder slate-50)
- **Shadow:** `shadow-sm` für visuelle Trennung
- **Padding:** `pr-4` um Schatten sichtbar zu machen

---

## 📐 Grid Template

```css
gridTemplateColumns: '150px 200px 120px 110px 110px 140px 100px 120px 120px 120px 200px'
```

**Spalten:**
1. Reservierung (150px) - **Sticky Left**
2. Gast (200px)
3. Zimmer (120px)
4. Check-in (110px)
5. Check-out (110px)
6. Status (140px) - **Dropdown**
7. Preis (100px)
8. Bezahlt (120px) - **Dropdown**
9. Rechnung (120px) - **Dropdown**
10. Stiftung (120px)
11. Aktionen (200px) - **Action Menu**

---

## 🔍 Warum diese Struktur?

### Problem Vorher:
```
Header (z-20) > Dropdowns (z-[100])
→ Dropdowns wurden vom Header verdeckt!
```

**Ursache:** Dropdowns sind in `position: absolute` Rows → **Stacking Context Problem**

### Lösung:
```
Dropdowns (z-[200]) > Header (z-10/20)
→ Dropdowns erscheinen über ALLEM!
```

**Warum z-[200]?**
- Muss DEUTLICH höher sein als alle sticky Elemente
- Verhindert Stacking Context Probleme
- Genug Abstand für zukünftige Elemente

---

## 🛠️ Implementierung Details

### BookingList.tsx (Haupttabelle)

**Header:**
```tsx
<div className="bg-slate-50 ... sticky top-0 z-10 shadow-sm">
  {/* Sticky Column Header */}
  <div className="sticky left-0 z-20 bg-slate-50 ...">
    Reservierung
  </div>
  {/* Normale Header */}
  ...
</div>
```

**Body Rows:**
```tsx
<div className={`... ${isEven ? 'bg-white' : 'bg-slate-50'}`}>
  {/* Sticky Column */}
  <div className={`sticky left-0 ${isEven ? 'bg-white' : 'bg-slate-50'} ...`} style={{ zIndex: 5 }}>
    {booking.reservierungsnummer}
  </div>
  {/* Normale Cells mit Dropdowns */}
</div>
```

**Action Menu:**
```tsx
<div className="absolute ... z-[200] ...">
  {/* Dropdown Content */}
</div>
```

### StatusDropdown.tsx

```tsx
<div className="absolute ... z-[200] ...">
  {/* Dropdown Options */}
</div>
```

### PaymentDropdown.tsx

**Beide Menüs:**
```tsx
{/* Main Menu */}
<div className="absolute ... z-[200] ...">
  Bezahlt/Offen
</div>

{/* Zahlungsmethoden Submenu */}
<div className="absolute ... z-[200] ...">
  Überweisung, Barzahlung, etc.
</div>
```

### InvoiceDropdown.tsx

```tsx
<div className="absolute ... z-[200] ...">
  {/* Invoice Status */}
</div>
```

---

## ✅ Checkliste: Funktioniert die Struktur?

- [ ] Header ist durchgehend hellgrau (bg-slate-50)
- [ ] Aktionsspalte gehört visuell dazu (gleiches Grau)
- [ ] Dropdowns erscheinen ÜBER dem sticky Header
- [ ] Dropdowns erscheinen ÜBER der sticky Column
- [ ] Keine transparenten Bereiche in sticky Column
- [ ] Hover-Effekte funktionieren (blue-50)
- [ ] Schatten bei sticky Column sichtbar

---

## 🚨 Wichtige Hinweise

### NIEMALS:
- ❌ Dropdowns mit z-index < 100
- ❌ Transparente Hintergründe (`bg-slate-50/50`) bei sticky Elementen
- ❌ Header mit weißem Hintergrund (sollte hellgrau sein)
- ❌ Verschiedene Farben für Header vs. Aktionsspalte-Header

### IMMER:
- ✅ Dropdowns mit z-[200] oder höher
- ✅ Opaque Backgrounds für sticky Elemente
- ✅ Schatten bei sticky Column für visuelle Trennung
- ✅ Durchgehende Header-Farbe (bg-slate-50)

---

## 🎯 Quick Reference

**Dropdown erscheint nicht über Header?**
→ Z-Index erhöhen (z-[200] oder höher)

**Sticky Column hat transparente Bereiche?**
→ Background muss opaque sein (bg-white oder bg-slate-50, KEIN /50 Suffix!)

**Header wirkt getrennt?**
→ Alle Header-Zellen müssen gleiche Farbe haben (bg-slate-50)

**Aktionsspalte sieht anders aus?**
→ Header-Zelle für Aktionen muss auch bg-slate-50 haben

---

**🎉 Mit dieser Struktur:**
- Dropdowns erscheinen IMMER über allen anderen Elementen
- Sticky Elemente funktionieren einwandfrei
- Visuell konsistentes Design
- Keine Overlap-Probleme mehr!
