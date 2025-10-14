import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  X, Calendar, Users, MapPin, Mail, Phone, DollarSign, Tag,
  UserCheck, ShoppingBag, Percent, Edit2, XCircle, FileText,
  Clock, Home, Send, CheckCircle, AlertCircle, Euro, Download,
  FolderOpen, Eye, AlertTriangle, MessageSquare, Search, History,
  ChevronDown, ChevronUp, Info, Plus, Trash2, Bookmark, Loader2, Briefcase, Wallet
} from 'lucide-react';
import { format } from 'date-fns';
import { de } from 'date-fns/locale';
import PaymentDropdown from './PaymentDropdown';
import { useData } from '../../context/DataContext';
import BookingReminders from '../Reminders/BookingReminders';
import SearchableGuestPicker from './SearchableGuestPicker';
import SearchableRoomPicker from './SearchableRoomPicker';
import EmailSelectionDialog from './EmailSelectionDialog';
import CompanionSelector from './CompanionSelector';

interface BookingSidebarProps {
  bookingId: number | null;
  isOpen: boolean;
  onClose: () => void;
  mode?: 'view' | 'edit' | 'create';
  prefillData?: { roomId?: number; checkinDate?: string; checkoutDate?: string };
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
  beruf?: string;
  bundesland?: string;
  dienststelle?: string;
  created_at?: string;
}

interface Booking {
  id?: number;
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
  ist_stiftungsfall?: boolean;
  payment_recipient_id?: number | null;
  room?: Room;
  guest?: Guest;
}

interface AccompanyingGuest {
  id?: number;
  vorname: string;
  nachname: string;
  geburtsdatum?: string;
  companion_id?: number;
}

interface AdditionalService {
  id?: number;
  service_name: string;
  service_price: number;
  template_id?: number;
}

interface Discount {
  id?: number;
  discount_name: string;
  discount_type: 'percent' | 'fixed';
  discount_value: number;
  template_id?: number;
}

interface InvoicePdfInfo {
  filename: string;
  path: string;
  size_bytes: number;
  created_at: number;
  reservierungsnummer: string;
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

interface GuestCreditBalance {
  guestId: number;
  balance: number;
  transactionCount: number;
}

const STATUS_OPTIONS = [
  { value: 'bestaetigt', label: 'Bestätigt', color: 'bg-emerald-100 text-emerald-700' },
  { value: 'eingecheckt', label: 'Eingecheckt', color: 'bg-blue-100 text-blue-700' },
  { value: 'ausgecheckt', label: 'Ausgecheckt', color: 'bg-slate-100 text-slate-700' },
  { value: 'storniert', label: 'Storniert', color: 'bg-red-100 text-red-700' },
];

export default function BookingSidebar({ bookingId, isOpen, onClose, mode: initialMode = 'view', prefillData }: BookingSidebarProps) {
  const { createBooking, updateBooking, reloadBooking, updateBookingPayment, refreshBookings } = useData();

  // Mode State
  const [mode, setMode] = useState<'view' | 'edit' | 'create'>(initialMode);

  // Data State
  const [booking, setBooking] = useState<Booking | null>(null);
  const [guests, setGuests] = useState<Guest[]>([]);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [accompanyingGuests, setAccompanyingGuests] = useState<AccompanyingGuest[]>([]);
  const [services, setServices] = useState<AdditionalService[]>([]);
  const [discounts, setDiscounts] = useState<Discount[]>([]);
  const [invoicePdfs, setInvoicePdfs] = useState<InvoicePdfInfo[]>([]);
  const [paymentRecipients, setPaymentRecipients] = useState<PaymentRecipient[]>([]);
  const [currentPaymentRecipient, setCurrentPaymentRecipient] = useState<PaymentRecipient | null>(null);

  // Form State (for Edit/Create mode)
  const [formData, setFormData] = useState<Booking>({
    room_id: 0,
    guest_id: 0,
    reservierungsnummer: '',
    checkin_date: '',
    checkout_date: '',
    anzahl_gaeste: 1,
    anzahl_begleitpersonen: 0,
    status: 'bestaetigt',
    bemerkungen: '',
    grundpreis: 0,
    services_preis: 0,
    rabatt_preis: 0,
    gesamtpreis: 0,
    anzahl_naechte: 0,
    bezahlt: false,
    ist_stiftungsfall: false,
  });

  // Templates State
  const [serviceTemplates, setServiceTemplates] = useState<any[]>([]);
  const [discountTemplates, setDiscountTemplates] = useState<any[]>([]);
  const [newService, setNewService] = useState<AdditionalService>({
    service_name: '',
    service_price: 0,
  });
  const [newDiscount, setNewDiscount] = useState<Discount>({
    discount_name: '',
    discount_type: 'percent',
    discount_value: 0,
  });

  // UI State
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sendingEmail, setSendingEmail] = useState(false);
  const [generatingPdf, setGeneratingPdf] = useState(false);
  const [priceInfo, setPriceInfo] = useState<any>(null);
  const [availabilityStatus, setAvailabilityStatus] = useState<{
    checking: boolean;
    available: boolean | null;
  }>({ checking: false, available: null });

  // Dialog State
  const [showCancelDialog, setShowCancelDialog] = useState(false);
  const [showSuccessDialog, setShowSuccessDialog] = useState<{ show: boolean; message: string }>({ show: false, message: '' });
  const [showErrorDialog, setShowErrorDialog] = useState<{ show: boolean; message: string }>({ show: false, message: '' });
  const [showEmailDialog, setShowEmailDialog] = useState(false);
  const [createdBookingId, setCreatedBookingId] = useState<number | null>(null);
  const [createdGuestEmail, setCreatedGuestEmail] = useState<string>('');

  // Guest History State
  const [guestBookings, setGuestBookings] = useState<any[]>([]);
  const [showBookingHistory, setShowBookingHistory] = useState(false);
  const [historySearch, setHistorySearch] = useState('');
  const [historyStatusFilter, setHistoryStatusFilter] = useState('all');
  const [historyYearFilter, setHistoryYearFilter] = useState('all');
  const [historyLocationFilter, setHistoryLocationFilter] = useState('all');

  // Guest Credit States
  const [creditBalance, setCreditBalance] = useState<number>(0);
  const [creditToApply, setCreditToApply] = useState<number>(0);
  const [loadingCredit, setLoadingCredit] = useState(false);

  // Load data when sidebar opens
  useEffect(() => {
    if (isOpen) {
      setMode(initialMode);
      if (bookingId && initialMode === 'view') {
        loadBookingDetails();
      } else if (initialMode === 'create') {
        loadGuestsAndRooms();
        loadTemplates();
        // Apply prefill data if provided
        if (prefillData) {
          setFormData({
            room_id: prefillData.roomId || 0,
            guest_id: 0,
            reservierungsnummer: '',
            checkin_date: prefillData.checkinDate || '',
            checkout_date: prefillData.checkoutDate || '',
            anzahl_gaeste: 1,
            anzahl_begleitpersonen: 0,
            status: 'bestaetigt',
            bemerkungen: '',
            grundpreis: 0,
            services_preis: 0,
            rabatt_preis: 0,
            gesamtpreis: 0,
            anzahl_naechte: 0,
            bezahlt: false,
            ist_stiftungsfall: false,
          });
        }
      } else if (bookingId && initialMode === 'edit') {
        loadBookingForEdit();
      }
    }
  }, [isOpen, bookingId, initialMode]);

  // Price calculation effect
  useEffect(() => {
    if (mode === 'edit' || mode === 'create') {
      if (formData.room_id && formData.guest_id && formData.checkin_date && formData.checkout_date && guests.length > 0 && rooms.length > 0) {
        calculatePrice();
      } else {
        setPriceInfo(null);
      }
    }
  }, [mode, formData.room_id, formData.guest_id, formData.checkin_date, formData.checkout_date, services, discounts, guests, rooms]);

  // Auto-load Payment Recipient when booking.payment_recipient_id changes
  useEffect(() => {
    const loadPaymentRecipient = async () => {
      if (!booking || mode !== 'view') return;

      if (booking.payment_recipient_id) {
        try {
          const recipientData = await invoke<PaymentRecipient>('get_payment_recipient', {
            id: booking.payment_recipient_id,
          });
          setCurrentPaymentRecipient(recipientData);
        } catch (error) {
          console.error('[BookingSidebar] Error loading payment recipient:', error);
          setCurrentPaymentRecipient(null);
        }
      } else {
        setCurrentPaymentRecipient(null);
      }
    };

    loadPaymentRecipient();
  }, [booking?.payment_recipient_id, mode]);

  // Availability check effect
  useEffect(() => {
    if ((mode === 'edit' || mode === 'create') && formData.room_id && formData.checkin_date && formData.checkout_date) {
      checkAvailability();
    } else {
      setAvailabilityStatus({ checking: false, available: null });
    }
  }, [mode, formData.room_id, formData.checkin_date, formData.checkout_date]);

  // Load guest credit balance when guest is selected
  useEffect(() => {
    const loadCreditBalance = async () => {
      if (formData.guest_id > 0 && isOpen && (mode === 'edit' || mode === 'create')) {
        setLoadingCredit(true);
        try {
          const balance = await invoke<GuestCreditBalance>('get_guest_credit_balance', {
            guestId: formData.guest_id,
          });
          setCreditBalance(balance.balance);
          setCreditToApply(0); // Reset amount when guest changes
          console.log('✅ [Credit] Loaded balance:', balance.balance, '€ for guest', formData.guest_id);
        } catch (err) {
          console.error('❌ [Credit] Fehler beim Laden des Guthabens:', err);
          setCreditBalance(0);
          setCreditToApply(0);
        } finally {
          setLoadingCredit(false);
        }
      } else {
        // Reset when no guest selected
        setCreditBalance(0);
        setCreditToApply(0);
      }
    };

    loadCreditBalance();
  }, [formData.guest_id, isOpen, mode]);

  const loadBookingDetails = async () => {
    if (!bookingId) return;

    try {
      setLoading(true);

      const bookingData = await invoke<Booking>('get_booking_with_details_by_id_command', { id: bookingId });

      if (!bookingData || !bookingData.guest || !bookingData.room) {
        throw new Error('Buchungsdaten konnten nicht vollständig geladen werden');
      }

      setBooking(bookingData);

      // Load accompanying guests
      const guestsData = await invoke<AccompanyingGuest[]>('get_booking_accompanying_guests_command', { bookingId });
      setAccompanyingGuests(guestsData);

      // Load services
      const servicesData = await invoke<AdditionalService[]>('get_booking_services_command', { bookingId });
      setServices(servicesData);

      // Load discounts
      const discountsData = await invoke<Discount[]>('get_booking_discounts_command', { bookingId });
      setDiscounts(discountsData);

      // Load invoice PDFs
      const pdfsData = await invoke<InvoicePdfInfo[]>('get_invoice_pdfs_for_booking_command', { bookingId });
      setInvoicePdfs(pdfsData);
    } catch (error) {
      console.error('Fehler beim Laden der Buchungsdetails:', error);
      setShowErrorDialog({ show: true, message: 'Fehler beim Laden der Buchungsdetails' });
    } finally {
      setLoading(false);
    }
  };

  const loadBookingForEdit = async () => {
    if (!bookingId) return;

    try {
      setLoading(true);

      // Load booking data
      const bookingData = await invoke<Booking>('get_booking_with_details_by_id_command', { id: bookingId });

      if (!bookingData) {
        throw new Error('Buchungsdaten konnten nicht geladen werden');
      }

      setBooking(bookingData);

      // Set form data from booking
      setFormData({
        ...bookingData,
        bemerkungen: bookingData.bemerkungen || '',
      });

      // Load guests and rooms
      await loadGuestsAndRooms();
      await loadTemplates();

      // Load accompanying guests, services, discounts
      if (bookingData.id) {
        const [guestsData, servicesData, discountsData] = await Promise.all([
          invoke<AccompanyingGuest[]>('get_booking_accompanying_guests_command', { bookingId: bookingData.id }),
          invoke<AdditionalService[]>('get_booking_services_command', { bookingId: bookingData.id }),
          invoke<Discount[]>('get_booking_discounts_command', { bookingId: bookingData.id }),
        ]);

        setAccompanyingGuests(guestsData);
        setServices(servicesData);
        setDiscounts(discountsData);

        // Load payment recipient if booking has one
        if (bookingData.payment_recipient_id) {
          try {
            const recipientData = await invoke<PaymentRecipient>('get_payment_recipient', {
              id: bookingData.payment_recipient_id,
            });
            setCurrentPaymentRecipient(recipientData);
          } catch (error) {
            console.error('[BookingSidebar] Error loading payment recipient:', error);
            setCurrentPaymentRecipient(null);
          }
        } else {
          setCurrentPaymentRecipient(null);
        }
      }
    } catch (error) {
      console.error('Fehler beim Laden der Buchung:', error);
      setShowErrorDialog({ show: true, message: 'Fehler beim Laden der Buchung' });
    } finally {
      setLoading(false);
    }
  };

  const loadGuestsAndRooms = async () => {
    try {
      const [guestsData, roomsData, recipientsData] = await Promise.all([
        invoke<Guest[]>('get_all_guests_command'),
        invoke<Room[]>('get_all_rooms'),
        invoke<PaymentRecipient[]>('get_payment_recipients'),
      ]);
      setGuests(guestsData);
      setRooms(roomsData);
      setPaymentRecipients(recipientsData);
    } catch (err) {
      console.error('Fehler beim Laden:', err);
    }
  };

  const loadTemplates = async () => {
    try {
      const [servicesData, discountsData] = await Promise.all([
        invoke<any[]>('get_active_service_templates_command'),
        invoke<any[]>('get_active_discount_templates_command'),
      ]);
      setServiceTemplates(servicesData);
      setDiscountTemplates(discountsData);
    } catch (err) {
      console.error('Fehler beim Laden der Templates:', err);
    }
  };

  const calculatePrice = async () => {
    try {
      const guest = guests.find(g => g.id === formData.guest_id);
      const room = rooms.find(r => r.id === formData.room_id);

      if (!guest || !room || !formData.checkin_date || !formData.checkout_date) {
        return;
      }

      const priceResult = await invoke<{
        grundpreis: number;
        servicesPreis: number;
        rabattPreis: number;
        gesamtpreis: number;
        anzahlNaechte: number;
        istHauptsaison: boolean;
        servicesList?: any[];
        discountsList?: any[];
      }>('calculate_booking_price_command', {
        roomId: formData.room_id,
        checkin: formData.checkin_date,
        checkout: formData.checkout_date,
        isMember: guest.dpolg_mitglied,
        services: services.map(s => [s.service_name, s.service_price]),
        discounts: discounts.map(d => [d.discount_name, d.discount_type, d.discount_value]),
      });

      const pricePerNight = priceResult.grundpreis / (priceResult.anzahlNaechte || 1);

      setPriceInfo({
        nights: priceResult.anzahlNaechte || 0,
        pricePerNight,
        basePrice: priceResult.grundpreis || 0,
        servicesTotal: priceResult.servicesPreis || 0,
        discountsTotal: priceResult.rabattPreis || 0,
        totalPrice: priceResult.gesamtpreis || 0,
        memberPrice: guest.dpolg_mitglied,
        istHauptsaison: priceResult.istHauptsaison,
        servicesList: priceResult.servicesList || [],
        discountsList: priceResult.discountsList || [],
      });
    } catch (err) {
      console.error('Fehler bei Preisberechnung:', err);
      setPriceInfo(null);
    }
  };

  const checkAvailability = async () => {
    try {
      setAvailabilityStatus({ checking: true, available: null });

      const isAvailable = await invoke<boolean>('check_room_availability_command', {
        roomId: formData.room_id,
        checkin: formData.checkin_date,
        checkout: formData.checkout_date,
        excludeBookingId: booking?.id || null,
      });

      setAvailabilityStatus({ checking: false, available: isAvailable });
    } catch (err) {
      console.error('Fehler bei Verfügbarkeitsprüfung:', err);
      setAvailabilityStatus({ checking: false, available: null });
    }
  };

  const handleSwitchToEdit = () => {
    if (booking) {
      setMode('edit');
      loadBookingForEdit();
    }
  };

  const handleCancelEdit = () => {
    if (bookingId) {
      setMode('view');
      loadBookingDetails();
    } else {
      onClose();
    }
  };

  const handleClose = () => {
    // Reset all state
    setMode('view');
    setBooking(null);
    setFormData({
      room_id: 0,
      guest_id: 0,
      reservierungsnummer: '',
      checkin_date: '',
      checkout_date: '',
      anzahl_gaeste: 1,
      anzahl_begleitpersonen: 0,
      status: 'bestaetigt',
      bemerkungen: '',
      grundpreis: 0,
      services_preis: 0,
      rabatt_preis: 0,
      gesamtpreis: 0,
      anzahl_naechte: 0,
      bezahlt: false,
      ist_stiftungsfall: false,
    });
    setAccompanyingGuests([]);
    setServices([]);
    setDiscounts([]);
    setInvoicePdfs([]);
    setError(null);
    setPriceInfo(null);
    onClose();
  };

  const validateForm = (): boolean => {
    if (!formData.guest_id) {
      setError('Bitte wählen Sie einen Gast aus');
      return false;
    }
    if (!formData.room_id) {
      setError('Bitte wählen Sie ein Zimmer aus');
      return false;
    }
    if (!formData.checkin_date || !formData.checkout_date) {
      setError('Bitte geben Sie Check-in und Check-out Datum ein');
      return false;
    }
    if (new Date(formData.checkout_date) <= new Date(formData.checkin_date)) {
      setError('Check-out muss nach Check-in liegen');
      return false;
    }
    if (formData.anzahl_gaeste < 1) {
      setError('Mindestens 1 Gast erforderlich');
      return false;
    }

    const room = rooms.find(r => r.id === formData.room_id);
    if (room && formData.anzahl_gaeste > room.capacity) {
      setError(`Die Kapazität des Zimmers beträgt ${room.capacity} Personen`);
      return false;
    }

    if (availabilityStatus.checking) {
      setError('Bitte warten Sie, während die Verfügbarkeit geprüft wird...');
      return false;
    }

    if (availabilityStatus.available === false) {
      setError('Das Zimmer ist für den gewählten Zeitraum nicht verfügbar');
      return false;
    }

    if (availabilityStatus.available === null) {
      setError('Verfügbarkeit konnte nicht geprüft werden. Bitte warten...');
      return false;
    }

    return true;
  };

  const handleSubmit = async () => {
    if (!validateForm()) return;

    console.log('🚀 [SIDEBAR handleSubmit] START');
    console.log('  📦 formData.payment_recipient_id:', formData.payment_recipient_id, 'type:', typeof formData.payment_recipient_id);
    console.log('  📦 Complete formData:', JSON.stringify(formData, null, 2));

    setError(null);
    setLoading(true);

    try {
      if (booking?.id) {
        // Update existing booking
        const nights = priceInfo?.nights || 1;
        const basePrice = priceInfo?.basePrice || 0;
        const servicesTotal = priceInfo?.servicesTotal || 0;
        const discountsTotal = priceInfo?.discountsTotal || 0;
        const totalPrice = priceInfo?.totalPrice || basePrice;

        const updatePayload = {
          roomId: formData.room_id,
          guestId: formData.guest_id,
          checkinDate: formData.checkin_date,
          checkoutDate: formData.checkout_date,
          anzahlGaeste: formData.anzahl_gaeste,
          status: formData.status,
          gesamtpreis: totalPrice,
          bemerkungen: formData.bemerkungen || null,
          anzahlBegleitpersonen: accompanyingGuests.length,
          grundpreis: basePrice,
          servicesPreis: servicesTotal,
          rabattPreis: discountsTotal,
          anzahlNaechte: nights,
          istStiftungsfall: formData.ist_stiftungsfall || false,
          paymentRecipientId: formData.payment_recipient_id, // ✅ FIX: camelCase für Tauri auto-conversion
        };

        console.log('📤 [SIDEBAR updatePayload] Payload being sent to updateBooking:');
        console.log('  payment_recipient_id:', updatePayload.payment_recipient_id, 'type:', typeof updatePayload.payment_recipient_id);
        console.log('  Complete payload:', JSON.stringify(updatePayload, null, 2));

        await updateBooking(booking.id, updatePayload);

        // 💰 Apply guest credit if any
        if (creditToApply > 0) {
          console.log('💰 [Credit] Applying credit:', creditToApply, '€ to booking', booking.id);
          try {
            await invoke('use_guest_credit_for_booking', {
              guestId: formData.guest_id,
              bookingId: booking.id,
              amount: creditToApply,
              notes: `Guthaben verrechnet für Buchung #${booking.id}`,
            });
            console.log('✅ [Credit] Successfully applied credit');

            // Guthaben-State aktualisieren
            const newBalance = creditBalance - creditToApply;
            setCreditBalance(newBalance);
            setCreditToApply(0);
          } catch (err) {
            console.error('❌ [Credit] Fehler beim Verrechnen des Guthabens:', err);
            // Nicht werfen - Buchung wurde bereits gespeichert
            // User kann Guthaben später manuell verrechnen
          }
        }

        // Switch back to view mode
        setMode('view');
        await loadBookingDetails();
      } else {
        // Create new booking
        const reservierungsnummer = `RES-${Date.now()}`;
        const nights = priceInfo?.nights || 1;
        const basePrice = priceInfo?.basePrice || 0;
        const servicesTotal = priceInfo?.servicesTotal || 0;
        const discountsTotal = priceInfo?.discountsTotal || 0;
        const totalPrice = priceInfo?.totalPrice || basePrice;

        const bookingData = {
          roomId: formData.room_id,
          guestId: formData.guest_id,
          reservierungsnummer,
          checkinDate: formData.checkin_date,
          checkoutDate: formData.checkout_date,
          anzahlGaeste: formData.anzahl_gaeste,
          status: formData.status,
          gesamtpreis: totalPrice,
          bemerkungen: formData.bemerkungen || null,
          anzahlBegleitpersonen: accompanyingGuests.length,
          grundpreis: basePrice,
          servicesPreis: servicesTotal,
          rabattPreis: discountsTotal,
          anzahlNaechte: nights,
          istStiftungsfall: formData.ist_stiftungsfall || false,
          paymentRecipientId: formData.payment_recipient_id, // ✅ FIX: camelCase für Tauri auto-conversion
        };

        console.log('📤 [SIDEBAR createPayload] Payload being sent to createBooking:');
        console.log('  payment_recipient_id:', bookingData.payment_recipient_id, 'type:', typeof bookingData.payment_recipient_id);
        console.log('  Complete payload:', JSON.stringify(bookingData, null, 2));

        const result = await createBooking(bookingData) as any;

        // Save accompanying guests
        if (accompanyingGuests.length > 0 && result.id) {
          for (const guest of accompanyingGuests) {
            await invoke('add_accompanying_guest_command', {
              bookingId: result.id,
              vorname: guest.vorname,
              nachname: guest.nachname,
              geburtsdatum: guest.geburtsdatum || null,
              companionId: guest.companion_id || null,
            });
          }
        }

        // Save services
        if (services.length > 0 && result.id) {
          for (const service of services) {
            if (service.template_id) {
              await invoke('link_service_template_to_booking_command', {
                bookingId: result.id,
                serviceTemplateId: service.template_id,
              });
            } else {
              await invoke('add_service_command', {
                bookingId: result.id,
                serviceName: service.service_name,
                servicePrice: service.service_price,
              });
            }
          }
        }

        // Save discounts
        if (discounts.length > 0 && result.id) {
          for (const discount of discounts) {
            if (discount.template_id) {
              await invoke('link_discount_template_to_booking_command', {
                bookingId: result.id,
                discountTemplateId: discount.template_id,
              });
            } else {
              await invoke('add_discount_command', {
                bookingId: result.id,
                discountName: discount.discount_name,
                discountType: discount.discount_type,
                discountValue: discount.discount_value,
              });
            }
          }
        }

        // 💰 Apply guest credit if any
        if (creditToApply > 0 && result.id) {
          console.log('💰 [Credit] Applying credit:', creditToApply, '€ to booking', result.id);
          try {
            await invoke('use_guest_credit_for_booking', {
              guestId: formData.guest_id,
              bookingId: result.id,
              amount: creditToApply,
              notes: `Guthaben verrechnet für Buchung #${result.id}`,
            });
            console.log('✅ [Credit] Successfully applied credit');

            // Guthaben-State aktualisieren
            const newBalance = creditBalance - creditToApply;
            setCreditBalance(newBalance);
            setCreditToApply(0);
          } catch (err) {
            console.error('❌ [Credit] Fehler beim Verrechnen des Guthabens:', err);
            // Nicht werfen - Buchung wurde bereits gespeichert
            // User kann Guthaben später manuell verrechnen
          }
        }

        // Reload booking with services
        await reloadBooking(result.id);

        // Open email selection dialog
        if (result.id) {
          const guestEmail = guests.find(g => g.id === formData.guest_id)?.email || '';
          setCreatedBookingId(result.id);
          setCreatedGuestEmail(guestEmail);
          setShowEmailDialog(true);
          // Sidebar bleibt offen, Email-Dialog schließt beides
        }
      }
    } catch (err) {
      console.error('Fehler beim Speichern:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
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

  return (
    <>
      {/* Backdrop */}
      <div
        className={`fixed inset-0 bg-black/50 backdrop-blur-sm z-40 transition-opacity duration-300 ${isOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'}`}
        onClick={handleClose}
      />

      {/* Sidebar */}
      <div className={`fixed right-0 top-0 h-full w-full max-w-3xl bg-white shadow-2xl z-50 transform transition-transform duration-300 ease-out ${isOpen ? 'translate-x-0' : 'translate-x-full'}`}>
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-500 to-blue-600 px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="bg-white/20 p-2 rounded-lg">
              <FileText className="w-6 h-6 text-white" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">
                {mode === 'create' ? 'Neue Buchung' : mode === 'edit' ? 'Buchung bearbeiten' : 'Buchungsdetails'}
              </h2>
              {booking?.reservierungsnummer && (
                <p className="text-sm text-blue-100">{booking.reservierungsnummer}</p>
              )}
            </div>
          </div>
          <button
            onClick={handleClose}
            className="text-white/80 hover:text-white transition-colors p-1 rounded-lg hover:bg-white/10"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Content */}
        <div className="h-[calc(100vh-180px)] overflow-y-auto p-6">
          {loading && !booking ? (
            <div className="flex flex-col items-center justify-center h-full">
              <div className="inline-block animate-spin rounded-full h-16 w-16 border-b-4 border-blue-600 mb-4"></div>
              <p className="text-slate-600 text-lg">Lade Daten...</p>
            </div>
          ) : mode === 'view' && booking ? (
            // VIEW MODE CONTENT
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
                        Diese Buchung ist als Stiftungsfall markiert. Es wird keine automatische Rechnungs-E-Mail versendet.
                      </p>
                    </div>
                  </div>
                </div>
              )}

              {/* Payment Recipient (External Invoice Recipient) */}
              {currentPaymentRecipient && (
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
                      <p className="font-semibold text-slate-900">{currentPaymentRecipient.name}</p>
                    </div>
                    {currentPaymentRecipient.company && (
                      <div>
                        <p className="text-sm text-slate-600 mb-1">Firma</p>
                        <p className="font-semibold text-slate-900">{currentPaymentRecipient.company}</p>
                      </div>
                    )}
                    {(currentPaymentRecipient.street || currentPaymentRecipient.plz || currentPaymentRecipient.city) && (
                      <div className="col-span-2">
                        <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                          <MapPin className="w-3.5 h-3.5" />
                          Adresse
                        </p>
                        <p className="font-medium text-slate-900">
                          {currentPaymentRecipient.street && <>{currentPaymentRecipient.street}<br /></>}
                          {currentPaymentRecipient.plz} {currentPaymentRecipient.city}
                          {currentPaymentRecipient.country !== 'Deutschland' && <><br />{currentPaymentRecipient.country}</>}
                        </p>
                      </div>
                    )}
                    {currentPaymentRecipient.contact_person && (
                      <div>
                        <p className="text-sm text-slate-600 mb-1">Ansprechpartner</p>
                        <p className="font-medium text-slate-900">{currentPaymentRecipient.contact_person}</p>
                      </div>
                    )}
                    {currentPaymentRecipient.notes && (
                      <div className="col-span-2">
                        <p className="text-sm text-slate-600 mb-1">Notizen</p>
                        <p className="text-sm text-slate-700">{currentPaymentRecipient.notes}</p>
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
                      {booking.guest?.vorname} {booking.guest?.nachname}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Mitgliedschaft</p>
                    <p className="font-semibold text-slate-900">
                      {booking.guest?.dpolg_mitglied ? (
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
                    <p className="font-medium text-slate-900">{booking.guest?.email}</p>
                  </div>
                  {booking.guest?.telefon && (
                    <div>
                      <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                        <Phone className="w-3.5 h-3.5" />
                        Telefon
                      </p>
                      <p className="font-medium text-slate-900">{booking.guest.telefon}</p>
                    </div>
                  )}
                  {(booking.guest?.strasse || booking.guest?.plz || booking.guest?.ort) && (
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
                    <p className="font-semibold text-slate-900">{booking.room?.name}</p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Gebäude/Typ</p>
                    <p className="font-semibold text-slate-900">{booking.room?.gebaeude_typ}</p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                      <MapPin className="w-3.5 h-3.5" />
                      Standort
                    </p>
                    <p className="font-medium text-slate-900">{booking.room?.ort}</p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Kapazität</p>
                    <p className="font-medium text-slate-900">{booking.room?.capacity} Personen</p>
                  </div>
                  {booking.room?.schluesselcode && (
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
                <div className="space-y-3">
                  <PaymentDropdown
                    isPaid={booking.bezahlt}
                    bookingId={bookingId!}
                    onPaymentChange={async (isPaid, zahlungsmethode) => {
                      try {
                        await updateBookingPayment(bookingId!, isPaid, zahlungsmethode);
                        await loadBookingDetails();
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
                      onClick={async () => {
                        if (!booking.id) return;
                        setGeneratingPdf(true);
                        try {
                          const pdfPath = await invoke<string>('generate_invoice_pdf_command', { bookingId: booking.id });
                          setShowSuccessDialog({ show: true, message: `PDF-Rechnung erfolgreich erstellt: ${pdfPath}` });
                          const pdfsData = await invoke<InvoicePdfInfo[]>('get_invoice_pdfs_for_booking_command', { bookingId: booking.id });
                          setInvoicePdfs(pdfsData);
                        } catch (error) {
                          setShowErrorDialog({ show: true, message: `Fehler beim Erstellen der PDF: ${error}` });
                        } finally {
                          setGeneratingPdf(false);
                        }
                      }}
                      disabled={generatingPdf}
                      className="flex items-center gap-2 px-3 py-1.5 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-400 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <Download className="w-4 h-4" />
                      {generatingPdf ? 'Erstellt...' : 'PDF erstellen'}
                    </button>
                    <button
                      onClick={async () => {
                        try {
                          await invoke<string>('open_invoices_folder_command');
                        } catch (error) {
                          setShowErrorDialog({ show: true, message: `Fehler beim Öffnen des Ordners: ${error}` });
                        }
                      }}
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
                              <span>{(pdf.size_bytes / 1024).toFixed(1)} KB</span>
                            </div>
                          </div>
                        </div>
                        <button
                          onClick={async () => {
                            try {
                              await invoke('open_pdf_file_command', { filePath: pdf.path });
                            } catch (error) {
                              setShowErrorDialog({ show: true, message: `Fehler beim Öffnen der PDF: ${error}` });
                            }
                          }}
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
              {booking.id && <BookingReminders bookingId={booking.id} />}

              {/* Email Buttons Section */}
              <div className="border border-slate-200 rounded-lg p-5 bg-gradient-to-br from-blue-50 to-white">
                <h3 className="flex items-center gap-2 text-lg font-bold text-slate-800 mb-4">
                  <Send className="w-5 h-5 text-blue-600" />
                  E-Mails versenden
                </h3>
                <div className="flex flex-wrap gap-2">
                  <button
                    onClick={async () => {
                      if (!booking.id) return;
                      setSendingEmail(true);
                      try {
                        const result = await invoke<string>('send_confirmation_email_command', { bookingId: booking.id });
                        setShowSuccessDialog({ show: true, message: result });
                      } catch (error) {
                        setShowErrorDialog({ show: true, message: `Fehler beim Senden: ${error}` });
                      } finally {
                        setSendingEmail(false);
                      }
                    }}
                    disabled={sendingEmail || booking.ist_stiftungsfall}
                    className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-400 disabled:cursor-not-allowed text-white rounded-lg font-semibold transition-colors text-sm"
                    title={booking.ist_stiftungsfall ? 'Stiftungsfall: Kein automatischer Email-Versand' : ''}
                  >
                    <Send className="w-4 h-4" />
                    {sendingEmail ? 'Sendet...' : 'Bestätigung'}
                  </button>
                  <button
                    onClick={async () => {
                      if (!booking.id) return;
                      setSendingEmail(true);
                      try {
                        const result = await invoke<string>('send_reminder_email_command', { bookingId: booking.id });
                        setShowSuccessDialog({ show: true, message: result });
                      } catch (error) {
                        setShowErrorDialog({ show: true, message: `Fehler beim Senden: ${error}` });
                      } finally {
                        setSendingEmail(false);
                      }
                    }}
                    disabled={sendingEmail || booking.ist_stiftungsfall}
                    className="flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 disabled:bg-slate-400 disabled:cursor-not-allowed text-white rounded-lg font-semibold transition-colors text-sm"
                    title={booking.ist_stiftungsfall ? 'Stiftungsfall: Kein automatischer Email-Versand' : ''}
                  >
                    <Send className="w-4 h-4" />
                    Reminder
                  </button>
                  <button
                    onClick={async () => {
                      if (!booking.id) return;
                      setSendingEmail(true);
                      try {
                        const result = await invoke<string>('generate_and_send_invoice_command', { bookingId: booking.id });
                        setShowSuccessDialog({ show: true, message: result });
                        await refreshBookings();
                        await loadBookingDetails();
                      } catch (error) {
                        setShowErrorDialog({ show: true, message: `Fehler beim Senden: ${error}` });
                      } finally {
                        setSendingEmail(false);
                      }
                    }}
                    disabled={sendingEmail || booking.ist_stiftungsfall}
                    className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-400 disabled:cursor-not-allowed text-white rounded-lg font-semibold transition-colors text-sm"
                    title={booking.ist_stiftungsfall ? 'Stiftungsfall: Kein automatischer Email-Versand' : ''}
                  >
                    <Send className="w-4 h-4" />
                    Rechnung
                  </button>
                </div>
              </div>
            </div>
          ) : (mode === 'edit' || mode === 'create') ? (
            // EDIT/CREATE MODE CONTENT
            <div className="space-y-6">
              {error && (
                <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
                  <p className="text-sm text-red-700">{error}</p>
                </div>
              )}

              {/* Guest & Room Selection */}
              <div className="grid grid-cols-2 gap-4">
                <SearchableGuestPicker
                  guests={guests}
                  selectedGuestId={formData.guest_id}
                  onSelectGuest={(guestId) => setFormData({ ...formData, guest_id: guestId })}
                  onCreateNew={() => {
                    alert('Neuen Gast anlegen - Feature kommt bald!');
                  }}
                />

                <SearchableRoomPicker
                  rooms={rooms}
                  selectedRoomId={formData.room_id}
                  onSelectRoom={(roomId) => setFormData({ ...formData, room_id: roomId })}
                />
              </div>

              {/* Guest Info Panel */}
              {formData.guest_id > 0 && (() => {
                const selectedGuest = guests.find(g => g.id === formData.guest_id);
                if (!selectedGuest) return null;

                return (
                  <div className="space-y-3">
                    {/* Guest Details Card */}
                    <div className="bg-slate-50 border border-slate-200 rounded-lg p-4">
                      <div className="flex items-center gap-2 mb-3">
                        <Info className="w-5 h-5 text-blue-500" />
                        <h3 className="font-semibold text-slate-700">Gast-Informationen</h3>
                      </div>
                      <div className="grid grid-cols-2 gap-x-6 gap-y-2 text-sm">
                        <div>
                          <span className="text-slate-500">Name:</span>
                          <span className="ml-2 font-medium text-slate-700">
                            {selectedGuest.vorname} {selectedGuest.nachname}
                          </span>
                        </div>
                        <div>
                          <span className="text-slate-500">E-Mail:</span>
                          <span className="ml-2 font-medium text-slate-700">{selectedGuest.email}</span>
                        </div>
                        <div>
                          <span className="text-slate-500">Telefon:</span>
                          <span className="ml-2 font-medium text-slate-700">{selectedGuest.telefon}</span>
                        </div>
                        <div>
                          <span className="text-slate-500">DPolG-Mitglied:</span>
                          <span className={`ml-2 font-semibold ${selectedGuest.dpolg_mitglied ? 'text-emerald-600' : 'text-slate-600'}`}>
                            {selectedGuest.dpolg_mitglied ? '✓ Ja' : '✗ Nein'}
                          </span>
                        </div>
                      </div>
                    </div>

                    {/* Guest Notes Alert */}
                    {selectedGuest.notizen && selectedGuest.notizen.trim() !== '' && (
                      <div className="bg-gradient-to-r from-amber-50 to-orange-50 border-2 border-amber-400 rounded-lg p-4 shadow-lg">
                        <div className="flex items-start gap-3">
                          <div className="flex-shrink-0">
                            <AlertTriangle className="w-6 h-6 text-amber-600" />
                          </div>
                          <div className="flex-1">
                            <h3 className="font-bold text-amber-900 text-lg mb-2 flex items-center gap-2">
                              ⚠️ WICHTIGE GAST-NOTIZEN
                            </h3>
                            <div className="bg-white border border-amber-200 rounded-lg p-3">
                              <p className="text-slate-700 whitespace-pre-wrap leading-relaxed">
                                {selectedGuest.notizen}
                              </p>
                            </div>
                          </div>
                        </div>
                      </div>
                    )}

                    {/* Booking History */}
                    <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                      <button
                        type="button"
                        onClick={() => {
                          if (!showBookingHistory) {
                            // Load guest bookings
                            invoke<any[]>('get_all_bookings').then(allBookings => {
                              const filtered = allBookings
                                .filter(b => b.guest_id === selectedGuest.id && b.id !== booking?.id)
                                .sort((a, b) => new Date(b.checkin_date).getTime() - new Date(a.checkin_date).getTime());
                              setGuestBookings(filtered);
                            });
                          }
                          setShowBookingHistory(!showBookingHistory);
                        }}
                        className="w-full flex items-center justify-between text-left hover:bg-blue-100 rounded-lg p-2 transition-colors"
                      >
                        <div className="flex items-center gap-2">
                          <History className="w-5 h-5 text-blue-600" />
                          <h3 className="font-semibold text-blue-900">
                            Buchungshistorie {guestBookings.length > 0 && `(${guestBookings.length})`}
                          </h3>
                        </div>
                        {showBookingHistory ? (
                          <ChevronUp className="w-5 h-5 text-blue-600" />
                        ) : (
                          <ChevronDown className="w-5 h-5 text-blue-600" />
                        )}
                      </button>

                      {showBookingHistory && (
                        <div className="mt-4 space-y-3">
                          {/* History Filters */}
                          <div className="p-3 bg-blue-50/50 rounded-lg border border-blue-200">
                            <div className="grid grid-cols-4 gap-3">
                              <div className="col-span-4 sm:col-span-1 relative">
                                <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-slate-400" />
                                <input
                                  type="text"
                                  placeholder="Suche..."
                                  value={historySearch}
                                  onChange={(e) => setHistorySearch(e.target.value)}
                                  className="w-full pl-12 pr-5 py-3.5 bg-white border border-slate-300 rounded-xl text-base text-slate-700 placeholder-slate-400 shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 focus:shadow-md transition-all"
                                />
                              </div>

                              <select
                                value={historyStatusFilter}
                                onChange={(e) => setHistoryStatusFilter(e.target.value)}
                                className="w-full px-5 py-3.5 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                style={{
                                  backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='%23475569' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E")`,
                                  backgroundRepeat: 'no-repeat',
                                  backgroundPosition: 'right 0.75rem center',
                                  backgroundSize: '1.5rem',
                                  paddingRight: '3rem'
                                }}
                              >
                                <option value="all">Alle Status</option>
                                <option value="bestaetigt">Bestätigt</option>
                                <option value="eingecheckt">Eingecheckt</option>
                                <option value="ausgecheckt">Ausgecheckt</option>
                                <option value="storniert">Storniert</option>
                              </select>

                              <select
                                value={historyYearFilter}
                                onChange={(e) => setHistoryYearFilter(e.target.value)}
                                className="w-full px-5 py-3.5 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                style={{
                                  backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='%23475569' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E")`,
                                  backgroundRepeat: 'no-repeat',
                                  backgroundPosition: 'right 0.75rem center',
                                  backgroundSize: '1.5rem',
                                  paddingRight: '3rem'
                                }}
                              >
                                <option value="all">Alle Jahre</option>
                                {Array.from(new Set(guestBookings.map(b => new Date(b.checkin_date).getFullYear())))
                                  .sort((a, b) => b - a)
                                  .map(year => (
                                    <option key={year} value={year}>{year}</option>
                                  ))}
                              </select>

                              <select
                                value={historyLocationFilter}
                                onChange={(e) => setHistoryLocationFilter(e.target.value)}
                                className="w-full px-5 py-3.5 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                                style={{
                                  backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='%23475569' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E")`,
                                  backgroundRepeat: 'no-repeat',
                                  backgroundPosition: 'right 0.75rem center',
                                  backgroundSize: '1.5rem',
                                  paddingRight: '3rem'
                                }}
                              >
                                <option value="all">Alle Orte</option>
                                <option value="Fall">Fall</option>
                                <option value="Lenggries">Lenggries</option>
                                <option value="Brauneckblick">Brauneckblick</option>
                              </select>
                            </div>
                          </div>

                          {/* History List */}
                          {guestBookings.length === 0 ? (
                            <div className="text-center py-6 text-slate-500">
                              Keine vorherigen Buchungen gefunden
                            </div>
                          ) : (
                            <div className="space-y-2 max-h-60 overflow-y-auto">
                              {guestBookings
                                .filter(b => {
                                  const matchesSearch =
                                    historySearch === '' ||
                                    b.reservierungsnummer.toLowerCase().includes(historySearch.toLowerCase());
                                  const matchesStatus = historyStatusFilter === 'all' || b.status === historyStatusFilter;
                                  const matchesYear = historyYearFilter === 'all' || new Date(b.checkin_date).getFullYear().toString() === historyYearFilter;
                                  const matchesLocation = historyLocationFilter === 'all' || b.room?.ort === historyLocationFilter;
                                  return matchesSearch && matchesStatus && matchesYear && matchesLocation;
                                })
                                .map(b => (
                                  <div key={b.id} className="bg-white border border-slate-200 rounded-lg p-3 hover:shadow-md transition-shadow">
                                    <div className="flex items-start justify-between gap-3">
                                      <div className="flex-1">
                                        <div className="flex items-center gap-2 mb-1">
                                          <span className="font-semibold text-slate-700">{b.reservierungsnummer}</span>
                                          <span className="px-2 py-0.5 rounded text-xs font-medium bg-emerald-100 text-emerald-700">
                                            {b.status}
                                          </span>
                                        </div>
                                        <div className="text-sm text-slate-600 space-y-0.5">
                                          <div>📅 {new Date(b.checkin_date).toLocaleDateString('de-DE')} - {new Date(b.checkout_date).toLocaleDateString('de-DE')}</div>
                                          <div>🏠 {b.room?.name} ({b.room?.ort})</div>
                                          <div className="font-semibold text-slate-700">💰 {b.gesamtpreis.toFixed(2)} €</div>
                                        </div>
                                      </div>
                                    </div>
                                  </div>
                                ))}
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                );
              })()}

              {/* Check-in & Check-out */}
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-semibold text-slate-700 mb-2">
                    Check-in Datum *
                  </label>
                  <input
                    type="date"
                    required
                    value={formData.checkin_date}
                    onChange={(e) => setFormData({ ...formData, checkin_date: e.target.value })}
                    className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-semibold text-slate-700 mb-2">
                    Check-out Datum *
                  </label>
                  <input
                    type="date"
                    required
                    value={formData.checkout_date}
                    onChange={(e) => setFormData({ ...formData, checkout_date: e.target.value })}
                    className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>

              {/* Availability Status */}
              {formData.room_id && formData.checkin_date && formData.checkout_date && (
                <div className={`border rounded-lg p-4 ${
                  availabilityStatus.checking ? 'bg-slate-50 border-slate-200' :
                  availabilityStatus.available === true ? 'bg-emerald-50 border-emerald-200' :
                  availabilityStatus.available === false ? 'bg-red-50 border-red-200' :
                  'bg-slate-50 border-slate-200'
                }`}>
                  <div className="flex items-center gap-2">
                    {availabilityStatus.checking ? (
                      <>
                        <Loader2 className="w-4 h-4 text-slate-600 animate-spin" />
                        <span className="text-sm font-semibold text-slate-700">Verfügbarkeit wird geprüft...</span>
                      </>
                    ) : availabilityStatus.available === true ? (
                      <>
                        <CheckCircle className="w-4 h-4 text-emerald-600" />
                        <span className="text-sm font-semibold text-emerald-700">Zimmer verfügbar</span>
                      </>
                    ) : availabilityStatus.available === false ? (
                      <>
                        <AlertCircle className="w-4 h-4 text-red-600" />
                        <span className="text-sm font-semibold text-red-700">Zimmer nicht verfügbar</span>
                        <span className="text-xs text-red-600 ml-auto">Es gibt bereits eine Buchung für diesen Zeitraum</span>
                      </>
                    ) : null}
                  </div>
                </div>
              )}

              {/* Anzahl Gäste & Status */}
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                    <Users className="w-4 h-4" />
                    Anzahl Gäste *
                  </label>
                  <input
                    type="number"
                    required
                    min="1"
                    max={rooms.find(r => r.id === formData.room_id)?.capacity || 10}
                    value={formData.anzahl_gaeste}
                    onChange={(e) => setFormData({ ...formData, anzahl_gaeste: parseInt(e.target.value) || 1 })}
                    className="w-full px-4 py-2.5 bg-white border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-slate-700 font-medium hover:border-slate-400 transition-colors"
                  />
                  {formData.room_id && rooms.find(r => r.id === formData.room_id) && (
                    <p className="text-xs text-slate-500 mt-1">
                      Max. Kapazität: {rooms.find(r => r.id === formData.room_id)?.capacity} Personen
                    </p>
                  )}
                </div>

                <div>
                  <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                    <Tag className="w-4 h-4" />
                    Status *
                  </label>
                  <select
                    required
                    value={formData.status}
                    onChange={(e) => setFormData({ ...formData, status: e.target.value })}
                    className="w-full px-4 py-2.5 bg-white border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 appearance-none cursor-pointer text-slate-700 font-medium hover:border-slate-400 transition-colors"
                    style={{
                      backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%23475569'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'%3E%3C/path%3E%3C/svg%3E")`,
                      backgroundRepeat: 'no-repeat',
                      backgroundPosition: 'right 0.75rem center',
                      backgroundSize: '1.25rem',
                      paddingRight: '2.5rem'
                    }}
                  >
                    {STATUS_OPTIONS.map(status => (
                      <option key={status.value} value={status.value}>
                        {status.label}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              {/* Price Info */}
              {priceInfo && (
                <div className="border border-blue-200 rounded-lg p-4 bg-blue-50">
                  <h3 className="flex items-center gap-2 text-sm font-semibold text-blue-900 mb-3">
                    <DollarSign className="w-4 h-4" />
                    Preisberechnung
                  </h3>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-blue-700">Anzahl Nächte:</span>
                      <span className="font-semibold text-blue-900">{priceInfo.nights ?? 0}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-blue-700">
                        Preis pro Nacht {priceInfo.istHauptsaison ? '(Hauptsaison)' : '(Nebensaison)'}:
                      </span>
                      <span className="font-semibold text-blue-900">{(priceInfo.pricePerNight ?? 0).toFixed(2)} €</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-blue-700">Grundpreis:</span>
                      <span className="font-semibold text-blue-900">{(priceInfo.basePrice ?? 0).toFixed(2)} €</span>
                    </div>

                    {priceInfo.servicesList && priceInfo.servicesList.length > 0 && (
                      <>
                        {priceInfo.servicesList.map((service: any, index: number) => (
                          <div key={index} className="flex justify-between">
                            <span className="text-emerald-700">+ {service.name}:</span>
                            <span className="font-semibold text-emerald-700">{service.price.toFixed(2)} €</span>
                          </div>
                        ))}
                      </>
                    )}

                    {priceInfo.discountsList && priceInfo.discountsList.length > 0 && (
                      <>
                        {priceInfo.discountsList.map((discount: any, index: number) => (
                          <div key={index} className="flex justify-between">
                            <span className="text-orange-700">
                              - {discount.name} {discount.type === 'percent' && `(${discount.value}%)`}:
                            </span>
                            <span className="font-semibold text-orange-700">{discount.amount.toFixed(2)} €</span>
                          </div>
                        ))}
                      </>
                    )}

                    <div className="border-t border-blue-300 pt-2 mt-2 flex justify-between">
                      <span className="font-bold text-blue-900">Gesamtpreis:</span>
                      <span className="font-bold text-blue-900 text-lg">{(priceInfo.totalPrice ?? 0).toFixed(2)} €</span>
                    </div>
                  </div>
                </div>
              )}

              {/* Guest Credit / Guthaben Section */}
              {formData.guest_id > 0 && (loadingCredit || creditBalance > 0) && (
                <div className="border border-emerald-200 rounded-lg p-3 bg-emerald-50">
                  <h3 className="flex items-center gap-2 text-sm font-semibold text-emerald-900 mb-2">
                    <Wallet className="w-4 h-4" />
                    💰 Gast-Guthaben
                  </h3>

                  {loadingCredit ? (
                    <div className="flex items-center gap-2 text-sm text-emerald-700">
                      <Loader2 className="w-4 h-4 animate-spin" />
                      <span>Lade Guthaben...</span>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      <div className="flex items-center justify-between text-sm">
                        <span className="text-emerald-700">Verfügbares Guthaben:</span>
                        <span className="font-bold text-emerald-900 text-base">{creditBalance.toFixed(2)} €</span>
                      </div>

                      {creditBalance > 0 && priceInfo && (
                        <>
                          <div className="border-t border-emerald-300 pt-2">
                            <label className="block text-xs font-medium text-emerald-700 mb-1.5">
                              Guthaben verrechnen:
                            </label>
                            <div className="flex items-center gap-2">
                              <input
                                type="number"
                                min="0"
                                max={Math.min(creditBalance, priceInfo.totalPrice)}
                                step="0.01"
                                value={creditToApply || ''}
                                onChange={(e) => {
                                  const value = parseFloat(e.target.value) || 0;
                                  const maxAmount = Math.min(creditBalance, priceInfo.totalPrice);
                                  setCreditToApply(Math.min(value, maxAmount));
                                }}
                                placeholder="Betrag"
                                className="flex-1 px-3 py-1.5 bg-white border border-emerald-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500 text-sm text-slate-700"
                              />
                              <button
                                type="button"
                                onClick={() => setCreditToApply(Math.min(creditBalance, priceInfo.totalPrice))}
                                className="px-2 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold rounded-lg transition-colors"
                              >
                                Max
                              </button>
                            </div>
                            <p className="text-xs text-emerald-600 mt-1">
                              Max. {Math.min(creditBalance, priceInfo.totalPrice).toFixed(2)} € (Guthaben oder Rechnungsbetrag)
                            </p>
                          </div>

                          {creditToApply > 0 && (
                            <div className="border-t border-emerald-300 pt-2 space-y-1.5">
                              <div className="flex justify-between text-xs">
                                <span className="text-emerald-700">Ursprünglicher Preis:</span>
                                <span className="font-semibold text-emerald-900">{priceInfo.totalPrice.toFixed(2)} €</span>
                              </div>
                              <div className="flex justify-between text-xs">
                                <span className="text-emerald-700">- Verrechnetes Guthaben:</span>
                                <span className="font-semibold text-emerald-700">-{creditToApply.toFixed(2)} €</span>
                              </div>
                              <div className="flex justify-between border-t border-emerald-300 pt-1.5">
                                <span className="font-bold text-emerald-900 text-sm">Zu zahlender Betrag:</span>
                                <span className="font-bold text-emerald-900 text-base">{(priceInfo.totalPrice - creditToApply).toFixed(2)} €</span>
                              </div>
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  )}
                </div>
              )}

              {/* Bemerkungen */}
              <div>
                <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                  <MessageSquare className="w-4 h-4" />
                  Bemerkungen
                </label>
                <textarea
                  value={formData.bemerkungen}
                  onChange={(e) => setFormData({ ...formData, bemerkungen: e.target.value })}
                  className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  rows={3}
                  placeholder="Besondere Wünsche, Anmerkungen..."
                />
              </div>

              {/* Payment Recipient Dropdown (conditional) */}
              {paymentRecipients.length > 0 && (
                <div>
                  <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                    <Briefcase className="w-4 h-4" />
                    Rechnungsempfänger (optional)
                  </label>
                  <select
                    value={formData.payment_recipient_id || ''}
                    onChange={(e) => {
                      console.log('🎯 [SIDEBAR DROPDOWN] onChange fired!');
                      console.log('  📦 e.target.value (raw):', e.target.value, 'type:', typeof e.target.value);
                      const newValue = e.target.value ? parseInt(e.target.value) : null;
                      console.log('  🔢 Parsed newValue:', newValue, 'type:', typeof newValue);
                      console.log('  📝 Current formData.payment_recipient_id:', formData.payment_recipient_id);
                      setFormData({
                        ...formData,
                        payment_recipient_id: newValue
                      });
                      console.log('  ✅ setFormData called with payment_recipient_id:', newValue);
                    }}
                    className="w-full px-5 py-3.5 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                    style={{
                      backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='%23475569' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E")`,
                      backgroundRepeat: 'no-repeat',
                      backgroundPosition: 'right 0.75rem center',
                      backgroundSize: '1.5rem',
                      paddingRight: '3rem'
                    }}
                  >
                    <option value="">Kein externer Empfänger (Standard: Gast)</option>
                    {paymentRecipients.map((recipient) => (
                      <option key={recipient.id} value={recipient.id}>
                        {recipient.name}{recipient.company ? ` - ${recipient.company}` : ''}
                      </option>
                    ))}
                  </select>
                  {formData.payment_recipient_id && (() => {
                    const selected = paymentRecipients.find(r => r.id === formData.payment_recipient_id);
                    return selected ? (
                      <div className="mt-2 p-3 bg-blue-50 border border-blue-200 rounded-lg text-xs text-blue-800">
                        <strong>Hinweis:</strong> Die Rechnung wird an <strong>{selected.name}</strong> adressiert.
                      </div>
                    ) : null;
                  })()}
                </div>
              )}

              {/* Stiftungsfall Checkbox */}
              <div className="border-2 border-red-300 rounded-lg p-4 bg-gradient-to-br from-red-50 to-rose-50">
                <label className="flex items-start gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.ist_stiftungsfall || false}
                    onChange={(e) => setFormData({ ...formData, ist_stiftungsfall: e.target.checked })}
                    className="mt-0.5 w-5 h-5 accent-red-600 bg-white border-red-300 rounded focus:ring-2 focus:ring-red-500"
                  />
                  <div className="flex-1">
                    <div className="flex items-center gap-2 text-red-900 font-semibold mb-1">
                      <AlertTriangle className="w-4 h-4" />
                      Stiftungsfall
                    </div>
                    <p className="text-sm text-red-700">
                      Diese Buchung ist ein Stiftungsfall. Der Email-Dialog wird geöffnet, aber die Rechnung-Option ist deaktiviert.
                    </p>
                  </div>
                </label>
              </div>

              {/* Companion Selector */}
              {formData.guest_id && (() => {
                const selectedGuest = guests.find(g => g.id === formData.guest_id);
                const selectedRoom = rooms.find(r => r.id === formData.room_id);
                if (!selectedGuest) return null;

                return (
                  <div className="border border-slate-200 rounded-lg p-4">
                    <CompanionSelector
                      guestId={selectedGuest.id}
                      bookingId={booking?.id}
                      roomCapacity={selectedRoom?.capacity}
                      onCompanionsChange={(companions) => {
                        setAccompanyingGuests(companions);
                        setFormData({ ...formData, anzahl_gaeste: 1 + companions.length });
                      }}
                    />
                  </div>
                );
              })()}

              {/* Additional Services */}
              <div className="border border-slate-200 rounded-lg p-4">
                <h3 className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-4">
                  <ShoppingBag className="w-4 h-4" />
                  Zusätzliche Services
                </h3>

                {/* List of Services */}
                {services.length > 0 && (
                  <div className="mb-4 space-y-2">
                    {services.map((service, index) => (
                      <div
                        key={service.id || index}
                        className="flex items-center justify-between bg-slate-50 px-4 py-2 rounded-lg border border-slate-200"
                      >
                        <div className="flex items-center gap-3">
                          <span className="text-sm font-semibold text-slate-900">
                            {service.service_name}
                          </span>
                          <span className="text-sm font-semibold text-emerald-600">
                            {service.service_price.toFixed(2)} €
                          </span>
                        </div>
                        <div className="flex items-center gap-1">
                          <button
                            type="button"
                            onClick={async () => {
                              try {
                                await invoke('create_service_template_command', {
                                  name: service.service_name,
                                  description: null,
                                  price: service.service_price,
                                });
                                await loadTemplates();
                                setShowSuccessDialog({ show: true, message: 'Service als Vorlage gespeichert!' });
                              } catch (err) {
                                setShowErrorDialog({ show: true, message: `Fehler: ${err}` });
                              }
                            }}
                            className="text-emerald-600 hover:bg-emerald-50 p-1 rounded transition-colors"
                            title="Als Vorlage speichern"
                          >
                            <Bookmark className="w-4 h-4" />
                          </button>
                          <button
                            type="button"
                            onClick={async () => {
                              if (service.id && booking?.id) {
                                try {
                                  // 1. Service vom Backend löschen
                                  await invoke('delete_service_command', { serviceId: service.id });

                                  // 2. Buchung NEU laden (inkl. aktualisierte Services mit Emojis)
                                  await reloadBooking(booking.id);

                                  // 3. Lokalen Sidebar-State aktualisieren
                                  const updatedServices = await invoke<AdditionalService[]>('get_booking_services_command', { bookingId: booking.id });
                                  setServices(updatedServices);

                                  // 4. AUTO-SYNC zu Turso (Mobile App) - Service-Emojis aktualisieren
                                  if (booking.checkout_date) {
                                    console.log('🔄 [BookingSidebar] Service gelöscht - Auto-Sync zu Turso für', booking.checkout_date);
                                    await invoke('sync_affected_dates', {
                                      oldCheckout: null,
                                      newCheckout: booking.checkout_date
                                    });
                                    console.log('✅ [BookingSidebar] Auto-Sync erfolgreich');
                                  }
                                } catch (error) {
                                  console.error('❌ Fehler beim Löschen des Service:', error);
                                  setError('Fehler beim Löschen des Service');
                                }
                              } else {
                                setServices(services.filter((_, i) => i !== index));
                              }
                            }}
                            className="text-red-600 hover:bg-red-50 p-1 rounded transition-colors"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {/* Template Selection */}
                {serviceTemplates.length > 0 && (
                  <div className="mb-4">
                    <label className="block text-xs font-medium text-slate-600 mb-2">
                      Aus Vorlagen wählen:
                    </label>
                    <div className="grid grid-cols-2 gap-2">
                      {serviceTemplates.map(template => {
                        // Prüfe ob Template bereits hinzugefügt wurde
                        const isAlreadyAdded = services.some(s => s.template_id === template.id);

                        return (
                          <button
                            key={template.id}
                            type="button"
                            onClick={async () => {
                              // Verhindere doppeltes Hinzufügen
                              if (isAlreadyAdded) {
                                setError('Dieser Service ist bereits hinzugefügt!');
                                setTimeout(() => setError(null), 3000);
                                return;
                              }

                              const newService: AdditionalService = {
                                service_name: template.name,
                                service_price: template.price,
                                template_id: template.id,
                              };

                              if (booking?.id) {
                                // EDIT mode - Service sofort zum Backend hinzufügen
                                try {
                                  // 1. Service-Template zum Backend linken
                                  await invoke('link_service_template_to_booking_command', {
                                    bookingId: booking.id,
                                    serviceTemplateId: template.id,
                                  });

                                  // 2. Buchung NEU laden (inkl. Services mit Emojis)
                                  await reloadBooking(booking.id);

                                  // 3. Lokalen Sidebar-State aktualisieren
                                  const updatedServices = await invoke<AdditionalService[]>('get_booking_services_command', { bookingId: booking.id });
                                  setServices(updatedServices);

                                  // 4. AUTO-SYNC zu Turso (Mobile App) - Service-Emojis aktualisieren
                                  if (booking.checkout_date) {
                                    console.log('🔄 [BookingSidebar] Service-Template verknüpft - Auto-Sync zu Turso für', booking.checkout_date);
                                    await invoke('sync_affected_dates', {
                                      oldCheckout: null,
                                      newCheckout: booking.checkout_date
                                    });
                                    console.log('✅ [BookingSidebar] Auto-Sync erfolgreich');
                                  }
                                } catch (error) {
                                  console.error('❌ Fehler beim Verknüpfen des Service-Templates:', error);
                                  setError('Fehler beim Verknüpfen des Service-Templates');
                                }
                              } else {
                                // CREATE mode - nur lokaler State
                                setServices([...services, newService]);
                              }
                            }}
                            disabled={isAlreadyAdded}
                            className={`flex items-center justify-between px-3 py-2 rounded-lg transition-colors text-left ${
                              isAlreadyAdded
                                ? 'bg-slate-100 border border-slate-200 opacity-50 cursor-not-allowed'
                                : 'bg-emerald-50 hover:bg-emerald-100 border border-emerald-200'
                            }`}
                          >
                          <span className="text-sm font-medium text-emerald-900 truncate">
                            {template.name}
                          </span>
                          <span className="text-xs font-semibold text-emerald-600 ml-2">
                            {template.price.toFixed(2)} €
                          </span>
                        </button>
                        );
                      })}
                    </div>
                  </div>
                )}

                {/* Add New Service */}
                <div className="space-y-3">
                  <label className="block text-xs font-medium text-slate-600">
                    Oder manuell hinzufügen:
                  </label>
                  <div className="grid grid-cols-2 gap-3">
                    <input
                      type="text"
                      placeholder="Service-Name *"
                      value={newService.service_name}
                      onChange={(e) => setNewService({ ...newService, service_name: e.target.value })}
                      className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                    />
                    <input
                      type="number"
                      placeholder="Preis in € *"
                      min="0"
                      step="0.01"
                      value={newService.service_price || ''}
                      onChange={(e) => setNewService({ ...newService, service_price: parseFloat(e.target.value) || 0 })}
                      className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                    />
                  </div>
                  <button
                    type="button"
                    onClick={async () => {
                      if (!newService.service_name || newService.service_price <= 0) {
                        setError('Bitte Service-Name und Preis eingeben (Preis muss > 0 sein)');
                        return;
                      }

                      if (booking?.id) {
                        try {
                          // 1. Service zum Backend hinzufügen
                          await invoke('add_service_command', {
                            bookingId: booking.id,
                            serviceName: newService.service_name,
                            servicePrice: newService.service_price,
                          });

                          // 2. Buchung NEU laden (inkl. Services mit Emojis)
                          await reloadBooking(booking.id);

                          // 3. Lokalen Sidebar-State aktualisieren
                          const updatedServices = await invoke<AdditionalService[]>('get_booking_services_command', { bookingId: booking.id });
                          setServices(updatedServices);

                          // 4. AUTO-SYNC zu Turso (Mobile App) - Service-Emojis aktualisieren
                          if (booking.checkout_date) {
                            console.log('🔄 [BookingSidebar] Service hinzugefügt - Auto-Sync zu Turso für', booking.checkout_date);
                            await invoke('sync_affected_dates', {
                              oldCheckout: null,
                              newCheckout: booking.checkout_date
                            });
                            console.log('✅ [BookingSidebar] Auto-Sync erfolgreich');
                          }

                          setNewService({ service_name: '', service_price: 0 });
                          setError(null);
                        } catch (error) {
                          console.error('❌ Fehler beim Hinzufügen des Service:', error);
                          setError('Fehler beim Hinzufügen des Service');
                        }
                      } else {
                        setServices([...services, { ...newService }]);
                        setNewService({ service_name: '', service_price: 0 });
                        setError(null);
                      }
                    }}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg transition-colors text-sm font-semibold"
                  >
                    <Plus className="w-4 h-4" />
                    Service hinzufügen
                  </button>
                </div>
              </div>

              {/* Discounts */}
              <div className="border border-slate-200 rounded-lg p-4">
                <h3 className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-4">
                  <Percent className="w-4 h-4" />
                  Rabatte
                </h3>

                {/* List of Discounts */}
                {discounts.length > 0 && (
                  <div className="mb-4 space-y-2">
                    {discounts.map((discount, index) => (
                      <div
                        key={discount.id || index}
                        className="flex items-center justify-between bg-slate-50 px-4 py-2 rounded-lg border border-slate-200"
                      >
                        <div className="flex items-center gap-3">
                          <span className="text-sm font-semibold text-slate-900">
                            {discount.discount_name}
                          </span>
                          <span className="text-sm font-semibold text-orange-600">
                            {discount.discount_type === 'percent'
                              ? `${discount.discount_value}%`
                              : `${discount.discount_value.toFixed(2)} €`}
                          </span>
                        </div>
                        <div className="flex items-center gap-1">
                          <button
                            type="button"
                            onClick={async () => {
                              try {
                                await invoke('create_discount_template_command', {
                                  name: discount.discount_name,
                                  description: null,
                                  discountType: discount.discount_type,
                                  discountValue: discount.discount_value,
                                });
                                await loadTemplates();
                                setShowSuccessDialog({ show: true, message: 'Rabatt als Vorlage gespeichert!' });
                              } catch (err) {
                                setShowErrorDialog({ show: true, message: `Fehler: ${err}` });
                              }
                            }}
                            className="text-amber-600 hover:bg-amber-50 p-1 rounded transition-colors"
                            title="Als Vorlage speichern"
                          >
                            <Bookmark className="w-4 h-4" />
                          </button>
                          <button
                            type="button"
                            onClick={() => {
                              if (discount.id && booking?.id) {
                                invoke('delete_discount_command', { discountId: discount.id }).then(() => {
                                  invoke<Discount[]>('get_booking_discounts_command', { bookingId: booking.id }).then(setDiscounts);
                                });
                              } else {
                                setDiscounts(discounts.filter((_, i) => i !== index));
                              }
                            }}
                            className="text-red-600 hover:bg-red-50 p-1 rounded transition-colors"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {/* Template Selection */}
                {discountTemplates.length > 0 && (
                  <div className="mb-4">
                    <label className="block text-xs font-medium text-slate-600 mb-2">
                      Aus Vorlagen wählen:
                    </label>
                    <div className="grid grid-cols-2 gap-2">
                      {discountTemplates.map(template => (
                        <button
                          key={template.id}
                          type="button"
                          onClick={() => {
                            const newDiscount: Discount = {
                              discount_name: template.name,
                              discount_type: template.discount_type,
                              discount_value: template.discount_value,
                              template_id: template.id,
                            };
                            setDiscounts([...discounts, newDiscount]);
                          }}
                          className="flex items-center justify-between px-3 py-2 bg-amber-50 hover:bg-amber-100 border border-amber-200 rounded-lg transition-colors text-left"
                        >
                          <span className="text-sm font-medium text-amber-900 truncate">
                            {template.name}
                          </span>
                          <span className="text-xs font-semibold text-amber-600 ml-2">
                            {template.discount_type === 'percent'
                              ? `${template.discount_value}%`
                              : `${template.discount_value.toFixed(2)} €`}
                          </span>
                        </button>
                      ))}
                    </div>
                  </div>
                )}

                {/* Add New Discount */}
                <div className="space-y-3">
                  <label className="block text-xs font-medium text-slate-600">
                    Oder manuell hinzufügen:
                  </label>
                  <div className="grid grid-cols-3 gap-3">
                    <input
                      type="text"
                      placeholder="Rabatt-Name *"
                      value={newDiscount.discount_name}
                      onChange={(e) => setNewDiscount({ ...newDiscount, discount_name: e.target.value })}
                      className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                    />
                    <select
                      value={newDiscount.discount_type}
                      onChange={(e) => setNewDiscount({ ...newDiscount, discount_type: e.target.value as 'percent' | 'fixed' })}
                      className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                    >
                      <option value="percent">Prozent (%)</option>
                      <option value="fixed">Fester Betrag (€)</option>
                    </select>
                    <input
                      type="number"
                      placeholder={newDiscount.discount_type === 'percent' ? 'Prozent *' : 'Betrag in € *'}
                      min="0"
                      step={newDiscount.discount_type === 'percent' ? '1' : '0.01'}
                      value={newDiscount.discount_value || ''}
                      onChange={(e) => setNewDiscount({ ...newDiscount, discount_value: parseFloat(e.target.value) || 0 })}
                      className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                    />
                  </div>
                  <button
                    type="button"
                    onClick={() => {
                      if (!newDiscount.discount_name || newDiscount.discount_value <= 0) {
                        setError('Bitte Rabatt-Name und Wert eingeben (Wert muss > 0 sein)');
                        return;
                      }

                      if (booking?.id) {
                        invoke('add_discount_command', {
                          bookingId: booking.id,
                          discountName: newDiscount.discount_name,
                          discountType: newDiscount.discount_type,
                          discountValue: newDiscount.discount_value,
                        }).then(() => {
                          invoke<Discount[]>('get_booking_discounts_command', { bookingId: booking.id! }).then(setDiscounts);
                          setNewDiscount({ discount_name: '', discount_type: 'percent', discount_value: 0 });
                          setError(null);
                        });
                      } else {
                        setDiscounts([...discounts, { ...newDiscount }]);
                        setNewDiscount({ discount_name: '', discount_type: 'percent', discount_value: 0 });
                        setError(null);
                      }
                    }}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg transition-colors text-sm font-semibold"
                  >
                    <Plus className="w-4 h-4" />
                    Rabatt hinzufügen
                  </button>
                </div>
              </div>
            </div>
          ) : null}
        </div>

        {/* Footer */}
        <div className="absolute bottom-0 left-0 right-0 px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-between">
          {mode === 'view' && booking ? (
            <>
              <div className="flex items-center gap-2">
                {booking.status !== 'storniert' && (
                  <>
                    <button
                      onClick={handleSwitchToEdit}
                      className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <Edit2 className="w-4 h-4" />
                      Bearbeiten
                    </button>
                    <button
                      onClick={() => setShowCancelDialog(true)}
                      className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-semibold transition-colors text-sm"
                    >
                      <XCircle className="w-4 h-4" />
                      Stornieren
                    </button>
                  </>
                )}
              </div>
              <button
                onClick={handleClose}
                className="px-6 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-lg font-semibold transition-colors"
              >
                Schließen
              </button>
            </>
          ) : (mode === 'edit' || mode === 'create') ? (
            <>
              <button
                onClick={handleCancelEdit}
                className="px-4 py-2 text-slate-700 hover:bg-slate-200 rounded-lg transition-colors font-semibold"
              >
                Abbrechen
              </button>
              <button
                onClick={handleSubmit}
                disabled={loading}
                className="px-6 py-2 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white rounded-lg font-semibold shadow-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loading ? 'Speichere...' : booking ? 'Aktualisieren' : 'Erstellen'}
              </button>
            </>
          ) : null}
        </div>
      </div>

      {/* Email Selection Dialog */}
      {createdBookingId && (
        <EmailSelectionDialog
          isOpen={showEmailDialog}
          onClose={() => {
            setShowEmailDialog(false);
            setCreatedBookingId(null);
            handleClose(); // Close sidebar after email dialog
          }}
          bookingId={createdBookingId}
          guestEmail={createdGuestEmail}
          istStiftungsfall={formData.ist_stiftungsfall || false}
        />
      )}

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
                onClick={async () => {
                  if (!booking.id) return;
                  try {
                    await invoke('cancel_booking_command', { id: booking.id });
                    setShowCancelDialog(false);
                    await loadBookingDetails();
                    setShowSuccessDialog({ show: true, message: 'Buchung erfolgreich storniert!' });
                  } catch (error) {
                    console.error('Fehler beim Stornieren der Buchung:', error);
                    setShowCancelDialog(false);
                    setShowErrorDialog({ show: true, message: 'Fehler beim Stornieren der Buchung' });
                  }
                }}
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
    </>
  );
}
