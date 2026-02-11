import { forwardRef, useMemo } from 'react';
import DatePicker, { registerLocale } from 'react-datepicker';
import { de } from 'date-fns/locale';
import { format, parse, eachDayOfInterval, isWithinInterval, startOfDay, addDays, isBefore, isAfter } from 'date-fns';
import 'react-datepicker/dist/react-datepicker.css';

// Register German locale
registerLocale('de', de);

interface BookingDatePickerProps {
  value: string; // YYYY-MM-DD format
  onChange: (value: string) => void;
  label: string;
  required?: boolean;
  minDate?: Date;
  maxDate?: Date;
  roomId?: number;
  bookings?: Array<{
    id?: number;
    room_id: number;
    checkin_date: string;
    checkout_date: string;
    status?: string;
  }>;
  currentBookingId?: number; // Exclude this booking from occupied dates
  isCheckout?: boolean; // If true, this is a checkout date picker
  checkinDate?: string; // For checkout picker: the selected checkin date
  className?: string;
  disabled?: boolean;
  placeholder?: string;
}

// Custom input component for styling
const CustomInput = forwardRef<HTMLInputElement, { value?: string; onClick?: () => void; placeholder?: string; disabled?: boolean; className?: string }>(
  ({ value, onClick, placeholder, disabled, className }, ref) => (
    <input
      ref={ref}
      type="text"
      value={value}
      onClick={onClick}
      placeholder={placeholder || 'Datum auswählen'}
      disabled={disabled}
      readOnly
      className={className || 'w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 cursor-pointer bg-white'}
    />
  )
);
CustomInput.displayName = 'CustomInput';

export default function BookingDatePicker({
  value,
  onChange,
  label,
  required = false,
  minDate,
  maxDate,
  roomId,
  bookings = [],
  currentBookingId,
  isCheckout = false,
  checkinDate,
  className,
  disabled = false,
  placeholder,
}: BookingDatePickerProps) {
  // Convert string value to Date object
  const selectedDate = useMemo(() => {
    if (!value) return null;
    try {
      return parse(value, 'yyyy-MM-dd', new Date());
    } catch {
      return null;
    }
  }, [value]);

  // Calculate the minimum date
  const calculatedMinDate = useMemo(() => {
    // For checkout picker: minimum is day after checkin
    if (isCheckout && checkinDate) {
      const checkin = parse(checkinDate, 'yyyy-MM-dd', new Date());
      return addDays(checkin, 1);
    }
    // Default: today (no past bookings)
    return minDate || startOfDay(new Date());
  }, [isCheckout, checkinDate, minDate]);

  // Calculate occupied dates for the room
  const occupiedDates = useMemo(() => {
    if (!roomId || !bookings.length) return [];

    const roomBookings = bookings.filter(b => {
      // Same room
      if (b.room_id !== roomId) return false;
      // Exclude current booking (when editing)
      if (currentBookingId && b.id === currentBookingId) return false;
      // Only consider active bookings (not cancelled)
      if (b.status === 'storniert') return false;
      return true;
    });

    const dates: Date[] = [];

    roomBookings.forEach(booking => {
      try {
        const start = parse(booking.checkin_date, 'yyyy-MM-dd', new Date());
        const end = parse(booking.checkout_date, 'yyyy-MM-dd', new Date());

        // Add all days from checkin to checkout (exclusive of checkout for checkin picker)
        // For checkout picker, the checkout day itself is available for new checkin
        const interval = eachDayOfInterval({ start, end: addDays(end, -1) });
        dates.push(...interval);
      } catch {
        // Skip invalid dates
      }
    });

    return dates;
  }, [roomId, bookings, currentBookingId]);

  // Check if a date is occupied
  const isDateOccupied = (date: Date) => {
    return occupiedDates.some(occupied =>
      format(occupied, 'yyyy-MM-dd') === format(date, 'yyyy-MM-dd')
    );
  };

  // Handle date change
  const handleChange = (date: Date | null) => {
    if (date) {
      onChange(format(date, 'yyyy-MM-dd'));
    } else {
      onChange('');
    }
  };

  // Custom day class name for styling
  const getDayClassName = (date: Date) => {
    if (isDateOccupied(date)) {
      return 'react-datepicker__day--occupied';
    }
    return '';
  };

  // Start date for the calendar view (for checkout, open at checkin month)
  const openToDate = useMemo(() => {
    if (selectedDate) return selectedDate;
    if (isCheckout && checkinDate) {
      try {
        return parse(checkinDate, 'yyyy-MM-dd', new Date());
      } catch {
        return new Date();
      }
    }
    return new Date();
  }, [selectedDate, isCheckout, checkinDate]);

  return (
    <div className="booking-date-picker">
      <label className="block text-sm font-semibold text-slate-700 mb-2">
        {label} {required && '*'}
      </label>
      <DatePicker
        selected={selectedDate}
        onChange={handleChange}
        dateFormat="dd.MM.yyyy"
        locale="de"
        minDate={calculatedMinDate}
        maxDate={maxDate}
        excludeDates={occupiedDates}
        openToDate={openToDate}
        disabled={disabled}
        placeholderText={placeholder || 'Datum auswählen'}
        customInput={<CustomInput className={className} disabled={disabled} />}
        dayClassName={getDayClassName}
        showPopperArrow={false}
        popperPlacement="bottom-start"
        calendarStartDay={1} // Start week on Monday
        showMonthDropdown
        showYearDropdown
        dropdownMode="select"
        // Highlight today
        highlightDates={[new Date()]}
        // Custom styles will be added via CSS
      />
      <style>{`
        .react-datepicker {
          font-family: inherit;
          border: 1px solid #e2e8f0;
          border-radius: 0.75rem;
          box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
          overflow: hidden;
        }

        .react-datepicker__header {
          background: linear-gradient(to bottom, #f8fafc, #f1f5f9);
          border-bottom: 1px solid #e2e8f0;
          padding: 12px;
        }

        .react-datepicker__current-month {
          font-weight: 600;
          color: #1e293b;
          font-size: 1rem;
          margin-bottom: 8px;
        }

        .react-datepicker__day-names {
          margin-top: 8px;
        }

        .react-datepicker__day-name {
          color: #64748b;
          font-weight: 500;
          width: 2.5rem;
          margin: 0.15rem;
        }

        .react-datepicker__month {
          padding: 8px;
        }

        .react-datepicker__day {
          width: 2.5rem;
          line-height: 2.5rem;
          margin: 0.15rem;
          border-radius: 0.5rem;
          color: #334155;
          transition: all 0.15s ease;
        }

        .react-datepicker__day:hover:not(.react-datepicker__day--disabled):not(.react-datepicker__day--selected) {
          background-color: #e0f2fe;
          color: #0369a1;
        }

        .react-datepicker__day--selected {
          background-color: #2563eb !important;
          color: white !important;
          font-weight: 600;
        }

        .react-datepicker__day--keyboard-selected {
          background-color: #93c5fd;
          color: #1e3a8a;
        }

        .react-datepicker__day--today {
          font-weight: 700;
          color: #2563eb;
          border: 2px solid #2563eb;
        }

        .react-datepicker__day--today.react-datepicker__day--selected {
          color: white;
          border: none;
        }

        .react-datepicker__day--disabled,
        .react-datepicker__day--excluded {
          color: #cbd5e1 !important;
          background-color: #f8fafc !important;
          cursor: not-allowed;
          text-decoration: line-through;
        }

        .react-datepicker__day--occupied {
          background-color: #fee2e2 !important;
          color: #dc2626 !important;
          text-decoration: line-through;
        }

        .react-datepicker__day--outside-month {
          color: #94a3b8;
        }

        .react-datepicker__navigation {
          top: 14px;
        }

        .react-datepicker__navigation-icon::before {
          border-color: #64748b;
        }

        .react-datepicker__navigation:hover .react-datepicker__navigation-icon::before {
          border-color: #334155;
        }

        .react-datepicker__month-dropdown-container,
        .react-datepicker__year-dropdown-container {
          margin: 0 4px;
        }

        .react-datepicker__month-select,
        .react-datepicker__year-select {
          padding: 4px 8px;
          border: 1px solid #e2e8f0;
          border-radius: 0.375rem;
          background-color: white;
          color: #334155;
          font-size: 0.875rem;
          cursor: pointer;
        }

        .react-datepicker__month-select:focus,
        .react-datepicker__year-select:focus {
          outline: none;
          border-color: #2563eb;
          ring: 2px solid #93c5fd;
        }

        .react-datepicker-popper {
          z-index: 100 !important;
        }

        /* Highlight dates */
        .react-datepicker__day--highlighted {
          background-color: #fef3c7;
        }
      `}</style>
    </div>
  );
}
