import { X, AlertCircle } from 'lucide-react';

interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  confirmText?: string;  // Alias für confirmLabel
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel?: () => void;
  onClose?: () => void;  // Alias für onCancel
  variant?: 'danger' | 'warning' | 'info';
  confirmButtonClass?: string;  // Custom button styling
}

export default function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmLabel,
  confirmText,
  cancelLabel = 'Abbrechen',
  onConfirm,
  onCancel,
  onClose,
  variant = 'danger',
  confirmButtonClass,
}: ConfirmDialogProps) {
  if (!isOpen) return null;

  // Aliase unterstützen
  const finalConfirmLabel = confirmText || confirmLabel || 'Bestätigen';
  const handleCancel = onClose || onCancel || (() => {});

  const variantStyles = {
    danger: {
      icon: 'bg-red-100 text-red-600',
      button: 'bg-red-600 hover:bg-red-700',
    },
    warning: {
      icon: 'bg-amber-100 text-amber-600',
      button: 'bg-amber-600 hover:bg-amber-700',
    },
    info: {
      icon: 'bg-blue-100 text-blue-600',
      button: 'bg-blue-600 hover:bg-blue-700',
    },
  };

  const styles = variantStyles[variant];
  const buttonClass = confirmButtonClass || styles.button;

  return (
    <div
      className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[100]"
      onClick={handleCancel}
    >
      <div
        className="bg-white rounded-2xl shadow-2xl max-w-md w-full mx-4 animate-in fade-in zoom-in duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-slate-200">
          <div className="flex items-center gap-3">
            <div className={`${styles.icon} p-2 rounded-lg`}>
              <AlertCircle className="w-6 h-6" />
            </div>
            <h2 className="text-xl font-bold text-slate-800">{title}</h2>
          </div>
          <button
            onClick={handleCancel}
            className="text-slate-400 hover:text-slate-600 transition-colors p-1 rounded-lg hover:bg-slate-100"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          <p className="text-slate-600 leading-relaxed">{message}</p>
        </div>

        {/* Actions */}
        <div className="flex items-center justify-end gap-3 p-6 bg-slate-50 rounded-b-2xl">
          <button
            onClick={handleCancel}
            className="px-4 py-2 bg-white border border-slate-300 text-slate-700 rounded-lg font-semibold hover:bg-slate-100 transition-colors"
          >
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            className={`px-4 py-2 ${buttonClass} text-white rounded-lg font-semibold transition-colors`}
          >
            {finalConfirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
