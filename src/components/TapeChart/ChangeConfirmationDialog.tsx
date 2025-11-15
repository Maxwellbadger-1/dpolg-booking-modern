import React, { useState } from 'react';
import { X, Calendar, Euro, ArrowRight, Mail, FileText } from 'lucide-react';
import { format, parseISO } from 'date-fns';
import { de } from 'date-fns/locale';

interface BookingChange {
  bookingId: number;
  reservierungsnummer: string;
  guestName: string;
  roomName: string;
  oldData: {
    checkin_date: string;
    checkout_date: string;
    room_id: number;
    gesamtpreis: number;
  };
  newData: {
    checkin_date: string;
    checkout_date: string;
    room_id: number;
    gesamtpreis: number;
  };
}

interface ChangeConfirmationDialogProps {
  change: BookingChange;
  onConfirm: (sendEmail: boolean, createInvoice: boolean) => void;
  onDiscard: () => void;
}

export default function ChangeConfirmationDialog({
  change,
  onConfirm,
  onDiscard,
}: ChangeConfirmationDialogProps) {
  const [sendEmail, setSendEmail] = useState(true);
  const [createInvoice, setCreateInvoice] = useState(true);

  const hasCheckinChanged = change.oldData.checkin_date !== change.newData.checkin_date;
  const hasCheckoutChanged = change.oldData.checkout_date !== change.newData.checkout_date;
  const hasRoomChanged = change.oldData.room_id !== change.newData.room_id;
  const hasPriceChanged = change.oldData.gesamtpreis !== change.newData.gesamtpreis;

  const priceDifference = change.newData.gesamtpreis - change.oldData.gesamtpreis;
  const priceChangePercent = (priceDifference / change.oldData.gesamtpreis) * 100;
  const isLargePriceIncrease = priceChangePercent > 50;

  const formatDate = (dateStr: string) => {
    return format(parseISO(dateStr), 'dd.MM.yyyy (EEE)', { locale: de });
  };

  const formatPrice = (price: number) => {
    return new Intl.NumberFormat('de-DE', {
      style: 'currency',
      currency: 'EUR',
    }).format(price);
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 max-w-2xl w-full mx-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-2xl font-bold text-white">Buchung wurde geändert</h2>
            <p className="text-slate-400 text-sm mt-1">
              {change.reservierungsnummer} · {change.guestName}
            </p>
          </div>
          <button
            onClick={onDiscard}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-slate-300" />
          </button>
        </div>

        {/* Changes Overview */}
        <div className="space-y-4 mb-6">
          {/* Check-in Change */}
          {hasCheckinChanged && (
            <div className="bg-slate-700/50 rounded-lg p-4">
              <div className="flex items-center gap-3">
                <Calendar className="w-5 h-5 text-blue-400" />
                <div className="flex-1">
                  <div className="text-sm text-slate-400 mb-1">Check-in</div>
                  <div className="flex items-center gap-2">
                    <span className="text-white font-medium">
                      {formatDate(change.oldData.checkin_date)}
                    </span>
                    <ArrowRight className="w-4 h-4 text-slate-500" />
                    <span className="text-emerald-400 font-bold">
                      {formatDate(change.newData.checkin_date)}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Check-out Change */}
          {hasCheckoutChanged && (
            <div className="bg-slate-700/50 rounded-lg p-4">
              <div className="flex items-center gap-3">
                <Calendar className="w-5 h-5 text-purple-400" />
                <div className="flex-1">
                  <div className="text-sm text-slate-400 mb-1">Check-out</div>
                  <div className="flex items-center gap-2">
                    <span className="text-white font-medium">
                      {formatDate(change.oldData.checkout_date)}
                    </span>
                    <ArrowRight className="w-4 h-4 text-slate-500" />
                    <span className="text-emerald-400 font-bold">
                      {formatDate(change.newData.checkout_date)}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Room Change */}
          {hasRoomChanged && (
            <div className="bg-slate-700/50 rounded-lg p-4">
              <div className="flex items-center gap-3">
                <div className="w-5 h-5 text-amber-400">🏠</div>
                <div className="flex-1">
                  <div className="text-sm text-slate-400 mb-1">Zimmer</div>
                  <div className="flex items-center gap-2">
                    <span className="text-white font-medium">{change.roomName}</span>
                    <ArrowRight className="w-4 h-4 text-slate-500" />
                    <span className="text-emerald-400 font-bold">
                      Zimmer {change.newData.room_id}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Price Change */}
          {hasPriceChanged && (
            <div
              className={`rounded-lg p-4 ${
                isLargePriceIncrease
                  ? 'bg-orange-500/20 border-2 border-orange-500'
                  : 'bg-slate-700/50'
              }`}
            >
              <div className="flex items-center gap-3">
                <Euro
                  className={`w-5 h-5 ${
                    isLargePriceIncrease ? 'text-orange-400' : 'text-emerald-400'
                  }`}
                />
                <div className="flex-1">
                  <div className="text-sm text-slate-400 mb-1">
                    Gesamtpreis
                    {isLargePriceIncrease && (
                      <span className="ml-2 text-orange-400 font-semibold">
                        ⚠️ Große Änderung (+{priceChangePercent.toFixed(0)}%)
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-white font-medium">
                      {formatPrice(change.oldData.gesamtpreis)}
                    </span>
                    <ArrowRight className="w-4 h-4 text-slate-500" />
                    <span
                      className={`font-bold ${
                        priceDifference > 0 ? 'text-orange-400' : 'text-emerald-400'
                      }`}
                    >
                      {formatPrice(change.newData.gesamtpreis)}
                    </span>
                    <span
                      className={`text-sm ${
                        priceDifference > 0 ? 'text-orange-300' : 'text-emerald-300'
                      }`}
                    >
                      ({priceDifference > 0 ? '+' : ''}
                      {formatPrice(priceDifference)})
                    </span>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Options */}
        <div className="space-y-3 mb-6">
          <label className="flex items-center gap-3 p-3 bg-slate-700/30 rounded-lg cursor-pointer hover:bg-slate-700/50 transition-colors">
            <input
              type="checkbox"
              checked={sendEmail}
              onChange={(e) => setSendEmail(e.target.checked)}
              className="w-4 h-4 rounded border-slate-600 text-blue-600 focus:ring-blue-500"
            />
            <Mail className="w-5 h-5 text-blue-400" />
            <div className="flex-1">
              <div className="text-white font-medium">Änderungs-Email senden</div>
              <div className="text-slate-400 text-xs">
                Gast über Änderungen informieren
              </div>
            </div>
          </label>

          <label className="flex items-center gap-3 p-3 bg-slate-700/30 rounded-lg cursor-pointer hover:bg-slate-700/50 transition-colors">
            <input
              type="checkbox"
              checked={createInvoice}
              onChange={(e) => setCreateInvoice(e.target.checked)}
              className="w-4 h-4 rounded border-slate-600 text-blue-600 focus:ring-blue-500"
            />
            <FileText className="w-5 h-5 text-purple-400" />
            <div className="flex-1">
              <div className="text-white font-medium">Neue Rechnung erstellen</div>
              <div className="text-slate-400 text-xs">
                Aktualisierte Rechnung generieren
              </div>
            </div>
          </label>
        </div>

        {/* Warning for large price increase */}
        {isLargePriceIncrease && (
          <div className="bg-orange-500/10 border border-orange-500/50 rounded-lg p-4 mb-6">
            <div className="flex items-start gap-3">
              <div className="text-2xl">⚠️</div>
              <div>
                <div className="text-orange-300 font-semibold mb-1">
                  Große Preisänderung
                </div>
                <div className="text-orange-200 text-sm">
                  Der Preis steigt um {priceChangePercent.toFixed(0)}% (
                  {formatPrice(priceDifference)}). Wir empfehlen dringend, den Gast per Email
                  zu informieren.
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-3">
          <button
            onClick={() => {
              console.log('🗑️ [ChangeConfirmationDialog] Verwerfen clicked!');
              onDiscard();
            }}
            className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
          >
            Verwerfen
          </button>
          <button
            onClick={() => {
              console.log('═══════════════════════════════════════');
              console.log('💾 [ChangeConfirmationDialog] SAVE BUTTON CLICKED!');
              console.log('📤 Calling onConfirm with:', { sendEmail, createInvoice });
              console.log('📦 change object:', JSON.stringify(change, null, 2));
              console.log('═══════════════════════════════════════');
              onConfirm(sendEmail, createInvoice);
            }}
            className="flex-1 px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors shadow-lg"
          >
            Änderungen speichern
          </button>
        </div>
      </div>
    </div>
  );
}
