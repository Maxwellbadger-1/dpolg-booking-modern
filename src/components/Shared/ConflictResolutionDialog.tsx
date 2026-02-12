import React, { useState } from 'react';
import { X, AlertTriangle, Check, XCircle, RefreshCw } from 'lucide-react';

interface FieldConflict {
  field: string;
  label: string;
  yourValue: any;
  currentValue: any;
  isDifferent: boolean;
}

interface ConflictResolutionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  conflicts: FieldConflict[];
  onResolve: (resolution: 'force' | 'discard' | 'merge', mergedData?: any) => void;
  entityName?: string;
}

export function ConflictResolutionDialog({
  isOpen,
  onClose,
  conflicts,
  onResolve,
  entityName = 'Buchung',
}: ConflictResolutionDialogProps) {
  const [selectedValues, setSelectedValues] = useState<Record<string, 'yours' | 'current'>>({});

  if (!isOpen) return null;

  const hasConflicts = conflicts.some(c => c.isDifferent);

  const handleMerge = () => {
    const mergedData: Record<string, any> = {};

    conflicts.forEach(conflict => {
      const selection = selectedValues[conflict.field];
      if (selection === 'yours') {
        mergedData[conflict.field] = conflict.yourValue;
      } else if (selection === 'current') {
        mergedData[conflict.field] = conflict.currentValue;
      } else {
        // Default to current value if not selected
        mergedData[conflict.field] = conflict.currentValue;
      }
    });

    onResolve('merge', mergedData);
  };

  const formatValue = (value: any): string => {
    if (value === null || value === undefined) return '(leer)';
    if (typeof value === 'boolean') return value ? 'Ja' : 'Nein';
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-amber-100 rounded-full">
              <AlertTriangle className="w-6 h-6 text-amber-600" />
            </div>
            <div>
              <h2 className="text-xl font-semibold text-gray-900">
                Konflikt erkannt
              </h2>
              <p className="text-sm text-gray-600 mt-1">
                Die {entityName} wurde von einem anderen Benutzer geändert
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {!hasConflicts ? (
            <div className="text-center py-8 text-gray-500">
              Keine Konflikte gefunden. Die Daten sind identisch.
            </div>
          ) : (
            <div className="space-y-4">
              <p className="text-sm text-gray-700 mb-4">
                Wählen Sie für jedes Feld aus, welche Version Sie behalten möchten:
              </p>

              {conflicts
                .filter(c => c.isDifferent)
                .map((conflict) => (
                  <div
                    key={conflict.field}
                    className="border border-gray-200 rounded-lg p-4 bg-gray-50"
                  >
                    <div className="font-medium text-gray-900 mb-3">
                      {conflict.label}
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                      {/* Your Value */}
                      <button
                        onClick={() =>
                          setSelectedValues((prev) => ({
                            ...prev,
                            [conflict.field]: 'yours',
                          }))
                        }
                        className={`p-3 rounded-lg border-2 transition-all text-left ${
                          selectedValues[conflict.field] === 'yours'
                            ? 'border-blue-500 bg-blue-50'
                            : 'border-gray-300 bg-white hover:border-blue-300'
                        }`}
                      >
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-sm font-medium text-gray-700">
                            Ihre Änderung
                          </span>
                          {selectedValues[conflict.field] === 'yours' && (
                            <Check className="w-5 h-5 text-blue-600" />
                          )}
                        </div>
                        <div className="text-sm text-gray-900 break-words">
                          {formatValue(conflict.yourValue)}
                        </div>
                      </button>

                      {/* Current Value */}
                      <button
                        onClick={() =>
                          setSelectedValues((prev) => ({
                            ...prev,
                            [conflict.field]: 'current',
                          }))
                        }
                        className={`p-3 rounded-lg border-2 transition-all text-left ${
                          selectedValues[conflict.field] === 'current'
                            ? 'border-green-500 bg-green-50'
                            : 'border-gray-300 bg-white hover:border-green-300'
                        }`}
                      >
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-sm font-medium text-gray-700">
                            Aktueller Wert
                          </span>
                          {selectedValues[conflict.field] === 'current' && (
                            <Check className="w-5 h-5 text-green-600" />
                          )}
                        </div>
                        <div className="text-sm text-gray-900 break-words">
                          {formatValue(conflict.currentValue)}
                        </div>
                      </button>
                    </div>
                  </div>
                ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between gap-3 p-6 border-t border-gray-200 bg-gray-50">
          <div className="text-sm text-gray-600">
            {hasConflicts
              ? `${conflicts.filter(c => c.isDifferent).length} Konflikt(e) gefunden`
              : 'Keine Konflikte'}
          </div>

          <div className="flex gap-3">
            {/* Discard Button */}
            <button
              onClick={() => onResolve('discard')}
              className="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100 transition-colors flex items-center gap-2"
            >
              <XCircle className="w-4 h-4" />
              Verwerfen
            </button>

            {/* Force Override Button */}
            <button
              onClick={() => onResolve('force')}
              className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors flex items-center gap-2"
            >
              <AlertTriangle className="w-4 h-4" />
              Überschreiben
            </button>

            {/* Merge Button */}
            <button
              onClick={handleMerge}
              disabled={Object.keys(selectedValues).length === 0}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              <RefreshCw className="w-4 h-4" />
              Zusammenführen
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * Helper function to compare two objects and detect conflicts
 */
export function detectConflicts(
  yourData: Record<string, any>,
  currentData: Record<string, any>,
  fieldLabels: Record<string, string>
): FieldConflict[] {
  const conflicts: FieldConflict[] = [];

  Object.keys(yourData).forEach((field) => {
    const yourValue = yourData[field];
    const currentValue = currentData[field];
    const isDifferent = JSON.stringify(yourValue) !== JSON.stringify(currentValue);

    conflicts.push({
      field,
      label: fieldLabels[field] || field,
      yourValue,
      currentValue,
      isDifferent,
    });
  });

  return conflicts;
}
