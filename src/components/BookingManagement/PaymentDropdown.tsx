import { useState, useRef } from 'react';
import { CheckCircle, AlertCircle, ChevronDown, CreditCard, Banknote, Wallet } from 'lucide-react';
import PaymentDateDialog from './PaymentDateDialog';
import PortalDropdown from '../PortalDropdown';

interface PaymentDropdownProps {
  isPaid: boolean;
  bookingId: number;
  onPaymentChange: (isPaid: boolean, zahlungsmethode?: string, paymentDate?: string) => void;
}

const ZAHLUNGSMETHODEN = [
  { value: 'Überweisung', label: 'Überweisung', icon: Banknote },
  { value: 'Barzahlung', label: 'Barzahlung', icon: Wallet },
  { value: 'EC-Karte', label: 'EC-Karte', icon: CreditCard },
  { value: 'Kreditkarte', label: 'Kreditkarte', icon: CreditCard },
  { value: 'PayPal', label: 'PayPal', icon: Wallet },
];

export default function PaymentDropdown({ isPaid, bookingId, onPaymentChange }: PaymentDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [showPaymentMethods, setShowPaymentMethods] = useState(false);
  const [showDateDialog, setShowDateDialog] = useState(false);
  const [selectedZahlungsmethode, setSelectedZahlungsmethode] = useState<string>('');
  const buttonRef = useRef<HTMLButtonElement>(null);

  const handleSelectPaid = () => {
    // Zeige Zahlungsmethoden-Auswahl
    setShowPaymentMethods(true);
  };

  const handleSelectPaymentMethod = (zahlungsmethode: string) => {
    setIsOpen(false);
    setShowPaymentMethods(false);
    setSelectedZahlungsmethode(zahlungsmethode);
    setShowDateDialog(true); // Zeige Datums-Dialog
  };

  const handleDateConfirm = (paymentDate: string) => {
    setShowDateDialog(false);
    onPaymentChange(true, selectedZahlungsmethode, paymentDate);
    setSelectedZahlungsmethode('');
  };

  const handleDateCancel = () => {
    setShowDateDialog(false);
    setSelectedZahlungsmethode('');
  };

  const handleSelectUnpaid = () => {
    setIsOpen(false);
    setShowPaymentMethods(false);
    if (isPaid) {
      onPaymentChange(false);
    }
  };

  return (
    <div className="relative inline-block">
      {/* Current Payment Status - Clickable */}
      <button
        ref={buttonRef}
        onClick={() => setIsOpen(!isOpen)}
        className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border transition-all cursor-pointer ${
          isPaid
            ? 'bg-emerald-100 text-emerald-800 border-emerald-200 hover:bg-emerald-200'
            : 'bg-amber-100 text-amber-800 border-amber-200 hover:bg-amber-200'
        }`}
      >
        {isPaid ? (
          <>
            <CheckCircle className="w-3.5 h-3.5" />
            Bezahlt
          </>
        ) : (
          <>
            <AlertCircle className="w-3.5 h-3.5" />
            Offen
          </>
        )}
        <ChevronDown className={`w-3 h-3 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {/* Dropdown Menu - Rendered via Portal */}
      <PortalDropdown
        isOpen={isOpen && !showPaymentMethods}
        onClose={() => {
          setIsOpen(false);
          setShowPaymentMethods(false);
        }}
        triggerRef={buttonRef}
        align="center"
      >
        <button
          onClick={handleSelectPaid}
          className={`w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-slate-50 transition-colors ${
            isPaid ? 'bg-slate-100 font-semibold' : ''
          }`}
        >
          <CheckCircle className="w-4 h-4 text-emerald-600" />
          <span className="text-emerald-700">Bezahlt</span>
          {isPaid && (
            <CheckCircle className="w-3.5 h-3.5 text-blue-600 ml-auto" />
          )}
        </button>
        <button
          onClick={handleSelectUnpaid}
          className={`w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-slate-50 transition-colors ${
            !isPaid ? 'bg-slate-100 font-semibold' : ''
          }`}
        >
          <AlertCircle className="w-4 h-4 text-amber-600" />
          <span className="text-amber-700">Offen</span>
          {!isPaid && (
            <CheckCircle className="w-3.5 h-3.5 text-blue-600 ml-auto" />
          )}
        </button>
      </PortalDropdown>

      {/* Zahlungsmethoden Menu - Rendered via Portal */}
      <PortalDropdown
        isOpen={isOpen && showPaymentMethods}
        onClose={() => {
          setIsOpen(false);
          setShowPaymentMethods(false);
        }}
        triggerRef={buttonRef}
        align="center"
      >
        <div className="px-3 py-2 border-b border-slate-200">
          <p className="text-xs font-semibold text-slate-600">Zahlungsmethode wählen</p>
        </div>
        {ZAHLUNGSMETHODEN.map((method) => {
          const Icon = method.icon;
          return (
            <button
              key={method.value}
              onClick={() => handleSelectPaymentMethod(method.value)}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-emerald-50 transition-colors"
            >
              <Icon className="w-4 h-4 text-slate-600" />
              <span className="text-slate-700">{method.label}</span>
            </button>
          );
        })}
      </PortalDropdown>

      {/* Payment Date Dialog */}
      <PaymentDateDialog
        isOpen={showDateDialog}
        zahlungsmethode={selectedZahlungsmethode}
        onConfirm={handleDateConfirm}
        onCancel={handleDateCancel}
      />
    </div>
  );
}
