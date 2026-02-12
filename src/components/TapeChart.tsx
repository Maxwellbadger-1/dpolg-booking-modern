import React, { useState, useRef, useEffect, useLayoutEffect, useCallback, useMemo } from 'react';
import { format, addDays, differenceInDays, startOfMonth, endOfMonth, eachDayOfInterval, addMonths, subMonths, startOfDay, isSameDay } from 'date-fns';
import { de } from 'date-fns/locale';
import { invoke } from '@tauri-apps/api/core';
import toast from 'react-hot-toast';
import { cn } from '../lib/utils';
import { useData } from '../context/DataContext';
import { commandManager, UpdateBookingDatesCommand } from '../lib/commandManager';
import { invokeWithRetry } from '../lib/retry';
import { useBatchPriceCalculation, getBookingPrice } from '../hooks/useBatchPriceCalculation';
import {
  DndContext,
  DragEndEvent,
  DragOverlay,
  useDraggable,
  useDroppable,
  DragStartEvent,
  pointerWithin,
  DragOverEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';
import ContextMenu, { ContextMenuItem } from './ContextMenu';
import { Edit2, Mail, XCircle, X, Users } from 'lucide-react';
import ChangeConfirmationDialog from './TapeChart/ChangeConfirmationDialog';
import TapeChartFilters from './TapeChart/TapeChartFilters';
import { filterBookings, getUniqueRoomTypes } from './TapeChart/TapeChartHelpers';

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  capacity: number;
  ort: string;
}

interface Guest {
  vorname: string;
  nachname: string;
}

interface ServiceTemplate {
  id: number;
  name: string;
  emoji?: string;
  color_hex?: string;
  cleaning_plan_position?: string; // 'start' or 'end'
}

interface DiscountTemplate {
  id: number;
  name: string;
  emoji?: string;
  color_hex?: string;
}

interface Booking {
  id: number;
  room_id: number;
  guest_id: number;
  reservierungsnummer: string;
  checkin_date: string;
  checkout_date: string;
  anzahl_gaeste: number;
  status: string;
  bezahlt: boolean;
  rechnung_versendet_am?: string | null;
  ist_stiftungsfall: boolean;
  // NORMALIZED STATE: No nested room/guest objects!
  // Use guestMap.get(booking.guest_id) and roomMap.get(booking.room_id) for lookups
  services: ServiceTemplate[];
  discounts: DiscountTemplate[];
}

interface TapeChartProps {
  startDate?: Date;
  endDate?: Date;
  onBookingClick?: (bookingId: number) => void;
  onCreateBooking?: (roomId: number, startDate: string, endDate: string) => void;
  onBookingEdit?: (bookingId: number) => void;
  onBookingCancel?: (bookingId: number) => void;
  onSendEmail?: (bookingId: number) => void;
}

const STATUS_COLORS: Record<string, { bg: string; border: string; text: string; shadow: string }> = {
  bestaetigt: {
    bg: 'bg-gradient-to-r from-emerald-500 to-emerald-600',
    border: 'border-emerald-700',
    text: 'text-white',
    shadow: 'shadow-lg shadow-emerald-500/50'
  },
  eingecheckt: {
    bg: 'bg-gradient-to-r from-blue-500 to-blue-600',
    border: 'border-blue-700',
    text: 'text-white',
    shadow: 'shadow-lg shadow-blue-500/50'
  },
  ausgecheckt: {
    bg: 'bg-gradient-to-r from-gray-400 to-gray-500',
    border: 'border-gray-600',
    text: 'text-white',
    shadow: 'shadow-lg shadow-gray-500/50'
  },
  storniert: {
    bg: 'bg-gradient-to-r from-red-500 to-red-600',
    border: 'border-red-700',
    text: 'text-white',
    shadow: 'shadow-lg shadow-red-500/50'
  },
  stiftungsfall: {
    bg: 'bg-gradient-to-r from-amber-500 to-orange-600',
    border: 'border-amber-700',
    text: 'text-white',
    shadow: 'shadow-lg shadow-amber-500/50'
  },
};

// Density Mode Settings
type DensityMode = 'compact' | 'comfortable' | 'spacious';

const DENSITY_SETTINGS = {
  compact: {
    cellWidth: 80,
    rowHeight: 60,
    headerHeight: 80,
    fontSize: 'text-xs',
    padding: 'p-1'
  },
  comfortable: {
    cellWidth: 120,
    rowHeight: 80,
    headerHeight: 100,
    fontSize: 'text-sm',
    padding: 'p-2'
  },
  spacious: {
    cellWidth: 160,
    rowHeight: 100,
    headerHeight: 120,
    fontSize: 'text-base',
    padding: 'p-3'
  }
};

const CELL_WIDTH = 120;
const ROW_HEIGHT = 80;
const HEADER_HEIGHT = 100;
const SIDEBAR_WIDTH = 200;
const RESIZE_HANDLE_WIDTH = 30; // Virtual resize zone on each edge (15px from edge)

type ResizeDirection = 'start' | 'end' | null;

interface DraggableBookingProps {
  booking: Booking;
  position: { left: number; width: number };
  isOverlay?: boolean;
  rowHeight: number;
  cellWidth: number; // KRITISCH: cellWidth übergeben für korrekte Resize-Berechnung bei Skalierung
  onResize?: (bookingId: number, direction: 'start' | 'end', daysDelta: number) => void;
  onClick?: (bookingId: number) => void;
  onContextMenu?: (bookingId: number, x: number, y: number) => void;
  hasOverlap?: boolean; // Red overlay when drag has overlap
  isPending?: boolean; // NEW: Shows save/discard buttons
  onManualSave?: () => void; // NEW: Manual save handler
  onManualDiscard?: () => void; // NEW: Manual discard handler
  continuesFromPreviousMonth?: boolean; // Booking started in previous month
  continuesNextMonth?: boolean; // Booking continues into next month
  originalCheckin?: string; // Original check-in date (for tooltip)
  originalCheckout?: string; // Original check-out date (for tooltip)
  // NORMALIZED STATE: Guest name passed as prop for O(1) lookup
  guestName?: string;
}

function DraggableBooking({ booking, position, isOverlay = false, rowHeight, cellWidth, onResize, onClick, onContextMenu, hasOverlap = false, isPending = false, onManualSave, onManualDiscard, continuesFromPreviousMonth = false, continuesNextMonth = false, originalCheckin, originalCheckout, guestName = 'Unbekannt' }: DraggableBookingProps) {
  const [isResizing, setIsResizing] = useState(false);
  const [resizeDirection, setResizeDirection] = useState<ResizeDirection>(null);
  const [cursor, setCursor] = useState<string>('move');
  const [resizePreview, setResizePreview] = useState<{ left: number; width: number } | null>(null);
  const dragStartX = useRef<number>(0);
  const resizeStartPosition = useRef<{ left: number; width: number }>({ left: 0, width: 0 });

  const { attributes, listeners, setNodeRef, isDragging, transform } = useDraggable({
    id: `booking-${booking.id}`,
    data: booking,
    disabled: isResizing || isPending, // Disable drag during resize OR wenn pending
  });

  // Stiftungsfall hat Priorität: Orange/Amber Farbe statt Status-Farbe
  const colors = booking.ist_stiftungsfall
    ? STATUS_COLORS.stiftungsfall
    : (STATUS_COLORS[booking.status] || STATUS_COLORS.bestaetigt);

  // Detect if pointer is in resize zone (dnd-timeline pattern)
  // FIX: Disable resize handles for clipped edges (month-spanning bookings)
  const getResizeDirection = useCallback((e: React.PointerEvent, element: HTMLElement): ResizeDirection => {
    const rect = element.getBoundingClientRect();
    const mouseX = e.clientX;

    // Check left edge (start) - ONLY if not clipped from previous month
    if (!continuesFromPreviousMonth && Math.abs(mouseX - rect.left) <= RESIZE_HANDLE_WIDTH / 2) {
      return 'start';
    }

    // Check right edge (end) - ONLY if not clipped into next month
    if (!continuesNextMonth && Math.abs(mouseX - rect.right) <= RESIZE_HANDLE_WIDTH / 2) {
      return 'end';
    }

    return null;
  }, [continuesFromPreviousMonth, continuesNextMonth]);

  // Track if this was a click or drag
  const clickStartPos = useRef<{ x: number; y: number } | null>(null);
  const hasMoved = useRef(false);

  // Handle pointer down - determine if resize or drag
  const handlePointerDown = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    if (isOverlay) return;

    // ALWAYS stop propagation to prevent DroppableCell drag-to-create
    e.stopPropagation();

    // Track click start position
    clickStartPos.current = { x: e.clientX, y: e.clientY };
    hasMoved.current = false;

    const direction = getResizeDirection(e, e.currentTarget);

    if (direction && onResize) {
      // Start resize
      setIsResizing(true);
      setResizeDirection(direction);
      dragStartX.current = e.clientX;
      resizeStartPosition.current = { ...position };
    } else {
      // Normal drag - let dnd-kit handle it
      listeners?.onPointerDown?.(e);
    }
  }, [getResizeDirection, onResize, position, listeners, isOverlay]);

  // Handle click (if not dragged)
  const handleClick = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (isOverlay || isResizing) return;

    // Check if pointer moved (drag vs click)
    if (clickStartPos.current) {
      const deltaX = Math.abs(e.clientX - clickStartPos.current.x);
      const deltaY = Math.abs(e.clientY - clickStartPos.current.y);

      // If moved less than 5px, it's a click
      if (deltaX < 5 && deltaY < 5 && onClick) {
        e.stopPropagation();
        onClick(booking.id);
      }
    }
  }, [isOverlay, isResizing, onClick, booking.id]);

  // Handle pointer move - update cursor
  const handlePointerMove = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    if (isOverlay || isResizing) return;

    const direction = getResizeDirection(e, e.currentTarget);

    if (direction) {
      setCursor('ew-resize');
    } else {
      // Check if hovering over clipped edge (show not-allowed cursor)
      const rect = e.currentTarget.getBoundingClientRect();
      const mouseX = e.clientX;
      const isNearLeftEdge = Math.abs(mouseX - rect.left) <= RESIZE_HANDLE_WIDTH / 2;
      const isNearRightEdge = Math.abs(mouseX - rect.right) <= RESIZE_HANDLE_WIDTH / 2;

      if ((isNearLeftEdge && continuesFromPreviousMonth) || (isNearRightEdge && continuesNextMonth)) {
        setCursor('not-allowed');
      } else {
        setCursor('move');
      }
    }
  }, [getResizeDirection, isOverlay, isResizing, continuesFromPreviousMonth, continuesNextMonth]);

  // Layout effect to handle resize events with LIVE preview
  useLayoutEffect(() => {
    if (!isResizing || !onResize) return;

    console.log('🎬 [RESIZE PREVIEW] useLayoutEffect ACTIVE');
    console.log('   Booking ID:', booking.id);
    console.log('   Direction:', resizeDirection);
    console.log('   Cell Width:', cellWidth, 'px (wichtig für korrekte Berechnung!)');
    console.log('   Start Position:', resizeStartPosition.current);

    const handlePointerMove = (e: PointerEvent) => {
      const deltaX = e.clientX - dragStartX.current;
      const daysDelta = Math.round(deltaX / cellWidth); // FIX: Verwende cellWidth statt CELL_WIDTH

      // Calculate LIVE preview position
      if (resizeDirection === 'start') {
        // Resize from left - change left and width
        const newLeft = resizeStartPosition.current.left + (daysDelta * cellWidth);
        const newWidth = resizeStartPosition.current.width - (daysDelta * cellWidth);

        // Only update if width stays positive
        if (newWidth > cellWidth / 2) {
          console.log('📏 [PREVIEW] ⬅️ Links:', daysDelta, 'Tage, Width:', Math.round(newWidth), 'px');
          setResizePreview({ left: newLeft, width: newWidth });
        }
      } else if (resizeDirection === 'end') {
        // Resize from right - only change width
        const newWidth = resizeStartPosition.current.width + (daysDelta * cellWidth);

        // Only update if width stays positive
        if (newWidth > cellWidth / 2) {
          console.log('📏 [PREVIEW] ➡️ Rechts:', daysDelta, 'Tage, Width:', Math.round(newWidth), 'px');
          setResizePreview({
            left: resizeStartPosition.current.left,
            width: newWidth
          });
        }
      }
    };

    const handlePointerUp = (e: PointerEvent) => {
      const deltaX = e.clientX - dragStartX.current;
      const daysDelta = Math.round(deltaX / cellWidth); // FIX: Verwende cellWidth statt CELL_WIDTH

      console.log('🏁 [RESIZE PREVIEW] Pointer Up - Delta:', daysDelta, 'Tage');

      if (daysDelta !== 0 && resizeDirection) {
        console.log('✅ Calling onResize Handler');
        onResize(booking.id, resizeDirection, daysDelta);
      } else {
        console.log('⏩ Kein Delta - kein Resize');
      }

      console.log('🧹 Cleaning up resize state');
      setIsResizing(false);
      setResizeDirection(null);
      setResizePreview(null); // Clear preview
    };

    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);

    return () => {
      console.log('🧹 [RESIZE PREVIEW] Cleanup - Removing event listeners');
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
  }, [isResizing, onResize, booking.id, resizeDirection, cellWidth]);

  // Use resize preview if resizing, otherwise use normal position
  const currentPosition = resizePreview || position;

  const style = isOverlay ? {
    width: `${position.width}px`,
    height: `${rowHeight - 16}px`,
  } : {
    left: `${currentPosition.left}px`,
    width: `${currentPosition.width}px`,
    top: '8px',
    bottom: '8px',
    transform: CSS.Transform.toString(transform),
    cursor: cursor,
    // FIX: Spezifische transitions statt "all" - verhindert dass Text beim Scrollen verschwindet
    transition: isResizing ? 'none' : 'transform 0.2s ease-out, opacity 0.2s ease-out',
  };

  return (
    <div
      ref={setNodeRef}
      {...attributes}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onClick={handleClick}
      onContextMenu={(e) => {
        if (isOverlay) return;
        e.preventDefault();
        e.stopPropagation();
        if (onContextMenu) {
          onContextMenu(booking.id, e.clientX, e.clientY);
        }
      }}
      className={cn(
        isOverlay ? "rounded-xl border-2" : "absolute rounded-xl border-2 group",
        // FIX: transition-transform statt transition-all - verhindert dass Text beim Scrollen verschwindet
        "flex items-center px-3 transition-transform duration-200",
        !isOverlay && !isResizing && "hover:scale-[1.02] active:scale-95",
        "backdrop-blur-sm select-none", // select-none prevents text selection
        isDragging && !isOverlay && "opacity-0",
        isResizing && "border-dashed border-4 scale-[1.02]",
        isPending && "z-50", // FIX: Elevate entire booking above sticky headers when pending (headers are z-10, z-20, z-40)
        colors.bg,
        colors.border,
        colors.text,
        colors.shadow
      )}
      style={{
        ...style,
        userSelect: 'none', // Prevent text selection
        WebkitUserSelect: 'none', // Safari
        // FIX: Hardware Acceleration - verhindert Text-Flickering beim Scrollen
        transform: style.transform ? `${style.transform} translateZ(0)` : 'translateZ(0)',
        WebkitBackfaceVisibility: 'hidden',
        backfaceVisibility: 'hidden',
        willChange: 'transform',
      }}
      title={`${booking.reservierungsnummer}\n${originalCheckin || booking.checkin_date} - ${originalCheckout || booking.checkout_date}${continuesFromPreviousMonth || continuesNextMonth ? '\n⚠️ Monatsübergreifende Buchung' : ''}`}
    >
      {/* LEFT ARROW - Continues from previous month */}
      {continuesFromPreviousMonth && (
        <div
          className="absolute left-0 top-0 bottom-0 w-5 flex items-center justify-center bg-black/30 rounded-l-lg pointer-events-none"
          title="Fortsetzung von vorigem Monat"
        >
          <span className="text-white/80 text-xs">◀</span>
        </div>
      )}

      <div className="flex-1 overflow-hidden">
        <div className="text-sm font-bold truncate drop-shadow-sm">
          {guestName}
        </div>
        <div className="text-xs truncate opacity-90 font-medium flex items-center gap-1">
          {booking.reservierungsnummer}
          {/* Personenanzahl */}
          <div className="inline-flex items-center justify-center gap-0.5 px-1.5 py-0.5 bg-white/20 rounded" title={`${booking.anzahl_gaeste} ${booking.anzahl_gaeste === 1 ? 'Person' : 'Personen'}`}>
            <Users className="w-3 h-3 text-white" />
            <span className="text-[10px] font-bold text-white">{booking.anzahl_gaeste}</span>
          </div>
          {booking.bezahlt && (
            <div className="inline-flex items-center justify-center w-4 h-4 bg-emerald-500 rounded-full" title="Bezahlt">
              <span className="text-[10px] font-bold text-white">€</span>
            </div>
          )}
          {booking.rechnung_versendet_am && (
            <div className="inline-flex items-center justify-center w-4 h-4 bg-blue-500 rounded-full" title="Rechnung versendet">
              <Mail className="w-3 h-3 text-white" />
            </div>
          )}
          {/* Service Emojis */}
          {booking.services?.filter(s => s.emoji).map((service) => (
            <span key={service.id} className="text-sm" title={service.name}>
              {service.emoji}
            </span>
          ))}
          {/* Discount Emojis */}
          {booking.discounts?.filter(d => d.emoji).map((discount) => (
            <span key={discount.id} className="text-sm" title={discount.name}>
              {discount.emoji}
            </span>
          ))}
        </div>
      </div>

      {/* RIGHT ARROW - Continues to next month */}
      {continuesNextMonth && (
        <div
          className="absolute right-0 top-0 bottom-0 w-5 flex items-center justify-center bg-black/30 rounded-r-lg pointer-events-none"
          title="Fortsetzung in nächsten Monat"
        >
          <span className="text-white/80 text-xs">▶</span>
        </div>
      )}

      {/* Manual Save/Discard Buttons (wenn Pending) */}
      {isPending && onManualSave && onManualDiscard && (
        <div className="absolute -top-10 right-0 flex gap-2 animate-in fade-in slide-in-from-top-2 duration-200">
          <button
            onClick={(e) => {
              e.stopPropagation();
              onManualSave();
            }}
            className="bg-green-500 hover:bg-green-600 text-white px-3 py-1.5 rounded-lg shadow-xl flex items-center gap-2 font-semibold text-sm transition-colors animate-pulse"
            title="Änderungen speichern"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            Speichern
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onManualDiscard();
            }}
            className="bg-red-500 hover:bg-red-600 text-white px-3 py-1.5 rounded-lg shadow-xl flex items-center gap-2 font-semibold text-sm transition-colors"
            title="Änderungen verwerfen"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      )}
    </div>
  );
}

function DroppableCell({ roomId, dayIndex, isWeekend, isToday, isEvenRow, cellWidth, rowHeight, hasOverlap, isCreateDragPreview, onCreateDragStart, onCreateDragMove, onCreateDragEnd, children }: {
  roomId: number;
  dayIndex: number;
  isWeekend: boolean;
  isToday: boolean;
  isEvenRow: boolean;
  cellWidth: number;
  rowHeight: number;
  hasOverlap?: boolean;
  isCreateDragPreview?: boolean;
  onCreateDragStart?: (roomId: number, dayIndex: number) => void;
  onCreateDragMove?: (roomId: number, dayIndex: number) => void;
  onCreateDragEnd?: () => void;
  children?: React.ReactNode;
}) {
  const { setNodeRef, isOver } = useDroppable({
    id: `cell-${roomId}-${dayIndex}`,
    data: { roomId, dayIndex },
  });

  return (
    <div
      ref={setNodeRef}
      onMouseDown={(e) => {
        // Only start drag-to-create if clicking directly on cell (not on booking)
        if (e.target === e.currentTarget && onCreateDragStart) {
          e.preventDefault();
          onCreateDragStart(roomId, dayIndex);
        }
      }}
      onMouseEnter={() => {
        if (onCreateDragMove) {
          onCreateDragMove(roomId, dayIndex);
        }
      }}
      onMouseUp={() => {
        if (onCreateDragEnd) {
          onCreateDragEnd();
        }
      }}
      className={cn(
        "border-r border-slate-200 transition-all duration-200 box-border relative",
        // Zebra Striping (subtil)
        isEvenRow ? "bg-white" : "bg-slate-50/50",
        // Weekend Override (subtiler Blaustich)
        isWeekend && "bg-blue-50/30",
        // Today Highlight (sehr deutlich)
        isToday && "bg-emerald-100/70",
        // Drag States (höchste Priorität)
        isOver && !hasOverlap && "bg-blue-300/60 ring-2 ring-blue-400 ring-inset",
        isOver && hasOverlap && "bg-red-500/60 ring-2 ring-red-600 ring-inset",
        isCreateDragPreview && "bg-emerald-400/40 ring-2 ring-emerald-500 ring-inset"
      )}
      style={{
        width: `${cellWidth}px`,
        height: `${rowHeight}px`,
        minWidth: `${cellWidth}px`,
        maxWidth: `${cellWidth}px`,
      }}
    >
      {children}
    </div>
  );
}

export default function TapeChart({ startDate, endDate, onBookingClick, onCreateBooking, onBookingEdit, onBookingCancel, onSendEmail }: TapeChartProps) {
  // Get data from global context
  // NORMALIZED STATE: Use guestMap/roomMap for O(1) lookups instead of nested objects
  const { rooms, bookings, updateBooking, syncBookingFromBackend, refreshBookings, guestMap, roomMap } = useData();

  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [showMonthPicker, setShowMonthPicker] = useState(false);
  const [localBookings, setLocalBookings] = useState<Booking[]>(bookings);
  const [activeBooking, setActiveBooking] = useState<Booking | null>(null);
  const [dragHasOverlap, setDragHasOverlap] = useState(false); // Track if drag position has overlap
  const [overlapDropZone, setOverlapDropZone] = useState<{ roomId: number; dayIndex: number } | null>(null); // Track which drop zone has overlap
  const [scrollToToday, setScrollToToday] = useState(false);
  const [densityMode, setDensityMode] = useState<DensityMode>(() => {
    const saved = localStorage.getItem('tapechart-density');
    return (saved as DensityMode) || 'comfortable';
  });

  // Drag-to-Create State
  const [isCreatingBooking, setIsCreatingBooking] = useState(false);
  const [createDragStart, setCreateDragStart] = useState<{ roomId: number; dayIndex: number } | null>(null);
  const [createDragPreview, setCreateDragPreview] = useState<{ roomId: number; startDay: number; endDay: number } | null>(null);

  // Context Menu State
  const [contextMenu, setContextMenu] = useState<{ bookingId: number; x: number; y: number } | null>(null);

  // Filter State
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [roomTypeFilter, setRoomTypeFilter] = useState('all');

  // Pending Changes State (for manual confirmation)
  const [pendingBookingId, setPendingBookingId] = useState<number | null>(null);
  const [pendingChange, setPendingChange] = useState<any | null>(null);
  const [showChangeConfirmation, setShowChangeConfirmation] = useState(false);

  // Batch price calculation for all bookings
  const bookingIds = useMemo(() => bookings.map(b => b.id), [bookings]);
  const { priceMap } = useBatchPriceCalculation(bookingIds);

  // KRITISCH: Sync localBookings mit bookings aus Context
  // FIX: Nur auf bookings Changes reagieren, NICHT auf pendingBookingId/pendingChange!
  // Sonst wird beim Reset von pendingBookingId ein Sync getriggert bevor LISTEN/NOTIFY angekommen ist
  useEffect(() => {
    // Only sync if no pending changes (prevents overwriting user edits)
    if (!pendingBookingId && !pendingChange) {
      console.log('╔═══════════════════════════════════════════════════════════════════════════╗');
      console.log('║                    🔄 STATE SYNC: Context → Local                         ║');
      console.log('╚═══════════════════════════════════════════════════════════════════════════╝');
      console.log('📊 CONTEXT BOOKINGS (Source):');
      console.log('   Total Count:', bookings.length);
      console.log('   With Services:', bookings.filter(b => b.services && b.services.length > 0).length);
      console.log('   With Discounts:', bookings.filter(b => b.discounts && b.discounts.length > 0).length);
      console.log('   Status Breakdown:', {
        bestaetigt: bookings.filter(b => b.status === 'bestaetigt').length,
        eingecheckt: bookings.filter(b => b.status === 'eingecheckt').length,
        ausgecheckt: bookings.filter(b => b.status === 'ausgecheckt').length,
        storniert: bookings.filter(b => b.status === 'storniert').length,
      });

      // KRITISCH: Zeige die DATEN jeder Buchung (Check-in/out, Room)
      console.log('\n📋 DETAILLIERTE BOOKING-DATEN IM CONTEXT:');
      bookings.forEach(b => {
        const guest = guestMap.get(b.guest_id);
        const room = roomMap.get(b.room_id);
        console.log(`   📦 Booking ${b.id} (${b.reservierungsnummer}):`);
        console.log(`      Gast: ${guest ? guest.vorname + ' ' + guest.nachname : 'Unbekannt'}`);
        console.log(`      Zimmer: ${room ? room.name : 'ID ' + b.room_id}`);
        console.log(`      Check-in: ${b.checkin_date}`);
        console.log(`      Check-out: ${b.checkout_date}`);
        console.log(`      Preis: ${b.gesamtpreis}€`);
      });

      console.log('\n🔄 SYNCING: localBookings ← bookings');
      setLocalBookings(bookings);
      console.log('✅ localBookings State updated');
      console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
    } else {
      console.log('⏸️  [STATE SYNC] SKIPPED - Pending changes active');
      console.log('   pendingBookingId:', pendingBookingId);
      console.log('   Has pendingChange:', !!pendingChange);
    }
  }, [bookings]); // FIX: NUR bookings in Dependencies! pendingBookingId/pendingChange werden im Effect gelesen
  const chartContainerRef = useRef<HTMLDivElement>(null);

  // Persist density mode to localStorage
  useEffect(() => {
    localStorage.setItem('tapechart-density', densityMode);
  }, [densityMode]);

  // Get current density settings
  const density = DENSITY_SETTINGS[densityMode];

  // Configure drag sensor with distance-based activation (like professional apps)
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,     // Drag activates after 8px movement (like Notion/Trello)
      }
    })
  );

  // Default date range: current month
  const defaultStart = startDate || startOfMonth(currentMonth);
  const defaultEnd = endDate || endOfMonth(currentMonth);

  const days = eachDayOfInterval({ start: defaultStart, end: defaultEnd });

  // Filter bookings
  // NORMALIZED STATE: Pass guestMap and roomMap for O(1) lookups
  const filteredBookings = filterBookings(localBookings, searchQuery, statusFilter, roomTypeFilter, guestMap, roomMap)
    // NUR Buchungen die den aktuellen Monat überlappen
    .filter((b) => {
      const checkin = startOfDay(new Date(b.checkin_date));
      const checkout = startOfDay(new Date(b.checkout_date));
      // Buchung muss Monat überlappen: checkin vor Monatsende UND checkout nach Monatsanfang
      return checkin <= defaultEnd && checkout >= defaultStart;
    });

  // Get unique room types for filter
  const uniqueRoomTypes = getUniqueRoomTypes(rooms);

  // Calculate today's index for column highlighting
  const today = startOfDay(new Date());
  const todayIndex = days.findIndex(day => isSameDay(day, today));

  const goToPreviousMonth = () => {
    setCurrentMonth(prev => subMonths(prev, 1));
  };

  const goToNextMonth = () => {
    setCurrentMonth(prev => addMonths(prev, 1));
  };

  const goToToday = () => {
    setCurrentMonth(new Date());
    setScrollToToday(true);
  };

  // Auto-scroll to today on initial mount
  useEffect(() => {
    // Delay to ensure chart is rendered first
    const timer = setTimeout(() => {
      if (chartContainerRef.current && days.length > 0) {
        const today = startOfDay(new Date());
        const todayIndex = days.findIndex(day => isSameDay(day, today));

        if (todayIndex !== -1) {
          const scrollPosition = todayIndex * density.cellWidth;
          chartContainerRef.current.scrollTo({
            left: scrollPosition,
            behavior: 'auto' // No animation on initial load
          });
        }
      }
    }, 100);

    return () => clearTimeout(timer);
  }, []); // Run only once on mount

  // Scroll to today's date column (triggered by button)
  useEffect(() => {
    if (scrollToToday && chartContainerRef.current) {
      const today = startOfDay(new Date());
      const todayIndex = days.findIndex(day => isSameDay(day, today));

      if (todayIndex !== -1) {
        // Each day column width depends on density mode
        const scrollPosition = todayIndex * density.cellWidth;
        chartContainerRef.current.scrollTo({
          left: scrollPosition,
          behavior: 'smooth'
        });
      }

      setScrollToToday(false);
    }
  }, [scrollToToday, days]);

  const [pickerYear, setPickerYear] = useState(new Date().getFullYear());

  const selectMonth = (month: number) => {
    setCurrentMonth(new Date(pickerYear, month, 1));
    setShowMonthPicker(false);
  };

  const monthsInYear = [
    'Januar', 'Februar', 'März', 'April', 'Mai', 'Juni',
    'Juli', 'August', 'September', 'Oktober', 'November', 'Dezember'
  ];

  const getBookingPosition = (booking: Booking) => {
    const checkin = startOfDay(new Date(booking.checkin_date));
    const checkout = startOfDay(new Date(booking.checkout_date));

    // Calculate full duration
    let startOffset = differenceInDays(checkin, defaultStart);
    let duration = differenceInDays(checkout, checkin) + 1;

    // Check if booking spans across month boundaries
    const continuesFromPreviousMonth = checkin < defaultStart;
    const continuesNextMonth = checkout > defaultEnd;

    // Clip to current month view
    if (continuesFromPreviousMonth) {
      // Booking started before current month - start from day 0
      startOffset = 0;
      // Recalculate duration from month start
      duration = differenceInDays(
        continuesNextMonth ? defaultEnd : checkout,
        defaultStart
      ) + 1;
    } else if (continuesNextMonth) {
      // Booking continues into next month - clip to month end
      duration = differenceInDays(defaultEnd, checkin) + 1;
    }

    // Add padding: 4px on each side
    const padding = 4;

    return {
      left: startOffset * density.cellWidth + padding,
      width: duration * density.cellWidth - (padding * 2),
      isVisible: startOffset >= 0 && startOffset < days.length,
      continuesFromPreviousMonth,
      continuesNextMonth,
      originalCheckin: booking.checkin_date,
      originalCheckout: booking.checkout_date,
    };
  };

  const handleDragStart = (event: DragStartEvent) => {
    const booking = event.active.data.current as Booking;
    const originalPrice = getBookingPrice(priceMap, booking.id);
    const guest = guestMap.get(booking.guest_id);
    const room = roomMap.get(booking.room_id);

    console.log('╔═══════════════════════════════════════════════════════════════════════════╗');
    console.log('║                          🎯 DRAG START EVENT                              ║');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝');
    console.log('📦 BOOKING DETAILS:');
    console.log('   ID:', booking.id);
    console.log('   Reservierungsnummer:', booking.reservierungsnummer);
    console.log('   Gast:', guest ? `${guest.vorname} ${guest.nachname}` : 'Unbekannt');
    console.log('   Zimmer:', room ? `${room.name} (${room.gebaeude_typ})` : 'Unbekannt');
    console.log('   Raum ID:', booking.room_id);
    console.log('   Check-in:', booking.checkin_date);
    console.log('   Check-out:', booking.checkout_date);
    console.log('   Status:', booking.status);
    console.log('   Anzahl Gäste:', booking.anzahl_gaeste);
    console.log('💰 PREISE (VOR DRAG):');
    console.log('   Grundpreis:', originalPrice.grundpreis, '€');
    console.log('   Services:', originalPrice.servicesTotal, '€');
    console.log('   Rabatte:', originalPrice.discountsTotal, '€');
    console.log('   Gesamt:', originalPrice.total, '€');
    console.log('🔍 STATE CHECK:');
    console.log('   localBookings Count:', localBookings.length);
    console.log('   context bookings Count:', bookings.length);
    console.log('   Pending ID:', pendingBookingId);
    console.log('   Has Pending Change:', !!pendingChange);
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');

    setActiveBooking(booking);
  };

  // Helper function to check if two bookings overlap
  const checkOverlap = (booking1Start: Date, booking1End: Date, booking2Start: Date, booking2End: Date): boolean => {
    // Bookings can TOUCH (same dates) but NOT overlap
    // Overlap exists if booking1End > booking2Start AND booking1Start < booking2End
    return booking1End > booking2Start && booking1Start < booking2End;
  };

  const handleDragEnd = (event: DragEndEvent) => {
    console.log('╔═══════════════════════════════════════════════════════════════════════════╗');
    console.log('║                          🏁 DRAG END EVENT                                ║');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝');
    console.log('📦 DROP TARGET:', event.over?.id);
    console.log('📦 ACTIVE BOOKING:', activeBooking ? {
      id: activeBooking.id,
      reservierungsnummer: activeBooking.reservierungsnummer,
      currentRoom: activeBooking.room_id,
      checkin: activeBooking.checkin_date,
      checkout: activeBooking.checkout_date
    } : null);

    if (!event.over || !activeBooking) {
      console.log('❌ DRAG ABGEBROCHEN - Kein Drop-Target oder keine Buchung');
      console.log('   event.over:', !!event.over);
      console.log('   activeBooking:', !!activeBooking);
      console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
      setActiveBooking(null);
      setDragHasOverlap(false);
      setOverlapDropZone(null);
      return;
    }

    const dropData = event.over.data.current as { roomId: number; dayIndex: number };

    console.log('🎯 DROP DATA:', dropData);

    if (!dropData) {
      console.log('❌ KEINE DROP-DATEN - Abbruch');
      console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
      setActiveBooking(null);
      return;
    }

    const { roomId: targetRoomId, dayIndex } = dropData;
    console.log('🎯 TARGET POSITION:');
    console.log('   Ziel-Zimmer ID:', targetRoomId);
    console.log('   Tag-Index:', dayIndex);
    console.log('   Datum:', format(addDays(defaultStart, dayIndex), 'yyyy-MM-dd'));

    // Find the target room
    const targetRoom = rooms.find(r => r.id === targetRoomId);
    if (!targetRoom) {
      console.error('Target room not found:', targetRoomId);
      setActiveBooking(null);
      return;
    }

    // Validate room capacity
    if (activeBooking.anzahl_gaeste > targetRoom.capacity) {
      console.warn(`❌ Kapazitätsfehler: ${activeBooking.anzahl_gaeste} Gäste > ${targetRoom.capacity} Kapazität`);

      // Show error toast
      window.dispatchEvent(new CustomEvent('show-toast', {
        detail: {
          type: 'error',
          message: `Zimmer ${targetRoom.name} hat nur Kapazität für ${targetRoom.capacity} ${targetRoom.capacity === 1 ? 'Person' : 'Personen'}, aber die Buchung hat ${activeBooking.anzahl_gaeste} ${activeBooking.anzahl_gaeste === 1 ? 'Gast' : 'Gäste'}.`,
          duration: 5000
        }
      }));

      setActiveBooking(null);
      setDragHasOverlap(false);
      setOverlapDropZone(null);
      return;
    }

    // Calculate new dates (normalized to start of day)
    const newCheckinDate = startOfDay(addDays(defaultStart, dayIndex));

    // FIX: Use ORIGINAL booking from context (bookings), NOT from localBookings
    // localBookings may have been modified by previous drag/drop operations
    const originalBooking = bookings.find(b => b.id === activeBooking.id) || activeBooking;
    const originalDuration = differenceInDays(
      startOfDay(new Date(originalBooking.checkout_date)),
      startOfDay(new Date(originalBooking.checkin_date))
    );
    const newCheckoutDate = startOfDay(addDays(newCheckinDate, originalDuration));

    // Check for overlaps with other bookings in the same room
    // IMPORTANT: Exclude cancelled bookings from overlap detection (they don't block dates)
    // FIX: Use bookings (context) instead of localBookings for overlap check
    // Context contains the latest server data via LISTEN/NOTIFY
    console.log('\n🔍 OVERLAP DETECTION (handleDragEnd):');
    console.log('   Quelle: bookings (Context) ✓');
    console.log('   Neue Daten:', {
      checkin: format(newCheckinDate, 'yyyy-MM-dd'),
      checkout: format(newCheckoutDate, 'yyyy-MM-dd'),
      targetRoom: targetRoomId
    });
    console.log('   Prüfe gegen', bookings.filter(b => b.room_id === targetRoomId && b.id !== activeBooking.id).length, 'Buchungen im Zielzimmer');

    const hasOverlap = bookings.some(b => {
      if (b.id === activeBooking.id || b.room_id !== targetRoomId) return false;

      // Skip cancelled bookings - they don't block availability
      if (b.status === 'storniert') {
        console.log('   ⏩ Übersprungen (storniert):', b.id, '-', guestMap.get(b.guest_id)?.nachname || 'Unbekannt');
        return false;
      }

      const existingStart = startOfDay(new Date(b.checkin_date));
      const existingEnd = startOfDay(new Date(b.checkout_date));

      const wouldOverlap = checkOverlap(newCheckinDate, newCheckoutDate, existingStart, existingEnd);

      const guest = guestMap.get(b.guest_id);
      console.log(`   ${wouldOverlap ? '❌ OVERLAP' : '✅ OK'}: Booking ${b.id} (${guest?.nachname || 'Unbekannt'})`);
      console.log('      Existing:', format(existingStart, 'yyyy-MM-dd'), '-', format(existingEnd, 'yyyy-MM-dd'));
      console.log('      New:     ', format(newCheckinDate, 'yyyy-MM-dd'), '-', format(newCheckoutDate, 'yyyy-MM-dd'));

      return wouldOverlap;
    });

    console.log('   ERGEBNIS:', hasOverlap ? '❌ OVERLAP GEFUNDEN' : '✅ KEIN OVERLAP');

    if (hasOverlap) {
      console.log('❌ DRAG BLOCKIERT - Overlap mit existierender Buchung!');
      console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
      setActiveBooking(null);
      setDragHasOverlap(false);
      setOverlapDropZone(null);
      return;
    }

    console.log('\n✅ VALIDATION PASSED - Aktualisiere Buchung');
    console.log('📝 ÄNDERUNGEN:');
    console.log('   Zimmer:    ', activeBooking.room_id, '→', targetRoomId);
    console.log('   Check-in:  ', activeBooking.checkin_date, '→', format(newCheckinDate, 'yyyy-MM-dd'));
    console.log('   Check-out: ', activeBooking.checkout_date, '→', format(newCheckoutDate, 'yyyy-MM-dd'));

    // Update booking with new room AND room object
    const updatedBookings = localBookings.map((b) => {
      if (b.id === activeBooking.id) {
        return {
          ...b,
          room_id: targetRoomId,
          room: targetRoom,
          checkin_date: format(newCheckinDate, 'yyyy-MM-dd'),
          checkout_date: format(newCheckoutDate, 'yyyy-MM-dd'),
        };
      }
      return b;
    });

    console.log('Updated bookings:', updatedBookings);
    setLocalBookings(updatedBookings);
    setActiveBooking(null);
    setDragHasOverlap(false);
    setOverlapDropZone(null);

    // Store pending change for manual confirmation
    // FIX: Use ORIGINAL booking from context (bookings) for oldData, NOT from activeBooking (which comes from localBookings)
    const originalBookingForOldData = bookings.find(b => b.id === activeBooking.id) || activeBooking;
    // NORMALIZED STATE: Look up guest from Map
    const activeGuest = guestMap.get(activeBooking.guest_id);
    const changeData = {
      bookingId: activeBooking.id,
      reservierungsnummer: activeBooking.reservierungsnummer,
      guestName: activeGuest ? `${activeGuest.vorname} ${activeGuest.nachname}` : 'Unbekannt',
      roomName: targetRoom.name,
      oldData: pendingChange?.oldData || {
        checkin_date: originalBookingForOldData.checkin_date,
        checkout_date: originalBookingForOldData.checkout_date,
        room_id: originalBookingForOldData.room_id,
        gesamtpreis: getBookingPrice(priceMap, originalBookingForOldData.id).total,
      },
      newData: {
        checkin_date: format(newCheckinDate, 'yyyy-MM-dd'),
        checkout_date: format(newCheckoutDate, 'yyyy-MM-dd'),
        room_id: targetRoomId,
        gesamtpreis: getBookingPrice(priceMap, activeBooking.id).total, // Price will be recalculated after save
      },
    };

    // Set pending state (zeigt Save-Button am Balken)
    console.log('\n💾 PENDING STATE WIRD GESETZT:');
    console.log('   Booking ID:', activeBooking.id);
    console.log('   OLD Data:');
    console.log('      Zimmer:', changeData.oldData.room_id);
    console.log('      Check-in:', changeData.oldData.checkin_date);
    console.log('      Check-out:', changeData.oldData.checkout_date);
    console.log('      Preis:', changeData.oldData.gesamtpreis, '€');
    console.log('   NEW Data:');
    console.log('      Zimmer:', changeData.newData.room_id);
    console.log('      Check-in:', changeData.newData.checkin_date);
    console.log('      Check-out:', changeData.newData.checkout_date);
    console.log('      Preis (vorläufig):', changeData.newData.gesamtpreis, '€');
    console.log('   💡 Grüner SPEICHERN Button sollte jetzt über Balken erscheinen!');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
    setPendingBookingId(activeBooking.id);
    setPendingChange(changeData);
  };

  const handleResize = useCallback((bookingId: number, direction: 'start' | 'end', daysDelta: number) => {
    console.log('╔═══════════════════════════════════════════════════════════════════════════╗');
    console.log('║                          📏 RESIZE EVENT                                  ║');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝');
    console.log('📦 RESIZE DETAILS:');
    console.log('   Booking ID:', bookingId);
    console.log('   Direction:', direction === 'start' ? '⬅️ Von links (Check-in ändern)' : '➡️ Von rechts (Check-out ändern)');
    console.log('   Days Delta:', daysDelta, 'Tage');
    console.log('   Density Mode:', densityMode);
    console.log('   Cell Width:', density.cellWidth, 'px');

    setLocalBookings(prev => {
      const currentBooking = prev.find(b => b.id === bookingId);
      if (!currentBooking) {
        console.log('❌ BOOKING NICHT GEFUNDEN in localBookings');
        console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
        return prev;
      }

      console.log('📋 CURRENT BOOKING (localBookings):');
      console.log('   Check-in:', currentBooking.checkin_date);
      console.log('   Check-out:', currentBooking.checkout_date);
      console.log('   Room:', currentBooking.room_id);

      // FIX: Use ORIGINAL booking data from context (not clipped data from localBookings)
      // This ensures resize works correctly for month-spanning bookings
      const originalBooking = bookings.find(b => b.id === bookingId) || currentBooking;
      console.log('📋 ORIGINAL BOOKING (context):');
      console.log('   Check-in:', originalBooking.checkin_date);
      console.log('   Check-out:', originalBooking.checkout_date);
      console.log('   Source:', bookings.find(b => b.id === bookingId) ? 'Context' : 'Fallback to localBookings');

      const currentCheckin = new Date(originalBooking.checkin_date);
      const currentCheckout = new Date(originalBooking.checkout_date);

      let newCheckin = currentCheckin;
      let newCheckout = currentCheckout;

      console.log('\n🔄 BERECHNUNG:');
      if (direction === 'start') {
        // Resize from left edge - change checkin date
        newCheckin = addDays(currentCheckin, daysDelta);
        console.log('   Neue Check-in:', format(newCheckin, 'yyyy-MM-dd'), `(${daysDelta > 0 ? '+' : ''}${daysDelta} Tage)`);
        console.log('   Check-out bleibt:', format(currentCheckout, 'yyyy-MM-dd'));

        // Prevent invalid range (checkin after checkout)
        if (newCheckin >= currentCheckout) {
          console.log('❌ UNGÜLTIG - Check-in würde nach Check-out sein!');
          console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
          return prev;
        }
      } else {
        // Resize from right edge - change checkout date
        newCheckout = addDays(currentCheckout, daysDelta);
        console.log('   Check-in bleibt:', format(currentCheckin, 'yyyy-MM-dd'));
        console.log('   Neue Check-out:', format(newCheckout, 'yyyy-MM-dd'), `(${daysDelta > 0 ? '+' : ''}${daysDelta} Tage)`);

        // Prevent invalid range (checkout before checkin)
        if (newCheckout <= currentCheckin) {
          console.log('❌ UNGÜLTIG - Check-out würde vor Check-in sein!');
          console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
          return prev;
        }
      }

      const newDuration = differenceInDays(newCheckout, newCheckin);
      console.log('   ✅ Neue Dauer:', newDuration, 'Nächte');

      // Check for overlaps with other bookings in the same room
      // IMPORTANT: Exclude cancelled bookings from overlap detection (they don't block dates)
      console.log('\n🔍 OVERLAP DETECTION (handleResize):');
      console.log('   Quelle: prev (localBookings während setLocalBookings)');
      console.log('   Prüfe gegen', prev.filter(b => b.room_id === currentBooking.room_id && b.id !== bookingId).length, 'Buchungen im selben Zimmer');

      const hasOverlap = prev.some(b => {
        if (b.id === bookingId) return false;
        if (b.room_id !== currentBooking.room_id) return false;
        if (b.status === 'storniert') {
          console.log('   ⏩ Übersprungen (storniert):', b.id);
          return false;
        }

        const wouldOverlap = checkOverlap(
          newCheckin,
          newCheckout,
          new Date(b.checkin_date),
          new Date(b.checkout_date)
        );

        const guest = guestMap.get(b.guest_id);
        console.log(`   ${wouldOverlap ? '❌ OVERLAP' : '✅ OK'}: Booking ${b.id} (${guest?.nachname || 'Unbekannt'})`);
        console.log('      Existing:', b.checkin_date, '-', b.checkout_date);
        console.log('      New:     ', format(newCheckin, 'yyyy-MM-dd'), '-', format(newCheckout, 'yyyy-MM-dd'));

        return wouldOverlap;
      });

      console.log('   ERGEBNIS:', hasOverlap ? '❌ OVERLAP GEFUNDEN' : '✅ KEIN OVERLAP');

      if (hasOverlap) {
        console.log('❌ RESIZE BLOCKIERT - Overlap mit existierender Buchung!');
        console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
        return prev;
      }

      console.log('\n✅ RESIZE ERFOLGREICH:');
      console.log('   ID:', bookingId);
      console.log('   Check-in:  ', currentBooking.checkin_date, '→', format(newCheckin, 'yyyy-MM-dd'));
      console.log('   Check-out: ', currentBooking.checkout_date, '→', format(newCheckout, 'yyyy-MM-dd'));

      const updatedBookings = prev.map(booking => {
        if (booking.id !== bookingId) return booking;

        return {
          ...booking,
          checkin_date: format(newCheckin, 'yyyy-MM-dd'),
          checkout_date: format(newCheckout, 'yyyy-MM-dd'),
        };
      });

      // Store pending change for manual confirmation
      const booking = prev.find(b => b.id === bookingId);
      if (booking) {
        setPendingChange(prevChange => {
          // FIX: Use ORIGINAL booking from context (bookings) for oldData, NOT from localBookings
          // This prevents issues when resizing multiple times without saving
          const originalBooking = bookings.find(b => b.id === bookingId) || currentBooking;
          // NORMALIZED STATE: Look up guest and room from Maps
          const bookingGuest = guestMap.get(booking.guest_id);
          const bookingRoom = roomMap.get(booking.room_id);
          const changeData = {
            bookingId: booking.id,
            reservierungsnummer: booking.reservierungsnummer,
            guestName: bookingGuest ? `${bookingGuest.vorname} ${bookingGuest.nachname}` : 'Unbekannt',
            roomName: bookingRoom?.name || 'Unbekannt',
            oldData: prevChange?.oldData || {
              checkin_date: originalBooking.checkin_date,
              checkout_date: originalBooking.checkout_date,
              room_id: originalBooking.room_id,
              gesamtpreis: getBookingPrice(priceMap, originalBooking.id).total,
            },
            newData: {
              checkin_date: format(newCheckin, 'yyyy-MM-dd'),
              checkout_date: format(newCheckout, 'yyyy-MM-dd'),
              room_id: booking.room_id,
              gesamtpreis: getBookingPrice(priceMap, booking.id).total, // Price will be recalculated after save
            },
          };
          return changeData;
        });

        // Set pending state (zeigt Save-Button am Balken)
        setPendingBookingId(booking.id);
      }

      return updatedBookings;
    });
  }, [bookings, guestMap, roomMap, priceMap, syncBookingFromBackend]);

  // Manual Confirmation Handlers
  const handleManualSave = useCallback(() => {
    console.log('═══════════════════════════════════════');
    console.log('💾 [TapeChart] GRÜNER SPEICHERN BUTTON GEKLICKT!');
    console.log('📦 pendingChange:', JSON.stringify(pendingChange, null, 2));
    console.log('🎬 Öffne jetzt Bestätigungsdialog...');
    console.log('═══════════════════════════════════════');
    setShowChangeConfirmation(true);
  }, [pendingChange]);

  const handleManualDiscard = useCallback(() => {
    console.log('╔═══════════════════════════════════════════════════════════════════════════╗');
    console.log('║                        🗑️  DISCARD CHANGES                                 ║');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝');
    console.log('❌ User hat Änderungen verworfen');
    console.log('🔄 ROLLBACK zu Original-Daten:');
    console.log('   localBookings ← bookings (Context)');
    console.log('   Anzahl Buchungen:', bookings.length);

    // Reset to original bookings
    setLocalBookings(bookings);
    setPendingBookingId(null);
    setPendingChange(null);

    console.log('✅ Rollback abgeschlossen');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
  }, [bookings]);

  const handleConfirmChange = async (sendEmail: boolean, sendInvoice: boolean) => {
    console.log('╔═══════════════════════════════════════════════════════════════════════════╗');
    console.log('║                        💾 SAVE CONFIRMED - BACKEND CALL                   ║');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝');
    console.log('⚙️  OPTIONS:');
    console.log('   E-Mail senden:', sendEmail ? '✅ JA' : '❌ NEIN');
    console.log('   Rechnung senden:', sendInvoice ? '✅ JA' : '❌ NEIN');

    if (!pendingChange) {
      console.error('❌ FEHLER - Kein pendingChange vorhanden!');
      console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
      return;
    }

    console.log('\n📤 BACKEND CALL WIRD VORBEREITET:');
    console.log('   Command: update_booking_dates_and_room_pg');
    console.log('   Booking ID:', pendingChange.bookingId);
    console.log('   Neue Daten:');
    console.log('      Room ID:', pendingChange.newData.room_id);
    console.log('      Check-in:', pendingChange.newData.checkin_date);
    console.log('      Check-out:', pendingChange.newData.checkout_date);

    try {
      const startTime = performance.now();

      // Persist to database with Retry Logic (max 3 attempts with exponential backoff)
      console.log('\n🔄 BACKEND CALL WIRD AUSGEFÜHRT...');
      const updatedBooking = await invokeWithRetry('update_booking_dates_and_room_pg', {
        id: pendingChange.bookingId,
        roomId: pendingChange.newData.room_id,
        checkinDate: pendingChange.newData.checkin_date,
        checkoutDate: pendingChange.newData.checkout_date,
      });

      const endTime = performance.now();
      const duration = Math.round(endTime - startTime);

      console.log('\n✅ BACKEND RESPONSE ERHALTEN (' + duration + 'ms):');
      console.log('   Booking ID:', pendingChange.bookingId);
      console.log('   💰 AKTUALISIERTE PREISE vom Backend:');
      console.log('      Grundpreis:', updatedBooking.grundpreis, '€');
      console.log('      Gesamtpreis:', updatedBooking.gesamtpreis, '€');
      console.log('   📋 Dates bestätigt:');
      console.log('      Check-in:', updatedBooking.checkin_date);
      console.log('      Check-out:', updatedBooking.checkout_date);
      console.log('      Room ID:', updatedBooking.room_id);

      // Vergleiche Preise
      const oldPrice = pendingChange.oldData.gesamtpreis;
      const newPrice = updatedBooking.gesamtpreis;
      const priceDiff = newPrice - oldPrice;
      if (priceDiff !== 0) {
        console.log('   💸 PREISÄNDERUNG:', oldPrice, '€ →', newPrice, '€ (', priceDiff > 0 ? '+' : '', priceDiff, '€)')
      } else {
        console.log('   💰 PREIS UNVERÄNDERT:', newPrice, '€');
      }

      // Update local state with backend response (includes recalculated prices)
      console.log('\n🔄 LOKALER STATE UPDATE (localBookings):');
      console.log('   Merge Backend-Response in localBookings');
      setLocalBookings(prev => {
        const updated = prev.map(b => {
          if (b.id === pendingChange.bookingId) {
            console.log('   📝 Updating Booking', b.id);
            console.log('      Services:', updatedBooking.services?.length || 0);
            console.log('      Discounts:', updatedBooking.discounts?.length || 0);
            return {
              ...b,
              ...updatedBooking,
              // IMPORTANT: Always use backend data for services and discounts
              // to ensure calculated_amount is up-to-date after price recalculation
              services: updatedBooking.services || b.services,
              discounts: updatedBooking.discounts || b.discounts,
            };
          }
          return b;
        });
        console.log('   ✅ localBookings State updated');
        return updated;
      });

      // FIX: Update Context IMMEDIATELY after save (don't wait for LISTEN/NOTIFY)
      // Professional apps (Cal.com, Todoist, ClickUp) do this to prevent visual jumps on tab switch
      console.log('\n🔄 CONTEXT UPDATE (bookings):');
      console.log('   Updating Context sofort (nicht auf LISTEN/NOTIFY warten)');
      syncBookingFromBackend(updatedBooking);
      console.log('   ✅ Context synchronisiert - kein Visual Jump beim Tab-Wechsel');

      // Execute command for undo/redo support
      const command = new UpdateBookingDatesCommand(
        pendingChange.bookingId,
        pendingChange.oldData.checkin_date,
        pendingChange.newData.checkin_date,
        pendingChange.oldData.checkout_date,
        pendingChange.newData.checkout_date,
        pendingChange.oldData.room_id,
        pendingChange.newData.room_id,
        setLocalBookings
      );
      commandManager.executeCommand(command);

      // Booking ID und Daten speichern bevor pendingChange auf null gesetzt wird
      const savedBookingId = pendingChange.bookingId;
      const oldCheckoutDate = pendingChange.oldData.checkout_date;
      const newCheckoutDate = pendingChange.newData.checkout_date;
      const oldCheckinDate = pendingChange.oldData.checkin_date;
      const newCheckinDate = pendingChange.newData.checkin_date;
      const oldRoomId = pendingChange.oldData.room_id;
      const newRoomId = pendingChange.newData.room_id;

      // Reset pending state (Dialog schließt SOFORT)
      console.log('\n🧹 PENDING STATE RESET:');
      console.log('   pendingBookingId: → null');
      console.log('   pendingChange: → null');
      console.log('   showChangeConfirmation: → false');
      console.log('   💡 Dialog schließt jetzt sofort!');
      setPendingBookingId(null);
      setPendingChange(null);
      setShowChangeConfirmation(false);

      // 🔄 SYNC zu Turso (Mobile App) - NUR wenn SPEICHERN Button geklickt wurde!
      const checkoutChanged = oldCheckoutDate !== newCheckoutDate;
      const checkinChanged = oldCheckinDate !== newCheckinDate;
      const roomChanged = oldRoomId !== newRoomId;

      if (checkoutChanged || checkinChanged || roomChanged) {
        if (checkoutChanged) {
          console.log('🔄 [TapeChart] Checkout-Datum geändert:', oldCheckoutDate, '→', newCheckoutDate);
        }
        if (checkinChanged) {
          console.log('🔄 [TapeChart] Checkin-Datum geändert:', oldCheckinDate, '→', newCheckinDate);
        }
        if (roomChanged) {
          console.log('🔄 [TapeChart] Zimmer geändert:', oldRoomId, '→', newRoomId);
        }

        // Loading Toast
        const syncToast = toast.loading('☁️ Synchronisiere Putzplan...', {
          style: {
            background: '#1e293b',
            color: '#fff',
            borderRadius: '0.75rem',
            padding: '1rem',
          }
        });

        // 🔥 KRITISCH: 2-Schritt Sync Strategie (wie Profis es machen!)
        // Schritt 1: DELETE alle Tasks dieser Booking ID (Booking-Level DELETE)
        // Schritt 2: Vollständiger Sync (sync_week_ahead) erstellt neue Tasks
        //
        // Problem das wir lösen:
        // - Booking #24 war auf Tag 20-25
        // - User verkürzt auf Tag 20-22
        // - Alte "occupied" Tasks auf Tag 23-25 bleiben in Mobile App
        //
        // Lösung: Lösche ALLE Tasks für Booking #24, dann sync neu
        console.log('🔄 [TapeChart] 2-Schritt Mobile App Sync: DELETE Booking Tasks → Full Sync');

        // Schritt 1: DELETE alle Tasks dieser Booking
        console.log('🗑️  [TapeChart] Schritt 1: Lösche alle Tasks für Booking #' + savedBookingId);
        invoke('delete_booking_tasks', { bookingId: savedBookingId })
          .then(() => {
            console.log('✅ [TapeChart] Booking Tasks gelöscht, starte Full Sync...');

            // Schritt 2: Vollständiger Sync
            return invoke('sync_week_ahead');
          })
          .then((result: string) => {
            console.log('✅ [TapeChart] Vollständiger Sync erfolgreich:', result);
            toast.success('✅ Putzplan aktualisiert', { id: syncToast });
          })
          .catch((error: any) => {
            console.error('❌ [TapeChart] Sync fehlgeschlagen:', error);
            toast.error('❌ Putzplan-Sync fehlgeschlagen', { id: syncToast });
          });
      } else {
        console.log('⚠️ [TapeChart] Keine Änderung - Daten unverändert (kein Datum-/Zimmer-Wechsel)');
      }

      // Email und Rechnung im HINTERGRUND erstellen (nicht-blockierend)
      // Dialog ist bereits geschlossen, User sieht keine Wartezeit
      if (sendInvoice || sendEmail) {
        console.log('📧 [TapeChart] Starting background email/invoice processing:', { sendEmail, sendInvoice });

        // Fire-and-forget: Promise läuft im Hintergrund mit Retry Logic
        (async () => {
          // Show loading toast
          const toastId = toast.loading(
            sendInvoice && sendEmail
              ? 'Rechnung wird erstellt und per Email versendet...'
              : sendInvoice
              ? 'Rechnung wird erstellt...'
              : 'Email wird versendet...'
          );

          try {
            if (sendInvoice && sendEmail) {
              // Rechnung erstellen UND per Email senden (mit Retry)
              console.log('📄 [TapeChart] Creating invoice PDF and sending via email...');
              await invokeWithRetry('send_invoice_email_command', { bookingId: savedBookingId });
              console.log('✅ [TapeChart] Invoice created and sent via email');
              toast.success('Rechnung erstellt und per Email versendet', { id: toastId });
            } else if (sendInvoice) {
              // Nur Rechnung erstellen (kein Email, mit Retry)
              console.log('📄 [TapeChart] Creating invoice PDF...');
              await invokeWithRetry('generate_invoice_pdf_command', { bookingId: savedBookingId });
              console.log('✅ [TapeChart] Invoice PDF created');
              toast.success('Rechnung erfolgreich erstellt', { id: toastId });
            } else if (sendEmail) {
              // Nur Info-Email senden (keine Rechnung, mit Retry)
              console.log('📧 [TapeChart] Sending change notification email...');
              await invokeWithRetry('send_confirmation_email_command', { bookingId: savedBookingId });
              console.log('✅ [TapeChart] Change notification email sent');
              toast.success('Email erfolgreich versendet', { id: toastId });
            }
          } catch (emailError) {
            console.error('⚠️ [TapeChart] Email/Invoice error (non-fatal):', emailError);
            toast.error('Fehler beim Erstellen/Versenden: ' + emailError, { id: toastId });
          }
        })();
      }
    } catch (error) {
      console.error('❌ [TapeChart] Failed to save change:', error);
      // Rollback UI to original state
      setLocalBookings(bookings);
      setPendingBookingId(null);
      setPendingChange(null);
      setShowChangeConfirmation(false);
      toast.error('Fehler beim Speichern: ' + error);
    }
  };

  const handleDiscardChange = () => {
    console.log('╔═══════════════════════════════════════════════════════════════════════════╗');
    console.log('║                  🗑️  DISCARD FROM DIALOG                                  ║');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝');
    console.log('❌ User hat im Bestätigungsdialog ABBRECHEN geklickt');
    console.log('🔄 ROLLBACK zu Original-Daten');

    setLocalBookings(bookings);
    setPendingBookingId(null);
    setPendingChange(null);
    setShowChangeConfirmation(false);

    console.log('✅ Dialog geschlossen, Änderungen verworfen');
    console.log('╚═══════════════════════════════════════════════════════════════════════════╝\n');
  };

  // Context Menu Handler
  const handleContextMenu = useCallback((bookingId: number, x: number, y: number) => {
    setContextMenu({ bookingId, x, y });
  }, []);

  const contextMenuItems: ContextMenuItem[] = contextMenu ? [
    {
      icon: Edit2,
      label: 'Bearbeiten',
      onClick: () => {
        console.log('🖱️ [ContextMenu] Bearbeiten clicked', { bookingId: contextMenu.bookingId, hasHandler: !!onBookingEdit });
        if (onBookingEdit) {
          onBookingEdit(contextMenu.bookingId);
        } else {
          console.error('❌ onBookingEdit handler missing!');
        }
        setContextMenu(null);
      },
    },
    {
      icon: Mail,
      label: 'Email senden',
      onClick: () => {
        console.log('🖱️ [ContextMenu] Email clicked', { bookingId: contextMenu.bookingId, hasHandler: !!onSendEmail });
        if (onSendEmail) {
          onSendEmail(contextMenu.bookingId);
        } else {
          console.error('❌ onSendEmail handler missing!');
        }
        setContextMenu(null);
      },
    },
    {
      icon: XCircle,
      label: 'Stornieren',
      variant: 'danger' as const,
      onClick: () => {
        if (onBookingCancel) {
          onBookingCancel(contextMenu.bookingId);
        }
        setContextMenu(null);
      },
    },
  ] : [];

  // Drag-to-Create Handlers
  const handleCreateDragStart = useCallback((roomId: number, dayIndex: number) => {
    console.log('🎨 Drag-to-Create START:', { roomId, dayIndex });
    setIsCreatingBooking(true);
    setCreateDragStart({ roomId, dayIndex });
    setCreateDragPreview({ roomId, startDay: dayIndex, endDay: dayIndex });
  }, []);

  const handleCreateDragMove = useCallback((roomId: number, dayIndex: number) => {
    if (!isCreatingBooking || !createDragStart) return;

    // Only allow dragging within same room
    if (roomId === createDragStart.roomId) {
      const startDay = Math.min(createDragStart.dayIndex, dayIndex);
      const endDay = Math.max(createDragStart.dayIndex, dayIndex);
      setCreateDragPreview({ roomId, startDay, endDay });
    }
  }, [isCreatingBooking, createDragStart]);

  const handleCreateDragEnd = useCallback(() => {
    if (!isCreatingBooking || !createDragStart || !createDragPreview || !onCreateBooking) {
      setIsCreatingBooking(false);
      setCreateDragStart(null);
      setCreateDragPreview(null);
      return;
    }

    console.log('🎨 Drag-to-Create END:', createDragPreview);

    // Calculate dates
    // FIX: endDay ist der letzte Aufenthaltstag (inklusiv in Preview)
    // checkout_date = Tag NACH dem letzten Aufenthaltstag (Standard Hotel-Logik)
    // Beispiel: Drag von Tag 0 bis Tag 1 (2 Zellen) = 1 Nacht
    //   - checkin_date = Tag 0
    //   - checkout_date = Tag 1 (KEIN +1, weil endDay schon der Abreisetag ist!)
    const startDate = format(addDays(defaultStart, createDragPreview.startDay), 'yyyy-MM-dd');
    const endDate = format(addDays(defaultStart, createDragPreview.endDay), 'yyyy-MM-dd');

    // Call create callback
    onCreateBooking(createDragPreview.roomId, startDate, endDate);

    // Reset state
    setIsCreatingBooking(false);
    setCreateDragStart(null);
    setCreateDragPreview(null);
  }, [isCreatingBooking, createDragStart, createDragPreview, onCreateBooking, defaultStart]);

  return (
    <DndContext
      sensors={sensors}
      onDragStart={handleDragStart}
      onDragOver={(event: DragOverEvent) => {
        if (!event.over || !activeBooking) {
          setDragHasOverlap(false);
          setOverlapDropZone(null);
          return;
        }

        const dropData = event.over.data.current as { roomId: number; dayIndex: number };
        if (!dropData) {
          setDragHasOverlap(false);
          setOverlapDropZone(null);
          return;
        }

        const { roomId: targetRoomId, dayIndex } = dropData;
        const newCheckinDate = startOfDay(addDays(defaultStart, dayIndex));

        // FIX: Use ORIGINAL booking from context (bookings), NOT from localBookings
        const originalBooking = bookings.find(b => b.id === activeBooking.id) || activeBooking;
        const originalDuration = differenceInDays(
          startOfDay(new Date(originalBooking.checkout_date)),
          startOfDay(new Date(originalBooking.checkin_date))
        );
        const newCheckoutDate = startOfDay(addDays(newCheckinDate, originalDuration));

        // Check if this would overlap
        // IMPORTANT: Exclude cancelled bookings from overlap detection (they don't block dates)
        // FIX: Use bookings (context) for consistent overlap detection, NOT localBookings
        console.log('🔍 [DRAG OVER] Overlap Check (Live Preview):');
        console.log('   Quelle: bookings (Context) ✓');
        console.log('   Target Room:', targetRoomId, 'Day:', dayIndex);
        console.log('   Preview Dates:', format(newCheckinDate, 'yyyy-MM-dd'), '-', format(newCheckoutDate, 'yyyy-MM-dd'));

        const hasOverlap = bookings.some(b => {
          if (b.id === activeBooking.id || b.room_id !== targetRoomId) return false;

          // Skip cancelled bookings - they don't block availability
          if (b.status === 'storniert') return false;

          const existingStart = startOfDay(new Date(b.checkin_date));
          const existingEnd = startOfDay(new Date(b.checkout_date));

          const wouldOverlap = checkOverlap(newCheckinDate, newCheckoutDate, existingStart, existingEnd);

          if (wouldOverlap) {
            const guest = guestMap.get(b.guest_id);
            console.log('   ❌ Overlap mit Booking', b.id, `(${guest?.nachname || 'Unbekannt'})`);
          }

          return wouldOverlap;
        });

        console.log('   PREVIEW RESULT:', hasOverlap ? '❌ RED (Overlap)' : '✅ GREEN (OK)');

        setDragHasOverlap(hasOverlap);
        setOverlapDropZone(hasOverlap ? { roomId: targetRoomId, dayIndex } : null);
      }}
      onDragEnd={handleDragEnd}
      collisionDetection={pointerWithin}
    >
      <div className="w-full h-full flex flex-col bg-gradient-to-br from-slate-50 to-slate-100">
        {/* Navigation Bar - Kompakt */}
        <div className="bg-gradient-to-r from-slate-700 to-slate-800 border-b border-slate-600 px-4 py-2 shadow-xl flex items-center justify-center relative">
          <div className="flex items-center gap-2">
            <button
              onClick={goToPreviousMonth}
              className="bg-slate-600/50 hover:bg-slate-600 backdrop-blur-sm text-white px-3 py-2 rounded-lg font-semibold transition-all shadow-lg hover:shadow-xl flex items-center gap-1.5 border border-slate-500/30"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M15 19l-7-7 7-7" />
              </svg>
            </button>

            {/* Month Picker */}
            <div className="relative">
              <button
                onClick={() => setShowMonthPicker(!showMonthPicker)}
                className="group bg-slate-700/80 hover:bg-slate-600/90 backdrop-blur-md text-white px-4 py-2 rounded-lg font-bold text-base shadow-lg border border-slate-500/40 hover:border-slate-400/60 min-w-[240px] text-center transition-all flex items-center justify-between gap-2"
              >
                <svg className="w-4 h-4 text-slate-400 group-hover:text-white transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                <span className="flex-1 px-2">{format(currentMonth, 'MMMM yyyy', { locale: de })}</span>
                <svg className={cn(
                  "w-4 h-4 text-slate-400 group-hover:text-white transition-all",
                  showMonthPicker && "rotate-180"
                )} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M19 9l-7 7-7-7" />
                </svg>
              </button>

              {/* Month Picker Dropdown */}
              {showMonthPicker && (
                <div className="absolute top-full mt-2 left-1/2 -translate-x-1/2 bg-slate-800 rounded-xl shadow-2xl border-2 border-slate-600 p-5 z-[60] w-[380px]">
                  {/* Year Navigation */}
                  <div className="flex items-center justify-between mb-4 pb-4 border-b border-slate-600">
                    <button
                      onClick={() => setPickerYear(pickerYear - 1)}
                      className="bg-slate-700 hover:bg-slate-600 text-white p-2 rounded-lg transition-all"
                    >
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M15 19l-7-7 7-7" />
                      </svg>
                    </button>

                    <div className="text-white font-bold text-xl">{pickerYear}</div>

                    <button
                      onClick={() => setPickerYear(pickerYear + 1)}
                      className="bg-slate-700 hover:bg-slate-600 text-white p-2 rounded-lg transition-all"
                    >
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 5l7 7-7 7" />
                      </svg>
                    </button>
                  </div>

                  {/* Month Grid */}
                  <div className="grid grid-cols-3 gap-2">
                    {monthsInYear.map((monthName, idx) => {
                      const isSelected =
                        pickerYear === currentMonth.getFullYear() &&
                        idx === currentMonth.getMonth();
                      return (
                        <button
                          key={idx}
                          onClick={() => selectMonth(idx)}
                          className={cn(
                            "px-4 py-3 rounded-lg font-semibold transition-all text-sm",
                            isSelected
                              ? "bg-blue-500 text-white shadow-lg scale-105"
                              : "bg-slate-700 hover:bg-slate-600 text-slate-200 hover:text-white hover:scale-105"
                          )}
                        >
                          {monthName}
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>

            <button
              onClick={goToNextMonth}
              className="bg-slate-600/50 hover:bg-slate-600 backdrop-blur-sm text-white px-3 py-2 rounded-lg font-semibold transition-all shadow-lg hover:shadow-xl flex items-center gap-1.5 border border-slate-500/30"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>

          {/* Right Side: Density Mode + Heute Button */}
          <div className="absolute right-4 flex items-center gap-2">
            {/* Density Mode Toggle */}
            <div className="flex items-center gap-2 bg-slate-700/80 backdrop-blur-md px-3 py-2 rounded-xl border-2 border-slate-500/40">
              <button
                onClick={() => setDensityMode('compact')}
                className={cn(
                  "px-3 py-1.5 rounded-lg font-semibold text-xs transition-all",
                  densityMode === 'compact'
                    ? "bg-blue-500 text-white shadow-lg"
                    : "text-slate-300 hover:text-white hover:bg-slate-600"
                )}
                title="Kompakt"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                </svg>
              </button>
              <button
                onClick={() => setDensityMode('comfortable')}
                className={cn(
                  "px-3 py-1.5 rounded-lg font-semibold text-xs transition-all",
                  densityMode === 'comfortable'
                    ? "bg-blue-500 text-white shadow-lg"
                    : "text-slate-300 hover:text-white hover:bg-slate-600"
                )}
                title="Komfortabel"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
                </svg>
              </button>
              <button
                onClick={() => setDensityMode('spacious')}
                className={cn(
                  "px-3 py-1.5 rounded-lg font-semibold text-xs transition-all",
                  densityMode === 'spacious'
                    ? "bg-blue-500 text-white shadow-lg"
                    : "text-slate-300 hover:text-white hover:bg-slate-600"
                )}
                title="Weitläufig"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 12h16" />
                </svg>
              </button>
            </div>

            <button
              onClick={goToToday}
              className="bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-600 hover:to-emerald-700 text-white py-2 rounded-lg font-bold text-sm transition-all shadow-lg flex items-center gap-2 border border-emerald-400/40 hover:border-emerald-300/60 px-3"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              <span>Heute</span>
            </button>
          </div>
        </div>

        {/* Filter Bar */}
        <TapeChartFilters
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          roomTypeFilter={roomTypeFilter}
          onRoomTypeFilterChange={setRoomTypeFilter}
          availableRoomTypes={uniqueRoomTypes}
        />

        {/* Chart Container */}
        <div ref={chartContainerRef} className="flex-1 overflow-auto">
          <div className="relative">
            {/* Header */}
            <div className="sticky top-0 z-50 bg-white">
              <div className="flex shadow-xl" style={{ minWidth: `${SIDEBAR_WIDTH + (days.length * density.cellWidth)}px` }}>
                {/* Top-left corner */}
                <div
                  className="sticky left-0 z-40 bg-gradient-to-br from-slate-800 to-slate-900 flex items-center justify-center font-bold text-white text-lg shadow-2xl box-border"
                  style={{
                    width: `${SIDEBAR_WIDTH}px`,
                    height: `${density.headerHeight}px`,
                    minWidth: `${SIDEBAR_WIDTH}px`,
                    maxWidth: `${SIDEBAR_WIDTH}px`,
                  }}
                >
                  <div className="flex items-center gap-2">
                    <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                    </svg>
                    <span>Zimmer</span>
                  </div>
                </div>

                {/* Date headers */}
                <div className="flex">
                  {days.map((day, idx) => {
                    const isToday = todayIndex === idx;
                    return (
                      <div
                        key={idx}
                        className={cn(
                          "flex flex-col items-center justify-center transition-all hover:bg-slate-100 border-r border-slate-200 box-border",
                          // Today Highlight (sehr deutlich)
                          isToday && 'bg-emerald-100',
                          // Weekend (nur wenn nicht heute)
                          !isToday && (format(day, 'i') === '6' || format(day, 'i') === '7')
                            ? 'bg-gradient-to-b from-blue-50 to-blue-100'
                            : !isToday && 'bg-white'
                        )}
                        style={{
                          width: `${density.cellWidth}px`,
                          height: `${density.headerHeight}px`,
                          minWidth: `${density.cellWidth}px`,
                          maxWidth: `${density.cellWidth}px`,
                        }}
                      >
                        <div className={cn("font-medium text-slate-500 uppercase tracking-wider", density.fontSize)}>
                          {format(day, 'EEE', { locale: de })}
                        </div>
                        <div className={cn("font-bold text-slate-800 my-1", densityMode === 'compact' ? 'text-lg' : densityMode === 'comfortable' ? 'text-2xl' : 'text-3xl')}>
                          {format(day, 'd')}
                        </div>
                        <div className={cn("font-semibold text-slate-600 uppercase", density.fontSize)}>
                          {format(day, 'MMM', { locale: de })}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>

            {/* Rows */}
            {/* ✅ SORTIERUNG: Nach Ort (Location) + Name (natürlich/numerisch) */}
            {[...rooms].sort((a, b) => {
              // Erst nach Ort sortieren
              if (a.ort !== b.ort) {
                return a.ort.localeCompare(b.ort);
              }
              // Dann innerhalb des Orts nach Name (natürliche Sortierung: 1, 2, 10 statt 1, 10, 2)
              return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
            }).map((room, roomIdx) => {
              const roomBookings = filteredBookings.filter((b) => b.room_id === room.id);

              return (
              <div key={room.id} className="flex transition-all group" style={{ minWidth: `${SIDEBAR_WIDTH + (days.length * density.cellWidth)}px` }}>
                {/* Room sidebar */}
                <div
                  className={cn("sticky left-0 z-20 bg-gradient-to-r from-slate-100 to-slate-50 flex flex-col justify-center shadow-md group-hover:shadow-lg transition-all box-border select-none", density.padding)}
                  style={{
                    width: `${SIDEBAR_WIDTH}px`,
                    height: `${density.rowHeight}px`,
                    minWidth: `${SIDEBAR_WIDTH}px`,
                    maxWidth: `${SIDEBAR_WIDTH}px`,
                    userSelect: 'none',
                    WebkitUserSelect: 'none',
                  }}
                >
                  <div className={cn("font-bold text-slate-800 mb-1", density.fontSize)}>{room.name}</div>
                  <div className={cn("flex items-center gap-2 text-slate-600", density.fontSize)}>
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                    </svg>
                    <span className="font-medium">{room.gebaeude_typ} • {room.capacity}P</span>
                  </div>
                  <div className={cn("flex items-center gap-2 text-slate-600", density.fontSize)}>
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    <span className="font-medium">{room.ort}</span>
                  </div>
                </div>

                {/* Timeline grid */}
                <div className="relative flex">
                  {days.map((day, dayIdx) => {
                    const isWeekend = format(day, 'i') === '6' || format(day, 'i') === '7';
                    const isToday = todayIndex === dayIdx;
                    const hasOverlapHere = overlapDropZone?.roomId === room.id && overlapDropZone?.dayIndex === dayIdx;
                    const isInCreatePreview = createDragPreview?.roomId === room.id &&
                                              dayIdx >= (createDragPreview?.startDay ?? 0) &&
                                              dayIdx <= (createDragPreview?.endDay ?? 0);
                    return (
                      <DroppableCell
                        key={dayIdx}
                        roomId={room.id}
                        dayIndex={dayIdx}
                        isWeekend={isWeekend}
                        isToday={isToday}
                        isEvenRow={roomIdx % 2 === 0}
                        cellWidth={density.cellWidth}
                        rowHeight={density.rowHeight}
                        hasOverlap={hasOverlapHere}
                        isCreateDragPreview={isInCreatePreview}
                        onCreateDragStart={handleCreateDragStart}
                        onCreateDragMove={handleCreateDragMove}
                        onCreateDragEnd={handleCreateDragEnd}
                      />
                    );
                  })}

                  {/* Bookings */}
                  {filteredBookings
                    .filter((b) => b.room_id === room.id)
                    .map((booking) => {
                      const pos = getBookingPosition(booking);
                      if (!pos.isVisible) return null;

                      // NORMALIZED STATE: Look up guest from Map
                      const guest = guestMap.get(booking.guest_id);
                      const guestName = guest ? `${guest.vorname} ${guest.nachname}` : 'Unbekannt';

                      return (
                        <DraggableBooking
                          key={booking.id}
                          booking={booking}
                          position={pos}
                          rowHeight={density.rowHeight}
                          cellWidth={density.cellWidth}
                          onResize={handleResize}
                          onClick={onBookingClick}
                          onContextMenu={handleContextMenu}
                          isPending={pendingBookingId === booking.id}
                          onManualSave={handleManualSave}
                          onManualDiscard={handleManualDiscard}
                          continuesFromPreviousMonth={pos.continuesFromPreviousMonth}
                          continuesNextMonth={pos.continuesNextMonth}
                          originalCheckin={pos.originalCheckin}
                          originalCheckout={pos.originalCheckout}
                          guestName={guestName}
                        />
                      );
                    })}
                </div>
              </div>
            );
            })}
          </div>

          {/* Status Filter Footer (Compact) */}
          <div className="sticky bottom-0 left-0 right-0 z-30 bg-gradient-to-r from-slate-800 to-slate-900 border-t-2 border-slate-700 px-6 py-3 shadow-2xl">
            <div className="flex gap-4 justify-center items-center flex-wrap">
              <div className="text-slate-400 font-semibold text-xs uppercase tracking-wider mr-2">Status:</div>

              {/* All Status Button */}
              <button
                onClick={() => setStatusFilter('all')}
                className={cn(
                  "flex items-center gap-2 px-3 py-1.5 rounded-lg border-2 transition-all",
                  statusFilter === 'all'
                    ? "bg-blue-500 border-blue-400 text-white scale-105"
                    : "bg-slate-700 border-slate-600 text-slate-300 hover:bg-slate-600 hover:border-slate-500"
                )}
              >
                <div className="w-5 h-5 rounded border-2 border-current opacity-70" />
                <span className="text-xs font-semibold">Alle</span>
              </button>

              {/* Individual Status Buttons */}
              {Object.entries(STATUS_COLORS).map(([status, colors]) => {
                // Special case for 'stiftungsfall': count bookings where ist_stiftungsfall === true
                const count = status === 'stiftungsfall'
                  ? localBookings.filter(b => b.ist_stiftungsfall).length
                  : localBookings.filter(b => b.status === status).length;
                const isActive = statusFilter === status;

                return (
                  <button
                    key={status}
                    onClick={() => setStatusFilter(status)}
                    className={cn(
                      "flex items-center gap-2 px-3 py-1.5 rounded-lg border-2 transition-all",
                      isActive
                        ? "scale-105 shadow-lg"
                        : "hover:scale-105",
                      isActive ? colors.bg : "bg-slate-700",
                      isActive ? colors.border : "border-slate-600",
                      isActive ? "text-white" : "text-slate-300 hover:text-white"
                    )}
                  >
                    <div className={cn(
                      "w-5 h-5 rounded border-2",
                      isActive ? "border-white" : colors.border
                    )} />
                    <span className="text-xs font-semibold capitalize">
                      {status}
                    </span>
                    <span className={cn(
                      "ml-1 text-xs px-1.5 py-0.5 rounded",
                      isActive ? "bg-white/20" : "bg-slate-600"
                    )}>
                      {count}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>
        </div>

        {/* Drag Overlay */}
        <DragOverlay dropAnimation={null}>
          {activeBooking ? (
            (() => {
              const pos = getBookingPosition(activeBooking);
              // NORMALIZED STATE: Look up guest from Map
              const guest = guestMap.get(activeBooking.guest_id);
              const guestName = guest ? `${guest.vorname} ${guest.nachname}` : 'Unbekannt';
              return (
                <DraggableBooking
                  booking={activeBooking}
                  position={pos}
                  rowHeight={density.rowHeight}
                  cellWidth={density.cellWidth}
                  isOverlay={true}
                  continuesFromPreviousMonth={pos.continuesFromPreviousMonth}
                  continuesNextMonth={pos.continuesNextMonth}
                  originalCheckin={pos.originalCheckin}
                  originalCheckout={pos.originalCheckout}
                  guestName={guestName}
                />
              );
            })()
          ) : null}
        </DragOverlay>

        {/* Context Menu */}
        {contextMenu && (
          <ContextMenu
            x={contextMenu.x}
            y={contextMenu.y}
            items={contextMenuItems}
            onClose={() => setContextMenu(null)}
          />
        )}

        {/* Change Confirmation Dialog */}
        {showChangeConfirmation && pendingChange && (
          <ChangeConfirmationDialog
            change={pendingChange}
            onConfirm={handleConfirmChange}
            onDiscard={handleDiscardChange}
          />
        )}
      </div>
    </DndContext>
  );
}