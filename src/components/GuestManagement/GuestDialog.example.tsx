// ============================================================================
// GuestDialog Usage Example
// ============================================================================
// This file demonstrates how to integrate the GuestDialog component
// into your application.

import { useState } from 'react';
import GuestDialog from './GuestDialog';
import { UserPlus } from 'lucide-react';

function GuestManagementPage() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [guestToEdit, setGuestToEdit] = useState<any>(null);

  // Example: Create new guest
  const handleCreateGuest = () => {
    setGuestToEdit(null); // No guest to edit = create mode
    setIsDialogOpen(true);
  };

  // Example: Edit existing guest
  const handleEditGuest = (guest: any) => {
    setGuestToEdit({
      id: guest.id,
      vorname: guest.vorname,
      nachname: guest.nachname,
      email: guest.email,
      telefon: guest.telefon,
      dpolgMitglied: guest.dpolg_mitglied, // Note: converting snake_case to camelCase
      strasse: guest.strasse,
      plz: guest.plz,
      ort: guest.ort,
      mitgliedsnummer: guest.mitgliedsnummer,
      notizen: guest.notizen,
    });
    setIsDialogOpen(true);
  };

  // Callback: Refresh data after successful save
  const handleSuccess = () => {
    console.log('Guest saved successfully!');
    // Reload your guest list here
    // e.g., fetchGuests();
  };

  // Callback: Close dialog
  const handleClose = () => {
    setIsDialogOpen(false);
    setGuestToEdit(null);
  };

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-white mb-6">Gästeverwaltung</h1>

      {/* Create Guest Button */}
      <button
        onClick={handleCreateGuest}
        className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white rounded-lg font-medium transition-all shadow-lg hover:shadow-xl"
      >
        <UserPlus className="w-5 h-5" />
        Neuer Gast
      </button>

      {/* Guest Dialog Component */}
      <GuestDialog
        isOpen={isDialogOpen}
        onClose={handleClose}
        onSuccess={handleSuccess}
        guestToEdit={guestToEdit}
      />

      {/* Your guest list table/grid here */}
    </div>
  );
}

export default GuestManagementPage;

// ============================================================================
// Integration with existing data
// ============================================================================

// Example: Converting backend guest data (snake_case) to dialog props (camelCase)
function convertBackendGuestToDialogProps(backendGuest: any) {
  return {
    id: backendGuest.id,
    vorname: backendGuest.vorname,
    nachname: backendGuest.nachname,
    email: backendGuest.email,
    telefon: backendGuest.telefon,
    dpolgMitglied: backendGuest.dpolg_mitglied, // snake_case -> camelCase
    strasse: backendGuest.strasse,
    plz: backendGuest.plz,
    ort: backendGuest.ort,
    mitgliedsnummer: backendGuest.mitgliedsnummer,
    notizen: backendGuest.notizen,
  };
}

// ============================================================================
// Tauri Commands Used by GuestDialog
// ============================================================================

/*
The GuestDialog component calls these Tauri commands:

1. validate_email_command(email: string)
   - Validates email format using backend regex
   - Returns: Result<String, String>
   - Called on email field blur and before submit

2. create_guest_command(...)
   - Creates a new guest in the database
   - Parameters:
     - vorname: string
     - nachname: string
     - email: string
     - telefon: string
     - dpolgMitglied: boolean (Note: camelCase in TypeScript, snake_case in Rust)
     - strasse: string | null
     - plz: string | null
     - ort: string | null
     - mitgliedsnummer: string | null
     - notizen: string | null
   - Returns: Result<Guest, String>

3. update_guest_command(id, ...)
   - Updates an existing guest
   - Same parameters as create_guest_command + id
   - Returns: Result<Guest, String>

IMPORTANT: The Rust backend uses snake_case (dpolg_mitglied) but TypeScript
uses camelCase (dpolgMitglied). The Tauri invoke handles this conversion
automatically when using the parameter name directly in the object.
*/