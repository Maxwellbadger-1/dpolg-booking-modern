import { useState, useRef, useEffect } from 'react';
import { CheckCircle, Circle, ChevronDown, Mail } from 'lucide-react';
import { format } from 'date-fns';
import { de } from 'date-fns/locale';

interface InvoiceDropdownProps {
  invoiceSentAt?: string | null;
  invoiceSentTo?: string | null;
  bookingId: number;
  guestEmail: string;
  onInvoiceStatusChange: (bookingId: number, emailAddress: string) => void;
}

export default function InvoiceDropdown({
  invoiceSentAt,
  invoiceSentTo,
  bookingId,
  guestEmail,
  onInvoiceStatusChange
}: InvoiceDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  const handleMarkAsSent = () => {
    setIsOpen(false);
    onInvoiceStatusChange(bookingId, guestEmail);
  };

  const isSent = !!invoiceSentAt;

  return (
    <div className="relative inline-block" ref={dropdownRef}>
      {/* Current Invoice Status - Clickable */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border transition-all cursor-pointer ${
          isSent
            ? 'bg-emerald-100 text-emerald-800 border-emerald-200 hover:bg-emerald-200'
            : 'bg-slate-100 text-slate-600 border-slate-200 hover:bg-slate-200'
        }`}
        title={isSent ? `Rechnung versendet an: ${invoiceSentTo || guestEmail}` : 'Rechnung noch nicht versendet'}
      >
        {isSent ? (
          <>
            <Mail className="w-3.5 h-3.5" />
            <CheckCircle className="w-3.5 h-3.5" />
          </>
        ) : (
          <>
            <Circle className="w-3.5 h-3.5" />
            <span>Nicht versendet</span>
          </>
        )}
        <ChevronDown className={`w-3 h-3 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {/* Date Display */}
      {isSent && invoiceSentAt && (
        <div className="text-xs text-slate-500 mt-1">
          {format(new Date(invoiceSentAt), 'dd.MM.yyyy', { locale: de })}
        </div>
      )}

      {/* Dropdown Menu */}
      {isOpen && (
        <div className="absolute top-full left-1/2 -translate-x-1/2 mt-2 bg-white rounded-lg shadow-lg border border-slate-200 py-1 z-50 min-w-[200px]">
          <div className="px-3 py-2 border-b border-slate-200 bg-slate-50">
            <p className="text-xs font-semibold text-slate-600">Rechnungsstatus</p>
          </div>

          {!isSent && (
            <button
              onClick={handleMarkAsSent}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-emerald-50 transition-colors"
            >
              <Mail className="w-4 h-4 text-emerald-600" />
              <div className="flex-1 text-left">
                <span className="text-emerald-700 font-medium">Als versendet markieren</span>
                <p className="text-xs text-slate-500 mt-0.5">An: {guestEmail}</p>
              </div>
            </button>
          )}

          {isSent && (
            <div className="px-3 py-2">
              <div className="flex items-start gap-2">
                <CheckCircle className="w-4 h-4 text-emerald-600 mt-0.5" />
                <div className="flex-1">
                  <p className="text-sm text-emerald-700 font-medium">Rechnung versendet</p>
                  <p className="text-xs text-slate-500 mt-1">
                    Am: {format(new Date(invoiceSentAt), 'dd.MM.yyyy HH:mm', { locale: de })}
                  </p>
                  <p className="text-xs text-slate-500">
                    An: {invoiceSentTo || guestEmail}
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
