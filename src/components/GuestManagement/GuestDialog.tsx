import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, User, Mail, Phone, MapPin, Hash, FileText, Check, Loader2, Briefcase, MapPinned, Building2 } from 'lucide-react';

// HMR Force Reload - Padding Fix Applied

interface GuestDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  guestToEdit?: {
    id: number;
    vorname: string;
    nachname: string;
    email: string;
    telefon: string;
    dpolgMitglied: boolean;
    strasse?: string;
    plz?: string;
    ort?: string;
    mitgliedsnummer?: string;
    beruf?: string;
    bundesland?: string;
    dienststelle?: string;
    notizen?: string;
  };
}

interface FormData {
  vorname: string;
  nachname: string;
  email: string;
  telefon: string;
  strasse: string;
  plz: string;
  ort: string;
  dpolgMitglied: boolean;
  mitgliedsnummer: string;
  beruf: string;
  bundesland: string;
  dienststelle: string;
  notizen: string;
}

interface FormErrors {
  vorname?: string;
  nachname?: string;
  email?: string;
  telefon?: string;
}

interface Toast {
  type: 'success' | 'error';
  message: string;
}

export default function GuestDialog({ isOpen, onClose, guestToEdit, onSuccess }: GuestDialogProps) {
  const [formData, setFormData] = useState<FormData>({
    vorname: '',
    nachname: '',
    email: '',
    telefon: '',
    strasse: '',
    plz: '',
    ort: '',
    dpolgMitglied: false,
    mitgliedsnummer: '',
    beruf: '',
    bundesland: '',
    dienststelle: '',
    notizen: '',
  });

  const [errors, setErrors] = useState<FormErrors>({});
  const [loading, setLoading] = useState(false);
  const [toast, setToast] = useState<Toast | null>(null);
  const [emailValidating, setEmailValidating] = useState(false);

  // Initialize form with guest data when editing
  useEffect(() => {
    if (guestToEdit) {
      setFormData({
        vorname: guestToEdit.vorname,
        nachname: guestToEdit.nachname,
        email: guestToEdit.email,
        telefon: guestToEdit.telefon,
        strasse: guestToEdit.strasse || '',
        plz: guestToEdit.plz || '',
        ort: guestToEdit.ort || '',
        dpolgMitglied: guestToEdit.dpolgMitglied,
        mitgliedsnummer: guestToEdit.mitgliedsnummer || '',
        beruf: guestToEdit.beruf || '',
        bundesland: guestToEdit.bundesland || '',
        dienststelle: guestToEdit.dienststelle || '',
        notizen: guestToEdit.notizen || '',
      });
    } else {
      // Reset form for new guest
      setFormData({
        vorname: '',
        nachname: '',
        email: '',
        telefon: '',
        strasse: '',
        plz: '',
        ort: '',
        dpolgMitglied: false,
        mitgliedsnummer: '',
        beruf: '',
        bundesland: '',
        dienststelle: '',
        notizen: '',
      });
    }
    setErrors({});
  }, [guestToEdit, isOpen]);

  // Show toast notification
  const showToast = (type: 'success' | 'error', message: string) => {
    setToast({ type, message });
    setTimeout(() => setToast(null), 5000);
  };

  // Validate email using backend command
  const validateEmail = async (email: string): Promise<boolean> => {
    if (!email) {
      setErrors(prev => ({ ...prev, email: 'Email ist erforderlich' }));
      return false;
    }

    setEmailValidating(true);
    try {
      await invoke('validate_email_command', { email });
      setErrors(prev => {
        const newErrors = { ...prev };
        delete newErrors.email;
        return newErrors;
      });
      setEmailValidating(false);
      return true;
    } catch (err) {
      setErrors(prev => ({ ...prev, email: String(err) }));
      setEmailValidating(false);
      return false;
    }
  };

  // Handle form input changes
  const handleChange = (field: keyof FormData, value: string | boolean) => {
    setFormData(prev => ({ ...prev, [field]: value }));

    // Clear error for this field
    if (field in errors) {
      setErrors(prev => {
        const newErrors = { ...prev };
        delete newErrors[field as keyof FormErrors];
        return newErrors;
      });
    }

    // Auto-clear Mitgliedsnummer if not a member
    if (field === 'dpolgMitglied' && !value) {
      setFormData(prev => ({ ...prev, mitgliedsnummer: '' }));
    }
  };

  // Validate required fields
  const validateForm = (): boolean => {
    const newErrors: FormErrors = {};

    if (!formData.vorname.trim()) {
      newErrors.vorname = 'Vorname ist erforderlich';
    }

    if (!formData.nachname.trim()) {
      newErrors.nachname = 'Nachname ist erforderlich';
    }

    if (!formData.email.trim()) {
      newErrors.email = 'Email ist erforderlich';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validate required fields
    if (!validateForm()) {
      showToast('error', 'Bitte füllen Sie alle Pflichtfelder aus');
      return;
    }

    // Validate email with backend
    const isEmailValid = await validateEmail(formData.email);
    if (!isEmailValid) {
      showToast('error', 'Bitte geben Sie eine gültige Email-Adresse ein');
      return;
    }

    setLoading(true);

    try {
      const guestData = {
        vorname: formData.vorname.trim(),
        nachname: formData.nachname.trim(),
        email: formData.email.trim(),
        telefon: formData.telefon.trim(),
        dpolgMitglied: formData.dpolgMitglied,
        strasse: formData.strasse.trim() || null,
        plz: formData.plz.trim() || null,
        ort: formData.ort.trim() || null,
        mitgliedsnummer: formData.dpolgMitglied && formData.mitgliedsnummer.trim()
          ? formData.mitgliedsnummer.trim()
          : null,
        beruf: formData.beruf.trim() || null,
        bundesland: formData.bundesland.trim() || null,
        dienststelle: formData.dienststelle.trim() || null,
        notizen: formData.notizen.trim() || null,
      };

      if (guestToEdit) {
        // Update existing guest
        await invoke('update_guest_command', {
          id: guestToEdit.id,
          ...guestData,
        });
        showToast('success', 'Gast erfolgreich aktualisiert');
      } else {
        // Create new guest
        await invoke('create_guest_command', guestData);
        showToast('success', 'Gast erfolgreich erstellt');
      }

      onSuccess();
      handleClose();
    } catch (err) {
      console.error('Error saving guest:', err);
      showToast('error', `Fehler beim Speichern: ${String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  // Close dialog and reset form
  const handleClose = () => {
    setFormData({
      vorname: '',
      nachname: '',
      email: '',
      telefon: '',
      strasse: '',
      plz: '',
      ort: '',
      dpolgMitglied: false,
      mitgliedsnummer: '',
      beruf: '',
      bundesland: '',
      dienststelle: '',
      notizen: '',
    });
    setErrors({});
    setToast(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 backdrop-blur-sm z-40 transition-opacity"
        onClick={handleClose}
      />

      {/* Dialog */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl border border-slate-700 w-full max-w-2xl max-h-[90vh] overflow-hidden animate-in zoom-in-95 duration-200"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between dialog-header border-b border-slate-700/50 bg-slate-900/50">
            <div className="flex items-center gap-3">
              <div className="bg-gradient-to-br from-blue-500 to-blue-600 p-2 rounded-lg">
                <User className="w-5 h-5 text-white" />
              </div>
              <h2 className="text-xl font-bold text-white">
                {guestToEdit ? 'Gast bearbeiten' : 'Neuer Gast'}
              </h2>
            </div>
            <button
              onClick={handleClose}
              className="text-slate-400 hover:text-white transition-colors p-2 hover:bg-slate-700/50 rounded-lg"
              disabled={loading}
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="form-container form-section-spacing overflow-y-auto max-h-[calc(90vh-180px)]">
            {/* Section: Pflichtfelder */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white border-b border-slate-600 pb-2">
                Pflichtfelder
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 grid-gap-fix">
                {/* Vorname */}
                <div className="space-y-2">
                  <label htmlFor="vorname" className="block text-sm font-medium text-slate-200">
                    Vorname *
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <User className="w-5 h-5 text-slate-400" />
                    </div>
                    <input
                      id="vorname"
                      type="text"
                      value={formData.vorname}
                      onChange={(e) => handleChange('vorname', e.target.value)}
                      className={`w-full input-with-icon bg-slate-700 border ${
                        errors.vorname ? 'border-red-500' : 'border-slate-600'
                      } rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent`}
                      placeholder="Max"
                      disabled={loading}
                    />
                  </div>
                  {errors.vorname && (
                    <p className="text-sm text-red-400">{errors.vorname}</p>
                  )}
                </div>

                {/* Nachname */}
                <div className="space-y-2">
                  <label htmlFor="nachname" className="block text-sm font-medium text-slate-200">
                    Nachname *
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <User className="w-5 h-5 text-slate-400" />
                    </div>
                    <input
                      id="nachname"
                      type="text"
                      value={formData.nachname}
                      onChange={(e) => handleChange('nachname', e.target.value)}
                      className={`w-full input-with-icon bg-slate-700 border ${
                        errors.nachname ? 'border-red-500' : 'border-slate-600'
                      } rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent`}
                      placeholder="Mustermann"
                      disabled={loading}
                    />
                  </div>
                  {errors.nachname && (
                    <p className="text-sm text-red-400">{errors.nachname}</p>
                  )}
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 grid-gap-fix">
                {/* Email */}
                <div className="space-y-2">
                  <label htmlFor="email" className="block text-sm font-medium text-slate-200">
                    Email *
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <Mail className="w-5 h-5 text-slate-400" />
                    </div>
                    <input
                      id="email"
                      type="email"
                      value={formData.email}
                      onChange={(e) => handleChange('email', e.target.value)}
                      onBlur={() => formData.email && validateEmail(formData.email)}
                      className={`w-full input-with-icon bg-slate-700 border ${
                        errors.email ? 'border-red-500' : 'border-slate-600'
                      } rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent`}
                      placeholder="max@beispiel.de"
                      disabled={loading}
                    />
                    {emailValidating && (
                      <div className="absolute inset-y-0 right-0 pr-3 flex items-center">
                        <Loader2 className="w-5 h-5 text-blue-400 animate-spin" />
                      </div>
                    )}
                  </div>
                  {errors.email && (
                    <p className="text-sm text-red-400">{errors.email}</p>
                  )}
                </div>

                {/* Telefon */}
                <div className="space-y-2">
                  <label htmlFor="telefon" className="block text-sm font-medium text-slate-200">
                    Telefon
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <Phone className="w-5 h-5 text-slate-400" />
                    </div>
                    <input
                      id="telefon"
                      type="tel"
                      value={formData.telefon}
                      onChange={(e) => handleChange('telefon', e.target.value)}
                      className="w-full input-with-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                      placeholder="+49 123 456789"
                      disabled={loading}
                    />
                  </div>
                </div>
              </div>
            </div>

            {/* Section: Adresse */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white border-b border-slate-600 pb-2">
                Adresse
              </h3>

              <div className="grid grid-cols-1 gap-6">
                {/* Straße */}
                <div className="space-y-2">
                  <label htmlFor="strasse" className="block text-sm font-medium text-slate-200">
                    Straße
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <MapPin className="w-5 h-5 text-slate-400" />
                    </div>
                    <input
                      id="strasse"
                      type="text"
                      value={formData.strasse}
                      onChange={(e) => handleChange('strasse', e.target.value)}
                      className="w-full input-with-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                      placeholder="Musterstraße 123"
                      disabled={loading}
                    />
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                {/* PLZ */}
                <div className="space-y-2">
                  <label htmlFor="plz" className="block text-sm font-medium text-slate-200">
                    PLZ
                  </label>
                  <input
                    id="plz"
                    type="text"
                    value={formData.plz}
                    onChange={(e) => handleChange('plz', e.target.value)}
                    className="w-full input-no-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    placeholder="12345"
                    maxLength={5}
                    disabled={loading}
                  />
                </div>

                {/* Ort */}
                <div className="space-y-2 md:col-span-2">
                  <label htmlFor="ort" className="block text-sm font-medium text-slate-200">
                    Ort
                  </label>
                  <input
                    id="ort"
                    type="text"
                    value={formData.ort}
                    onChange={(e) => handleChange('ort', e.target.value)}
                    className="w-full input-no-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    placeholder="Musterstadt"
                    disabled={loading}
                  />
                </div>
              </div>
            </div>

            {/* Section: Berufliche Informationen */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white border-b border-slate-600 pb-2">
                Berufliche Informationen
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 grid-gap-fix">
                {/* Beruf */}
                <div className="space-y-2">
                  <label htmlFor="beruf" className="block text-sm font-medium text-slate-200">
                    Beruf
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <Briefcase className="w-5 h-5 text-slate-400" />
                    </div>
                    <input
                      id="beruf"
                      type="text"
                      value={formData.beruf}
                      onChange={(e) => handleChange('beruf', e.target.value)}
                      className="w-full input-with-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                      placeholder="Polizeibeamter"
                      disabled={loading}
                    />
                  </div>
                </div>

                {/* Bundesland */}
                <div className="space-y-2">
                  <label htmlFor="bundesland" className="block text-sm font-medium text-slate-200">
                    Bundesland
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <MapPinned className="w-5 h-5 text-slate-400" />
                    </div>
                    <input
                      id="bundesland"
                      type="text"
                      value={formData.bundesland}
                      onChange={(e) => handleChange('bundesland', e.target.value)}
                      className="w-full input-with-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                      placeholder="Bayern"
                      disabled={loading}
                    />
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 gap-6">
                {/* Dienststelle */}
                <div className="space-y-2">
                  <label htmlFor="dienststelle" className="block text-sm font-medium text-slate-200">
                    Dienststelle
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                      <Building2 className="w-5 h-5 text-slate-400" />
                    </div>
                    <input
                      id="dienststelle"
                      type="text"
                      value={formData.dienststelle}
                      onChange={(e) => handleChange('dienststelle', e.target.value)}
                      className="w-full input-with-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                      placeholder="Polizeiinspektion München"
                      disabled={loading}
                    />
                  </div>
                </div>
              </div>
            </div>

            {/* Section: Mitgliedschaft */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white border-b border-slate-600 pb-2">
                Mitgliedschaft
              </h3>

              <div className="flex items-center space-x-3 p-4 bg-slate-700/50 rounded-lg">
                <label className="flex items-center gap-3 cursor-pointer group">
                  <div className="relative">
                    <input
                      type="checkbox"
                      checked={formData.dpolgMitglied}
                      onChange={(e) => handleChange('dpolgMitglied', e.target.checked)}
                      className="sr-only peer"
                      disabled={loading}
                    />
                    <div className="w-11 h-6 bg-slate-700 border border-slate-600 rounded-full peer-checked:bg-blue-500 transition-colors"></div>
                    <div className="absolute left-1 top-1 w-4 h-4 bg-white rounded-full transition-transform peer-checked:translate-x-5"></div>
                  </div>
                  <span className="text-sm font-medium text-slate-200 group-hover:text-white transition-colors">
                    DPolG Mitglied
                  </span>
                </label>
              </div>

              {formData.dpolgMitglied && (
                <div className="grid grid-cols-1 gap-6">
                  {/* Mitgliedsnummer */}
                  <div className="space-y-2">
                    <label htmlFor="mitgliedsnummer" className="block text-sm font-medium text-slate-200">
                      Mitgliedsnummer
                    </label>
                    <div className="relative">
                      <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                        <Hash className="w-5 h-5 text-slate-400" />
                      </div>
                      <input
                        id="mitgliedsnummer"
                        type="text"
                        value={formData.mitgliedsnummer}
                        onChange={(e) => handleChange('mitgliedsnummer', e.target.value)}
                        className="w-full input-with-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        placeholder="12345678"
                        disabled={loading}
                      />
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Section: Notizen */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white border-b border-slate-600 pb-2">
                Notizen
              </h3>

              <div className="grid grid-cols-1 gap-6">
                {/* Notizen Textarea */}
                <div className="space-y-2">
                  <label htmlFor="notizen" className="block text-sm font-medium text-slate-200">
                    Notizen
                  </label>
                  <div className="relative">
                    <div className="absolute top-3 left-0 pl-3 pointer-events-none">
                      <FileText className="w-5 h-5 text-slate-400" />
                    </div>
                    <textarea
                      id="notizen"
                      value={formData.notizen}
                      onChange={(e) => handleChange('notizen', e.target.value)}
                      rows={4}
                      className="w-full input-with-icon bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                      placeholder="Zusätzliche Informationen..."
                      disabled={loading}
                    />
                  </div>
                </div>
              </div>
            </div>

            {/* Footer: Buttons */}
            <div className="flex justify-end gap-3 pt-6 border-t border-slate-600">
              <button
                type="button"
                onClick={handleClose}
                className="btn-secondary text-slate-300 hover:text-white hover:bg-slate-700/50 rounded-lg transition-all font-medium"
                disabled={loading}
              >
                Abbrechen
              </button>
              <button
                type="submit"
                disabled={loading || emailValidating}
                className="flex items-center gap-2 btn-primary bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white rounded-lg font-medium transition-all shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? (
                  <>
                    <Loader2 className="w-5 h-5 animate-spin" />
                    <span>Speichert...</span>
                  </>
                ) : (
                  <>
                    <Check className="w-5 h-5" />
                    <span>Speichern</span>
                  </>
                )}
              </button>
            </div>
          </form>
        </div>
      </div>

      {/* Toast Notification */}
      {toast && (
        <div className="fixed bottom-4 right-4 z-[60] animate-in slide-in-from-bottom-2 duration-300">
          <div className={`px-6 py-3 rounded-lg shadow-2xl flex items-center gap-3 ${
            toast.type === 'success'
              ? 'bg-emerald-500 text-white'
              : 'bg-red-500 text-white'
          }`}>
            {toast.type === 'success' ? (
              <Check className="w-5 h-5" />
            ) : (
              <X className="w-5 h-5" />
            )}
            <span className="font-medium">{toast.message}</span>
          </div>
        </div>
      )}
    </>
  );
}
