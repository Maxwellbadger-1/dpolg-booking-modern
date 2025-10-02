import { useState, useRef, useEffect, useLayoutEffect, useCallback } from 'react';
import { format, addDays, differenceInDays, startOfMonth, endOfMonth, eachDayOfInterval, addMonths, subMonths, startOfDay, isSameDay } from 'date-fns';
import { de } from 'date-fns/locale';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../lib/utils';
import { useData } from '../context/DataContext';
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
  startDate?: Date;
  endDate?: Date;
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
  onResize?: (bookingId: number, direction: 'start' | 'end', daysDelta: number) => void;
  hasOverlap?: boolean; // Red overlay when drag has overlap
}

function DraggableBooking({ booking, position, isOverlay = false, rowHeight, onResize, hasOverlap = false }: DraggableBookingProps) {
  const [isResizing, setIsResizing] = useState(false);
  const [resizeDirection, setResizeDirection] = useState<ResizeDirection>(null);
  const [cursor, setCursor] = useState<string>('move');
  const [resizePreview, setResizePreview] = useState<{ left: number; width: number } | null>(null);
  const dragStartX = useRef<number>(0);
  const resizeStartPosition = useRef<{ left: number; width: number }>({ left: 0, width: 0 });

  const { attributes, listeners, setNodeRef, isDragging, transform } = useDraggable({
    id: `booking-${booking.id}`,
    data: booking,
    disabled: isResizing, // Disable drag during resize
  });

  const colors = STATUS_COLORS[booking.status] || STATUS_COLORS.bestaetigt;

  // Detect if pointer is in resize zone (dnd-timeline pattern)
  const getResizeDirection = useCallback((e: React.PointerEvent, element: HTMLElement): ResizeDirection => {
    const rect = element.getBoundingClientRect();
    const mouseX = e.clientX;

    // Check left edge (start)
    if (Math.abs(mouseX - rect.left) <= RESIZE_HANDLE_WIDTH / 2) {
      return 'start';
    }

    // Check right edge (end)
    if (Math.abs(mouseX - rect.right) <= RESIZE_HANDLE_WIDTH / 2) {
      return 'end';
    }

    return null;
  }, []);

  // Handle pointer down - determine if resize or drag
  const handlePointerDown = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    if (isOverlay) return;

    const direction = getResizeDirection(e, e.currentTarget);

    if (direction && onResize) {
      // Start resize
      e.stopPropagation();
      setIsResizing(true);
      setResizeDirection(direction);
      dragStartX.current = e.clientX;
      resizeStartPosition.current = { ...position };
    } else {
      // Normal drag - let dnd-kit handle it
      listeners?.onPointerDown?.(e);
    }
  }, [getResizeDirection, onResize, position, listeners, isOverlay]);

  // Handle pointer move - update cursor
  const handlePointerMove = useCallback((e: React.PointerEvent<HTMLDivElement>) => {
    if (isOverlay || isResizing) return;

    const direction = getResizeDirection(e, e.currentTarget);

    if (direction) {
      setCursor('ew-resize');
    } else {
      setCursor('move');
    }
  }, [getResizeDirection, isOverlay, isResizing]);

  // Layout effect to handle resize events with LIVE preview
  useLayoutEffect(() => {
    if (!isResizing || !onResize) return;

    const handlePointerMove = (e: PointerEvent) => {
      const deltaX = e.clientX - dragStartX.current;
      const daysDelta = Math.round(deltaX / CELL_WIDTH);

      // Calculate LIVE preview position
      if (resizeDirection === 'start') {
        // Resize from left - change left and width
        const newLeft = resizeStartPosition.current.left + (daysDelta * CELL_WIDTH);
        const newWidth = resizeStartPosition.current.width - (daysDelta * CELL_WIDTH);

        // Only update if width stays positive
        if (newWidth > CELL_WIDTH / 2) {
          setResizePreview({ left: newLeft, width: newWidth });
        }
      } else if (resizeDirection === 'end') {
        // Resize from right - only change width
        const newWidth = resizeStartPosition.current.width + (daysDelta * CELL_WIDTH);

        // Only update if width stays positive
        if (newWidth > CELL_WIDTH / 2) {
          setResizePreview({
            left: resizeStartPosition.current.left,
            width: newWidth
          });
        }
      }
    };

    const handlePointerUp = (e: PointerEvent) => {
      const deltaX = e.clientX - dragStartX.current;
      const daysDelta = Math.round(deltaX / CELL_WIDTH);

      if (daysDelta !== 0 && resizeDirection) {
        onResize(booking.id, resizeDirection, daysDelta);
      }

      setIsResizing(false);
      setResizeDirection(null);
      setResizePreview(null); // Clear preview
    };

    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);

    return () => {
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
  }, [isResizing, onResize, booking.id, resizeDirection]);

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
    transition: isResizing ? 'none' : 'all 0.2s', // Disable transition during resize for instant feedback
  };

  return (
    <div
      ref={setNodeRef}
      {...attributes}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      className={cn(
        isOverlay ? "rounded-xl border-2" : "absolute rounded-xl border-2",
        "flex items-center px-3 transition-all duration-200",
        !isOverlay && !isResizing && "hover:scale-[1.02] active:scale-95",
        "backdrop-blur-sm",
        isDragging && !isOverlay && "opacity-0",
        isResizing && "border-dashed border-4 scale-[1.02]",
        colors.bg,
        colors.border,
        colors.text,
        colors.shadow
      )}
      style={style}
      title={`${booking.guest.vorname} ${booking.guest.nachname}\n${booking.reservierungsnummer}\n${booking.checkin_date} - ${booking.checkout_date}`}
    >
      <div className="flex-1 overflow-hidden">
        <div className="text-sm font-bold truncate drop-shadow-sm">
          {booking.guest.vorname} {booking.guest.nachname}
        </div>
        <div className="text-xs truncate opacity-90 font-medium">
          {booking.reservierungsnummer}
        </div>
      </div>
    </div>
  );
}

function DroppableCell({ roomId, dayIndex, isWeekend, cellWidth, rowHeight, hasOverlap, children }: { roomId: number; dayIndex: number; isWeekend: boolean; cellWidth: number; rowHeight: number; hasOverlap?: boolean; children?: React.ReactNode }) {
  const { setNodeRef, isOver } = useDroppable({
    id: `cell-${roomId}-${dayIndex}`,
    data: { roomId, dayIndex },
  });

  return (
    <div
      ref={setNodeRef}
      className={cn(
        "border-r border-slate-200 transition-all duration-200 box-border relative",
        isWeekend ? "bg-blue-50/30" : "bg-white",
        isOver && !hasOverlap && "bg-blue-300/60 ring-2 ring-blue-400 ring-inset",
        isOver && hasOverlap && "bg-red-500/60 ring-2 ring-red-600 ring-inset",
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

export default function TapeChart({ startDate, endDate }: TapeChartProps) {
  // Get data from global context
  const { rooms, bookings, refreshAll, updateBooking } = useData();

  console.log('🎨 [TapeChart] Render:', {
    roomsCount: rooms.length,
    bookingsCount: bookings.length,
    bookings: bookings.map(b => ({ id: b.id, reservierungsnummer: b.reservierungsnummer }))
  });

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

  // KRITISCH: Sync localBookings mit bookings aus Context
  useEffect(() => {
    console.log('🔄 [TapeChart] useEffect: bookings changed', {
      bookingsLength: bookings.length,
      localBookingsLength: localBookings.length
    });
    setLocalBookings(bookings);
  }, [bookings]);
  const chartContainerRef = useRef<HTMLDivElement>(null);

  // Persist density mode to localStorage
  useEffect(() => {
    localStorage.setItem('tapechart-density', densityMode);
  }, [densityMode]);

  // Get current density settings
  const density = DENSITY_SETTINGS[densityMode];

  // Configure drag sensor with delay to prevent accidental drags
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        delay: 150,      // 150ms delay before drag activates
        tolerance: 5     // Allow 5px movement during delay
      }
    })
  );

  // Default date range: current month
  const defaultStart = startDate || startOfMonth(currentMonth);
  const defaultEnd = endDate || endOfMonth(currentMonth);

  const days = eachDayOfInterval({ start: defaultStart, end: defaultEnd });

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

  // Scroll to today's date column
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
    const checkin = new Date(booking.checkin_date);
    const checkout = new Date(booking.checkout_date);

    const startOffset = differenceInDays(checkin, defaultStart);
    // Include checkout day: add +1 to duration
    const duration = differenceInDays(checkout, checkin) + 1;

    // Add padding: 4px on each side
    const padding = 4;

    return {
      left: startOffset * density.cellWidth + padding,
      width: duration * density.cellWidth - (padding * 2),
      isVisible: startOffset >= 0 && startOffset < days.length,
    };
  };

  const handleDragStart = (event: DragStartEvent) => {
    console.log('DRAG START with dnd-kit!', event.active.data.current);
    setActiveBooking(event.active.data.current as Booking);
  };

  // Helper function to check if two bookings overlap
  const checkOverlap = (booking1Start: Date, booking1End: Date, booking2Start: Date, booking2End: Date): boolean => {
    // Bookings can TOUCH (same dates) but NOT overlap
    // Overlap exists if booking1End > booking2Start AND booking1Start < booking2End
    return booking1End > booking2Start && booking1Start < booking2End;
  };

  const handleDragEnd = (event: DragEndEvent) => {
    console.log('DRAG END!', event);

    if (!event.over || !activeBooking) {
      setActiveBooking(null);
      setDragHasOverlap(false);
      setOverlapDropZone(null);
      return;
    }

    const dropData = event.over.data.current as { roomId: number; dayIndex: number };
    console.log('DROP DATA:', dropData);

    if (!dropData) {
      setActiveBooking(null);
      return;
    }

    const { roomId: targetRoomId, dayIndex } = dropData;

    // Find the target room
    const targetRoom = rooms.find(r => r.id === targetRoomId);
    if (!targetRoom) {
      console.error('Target room not found:', targetRoomId);
      setActiveBooking(null);
      return;
    }

    // Calculate new dates (normalized to start of day)
    const newCheckinDate = startOfDay(addDays(defaultStart, dayIndex));
    const originalDuration = differenceInDays(
      startOfDay(new Date(activeBooking.checkout_date)),
      startOfDay(new Date(activeBooking.checkin_date))
    );
    const newCheckoutDate = startOfDay(addDays(newCheckinDate, originalDuration));

    // Check for overlaps with other bookings in the same room
    const hasOverlap = localBookings.some(b => {
      if (b.id === activeBooking.id || b.room_id !== targetRoomId) return false;

      const existingStart = startOfDay(new Date(b.checkin_date));
      const existingEnd = startOfDay(new Date(b.checkout_date));

      const wouldOverlap = checkOverlap(newCheckinDate, newCheckoutDate, existingStart, existingEnd);

      console.log('DROP - Checking overlap:', {
        new: { start: format(newCheckinDate, 'yyyy-MM-dd'), end: format(newCheckoutDate, 'yyyy-MM-dd') },
        existing: { start: format(existingStart, 'yyyy-MM-dd'), end: format(existingEnd, 'yyyy-MM-dd'), booking: b.guest.nachname },
        wouldOverlap
      });

      return wouldOverlap;
    });

    if (hasOverlap) {
      console.warn('Cannot move booking: would overlap with existing booking');
      setActiveBooking(null);
      setDragHasOverlap(false);
      setOverlapDropZone(null);
      return;
    }

    console.log('Updating booking:', {
      id: activeBooking.id,
      old_room: activeBooking.room_id,
      new_room: targetRoomId,
      old_checkin: activeBooking.checkin_date,
      new_checkin: format(newCheckinDate, 'yyyy-MM-dd'),
      old_checkout: activeBooking.checkout_date,
      new_checkout: format(newCheckoutDate, 'yyyy-MM-dd'),
    });

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

    // Save to database
    invoke('update_booking_dates_and_room_command', {
      id: activeBooking.id,
      roomId: targetRoomId,
      checkinDate: format(newCheckinDate, 'yyyy-MM-dd'),
      checkoutDate: format(newCheckoutDate, 'yyyy-MM-dd'),
    }).then(() => {
      console.log('Buchung erfolgreich in DB gespeichert');
      refreshAll(); // Refresh to get updated data
    }).catch(err => {
      console.error('Fehler beim Speichern der Buchung:', err);
      alert('Fehler beim Speichern der Buchungsänderung');
      refreshAll(); // Reload to get original state
    });
  };

  const handleResize = useCallback((bookingId: number, direction: 'start' | 'end', daysDelta: number) => {
    console.log('RESIZE!', { bookingId, direction, daysDelta });

    setLocalBookings(prev => {
      const currentBooking = prev.find(b => b.id === bookingId);
      if (!currentBooking) return prev;

      const currentCheckin = new Date(currentBooking.checkin_date);
      const currentCheckout = new Date(currentBooking.checkout_date);

      let newCheckin = currentCheckin;
      let newCheckout = currentCheckout;

      if (direction === 'start') {
        // Resize from left edge - change checkin date
        newCheckin = addDays(currentCheckin, daysDelta);

        // Prevent invalid range (checkin after checkout)
        if (newCheckin >= currentCheckout) {
          console.warn('Invalid resize: checkin would be after checkout');
          return prev;
        }
      } else {
        // Resize from right edge - change checkout date
        newCheckout = addDays(currentCheckout, daysDelta);

        // Prevent invalid range (checkout before checkin)
        if (newCheckout <= currentCheckin) {
          console.warn('Invalid resize: checkout would be before checkin');
          return prev;
        }
      }

      // Check for overlaps with other bookings in the same room
      const hasOverlap = prev.some(b =>
        b.id !== bookingId &&
        b.room_id === currentBooking.room_id &&
        checkOverlap(
          newCheckin,
          newCheckout,
          new Date(b.checkin_date),
          new Date(b.checkout_date)
        )
      );

      if (hasOverlap) {
        console.warn('Cannot resize booking: would overlap with existing booking');
        return prev;
      }

      console.log('Resized booking:', {
        id: bookingId,
        old_checkin: currentBooking.checkin_date,
        new_checkin: format(newCheckin, 'yyyy-MM-dd'),
        old_checkout: currentBooking.checkout_date,
        new_checkout: format(newCheckout, 'yyyy-MM-dd'),
      });

      const updatedBookings = prev.map(booking => {
        if (booking.id !== bookingId) return booking;

        return {
          ...booking,
          checkin_date: format(newCheckin, 'yyyy-MM-dd'),
          checkout_date: format(newCheckout, 'yyyy-MM-dd'),
        };
      });

      // Save resize to database
      const booking = prev.find(b => b.id === bookingId);
      if (booking) {
        invoke('update_booking_dates_and_room_command', {
          id: bookingId,
          roomId: booking.room_id,
          checkinDate: format(newCheckin, 'yyyy-MM-dd'),
          checkoutDate: format(newCheckout, 'yyyy-MM-dd'),
        }).then(() => {
          console.log('Resize erfolgreich in DB gespeichert');
          refreshAll();
        }).catch(err => {
          console.error('Fehler beim Speichern der Resize-Änderung:', err);
          alert('Fehler beim Speichern der Buchungsänderung');
          refreshAll();
        });
      }

      return updatedBookings;
    });
  }, [updateBooking, refreshAll]);

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
        const originalDuration = differenceInDays(
          startOfDay(new Date(activeBooking.checkout_date)),
          startOfDay(new Date(activeBooking.checkin_date))
        );
        const newCheckoutDate = startOfDay(addDays(newCheckinDate, originalDuration));

        // Check if this would overlap
        const hasOverlap = localBookings.some(b => {
          if (b.id === activeBooking.id || b.room_id !== targetRoomId) return false;

          const existingStart = startOfDay(new Date(b.checkin_date));
          const existingEnd = startOfDay(new Date(b.checkout_date));

          console.log('Checking overlap:', {
            new: { start: format(newCheckinDate, 'yyyy-MM-dd'), end: format(newCheckoutDate, 'yyyy-MM-dd') },
            existing: { start: format(existingStart, 'yyyy-MM-dd'), end: format(existingEnd, 'yyyy-MM-dd') },
            wouldOverlap: checkOverlap(newCheckinDate, newCheckoutDate, existingStart, existingEnd)
          });

          return checkOverlap(newCheckinDate, newCheckoutDate, existingStart, existingEnd);
        });

        setDragHasOverlap(hasOverlap);
        setOverlapDropZone(hasOverlap ? { roomId: targetRoomId, dayIndex } : null);
      }}
      onDragEnd={handleDragEnd}
      collisionDetection={pointerWithin}
    >
      <div className="w-full h-full flex flex-col bg-gradient-to-br from-slate-50 to-slate-100">
        {/* Navigation Bar */}
        <div className="bg-gradient-to-r from-slate-700 to-slate-800 border-b border-slate-600 px-8 py-4 shadow-xl flex items-center justify-center relative">
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
                className="group bg-slate-700/80 hover:bg-slate-600/90 backdrop-blur-md text-white px-6 py-3 rounded-2xl font-bold text-xl shadow-xl border-2 border-slate-500/40 hover:border-slate-400/60 min-w-[320px] text-center transition-all hover:scale-[1.03] flex items-center justify-between gap-4"
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

              {/* Month Picker Dropdown */}
              {showMonthPicker && (
                <div className="absolute top-full mt-2 left-1/2 -translate-x-1/2 bg-slate-800 rounded-xl shadow-2xl border-2 border-slate-600 p-5 z-50 w-[380px]">
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
              className="bg-slate-600/50 hover:bg-slate-600 backdrop-blur-sm text-white px-4 py-2.5 rounded-xl font-semibold transition-all shadow-lg hover:shadow-xl hover:scale-105 flex items-center gap-2 border border-slate-500/30"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>

          {/* Right Side: Density Mode + Heute Button */}
          <div className="absolute right-8 flex items-center gap-3">
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
              onClick={async () => {
                try {
                  const changedCount = await invoke<number>('update_booking_statuses_command');
                  if (changedCount > 0) {
                    console.log(`✅ ${changedCount} Buchungs-Status aktualisiert`);
                    window.location.reload(); // Einfacher Reload
                  } else {
                    console.log('ℹ️ Keine Status-Änderungen nötig');
                  }
                } catch (error) {
                  console.error('❌ Fehler beim Status-Update:', error);
                }
              }}
              className="bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white py-3 rounded-2xl font-bold text-lg transition-all shadow-xl hover:shadow-2xl hover:scale-[1.03] flex items-center gap-3 border-2 border-blue-400/40 hover:border-blue-300/60"
              title="Buchungs-Status aktualisieren (basierend auf Datum)"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              <span className="px-1">Status Update</span>
            </button>

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
          <div className="relative">
            {/* Header */}
            <div className="sticky top-0 z-20 bg-white shadow-xl">
              <div className="flex">
                {/* Top-left corner */}
                <div
                  className="sticky left-0 z-30 bg-gradient-to-br from-slate-800 to-slate-900 flex items-center justify-center font-bold text-white text-lg shadow-2xl box-border"
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
                  {days.map((day, idx) => (
                    <div
                      key={idx}
                      className={cn(
                        "flex flex-col items-center justify-center transition-all hover:bg-slate-100 border-r border-slate-200 box-border",
                        format(day, 'i') === '6' || format(day, 'i') === '7'
                          ? 'bg-gradient-to-b from-blue-50 to-blue-100'
                          : 'bg-white'
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
                  ))}
                </div>
              </div>
            </div>

            {/* Rows */}
            {rooms.map((room, roomIdx) => {
              const roomBookings = localBookings.filter((b) => b.room_id === room.id);
              console.log(`🏨 [TapeChart] Zimmer ${roomIdx}: ${room.name} (ID: ${room.id}) hat ${roomBookings.length} Buchungen:`, roomBookings.map(b => ({ booking_id: b.id, room_id: b.room_id, checkin: b.checkin_date, checkout: b.checkout_date })));

              return (
              <div key={room.id} className="flex hover:bg-slate-50 transition-all group">
                {/* Room sidebar */}
                <div
                  className={cn("sticky left-0 z-20 bg-gradient-to-r from-slate-100 to-slate-50 flex flex-col justify-center shadow-md group-hover:shadow-lg transition-all box-border", density.padding)}
                  style={{
                    width: `${SIDEBAR_WIDTH}px`,
                    height: `${density.rowHeight}px`,
                    minWidth: `${SIDEBAR_WIDTH}px`,
                    maxWidth: `${SIDEBAR_WIDTH}px`,
                  }}
                >
                  <div className={cn("font-bold text-slate-800 mb-1", density.fontSize)}>{room.name}</div>
                  <div className={cn("flex items-center gap-2 text-slate-600", density.fontSize)}>
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                    </svg>
                    <span className="font-medium">{room.gebaeude_typ}</span>
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
                    const hasOverlapHere = overlapDropZone?.roomId === room.id && overlapDropZone?.dayIndex === dayIdx;
                    return (
                      <DroppableCell
                        key={dayIdx}
                        roomId={room.id}
                        dayIndex={dayIdx}
                        isWeekend={isWeekend}
                        cellWidth={density.cellWidth}
                        rowHeight={density.rowHeight}
                        hasOverlap={hasOverlapHere}
                      />
                    );
                  })}

                  {/* Bookings */}
                  {localBookings
                    .filter((b) => b.room_id === room.id)
                    .map((booking) => {
                      const pos = getBookingPosition(booking);
                      if (!pos.isVisible) return null;

                      return (
                        <DraggableBooking
                          key={booking.id}
                          booking={booking}
                          position={pos}
                          rowHeight={density.rowHeight}
                          onResize={handleResize}
                        />
                      );
                    })}
                </div>
              </div>
            );
            })}
          </div>

          {/* Legend */}
          <div className="sticky bottom-0 left-0 right-0 z-30 bg-gradient-to-r from-slate-800 to-slate-900 border-t-2 border-slate-700 px-8 py-5 shadow-2xl">
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
              rowHeight={density.rowHeight}
              isOverlay={true}
            />
          ) : null}
        </DragOverlay>
      </div>
    </DndContext>
  );
}