import { Plus } from 'lucide-react';

interface QuickBookingFABProps {
  onClick: () => void;
}

export default function QuickBookingFAB({ onClick }: QuickBookingFABProps) {
  return (
    <button
      onClick={onClick}
      className="fixed bottom-6 right-6 w-16 h-16 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 text-white rounded-full shadow-2xl flex items-center justify-center z-40 transition-all hover:scale-110 active:scale-95 group"
      title="Neue Buchung erstellen"
      aria-label="Neue Buchung erstellen"
    >
      <Plus className="w-8 h-8" />

      {/* Tooltip */}
      <div className="absolute right-full mr-3 px-3 py-2 bg-slate-900 text-white text-sm font-semibold rounded-lg opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
        Neue Buchung erstellen
        <div className="absolute top-1/2 -right-1 transform -translate-y-1/2 w-2 h-2 bg-slate-900 rotate-45"></div>
      </div>

      {/* Pulse Animation */}
      <div className="absolute inset-0 rounded-full bg-blue-500 animate-ping opacity-20"></div>
    </button>
  );
}
