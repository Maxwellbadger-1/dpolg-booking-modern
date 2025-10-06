import { useState, useEffect, useRef } from 'react';
import { Undo, Redo, ChevronDown, Clock, RefreshCw } from 'lucide-react';
import { commandManager } from '../lib/commandManager';

export default function UndoRedoButtons() {
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);
  const [showDropdown, setShowDropdown] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Listen to command manager changes
  useEffect(() => {
    const updateState = () => {
      setCanUndo(commandManager.canUndo());
      setCanRedo(commandManager.canRedo());
    };

    updateState();
    commandManager.addListener(updateState);

    return () => commandManager.removeListener(updateState);
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Z / Cmd+Z: Undo
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        handleUndo();
      }
      // Ctrl+Shift+Z / Cmd+Shift+Z: Redo
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && e.shiftKey) {
        e.preventDefault();
        handleRedo();
      }
      // Escape: Close dropdown
      if (e.key === 'Escape' && showDropdown) {
        setShowDropdown(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showDropdown]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false);
      }
    };

    if (showDropdown) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [showDropdown]);

  const handleUndo = () => {
    if (commandManager.canUndo()) {
      commandManager.undo();
    }
  };

  const handleRedo = () => {
    if (commandManager.canRedo()) {
      commandManager.redo();
    }
  };

  const undoStack = commandManager.getUndoStack();
  const latestCommand = undoStack[0];

  return (
    <div className="relative flex items-center gap-2" ref={dropdownRef}>
      {/* Undo Button */}
      <button
        onClick={handleUndo}
        disabled={!canUndo}
        className={`flex items-center gap-2 px-3 py-2 rounded-l-lg transition-colors ${
          canUndo
            ? 'hover:bg-slate-700 text-white'
            : 'text-slate-500 cursor-not-allowed'
        }`}
        title={canUndo ? `Rückgängig: ${latestCommand?.description} (Strg+Z)` : 'Nichts rückgängig zu machen'}
      >
        <Undo className="w-4 h-4" />
        <span className="text-sm font-medium">Rückgängig</span>
      </button>

      {/* Dropdown Toggle Button */}
      {canUndo && (
        <button
          onClick={() => setShowDropdown(!showDropdown)}
          className="px-2 py-2 rounded-r-lg border-l border-slate-600 transition-colors hover:bg-slate-700 text-white"
          title="Verlauf anzeigen"
        >
          <ChevronDown className={`w-4 h-4 transition-transform ${showDropdown ? 'rotate-180' : ''}`} />
        </button>
      )}

      {/* Separator */}
      {canUndo && <div className="h-6 w-px bg-slate-600"></div>}

      {/* Redo Button */}
      {canRedo && (
        <button
          onClick={handleRedo}
          className="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors hover:bg-slate-700 text-white"
          title="Wiederherstellen (Strg+Shift+Z)"
        >
          <Redo className="w-4 h-4" />
          <span className="text-sm font-medium">Wiederherstellen</span>
        </button>
      )}

      {/* Dropdown Menu - Undo Stack */}
      {showDropdown && canUndo && (
        <div className="absolute top-full right-0 mt-2 w-96 bg-slate-800 border border-slate-700 rounded-xl shadow-2xl z-50 overflow-hidden">
          {/* Header */}
          <div className="px-4 py-3 bg-slate-900/50 border-b border-slate-700">
            <div className="flex items-center gap-2">
              <Clock className="w-4 h-4 text-purple-400" />
              <h3 className="text-sm font-semibold text-white">Änderungsverlauf</h3>
            </div>
            <p className="text-xs text-slate-400 mt-1">Klicken Sie um zu dieser Stelle zurückzukehren</p>
          </div>

          {/* Command List */}
          <div className="max-h-96 overflow-y-auto">
            <div className="py-2">
              {undoStack.map((command, index) => (
                <button
                  key={index}
                  onClick={() => {
                    // Undo multiple times to get to this command
                    for (let i = 0; i <= index; i++) {
                      commandManager.undo();
                    }
                    setShowDropdown(false);
                  }}
                  className="w-full px-4 py-3 hover:bg-slate-700/50 transition-colors text-left flex items-start gap-3 group"
                >
                  {/* Icon */}
                  <div className="flex-shrink-0 mt-0.5">
                    <RefreshCw className="w-3 h-3 text-slate-400" />
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-sm font-medium text-white truncate">
                        {command.description}
                      </span>
                      {index === 0 && (
                        <span className="flex-shrink-0 px-2 py-0.5 bg-purple-500/20 text-purple-300 text-xs rounded">
                          Neueste
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-2 text-xs text-slate-400">
                      <Clock className="w-3 h-3" />
                      <span>{command.timestamp.toLocaleString('de-DE')}</span>
                    </div>
                  </div>

                  {/* Undo Icon (appears on hover) */}
                  <div className="flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Undo className="w-4 h-4 text-purple-400" />
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Footer */}
          <div className="px-4 py-3 bg-slate-900/50 border-t border-slate-700">
            <p className="text-xs text-slate-400 flex items-center gap-2">
              <span className="text-purple-400">ℹ️</span>
              Strg+Z / Strg+Shift+Z für Undo/Redo
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
