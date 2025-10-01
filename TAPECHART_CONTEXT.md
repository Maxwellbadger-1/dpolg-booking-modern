# TapeChart Kontext & Implementierungsdetails

Dieses Dokument enthält alle wichtigen Informationen über die TapeChart-Komponente des DPolG Buchungssystems.

## Tech Stack

- **React 18** mit TypeScript
- **@dnd-kit/core v6.3.1** - Drag & Drop Library (headless)
- **date-fns v4** - Date utilities
- **Tailwind CSS v3** - Styling
- **Tauri** - Desktop App Framework
- **Rust Backend** - SQLite Datenbank

## Wichtige Referenzen

### dnd-timeline GitHub Repository
**URL**: https://github.com/samuelarbibe/dnd-timeline

Dieses Repository ist die Hauptreferenz für:
- Drag & Drop Implementierungsmuster
- Resize-Handle Logic (virtuelle Handles, keine DOM-Elemente)
- Sensor-Konfiguration (PointerSensor mit Activation Constraints)
- Headless Architecture Patterns

**Key Learnings aus dnd-timeline:**
- Resize Handles sind virtuell (Math-basierte Positionserkennung, keine separaten DOM-Elemente)
- Standard Resize Handle Width: 20px (Aktivierung bei 10px vom Rand)
- Drag Activation Delay verhindert versehentliche Drags (150ms + 5px tolerance)
- **KEINE automatische Überlappungs-Prävention** - muss manuell implementiert werden
- **KEINE eingebauten Animationen** - Headless Library

## Aktuelle Implementierung

### Datei: `/src/components/TapeChart.tsx`

#### Kernfeatures

1. **Drag & Drop**
   - Implementiert mit `@dnd-kit/core`
   - PointerSensor mit 150ms delay, 5px tolerance
   - Überlappungs-Prävention: Bookings können sich berühren aber nicht überlappen

2. **Resize Functionality**
   - Virtuelle Handles: 30px Breite (15px Aktivierungszone von jedem Rand)
   - Live Preview während Resize
   - Überlappungs-Check vor Finalisierung
   - `select-none` CSS verhindert Textmarkierung

3. **Density Modes** (localStorage-persistent)
   - **Compact**: 80px cells, 60px rows, 80px header
   - **Comfortable**: 120px cells, 80px rows, 100px header (default)
   - **Spacious**: 160px cells, 100px rows, 120px header

4. **UI Features**
   - Monatliche Kalenderansicht mit Vor/Zurück Navigation
   - "Heute" Button mit Auto-Scroll
   - Status-basierte Farbcodierung (5 Status-Typen)
   - Wochenend-Hervorhebung
   - Hover-Effekte und Tooltips

### Wichtige Konstanten

```typescript
const RESIZE_HANDLE_WIDTH = 30; // Virtual resize zone
const DENSITY_SETTINGS = {
  compact: { cellWidth: 80, rowHeight: 60, headerHeight: 80 },
  comfortable: { cellWidth: 120, rowHeight: 80, headerHeight: 100 },
  spacious: { cellWidth: 160, rowHeight: 100, headerHeight: 120 }
};
```

### Drag & Resize Logic

#### Sensor Konfiguration
```typescript
const sensors = useSensors(
  useSensor(PointerSensor, {
    activationConstraint: {
      delay: 150,      // Verhindert versehentliche Drags
      tolerance: 5     // Erlaubt 5px Bewegung während Delay
    }
  })
);
```

#### Resize Direction Detection
```typescript
const getResizeDirection = (e: React.PointerEvent, element: HTMLElement): ResizeDirection => {
  const rect = element.getBoundingClientRect();
  const mouseX = e.clientX;

  // Check left edge (start)
  if (Math.abs(mouseX - rect.left) <= RESIZE_HANDLE_WIDTH / 2) return 'start';

  // Check right edge (end)
  if (Math.abs(mouseX - rect.right) <= RESIZE_HANDLE_WIDTH / 2) return 'end';

  return null;
};
```

#### Overlap Prevention
```typescript
const checkOverlap = (booking1Start: Date, booking1End: Date,
                     booking2Start: Date, booking2End: Date): boolean => {
  // Bookings dürfen sich berühren (gleiche Dates) aber nicht überlappen
  return !(booking1End <= booking2Start || booking1Start >= booking2End);
};
```

## Bekannte Patterns & Solutions

### Problem: Drag funktioniert nicht
**Lösung**:
- `{...listeners}` direkt auf Element anwenden
- Nur deaktivieren während `isResizing === true`
- NICHT manuell `listeners.onPointerDown()` aufrufen

### Problem: Text wird beim Resize markiert
**Lösung**:
- `select-none` CSS-Klasse auf Booking-Element
- `e.preventDefault()` im Resize PointerDown Handler

### Problem: Resize Handles zu klein
**Lösung**:
- Resize Handle Width erhöhen (z.B. 30px)
- Aktivierungszone = Width / 2

### Problem: Versehentliche Drags bei Klicks
**Lösung**:
- PointerSensor mit `activationConstraint`
- `delay: 150ms` und `tolerance: 5px`

## Datenmodell

### Booking Interface
```typescript
interface Booking {
  id: number;
  room_id: number;
  guest_id: number;
  reservierungsnummer: string;
  checkin_date: string;      // Format: 'YYYY-MM-DD'
  checkout_date: string;     // Format: 'YYYY-MM-DD'
  status: string;            // 'reserviert' | 'bestaetigt' | 'eingecheckt' | 'ausgecheckt' | 'storniert'
  gesamtpreis: number;
  bemerkungen?: string;
  room: Room;
  guest: Guest;
}
```

### Room Interface
```typescript
interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  price_member: number;
  price_non_member: number;
  ort: string;
  schluesselcode?: string;
}
```

## Git Commits Historie (Relevant)

- `ed60d16` - Add live resize preview animation
- `8f9816c` - Improve drag & resize UX based on dnd-timeline best practices
- `1c159f6` - Add overlap prevention for drag & resize operations
- `ddb9ccd` - Add density mode toggle with localStorage persistence
- `33460cf` - Fix drag functionality and prevent text selection during resize

## Häufige Aufgaben

### Neue Features aus dnd-timeline implementieren
1. dnd-timeline GitHub Repository durchsuchen
2. Relevante Code-Patterns identifizieren
3. An unser Booking-System anpassen (mit TypeScript + Tailwind)
4. Overlap-Prevention manuell hinzufügen (nicht in dnd-timeline enthalten)

### Debugging Drag & Drop Issues
1. `console.log` in DragStart/DragEnd/DragOver Handlern
2. Prüfen ob `sensors` korrekt konfiguriert
3. Prüfen ob `{...listeners}` auf Element angewendet
4. Prüfen ob `disabled` prop gesetzt ist

### Performance Optimierung
- Virtualisierung erwägen für viele Bookings (>100)
- `React.memo` für Booking-Komponenten
- `useMemo` für teure Berechnungen (z.B. `getBookingPosition`)

## Web Search Strategie für komplexe Probleme

Wenn du bei einem komplexen Problem nicht weiterkommst:

1. **Suche nach aktuellen Implementierungen:**
   - "dnd-kit timeline resize 2024"
   - "@dnd-kit/core drag drop calendar 2025"
   - "react timeline booking system drag resize"

2. **GitHub Code Search:**
   - Suche nach ähnlichen Projekten die dnd-kit verwenden
   - Schaue nach Issues im dnd-timeline Repository

3. **Stack Overflow:**
   - Tags: `[dnd-kit]`, `[react-dnd]`, `[drag-and-drop]`
   - Suche nach ähnlichen Fehlermeldungen

## Zukünftige Verbesserungen

- [ ] Virtualisierung für Performance bei vielen Bookings
- [ ] Keyboard-Navigation (Arrow Keys, Tab)
- [ ] Accessibility (ARIA Labels, Screen Reader Support)
- [ ] Multi-Select für Batch-Operationen
- [ ] Undo/Redo Funktionalität
- [ ] Backend-Sync nach Drag/Resize
- [ ] Konflikt-Visualisierung vor Drop
- [ ] Drag Preview mit Ghost-Element
- [ ] Touch-Support für Tablets

## Kontakt & Maintenance

Bei Fragen oder Problemen:
1. Dieses Dokument konsultieren
2. dnd-timeline GitHub Repository prüfen
3. Aktuelle Web-Suche nach ähnlichen Setups durchführen
