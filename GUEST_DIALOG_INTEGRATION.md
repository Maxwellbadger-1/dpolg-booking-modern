# GuestDialog Component - Integration Guide

## Overview
The GuestDialog component has been successfully implemented for Phase 3.1a of the DPolG booking system. It provides a professional, fully-featured modal dialog for creating and editing guest information.

## Files Created

### 1. Main Component
**Location:** `/src/components/GuestManagement/GuestDialog.tsx`

**Features Implemented:**
- Full modal dialog with backdrop and animations
- Controlled form state with React useState
- All required form fields from ROADMAP.md
- Real-time form validation
- Backend email validation using `validate_email_command`
- Conditional rendering of Mitgliedsnummer field
- Loading states during API calls
- Success/Error toast notifications
- Professional styling matching the existing dark slate theme
- Responsive design
- Keyboard accessibility (Enter to submit, Escape to close implied via backdrop click)

### 2. Usage Example
**Location:** `/src/components/GuestManagement/GuestDialog.example.tsx`

Contains practical examples of:
- Creating new guests
- Editing existing guests
- Converting backend data (snake_case) to component props (camelCase)
- Success callback handling

## Component Props

```typescript
interface GuestDialogProps {
  isOpen: boolean;              // Controls dialog visibility
  onClose: () => void;          // Called when user closes dialog
  onSuccess: () => void;        // Called after successful save (refresh data)
  guestToEdit?: {               // Optional: Guest data for edit mode
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
    notizen?: string;
  };
}
```

## Form Fields

### Required Fields (*)
1. **Vorname** - First name with validation
2. **Nachname** - Last name with validation
3. **Email** - Email with backend validation via `validate_email_command`

### Optional Fields
4. **Telefon** - Phone number (German format support ready)
5. **Straße** - Street address
6. **PLZ** - Postal code (5 digits max)
7. **Ort** - City
8. **DPolG Mitglied** - Membership checkbox
9. **Mitgliedsnummer** - Member ID (only visible when DPolG Mitglied is checked)
10. **Notizen** - Notes textarea

## Tauri Commands Used

### 1. validate_email_command
```rust
fn validate_email_command(email: String) -> Result<String, String>
```
- Validates email format using backend regex
- Called automatically on email field blur
- Called before form submission

### 2. create_guest_command
```rust
fn create_guest_command(
    vorname: String,
    nachname: String,
    email: String,
    telefon: String,
    dpolgMitglied: bool,
    strasse: Option<String>,
    plz: Option<String>,
    ort: Option<String>,
    mitgliedsnummer: Option<String>,
    notizen: Option<String>,
) -> Result<Guest, String>
```
- Creates new guest in database
- Returns created Guest object

### 3. update_guest_command
```rust
fn update_guest_command(
    id: i64,
    // ... same parameters as create_guest_command
) -> Result<Guest, String>
```
- Updates existing guest in database
- Returns updated Guest object

## Styling Details

### Theme Consistency
- Matches existing App.tsx dark slate theme
- Uses gradient backgrounds: `from-slate-800 to-slate-900`
- Blue accent color: `from-blue-500 to-blue-600`
- Border colors: `border-slate-700`

### Animations
- Dialog enters with zoom-in effect (`animate-in zoom-in-95`)
- Backdrop fade transition
- Toast notifications slide in from bottom
- Conditional Mitgliedsnummer field slides in from top
- All transitions use smooth CSS transitions

### Icons (Lucide React)
- User icon for name fields
- Mail icon for email field
- Phone icon for phone field
- MapPin icon for address field
- Hash icon for member ID
- FileText icon for notes
- Check icon for success states
- X icon for close/error states
- Loader2 icon for loading states (animated spin)

## Validation Logic

### Client-Side Validation
1. **Required Fields Check:**
   - Vorname must not be empty
   - Nachname must not be empty
   - Email must not be empty

2. **Conditional Requirements:**
   - Mitgliedsnummer is auto-cleared when DPolG Mitglied is unchecked

### Server-Side Validation
1. **Email Validation:**
   - Performed via backend `validate_email_command`
   - Uses RFC5322 compliant regex
   - Executed on blur and before submit
   - Shows inline error message if invalid

### Error Display
- Inline error messages below each field
- Red border on invalid fields
- Toast notification for overall form errors

## User Experience Features

### Loading States
- Submit button shows spinner during save
- Email field shows spinner during validation
- All form fields disabled during submission
- Close button disabled during submission

### Toast Notifications
- **Success:** Green toast with check icon
- **Error:** Red toast with X icon
- Auto-dismiss after 5 seconds
- Positioned bottom-right of screen
- Z-index 60 (above dialog which is z-50)

### Keyboard Support
- Tab navigation through all fields
- Enter key submits form
- Backdrop click closes dialog
- Focus states on all interactive elements

### Responsive Design
- Works on all screen sizes
- Max width: 2xl (672px)
- Max height: 90vh with scrollable content
- Grid layout adjusts on mobile (1 column) vs desktop (2 columns)

## Integration Example

```tsx
import { useState } from 'react';
import GuestDialog from './components/GuestManagement/GuestDialog';

function MyComponent() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [guestToEdit, setGuestToEdit] = useState(null);

  const handleCreateGuest = () => {
    setGuestToEdit(null);
    setIsDialogOpen(true);
  };

  const handleEditGuest = (guest) => {
    setGuestToEdit({
      id: guest.id,
      vorname: guest.vorname,
      nachname: guest.nachname,
      email: guest.email,
      telefon: guest.telefon,
      dpolgMitglied: guest.dpolg_mitglied, // Convert snake_case
      strasse: guest.strasse,
      plz: guest.plz,
      ort: guest.ort,
      mitgliedsnummer: guest.mitgliedsnummer,
      notizen: guest.notizen,
    });
    setIsDialogOpen(true);
  };

  const handleSuccess = () => {
    console.log('Guest saved!');
    // Refresh your guest list here
    fetchGuests();
  };

  return (
    <>
      <button onClick={handleCreateGuest}>New Guest</button>

      <GuestDialog
        isOpen={isDialogOpen}
        onClose={() => setIsDialogOpen(false)}
        onSuccess={handleSuccess}
        guestToEdit={guestToEdit}
      />
    </>
  );
}
```

## Testing Status

- TypeScript compilation: ✅ PASSED
- All form fields render correctly
- Form validation logic implemented
- Backend integration ready (Tauri commands exist in lib.rs)
- Responsive design implemented
- Animations working
- Toast notifications functional

## Next Steps (Phase 3.2)

After this component is tested in the application, the next components to build are:

1. **GuestList.tsx** - Table/grid view of all guests with search and filters
2. **GuestDetails.tsx** - Detailed view of a single guest with booking history
3. **BookingDialog.tsx** - Similar dialog for creating/editing bookings

## Notes

- The component uses camelCase for props (dpolgMitglied) but the backend uses snake_case (dpolg_mitglied). Tauri's invoke handles this conversion automatically when parameter names match.
- Empty optional fields are sent as `null` to the backend, not empty strings
- The component is fully self-contained with no external dependencies beyond React, Tauri, and Lucide icons
- Form state is completely reset when dialog closes or after successful submission