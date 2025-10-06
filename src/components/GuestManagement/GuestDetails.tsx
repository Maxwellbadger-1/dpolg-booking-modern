import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  X, User, Mail, Phone, MapPin, Calendar, CreditCard,
  FileText, Edit2, TrendingUp, Clock, Euro, ChevronDown, ChevronUp,
  Users as UsersIcon, ShoppingBag, Tag, CheckCircle, AlertCircle
} from 'lucide-react';
import { format } from 'date-fns';
import { de } from 'date-fns/locale';

interface GuestDetailsProps {
  guestId: number;
  isOpen: boolean;
  onClose: () => void;
  onEdit?: () => void;
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
  created_at: string;
}

interface Room {
  id: number;
  name: string;
  gebaeude_typ: string;
  ort: string;
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
  bezahlt_am?: string;
  zahlungsmethode?: string;
  room: Room;
}

interface AdditionalService {
  id: number;
  service_name: string;
  service_price: number;
}

interface AccompanyingGuest {
  id: number;
  vorname: string;
  nachname: string;
  geburtsdatum?: string;
}

interface Discount {
  id: number;
  discount_name: string;
  discount_type: string;
  discount_value: number;
}

export default function GuestDetails({ guestId, isOpen, onClose, onEdit }: GuestDetailsProps) {
  const [guest, setGuest] = useState<Guest | null>(null);
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [loading, setLoading] = useState(true);
  const [expandedBookings, setExpandedBookings] = useState<Set<number>>(new Set());
  const [bookingDetails, setBookingDetails] = useState<{
    [key: number]: {
      services: AdditionalService[];
      accompanyingGuests: AccompanyingGuest[];
      discounts: Discount[];
    };
  }>({});

  useEffect(() => {
    if (isOpen && guestId) {
      loadGuestDetails();
    }
  }, [isOpen, guestId]);

  const loadGuestDetails = async () => {
    try {
      setLoading(true);

      // Load guest
      const guestData = await invoke<Guest>('get_guest_by_id_command', { id: guestId });
      setGuest(guestData);

      // Load guest's bookings
      const allBookings = await invoke<Booking[]>('get_all_bookings');
      const guestBookings = allBookings.filter(b => b.guest_id === guestId);
      setBookings(guestBookings);
    } catch (error) {
      console.error('Fehler beim Laden der Gastdetails:', error);
    } finally {
      setLoading(false);
    }
  };

  const toggleBookingDetails = async (bookingId: number) => {
    const newExpanded = new Set(expandedBookings);

    if (newExpanded.has(bookingId)) {
      // Collapse
      newExpanded.delete(bookingId);
    } else {
      // Expand - load details if not loaded yet
      newExpanded.add(bookingId);

      if (!bookingDetails[bookingId]) {
        try {
          const [services, accompanyingGuests, discounts] = await Promise.all([
            invoke<AdditionalService[]>('get_booking_services_command', { bookingId }),
            invoke<AccompanyingGuest[]>('get_booking_accompanying_guests_command', { bookingId }),
            invoke<Discount[]>('get_booking_discounts_command', { bookingId }),
          ]);

          setBookingDetails(prev => ({
            ...prev,
            [bookingId]: { services, accompanyingGuests, discounts },
          }));
        } catch (error) {
          console.error('Fehler beim Laden der Buchungsdetails:', error);
        }
      }
    }

    setExpandedBookings(newExpanded);
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
      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-semibold border ${styles[status as keyof typeof styles] || styles.reserviert}`}>
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </span>
    );
  };

  // Calculate stats
  const totalBookings = bookings.length;
  const completedBookings = bookings.filter(b => b.status === 'ausgecheckt').length;
  const activeBookings = bookings.filter(b => ['reserviert', 'bestaetigt', 'eingecheckt'].includes(b.status)).length;
  const cancelledBookings = bookings.filter(b => b.status === 'storniert').length;
  const totalRevenue = bookings
    .filter(b => b.status !== 'storniert')
    .reduce((sum, b) => sum + b.gesamtpreis, 0);
  const lastBooking = bookings.length > 0
    ? bookings.sort((a, b) => new Date(b.checkin_date).getTime() - new Date(a.checkin_date).getTime())[0]
    : null;

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-2xl shadow-2xl w-full max-w-5xl max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-emerald-500 to-emerald-600 px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="bg-white/20 p-2 rounded-lg">
              <User className="w-6 h-6 text-white" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">Gastdetails</h2>
              {guest && (
                <p className="text-sm text-emerald-100">
                  {guest.vorname} {guest.nachname}
                </p>
              )}
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
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <div className="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-emerald-600"></div>
            </div>
          ) : guest ? (
            <div className="space-y-6">
              {/* Guest Information */}
              <div className="border border-slate-200 rounded-lg p-5">
                <div className="flex items-start justify-between mb-4">
                  <h3 className="text-lg font-bold text-slate-800">Persönliche Informationen</h3>
                  <button
                    onClick={onEdit}
                    className="flex items-center gap-2 px-4 py-2 bg-emerald-100 hover:bg-emerald-200 text-emerald-700 rounded-lg font-semibold transition-colors"
                  >
                    <Edit2 className="w-4 h-4" />
                    Bearbeiten
                  </button>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Vorname</p>
                    <p className="font-semibold text-slate-900">{guest.vorname}</p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1">Nachname</p>
                    <p className="font-semibold text-slate-900">{guest.nachname}</p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                      <Mail className="w-3.5 h-3.5" />
                      E-Mail
                    </p>
                    <p className="font-medium text-slate-900">{guest.email}</p>
                  </div>
                  {guest.telefon && (
                    <div>
                      <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                        <Phone className="w-3.5 h-3.5" />
                        Telefon
                      </p>
                      <p className="font-medium text-slate-900">{guest.telefon}</p>
                    </div>
                  )}
                  {(guest.strasse || guest.plz || guest.ort) && (
                    <div className="col-span-2">
                      <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                        <MapPin className="w-3.5 h-3.5" />
                        Adresse
                      </p>
                      <p className="font-medium text-slate-900">
                        {guest.strasse && <>{guest.strasse}<br /></>}
                        {guest.plz} {guest.ort}
                      </p>
                    </div>
                  )}
                  <div>
                    <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                      <CreditCard className="w-3.5 h-3.5" />
                      Mitgliedschaft
                    </p>
                    <p className="font-semibold text-slate-900">
                      {guest.dpolg_mitglied ? (
                        <span className="text-emerald-600">
                          DPolG Stiftung Mitglied {guest.mitgliedsnummer && `(${guest.mitgliedsnummer})`}
                        </span>
                      ) : (
                        <span className="text-slate-500">Kein Mitglied</span>
                      )}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                      <Calendar className="w-3.5 h-3.5" />
                      Erstellt am
                    </p>
                    <p className="font-medium text-slate-900">
                      {format(new Date(guest.created_at), 'dd. MMMM yyyy', { locale: de })}
                    </p>
                  </div>
                  {guest.notizen && (
                    <div className="col-span-2">
                      <p className="text-sm text-slate-600 mb-1 flex items-center gap-1">
                        <FileText className="w-3.5 h-3.5" />
                        Notizen
                      </p>
                      <p className="font-medium text-slate-700 whitespace-pre-wrap bg-slate-50 p-3 rounded-lg">
                        {guest.notizen}
                      </p>
                    </div>
                  )}
                </div>
              </div>

              {/* Statistics */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="border border-slate-200 rounded-lg p-4 bg-gradient-to-br from-blue-50 to-blue-100">
                  <div className="flex items-center gap-2 text-blue-700 mb-2">
                    <Calendar className="w-5 h-5" />
                    <p className="text-sm font-semibold">Gesamt</p>
                  </div>
                  <p className="text-2xl font-bold text-blue-900">{totalBookings}</p>
                  <p className="text-xs text-blue-700 mt-1">Buchungen</p>
                </div>

                <div className="border border-slate-200 rounded-lg p-4 bg-gradient-to-br from-emerald-50 to-emerald-100">
                  <div className="flex items-center gap-2 text-emerald-700 mb-2">
                    <Clock className="w-5 h-5" />
                    <p className="text-sm font-semibold">Aktiv</p>
                  </div>
                  <p className="text-2xl font-bold text-emerald-900">{activeBookings}</p>
                  <p className="text-xs text-emerald-700 mt-1">Buchungen</p>
                </div>

                <div className="border border-slate-200 rounded-lg p-4 bg-gradient-to-br from-slate-50 to-slate-100">
                  <div className="flex items-center gap-2 text-slate-700 mb-2">
                    <TrendingUp className="w-5 h-5" />
                    <p className="text-sm font-semibold">Abgeschlossen</p>
                  </div>
                  <p className="text-2xl font-bold text-slate-900">{completedBookings}</p>
                  <p className="text-xs text-slate-700 mt-1">Buchungen</p>
                </div>

                <div className="border border-slate-200 rounded-lg p-4 bg-gradient-to-br from-purple-50 to-purple-100">
                  <div className="flex items-center gap-2 text-purple-700 mb-2">
                    <Euro className="w-5 h-5" />
                    <p className="text-sm font-semibold">Umsatz</p>
                  </div>
                  <p className="text-2xl font-bold text-purple-900">{totalRevenue.toFixed(0)} €</p>
                  <p className="text-xs text-purple-700 mt-1">Gesamt</p>
                </div>
              </div>

              {/* Last Booking */}
              {lastBooking && (
                <div className="border border-slate-200 rounded-lg p-5">
                  <h3 className="text-lg font-bold text-slate-800 mb-3">Letzte Buchung</h3>
                  <div className="grid grid-cols-2 gap-4 bg-slate-50 p-4 rounded-lg">
                    <div>
                      <p className="text-sm text-slate-600 mb-1">Reservierungsnummer</p>
                      <p className="font-semibold text-slate-900">{lastBooking.reservierungsnummer}</p>
                    </div>
                    <div>
                      <p className="text-sm text-slate-600 mb-1">Status</p>
                      {getStatusBadge(lastBooking.status)}
                    </div>
                    <div>
                      <p className="text-sm text-slate-600 mb-1">Zimmer</p>
                      <p className="font-medium text-slate-900">{lastBooking.room.name}</p>
                    </div>
                    <div>
                      <p className="text-sm text-slate-600 mb-1">Zeitraum</p>
                      <p className="font-medium text-slate-900">
                        {format(new Date(lastBooking.checkin_date), 'dd.MM.yy', { locale: de })} -{' '}
                        {format(new Date(lastBooking.checkout_date), 'dd.MM.yy', { locale: de })}
                      </p>
                    </div>
                  </div>
                </div>
              )}

              {/* Booking History */}
              <div className="border border-slate-200 rounded-lg p-5">
                <h3 className="text-lg font-bold text-slate-800 mb-4">
                  Buchungshistorie ({bookings.length})
                </h3>
                {bookings.length === 0 ? (
                  <div className="text-center py-8 text-slate-600">
                    <Calendar className="w-12 h-12 text-slate-300 mx-auto mb-3" />
                    <p>Noch keine Buchungen vorhanden</p>
                  </div>
                ) : (
                  <div className="space-y-3 max-h-96 overflow-y-auto">
                    {bookings
                      .sort((a, b) => new Date(b.checkin_date).getTime() - new Date(a.checkin_date).getTime())
                      .map((booking) => {
                        const isExpanded = expandedBookings.has(booking.id);
                        const details = bookingDetails[booking.id];

                        return (
                          <div
                            key={booking.id}
                            className="border border-slate-200 rounded-lg overflow-hidden bg-white"
                          >
                            {/* Card Header - Clickable */}
                            <button
                              onClick={() => toggleBookingDetails(booking.id)}
                              className="w-full flex items-center justify-between p-4 hover:bg-slate-50 transition-colors text-left"
                            >
                              <div className="flex-1">
                                <div className="flex items-center gap-3 mb-2">
                                  <p className="font-semibold text-slate-900">{booking.reservierungsnummer}</p>
                                  {getStatusBadge(booking.status)}
                                  {booking.bezahlt && (
                                    <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-semibold bg-green-100 text-green-700 border border-green-200">
                                      <CheckCircle className="w-3 h-3 mr-1" />
                                      Bezahlt
                                    </span>
                                  )}
                                </div>
                                <div className="flex items-center gap-4 text-sm text-slate-600">
                                  <span>{booking.room.name}</span>
                                  <span>•</span>
                                  <span>
                                    {format(new Date(booking.checkin_date), 'dd.MM.yyyy', { locale: de })} -{' '}
                                    {format(new Date(booking.checkout_date), 'dd.MM.yyyy', { locale: de })}
                                  </span>
                                  <span>•</span>
                                  <span>{booking.anzahl_naechte} Nächte</span>
                                </div>
                              </div>
                              <div className="flex items-center gap-3">
                                <div className="text-right">
                                  <p className="font-bold text-slate-900">{booking.gesamtpreis.toFixed(2)} €</p>
                                  <p className="text-xs text-slate-500">{booking.anzahl_gaeste} Gäste</p>
                                </div>
                                {isExpanded ? (
                                  <ChevronUp className="w-5 h-5 text-slate-400" />
                                ) : (
                                  <ChevronDown className="w-5 h-5 text-slate-400" />
                                )}
                              </div>
                            </button>

                            {/* Expandable Details */}
                            {isExpanded && (
                              <div
                                className="border-t border-slate-200 p-4 bg-slate-50"
                                style={{
                                  animation: 'slideDown 300ms ease-out',
                                }}
                              >
                                <div className="grid grid-cols-2 gap-4">
                                  {/* Price Breakdown */}
                                  <div className="col-span-2">
                                    <h4 className="text-sm font-semibold text-slate-700 mb-3 flex items-center gap-2">
                                      <Euro className="w-4 h-4" />
                                      Preisaufschlüsselung
                                    </h4>
                                    <div className="space-y-2 text-sm">
                                      <div className="flex justify-between">
                                        <span className="text-slate-600">Grundpreis:</span>
                                        <span className="font-medium text-slate-900">{booking.grundpreis.toFixed(2)} €</span>
                                      </div>
                                      {booking.services_preis > 0 && (
                                        <div className="flex justify-between">
                                          <span className="text-slate-600">+ Services:</span>
                                          <span className="font-medium text-emerald-600">{booking.services_preis.toFixed(2)} €</span>
                                        </div>
                                      )}
                                      {booking.rabatt_preis > 0 && (
                                        <div className="flex justify-between">
                                          <span className="text-slate-600">- Rabatte:</span>
                                          <span className="font-medium text-orange-600">{booking.rabatt_preis.toFixed(2)} €</span>
                                        </div>
                                      )}
                                      <div className="flex justify-between pt-2 border-t border-slate-200">
                                        <span className="font-semibold text-slate-900">Gesamt:</span>
                                        <span className="font-bold text-slate-900">{booking.gesamtpreis.toFixed(2)} €</span>
                                      </div>
                                    </div>
                                  </div>

                                  {/* Payment Status */}
                                  <div className="col-span-2">
                                    <h4 className="text-sm font-semibold text-slate-700 mb-2 flex items-center gap-2">
                                      <CreditCard className="w-4 h-4" />
                                      Zahlungsstatus
                                    </h4>
                                    <div className="flex items-center gap-2">
                                      {booking.bezahlt ? (
                                        <div className="flex items-center gap-2">
                                          <CheckCircle className="w-4 h-4 text-green-600" />
                                          <span className="text-sm text-slate-700">
                                            Bezahlt am {booking.bezahlt_am && format(new Date(booking.bezahlt_am), 'dd.MM.yyyy', { locale: de })}
                                            {booking.zahlungsmethode && ` via ${booking.zahlungsmethode}`}
                                          </span>
                                        </div>
                                      ) : (
                                        <div className="flex items-center gap-2">
                                          <AlertCircle className="w-4 h-4 text-orange-600" />
                                          <span className="text-sm text-slate-700">Ausstehend</span>
                                        </div>
                                      )}
                                    </div>
                                  </div>

                                  {/* Services */}
                                  {details?.services && details.services.length > 0 && (
                                    <div className="col-span-2">
                                      <h4 className="text-sm font-semibold text-slate-700 mb-2 flex items-center gap-2">
                                        <ShoppingBag className="w-4 h-4" />
                                        Zusätzliche Services
                                      </h4>
                                      <div className="space-y-1">
                                        {details.services.map((service) => (
                                          <div key={service.id} className="flex justify-between text-sm">
                                            <span className="text-slate-600">{service.service_name}</span>
                                            <span className="font-medium text-slate-900">{service.service_price.toFixed(2)} €</span>
                                          </div>
                                        ))}
                                      </div>
                                    </div>
                                  )}

                                  {/* Discounts */}
                                  {details?.discounts && details.discounts.length > 0 && (
                                    <div className="col-span-2">
                                      <h4 className="text-sm font-semibold text-slate-700 mb-2 flex items-center gap-2">
                                        <Tag className="w-4 h-4" />
                                        Rabatte
                                      </h4>
                                      <div className="space-y-1">
                                        {details.discounts.map((discount) => (
                                          <div key={discount.id} className="flex justify-between text-sm">
                                            <span className="text-slate-600">
                                              {discount.discount_name} ({discount.discount_type === 'percent' ? `${discount.discount_value}%` : 'Fix'})
                                            </span>
                                            <span className="font-medium text-orange-600">
                                              {discount.discount_type === 'percent'
                                                ? `${((booking.grundpreis + booking.services_preis) * discount.discount_value / 100).toFixed(2)} €`
                                                : `${discount.discount_value.toFixed(2)} €`
                                              }
                                            </span>
                                          </div>
                                        ))}
                                      </div>
                                    </div>
                                  )}

                                  {/* Accompanying Guests */}
                                  {details?.accompanyingGuests && details.accompanyingGuests.length > 0 && (
                                    <div className="col-span-2">
                                      <h4 className="text-sm font-semibold text-slate-700 mb-2 flex items-center gap-2">
                                        <UsersIcon className="w-4 h-4" />
                                        Begleitpersonen ({details.accompanyingGuests.length})
                                      </h4>
                                      <div className="space-y-1">
                                        {details.accompanyingGuests.map((guest) => (
                                          <div key={guest.id} className="text-sm text-slate-700">
                                            {guest.vorname} {guest.nachname}
                                            {guest.geburtsdatum && (
                                              <span className="text-slate-500 ml-2">
                                                ({format(new Date(guest.geburtsdatum), 'dd.MM.yyyy', { locale: de })})
                                              </span>
                                            )}
                                          </div>
                                        ))}
                                      </div>
                                    </div>
                                  )}

                                  {/* Notes */}
                                  {booking.bemerkungen && (
                                    <div className="col-span-2">
                                      <h4 className="text-sm font-semibold text-slate-700 mb-2 flex items-center gap-2">
                                        <FileText className="w-4 h-4" />
                                        Bemerkungen
                                      </h4>
                                      <p className="text-sm text-slate-700 bg-white p-3 rounded-lg border border-slate-200">
                                        {booking.bemerkungen}
                                      </p>
                                    </div>
                                  )}
                                </div>
                              </div>
                            )}
                          </div>
                        );
                      })}
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="text-center py-12">
              <p className="text-slate-600">Gast konnte nicht geladen werden.</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-end">
          <button
            onClick={onClose}
            className="px-6 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-lg font-semibold transition-colors"
          >
            Schließen
          </button>
        </div>
      </div>
    </div>
  );
}
