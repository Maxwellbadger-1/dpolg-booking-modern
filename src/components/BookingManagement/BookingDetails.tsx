import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  X, Calendar, Users, MapPin, Mail, Phone, DollarSign, Tag,
  UserCheck, ShoppingBag, Percent, Edit2, XCircle, FileText,
  Clock, Home, Send, CheckCircle, AlertCircle, Euro, Download,
  FolderOpen, Eye
} from 'lucide-react';
import { format } from 'date-fns';
import { de } from 'date-fns/locale';

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
  mahnung_gesendet_am?: string | null;
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
  const [booking, setBooking] = useState<Booking | null>(null);
  const [accompanyingGuests, setAccompanyingGuests] = useState<AccompanyingGuest[]>([]);
  const [services, setServices] = useState<AdditionalService[]>([]);
  const [discounts, setDiscounts] = useState<Discount[]>([]);
  const [invoicePdfs, setInvoicePdfs] = useState<InvoicePdfInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [sendingEmail, setSendingEmail] = useState(false);
  const [markingPaid, setMarkingPaid] = useState(false);
  const [generatingPdf, setGeneratingPdf] = useState(false);
  const [showPaymentDialog, setShowPaymentDialog] = useState(false);
  const [zahlungsmethode, setZahlungsmethode] = useState('Überweisung');

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

      // Load invoice PDFs
      const pdfsData = await invoke<InvoicePdfInfo[]>('get_invoice_pdfs_for_booking_command', {
        bookingId,
      });
      setInvoicePdfs(pdfsData);
    } catch (error) {
      console.error('Fehler beim Laden der Buchungsdetails:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCancelBooking = async () => {
    if (!booking) return;

    if (!confirm(`Buchung ${booking.reservierungsnummer} wirklich stornieren?`)) {
      return;
    }

    try {
      await invoke('cancel_booking_command', { id: bookingId });
      loadBookingDetails(); // Reload to show updated status
    } catch (error) {
      console.error('Fehler beim Stornieren der Buchung:', error);
      alert('Fehler beim Stornieren der Buchung');
    }
  };

  const handleSendConfirmationEmail = async () => {
    if (!booking) return;

    setSendingEmail(true);
    try {
      const result = await invoke<string>('send_confirmation_email_command', { bookingId });
      alert(result);
    } catch (error) {
      alert(`Fehler beim Senden: ${error}`);
    } finally {
      setSendingEmail(false);
    }
  };

  const handleSendReminderEmail = async () => {
    if (!booking) return;

    setSendingEmail(true);
    try {
      const result = await invoke<string>('send_reminder_email_command', { bookingId });
      alert(result);
    } catch (error) {
      alert(`Fehler beim Senden: ${error}`);
    } finally {
      setSendingEmail(false);
    }
  };

  const handleSendInvoiceEmail = async () => {
    if (!booking) return;

    setSendingEmail(true);
    try {
      // Verwende generate_and_send_invoice_command statt send_invoice_email_command
      // damit PDF automatisch generiert und angehängt wird
      const result = await invoke<string>('generate_and_send_invoice_command', { bookingId });
      alert(result);
    } catch (error) {
      alert(`Fehler beim Senden: ${error}`);
    } finally {
      setSendingEmail(false);
    }
  };

  const handleMarkAsPaid = async () => {
    if (!booking) return;

    setMarkingPaid(true);
    try {
      const updatedBooking = await invoke<Booking>('mark_booking_as_paid_command', {
        bookingId,
        zahlungsmethode,
      });
      setBooking(updatedBooking);
      setShowPaymentDialog(false);
      alert('Buchung erfolgreich als bezahlt markiert!');
    } catch (error) {
      alert(`Fehler beim Markieren als bezahlt: ${error}`);
    } finally {
      setMarkingPaid(false);
    }
  };

  const handleGeneratePdf = async () => {
    if (!booking) return;

    setGeneratingPdf(true);
    try {
      const pdfPath = await invoke<string>('generate_invoice_pdf_command', { bookingId });
      alert(`PDF-Rechnung erfolgreich erstellt: ${pdfPath}`);

      // Reload PDFs to show newly generated one
      const pdfsData = await invoke<InvoicePdfInfo[]>('get_invoice_pdfs_for_booking_command', {
        bookingId,
      });
      setInvoicePdfs(pdfsData);
    } catch (error) {
      alert(`Fehler beim Erstellen der PDF: ${error}`);
    } finally {
      setGeneratingPdf(false);
    }
  };

  const handleOpenPdf = async (pdfPath: string) => {
    try {
      await invoke('open_pdf_file_command', { filePath: pdfPath });
    } catch (error) {
      alert(`Fehler beim Öffnen der PDF: ${error}`);
    }
  };

  const handleOpenInvoicesFolder = async () => {
    try {
      const result = await invoke<string>('open_invoices_folder_command');
      console.log(result);
    } catch (error) {
      alert(`Fehler beim Öffnen des Ordners: ${error}`);
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
                        onClick={handleCancelBooking}
                        className="flex items-center gap-2 px-4 py-2 bg-red-100 hover:bg-red-200 text-red-700 rounded-lg font-semibold transition-colors"
                      >
                        <XCircle className="w-4 h-4" />
                        Stornieren
                      </button>
                    </>
                  )}
                </div>
              </div>

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
                        <p className="font-semibold text-slate-900">{service.service_name}</p>
                        <p className="font-semibold text-emerald-600">
                          {service.service_price.toFixed(2)} €
                        </p>
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
                {booking.bezahlt ? (
                  <div className="space-y-3">
                    <div className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-100 border border-emerald-200">
                      <CheckCircle className="w-5 h-5 text-emerald-600" />
                      <span className="font-semibold text-emerald-800">Bezahlt</span>
                    </div>
                    <div className="grid grid-cols-2 gap-4 text-sm">
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
                  </div>
                ) : (
                  <div className="space-y-3">
                    <div className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-amber-100 border border-amber-200">
                      <AlertCircle className="w-5 h-5 text-amber-600" />
                      <span className="font-semibold text-amber-800">Noch nicht bezahlt</span>
                    </div>
                    <button
                      onClick={() => setShowPaymentDialog(true)}
                      className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <CheckCircle className="w-4 h-4" />
                      Als bezahlt markieren
                    </button>
                  </div>
                )}
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
                      disabled={generatingPdf}
                      className="flex items-center gap-2 px-3 py-1.5 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-400 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <Download className="w-4 h-4" />
                      {generatingPdf ? 'Erstellt...' : 'PDF erstellen'}
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
                              <span>•</span>
                              <span>Erstellt: {formatDate(pdf.created_at)}</span>
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
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <button
              onClick={handleSendConfirmationEmail}
              disabled={sendingEmail || !booking}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-400 text-white rounded-lg font-semibold transition-colors text-sm"
            >
              <Send className="w-4 h-4" />
              {sendingEmail ? 'Sendet...' : 'Bestätigung'}
            </button>
            <button
              onClick={handleSendReminderEmail}
              disabled={sendingEmail || !booking}
              className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-slate-400 text-white rounded-lg font-semibold transition-colors text-sm"
            >
              <Send className="w-4 h-4" />
              Reminder
            </button>
            <button
              onClick={handleSendInvoiceEmail}
              disabled={sendingEmail || !booking}
              className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-400 text-white rounded-lg font-semibold transition-colors text-sm"
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

      {/* Payment Dialog */}
      {showPaymentDialog && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-2xl p-6 max-w-md w-full mx-4">
            <h3 className="text-xl font-bold text-slate-800 mb-4">
              Buchung als bezahlt markieren
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Zahlungsmethode
                </label>
                <select
                  value={zahlungsmethode}
                  onChange={(e) => setZahlungsmethode(e.target.value)}
                  className="w-full px-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="Überweisung">Überweisung</option>
                  <option value="Barzahlung">Barzahlung</option>
                  <option value="EC-Karte">EC-Karte</option>
                  <option value="Kreditkarte">Kreditkarte</option>
                  <option value="PayPal">PayPal</option>
                  <option value="Sonstige">Sonstige</option>
                </select>
              </div>
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <p className="text-sm text-blue-800">
                  Die Buchung wird mit dem heutigen Datum als bezahlt markiert.
                </p>
              </div>
            </div>
            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowPaymentDialog(false)}
                className="px-4 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-lg font-semibold transition-colors"
              >
                Abbrechen
              </button>
              <button
                onClick={handleMarkAsPaid}
                disabled={markingPaid}
                className="px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-400 text-white rounded-lg font-semibold transition-colors flex items-center gap-2"
              >
                {markingPaid ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    Speichert...
                  </>
                ) : (
                  <>
                    <CheckCircle className="w-4 h-4" />
                    Als bezahlt markieren
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
