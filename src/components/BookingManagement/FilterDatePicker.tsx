import { forwardRef, useMemo } from 'react';
import DatePicker, { registerLocale } from 'react-datepicker';
import { de } from 'date-fns/locale';
import { format, parse } from 'date-fns';
import 'react-datepicker/dist/react-datepicker.css';

// Register German locale
registerLocale('de', de);

interface FilterDatePickerProps {
  value: string; // YYYY-MM-DD format
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  disabled?: boolean;
  minDate?: Date;
  maxDate?: Date;
}

// Custom input component for inline styling
const CustomInput = forwardRef<HTMLInputElement, { value?: string; onClick?: () => void; placeholder?: string; disabled?: boolean; className?: string }>(
  ({ value, onClick, placeholder, disabled, className }, ref) => (
    <input
      ref={ref}
      type="text"
      value={value}
      onClick={onClick}
      placeholder={placeholder || 'Datum'}
      disabled={disabled}
      readOnly
      className={className || 'px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 cursor-pointer bg-white w-36'}
    />
  )
);
CustomInput.displayName = 'FilterCustomInput';

export default function FilterDatePicker({
  value,
  onChange,
  placeholder,
  className,
  disabled = false,
  minDate,
  maxDate,
}: FilterDatePickerProps) {
  // Convert string value to Date object
  const selectedDate = useMemo(() => {
    if (!value) return null;
    try {
      return parse(value, 'yyyy-MM-dd', new Date());
    } catch {
      return null;
    }
  }, [value]);

  // Handle date change
  const handleChange = (date: Date | null) => {
    if (date) {
      onChange(format(date, 'yyyy-MM-dd'));
    } else {
      onChange('');
    }
  };

  return (
    <div className="filter-date-picker">
      <DatePicker
        selected={selectedDate}
        onChange={handleChange}
        dateFormat="dd.MM.yyyy"
        locale="de"
        disabled={disabled}
        minDate={minDate}
        maxDate={maxDate}
        placeholderText={placeholder || 'Datum'}
        customInput={<CustomInput className={className} disabled={disabled} placeholder={placeholder} />}
        showPopperArrow={false}
        popperPlacement="bottom-start"
        calendarStartDay={1}
        showMonthDropdown
        showYearDropdown
        dropdownMode="select"
        isClearable
        highlightDates={[new Date()]}
      />
      <style>{`
        .filter-date-picker .react-datepicker {
          font-family: inherit;
          border: 1px solid #e2e8f0;
          border-radius: 0.75rem;
          box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
          overflow: hidden;
        }

        .filter-date-picker .react-datepicker__header {
          background: linear-gradient(to bottom, #f8fafc, #f1f5f9);
          border-bottom: 1px solid #e2e8f0;
          padding: 12px;
        }

        .filter-date-picker .react-datepicker__current-month {
          font-weight: 600;
          color: #1e293b;
          font-size: 1rem;
          margin-bottom: 8px;
        }

        .filter-date-picker .react-datepicker__day-name {
          color: #64748b;
          font-weight: 500;
          width: 2.5rem;
          margin: 0.15rem;
        }

        .filter-date-picker .react-datepicker__month {
          padding: 8px;
        }

        .filter-date-picker .react-datepicker__day {
          width: 2.5rem;
          line-height: 2.5rem;
          margin: 0.15rem;
          border-radius: 0.5rem;
          color: #334155;
          transition: all 0.15s ease;
        }

        .filter-date-picker .react-datepicker__day:hover:not(.react-datepicker__day--disabled):not(.react-datepicker__day--selected) {
          background-color: #e0f2fe;
          color: #0369a1;
        }

        .filter-date-picker .react-datepicker__day--selected {
          background-color: #2563eb !important;
          color: white !important;
          font-weight: 600;
        }

        .filter-date-picker .react-datepicker__day--today {
          font-weight: 700;
          color: #2563eb;
          border: 2px solid #2563eb;
        }

        .filter-date-picker .react-datepicker__day--today.react-datepicker__day--selected {
          color: white;
          border: none;
        }

        .filter-date-picker .react-datepicker__day--outside-month {
          color: #94a3b8;
        }

        .filter-date-picker .react-datepicker__navigation {
          top: 14px;
        }

        .filter-date-picker .react-datepicker__navigation-icon::before {
          border-color: #64748b;
        }

        .filter-date-picker .react-datepicker__month-select,
        .filter-date-picker .react-datepicker__year-select {
          padding: 4px 8px;
          border: 1px solid #e2e8f0;
          border-radius: 0.375rem;
          background-color: white;
          color: #334155;
          font-size: 0.875rem;
          cursor: pointer;
        }

        .filter-date-picker .react-datepicker-popper {
          z-index: 100 !important;
        }

        .filter-date-picker .react-datepicker__close-icon {
          right: 8px;
        }

        .filter-date-picker .react-datepicker__close-icon::after {
          background-color: #94a3b8;
          font-size: 14px;
          height: 18px;
          width: 18px;
          line-height: 16px;
        }

        .filter-date-picker .react-datepicker__close-icon:hover::after {
          background-color: #64748b;
        }
      `}</style>
    </div>
  );
}
