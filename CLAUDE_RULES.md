# Claude Code Rules für DPolG Buchungssystem

## Regel 1: Automatische Deep Web Search bei komplexen Problemen

### Trigger Bedingungen

Führe automatisch eine **Deep Web Search** durch wenn:

1. **Du bei einem Problem nicht weiterkommst** nach 2-3 Versuchen
2. **Eine Implementierung unklar ist** trotz Code-Analyse
3. **Ein Bug auftritt** der nicht offensichtlich ist
4. **Eine Library-spezifische Funktion** nicht wie erwartet funktioniert
5. **Best Practices** für ein Pattern gefragt sind

### Such-Strategie

#### Phase 1: Tech Stack Spezifische Suche
```
Suche nach: "[Library Name] [Feature] [Current Year]"
Beispiel: "dnd-kit drag drop resize 2025"
```

#### Phase 2: GitHub Repository & Code Search
```
Suche nach: "github [Library Name] [Feature] example"
Beispiel: "github dnd-timeline resize implementation"
```

#### Phase 3: Kombinierte Setup-Suche
```
Suche nach: "[Tech1] + [Tech2] + [Feature] implementation"
Beispiel: "react typescript dnd-kit calendar drag drop"
```

#### Phase 4: Problem-Spezifische Suche
```
Suche nach: "[Error Message]" OR "[Specific Behavior]"
Beispiel: "dnd-kit drag not working after resize"
```

### Pflicht Web Searches für TapeChart

Bei TapeChart-bezogenen Aufgaben **IMMER** folgende Quellen prüfen:

1. **dnd-timeline GitHub Repository**
   - URL: https://github.com/samuelarbibe/dnd-timeline
   - Suche nach: Code patterns, Issues, Discussions

2. **@dnd-kit Documentation**
   - Aktuelle Version: 6.3.1
   - Suche nach: PointerSensor, useDraggable, useDroppable

3. **Stack Overflow**
   - Tags: `[dnd-kit]`, `[react-dnd]`, `[drag-and-drop]`
   - Suche nach: ähnlichen Problemen mit unserem Setup

4. **GitHub Code Search**
   - Suche nach: Repositories die dnd-kit für Calendar/Timeline verwenden
   - Filter: Updated in last year

### Beispiel: Komplexes Problem Workflow

**Scenario**: Drag funktioniert nicht mehr nach Density Mode Update

**Automatischer Workflow:**

1. **Sofort**: Lies `TAPECHART_CONTEXT.md` für bekannte Lösungen
2. **Web Search 1**: "dnd-kit drag not working listeners 2025"
3. **Web Search 2**: "github dnd-kit conditional listeners"
4. **Web Search 3**: "dnd-timeline drag resize conflict solution"
5. **Code Analysis**: Vergleiche gefundene Patterns mit aktuellem Code
6. **Implementation**: Wende beste Lösung an
7. **Documentation**: Update `TAPECHART_CONTEXT.md` mit neuer Lösung

### Output Format

Nach jeder Web Search dokumentiere:

```markdown
## Web Search Results: [Problem Description]

**Search Query**: "[exact query]"
**Source**: [URL oder "Multiple sources"]
**Key Findings**:
- Finding 1
- Finding 2
- Finding 3

**Applied Solution**: [Beschreibung der implementierten Lösung]
**File**: [path/to/file.tsx:line_number]
```

## Regel 2: TapeChart Spezialist Modus

Wenn der User über TapeChart spricht oder Änderungen anfragt:

### Automatische Aktionen

1. **Lies TAPECHART_CONTEXT.md** vor jeder Antwort
2. **Prüfe dnd-timeline GitHub** für ähnliche Implementierungen
3. **Verwende bekannte Patterns** aus unserer Historie
4. **Dokumentiere alle Änderungen** im Context-Dokument

### Wissen & Context

Du bist Experte für:
- **@dnd-kit/core v6.3.1** Drag & Drop Library
- **dnd-timeline** Patterns und Best Practices
- **React TypeScript** mit Hooks
- **Tailwind CSS v3** Styling
- **date-fns v4** Date manipulation
- **Overlap Prevention** Custom Logic
- **Density Modes** mit localStorage

### Code Style

Halte dich an diese Standards:
- **TypeScript strict mode** aktiviert
- **Functional components** mit Hooks (keine Class Components)
- **useCallback** für Event Handler
- **useMemo** für teure Berechnungen
- **cn()** utility für Tailwind classnames
- **select-none** für Drag/Resize Elemente

## Regel 3: Git Commit Best Practices

Bei jedem Code-Change:

1. **Vor Commit**: Zeige `git status` und `git diff`
2. **Commit Message Format**:
   ```
   [Action] [Component]: [Short Description]

   [Detailed explanation]
   - Change 1
   - Change 2

   🤖 Generated with Claude Code
   Co-Authored-By: Claude <noreply@anthropic.com>
   ```
3. **Nach Commit**: Zeige `git log -1 --oneline`

## Regel 4: Problemlösung Priorität

Bei Bugs oder Fehlern folge dieser Reihenfolge:

1. **Check TAPECHART_CONTEXT.md** - Bekannte Lösungen
2. **Web Search** - Aktuelle Implementierungen/Lösungen
3. **dnd-timeline GitHub** - Issues & Discussions
4. **Code Analysis** - Vergleich mit Working Patterns
5. **Experimentation** - Nur wenn keine Lösung gefunden
6. **Documentation** - Update Context mit Lösung

## Regel 5: Performance & Best Practices

**Immer beachten:**
- Overlap checks BEVOR State Updates
- `disabled` prop für Drag während Resize
- `user-select: none` für Drag-Elemente
- `e.preventDefault()` bei Resize Start
- Sensor constraints (150ms delay, 5px tolerance)
- localStorage für User Preferences

## Regel 6: Communication Style

- **Kurz und präzise** - keine unnötigen Erklärungen
- **Code first** - zeige Lösungen, nicht nur Beschreibungen
- **Web Searches transparent** - teile Findings immer
- **Deutsche UI Text** - aber englische Code Comments
- **Direkt zur Sache** - kein "Ich werde jetzt..." Preamble

## Zusammenfassung

Diese Rules stellen sicher dass:
- ✅ Deep Web Searches **automatisch** bei Problemen durchgeführt werden
- ✅ dnd-timeline Repository als **primäre Referenz** genutzt wird
- ✅ Aktuelle Best Practices **aus dem Web** angewendet werden
- ✅ Alle Lösungen **dokumentiert** werden für zukünftige Sessions
- ✅ Code-Qualität und **Konsistenz** erhalten bleibt

---

**Last Updated**: 2025-10-01
**Version**: 1.0
