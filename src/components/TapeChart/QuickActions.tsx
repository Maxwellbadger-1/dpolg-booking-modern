import { Edit2, Mail, Copy } from 'lucide-react';

interface QuickActionsProps {
  bookingId: number;
  onEdit: (id: number) => void;
  onEmail: (id: number) => void;
  onDuplicate: (id: number) => void;
}

export default function QuickActions({
  bookingId,
  onEdit,
  onEmail,
  onDuplicate,
}: QuickActionsProps) {
  return (
    <div className="absolute top-1 right-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity z-10">
      <button
        onClick={(e) => {
          e.stopPropagation();
          onEdit(bookingId);
        }}
        className="p-1.5 bg-white/95 hover:bg-blue-500 hover:text-white rounded shadow-lg transition-colors"
        title="Bearbeiten"
      >
        <Edit2 className="w-3 h-3" />
      </button>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onEmail(bookingId);
        }}
        className="p-1.5 bg-white/95 hover:bg-green-500 hover:text-white rounded shadow-lg transition-colors"
        title="Email senden"
      >
        <Mail className="w-3 h-3" />
      </button>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onDuplicate(bookingId);
        }}
        className="p-1.5 bg-white/95 hover:bg-purple-500 hover:text-white rounded shadow-lg transition-colors"
        title="Duplizieren"
      >
        <Copy className="w-3 h-3" />
      </button>
    </div>
  );
}
