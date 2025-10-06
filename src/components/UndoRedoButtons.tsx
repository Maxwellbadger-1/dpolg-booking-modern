import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Undo, RotateCw, RefreshCw, ChevronDown, Clock, FileText, UserCircle, Database as DatabaseIcon } from 'lucide-react';

interface TransactionLog {
  id: number;
  operation_type: string;
  table_name: string;
  record_id: number;
  old_data: string | null;
  new_data: string | null;
  user_action: string;
  created_at: string;
  can_undo: boolean;
}

export default function UndoRedoButtons() {
  const [transactions, setTransactions] = useState<TransactionLog[]>([]);
  const [undoing, setUndoing] = useState(false);
  const [loading, setLoading] = useState(false);
  const [showDropdown, setShowDropdown] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadTransactions();

    // Listen for refresh-data events (triggered after undo or any data change)
    const handleRefresh = () => {
      console.log('🔄 [UndoRedoButtons] Refreshing transactions after data change');
      loadTransactions();
    };

    window.addEventListener('refresh-data', handleRefresh);
    return () => window.removeEventListener('refresh-data', handleRefresh);
  }, []);

  useEffect(() => {
    // Keyboard shortcuts
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Z (Windows/Linux) oder Cmd+Z (Mac)
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        handleUndo();
      }
      // Escape to close dropdown
      if (e.key === 'Escape' && showDropdown) {
        setShowDropdown(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [transactions, showDropdown]);

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

  const loadTransactions = async () => {
    setLoading(true);
    try {
      // Load last 10 transactions for dropdown
      const result = await invoke<TransactionLog[]>('get_recent_transactions_command', { limit: 10 });
      setTransactions(result);
    } catch (err) {
      console.error('Fehler beim Laden der Transaktionen:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleUndo = async (transactionId?: number) => {
    const targetTransaction = transactionId
      ? transactions.find(t => t.id === transactionId)
      : transactions.find(t => t.can_undo);

    if (!targetTransaction || undoing) return;

    setUndoing(true);
    setShowDropdown(false);

    try {
      const result = await invoke<string>('undo_transaction_command', { logId: targetTransaction.id });
      console.log('✅ Undo erfolgreich:', result);

      // Reload transactions
      await loadTransactions();

      // Trigger UNDO-specific refresh (causes full data reload in DataContext)
      window.dispatchEvent(new CustomEvent('undo-executed'));
    } catch (err) {
      console.error('❌ Undo fehlgeschlagen:', err);
      alert(`Fehler beim Rückgängig machen: ${err}`);
    } finally {
      setUndoing(false);
    }
  };

  const getOperationIcon = (operationType: string) => {
    switch (operationType) {
      case 'CREATE':
        return <FileText className="w-3 h-3 text-emerald-400" />;
      case 'UPDATE':
        return <RefreshCw className="w-3 h-3 text-blue-400" />;
      case 'DELETE':
        return <RotateCw className="w-3 h-3 text-red-400" />;
      default:
        return <DatabaseIcon className="w-3 h-3 text-slate-400" />;
    }
  };

  const getTableIcon = (tableName: string) => {
    switch (tableName) {
      case 'bookings':
        return <DatabaseIcon className="w-3 h-3" />;
      case 'guests':
        return <UserCircle className="w-3 h-3" />;
      case 'rooms':
        return <DatabaseIcon className="w-3 h-3" />;
      default:
        return <FileText className="w-3 h-3" />;
    }
  };

  const canUndo = transactions.some(t => t.can_undo);
  const lastTransaction = transactions.find(t => t.can_undo);

  return (
    <div className="relative flex items-center gap-2" ref={dropdownRef}>
      {/* Main Undo Button */}
      <button
        onClick={() => handleUndo()}
        disabled={!canUndo || undoing || loading}
        className={`flex items-center gap-2 px-3 py-2 rounded-l-lg transition-colors ${
          canUndo && !undoing && !loading
            ? 'hover:bg-slate-700 text-white'
            : 'text-slate-500 cursor-not-allowed'
        }`}
        title={canUndo ? `Rückgängig: ${lastTransaction?.user_action} (Strg+Z)` : 'Nichts rückgängig zu machen'}
      >
        {undoing ? (
          <RefreshCw className="w-4 h-4 animate-spin" />
        ) : (
          <Undo className="w-4 h-4" />
        )}
        <span className="text-sm font-medium">Rückgängig</span>
      </button>

      {/* Dropdown Toggle Button */}
      {canUndo && (
        <button
          onClick={() => setShowDropdown(!showDropdown)}
          disabled={undoing || loading}
          className={`px-2 py-2 rounded-r-lg border-l border-slate-600 transition-colors ${
            !undoing && !loading
              ? 'hover:bg-slate-700 text-white'
              : 'text-slate-500 cursor-not-allowed'
          }`}
          title="Verlauf anzeigen"
        >
          <ChevronDown className={`w-4 h-4 transition-transform ${showDropdown ? 'rotate-180' : ''}`} />
        </button>
      )}

      {/* Dropdown Menu */}
      {showDropdown && canUndo && (
        <div className="absolute top-full right-0 mt-2 w-96 bg-slate-800 border border-slate-700 rounded-xl shadow-2xl z-50 overflow-hidden">
          {/* Header */}
          <div className="px-4 py-3 bg-slate-900/50 border-b border-slate-700">
            <div className="flex items-center gap-2">
              <Clock className="w-4 h-4 text-purple-400" />
              <h3 className="text-sm font-semibold text-white">Letzte Änderungen</h3>
            </div>
            <p className="text-xs text-slate-400 mt-1">Klicken Sie auf eine Aktion um sie rückgängig zu machen</p>
          </div>

          {/* Transaction List */}
          <div className="max-h-96 overflow-y-auto">
            {transactions.filter(t => t.can_undo).length === 0 ? (
              <div className="px-4 py-8 text-center">
                <Clock className="w-8 h-8 text-slate-600 mx-auto mb-2" />
                <p className="text-sm text-slate-400">Keine rückgängig zu machenden Aktionen</p>
              </div>
            ) : (
              <div className="py-2">
                {transactions.filter(t => t.can_undo).map((transaction, index) => (
                  <button
                    key={transaction.id}
                    onClick={() => handleUndo(transaction.id)}
                    className="w-full px-4 py-3 hover:bg-slate-700/50 transition-colors text-left flex items-start gap-3 group"
                  >
                    {/* Icon */}
                    <div className="flex-shrink-0 mt-0.5">
                      {getOperationIcon(transaction.operation_type)}
                    </div>

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-sm font-medium text-white truncate">
                          {transaction.user_action}
                        </span>
                        {index === 0 && (
                          <span className="flex-shrink-0 px-2 py-0.5 bg-purple-500/20 text-purple-300 text-xs rounded">
                            Neueste
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-2 text-xs text-slate-400">
                        <Clock className="w-3 h-3" />
                        <span>{transaction.created_at}</span>
                        <span>•</span>
                        <div className="flex items-center gap-1">
                          {getTableIcon(transaction.table_name)}
                          <span>ID: {transaction.record_id}</span>
                        </div>
                      </div>
                    </div>

                    {/* Undo Icon (appears on hover) */}
                    <div className="flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                      <Undo className="w-4 h-4 text-purple-400" />
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="px-4 py-3 bg-slate-900/50 border-t border-slate-700">
            <p className="text-xs text-slate-400 flex items-center gap-2">
              <span className="text-purple-400">ℹ️</span>
              Strg+Z für schnelles Rückgängig machen
            </p>
          </div>
        </div>
      )}

      {/* Separator */}
      {canUndo && (
        <div className="h-6 w-px bg-slate-600"></div>
      )}
    </div>
  );
}
