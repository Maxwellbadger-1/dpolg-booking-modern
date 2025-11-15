import { useEffect, useRef } from 'react';

// Global event handler registry (outside React lifecycle)
const globalHandlers = new Map<number, (reminderType: string) => void>();

// Global event listener (registered ONCE when first component mounts)
let isGlobalListenerRegistered = false;

function registerGlobalListener() {
  if (isGlobalListenerRegistered) {
    console.log('ℹ️ [GLOBAL] Listener already registered - skipping');
    return;
  }

  const handleGlobalEvent = (event: Event) => {
    console.log('🎯 [GLOBAL] Raw event received on window', { event });
    const customEvent = event as CustomEvent;
    const { reminderType } = customEvent.detail || {};

    console.log('🌍 [GLOBAL] reminder-updated event received', {
      reminderType,
      handlers: globalHandlers.size,
      handlerKeys: Array.from(globalHandlers.keys())
    });

    if (reminderType) {
      console.log(`🔔 [GLOBAL] Notifying ${globalHandlers.size} handler(s)...`);
      // Notify ALL registered handlers
      globalHandlers.forEach((handler, bookingId) => {
        console.log(`   → Notifying handler for booking ${bookingId}`);
        try {
          handler(reminderType);
          console.log(`   ✅ Handler for booking ${bookingId} executed successfully`);
        } catch (error) {
          console.error(`   ❌ Handler for booking ${bookingId} failed:`, error);
        }
      });
    } else {
      console.warn('⚠️ [GLOBAL] No reminderType in event detail!');
    }
  };

  window.addEventListener('reminder-updated', handleGlobalEvent as EventListener);
  isGlobalListenerRegistered = true;
  console.log('✅ [GLOBAL] Global reminder-updated listener registered on window');
}

/**
 * Custom hook to subscribe to global reminder updates
 * Survives component unmount/remount cycles
 */
export function useGlobalReminderUpdates(
  bookingId: number,
  onUpdate: (reminderType: string) => void
) {
  const onUpdateRef = useRef(onUpdate);

  // Keep callback ref fresh
  useEffect(() => {
    onUpdateRef.current = onUpdate;
  });

  useEffect(() => {
    // Register global listener (only happens once)
    registerGlobalListener();

    // Register this component's handler
    const handler = (reminderType: string) => {
      onUpdateRef.current(reminderType);
    };

    globalHandlers.set(bookingId, handler);
    console.log(`✅ [HOOK] Registered handler for booking ${bookingId} (total: ${globalHandlers.size})`);

    return () => {
      globalHandlers.delete(bookingId);
      console.log(`🗑️ [HOOK] Unregistered handler for booking ${bookingId} (remaining: ${globalHandlers.size})`);
    };
  }, [bookingId]);
}
