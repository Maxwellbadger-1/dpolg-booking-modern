import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ActiveLock, LockChangeEvent, LockStatus } from '../types/lock';
import { useUser } from '../context/UserContext';

interface UseLockManagerOptions {
  bookingId: number | null;
  onLockConflict?: (lockedBy: string) => void;
}

export function useLockManager({ bookingId, onLockConflict }: UseLockManagerOptions) {
  const { userName } = useUser();
  const [lockStatus, setLockStatus] = useState<LockStatus>({
    isLocked: false,
    isOwnLock: false,
  });
  const [allLocks, setAllLocks] = useState<ActiveLock[]>([]);
  const heartbeatIntervalRef = useRef<number | null>(null);

  // Acquire lock when bookingId changes
  const acquireLock = useCallback(async () => {
    if (!bookingId || !userName) return;

    try {
      const lock: ActiveLock = await invoke('acquire_booking_lock_pg', {
        bookingId,
        userName,
      });

      setLockStatus({
        isLocked: true,
        lockedBy: lock.userName,
        lockedSince: new Date(lock.lockedAt),
        isOwnLock: lock.userName === userName,
      });

      // Start heartbeat (every 30 seconds)
      if (heartbeatIntervalRef.current) {
        clearInterval(heartbeatIntervalRef.current);
      }

      heartbeatIntervalRef.current = window.setInterval(async () => {
        try {
          await invoke('update_lock_heartbeat_pg', { bookingId });
        } catch (error) {
          console.error('❌ Heartbeat failed:', error);
          // Lock might have expired - try to reacquire
          acquireLock();
        }
      }, 30000); // 30 seconds

    } catch (error) {
      console.error('❌ Failed to acquire lock:', error);

      // Check if it's a conflict error
      const errorMsg = String(error);
      if (errorMsg.includes('wird bereits von')) {
        const match = errorMsg.match(/'([^']+)'/);
        const lockedBy = match ? match[1] : 'einem anderen Benutzer';

        setLockStatus({
          isLocked: true,
          lockedBy,
          isOwnLock: false,
        });

        if (onLockConflict) {
          onLockConflict(lockedBy);
        }
      }
    }
  }, [bookingId, userName, onLockConflict]);

  // Release lock
  const releaseLock = useCallback(async () => {
    if (!bookingId || !userName) return;

    // Stop heartbeat
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
    }

    try {
      await invoke('release_booking_lock_pg', {
        bookingId,
        userName,
      });

      setLockStatus({
        isLocked: false,
        isOwnLock: false,
      });
    } catch (error) {
      console.error('❌ Failed to release lock:', error);
    }
  }, [bookingId, userName]);

  // Get all active locks (for debugging/admin view)
  const refreshAllLocks = useCallback(async () => {
    try {
      const locks: ActiveLock[] = await invoke('get_all_active_locks_pg');
      setAllLocks(locks);
    } catch (error) {
      console.error('❌ Failed to get active locks:', error);
    }
  }, []);

  // Listen for lock changes from other users
  useEffect(() => {
    const unlisten = listen<LockChangeEvent>('lock-change', (event) => {
      const lockEvent = event.payload;

      // Only update if it's for our booking
      if (lockEvent.bookingId === bookingId) {
        if (lockEvent.action === 'ACQUIRED') {
          setLockStatus({
            isLocked: true,
            lockedBy: lockEvent.userName,
            lockedSince: lockEvent.lockedAt ? new Date(lockEvent.lockedAt) : undefined,
            isOwnLock: lockEvent.userName === userName,
          });
        } else if (lockEvent.action === 'RELEASED') {
          // Only clear if it was locked by the same user
          setLockStatus(prev => {
            if (prev.lockedBy === lockEvent.userName) {
              return {
                isLocked: false,
                isOwnLock: false,
              };
            }
            return prev;
          });
        }
      }

      // Refresh all locks list
      refreshAllLocks();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [bookingId, userName, refreshAllLocks]);

  // Acquire lock when booking opens
  useEffect(() => {
    if (bookingId) {
      acquireLock();
    }

    // Release lock when component unmounts or bookingId changes
    return () => {
      if (bookingId) {
        releaseLock();
      }
    };
  }, [bookingId]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (heartbeatIntervalRef.current) {
        clearInterval(heartbeatIntervalRef.current);
      }
    };
  }, []);

  return {
    lockStatus,
    allLocks,
    acquireLock,
    releaseLock,
    refreshAllLocks,
  };
}
