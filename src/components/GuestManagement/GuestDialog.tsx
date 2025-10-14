import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, User, Mail, Phone, MapPin, Hash, FileText, Check, Loader2, Briefcase, MapPinned, Building2, Wallet, Plus, Minus, Clock, History } from 'lucide-react';
import { useData } from '../../context/DataContext';

// HMR Force Reload - Padding Fix Applied

interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon: string;
  dpolg_mitglied: boolean;
  strasse?: string;
  plz?: string;
  ort?: string;
  mitgliedsnummer?: string;
  notizen?: string;
}

// Credit System Types
interface GuestCreditTransaction {
  id: number;
  guestId: number;
  amount: number;
  transactionType: 'added' | 'used' | 'expired' | 'refund';
  bookingId?: number | null;
  notes?: string | null;
  createdBy: string;
  createdAt: string;
}

interface GuestCreditBalance {
  guestId: number;
  balance: number;
  transactionCount: number;
}

interface GuestDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  guest?: Guest;
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

export default function GuestDialog({ isOpen, onClose, guest, onSuccess }: GuestDialogProps) {
  const { createGuest, updateGuest } = useData();

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

  // Credit Management State
  const [showCreditModal, setShowCreditModal] = useState(false);
  const [creditBalance, setCreditBalance] = useState<GuestCreditBalance | null>(null);
  const [creditTransactions, setCreditTransactions] = useState<GuestCreditTransaction[]>([]);
  const [creditLoading, setCreditLoading] = useState(false);
  const [creditAmount, setCreditAmount] = useState('');
  const [creditNotes, setCreditNotes] = useState('');
  const [creditType, setCreditType] = useState<'added' | 'expired' | 'refund'>('added');

  // Initialize form with guest data when editing
  useEffect(() => {
    if (guest) {
      setFormData({
        vorname: guest.vorname,
        nachname: guest.nachname,
        email: guest.email,
        telefon: guest.telefon,
        strasse: guest.strasse || '',
        plz: guest.plz || '',
        ort: guest.ort || '',
        dpolgMitglied: guest.dpolg_mitglied,
        mitgliedsnummer: guest.mitgliedsnummer || '',
        beruf: '',
        bundesland: '',
        dienststelle: '',
        notizen: guest.notizen || '',
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
    setToast(null);
  }, [guest, isOpen]);

  // Show toast notification
  const showToast = (type: 'success' | 'error', message: string) => {
    setToast({ type, message });
    setTimeout(() => setToast(null), 5000);
  };

  // Load credit data when editing guest
  useEffect(() => {
    if (guest && isOpen) {
      loadCreditData();
    }
  }, [guest, isOpen]);

  // Load credit balance and transactions
  const loadCreditData = async () => {
    if (!guest) return;

    setCreditLoading(true);
    try {
      const [balance, transactions] = await Promise.all([
        invoke<GuestCreditBalance>('get_guest_credit_balance', { guestId: guest.id }),
        invoke<GuestCreditTransaction[]>('get_guest_credit_transactions', { guestId: guest.id }),
      ]);

      setCreditBalance(balance);
      setCreditTransactions(transactions);
    } catch (err) {
      console.error('Error loading credit data:', err);
      showToast('error', `Fehler beim Laden des Guthabens: ${String(err)}`);
    } finally {
      setCreditLoading(false);
    }
  };

  // Add credit to guest
  const handleAddCredit = async () => {
    if (!guest) return;

    const amount = parseFloat(creditAmount);
    if (isNaN(amount) || amount === 0) {
      showToast('error', 'Bitte geben Sie einen gültigen Betrag ein');
      return;
    }

    if (!creditNotes.trim()) {
      showToast('error', 'Bitte geben Sie eine Begründung ein');
      return;
    }

    setCreditLoading(true);
    try {
      await invoke('add_guest_credit', {
        guestId: guest.id,
        amount,
        transactionType: creditType,
        notes: creditNotes.trim(),
        createdBy: 'System',
      });

      // Reload credit data
      await loadCreditData();

      // Reset form
      setCreditAmount('');
      setCreditNotes('');
      setCreditType('added');

      showToast('success', 'Guthaben erfolgreich hinzugefügt');
    } catch (err) {
      console.error('Error adding credit:', err);
      showToast('error', `Fehler beim Hinzufügen: ${String(err)}`);
    } finally {
      setCreditLoading(false);
    }
  };

  // Format date for display
  const formatDate = (dateStr: string): string => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString('de-DE', {
        day: '2-digit',
        month: '2-digit',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return dateStr;
    }
  };

  // Format currency
  const formatCurrency = (amount: number): string => {
    return new Intl.NumberFormat('de-DE', {
      style: 'currency',
      currency: 'EUR',
    }).format(amount);
  };

  // Get transaction type label
  const getTransactionTypeLabel = (type: string): string => {
    const labels: Record<string, string> = {
      added: 'Gutgeschrieben',
      used: 'Verrechnet',
      expired: 'Verfallen',
      refund: 'Rückerstattet',
    };
    return labels[type] || type;
  };

  // Get transaction type color
  const getTransactionTypeColor = (type: string): string => {
    const colors: Record<string, string> = {
      added: 'text-emerald-400',
      used: 'text-red-400',
      expired: 'text-amber-400',
      refund: 'text-blue-400',
    };
    return colors[type] || 'text-slate-400';
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

      if (guest) {
        // Update existing guest
        await updateGuest(guest.id, guestData);
        showToast('success', 'Gast erfolgreich aktualisiert');
      } else {
        // Create new guest
        await createGuest(guestData);
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
                {guest ? 'Gast bearbeiten' : 'Neuer Gast'}
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
                    DPolG Stiftung Mitglied
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

            {/* Section: Guthaben (only for existing guests) */}
            {guest && (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-semibold text-white border-b border-slate-600 pb-2 flex-1">
                    Guthaben
                  </h3>
                </div>

                {/* Current Balance Display */}
                <div className="p-4 bg-gradient-to-r from-emerald-500/10 to-emerald-600/10 border border-emerald-500/30 rounded-lg">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="p-2 bg-emerald-500/20 rounded-lg">
                        <Wallet className="w-5 h-5 text-emerald-400" />
                      </div>
                      <div>
                        <p className="text-sm text-slate-300">Aktuelles Guthaben</p>
                        {creditLoading ? (
                          <div className="flex items-center gap-2 mt-1">
                            <Loader2 className="w-4 h-4 text-emerald-400 animate-spin" />
                            <span className="text-sm text-slate-400">Lädt...</span>
                          </div>
                        ) : (
                          <p className={`text-2xl font-bold ${
                            (creditBalance?.balance || 0) > 0 ? 'text-emerald-400' : 'text-slate-300'
                          }`}>
                            {formatCurrency(creditBalance?.balance || 0)}
                          </p>
                        )}
                      </div>
                    </div>
                    <button
                      type="button"
                      onClick={() => setShowCreditModal(true)}
                      className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors text-sm font-medium"
                    >
                      <History className="w-4 h-4" />
                      Guthaben verwalten
                    </button>
                  </div>

                  {/* Transaction Count Info */}
                  {creditBalance && creditBalance.transactionCount > 0 && (
                    <div className="mt-3 pt-3 border-t border-emerald-500/20">
                      <p className="text-xs text-slate-400">
                        {creditBalance.transactionCount} {creditBalance.transactionCount === 1 ? 'Transaktion' : 'Transaktionen'}
                      </p>
                    </div>
                  )}
                </div>
              </div>
            )}

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

      {/* Credit Management Modal */}
      {showCreditModal && guest && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 bg-black/60 backdrop-blur-sm z-[70] transition-opacity"
            onClick={() => setShowCreditModal(false)}
          />

          {/* Modal */}
          <div className="fixed inset-0 z-[80] flex items-center justify-center p-4">
            <div
              className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl border border-slate-700 w-full max-w-4xl max-h-[90vh] overflow-hidden animate-in zoom-in-95 duration-200"
              onClick={(e) => e.stopPropagation()}
            >
              {/* Modal Header */}
              <div className="flex items-center justify-between p-6 border-b border-slate-700/50 bg-slate-900/50">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-gradient-to-br from-emerald-500 to-emerald-600 rounded-lg">
                    <Wallet className="w-5 h-5 text-white" />
                  </div>
                  <div>
                    <h2 className="text-xl font-bold text-white">Guthaben verwalten</h2>
                    <p className="text-sm text-slate-400 mt-1">
                      {guest.vorname} {guest.nachname} - Aktuelles Guthaben: {formatCurrency(creditBalance?.balance || 0)}
                    </p>
                  </div>
                </div>
                <button
                  onClick={() => setShowCreditModal(false)}
                  className="text-slate-400 hover:text-white transition-colors p-2 hover:bg-slate-700/50 rounded-lg"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>

              {/* Modal Content */}
              <div className="p-6 overflow-y-auto max-h-[calc(90vh-180px)] space-y-6">
                {/* Add Credit Form */}
                <div className="bg-slate-800/50 rounded-xl border border-slate-700 p-4">
                  <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                    <Plus className="w-5 h-5 text-emerald-400" />
                    Guthaben hinzufügen
                  </h3>

                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    {/* Transaction Type */}
                    <div className="space-y-2">
                      <label className="block text-sm font-medium text-slate-200">
                        Typ
                      </label>
                      <select
                        value={creditType}
                        onChange={(e) => setCreditType(e.target.value as 'added' | 'expired' | 'refund')}
                        className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
                        disabled={creditLoading}
                      >
                        <option value="added">Gutschrift</option>
                        <option value="refund">Rückerstattung</option>
                        <option value="expired">Verfall</option>
                      </select>
                    </div>

                    {/* Amount */}
                    <div className="space-y-2">
                      <label className="block text-sm font-medium text-slate-200">
                        Betrag (€)
                      </label>
                      <input
                        type="number"
                        step="0.01"
                        value={creditAmount}
                        onChange={(e) => setCreditAmount(e.target.value)}
                        placeholder="50.00"
                        className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent"
                        disabled={creditLoading}
                      />
                    </div>

                    {/* Add Button */}
                    <div className="space-y-2">
                      <label className="block text-sm font-medium text-slate-200 opacity-0">
                        Action
                      </label>
                      <button
                        type="button"
                        onClick={handleAddCredit}
                        disabled={creditLoading || !creditAmount || !creditNotes.trim()}
                        className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        {creditLoading ? (
                          <>
                            <Loader2 className="w-4 h-4 animate-spin" />
                            <span>Lädt...</span>
                          </>
                        ) : (
                          <>
                            <Plus className="w-4 h-4" />
                            <span>Hinzufügen</span>
                          </>
                        )}
                      </button>
                    </div>
                  </div>

                  {/* Notes */}
                  <div className="mt-4 space-y-2">
                    <label className="block text-sm font-medium text-slate-200">
                      Begründung *
                    </label>
                    <textarea
                      value={creditNotes}
                      onChange={(e) => setCreditNotes(e.target.value)}
                      rows={2}
                      placeholder="z.B. Gutschrift für Beschwerde, Erstattung für Stornierung, ..."
                      className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:border-transparent resize-none"
                      disabled={creditLoading}
                    />
                  </div>
                </div>

                {/* Transaction History */}
                <div>
                  <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                    <Clock className="w-5 h-5 text-blue-400" />
                    Transaktionshistorie
                  </h3>

                  {creditTransactions.length === 0 ? (
                    <div className="text-center py-8 text-slate-400">
                      <History className="w-12 h-12 mx-auto mb-3 opacity-50" />
                      <p>Noch keine Transaktionen vorhanden</p>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {creditTransactions.map((transaction) => (
                        <div
                          key={transaction.id}
                          className="p-4 bg-slate-800/50 rounded-lg border border-slate-700 hover:border-slate-600 transition-colors"
                        >
                          <div className="flex items-start justify-between">
                            <div className="flex-1">
                              <div className="flex items-center gap-3 mb-2">
                                <span className={`text-sm font-medium ${getTransactionTypeColor(transaction.transactionType)}`}>
                                  {getTransactionTypeLabel(transaction.transactionType)}
                                </span>
                                <span className="text-xs text-slate-500">
                                  {formatDate(transaction.createdAt)}
                                </span>
                              </div>
                              {transaction.notes && (
                                <p className="text-sm text-slate-300 mb-2">{transaction.notes}</p>
                              )}
                              <div className="flex items-center gap-2 text-xs text-slate-500">
                                <span>Erstellt von: {transaction.createdBy}</span>
                                {transaction.bookingId && (
                                  <>
                                    <span>•</span>
                                    <span>Buchung #{transaction.bookingId}</span>
                                  </>
                                )}
                              </div>
                            </div>
                            <div className={`text-right ${
                              transaction.amount > 0 ? 'text-emerald-400' : 'text-red-400'
                            }`}>
                              <p className="text-lg font-bold">
                                {transaction.amount > 0 ? '+' : ''}{formatCurrency(transaction.amount)}
                              </p>
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>

              {/* Modal Footer */}
              <div className="flex justify-end gap-3 p-6 border-t border-slate-700/50 bg-slate-900/50">
                <button
                  onClick={() => setShowCreditModal(false)}
                  className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors font-medium"
                >
                  Schließen
                </button>
              </div>
            </div>
          </div>
        </>
      )}
    </>
  );
}
