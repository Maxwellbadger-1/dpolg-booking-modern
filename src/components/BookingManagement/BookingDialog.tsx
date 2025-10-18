import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Calendar, Users, MessageSquare, UserPlus, DollarSign, Tag, CheckCircle, AlertCircle, Loader2, UserCheck, Trash2, Plus, ShoppingBag, Percent, Bookmark, AlertTriangle, Info, MapPin, Briefcase, ChevronDown, ChevronUp, Search, History, Wallet } from 'lucide-react';
import { useData } from '../../context/DataContext';
import SearchableGuestPicker from './SearchableGuestPicker';
import SearchableRoomPicker from './SearchableRoomPicker';
import EmailSelectionDialog from './EmailSelectionDialog';
import CompanionSelector from './CompanionSelector';
import BookingReminders from '../Reminders/BookingReminders';

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
  beruf?: string;
  bundesland?: string;
  dienststelle?: string;
  created_at?: string;
}

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  ort: string;
  capacity: number;
  price_member: number;
  price_non_member: number;
}

interface Booking {
  id?: number;
  room_id: number;
  guest_id: number;
  checkin_date: string;
  checkout_date: string;
  anzahl_gaeste: number;
  status: string;
  bemerkungen?: string;
  ist_stiftungsfall?: boolean;
  payment_recipient_id?: number | null;
  putzplan_checkout_date?: string | null;
}

interface PaymentRecipient {
  id: number;
  name: string;
  company?: string;
  street?: string;
  plz?: string;
  city?: string;
  country: string;
}

interface GuestCreditBalance {
  guestId: number;
  balance: number;
  totalAdded: number;
  totalUsed: number;
}

interface AccompanyingGuest {
  id?: number;
  vorname: string;
  nachname: string;
  geburtsdatum?: string;
}

interface AdditionalService {
  id?: number;
  service_name: string;
  service_price: number;
  template_id?: number; // Wenn aus Template erstellt, ID des Templates
}

interface Discount {
  id?: number;
  discount_name: string;
  discount_type: 'percent' | 'fixed';
  discount_value: number;
  template_id?: number; // Wenn aus Template erstellt, ID des Templates
}

interface BookingDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  booking?: Booking;
  prefillData?: { roomId?: number; checkinDate?: string; checkoutDate?: string };
}

const STATUS_OPTIONS = [
  { value: 'bestaetigt', label: 'Bestätigt', color: 'bg-emerald-100 text-emerald-700' },
  { value: 'eingecheckt', label: 'Eingecheckt', color: 'bg-blue-100 text-blue-700' },
  { value: 'ausgecheckt', label: 'Ausgecheckt', color: 'bg-slate-100 text-slate-700' },
  { value: 'storniert', label: 'Storniert', color: 'bg-red-100 text-red-700' },
];

export default function BookingDialog({ isOpen, onClose, onSuccess, booking, prefillData }: BookingDialogProps) {
  const { createBooking, updateBooking, reloadBooking } = useData();

  const [formData, setFormData] = useState<Booking>({
    room_id: 0,
    guest_id: 0,
    checkin_date: '',
    checkout_date: '',
    anzahl_gaeste: 1,
    status: 'bestaetigt',
    bemerkungen: '',
    payment_recipient_id: null, // FIX: Initialize to null (not undefined!)
  });

  const [guests, setGuests] = useState<Guest[]>([]);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [paymentRecipients, setPaymentRecipients] = useState<PaymentRecipient[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [priceInfo, setPriceInfo] = useState<any>(null);

  console.log('🚀 [BookingDialog] Component rendered', { isOpen, formData, priceInfo });
  const [availabilityStatus, setAvailabilityStatus] = useState<{
    checking: boolean;
    available: boolean | null;
  }>({ checking: false, available: null });

  const [accompanyingGuests, setAccompanyingGuests] = useState<AccompanyingGuest[]>([]);

  const [additionalServices, setAdditionalServices] = useState<AdditionalService[]>([]);
  const [newService, setNewService] = useState<AdditionalService>({
    service_name: '',
    service_price: 0,
  });

  const [discounts, setDiscounts] = useState<Discount[]>([]);
  const [newDiscount, setNewDiscount] = useState<Discount>({
    discount_name: '',
    discount_type: 'percent',
    discount_value: 0,
  });

  // Template States
  const [serviceTemplates, setServiceTemplates] = useState<any[]>([]);
  const [discountTemplates, setDiscountTemplates] = useState<any[]>([]);

  // Email Dialog State
  const [showEmailDialog, setShowEmailDialog] = useState(false);
  const [createdBookingId, setCreatedBookingId] = useState<number | null>(null);
  const [createdGuestEmail, setCreatedGuestEmail] = useState<string>('');

  // Buchungshistorie States
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

  // Putzplan Checkout State
  const [useAlternativeCleaningDate, setUseAlternativeCleaningDate] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadGuestsAndRooms();
      loadTemplates();
    }
  }, [isOpen]);

  useEffect(() => {
    if (booking) {
      setFormData({
        ...booking,
        bemerkungen: booking.bemerkungen || '',
        payment_recipient_id: booking.payment_recipient_id || null,
        putzplan_checkout_date: booking.putzplan_checkout_date || null,
      });
      // Set checkbox state based on whether alternative date exists
      setUseAlternativeCleaningDate(!!booking.putzplan_checkout_date);
      // Load accompanying guests, services and discounts if editing
      if (booking.id) {
        loadAccompanyingGuests(booking.id);
        loadAdditionalServices(booking.id);
        loadDiscounts(booking.id);
      }
    } else if (prefillData) {
      // Drag-to-Create: Prefill with room and dates
      setFormData({
        room_id: prefillData.roomId || 0,
        guest_id: 0,
        checkin_date: prefillData.checkinDate || '',
        checkout_date: prefillData.checkoutDate || '',
        anzahl_gaeste: 1,
        status: 'bestaetigt',
        bemerkungen: '',
        payment_recipient_id: null, // FIX: Initialize to null (not undefined!)
        putzplan_checkout_date: null,
      });
      setUseAlternativeCleaningDate(false);
      setAccompanyingGuests([]);
      setAdditionalServices([]);
      setDiscounts([]);
    } else {
      setFormData({
        room_id: 0,
        guest_id: 0,
        checkin_date: '',
        checkout_date: '',
        anzahl_gaeste: 1,
        status: 'bestaetigt',
        bemerkungen: '',
        payment_recipient_id: null, // FIX: Initialize to null (not undefined!)
        putzplan_checkout_date: null,
      });
      setUseAlternativeCleaningDate(false);
      setAccompanyingGuests([]);
      setAdditionalServices([]);
      setDiscounts([]);
    }
    setError(null);
    setPriceInfo(null);
    setAvailabilityStatus({ checking: false, available: null });
    setNewService({ service_name: '', service_price: 0 });
    setNewDiscount({ discount_name: '', discount_type: 'percent', discount_value: 0 });
  }, [booking, prefillData, isOpen]);

  // Calculate price when relevant fields change
  useEffect(() => {
    console.log('🔄 [useEffect] Price calculation trigger check:', {
      room_id: formData.room_id,
      guest_id: formData.guest_id,
      checkin: formData.checkin_date,
      checkout: formData.checkout_date,
      guests_count: guests.length,
      rooms_count: rooms.length,
    });

    if (formData.room_id && formData.guest_id && formData.checkin_date && formData.checkout_date && guests.length > 0 && rooms.length > 0) {
      console.log('✅ [useEffect] Calling calculatePrice()');
      calculatePrice();
    } else {
      console.log('⚠️ [useEffect] Skipping calculatePrice - missing data');
      setPriceInfo(null); // Clear price if data incomplete
    }
  }, [formData.room_id, formData.guest_id, formData.checkin_date, formData.checkout_date, additionalServices, discounts, guests, rooms]);

  // Check room availability when room or dates change
  useEffect(() => {
    if (formData.room_id && formData.checkin_date && formData.checkout_date) {
      checkAvailability();
    } else {
      setAvailabilityStatus({ checking: false, available: null });
    }
  }, [formData.room_id, formData.checkin_date, formData.checkout_date]);

  // Load guest credit balance when guest is selected
  useEffect(() => {
    const loadCreditBalance = async () => {
      if (formData.guest_id > 0 && isOpen) {
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
  }, [formData.guest_id, isOpen]);

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

  const addServiceFromTemplate = (template: any) => {
    const newService: AdditionalService = {
      service_name: template.name,
      service_price: template.price,
      template_id: template.id, // ✅ Template-ID speichern!
    };
    setAdditionalServices([...additionalServices, newService]);
  };

  const addDiscountFromTemplate = (template: any) => {
    const newDiscount: Discount = {
      discount_name: template.name,
      discount_type: template.discount_type,
      discount_value: template.discount_value,
      template_id: template.id, // ✅ Template-ID speichern!
    };
    setDiscounts([...discounts, newDiscount]);
  };

  const saveServiceAsTemplate = async (service: AdditionalService) => {
    try {
      await invoke('create_service_template_command', {
        name: service.service_name,
        description: null,
        price: service.service_price,
      });
      // Reload templates
      loadTemplates();
      alert('Service als Vorlage gespeichert!');
    } catch (err) {
      console.error('Fehler beim Speichern:', err);
      alert(err instanceof Error ? err.message : String(err));
    }
  };

  const saveDiscountAsTemplate = async (discount: Discount) => {
    try {
      await invoke('create_discount_template_command', {
        name: discount.discount_name,
        description: null,
        discountType: discount.discount_type,
        discountValue: discount.discount_value,
      });
      // Reload templates
      loadTemplates();
      alert('Rabatt als Vorlage gespeichert!');
    } catch (err) {
      console.error('Fehler beim Speichern:', err);
      alert(err instanceof Error ? err.message : String(err));
    }
  };

  const calculatePrice = async () => {
    try {
      console.log('💰 [calculatePrice] START', {
        room_id: formData.room_id,
        guest_id: formData.guest_id,
        checkin: formData.checkin_date,
        checkout: formData.checkout_date,
        guests_count: guests.length,
        rooms_count: rooms.length,
      });

      const guest = guests.find(g => g.id === formData.guest_id);
      const room = rooms.find(r => r.id === formData.room_id);

      console.log('💰 [calculatePrice] Found:', { guest: !!guest, room: !!room });

      if (!guest || !room || !formData.checkin_date || !formData.checkout_date) {
        console.warn('⚠️ [calculatePrice] Missing data - aborting');
        return;
      }

      // Use backend command for price calculation (supports seasonal pricing + auto DPolG discount + Endreinigung)
      const priceResult = await invoke<{
        grundpreis: number;
        servicesPreis: number;
        rabattPreis: number;
        gesamtpreis: number;
        anzahlNaechte: number;
        istHauptsaison: boolean;
      }>('calculate_booking_price_command', {
        roomId: formData.room_id,
        checkin: formData.checkin_date,
        checkout: formData.checkout_date,
        isMember: guest.dpolg_mitglied,
        services: additionalServices.map(s => [s.service_name, s.service_price]),
        discounts: discounts.map(d => [d.discount_name, d.discount_type, d.discount_value]),
      });

      console.log('✅ [calculatePrice] Backend result:', priceResult);

      const pricePerNight = priceResult.grundpreis / (priceResult.anzahlNaechte || 1);

      const newPriceInfo = {
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
      };

      console.log('📦 [calculatePrice] Setting priceInfo to:', newPriceInfo);
      setPriceInfo(newPriceInfo);
      console.log('✅ [calculatePrice] Price info set successfully');
    } catch (err) {
      console.error('❌ [calculatePrice] Fehler:', err);
      setPriceInfo(null);
    }
  };

  const checkAvailability = async () => {
    try {
      console.log('🔍 [BookingDialog] checkAvailability gestartet', {
        room_id: formData.room_id,
        checkin: formData.checkin_date,
        checkout: formData.checkout_date,
        excludeBookingId: booking?.id || null
      });

      setAvailabilityStatus({ checking: true, available: null });

      const isAvailable = await invoke<boolean>('check_room_availability_command', {
        roomId: formData.room_id,
        checkin: formData.checkin_date,
        checkout: formData.checkout_date,
        excludeBookingId: booking?.id || null,
      });

      console.log('✅ [BookingDialog] checkAvailability Ergebnis:', isAvailable);
      setAvailabilityStatus({ checking: false, available: isAvailable });
    } catch (err) {
      console.error('❌ [BookingDialog] Fehler bei Verfügbarkeitsprüfung:', err);
      setAvailabilityStatus({ checking: false, available: null });
    }
  };

  const loadAccompanyingGuests = async (bookingId: number) => {
    try {
      const guests = await invoke<AccompanyingGuest[]>('get_booking_accompanying_guests_command', {
        bookingId,
      });
      setAccompanyingGuests(guests);
    } catch (err) {
      console.error('Fehler beim Laden der Begleitpersonen:', err);
    }
  };

  const loadGuestBookings = async (guestId: number) => {
    try {
      const bookings = await invoke<any[]>('get_all_bookings');
      // Filter für Buchungen des ausgewählten Gastes (außer aktuelle Buchung wenn Edit-Mode)
      const guestFilteredBookings = bookings
        .filter((b) => b.guest_id === guestId && b.id !== booking?.id)
        .sort((a, b) => new Date(b.checkin_date).getTime() - new Date(a.checkin_date).getTime());
      setGuestBookings(guestFilteredBookings);
    } catch (err) {
      console.error('Fehler beim Laden der Buchungshistorie:', err);
      setGuestBookings([]);
    }
  };

  const loadAdditionalServices = async (bookingId: number) => {
    try {
      const services = await invoke<AdditionalService[]>('get_booking_services_command', {
        bookingId,
      });
      setAdditionalServices(services);
    } catch (err) {
      console.error('Fehler beim Laden der Services:', err);
    }
  };

  const handleAddService = () => {
    if (!newService.service_name || newService.service_price <= 0) {
      setError('Bitte Service-Name und Preis eingeben (Preis muss > 0 sein)');
      return;
    }

    // If editing existing booking, save to database immediately
    if (booking?.id) {
      saveService();
    } else {
      // If creating new booking, just add to temporary list
      setAdditionalServices([...additionalServices, { ...newService }]);
      setNewService({ service_name: '', service_price: 0 });
      setError(null);
    }
  };

  const saveService = async () => {
    if (!booking?.id) return;

    try {
      await invoke('add_service_command', {
        bookingId: booking.id,
        serviceName: newService.service_name,
        servicePrice: newService.service_price,
      });
      await loadAdditionalServices(booking.id);
      setNewService({ service_name: '', service_price: 0 });
      setError(null);
    } catch (err) {
      console.error('Fehler beim Hinzufügen des Services:', err);
      setError('Fehler beim Hinzufügen des Services');
    }
  };

  const handleRemoveService = async (index: number, serviceId?: number) => {
    if (serviceId && booking?.id) {
      // Delete from database
      try {
        await invoke('delete_service_command', {
          serviceId,
        });
        await loadAdditionalServices(booking.id);
      } catch (err) {
        console.error('Fehler beim Löschen des Services:', err);
        setError('Fehler beim Löschen des Services');
      }
    } else {
      // Remove from temporary list
      setAdditionalServices(additionalServices.filter((_, i) => i !== index));
    }
  };

  const loadDiscounts = async (bookingId: number) => {
    try {
      const discountData = await invoke<Discount[]>('get_booking_discounts_command', {
        bookingId,
      });
      setDiscounts(discountData);
    } catch (err) {
      console.error('Fehler beim Laden der Rabatte:', err);
    }
  };

  const handleAddDiscount = () => {
    if (!newDiscount.discount_name || newDiscount.discount_value <= 0) {
      setError('Bitte Rabatt-Name und Wert eingeben (Wert muss > 0 sein)');
      return;
    }

    // If editing existing booking, save to database immediately
    if (booking?.id) {
      saveDiscount();
    } else {
      // If creating new booking, just add to temporary list
      setDiscounts([...discounts, { ...newDiscount }]);
      setNewDiscount({ discount_name: '', discount_type: 'percent', discount_value: 0 });
      setError(null);
    }
  };

  const saveDiscount = async () => {
    if (!booking?.id) return;

    try {
      await invoke('add_discount_command', {
        bookingId: booking.id,
        discountName: newDiscount.discount_name,
        discountType: newDiscount.discount_type,
        discountValue: newDiscount.discount_value,
      });
      await loadDiscounts(booking.id);
      setNewDiscount({ discount_name: '', discount_type: 'percent', discount_value: 0 });
      setError(null);
    } catch (err) {
      console.error('Fehler beim Hinzufügen des Rabatts:', err);
      setError('Fehler beim Hinzufügen des Rabatts');
    }
  };

  const handleRemoveDiscount = async (index: number, discountId?: number) => {
    if (discountId && booking?.id) {
      // Delete from database
      try {
        await invoke('delete_discount_command', {
          discountId,
        });
        await loadDiscounts(booking.id);
      } catch (err) {
        console.error('Fehler beim Löschen des Rabatts:', err);
        setError('Fehler beim Löschen des Rabatts');
      }
    } else {
      // Remove from temporary list
      setDiscounts(discounts.filter((_, i) => i !== index));
    }
  };

  const validateForm = (): boolean => {
    console.log('🔍 [BookingDialog] validateForm aufgerufen', {
      guest_id: formData.guest_id,
      room_id: formData.room_id,
      checkin_date: formData.checkin_date,
      checkout_date: formData.checkout_date,
      availabilityStatus
    });

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

    // Check room availability - KRITISCH!
    console.log('🔍 [BookingDialog] Availability Check:', {
      checking: availabilityStatus.checking,
      available: availabilityStatus.available
    });

    if (availabilityStatus.checking) {
      setError('Bitte warten Sie, während die Verfügbarkeit geprüft wird...');
      return false;
    }

    if (availabilityStatus.available === false) {
      console.log('❌ [BookingDialog] Zimmer NICHT verfügbar - Validierung fehlgeschlagen');
      setError('Das Zimmer ist für den gewählten Zeitraum nicht verfügbar');
      return false;
    }

    if (availabilityStatus.available === null) {
      console.log('⚠️ [BookingDialog] Verfügbarkeit noch nicht geprüft!');
      setError('Verfügbarkeit konnte nicht geprüft werden. Bitte warten...');
      return false;
    }

    console.log('✅ [BookingDialog] Validierung erfolgreich - Zimmer verfügbar');
    return true;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    console.log('🚀 [handleSubmit] START');
    console.log('  📦 formData.payment_recipient_id:', formData.payment_recipient_id, 'type:', typeof formData.payment_recipient_id);
    console.log('  📦 Complete formData:', JSON.stringify(formData, null, 2));

    if (!validateForm()) return;

    setError(null);
    setLoading(true);

    try {
      if (booking?.id) {
        // Calculate price info for update
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
          anzahlBegleitpersonen: 0,
          grundpreis: basePrice,
          servicesPreis: servicesTotal,
          rabattPreis: discountsTotal,
          anzahlNaechte: nights,
          istStiftungsfall: formData.ist_stiftungsfall || false,
          paymentRecipientId: formData.payment_recipient_id || null, // ✅ FIX: camelCase for Tauri auto-conversion
          putzplanCheckoutDate: formData.putzplan_checkout_date || null, // ✅ Alternative Cleaning Checkout
        };

        console.log('📤 [updatePayload] Payload being sent to updateBooking:');
        console.log('  paymentRecipientId:', updatePayload.paymentRecipientId, 'type:', typeof updatePayload.paymentRecipientId);
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

        // Bei Update direkt schließen
        onSuccess();
        onClose();
      } else {
        // Create booking - need to generate reservation number first
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
          paymentRecipientId: formData.payment_recipient_id || null,
          putzplanCheckoutDate: formData.putzplan_checkout_date || null, // ✅ Alternative Cleaning Checkout
        };

        const result = await createBooking(bookingData) as any;

        // Save accompanying guests if any
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

        // Save additional services if any
        console.log('🔍 [BookingDialog] Saving services...', {
          count: additionalServices.length,
          services: additionalServices,
          bookingId: result.id
        });

        if (additionalServices.length > 0 && result.id) {
          for (const service of additionalServices) {
            console.log('🔍 [BookingDialog] Processing service:', {
              service_name: service.service_name,
              template_id: service.template_id,
              has_template: !!service.template_id
            });

            if (service.template_id) {
              // Template-basierter Service → Junction-Table verwenden
              console.log('🔗 [BookingDialog] Calling link_service_template_to_booking_command');
              await invoke('link_service_template_to_booking_command', {
                bookingId: result.id,
                serviceTemplateId: service.template_id,
              });
              console.log('✅ [BookingDialog] Service template linked successfully');
            } else {
              // Custom Service → alte Tabelle verwenden
              console.log('📝 [BookingDialog] Calling add_service_command');
              await invoke('add_service_command', {
                bookingId: result.id,
                serviceName: service.service_name,
                servicePrice: service.service_price,
              });
              console.log('✅ [BookingDialog] Custom service added successfully');
            }
          }
        } else {
          console.log('⚠️ [BookingDialog] No services to save or no booking ID');
        }

        // Save discounts if any
        if (discounts.length > 0 && result.id) {
          for (const discount of discounts) {
            if (discount.template_id) {
              // Template-basierter Rabatt → Junction-Table verwenden
              await invoke('link_discount_template_to_booking_command', {
                bookingId: result.id,
                discountTemplateId: discount.template_id,
              });
            } else {
              // Custom Rabatt → alte Tabelle verwenden
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

        // ✅ Booking nochmal laden um Services/Discounts/Emojis zu erhalten (Optimistic Update!)
        console.log('🔄 [BookingDialog] Lade Buchung neu mit Services...');
        await reloadBooking(result.id);
        console.log('✅ [BookingDialog] Buchung neu geladen - Services sollten jetzt sichtbar sein!');

        // 🆕 Email-Auswahl-Dialog öffnen (IMMER - bei Stiftungsfall ist Rechnung ausgegraut)
        if (result.id) {
          const guestEmail = guests.find(g => g.id === formData.guest_id)?.email || '';
          setCreatedBookingId(result.id);
          setCreatedGuestEmail(guestEmail);
          setShowEmailDialog(true);
          // Dialog bleibt offen, wird vom EmailSelectionDialog geschlossen
        }
      }
    } catch (err) {
      console.error('Fehler beim Speichern:', err);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <>
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-500 to-blue-600 px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="bg-white/20 p-2 rounded-lg">
              <Calendar className="w-6 h-6 text-white" />
            </div>
            <h2 className="text-xl font-bold text-white">
              {booking ? 'Buchung bearbeiten' : 'Neue Buchung'}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="text-white/80 hover:text-white transition-colors p-1 rounded-lg hover:bg-white/10"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 overflow-y-auto max-h-[calc(90vh-200px)]">
          {error && (
            <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-700">{error}</p>
            </div>
          )}

          <div className="space-y-6">
            {/* Guest & Room Selection */}
            <div className="grid grid-cols-2 gap-4">
              <SearchableGuestPicker
                guests={guests}
                selectedGuestId={formData.guest_id}
                onSelectGuest={(guestId) => setFormData({ ...formData, guest_id: guestId })}
                onCreateNew={() => {
                  // TODO: Öffne GuestDialog zum Erstellen eines neuen Gastes
                  alert('Neuen Gast anlegen - Feature kommt bald!');
                }}
              />

              <SearchableRoomPicker
                rooms={rooms}
                selectedRoomId={formData.room_id}
                onSelectRoom={(roomId) => setFormData({ ...formData, room_id: roomId })}
              />
            </div>

            {/* Payment Recipient Selection */}
            {paymentRecipients.length > 0 && (
              <div>
                <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                  <Briefcase className="w-4 h-4" />
                  Rechnungsempfänger (optional)
                </label>
                <select
                  value={formData.payment_recipient_id || ''}
                  onChange={(e) => {
                    console.log('🎯 [DROPDOWN] onChange fired!');
                    console.log('  📦 e.target.value (raw):', e.target.value, 'type:', typeof e.target.value);
                    const newValue = e.target.value ? parseInt(e.target.value) : null;
                    console.log('  🔢 Parsed newValue:', newValue, 'type:', typeof newValue);
                    console.log('  📝 Current formData.payment_recipient_id:', formData.payment_recipient_id);
                    setFormData({ ...formData, payment_recipient_id: newValue });
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
                <p className="text-xs text-slate-500 mt-1">
                  Bei Auswahl wird die Rechnung an den externen Empfänger adressiert (z.B. Polizeidienststelle statt Gast)
                </p>
              </div>
            )}

            {/* Gast-Info-Panel (wird angezeigt wenn Gast ausgewählt) */}
            {formData.guest_id > 0 && (() => {
              const selectedGuest = guests.find(g => g.id === formData.guest_id);
              if (!selectedGuest) return null;

              return (
                <div className="space-y-3">
                  {/* Gast-Details Karte */}
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
                      {selectedGuest.ort && (
                        <div>
                          <span className="text-slate-500">Ort:</span>
                          <span className="ml-2 font-medium text-slate-700">
                            {selectedGuest.plz ? `${selectedGuest.plz} ` : ''}{selectedGuest.ort}
                          </span>
                        </div>
                      )}
                      {selectedGuest.mitgliedsnummer && (
                        <div>
                          <span className="text-slate-500">Mitgl.-Nr.:</span>
                          <span className="ml-2 font-medium text-slate-700">{selectedGuest.mitgliedsnummer}</span>
                        </div>
                      )}
                      {selectedGuest.beruf && (
                        <div>
                          <span className="text-slate-500">Beruf:</span>
                          <span className="ml-2 font-medium text-slate-700">{selectedGuest.beruf}</span>
                        </div>
                      )}
                      {selectedGuest.bundesland && (
                        <div>
                          <span className="text-slate-500">Bundesland:</span>
                          <span className="ml-2 font-medium text-slate-700">{selectedGuest.bundesland}</span>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Notizen-Alert (prominent wenn vorhanden) */}
                  {selectedGuest.notizen && selectedGuest.notizen.trim() !== '' && (
                    <div className="bg-gradient-to-r from-amber-50 to-orange-50 border-2 border-amber-400 rounded-lg p-4 shadow-lg animate-pulse-slow">
                      <div className="flex items-start gap-3">
                        <div className="flex-shrink-0">
                          <AlertTriangle className="w-6 h-6 text-amber-600 animate-bounce-slow" />
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

                  {/* Buchungshistorie */}
                  <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <button
                      type="button"
                      onClick={() => {
                        if (!showBookingHistory) {
                          loadGuestBookings(selectedGuest.id);
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
                        {/* Suchfilter - IMMER anzeigen */}
                        <div className="p-3 bg-blue-50/50 rounded-lg border border-blue-200">
                          <div className="grid grid-cols-4 gap-3">
                            {/* Suche */}
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

                            {/* Status Filter */}
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

                            {/* Jahr Filter */}
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
                              {Array.from(new Set(guestBookings.map((b) => new Date(b.checkin_date).getFullYear())))
                                .sort((a, b) => b - a)
                                .map((year) => (
                                  <option key={year} value={year}>
                                    {year}
                                  </option>
                                ))}
                            </select>

                            {/* Ort Filter */}
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

                        {/* Buchungsliste */}
                        {guestBookings.length === 0 ? (
                          <div className="text-center py-6 text-slate-500">
                            Keine vorherigen Buchungen gefunden
                          </div>
                        ) : (
                          <div className="space-y-2 max-h-60 overflow-y-auto">
                            {guestBookings
                              .filter((b) => {
                                const matchesSearch =
                                  historySearch === '' ||
                                  b.reservierungsnummer.toLowerCase().includes(historySearch.toLowerCase()) ||
                                  new Date(b.checkin_date).toLocaleDateString('de-DE').includes(historySearch) ||
                                  new Date(b.checkout_date).toLocaleDateString('de-DE').includes(historySearch);

                                const matchesStatus =
                                  historyStatusFilter === 'all' || b.status === historyStatusFilter;

                                const bookingYear = new Date(b.checkin_date).getFullYear().toString();
                                const matchesYear = historyYearFilter === 'all' || bookingYear === historyYearFilter;

                                const matchesLocation =
                                  historyLocationFilter === 'all' || b.room?.ort === historyLocationFilter;

                                return matchesSearch && matchesStatus && matchesYear && matchesLocation;
                              })
                              .map((booking) => {
                                const statusColors = {
                                  bestaetigt: 'bg-emerald-100 text-emerald-700',
                                  eingecheckt: 'bg-blue-100 text-blue-700',
                                  ausgecheckt: 'bg-slate-100 text-slate-700',
                                  storniert: 'bg-red-100 text-red-700',
                                };

                                return (
                                  <div
                                    key={booking.id}
                                    className="bg-white border border-slate-200 rounded-lg p-3 hover:shadow-md transition-shadow"
                                  >
                                    <div className="flex items-start justify-between gap-3">
                                      <div className="flex-1">
                                        <div className="flex items-center gap-2 mb-1">
                                          <span className="font-semibold text-slate-700">
                                            {booking.reservierungsnummer}
                                          </span>
                                          <span
                                            className={`px-2 py-0.5 rounded text-xs font-medium ${
                                              statusColors[booking.status as keyof typeof statusColors] ||
                                              'bg-slate-100 text-slate-700'
                                            }`}
                                          >
                                            {booking.status}
                                          </span>
                                        </div>
                                        <div className="text-sm text-slate-600 space-y-0.5">
                                          <div>
                                            📅 {new Date(booking.checkin_date).toLocaleDateString('de-DE')} -{' '}
                                            {new Date(booking.checkout_date).toLocaleDateString('de-DE')} ({booking.anzahl_naechte}{' '}
                                            Nächte)
                                          </div>
                                          <div>
                                            🏠 {booking.room?.name} ({booking.room?.ort})
                                          </div>
                                          <div className="font-semibold text-slate-700">
                                            💰 {booking.gesamtpreis.toFixed(2)} €
                                          </div>
                                        </div>
                                      </div>
                                    </div>
                                  </div>
                                );
                              })}
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

            {/* Alternative Cleaning Checkout Date */}
            <div className="border-2 border-blue-300 rounded-lg p-4 bg-gradient-to-br from-blue-50 to-cyan-50">
              <label className="flex items-start gap-3 cursor-pointer mb-3">
                <input
                  type="checkbox"
                  checked={useAlternativeCleaningDate}
                  onChange={(e) => {
                    const checked = e.target.checked;
                    setUseAlternativeCleaningDate(checked);
                    if (!checked) {
                      // Clear alternative date when checkbox is unchecked
                      setFormData({ ...formData, putzplan_checkout_date: null });
                    }
                  }}
                  className="mt-0.5 w-5 h-5 accent-blue-600 bg-white border-blue-300 rounded focus:ring-2 focus:ring-blue-500"
                />
                <div className="flex-1">
                  <div className="flex items-center gap-2 text-blue-900 font-semibold mb-1">
                    <Calendar className="w-4 h-4" />
                    Alternatives Putzplan-Checkout
                  </div>
                  <p className="text-sm text-blue-700">
                    Setze ein abweichendes Checkout-Datum für die Mobile Cleaning App. Das TapeChart zeigt weiterhin das normale Checkout-Datum.
                  </p>
                </div>
              </label>

              {/* Conditional Date Picker */}
              {useAlternativeCleaningDate && (
                <div className="border-t border-blue-200 pt-3 mt-1">
                  <label className="block text-sm font-semibold text-blue-900 mb-2">
                    Putzplan Checkout-Datum *
                  </label>
                  <input
                    type="date"
                    required={useAlternativeCleaningDate}
                    value={formData.putzplan_checkout_date || ''}
                    onChange={(e) => setFormData({ ...formData, putzplan_checkout_date: e.target.value })}
                    min={formData.checkin_date}
                    className="w-full px-4 py-2 bg-white border border-blue-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-slate-700"
                  />
                  <p className="text-xs text-blue-600 mt-1">
                    📱 Dieses Datum wird in der Mobile Cleaning App als Checkout angezeigt
                  </p>
                </div>
              )}
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
                {accompanyingGuests.length > 0 && (
                  <p className="text-xs text-blue-600 mt-1">
                    ℹ️ Wird automatisch berechnet: 1 Hauptgast + {accompanyingGuests.length} Begleitperson{accompanyingGuests.length > 1 ? 'en' : ''}
                  </p>
                )}
              </div>

              <div>
                <label className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-2">
                  <Tag className="w-4 h-4" />
                  Status *
                </label>
                <div className="relative">
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
                    {STATUS_OPTIONS.map((status) => (
                      <option key={status.value} value={status.value}>
                        {status.label}
                      </option>
                    ))}
                  </select>
                </div>
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

                  {/* Einzelne Services auflisten */}
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

                  {/* Einzelne Rabatte auflisten */}
                  {priceInfo.discountsList && priceInfo.discountsList.length > 0 && (
                    <>
                      {priceInfo.discountsList.map((discount: any, index: number) => (
                        <div key={index} className="flex justify-between">
                          <span className="text-orange-700">
                            - {discount.name} {discount.type === 'percent' && `(${discount.value}%)`}:
                          </span>
                          <span className="font-semibold text-orange-700">
                            {discount.amount.toFixed(2)} €
                          </span>
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
              <div className="border border-emerald-200 rounded-lg p-4 bg-emerald-50">
                <h3 className="flex items-center gap-2 text-sm font-semibold text-emerald-900 mb-3">
                  <Wallet className="w-4 h-4" />
                  Gast-Guthaben
                </h3>

                {loadingCredit ? (
                  <div className="flex items-center gap-2 text-sm text-emerald-700">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span>Lade Guthaben...</span>
                  </div>
                ) : (
                  <div className="space-y-3">
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-emerald-700">Verfügbares Guthaben:</span>
                      <span className="font-bold text-emerald-900 text-lg">{creditBalance.toFixed(2)} €</span>
                    </div>

                    {creditBalance > 0 && priceInfo && (
                      <>
                        <div className="border-t border-emerald-300 pt-3">
                          <label className="block text-sm font-medium text-emerald-700 mb-2">
                            Guthaben verrechnen:
                          </label>
                          <div className="flex items-center gap-3">
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
                              placeholder="Betrag eingeben"
                              className="flex-1 px-4 py-2 bg-white border border-emerald-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500 text-slate-700"
                            />
                            <button
                              type="button"
                              onClick={() => setCreditToApply(Math.min(creditBalance, priceInfo.totalPrice))}
                              className="px-3 py-2 bg-emerald-600 hover:bg-emerald-700 text-white text-sm font-semibold rounded-lg transition-colors"
                            >
                              Max
                            </button>
                          </div>
                          <p className="text-xs text-emerald-600 mt-1">
                            Max. {Math.min(creditBalance, priceInfo.totalPrice).toFixed(2)} € (Guthaben oder Rechnungsbetrag)
                          </p>
                        </div>

                        {creditToApply > 0 && (
                          <div className="border-t border-emerald-300 pt-3 space-y-2">
                            <div className="flex justify-between text-sm">
                              <span className="text-emerald-700">Ursprünglicher Preis:</span>
                              <span className="font-semibold text-emerald-900">{priceInfo.totalPrice.toFixed(2)} €</span>
                            </div>
                            <div className="flex justify-between text-sm">
                              <span className="text-emerald-700">- Verrechnetes Guthaben:</span>
                              <span className="font-semibold text-emerald-700">-{creditToApply.toFixed(2)} €</span>
                            </div>
                            <div className="flex justify-between border-t border-emerald-300 pt-2">
                              <span className="font-bold text-emerald-900">Zu zahlender Betrag:</span>
                              <span className="font-bold text-emerald-900 text-lg">{(priceInfo.totalPrice - creditToApply).toFixed(2)} €</span>
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

            {/* Stiftungsfall Checkbox - FIX: ROT statt Amber (Amber wird für Gast-Notizen verwendet) */}
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
                    Diese Buchung ist ein Stiftungsfall. Der Email-Dialog wird geöffnet, aber die Rechnung-Option ist deaktiviert (PDF kann manuell erstellt werden).
                  </p>
                </div>
              </label>
            </div>

            {/* Reminders Section - Only show when editing existing booking */}
            {booking?.id && (
              <div className="border border-slate-200 rounded-lg p-4 bg-slate-50">
                <BookingReminders bookingId={booking.id} />
              </div>
            )}

            {/* Companion Selector - Neues System mit Pool */}
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
                      // Auto-update anzahl_gaeste: 1 Hauptgast + Begleitpersonen
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
              {additionalServices.length > 0 && (
                <div className="mb-4 space-y-2">
                  {additionalServices.map((service, index) => (
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
                          onClick={() => saveServiceAsTemplate(service)}
                          className="text-emerald-600 hover:bg-emerald-50 p-1 rounded transition-colors"
                          title="Als Vorlage speichern"
                        >
                          <Bookmark className="w-4 h-4" />
                        </button>
                        <button
                          type="button"
                          onClick={() => handleRemoveService(index, service.id)}
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
                    {serviceTemplates.map((template) => (
                      <button
                        key={template.id}
                        type="button"
                        onClick={() => addServiceFromTemplate(template)}
                        className="flex items-center justify-between px-3 py-2 bg-emerald-50 hover:bg-emerald-100 border border-emerald-200 rounded-lg transition-colors text-left"
                      >
                        <span className="text-sm font-medium text-emerald-900 truncate">
                          {template.name}
                        </span>
                        <span className="text-xs font-semibold text-emerald-600 ml-2">
                          {template.price.toFixed(2)} €
                        </span>
                      </button>
                    ))}
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
                  onClick={handleAddService}
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
                          onClick={() => saveDiscountAsTemplate(discount)}
                          className="text-amber-600 hover:bg-amber-50 p-1 rounded transition-colors"
                          title="Als Vorlage speichern"
                        >
                          <Bookmark className="w-4 h-4" />
                        </button>
                        <button
                          type="button"
                          onClick={() => handleRemoveDiscount(index, discount.id)}
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
                    {discountTemplates.map((template) => (
                      <button
                        key={template.id}
                        type="button"
                        onClick={() => addDiscountFromTemplate(template)}
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
                  onClick={handleAddDiscount}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg transition-colors text-sm font-semibold"
                >
                  <Plus className="w-4 h-4" />
                  Rabatt hinzufügen
                </button>
              </div>
            </div>
          </div>
        </form>

        {/* Footer */}
        <div className="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
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
        </div>
      </div>
    </div>

    {/* Email Selection Dialog */}
    {createdBookingId && (
      <EmailSelectionDialog
        isOpen={showEmailDialog}
        onClose={() => {
          setShowEmailDialog(false);
          setCreatedBookingId(null);
          onSuccess();
          onClose();
        }}
        bookingId={createdBookingId}
        guestEmail={createdGuestEmail}
        istStiftungsfall={formData.ist_stiftungsfall || false}
      />
    )}
    </>
  );
}
