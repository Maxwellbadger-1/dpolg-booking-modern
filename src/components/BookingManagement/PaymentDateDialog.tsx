import React, { useState } from 'react';
import { createPortal } from 'react-dom';
import { X, Calendar, CheckCircle } from 'lucide-react';
import { format } from 'date-fns';
import { de } from 'date-fns/locale';

interface PaymentDateDialogProps {
  isOpen: boolean;
  zahlungsmethode: string;
  onConfirm: (paymentDate: string) => void;
  onCancel: () => void;
}

export default function PaymentDateDialog({
  isOpen,
  zahlungsmethode,
  onConfirm,
  onCancel,
}: PaymentDateDialogProps) {
  const today = new Date().toISOString().split('T')[0];
  const [paymentDate, setPaymentDate] = useState<string>(today);

  if (!isOpen) return null;

  const handleDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setPaymentDate(e.target.value);
  };

  const handleConfirm = () => {
    onConfirm(paymentDate);
  };

  return createPortal(
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[9999]">
      <div
        className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 max-w-md w-full mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-emerald-500/10 rounded-full">
              <Calendar className="w-6 h-6 text-emerald-400" />
            </div>
            <div>
              <h2 className="text-2xl font-bold text-white">Bezahldatum</h2>
              <p className="text-slate-400 text-sm mt-1">{zahlungsmethode}</p>
            </div>
          </div>
          <button
            onClick={onCancel}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-slate-300" />
          </button>
        </div>

        {/* Content */}
        <div className="space-y-4 mb-6">
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Wann wurde die Zahlung erhalten?
            </label>
            <input
              type="date"
              value={paymentDate}
              onChange={handleDateChange}
              max={today}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
            />
            <p className="text-xs text-slate-400 mt-2">
              💡 Nur Daten bis heute auswählbar (kein Bezahldatum in der Zukunft)
            </p>
          </div>

          {/* Preview */}
          <div className="bg-slate-700/50 rounded-lg p-4">
            <div className="text-sm text-slate-400 mb-1">Ausgewähltes Datum:</div>
            <div className="text-lg font-semibold text-emerald-400">
              {format(new Date(paymentDate), 'dd. MMMM yyyy', { locale: de })}
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-3">
          <button
            onClick={onCancel}
            className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
          >
            Abbrechen
          </button>
          <button
            onClick={handleConfirm}
            className="flex-1 px-4 py-3 bg-emerald-600 hover:bg-emerald-700 text-white font-semibold rounded-lg transition-colors shadow-lg flex items-center justify-center gap-2"
          >
            <CheckCircle className="w-5 h-5" />
            Bestätigen
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
