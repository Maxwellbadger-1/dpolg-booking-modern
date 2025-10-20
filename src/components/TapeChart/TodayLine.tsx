import { format } from 'date-fns';
import { de } from 'date-fns/locale';

interface TodayLineProps {
  position: number; // Left position in pixels
  visible: boolean;
}

export default function TodayLine({ position, visible }: TodayLineProps) {
  if (!visible) return null;

  return (
    <div
      className="absolute inset-y-0 border-l-2 border-emerald-500 z-20 pointer-events-none"
      style={{ left: `${position}px` }}
    >
      <div className="sticky top-0 bg-emerald-500 text-white text-[10px] font-bold px-2 py-1 rounded-br shadow-lg">
        HEUTE
      </div>
    </div>
  );
}
