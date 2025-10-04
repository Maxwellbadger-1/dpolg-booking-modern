import { useState, useRef, useEffect } from 'react';
import { CheckCircle, AlertCircle, ChevronDown } from 'lucide-react';

interface PaymentDropdownProps {
  isPaid: boolean;
  bookingId: number;
  onPaymentChange: (isPaid: boolean) => void;
}

export default function PaymentDropdown({ isPaid, bookingId, onPaymentChange }: PaymentDropdownProps) {
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

  const handleSelect = (newIsPaid: boolean) => {
    setIsOpen(false);
    if (newIsPaid !== isPaid) {
      onPaymentChange(newIsPaid);
    }
  };

  return (
    <div className="relative inline-block" ref={dropdownRef}>
      {/* Current Payment Status - Clickable */}
      <button
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

      {/* Dropdown Menu */}
      {isOpen && (
        <div className="absolute top-full left-1/2 -translate-x-1/2 mt-2 bg-white rounded-lg shadow-lg border border-slate-200 py-1 z-50 min-w-[140px]">
          <button
            onClick={() => handleSelect(true)}
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
            onClick={() => handleSelect(false)}
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
        </div>
      )}
    </div>
  );
}
