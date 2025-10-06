/**
 * Command Pattern Implementation for Undo/Redo
 *
 * Industry standard approach used by Figma, Notion, VSCode, etc.
 * - Frontend-only undo/redo (no backend roundtrip)
 * - Instant response (< 10ms)
 * - Two stacks: undoStack and redoStack
 */

import { Booking, Guest, Room } from '../types/booking';

// EmailLog interface (from EmailHistoryView)
interface EmailLog {
  id: number;
  booking_id: number;
  guest_id: number;
  template_name: string;
  recipient_email: string;
  subject: string;
  status: string;
  error_message: string | null;
  sent_at: string;
}

// Command Interface
export interface Command {
  execute: () => void;
  undo: () => void;
  description: string; // For UI display
  timestamp: Date;
}

// ============================================
// BOOKING COMMANDS
// ============================================

export class UpdateBookingStatusCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private bookingId: number,
    private oldStatus: string,
    private newStatus: string,
    private setBookings: React.Dispatch<React.SetStateAction<Booking[]>>
  ) {
    this.description = `Status geändert: ${oldStatus} → ${newStatus}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setBookings(prev => prev.map(b =>
      b.id === this.bookingId ? { ...b, status: this.newStatus } : b
    ));
  }

  undo() {
    this.setBookings(prev => prev.map(b =>
      b.id === this.bookingId ? { ...b, status: this.oldStatus } : b
    ));
  }
}

export class UpdateBookingPaymentCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private bookingId: number,
    private oldPaid: boolean,
    private newPaid: boolean,
    private oldPaidAt: string | null,
    private newPaidAt: string | null,
    private oldMethod: string | null,
    private newMethod: string | null,
    private setBookings: React.Dispatch<React.SetStateAction<Booking[]>>
  ) {
    this.description = `Zahlung ${newPaid ? 'markiert' : 'entfernt'}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setBookings(prev => prev.map(b =>
      b.id === this.bookingId ? {
        ...b,
        bezahlt: this.newPaid,
        bezahlt_am: this.newPaidAt,
        zahlungsmethode: this.newMethod
      } : b
    ));
  }

  undo() {
    this.setBookings(prev => prev.map(b =>
      b.id === this.bookingId ? {
        ...b,
        bezahlt: this.oldPaid,
        bezahlt_am: this.oldPaidAt,
        zahlungsmethode: this.oldMethod
      } : b
    ));
  }
}

export class CreateBookingCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private booking: Booking,
    private setBookings: React.Dispatch<React.SetStateAction<Booking[]>>
  ) {
    this.description = `Buchung erstellt: ${booking.reservierungsnummer}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setBookings(prev => [...prev, this.booking]);
  }

  undo() {
    this.setBookings(prev => prev.filter(b => b.id !== this.booking.id));
  }
}

export class UpdateBookingCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private bookingId: number,
    private oldBooking: Booking,
    private newBooking: Booking,
    private setBookings: React.Dispatch<React.SetStateAction<Booking[]>>
  ) {
    this.description = `Buchung bearbeitet: ${newBooking.reservierungsnummer}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setBookings(prev => prev.map(b =>
      b.id === this.bookingId ? this.newBooking : b
    ));
  }

  undo() {
    this.setBookings(prev => prev.map(b =>
      b.id === this.bookingId ? this.oldBooking : b
    ));
  }
}

export class DeleteBookingCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private booking: Booking,
    private setBookings: React.Dispatch<React.SetStateAction<Booking[]>>
  ) {
    this.description = `Buchung gelöscht: ${booking.reservierungsnummer}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setBookings(prev => prev.filter(b => b.id !== this.booking.id));
  }

  undo() {
    this.setBookings(prev => [...prev, this.booking]);
  }
}

// ============================================
// GUEST COMMANDS
// ============================================

export class CreateGuestCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private guest: Guest,
    private setGuests: React.Dispatch<React.SetStateAction<Guest[]>>
  ) {
    this.description = `Gast erstellt: ${guest.vorname} ${guest.nachname}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setGuests(prev => [...prev, this.guest]);
  }

  undo() {
    this.setGuests(prev => prev.filter(g => g.id !== this.guest.id));
  }
}

export class UpdateGuestCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private guestId: number,
    private oldGuest: Guest,
    private newGuest: Guest,
    private setGuests: React.Dispatch<React.SetStateAction<Guest[]>>
  ) {
    this.description = `Gast bearbeitet: ${newGuest.vorname} ${newGuest.nachname}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setGuests(prev => prev.map(g =>
      g.id === this.guestId ? this.newGuest : g
    ));
  }

  undo() {
    this.setGuests(prev => prev.map(g =>
      g.id === this.guestId ? this.oldGuest : g
    ));
  }
}

export class DeleteGuestCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private guest: Guest,
    private setGuests: React.Dispatch<React.SetStateAction<Guest[]>>
  ) {
    this.description = `Gast gelöscht: ${guest.vorname} ${guest.nachname}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setGuests(prev => prev.filter(g => g.id !== this.guest.id));
  }

  undo() {
    this.setGuests(prev => [...prev, this.guest]);
  }
}

// ============================================
// ROOM COMMANDS
// ============================================

export class CreateRoomCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private room: Room,
    private setRooms: React.Dispatch<React.SetStateAction<Room[]>>
  ) {
    this.description = `Zimmer erstellt: ${room.name}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setRooms(prev => [...prev, this.room]);
  }

  undo() {
    this.setRooms(prev => prev.filter(r => r.id !== this.room.id));
  }
}

export class UpdateRoomCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private roomId: number,
    private oldRoom: Room,
    private newRoom: Room,
    private setRooms: React.Dispatch<React.SetStateAction<Room[]>>
  ) {
    this.description = `Zimmer bearbeitet: ${newRoom.name}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setRooms(prev => prev.map(r =>
      r.id === this.roomId ? this.newRoom : r
    ));
  }

  undo() {
    this.setRooms(prev => prev.map(r =>
      r.id === this.roomId ? this.oldRoom : r
    ));
  }
}

export class DeleteRoomCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private room: Room,
    private setRooms: React.Dispatch<React.SetStateAction<Room[]>>
  ) {
    this.description = `Zimmer gelöscht: ${room.name}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setRooms(prev => prev.filter(r => r.id !== this.room.id));
  }

  undo() {
    this.setRooms(prev => [...prev, this.room]);
  }
}

// ============================================
// TAPE CHART COMMANDS (Drag & Drop, Resize)
// ============================================

export class UpdateBookingDatesCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private bookingId: number,
    private oldCheckinDate: string,
    private newCheckinDate: string,
    private oldCheckoutDate: string,
    private newCheckoutDate: string,
    private oldRoomId: number,
    private newRoomId: number,
    private setBookings: React.Dispatch<React.SetStateAction<Booking[]>>
  ) {
    const formatDate = (date: string) => new Date(date).toLocaleDateString('de-DE');
    this.description = `Buchung verschoben: ${formatDate(oldCheckinDate)} → ${formatDate(newCheckinDate)}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setBookings(prev => prev.map(b =>
      b.id === this.bookingId ? {
        ...b,
        checkin_date: this.newCheckinDate,
        checkout_date: this.newCheckoutDate,
        room_id: this.newRoomId
      } : b
    ));
  }

  undo() {
    this.setBookings(prev => prev.map(b =>
      b.id === this.bookingId ? {
        ...b,
        checkin_date: this.oldCheckinDate,
        checkout_date: this.oldCheckoutDate,
        room_id: this.oldRoomId
      } : b
    ));
  }
}

// ============================================
// EMAIL LOG COMMANDS
// ============================================

export class DeleteEmailLogCommand implements Command {
  description: string;
  timestamp: Date;

  constructor(
    private emailLog: EmailLog,
    private setEmailLogs: React.Dispatch<React.SetStateAction<EmailLog[]>>
  ) {
    this.description = `Email-Log gelöscht: ${emailLog.recipient_email}`;
    this.timestamp = new Date();
  }

  execute() {
    this.setEmailLogs(prev => prev.filter(e => e.id !== this.emailLog.id));
  }

  undo() {
    this.setEmailLogs(prev => [...prev, this.emailLog]);
  }
}

// ============================================
// COMMAND MANAGER
// ============================================

export class CommandManager {
  private undoStack: Command[] = [];
  private redoStack: Command[] = [];
  private maxStackSize = 50; // Limit to prevent memory issues
  private listeners: (() => void)[] = [];

  executeCommand(command: Command) {
    command.execute();
    this.undoStack.push(command);
    this.redoStack = []; // Clear redo stack on new command

    // Limit stack size
    if (this.undoStack.length > this.maxStackSize) {
      this.undoStack.shift();
    }

    this.notifyListeners();
  }

  undo(): boolean {
    const command = this.undoStack.pop();
    if (!command) return false;

    command.undo();
    this.redoStack.push(command);
    this.notifyListeners();
    return true;
  }

  redo(): boolean {
    const command = this.redoStack.pop();
    if (!command) return false;

    command.execute();
    this.undoStack.push(command);
    this.notifyListeners();
    return true;
  }

  canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  getUndoStack(): Command[] {
    return [...this.undoStack].reverse(); // Newest first
  }

  getRedoStack(): Command[] {
    return [...this.redoStack].reverse(); // Newest first
  }

  clear() {
    this.undoStack = [];
    this.redoStack = [];
    this.notifyListeners();
  }

  // Observer pattern for UI updates
  addListener(callback: () => void) {
    this.listeners.push(callback);
  }

  removeListener(callback: () => void) {
    this.listeners = this.listeners.filter(l => l !== callback);
  }

  private notifyListeners() {
    this.listeners.forEach(callback => callback());
  }
}

// Global singleton instance
export const commandManager = new CommandManager();
