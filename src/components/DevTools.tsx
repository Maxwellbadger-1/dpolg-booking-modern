import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Play, CheckCircle, XCircle } from 'lucide-react';

interface TestResult {
  test: string;
  status: 'success' | 'error';
  message: string;
  data?: any;
}

export default function DevTools() {
  const [isOpen, setIsOpen] = useState(true);
  const [results, setResults] = useState<TestResult[]>([]);
  const [running, setRunning] = useState(false);

  const addResult = (test: string, status: 'success' | 'error', message: string, data?: any) => {
    setResults(prev => [...prev, { test, status, message, data }]);
  };

  const clearResults = () => setResults([]);

  // ==================== GUEST TESTS ====================

  const testCreateGuest = async () => {
    try {
      const guest = await invoke('create_guest_command', {
        vorname: 'Test',
        nachname: 'User',
        email: 'test@example.com',
        telefon: '+49 123 456789',
        dpolgMitglied: true,
        strasse: 'Teststraße 1',
        plz: '12345',
        ort: 'Berlin',
        mitgliedsnummer: 'M123',
        notizen: 'Test-Gast erstellt via DevTools',
      });
      addResult('Create Guest', 'success', 'Gast erfolgreich erstellt', guest);
      return guest;
    } catch (error) {
      addResult('Create Guest', 'error', String(error));
    }
  };

  const testGetAllGuests = async () => {
    try {
      const guests = await invoke('get_all_guests_command');
      addResult('Get All Guests', 'success', `${Array.isArray(guests) ? guests.length : 0} Gäste gefunden`, guests);
    } catch (error) {
      addResult('Get All Guests', 'error', String(error));
    }
  };

  const testSearchGuests = async () => {
    try {
      const guests = await invoke('search_guests_command', {
        query: 'Test'
      });
      addResult('Search Guests', 'success', `${Array.isArray(guests) ? guests.length : 0} Gäste gefunden`, guests);
    } catch (error) {
      addResult('Search Guests', 'error', String(error));
    }
  };

  const testUpdateGuest = async (guestId: number) => {
    try {
      const guest = await invoke('update_guest_command', {
        id: guestId,
        vorname: 'Updated',
        nachname: 'User',
        email: 'updated@example.com',
        telefon: '+49 987 654321',
        dpolgMitglied: true,
        strasse: 'Neue Straße 2',
        plz: '54321',
        ort: 'München',
        mitgliedsnummer: 'M456',
        notizen: 'Aktualisiert',
      });
      addResult('Update Guest', 'success', 'Gast aktualisiert', guest);
    } catch (error) {
      addResult('Update Guest', 'error', String(error));
    }
  };

  const testDeleteGuest = async (guestId: number) => {
    try {
      await invoke('delete_guest_command', { id: guestId });
      addResult('Delete Guest', 'success', `Gast #${guestId} gelöscht`);
    } catch (error) {
      addResult('Delete Guest', 'error', String(error));
    }
  };

  // ==================== ROOM TESTS ====================

  const testCreateRoom = async () => {
    try {
      const room = await invoke('create_room_command', {
        name: 'Test Zimmer ' + Date.now(),
        gebaeudeTyp: 'Testgebäude',
        capacity: 2,
        priceMember: 45.0,
        priceNonMember: 65.0,
        ort: 'Berlin',
        schluesselcode: '9999',
      });
      addResult('Create Room', 'success', 'Zimmer erstellt', room);
      return room;
    } catch (error) {
      addResult('Create Room', 'error', String(error));
    }
  };

  const testUpdateRoom = async (roomId: number) => {
    try {
      const room = await invoke('update_room_command', {
        id: roomId,
        name: 'Updated Room',
        gebaeudeTyp: 'Hauptgebäude',
        capacity: 3,
        priceMember: 50.0,
        priceNonMember: 70.0,
        ort: 'München',
        schluesselcode: '8888',
      });
      addResult('Update Room', 'success', 'Zimmer aktualisiert', room);
    } catch (error) {
      addResult('Update Room', 'error', String(error));
    }
  };

  const testDeleteRoom = async (roomId: number) => {
    try {
      await invoke('delete_room_command', { id: roomId });
      addResult('Delete Room', 'success', `Zimmer #${roomId} gelöscht`);
    } catch (error) {
      addResult('Delete Room', 'error', String(error));
    }
  };

  // ==================== BOOKING TESTS ====================

  const testCreateBooking = async (roomId: number, guestId: number) => {
    try {
      const booking = await invoke('create_booking_command', {
        roomId: roomId,
        guestId: guestId,
        reservierungsnummer: 'TEST-' + Date.now(),
        checkinDate: '2025-12-01',
        checkoutDate: '2025-12-05',
        anzahlGaeste: 2,
        status: 'reserviert',
        gesamtpreis: 200.0,
        bemerkungen: 'Test-Buchung via DevTools',
        anzahlBegleitpersonen: 1,
        grundpreis: 180.0,
        servicesPreis: 20.0,
        rabattPreis: 0.0,
        anzahlNaechte: 4,
      });
      addResult('Create Booking', 'success', 'Buchung erstellt', booking);
      return booking;
    } catch (error) {
      addResult('Create Booking', 'error', String(error));
    }
  };

  const testCancelBooking = async (bookingId: number) => {
    try {
      const booking = await invoke('cancel_booking_command', { id: bookingId });
      addResult('Cancel Booking', 'success', 'Buchung storniert', booking);
    } catch (error) {
      addResult('Cancel Booking', 'error', String(error));
    }
  };

  // ==================== SERVICES TESTS ====================

  const testAddService = async (bookingId: number) => {
    try {
      const service = await invoke('add_service_command', {
        bookingId: bookingId,
        serviceName: 'Frühstück',
        servicePrice: 10.0,
      });
      addResult('Add Service', 'success', 'Service hinzugefügt', service);
    } catch (error) {
      addResult('Add Service', 'error', String(error));
    }
  };

  const testGetBookingServices = async (bookingId: number) => {
    try {
      const services = await invoke('get_booking_services_command', { bookingId: bookingId });
      addResult('Get Services', 'success', `${Array.isArray(services) ? services.length : 0} Services gefunden`, services);
    } catch (error) {
      addResult('Get Services', 'error', String(error));
    }
  };

  // ==================== VALIDATION TESTS ====================

  const testValidateEmail = async () => {
    try {
      const result = await invoke('validate_email_command', {
        email: 'test@example.com',
      });
      addResult('Validate Email', 'success', String(result));
    } catch (error) {
      addResult('Validate Email', 'error', String(error));
    }
  };

  const testValidateDateRange = async () => {
    try {
      const result = await invoke('validate_date_range_command', {
        checkin: '2025-10-01',
        checkout: '2025-10-05',
      });
      addResult('Validate Date Range', 'success', String(result));
    } catch (error) {
      addResult('Validate Date Range', 'error', String(error));
    }
  };

  const testCheckAvailability = async (roomId: number) => {
    try {
      const available = await invoke('check_room_availability_command', {
        roomId: roomId,
        checkin: '2025-12-01',
        checkout: '2025-12-05',
      });
      addResult('Check Availability', 'success', `Zimmer ${available ? 'verfügbar' : 'nicht verfügbar'}`);
    } catch (error) {
      addResult('Check Availability', 'error', String(error));
    }
  };

  // ==================== PRICING TESTS ====================

  const testCalculateNights = async () => {
    try {
      const nights = await invoke('calculate_nights_command', {
        checkin: '2025-10-01',
        checkout: '2025-10-05',
      });
      addResult('Calculate Nights', 'success', `${nights} Nächte`, nights);
    } catch (error) {
      addResult('Calculate Nights', 'error', String(error));
    }
  };

  const testCalculatePrice = async (roomId: number) => {
    try {
      const price = await invoke('calculate_booking_price_command', {
        roomId: roomId,
        checkin: '2025-10-01',
        checkout: '2025-10-05',
        isMember: true,
      });
      addResult('Calculate Price', 'success', 'Preisberechnung erfolgreich', price);
    } catch (error) {
      addResult('Calculate Price', 'error', String(error));
    }
  };

  // ==================== COMPLETE TEST SUITE ====================

  const runCompleteTest = async () => {
    setRunning(true);
    clearResults();

    // 1. Create Guest
    const guest: any = await testCreateGuest();
    if (!guest) {
      setRunning(false);
      return;
    }

    // 2. Get All Guests
    await testGetAllGuests();

    // 3. Search Guests
    await testSearchGuests();

    // 4. Create Room
    const room: any = await testCreateRoom();
    if (!room) {
      setRunning(false);
      return;
    }

    // 5. Create Booking
    const booking: any = await testCreateBooking(room.id, guest.id);
    if (!booking) {
      setRunning(false);
      return;
    }

    // 6. Add Service
    await testAddService(booking.id);

    // 7. Get Services
    await testGetBookingServices(booking.id);

    // 8. Cancel Booking
    await testCancelBooking(booking.id);

    // 9. Update Guest
    await testUpdateGuest(guest.id);

    // 10. Validation Tests
    await testValidateEmail();
    await testValidateDateRange();
    await testCheckAvailability(room.id);

    // 11. Pricing Tests
    await testCalculateNights();
    await testCalculatePrice(room.id);

    // 12. Cleanup: Delete (optional - kann FK constraint errors geben)
    // await testDeleteRoom(room.id);
    // await testDeleteGuest(guest.id);

    setRunning(false);
    addResult('Complete Test', 'success', '✅ Alle Tests durchgelaufen!');
  };

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="fixed bottom-4 right-4 px-4 py-2 bg-yellow-500 text-black font-bold rounded-lg shadow-xl hover:bg-yellow-400 transition-colors z-50"
      >
        🧪 DevTools
      </button>
    );
  }

  return (
    <div className="fixed bottom-0 right-0 w-[600px] h-[500px] bg-gradient-to-br from-yellow-50 to-yellow-100 border-4 border-yellow-500 rounded-tl-2xl shadow-2xl z-50 flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 bg-yellow-500 text-black border-b-2 border-yellow-600">
        <div className="flex items-center gap-2">
          <span className="text-2xl">🧪</span>
          <h2 className="text-lg font-bold">DevTools - Backend Command Tests</h2>
        </div>
        <button
          onClick={() => setIsOpen(false)}
          className="p-1 hover:bg-yellow-600 rounded transition-colors"
        >
          <X className="w-5 h-5" />
        </button>
      </div>

      {/* Controls */}
      <div className="p-4 bg-yellow-200 border-b-2 border-yellow-400 flex gap-2 flex-wrap">
        <button
          onClick={runCompleteTest}
          disabled={running}
          className="flex items-center gap-2 px-3 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white font-semibold rounded-lg transition-colors"
        >
          <Play className="w-4 h-4" />
          {running ? 'Läuft...' : 'Alle Tests'}
        </button>

        <button
          onClick={testGetAllGuests}
          className="px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors"
        >
          Get Guests
        </button>

        <button
          onClick={testCreateGuest}
          className="px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors"
        >
          Create Guest
        </button>

        <button
          onClick={testCreateRoom}
          className="px-3 py-2 bg-purple-600 hover:bg-purple-700 text-white font-semibold rounded-lg transition-colors"
        >
          Create Room
        </button>

        <button
          onClick={testValidateEmail}
          className="px-3 py-2 bg-orange-600 hover:bg-orange-700 text-white font-semibold rounded-lg transition-colors"
        >
          Validate Email
        </button>

        <button
          onClick={testCalculateNights}
          className="px-3 py-2 bg-teal-600 hover:bg-teal-700 text-white font-semibold rounded-lg transition-colors"
        >
          Calculate Nights
        </button>

        <button
          onClick={clearResults}
          className="px-3 py-2 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors"
        >
          Clear
        </button>
      </div>

      {/* Results */}
      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {results.length === 0 && (
          <div className="text-center text-gray-500 mt-8">
            Keine Tests gelaufen. Klicke auf "Alle Tests" um zu starten.
          </div>
        )}

        {results.map((result, index) => (
          <div
            key={index}
            className={`p-3 rounded-lg border-2 ${
              result.status === 'success'
                ? 'bg-green-50 border-green-500'
                : 'bg-red-50 border-red-500'
            }`}
          >
            <div className="flex items-start gap-2">
              {result.status === 'success' ? (
                <CheckCircle className="w-5 h-5 text-green-600 flex-shrink-0 mt-0.5" />
              ) : (
                <XCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
              )}
              <div className="flex-1">
                <div className="font-bold text-sm">{result.test}</div>
                <div className="text-sm text-gray-700">{result.message}</div>
                {result.data && (
                  <details className="mt-2">
                    <summary className="text-xs text-gray-600 cursor-pointer hover:text-gray-800">
                      Daten anzeigen
                    </summary>
                    <pre className="mt-1 text-xs bg-white p-2 rounded overflow-x-auto">
                      {JSON.stringify(result.data, null, 2)}
                    </pre>
                  </details>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}