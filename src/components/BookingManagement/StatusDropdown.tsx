import { useState, useRef, useEffect } from 'react';
import { Circle, CheckCircle, Clock, X, ChevronDown } from 'lucide-react';

interface StatusDropdownProps {
  currentStatus: string;
  bookingId: number;
  onStatusChange: (status: string) => void;
}

export default function StatusDropdown({ currentStatus, bookingId, onStatusChange }: StatusDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const statusOptions = [
    { value: 'reserviert', label: 'Reserviert', icon: Circle, color: 'blue' },
    { value: 'bestaetigt', label: 'Bestätigt', icon: CheckCircle, color: 'emerald' },
    { value: 'eingecheckt', label: 'Eingecheckt', icon: Clock, color: 'purple' },
    { value: 'ausgecheckt', label: 'Ausgecheckt', icon: CheckCircle, color: 'slate' },
    { value: 'storniert', label: 'Storniert', icon: X, color: 'red' },
  ];

  const currentOption = statusOptions.find(opt => opt.value === currentStatus);
  const CurrentIcon = currentOption?.icon || Circle;

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

  const handleSelect = (status: string) => {
    setIsOpen(false);
    if (status !== currentStatus) {
      onStatusChange(status);
    }
  };

  const getColorClasses = (color: string) => {
    const colors = {
      blue: 'bg-blue-100 text-blue-700 border-blue-200 hover:bg-blue-200',
      emerald: 'bg-emerald-100 text-emerald-700 border-emerald-200 hover:bg-emerald-200',
      purple: 'bg-purple-100 text-purple-700 border-purple-200 hover:bg-purple-200',
      slate: 'bg-slate-100 text-slate-700 border-slate-200 hover:bg-slate-200',
      red: 'bg-red-100 text-red-700 border-red-200 hover:bg-red-200',
    };
    return colors[color as keyof typeof colors] || colors.blue;
  };

  return (
    <div className="relative inline-block" ref={dropdownRef}>
      {/* Current Status Badge - Clickable */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold border transition-all cursor-pointer ${getColorClasses(currentOption?.color || 'blue')}`}
      >
        <CurrentIcon className="w-3 h-3" />
        {currentOption?.label}
        <ChevronDown className={`w-3 h-3 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
      </button>

      {/* Dropdown Menu */}
      {isOpen && (
        <div className="absolute top-full left-0 mt-2 bg-white rounded-lg shadow-lg border border-slate-200 py-1 z-50 min-w-[160px]">
          {statusOptions.map((option) => {
            const Icon = option.icon;
            return (
              <button
                key={option.value}
                onClick={() => handleSelect(option.value)}
                className={`w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-slate-50 transition-colors ${
                  option.value === currentStatus ? 'bg-slate-100 font-semibold' : ''
                }`}
              >
                <Icon className={`w-4 h-4 text-${option.color}-600`} />
                <span className={`text-${option.color}-700`}>{option.label}</span>
                {option.value === currentStatus && (
                  <CheckCircle className="w-3.5 h-3.5 text-blue-600 ml-auto" />
                )}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
