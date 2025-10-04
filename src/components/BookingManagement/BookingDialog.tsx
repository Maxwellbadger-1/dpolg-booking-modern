import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Calendar, Users, MessageSquare, UserPlus, DollarSign, Tag, CheckCircle, AlertCircle, Loader2, UserCheck, Trash2, Plus, ShoppingBag, Percent, Bookmark } from 'lucide-react';
import { useData } from '../../context/DataContext';
import SearchableGuestPicker from './SearchableGuestPicker';
import SearchableRoomPicker from './SearchableRoomPicker';
import EmailSelectionDialog from './EmailSelectionDialog';

interface Guest {
  id: number;
  vorname: string;
  nachname: string;
  email: string;
  telefon: string;
  dpolg_mitglied: boolean;
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
}

interface Discount {
  id?: number;
  discount_name: string;
  discount_type: 'percent' | 'fixed';
  discount_value: number;
}

interface BookingDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  booking?: Booking;
}

const STATUS_OPTIONS = [
  { value: 'bestaetigt', label: 'Bestätigt', color: 'bg-emerald-100 text-emerald-700' },
  { value: 'eingecheckt', label: 'Eingecheckt', color: 'bg-blue-100 text-blue-700' },
  { value: 'ausgecheckt', label: 'Ausgecheckt', color: 'bg-slate-100 text-slate-700' },
  { value: 'storniert', label: 'Storniert', color: 'bg-red-100 text-red-700' },
];

export default function BookingDialog({ isOpen, onClose, onSuccess, booking }: BookingDialogProps) {
  const { createBooking, updateBooking } = useData();

  const [formData, setFormData] = useState<Booking>({
    room_id: 0,
    guest_id: 0,
    checkin_date: '',
    checkout_date: '',
    anzahl_gaeste: 1,
    status: 'bestaetigt',
    bemerkungen: '',
  });

  const [guests, setGuests] = useState<Guest[]>([]);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [priceInfo, setPriceInfo] = useState<any>(null);
  const [availabilityStatus, setAvailabilityStatus] = useState<{
    checking: boolean;
    available: boolean | null;
  }>({ checking: false, available: null });

  const [accompanyingGuests, setAccompanyingGuests] = useState<AccompanyingGuest[]>([]);
  const [newAccompanyingGuest, setNewAccompanyingGuest] = useState<AccompanyingGuest>({
    vorname: '',
    nachname: '',
    geburtsdatum: '',
  });

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
      });
      // Load accompanying guests, services and discounts if editing
      if (booking.id) {
        loadAccompanyingGuests(booking.id);
        loadAdditionalServices(booking.id);
        loadDiscounts(booking.id);
      }
    } else {
      setFormData({
        room_id: 0,
        guest_id: 0,
        checkin_date: '',
        checkout_date: '',
        anzahl_gaeste: 1,
        status: 'bestaetigt',
        bemerkungen: '',
      });
      setAccompanyingGuests([]);
      setAdditionalServices([]);
      setDiscounts([]);
    }
    setError(null);
    setPriceInfo(null);
    setAvailabilityStatus({ checking: false, available: null });
    setNewAccompanyingGuest({ vorname: '', nachname: '', geburtsdatum: '' });
    setNewService({ service_name: '', service_price: 0 });
    setNewDiscount({ discount_name: '', discount_type: 'percent', discount_value: 0 });
  }, [booking, isOpen]);

  // Calculate price when relevant fields change
  useEffect(() => {
    if (formData.room_id && formData.guest_id && formData.checkin_date && formData.checkout_date) {
      calculatePrice();
    }
  }, [formData.room_id, formData.guest_id, formData.checkin_date, formData.checkout_date, additionalServices, discounts]);

  // Check room availability when room or dates change
  useEffect(() => {
    if (formData.room_id && formData.checkin_date && formData.checkout_date) {
      checkAvailability();
    } else {
      setAvailabilityStatus({ checking: false, available: null });
    }
  }, [formData.room_id, formData.checkin_date, formData.checkout_date]);

  const loadGuestsAndRooms = async () => {
    try {
      const [guestsData, roomsData] = await Promise.all([
        invoke<Guest[]>('get_all_guests_command'),
        invoke<Room[]>('get_all_rooms'),
      ]);
      setGuests(guestsData);
      setRooms(roomsData);
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
    };
    setAdditionalServices([...additionalServices, newService]);
  };

  const addDiscountFromTemplate = (template: any) => {
    const newDiscount: Discount = {
      discount_name: template.name,
      discount_type: template.discount_type,
      discount_value: template.discount_value,
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
      const guest = guests.find(g => g.id === formData.guest_id);
      const room = rooms.find(r => r.id === formData.room_id);

      if (!guest || !room) return;

      const nights = await invoke<number>('calculate_nights_command', {
        checkin: formData.checkin_date,
        checkout: formData.checkout_date,
      });

      const pricePerNight = guest.dpolg_mitglied ? room.price_member : room.price_non_member;
      const basePrice = nights * pricePerNight;

      // Calculate services total
      const servicesTotal = additionalServices.reduce((sum, service) => sum + service.service_price, 0);

      // Calculate discounts total
      let discountsTotal = 0;
      const subtotal = basePrice + servicesTotal;

      for (const discount of discounts) {
        if (discount.discount_type === 'percent') {
          discountsTotal += subtotal * (discount.discount_value / 100);
        } else {
          discountsTotal += discount.discount_value;
        }
      }

      // Ensure discounts don't exceed subtotal
      discountsTotal = Math.min(discountsTotal, subtotal);

      const totalPrice = Math.max(0, subtotal - discountsTotal);

      setPriceInfo({
        nights,
        pricePerNight,
        basePrice,
        servicesTotal,
        discountsTotal,
        totalPrice,
        memberPrice: guest.dpolg_mitglied,
      });
    } catch (err) {
      console.error('Fehler bei Preisberechnung:', err);
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

  const handleAddAccompanyingGuest = () => {
    if (!newAccompanyingGuest.vorname || !newAccompanyingGuest.nachname) {
      setError('Bitte Vorname und Nachname für die Begleitperson eingeben');
      return;
    }

    // If editing existing booking, save to database immediately
    if (booking?.id) {
      saveAccompanyingGuest();
    } else {
      // If creating new booking, just add to temporary list
      setAccompanyingGuests([...accompanyingGuests, { ...newAccompanyingGuest }]);
      setNewAccompanyingGuest({ vorname: '', nachname: '', geburtsdatum: '' });
      setError(null);
    }
  };

  const saveAccompanyingGuest = async () => {
    if (!booking?.id) return;

    try {
      await invoke('add_accompanying_guest_command', {
        bookingId: booking.id,
        vorname: newAccompanyingGuest.vorname,
        nachname: newAccompanyingGuest.nachname,
        geburtsdatum: newAccompanyingGuest.geburtsdatum || null,
      });
      await loadAccompanyingGuests(booking.id);
      setNewAccompanyingGuest({ vorname: '', nachname: '', geburtsdatum: '' });
      setError(null);
    } catch (err) {
      console.error('Fehler beim Hinzufügen der Begleitperson:', err);
      setError('Fehler beim Hinzufügen der Begleitperson');
    }
  };

  const handleRemoveAccompanyingGuest = async (index: number, guestId?: number) => {
    if (guestId && booking?.id) {
      // Delete from database
      try {
        await invoke('delete_accompanying_guest_command', {
          guestId,
        });
        await loadAccompanyingGuests(booking.id);
      } catch (err) {
        console.error('Fehler beim Löschen der Begleitperson:', err);
        setError('Fehler beim Löschen der Begleitperson');
      }
    } else {
      // Remove from temporary list
      setAccompanyingGuests(accompanyingGuests.filter((_, i) => i !== index));
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
        };

        await updateBooking(booking.id, updatePayload);
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
            });
          }
        }

        // Save additional services if any
        if (additionalServices.length > 0 && result.id) {
          for (const service of additionalServices) {
            await invoke('add_service_command', {
              bookingId: result.id,
              serviceName: service.service_name,
              servicePrice: service.service_price,
            });
          }
        }

        // Save discounts if any
        if (discounts.length > 0 && result.id) {
          for (const discount of discounts) {
            await invoke('add_discount_command', {
              bookingId: result.id,
              discountName: discount.discount_name,
              discountType: discount.discount_type,
              discountValue: discount.discount_value,
            });
          }
        }

        // 🆕 Email-Auswahl-Dialog öffnen
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
                  value={formData.anzahl_gaeste}
                  onChange={(e) => setFormData({ ...formData, anzahl_gaeste: parseInt(e.target.value) || 1 })}
                  className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
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
                  className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  {STATUS_OPTIONS.map((status) => (
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
                    <span className="font-semibold text-blue-900">{priceInfo.nights}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-blue-700">Preis pro Nacht {priceInfo.memberPrice && '(Mitglied)'}:</span>
                    <span className="font-semibold text-blue-900">{priceInfo.pricePerNight.toFixed(2)} €</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-blue-700">Grundpreis:</span>
                    <span className="font-semibold text-blue-900">{priceInfo.basePrice.toFixed(2)} €</span>
                  </div>
                  {priceInfo.servicesTotal > 0 && (
                    <div className="flex justify-between">
                      <span className="text-emerald-700">+ Services:</span>
                      <span className="font-semibold text-emerald-700">{priceInfo.servicesTotal.toFixed(2)} €</span>
                    </div>
                  )}
                  {priceInfo.discountsTotal > 0 && (
                    <div className="flex justify-between">
                      <span className="text-orange-700">- Rabatte:</span>
                      <span className="font-semibold text-orange-700">{priceInfo.discountsTotal.toFixed(2)} €</span>
                    </div>
                  )}
                  <div className="border-t border-blue-300 pt-2 mt-2 flex justify-between">
                    <span className="font-bold text-blue-900">Gesamtpreis:</span>
                    <span className="font-bold text-blue-900 text-lg">{priceInfo.totalPrice.toFixed(2)} €</span>
                  </div>
                </div>
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

            {/* Accompanying Guests */}
            <div className="border border-slate-200 rounded-lg p-4">
              <h3 className="flex items-center gap-2 text-sm font-semibold text-slate-700 mb-4">
                <UserCheck className="w-4 h-4" />
                Begleitpersonen
              </h3>

              {/* List of Accompanying Guests */}
              {accompanyingGuests.length > 0 && (
                <div className="mb-4 space-y-2">
                  {accompanyingGuests.map((guest, index) => (
                    <div
                      key={guest.id || index}
                      className="flex items-center justify-between bg-slate-50 px-4 py-2 rounded-lg border border-slate-200"
                    >
                      <div>
                        <span className="text-sm font-semibold text-slate-900">
                          {guest.vorname} {guest.nachname}
                        </span>
                        {guest.geburtsdatum && (
                          <span className="text-xs text-slate-500 ml-2">
                            (Geburtsdatum: {guest.geburtsdatum})
                          </span>
                        )}
                      </div>
                      <button
                        type="button"
                        onClick={() => handleRemoveAccompanyingGuest(index, guest.id)}
                        className="text-red-600 hover:bg-red-50 p-1 rounded transition-colors"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  ))}
                </div>
              )}

              {/* Add New Accompanying Guest */}
              <div className="space-y-3">
                <div className="grid grid-cols-3 gap-3">
                  <input
                    type="text"
                    placeholder="Vorname *"
                    value={newAccompanyingGuest.vorname}
                    onChange={(e) => setNewAccompanyingGuest({ ...newAccompanyingGuest, vorname: e.target.value })}
                    className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                  />
                  <input
                    type="text"
                    placeholder="Nachname *"
                    value={newAccompanyingGuest.nachname}
                    onChange={(e) => setNewAccompanyingGuest({ ...newAccompanyingGuest, nachname: e.target.value })}
                    className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                  />
                  <input
                    type="date"
                    placeholder="Geburtsdatum"
                    value={newAccompanyingGuest.geburtsdatum}
                    onChange={(e) => setNewAccompanyingGuest({ ...newAccompanyingGuest, geburtsdatum: e.target.value })}
                    className="px-3 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                  />
                </div>
                <button
                  type="button"
                  onClick={handleAddAccompanyingGuest}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg transition-colors text-sm font-semibold"
                >
                  <Plus className="w-4 h-4" />
                  Begleitperson hinzufügen
                </button>
              </div>
            </div>

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
      />
    )}
    </>
  );
}
