// DEBUG LOGS ADDED - Version 1.0
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  X, Calendar, Users, MapPin, Mail, Phone, DollarSign, Tag,
  UserCheck, ShoppingBag, Percent, Edit2, XCircle, FileText,
  Clock, Home, Send, CheckCircle, AlertCircle, Euro, Download,
  FolderOpen, Eye, AlertTriangle, Wallet
} from 'lucide-react';
import { format } from 'date-fns';
import { de } from 'date-fns/locale';
import PaymentDropdown from './PaymentDropdown';
import { useData } from '../../context/DataContext';
import BookingReminders from '../Reminders/BookingReminders';
import { formatCalculatedServicePrice, getServicePriceIcon } from '../../utils/priceFormatting';

interface BookingDetailsProps {
  bookingId: number;
  isOpen: boolean;
  onClose: () => void;
  onEdit?: () => void;
}

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  ort: string;
  capacity: number;
  price_member: number;
  price_non_member: number;
  schluesselcode?: string;
}

interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon?: string;
  strasse?: string;
  plz?: string;
  ort?: string;
  dpolg_mitglied: boolean;
  mitgliedsnummer?: string;
  notizen?: string;
}

interface PaymentRecipient {
  id: number;
  name: string;
  company?: string;
  street?: string;
  plz?: string;
  city?: string;
  country: string;
  contact_person?: string;
  notes?: string;
}

interface Booking {
  id: number;
  room_id: number;
  guest_id: number;
  reservierungsnummer: string;
  checkin_date: string;
  checkout_date: string;
  anzahl_gaeste: number;
  anzahl_begleitpersonen: number;
  status: string;
  grundpreis: number;
  services_preis: number;
  rabatt_preis: number;
  gesamtpreis: number;
  anzahl_naechte: number;
  bemerkungen?: string;
  bezahlt: boolean;
  bezahlt_am?: string | null;
  zahlungsmethode?: string | null;
  rechnung_versendet_am?: string | null;
  rechnung_versendet_an?: string | null;
  mahnung_gesendet_am?: string | null;
  ist_stiftungsfall: boolean;
  payment_recipient_id?: number | null;
  room: Room;
  guest: Guest;
}

interface AccompanyingGuest {
  id: number;
  vorname: string;
  nachname: string;
  geburtsdatum?: string;
}

interface AdditionalService {
  id: number;
  service_name: string;
  service_price: number;
  // Neue Felder für prozentuale Services:
  price_type: 'fixed' | 'percent';
  original_value: number;
  applies_to: 'overnight_price' | 'total_price';
}

interface Discount {
  id: number;
  discount_name: string;
  discount_type: string;
  discount_value: number;
}

interface InvoicePdfInfo {
  filename: string;
  path: string;
  size_bytes: number;
  created_at: number;
  reservierungsnummer: string;
}

export default function BookingDetails({ bookingId, isOpen, onClose, onEdit }: BookingDetailsProps) {
  const { updateBookingPayment, refreshBookings } = useData();
  const [booking, setBooking] = useState<Booking | null>(null);
  const [accompanyingGuests, setAccompanyingGuests] = useState<AccompanyingGuest[]>([]);
  const [services, setServices] = useState<AdditionalService[]>([]);
  const [discounts, setDiscounts] = useState<Discount[]>([]);
  const [invoicePdfs, setInvoicePdfs] = useState<InvoicePdfInfo[]>([]);
  const [paymentRecipient, setPaymentRecipient] = useState<PaymentRecipient | null>(null);
  const [creditUsed, setCreditUsed] = useState<number>(0);
  const [loading, setLoading] = useState(true);
  const [showCancelDialog, setShowCancelDialog] = useState(false);
  const [showSuccessDialog, setShowSuccessDialog] = useState<{ show: boolean; message: string }>({ show: false, message: '' });
  const [showErrorDialog, setShowErrorDialog] = useState<{ show: boolean; message: string }>({ show: false, message: '' });

  useEffect(() => {
    if (isOpen && bookingId) {
      loadBookingDetails();
    }
  }, [isOpen, bookingId]);

  const loadBookingDetails = async () => {
    try {
      setLoading(true);

      // Load booking WITH nested guest and room details
      const bookingData = await invoke<Booking>('get_booking_with_details_by_id_command', { id: bookingId });

      if (!bookingData) {
        throw new Error('Buchungsdaten konnten nicht geladen werden');
      }
      if (!bookingData.guest) {
        throw new Error('Gastdaten fehlen');
      }
      if (!bookingData.room) {
        throw new Error('Zimmerdaten fehlen');
      }

      setBooking(bookingData);

      // Load accompanying guests
      const guestsData = await invoke<AccompanyingGuest[]>('get_booking_accompanying_guests_command', {
        bookingId,
      });
      setAccompanyingGuests(guestsData);

      // Load services
      const servicesData = await invoke<AdditionalService[]>('get_booking_services_command', {
        bookingId,
      });
      setServices(servicesData);

      // Load discounts
      const discountsData = await invoke<Discount[]>('get_booking_discounts_command', {
        bookingId,
      });
      setDiscounts(discountsData);

      // Load guest credit usage
      try {
        const credit = await invoke<number>('get_booking_credit_usage', { bookingId });
        setCreditUsed(credit || 0);
      } catch (error) {
        console.error('❌ Fehler beim Laden des Gast-Guthabens:', error);
        setCreditUsed(0);
      }

      // Load invoice PDFs
      const pdfsData = await invoke<InvoicePdfInfo[]>('get_invoice_pdfs_for_booking_command', {
        bookingId,
      });
      setInvoicePdfs(pdfsData);

      // Load payment recipient if booking has one
      console.log('🔍 [BookingDetails] bookingData.payment_recipient_id:', bookingData.payment_recipient_id, 'type:', typeof bookingData.payment_recipient_id);

      if (bookingData.payment_recipient_id) {
        console.log('✅ [BookingDetails] Loading payment recipient with ID:', bookingData.payment_recipient_id);
        try {
          const recipientData = await invoke<PaymentRecipient>('get_payment_recipient', {
            id: bookingData.payment_recipient_id,
          });
          console.log('✅ [BookingDetails] Payment recipient loaded:', recipientData);
          setPaymentRecipient(recipientData);
        } catch (error) {
          console.error('❌ [BookingDetails] Fehler beim Laden des Zahlungsempfängers:', error);
          // Don't fail the whole load if payment recipient fails
        }
      } else {
        console.log('⚠️ [BookingDetails] No payment_recipient_id - setting to null');
        setPaymentRecipient(null);
      }
    } catch (error) {
      console.error('Fehler beim Laden der Buchungsdetails:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCancelBooking = async () => {
    if (!booking) return;

    try {
      await invoke('cancel_booking_command', { id: bookingId });
      setShowCancelDialog(false);
      loadBookingDetails(); // Reload to show updated status
      setShowSuccessDialog({ show: true, message: 'Buchung erfolgreich storniert!' });

      // 🔄 SYNC zu Turso (Mobile App) - Entferne stornierte Buchung
      // Wichtig: Backend filtert automatisch stornierte Buchungen (WHERE status != 'storniert')
      // Wir müssen nur beide Dates (checkin + checkout) synchronisieren
      console.log('🔄 [BookingDetails] Buchung storniert - Sync zu Mobile App für checkin + checkout');

      // FIX (2025-10-21): Nutze sync_affected_dates um BEIDE Daten zu synchronisieren
      // Verhindert Bug wo stornierte Buchungen teilweise im PDF bleiben
      if (booking.checkout_date && booking.checkin_date) {
        invoke('sync_affected_dates', {
          bookingId: booking.id,
          checkinDate: booking.checkin_date,
          oldCheckout: booking.checkout_date,
          newCheckout: booking.checkout_date
        });
      }

      console.log('✅ [BookingDetails] Mobile App Sync getriggert - stornierte Buchung wird entfernt');
    } catch (error) {
      console.error('Fehler beim Stornieren der Buchung:', error);
      setShowCancelDialog(false);
      setShowErrorDialog({ show: true, message: 'Fehler beim Stornieren der Buchung' });
    }
  };

  const handleSendConfirmationEmail = async () => {
    console.log('🔵 [BookingDetails] handleSendConfirmationEmail CALLED', { bookingId, hasBooking: !!booking });
    if (!booking) {
      console.log('❌ [BookingDetails] No booking - aborting');
      return;
    }

    // ✅ LOADING TOAST - bleibt offen bis fertig
    const toastId = `confirmation-${bookingId}`;
    console.log('🍞 [BookingDetails] Dispatching loading toast', { toastId });
    window.dispatchEvent(new CustomEvent('show-toast', {
      detail: {
        id: toastId,
        message: '📧 Bestätigung wird versendet...',
        type: 'loading'
      }
    }));

    // ✅ ASYNC BACKGROUND PROCESS
    const sendEmailInBackground = async () => {
      try {
        console.log('📧 [BookingDetails] Invoking send_confirmation_email_command', { bookingId });
        const result = await invoke<string>('send_confirmation_email_command', { bookingId });
        console.log('✅ [BookingDetails] Email sent successfully', { result });

        // ✅ UPDATE LOADING TOAST mit Success
        console.log('🍞 [BookingDetails] Dispatching success toast', { toastId });
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId,
            message: `✅ ${result}`,
            type: 'success',
            duration: 3000
          }
        }));

        // ✅ REMINDER BADGE UPDATE: Trigger Bell-Icon Count Refresh + Optimistic Update
        console.log('📤 [BookingDetails] Dispatching reminder-updated event with type: auto_confirmation');
        window.dispatchEvent(new CustomEvent('reminder-updated', {
          detail: { reminderType: 'auto_confirmation' }
        }));
        console.log('✅ [BookingDetails] reminder-updated event dispatched');
      } catch (error) {
        // ✅ UPDATE LOADING TOAST mit Error
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId,
            message: `❌ Fehler beim Senden: ${error}`,
            type: 'error',
            duration: 5000
          }
        }));
      }
    };

    // Start background process (no await!)
    sendEmailInBackground();
  };

  const handleSendReminderEmail = async () => {
    if (!booking) return;

    // ✅ LOADING TOAST - bleibt offen bis fertig
    const toastId = `reminder-${bookingId}`;
    window.dispatchEvent(new CustomEvent('show-toast', {
      detail: {
        id: toastId,
        message: '📧 Erinnerung wird versendet...',
        type: 'loading'
      }
    }));

    // ✅ ASYNC BACKGROUND PROCESS
    const sendEmailInBackground = async () => {
      try {
        const result = await invoke<string>('send_reminder_email_command', { bookingId });

        // ✅ UPDATE LOADING TOAST mit Success
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId,
            message: `✅ ${result}`,
            type: 'success',
            duration: 3000
          }
        }));
      } catch (error) {
        // ✅ UPDATE LOADING TOAST mit Error
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId,
            message: `❌ Fehler beim Senden: ${error}`,
            type: 'error',
            duration: 5000
          }
        }));
      }
    };

    // Start background process (no await!)
    sendEmailInBackground();
  };

  const handleSendInvoiceEmail = async () => {
    console.log('🟢 [BookingDetails] handleSendInvoiceEmail CALLED', { bookingId, hasBooking: !!booking });
    if (!booking) {
      console.log('❌ [BookingDetails] No booking - aborting');
      return;
    }

    // ✅ LOADING TOAST - bleibt offen bis fertig
    const toastId = `invoice-${bookingId}`;
    console.log('🍞 [BookingDetails] Dispatching loading toast', { toastId });
    window.dispatchEvent(new CustomEvent('show-toast', {
      detail: {
        id: toastId,
        message: '📧 Rechnung wird generiert und versendet...',
        type: 'loading'
      }
    }));

    // ✅ ASYNC BACKGROUND PROCESS
    const sendInvoiceInBackground = async () => {
      try {
        console.log('📧 [BookingDetails] Invoking generate_and_send_invoice_command', { bookingId });
        // Verwende generate_and_send_invoice_command statt send_invoice_email_command
        // damit PDF automatisch generiert und angehängt wird
        const result = await invoke<string>('generate_and_send_invoice_command', { bookingId });
        console.log('✅ [BookingDetails] Invoice sent successfully', { result });

        // ✅ UPDATE LOADING TOAST mit Success
        console.log('🍞 [BookingDetails] Dispatching success toast', { toastId });
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId,
            message: `✅ ${result}`,
            type: 'success',
            duration: 3000
          }
        }));

        // Backend markiert Rechnung automatisch als versendet
        // Refresh globale BookingList und lokale Details (auch im Hintergrund)
        console.log('🔄 [BookingDetails] Refreshing bookings and details');
        await refreshBookings();
        await loadBookingDetails();

        // ✅ REMINDER BADGE UPDATE: Trigger Bell-Icon Count Refresh + Optimistic Update
        console.log('📤 [BookingDetails] Dispatching reminder-updated event with type: auto_invoice');
        window.dispatchEvent(new CustomEvent('reminder-updated', {
          detail: { reminderType: 'auto_invoice' }
        }));
        console.log('✅ [BookingDetails] reminder-updated event dispatched');
      } catch (error) {
        // ✅ UPDATE LOADING TOAST mit Error
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId,
            message: `❌ Fehler beim Senden: ${error}`,
            type: 'error',
            duration: 5000
          }
        }));
      }
    };

    // Start background process (no await!)
    sendInvoiceInBackground();
  };

  const handleGeneratePdf = async () => {
    if (!booking) return;

    // ✅ LOADING TOAST - bleibt offen bis fertig
    const toastId = `pdf-${bookingId}`;
    window.dispatchEvent(new CustomEvent('show-toast', {
      detail: {
        id: toastId,
        message: '📄 PDF-Rechnung wird erstellt...',
        type: 'loading'
      }
    }));

    // ✅ ASYNC BACKGROUND PROCESS
    const generatePdfInBackground = async () => {
      try {
        const pdfPath = await invoke<string>('generate_invoice_pdf_command', { bookingId });

        // ✅ UPDATE LOADING TOAST mit Success
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId,
            message: `✅ PDF-Rechnung erfolgreich erstellt`,
            type: 'success',
            duration: 3000
          }
        }));

        // Reload PDFs to show newly generated one (auch im Hintergrund)
        const pdfsData = await invoke<InvoicePdfInfo[]>('get_invoice_pdfs_for_booking_command', {
          bookingId,
        });
        setInvoicePdfs(pdfsData);
      } catch (error) {
        console.error('❌ ERROR generating PDF:', error);

        // ✅ UPDATE LOADING TOAST mit Error
        window.dispatchEvent(new CustomEvent('show-toast', {
          detail: {
            id: toastId,
            message: `❌ Fehler beim Erstellen der PDF: ${error}`,
            type: 'error',
            duration: 5000
          }
        }));
      }
    };

    // Start background process (no await!)
    generatePdfInBackground();
  };

  const handleOpenPdf = async (pdfPath: string) => {
    try {
      await invoke('open_pdf_file_command', { filePath: pdfPath });
    } catch (error) {
      setShowErrorDialog({ show: true, message: `Fehler beim Öffnen der PDF: ${error}` });
    }
  };

  const handleOpenInvoicesFolder = async () => {
    try {
      const result = await invoke<string>('open_invoices_folder_command');
      console.log(result);
    } catch (error) {
      setShowErrorDialog({ show: true, message: `Fehler beim Öffnen des Ordners: ${error}` });
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const formatDate = (timestamp: number): string => {
    const date = new Date(timestamp * 1000);
    return format(date, 'dd.MM.yyyy HH:mm', { locale: de });
  };

  const getStatusBadge = (status: string) => {
    const styles = {
      reserviert: 'bg-blue-100 text-blue-700 border-blue-200',
      bestaetigt: 'bg-emerald-100 text-emerald-700 border-emerald-200',
      eingecheckt: 'bg-purple-100 text-purple-700 border-purple-200',
      ausgecheckt: 'bg-slate-100 text-slate-700 border-slate-200',
      storniert: 'bg-red-100 text-red-700 border-red-200',
    };

    return (
      <span className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-semibold border ${styles[status as keyof typeof styles] || styles.reserviert}`}>
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </span>
    );
  };

  if (!isOpen) return null;

  // Show loading overlay while data is being fetched
  if (loading || !booking) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
        <div className="bg-white rounded-2xl shadow-2xl p-12">
          <div className="flex flex-col items-center justify-center">
            <div className="inline-block animate-spin rounded-full h-16 w-16 border-b-4 border-blue-600 mb-4"></div>
            <p className="text-slate-600 text-lg">Lade Buchungsdetails...</p>
          </div>
        </div>
      </div>
    );
  }

  // Safety check - should never happen due to validation in loadBookingDetails
  if (!booking.guest || !booking.room) {
    console.error('Missing guest or room data in booking');
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
        <div className="bg-white rounded-2xl shadow-2xl p-12">
          <div className="flex flex-col items-center justify-center">
            <p className="text-red-600 text-lg font-bold">❌ Fehler: Gast- oder Zimmerdaten fehlen!</p>
            <button onClick={onClose} className="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg">
              Schließen
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-2xl shadow-2xl w-full max-w-5xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-500 to-blue-600 px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="bg-white/20 p-2 rounded-lg">
              <FileText className="w-6 h-6 text-white" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">Buchungsdetails</h2>
              <p className="text-sm text-blue-100">{booking.reservierungsnummer}</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="text-white/80 hover:text-white transition-colors p-1 rounded-lg hover:bg-white/10"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto max-h-[calc(90vh-180px)]">
          <div className="space-y-6">
              {/* Status & Basic Info */}
              <div className="flex items-start justify-between">
                <div>
                  {getStatusBadge(booking.status)}
                  <div className="mt-3 flex items-center gap-4 text-sm text-slate-600">
                    <div className="flex items-center gap-2">
                      <Users className="w-4 h-4" />
                      {booking.anzahl_gaeste} Gäste
                    </div>
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4" />
                      {booking.anzahl_naechte} Nächte
                    </div>
                  </div>
                </div>
                <div className="flex gap-2">
                  {booking.status !== 'storniert' && (
                    <>
                      <button
                        onClick={onEdit}
                        className="flex items-center gap-2 px-4 py-2 bg-blue-100 hover:bg-blue-200 text-blue-700 rounded-lg font-semibold transition-colors"
                      >
                        <Edit2 className="w-4 h-4" />
                        Bearbeiten
                      </button>
                      <button
                        onClick={() => setShowCancelDialog(true)}
                        className="flex items-center gap-2 px-4 py-2 bg-red-100 hover:bg-red-200 text-red-700 rounded-lg font-semibold transition-colors"
                      >
                        <XCircle className="w-4 h-4" />
                        Stornieren
                      </button>
                    </>
                  )}
                </div>
              </div>

              {/* Stiftungsfall Warning */}
              {booking.ist_stiftungsfall && (
                <div className="border-2 border-amber-300 rounded-lg p-4 bg-gradient-to-br from-amber-50 to-orange-50">
                  <div className="flex items-start gap-3">
                    <div className="flex-shrink-0">
                      <AlertTriangle className="w-6 h-6 text-amber-600" />
                    </div>
                    <div className="flex-1">
                      <h3 className="font-bold text-amber-900 text-lg mb-2">
                        🟠 Stiftungsfall
                      </h3>
                      <p className="text-sm text-amber-700 leading-relaxed">
                        Diese Buchung ist als Stiftungsfall markiert. Es wird keine automatische Rechnungs-E-Mail versendet, aber PDF-Rechnungen können erstellt und heruntergeladen werden.
                      </p>
                    </div>
                  </div>
                </div>
              )}

              {/* Payment Recipient (External Invoice Recipient) */}
              {console.log('🎨 [BookingDetails RENDER] paymentRecipient:', paymentRecipient)}
              {paymentRecipient && (
                <div className="border-2 border-blue-300 rounded-lg p-5 bg-gradient-to-br from-blue-50 to-indigo-50">
                  <div className="flex items-start gap-3 mb-4">
                    <div className="flex-shrink-0 p-2 bg-blue-500 rounded-lg">
                      <FileText className="w-5 h-5 text-white" />
                    </div>
                    <div className="flex-1">
                      <h3 className="text-lg font-bold text-blue-900 mb-1">
                        🔵 Externer Rechnungsempfänger
                      </h3>
                      <p className="text-sm text-blue-700">
                        Die Rechnung für diese Buchung wird an den folgenden externen Empfänger adressiert:
                      </p>
                    </div>
                  </div>

                  <div className="bg-white/60 rounded-lg p-4 grid grid-cols-2 gap-4">
                    <div>
                      <p className="text-sm text-slate-600 mb-1">Name</p>
                      <p className="font-semibold text-slate-900">{paymentRecipient.name}</p>
                    </div>
                    {paymentRecipient.company && (
                      <div>
                        <p className="text-sm text-slate-600 mb-1">Firma</p>
                        <p className="font-semibold text-slate-900">{paymentRecipient.company}</p>
                      </div>
                    )}
                    {(paymentRecipient.street || paymentRecipient.plz || paymentRecipient.city) && (
                      <div className="col-span-2">
                        <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                          <MapPin className="w-3.5 h-3.5" />
                          Adresse
                        </p>
                        <p className="font-medium text-slate-900">
                          {paymentRecipient.street && <>{paymentRecipient.street}<br /></>}
                          {paymentRecipient.plz} {paymentRecipient.city}
                          {paymentRecipient.country !== 'Deutschland' && <><br />{paymentRecipient.country}</>}
                        </p>
                      </div>
                    )}
                    {paymentRecipient.contact_person && (
                      <div>
                        <p className="text-sm text-slate-600 mb-1">Ansprechpartner</p>
                        <p className="font-medium text-slate-900">{paymentRecipient.contact_person}</p>
                      </div>
                    )}
                    {paymentRecipient.notes && (
                      <div className="col-span-2">
                        <p className="text-sm text-slate-600 mb-1">Notizen</p>
                        <p className="text-sm text-slate-700">{paymentRecipient.notes}</p>
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* Guest Information */}
              <div className="border border-slate-200 rounded-lg p-5">
                <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-4">
                  <UserCheck className="w-5 h-5 text-blue-600" />
                  Gast
                </h3>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Name</p>
                    <p className="font-semibold text-slate-900">
                      {booking.guest.vorname} {booking.guest.nachname}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Mitgliedschaft</p>
                    <p className="font-semibold text-slate-900">
                      {booking.guest.dpolg_mitglied ? (
                        <span className="text-emerald-600">DPolG Stiftung Mitglied {booking.guest.mitgliedsnummer && `(${booking.guest.mitgliedsnummer})`}</span>
                      ) : (
                        <span className="text-slate-500">Kein Mitglied</span>
                      )}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                      <Mail className="w-3.5 h-3.5" />
                      E-Mail
                    </p>
                    <p className="font-medium text-slate-900">{booking.guest.email}</p>
                  </div>
                  {booking.guest.telefon && (
                    <div>
                      <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                        <Phone className="w-3.5 h-3.5" />
                        Telefon
                      </p>
                      <p className="font-medium text-slate-900">{booking.guest.telefon}</p>
                    </div>
                  )}
                  {(booking.guest.strasse || booking.guest.plz || booking.guest.ort) && (
                    <div className="col-span-2">
                      <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                        <MapPin className="w-3.5 h-3.5" />
                        Adresse
                      </p>
                      <p className="font-medium text-slate-900">
                        {booking.guest.strasse && <>{booking.guest.strasse}<br /></>}
                        {booking.guest.plz} {booking.guest.ort}
                      </p>
                    </div>
                  )}
                </div>
              </div>

              {/* Room Information */}
              <div className="border border-slate-200 rounded-lg p-5">
                <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-4">
                  <Home className="w-5 h-5 text-blue-600" />
                  Zimmer
                </h3>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Zimmername</p>
                    <p className="font-semibold text-slate-900">{booking.room.name}</p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Gebäude/Typ</p>
                    <p className="font-semibold text-slate-900">{booking.room.gebaeude_typ}</p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                      <MapPin className="w-3.5 h-3.5" />
                      Standort
                    </p>
                    <p className="font-medium text-slate-900">{booking.room.ort}</p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Kapazität</p>
                    <p className="font-medium text-slate-900">{booking.room.capacity} Personen</p>
                  </div>
                  {booking.room.schluesselcode && (
                    <div>
                      <p className="text-sm text-slate-600 mb-1">Schlüsselcode</p>
                      <p className="font-mono text-sm font-semibold text-slate-900 bg-slate-100 px-2 py-1 rounded inline-block">
                        {booking.room.schluesselcode}
                      </p>
                    </div>
                  )}
                </div>
              </div>

              {/* Dates */}
              <div className="border border-slate-200 rounded-lg p-5">
                <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-4">
                  <Calendar className="w-5 h-5 text-blue-600" />
                  Aufenthaltszeitraum
                </h3>
                <div className="grid grid-cols-3 gap-4">
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Check-in</p>
                    <p className="font-semibold text-slate-900">
                      {format(new Date(booking.checkin_date), 'dd. MMMM yyyy', { locale: de })}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Check-out</p>
                    <p className="font-semibold text-slate-900">
                      {format(new Date(booking.checkout_date), 'dd. MMMM yyyy', { locale: de })}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Anzahl Nächte</p>
                    <p className="font-semibold text-slate-900">{booking.anzahl_naechte} Nächte</p>
                  </div>
                </div>
              </div>

              {/* Accompanying Guests */}
              {accompanyingGuests.length > 0 && (
                <div className="border border-slate-200 rounded-lg p-5">
                  <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-4">
                    <UserCheck className="w-5 h-5 text-blue-600" />
                    Begleitpersonen ({accompanyingGuests.length})
                  </h3>
                  <div className="space-y-2">
                    {accompanyingGuests.map((guest) => (
                      <div
                        key={guest.id}
                        className="flex items-center justify-between bg-slate-50 px-4 py-3 rounded-lg"
                      >
                        <div>
                          <p className="font-semibold text-slate-900">
                            {guest.vorname} {guest.nachname}
                          </p>
                          {guest.geburtsdatum && (
                            <p className="text-sm text-slate-500">
                              Geburtsdatum: {format(new Date(guest.geburtsdatum), 'dd.MM.yyyy', { locale: de })}
                            </p>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Additional Services */}
              {services.length > 0 && (
                <div className="border border-slate-200 rounded-lg p-5">
                  <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-4">
                    <ShoppingBag className="w-5 h-5 text-blue-600" />
                    Zusätzliche Services ({services.length})
                  </h3>
                  <div className="space-y-2">
                    {services.map((service) => (
                      <div
                        key={service.id}
                        className="flex items-center justify-between bg-slate-50 px-4 py-3 rounded-lg"
                      >
                        <div className="flex items-center gap-3">
                          <p className="font-semibold text-slate-900">{service.service_name}</p>
                          {service.price_type === 'percent' && (
                            <span className="text-sm text-slate-500">
                              ({service.original_value}% {service.applies_to === 'overnight_price' ? 'vom Grundpreis' : 'vom Gesamtpreis'})
                            </span>
                          )}
                        </div>
                        <div className="flex items-center gap-2">
                          {getServicePriceIcon(service) === 'Percent' ? (
                            <Percent className="w-4 h-4 text-emerald-500" />
                          ) : (
                            <Euro className="w-4 h-4 text-emerald-500" />
                          )}
                          <p className="font-semibold text-emerald-600">
                            {formatCalculatedServicePrice(service, booking.grundpreis)}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Discounts */}
              {discounts.length > 0 && (
                <div className="border border-slate-200 rounded-lg p-5">
                  <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-4">
                    <Percent className="w-5 h-5 text-blue-600" />
                    Rabatte ({discounts.length})
                  </h3>
                  <div className="space-y-2">
                    {discounts.map((discount) => (
                      <div
                        key={discount.id}
                        className="flex items-center justify-between bg-slate-50 px-4 py-3 rounded-lg"
                      >
                        <p className="font-semibold text-slate-900">{discount.discount_name}</p>
                        <p className="font-semibold text-orange-600">
                          {discount.discount_type === 'percent'
                            ? `${discount.discount_value}%`
                            : `${discount.discount_value.toFixed(2)} €`}
                        </p>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Price Breakdown */}
              <div className="border-2 border-blue-200 rounded-lg p-5 bg-blue-50">
                <h3 className="flex items-center gap-2 text-lg font-bold text-blue-900 mb-4">
                  <DollarSign className="w-5 h-5" />
                  Preisaufschlüsselung
                </h3>
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <span className="text-slate-700">Grundpreis ({booking.anzahl_naechte} Nächte)</span>
                    <span className="font-semibold text-slate-900">{booking.grundpreis.toFixed(2)} €</span>
                  </div>
                  {booking.services_preis > 0 && (
                    <div className="flex justify-between items-center">
                      <span className="text-emerald-700">+ Zusätzliche Services</span>
                      <span className="font-semibold text-emerald-700">{booking.services_preis.toFixed(2)} €</span>
                    </div>
                  )}
                  {booking.rabatt_preis > 0 && (
                    <div className="flex justify-between items-center">
                      <span className="text-orange-700">- Rabatte</span>
                      <span className="font-semibold text-orange-700">{booking.rabatt_preis.toFixed(2)} €</span>
                    </div>
                  )}
                  {creditUsed > 0 && (
                    <div className="flex justify-between items-center">
                      <span className="text-emerald-700 flex items-center gap-1">
                        <Wallet className="w-4 h-4" />
                        💰 Gast-Guthaben
                      </span>
                      <span className="font-semibold text-emerald-700">-{creditUsed.toFixed(2)} €</span>
                    </div>
                  )}
                  <div className="border-t-2 border-blue-300 pt-3 flex justify-between items-center">
                    <span className="text-lg font-bold text-blue-900">Gesamtpreis</span>
                    <span className="text-2xl font-bold text-blue-900">{booking.gesamtpreis.toFixed(2)} €</span>
                  </div>
                </div>
              </div>

              {/* Payment Status */}
              <div className="border border-slate-200 rounded-lg p-5 bg-gradient-to-br from-slate-50 to-white">
                <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-3">
                  <Euro className="w-5 h-5 text-blue-600" />
                  Zahlungsstatus
                </h3>
                <div className="space-y-3">
                  <PaymentDropdown
                    isPaid={booking.bezahlt}
                    bookingId={bookingId}
                    onPaymentChange={async (isPaid, zahlungsmethode) => {
                      try {
                        console.log('💰 [BookingDetails] Payment status changed', { bookingId, isPaid, zahlungsmethode });
                        await updateBookingPayment(bookingId, isPaid, zahlungsmethode);
                        console.log('✅ [BookingDetails] Payment updated successfully');

                        // Reload booking details um aktuelle Daten zu zeigen
                        console.log('🔄 [BookingDetails] Reloading booking details');
                        await loadBookingDetails();

                        // ✅ Optimistic Update: Trigger reminder update wenn bezahlt
                        if (isPaid) {
                          console.log('📤 [BookingDetails] Dispatching reminder-updated event with type: auto_payment');
                          window.dispatchEvent(new CustomEvent('reminder-updated', {
                            detail: { reminderType: 'auto_payment' }
                          }));
                          console.log('✅ [BookingDetails] reminder-updated event dispatched');
                        } else {
                          console.log('ℹ️ [BookingDetails] Payment set to NOT paid - no event dispatched');
                        }
                      } catch (error) {
                        setShowErrorDialog({
                          show: true,
                          message: `Fehler beim Ändern des Zahlungsstatus: ${error}`
                        });
                      }
                    }}
                  />
                  {booking.bezahlt && (
                    <div className="grid grid-cols-2 gap-4 text-sm mt-3">
                      {booking.bezahlt_am && (
                        <div>
                          <span className="text-slate-600">Bezahlt am:</span>
                          <p className="font-semibold text-slate-900">
                            {format(new Date(booking.bezahlt_am), 'dd.MM.yyyy', { locale: de })}
                          </p>
                        </div>
                      )}
                      {booking.zahlungsmethode && (
                        <div>
                          <span className="text-slate-600">Zahlungsmethode:</span>
                          <p className="font-semibold text-slate-900">{booking.zahlungsmethode}</p>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              </div>

              {/* Invoice PDFs */}
              <div className="border border-slate-200 rounded-lg p-5 bg-gradient-to-br from-emerald-50 to-white">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800">
                    <FileText className="w-5 h-5 text-emerald-600" />
                    Rechnungen ({invoicePdfs.length})
                  </h3>
                  <div className="flex gap-2">
                    <button
                      onClick={handleGeneratePdf}
                      className="flex items-center gap-2 px-3 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <Download className="w-4 h-4" />
                      PDF erstellen
                    </button>
                    <button
                      onClick={handleOpenInvoicesFolder}
                      className="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <FolderOpen className="w-4 h-4" />
                      Ordner öffnen
                    </button>
                  </div>
                </div>

                {invoicePdfs.length === 0 ? (
                  <div className="text-center py-8 bg-slate-50 rounded-lg border-2 border-dashed border-slate-300">
                    <FileText className="w-12 h-12 text-slate-400 mx-auto mb-3" />
                    <p className="text-slate-600 font-medium">Noch keine Rechnung erstellt</p>
                    <p className="text-sm text-slate-500 mt-1">
                      Klicken Sie auf "PDF erstellen" um eine Rechnung zu generieren
                    </p>
                  </div>
                ) : (
                  <div className="space-y-2">
                    {invoicePdfs.map((pdf, index) => (
                      <div
                        key={index}
                        className="flex items-center justify-between bg-white border border-emerald-200 px-4 py-3 rounded-lg hover:shadow-md transition-shadow"
                      >
                        <div className="flex items-center gap-3">
                          <div className="bg-emerald-100 p-2 rounded-lg">
                            <FileText className="w-5 h-5 text-emerald-600" />
                          </div>
                          <div>
                            <p className="font-semibold text-slate-900">{pdf.filename}</p>
                            <div className="flex items-center gap-3 text-xs text-slate-500 mt-1">
                              <span>{formatFileSize(pdf.size_bytes)}</span>
                            </div>
                          </div>
                        </div>
                        <button
                          onClick={() => handleOpenPdf(pdf.path)}
                          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-colors text-sm"
                        >
                          <Eye className="w-4 h-4" />
                          Öffnen
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Remarks */}
              {booking.bemerkungen && (
                <div className="border border-slate-200 rounded-lg p-5">
                  <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-3">
                    <FileText className="w-5 h-5 text-blue-600" />
                    Bemerkungen
                  </h3>
                  <p className="text-slate-700 whitespace-pre-wrap">{booking.bemerkungen}</p>
                </div>
              )}

              {/* Reminders Section */}
              <BookingReminders bookingId={bookingId} />
          </div>
        </div>

        {/* Footer - Debug Version 2.0 */}
        <div className="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <button
              onClick={() => {
                console.log('🔵🔵🔵 BUTTON CLICKED - handleSendConfirmationEmail');
                handleSendConfirmationEmail();
              }}
              disabled={!booking || booking.ist_stiftungsfall}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-400 disabled:cursor-not-allowed text-white rounded-lg font-semibold transition-colors text-sm"
              title={booking?.ist_stiftungsfall ? 'Stiftungsfall: Kein automatischer Email-Versand' : ''}
            >
              <Send className="w-4 h-4" />
              Bestätigung
            </button>
            <button
              onClick={handleSendReminderEmail}
              disabled={!booking || booking.ist_stiftungsfall}
              className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-slate-400 disabled:cursor-not-allowed text-white rounded-lg font-semibold transition-colors text-sm"
              title={booking?.ist_stiftungsfall ? 'Stiftungsfall: Kein automatischer Email-Versand' : ''}
            >
              <Send className="w-4 h-4" />
              Reminder
            </button>
            <button
              onClick={handleSendInvoiceEmail}
              disabled={!booking || booking.ist_stiftungsfall}
              className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-400 disabled:cursor-not-allowed text-white rounded-lg font-semibold transition-colors text-sm"
              title={booking?.ist_stiftungsfall ? 'Stiftungsfall: Kein automatischer Email-Versand' : ''}
            >
              <Send className="w-4 h-4" />
              Rechnung
            </button>
          </div>
          <button
            onClick={onClose}
            className="px-6 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-lg font-semibold transition-colors"
          >
            Schließen
          </button>
        </div>
      </div>

      {/* Cancel Booking Confirmation Dialog */}
      {showCancelDialog && booking && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[60]">
          <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-red-500/10 rounded-full">
                <XCircle className="w-6 h-6 text-red-400" />
              </div>
              <div className="flex-1">
                <h3 className="text-xl font-bold text-white mb-2">Buchung stornieren?</h3>
                <p className="text-slate-300 text-sm">
                  Möchten Sie diese Buchung wirklich stornieren?
                </p>
              </div>
            </div>

            <div className="bg-slate-700/30 rounded-lg p-4 mb-6">
              <p className="text-sm text-white">
                <span className="font-semibold">Reservierungsnummer:</span> {booking.reservierungsnummer}
              </p>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => setShowCancelDialog(false)}
                className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
              >
                Abbrechen
              </button>
              <button
                onClick={handleCancelBooking}
                className="flex-1 px-4 py-3 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors flex items-center justify-center gap-2"
              >
                <XCircle className="w-4 h-4" />
                Stornieren
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Success Dialog */}
      {showSuccessDialog.show && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[60]">
          <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-emerald-500/10 rounded-full">
                <CheckCircle className="w-6 h-6 text-emerald-400" />
              </div>
              <div className="flex-1">
                <h3 className="text-xl font-bold text-white mb-2">Erfolgreich!</h3>
                <p className="text-slate-300 text-sm">{showSuccessDialog.message}</p>
              </div>
            </div>

            <button
              onClick={() => setShowSuccessDialog({ show: false, message: '' })}
              className="w-full px-4 py-3 bg-emerald-600 hover:bg-emerald-700 text-white font-semibold rounded-lg transition-colors"
            >
              OK
            </button>
          </div>
        </div>
      )}

      {/* Error Dialog */}
      {showErrorDialog.show && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[60]">
          <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-red-500/10 rounded-full">
                <AlertTriangle className="w-6 h-6 text-red-400" />
              </div>
              <div className="flex-1">
                <h3 className="text-xl font-bold text-white mb-2">Fehler</h3>
                <p className="text-slate-300 text-sm">{showErrorDialog.message}</p>
              </div>
            </div>

            <button
              onClick={() => setShowErrorDialog({ show: false, message: '' })}
              className="w-full px-4 py-3 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors"
            >
              OK
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
