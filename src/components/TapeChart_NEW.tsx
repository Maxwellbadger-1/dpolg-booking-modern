import { useState, useRef, useEffect } from 'react';
import { format, addDays, differenceInDays, startOfMonth, endOfMonth, eachDayOfInterval, addMonths, subMonths, startOfDay, isSameDay, isToday } from 'date-fns';
import { de } from 'date-fns/locale';
import { cn } from '../lib/utils';
import {
  DndContext,
  DragEndEvent,
  DragOverlay,
  useDraggable,
  useDroppable,
  DragStartEvent,
  pointerWithin,
  DragOverEvent,
} from '@dnd-kit/core';
import { CSS } from '@dnd-kit/utilities';

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  ort: string;
}

interface Guest {
  vorname: string;
  nachname: string;
}

interface Booking {
  id: number;
  room_id: number;
  reservierungsnummer: string;
  checkin_date: string;
  checkout_date: string;
  status: string;
  room: Room;
  guest: Guest;
}

interface TapeChartProps {
  rooms: Room[];
  bookings: Booking[];
  startDate?: Date;
  endDate?: Date;
}

const STATUS_COLORS: Record<string, { bg: string; border: string; text: string; shadow: string }> = {
  reserviert: {
    bg: 'bg-gradient-to-r from-blue-500 to-blue-600',
    border: 'border-blue-700',
    text: 'text-white',
    shadow: 'shadow-lg shadow-blue-500/50'
  },
  bestaetigt: {
    bg: 'bg-gradient-to-r from-emerald-500 to-emerald-600',
    border: 'border-emerald-700',
    text: 'text-white',
    shadow: 'shadow-lg shadow-emerald-500/50'
  },
  eingecheckt: {
    bg: 'bg-gradient-to-r from-purple-500 to-purple-600',
    border: 'border-purple-700',
    text: 'text-white',
    shadow: 'shadow-lg shadow-purple-500/50'
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
};

// Density settings
type DensityMode = 'compact' | 'normal' | 'comfortable';

const DENSITY_SETTINGS = {
  compact: {
    cellWidth: 80,
    rowHeight: 60,
    headerHeight: 80,
    sidebarWidth: 180,
  },
  normal: {
    cellWidth: 120,
    rowHeight: 80,
    headerHeight: 100,
    sidebarWidth: 220,
  },
  comfortable: {
    cellWidth: 160,
    rowHeight: 100,
    headerHeight: 120,
    sidebarWidth: 260,
  },
};

// Draggable Booking Component with Resize Handles
function DraggableBooking({
  booking,
  position,
  rowHeight,
  isOverlay = false,
  onResize,
  onDragStartCustom,
}: {
  booking: Booking;
  position: { left: number; width: number };
  rowHeight: number;
  isOverlay?: boolean;
  onResize?: (bookingId: number, direction: 'left' | 'right', days: number) => void;
  onDragStartCustom?: (bookingId: number, x: number) => void;
}) {
  const [isHovered, setIsHovered] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const [resizePreview, setResizePreview] = useState<{ left: number; width: number } | null>(null);

  const { attributes, listeners, setNodeRef, isDragging, transform } = useDraggable({
    id: `booking-${booking.id}`,
    data: booking,
  });

  const colors = STATUS_COLORS[booking.status] || STATUS_COLORS.reserviert;

  // Handle resize start
  const handleResizeStart = (e: React.MouseEvent, direction: 'left' | 'right') => {
    e.stopPropagation();
    setIsResizing(true);

    const startX = e.clientX;
    const startLeft = position.left;
    const startWidth = position.width;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.clientX - startX;
      const cellWidth = 120; // Will be dynamic based on density
      const daysDelta = Math.round(deltaX / cellWidth);

      if (direction === 'left') {
        const newLeft = startLeft + daysDelta * cellWidth;
        const newWidth = startWidth - daysDelta * cellWidth;
        if (newWidth > cellWidth * 0.8) {
          setResizePreview({ left: newLeft, width: newWidth });
        }
      } else {
        const newWidth = startWidth + daysDelta * cellWidth;
        if (newWidth > cellWidth * 0.8) {
          setResizePreview({ left: startLeft, width: newWidth });
        }
      }
    };

    const handleMouseUp = (upEvent: MouseEvent) => {
      setIsResizing(false);
      const deltaX = upEvent.clientX - startX;
      const cellWidth = 120;
      const daysDelta = Math.round(deltaX / cellWidth);

      if (daysDelta !== 0 && onResize) {
        onResize(booking.id, direction, daysDelta);
      }

      setResizePreview(null);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  // Handle drag start
  const handleMouseDown = (e: React.MouseEvent) => {
    if (onDragStartCustom) {
      onDragStartCustom(booking.id, e.clientX);
    }
  };

  const displayPosition = resizePreview || position;

  const style = isOverlay ? {
    width: `${position.width}px`,
    height: `${rowHeight - 16}px`,
  } : {
    left: `${displayPosition.left}px`,
    width: `${displayPosition.width}px`,
    top: '8px',
    bottom: '8px',
    transform: CSS.Transform.toString(transform),
    transition: isResizing ? 'none' : 'left 100ms ease-out, width 100ms ease-out',
  };

  return (
    <div
      ref={setNodeRef}
      {...attributes}
      onMouseEnter={() => !isResizing && setIsHovered(true)}
      onMouseLeave={() => !isResizing && setIsHovered(false)}
      className={cn(
        isOverlay ? "rounded-xl border-2" : "absolute rounded-xl border-2",
        "flex items-center px-3 relative",
        !isOverlay && !isResizing && !isDragging && "hover:scale-[1.02] transition-all duration-200",
        "backdrop-blur-sm",
        isDragging && !isOverlay && "opacity-0",
        isResizing && "border-dashed border-4 ring-2 ring-white/50 ring-inset shadow-2xl z-50",
        colors.bg,
        colors.border,
        colors.text,
        colors.shadow
      )}
      style={style}
      title={`${booking.guest.vorname} ${booking.guest.nachname}\n${booking.reservierungsnummer}\n${booking.checkin_date} - ${booking.checkout_date}`}
    >
      {/* Left Resize Handle */}
      {!isOverlay && onResize && (
        <div
          className="absolute left-0 top-0 bottom-0 w-3 cursor-ew-resize z-20 bg-transparent hover:bg-white/30 transition-colors flex items-center justify-center"
          onMouseDown={(e) => handleResizeStart(e, 'left')}
        >
          <div className={cn(
            "w-0.5 h-6 bg-white/60 rounded-full transition-opacity",
            (isHovered || isResizing) && "opacity-100",
            !isHovered && !isResizing && "opacity-0"
          )} />
        </div>
      )}

      {/* Middle Drag Handle */}
      {!isOverlay && (
        <div
          {...listeners}
          onMouseDown={handleMouseDown}
          className="absolute left-3 right-3 top-0 bottom-0 z-10 cursor-move"
          style={{ pointerEvents: isResizing ? 'none' : 'auto' }}
        />
      )}

      {/* Content */}
      <div className="flex-1 overflow-hidden pointer-events-none relative z-0">
        <div className="text-sm font-bold truncate drop-shadow-sm">
          {booking.guest.vorname} {booking.guest.nachname}
        </div>
        <div className="text-xs truncate opacity-90 font-medium">
          {booking.reservierungsnummer}
        </div>
      </div>

      {/* Right Resize Handle */}
      {!isOverlay && onResize && (
        <div
          className="absolute right-0 top-0 bottom-0 w-3 cursor-ew-resize z-20 bg-transparent hover:bg-white/30 transition-colors flex items-center justify-center"
          onMouseDown={(e) => handleResizeStart(e, 'right')}
        >
          <div className={cn(
            "w-0.5 h-6 bg-white/60 rounded-full transition-opacity",
            (isHovered || isResizing) && "opacity-100",
            !isHovered && !isResizing && "opacity-0"
          )} />
        </div>
      )}
    </div>
  );
}

// Droppable Cell Component
function DroppableCell({
  roomId,
  dayIndex,
  isWeekend,
  cellWidth,
  rowHeight,
  isPreview,
  hasConflict,
}: {
  roomId: number;
  dayIndex: number;
  isWeekend: boolean;
  cellWidth: number;
  rowHeight: number;
  isPreview?: boolean;
  hasConflict?: boolean;
}) {
  const { setNodeRef, isOver } = useDroppable({
    id: `cell-${roomId}-${dayIndex}`,
    data: { roomId, dayIndex },
  });

  return (
    <div
      ref={setNodeRef}
      className={cn(
        "border-r border-slate-200 transition-all duration-100 box-border flex-shrink-0",
        isWeekend ? "bg-blue-50/30" : "bg-white",
        isOver && !isPreview && "bg-blue-300/60 ring-2 ring-blue-400 ring-inset",
        isPreview && !hasConflict && "bg-emerald-400/40 ring-2 ring-emerald-500 ring-inset",
        isPreview && hasConflict && "bg-red-400/40 ring-2 ring-red-500 ring-inset cursor-not-allowed",
      )}
      style={{
        width: `${cellWidth}px`,
        height: `${rowHeight}px`,
        minWidth: `${cellWidth}px`,
        maxWidth: `${cellWidth}px`,
      }}
    >
      {isPreview && hasConflict && (
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          <svg className="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </div>
      )}
    </div>
  );
}

export default function TapeChart({ rooms, bookings, startDate, endDate }: TapeChartProps) {
  const [currentMonth, setCurrentMonth] = useState(new Date());
  const [showMonthPicker, setShowMonthPicker] = useState(false);
  const [localBookings, setLocalBookings] = useState<Booking[]>(bookings);
  const [activeBooking, setActiveBooking] = useState<Booking | null>(null);
  const [scrollToToday, setScrollToToday] = useState(false);
  const chartContainerRef = useRef<HTMLDivElement>(null);

  // Drag preview state
  const [dragPreview, setDragPreview] = useState<{ roomId: number; dayIndex: number; hasConflict: boolean } | null>(null);
  const [dragOffset, setDragOffset] = useState<number>(0);

  // Density mode with localStorage persistence
  const [densityMode, setDensityMode] = useState<DensityMode>(() => {
    const saved = localStorage.getItem('tapeChartDensity');
    return (saved as DensityMode) || 'normal';
  });

  const { cellWidth: CELL_WIDTH, rowHeight: ROW_HEIGHT, headerHeight: HEADER_HEIGHT, sidebarWidth: SIDEBAR_WIDTH } = DENSITY_SETTINGS[densityMode];

  // Save density preference
  useEffect(() => {
    localStorage.setItem('tapeChartDensity', densityMode);
  }, [densityMode]);

  // Single month view
  const defaultStart = startDate || startOfMonth(currentMonth);
  const defaultEnd = endDate || endOfMonth(currentMonth);

  const days = eachDayOfInterval({ start: defaultStart, end: defaultEnd });

  const goToPreviousMonth = () => setCurrentMonth(prev => subMonths(prev, 1));
  const goToNextMonth = () => setCurrentMonth(prev => addMonths(prev, 1));
  const goToToday = () => {
    setCurrentMonth(new Date());
    setScrollToToday(true);
  };

  // Scroll to today
  useEffect(() => {
    if (scrollToToday && chartContainerRef.current) {
      const today = startOfDay(new Date());
      const todayIndex = days.findIndex(day => isSameDay(day, today));

      if (todayIndex !== -1) {
        const scrollPosition = todayIndex * CELL_WIDTH;
        chartContainerRef.current.scrollTo({
          left: scrollPosition,
          behavior: 'smooth'
        });
      }

      setScrollToToday(false);
    }
  }, [scrollToToday, days, CELL_WIDTH]);

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

    const startOffset = differenceInDays(checkin, startOfDay(defaultStart));
    const duration = differenceInDays(checkout, checkin);

    const padding = 4;

    return {
      left: startOffset * CELL_WIDTH + padding,
      width: duration * CELL_WIDTH - (padding * 2),
    };
  };

  const handleDragStart = (event: DragStartEvent) => {
    setActiveBooking(event.active.data.current as Booking);
  };

  const handleDragOver = (event: DragOverEvent) => {
    if (!event.over || !activeBooking) {
      setDragPreview(null);
      return;
    }

    const dropData = event.over.data.current as { roomId: number; dayIndex: number };
    if (!dropData) return;

    const { roomId, dayIndex } = dropData;

    // Calculate preview dates
    const adjustedDayIndex = dayIndex - dragOffset;
    const newCheckinDate = addDays(defaultStart, adjustedDayIndex);
    const originalDuration = differenceInDays(
      new Date(activeBooking.checkout_date),
      new Date(activeBooking.checkin_date)
    );
    const newCheckoutDate = addDays(newCheckinDate, originalDuration);

    // Check for conflicts
    const hasConflict = localBookings.some(b => {
      if (b.id === activeBooking.id || b.room_id !== roomId || b.status === 'storniert') return false;
      const existingCheckin = startOfDay(new Date(b.checkin_date));
      const existingCheckout = startOfDay(new Date(b.checkout_date));
      return newCheckinDate < existingCheckout && newCheckoutDate > existingCheckin;
    });

    setDragPreview({ roomId, dayIndex: adjustedDayIndex, hasConflict });
  };

  const handleDragEnd = (event: DragEndEvent) => {
    if (!event.over || !activeBooking || dragPreview?.hasConflict) {
      setActiveBooking(null);
      setDragPreview(null);
      setDragOffset(0);
      return;
    }

    const dropData = event.over.data.current as { roomId: number; dayIndex: number };
    if (!dropData) {
      setActiveBooking(null);
      setDragPreview(null);
      setDragOffset(0);
      return;
    }

    const { roomId: targetRoomId } = dropData;
    const targetRoom = rooms.find(r => r.id === targetRoomId);
    if (!targetRoom) {
      setActiveBooking(null);
      setDragPreview(null);
      setDragOffset(0);
      return;
    }

    const adjustedDayIndex = dragPreview.dayIndex;
    const newCheckinDate = addDays(defaultStart, adjustedDayIndex);
    const originalDuration = differenceInDays(
      new Date(activeBooking.checkout_date),
      new Date(activeBooking.checkin_date)
    );
    const newCheckoutDate = addDays(newCheckinDate, originalDuration);

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

    setLocalBookings(updatedBookings);
    setActiveBooking(null);
    setDragPreview(null);
    setDragOffset(0);
  };

  const handleCustomDragStart = (bookingId: number, x: number) => {
    const booking = localBookings.find(b => b.id === bookingId);
    if (!booking) return;

    const pos = getBookingPosition(booking);
    const relativeX = x - pos.left;
    const dayOffset = Math.floor(relativeX / CELL_WIDTH);
    setDragOffset(Math.max(0, dayOffset));
  };

  const handleResize = (bookingId: number, direction: 'left' | 'right', days: number) => {
    const booking = localBookings.find(b => b.id === bookingId);
    if (!booking) return;

    const checkinDate = startOfDay(new Date(booking.checkin_date));
    const checkoutDate = startOfDay(new Date(booking.checkout_date));

    let newCheckin = checkinDate;
    let newCheckout = checkoutDate;

    if (direction === 'left') {
      newCheckin = startOfDay(addDays(checkinDate, days));
    } else {
      newCheckout = startOfDay(addDays(checkoutDate, days));
    }

    // Validate minimum duration
    if (differenceInDays(newCheckout, newCheckin) < 1) return;

    // Check for conflicts
    const hasConflict = localBookings.some(b => {
      if (b.id === bookingId || b.room_id !== booking.room_id || b.status === 'storniert') return false;
      const existingCheckin = startOfDay(new Date(b.checkin_date));
      const existingCheckout = startOfDay(new Date(b.checkout_date));
      return newCheckin < existingCheckout && newCheckout > existingCheckin;
    });

    if (hasConflict) return;

    // Update booking
    const updatedBookings = localBookings.map(b =>
      b.id === bookingId
        ? {
            ...b,
            checkin_date: format(newCheckin, 'yyyy-MM-dd'),
            checkout_date: format(newCheckout, 'yyyy-MM-dd'),
          }
        : b
    );

    setLocalBookings(updatedBookings);
  };

  return (
    <DndContext
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
      collisionDetection={pointerWithin}
    >
      <div className="w-full h-full flex flex-col bg-gradient-to-br from-slate-50 to-slate-100">
        {/* Navigation Bar */}
        <div className="bg-gradient-to-r from-slate-700 to-slate-800 border-b border-slate-600 px-8 py-4 shadow-xl flex items-center justify-between relative">
          <div className="flex items-center gap-3">
            <button
              onClick={goToPreviousMonth}
              className="bg-slate-600/50 hover:bg-slate-600 backdrop-blur-sm text-white px-4 py-2.5 rounded-xl font-semibold transition-all shadow-lg hover:shadow-xl hover:scale-105 flex items-center gap-2 border border-slate-500/30"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M15 19l-7-7 7-7" />
              </svg>
            </button>

            {/* Month Picker */}
            <div className="relative">
              <button
                onClick={() => setShowMonthPicker(!showMonthPicker)}
                className="group bg-slate-700/80 hover:bg-slate-600/90 backdrop-blur-md text-white px-6 py-3 rounded-2xl font-bold text-xl shadow-xl border-2 border-slate-500/40 hover:border-slate-400/60 min-w-[280px] text-center transition-all hover:scale-[1.03] flex items-center justify-between gap-4"
              >
                <svg className="w-5 h-5 text-slate-400 group-hover:text-white transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                <span className="flex-1 px-3">{format(currentMonth, 'MMMM yyyy', { locale: de })}</span>
                <svg className={cn(
                  "w-5 h-5 text-slate-400 group-hover:text-white transition-all",
                  showMonthPicker && "rotate-180"
                )} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M19 9l-7 7-7-7" />
                </svg>
              </button>

              {/* Month Picker Dropdown - Shortened for brevity */}
            </div>

            <button
              onClick={goToNextMonth}
              className="bg-slate-600/50 hover:bg-slate-600 backdrop-blur-sm text-white px-4 py-2.5 rounded-xl font-semibold transition-all shadow-lg hover:shadow-xl hover:scale-105 flex items-center gap-2 border border-slate-500/30"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>

          {/* Density Selector + Heute Button */}
          <div className="flex items-center gap-3">
            {/* Density Mode Buttons */}
            <div className="flex items-center gap-2 bg-slate-800/50 px-3 py-2 rounded-xl">
              <span className="text-xs font-semibold text-slate-400 mr-2">Ansicht:</span>
              <button
                onClick={() => setDensityMode('compact')}
                className={cn(
                  "p-2 rounded-lg transition-all",
                  densityMode === 'compact'
                    ? "bg-blue-500 text-white shadow-lg"
                    : "bg-slate-600/50 text-slate-300 hover:bg-slate-600 hover:text-white"
                )}
                title="Kompakt"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <rect x="3" y="3" width="7" height="7" strokeWidth={2} />
                  <rect x="14" y="3" width="7" height="7" strokeWidth={2} />
                  <rect x="3" y="14" width="7" height="7" strokeWidth={2} />
                  <rect x="14" y="14" width="7" height="7" strokeWidth={2} />
                </svg>
              </button>
              <button
                onClick={() => setDensityMode('normal')}
                className={cn(
                  "p-2 rounded-lg transition-all",
                  densityMode === 'normal'
                    ? "bg-blue-500 text-white shadow-lg"
                    : "bg-slate-600/50 text-slate-300 hover:bg-slate-600 hover:text-white"
                )}
                title="Normal"
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <rect x="4" y="4" width="7" height="7" strokeWidth={2} />
                  <rect x="13" y="4" width="7" height="7" strokeWidth={2} />
                  <rect x="4" y="13" width="7" height="7" strokeWidth={2} />
                  <rect x="13" y="13" width="7" height="7" strokeWidth={2} />
                </svg>
              </button>
              <button
                onClick={() => setDensityMode('comfortable')}
                className={cn(
                  "p-2 rounded-lg transition-all",
                  densityMode === 'comfortable'
                    ? "bg-blue-500 text-white shadow-lg"
                    : "bg-slate-600/50 text-slate-300 hover:bg-slate-600 hover:text-white"
                )}
                title="Komfortabel"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <rect x="5" y="5" width="6" height="6" strokeWidth={2} />
                  <rect x="13" y="5" width="6" height="6" strokeWidth={2} />
                  <rect x="5" y="13" width="6" height="6" strokeWidth={2} />
                  <rect x="13" y="13" width="6" height="6" strokeWidth={2} />
                </svg>
              </button>
            </div>

            <button
              onClick={goToToday}
              className="bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-600 hover:to-emerald-700 text-white heute-button py-3 rounded-2xl font-bold text-lg transition-all shadow-xl hover:shadow-2xl hover:scale-[1.03] flex items-center gap-3 border-2 border-emerald-400/40 hover:border-emerald-300/60"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              <span className="px-1">Heute</span>
            </button>
          </div>
        </div>

        {/* Chart Container */}
        <div ref={chartContainerRef} className="flex-1 overflow-auto">
          <div className="">
            {/* Header */}
            <div className="sticky top-0 z-20 bg-white shadow-xl">
              <div className="flex">
                {/* Top-left corner */}
                <div
                  className="sticky left-0 z-30 bg-gradient-to-br from-slate-800 to-slate-900 flex items-center justify-center font-bold text-white text-lg shadow-2xl box-border"
                  style={{
                    width: `${SIDEBAR_WIDTH}px`,
                    height: `${HEADER_HEIGHT}px`,
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
                <div className="flex relative">
                  {days.map((day, idx) => {
                    const isTodayDate = isToday(day);

                    return (
                      <div
                        key={idx}
                        className={cn(
                          "flex flex-col items-center justify-center transition-all hover:bg-slate-100 border-r border-slate-200 box-border relative",
                          isTodayDate && "bg-gradient-to-b from-emerald-100 to-emerald-200 ring-2 ring-emerald-400 ring-inset",
                          !isTodayDate && (format(day, 'i') === '6' || format(day, 'i') === '7')
                            ? 'bg-gradient-to-b from-blue-50 to-blue-100'
                            : !isTodayDate && 'bg-white'
                        )}
                        style={{
                          width: `${CELL_WIDTH}px`,
                          height: `${HEADER_HEIGHT}px`,
                          minWidth: `${CELL_WIDTH}px`,
                          maxWidth: `${CELL_WIDTH}px`,
                        }}
                      >
                        <div className="absolute inset-0 shadow-sm pointer-events-none" style={{ height: '100vh' }} />

                        <div className={cn(
                          "text-xs font-medium uppercase tracking-wider relative z-10",
                          isTodayDate ? "text-emerald-700" : "text-slate-500"
                        )}>
                          {format(day, 'EEE', { locale: de })}
                        </div>
                        <div className={cn(
                          "text-2xl font-bold my-1 relative z-10",
                          isTodayDate ? "text-emerald-800" : "text-slate-800"
                        )}>
                          {format(day, 'd')}
                        </div>
                        <div className={cn(
                          "text-xs font-semibold uppercase relative z-10",
                          isTodayDate ? "text-emerald-700" : "text-slate-600"
                        )}>
                          {format(day, 'MMM', { locale: de })}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>

            {/* Rows */}
            {rooms.map((room) => (
              <div key={room.id} className="flex hover:bg-slate-50 transition-all group">
                {/* Room sidebar */}
                <div
                  className="sticky left-0 z-10 bg-gradient-to-r from-slate-100 to-slate-50 flex flex-col justify-center px-5 shadow-md group-hover:shadow-lg transition-all box-border"
                  style={{
                    width: `${SIDEBAR_WIDTH}px`,
                    height: `${ROW_HEIGHT}px`,
                    minWidth: `${SIDEBAR_WIDTH}px`,
                    maxWidth: `${SIDEBAR_WIDTH}px`,
                  }}
                >
                  <div className="font-bold text-base text-slate-800 mb-1">{room.name}</div>
                  <div className="flex items-center gap-2 text-xs text-slate-600">
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                    </svg>
                    <span className="font-medium">{room.gebaeude_typ}</span>
                  </div>
                  <div className="flex items-center gap-2 text-xs text-slate-600">
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    <span className="font-medium">{room.ort}</span>
                  </div>
                </div>

                {/* Timeline grid */}
                <div className="relative flex" style={{ minWidth: `${days.length * CELL_WIDTH}px` }}>
                  {days.map((day, dayIdx) => {
                    const isWeekend = format(day, 'i') === '6' || format(day, 'i') === '7';

                    // Check if preview
                    let isPreview = false;
                    let hasConflict = false;
                    if (dragPreview && dragPreview.roomId === room.id && activeBooking) {
                      const checkinDate = new Date(activeBooking.checkin_date);
                      const checkoutDate = new Date(activeBooking.checkout_date);
                      const duration = differenceInDays(checkoutDate, checkinDate);

                      if (dayIdx >= dragPreview.dayIndex && dayIdx < dragPreview.dayIndex + duration) {
                        isPreview = true;
                        hasConflict = dragPreview.hasConflict;
                      }
                    }

                    return (
                      <DroppableCell
                        key={dayIdx}
                        roomId={room.id}
                        dayIndex={dayIdx}
                        isWeekend={isWeekend}
                        cellWidth={CELL_WIDTH}
                        rowHeight={ROW_HEIGHT}
                        isPreview={isPreview}
                        hasConflict={hasConflict}
                      />
                    );
                  })}

                  {/* Bookings */}
                  {localBookings
                    .filter((b) => {
                      if (b.room_id !== room.id) return false;

                      const checkin = new Date(b.checkin_date);
                      const checkout = new Date(b.checkout_date);

                      return checkin <= defaultEnd && checkout >= defaultStart;
                    })
                    .map((booking) => {
                      const pos = getBookingPosition(booking);

                      return (
                        <DraggableBooking
                          key={booking.id}
                          booking={booking}
                          position={pos}
                          rowHeight={ROW_HEIGHT}
                          onResize={handleResize}
                          onDragStartCustom={handleCustomDragStart}
                        />
                      );
                    })}
                </div>
              </div>
            ))}
          </div>

          {/* Legend */}
          <div className="sticky bottom-0 left-0 right-0 bg-gradient-to-r from-slate-800 to-slate-900 border-t-2 border-slate-700 px-8 py-5 shadow-2xl">
            <div className="flex gap-8 justify-center items-center flex-wrap">
              <div className="text-white font-bold text-sm uppercase tracking-wider mr-4">Status:</div>
              {Object.entries(STATUS_COLORS).map(([status, colors]) => (
                <div key={status} className="flex items-center gap-3 group cursor-pointer">
                  <div className={cn(
                    "w-8 h-8 rounded-lg border-2 transition-transform group-hover:scale-110",
                    colors.bg,
                    colors.border,
                    colors.shadow
                  )} />
                  <span className="text-sm font-semibold capitalize text-white group-hover:text-blue-300 transition-colors">
                    {status}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Drag Overlay */}
        <DragOverlay dropAnimation={null}>
          {activeBooking ? (
            <DraggableBooking
              booking={activeBooking}
              position={getBookingPosition(activeBooking)}
              rowHeight={ROW_HEIGHT}
              isOverlay={true}
            />
          ) : null}
        </DragOverlay>
      </div>
    </DndContext>
  );
}
