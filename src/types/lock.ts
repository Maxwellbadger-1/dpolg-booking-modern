// Lock System Types (Presence System)

export interface ActiveLock {
  id: number;
  bookingId: number;
  userName: string;
  lockedAt: string;
  lastHeartbeat: string;
}

export interface LockChangeEvent {
  action: 'ACQUIRED' | 'RELEASED';
  bookingId: number;
  userName: string;
  lockedAt?: string;
}

export interface LockStatus {
  isLocked: boolean;
  lockedBy?: string;
  lockedSince?: Date;
  isOwnLock: boolean;
}
