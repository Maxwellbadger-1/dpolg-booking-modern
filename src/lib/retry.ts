import { invoke } from '@tauri-apps/api/core';

/**
 * Retry Logic Helper - Professionelle Fehlerbehandlung für Backend-Calls
 *
 * Best Practice von professionellen Apps (AWS SDK, Stripe, etc.)
 * - Automatisches Retry bei transienten Fehlern
 * - Exponential Backoff (1s, 2s, 4s, ...)
 * - Detailliertes Logging für Debugging
 */

interface RetryOptions {
  maxRetries?: number;
  initialDelay?: number; // in milliseconds
  backoffMultiplier?: number;
  onRetry?: (attempt: number, error: unknown) => void;
}

const DEFAULT_OPTIONS: Required<RetryOptions> = {
  maxRetries: 3,
  initialDelay: 1000, // 1 second
  backoffMultiplier: 2, // exponential: 1s, 2s, 4s
  onRetry: (attempt, error) => {
    console.log(`Retry attempt ${attempt}:`, error);
  },
};

/**
 * Sleep-Funktion für Exponential Backoff
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Prüft ob ein Fehler retry-fähig ist (transient error)
 * Transiente Fehler: Netzwerk-Timeouts, temporäre Server-Fehler, etc.
 */
function isRetryableError(error: unknown): boolean {
  if (typeof error === 'string') {
    const errorLower = error.toLowerCase();
    return (
      errorLower.includes('network') ||
      errorLower.includes('timeout') ||
      errorLower.includes('connection') ||
      errorLower.includes('temporarily unavailable') ||
      errorLower.includes('503') || // Service Unavailable
      errorLower.includes('502') || // Bad Gateway
      errorLower.includes('504') // Gateway Timeout
    );
  }
  if (error instanceof Error) {
    return isRetryableError(error.message);
  }
  return false;
}

/**
 * Wrapper für Tauri invoke mit Retry Logic
 *
 * @example
 * const booking = await invokeWithRetry<Booking>('get_booking_command', { id: 123 });
 */
export async function invokeWithRetry<T>(
  command: string,
  args?: Record<string, unknown>,
  options?: RetryOptions
): Promise<T> {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  let lastError: unknown;

  for (let attempt = 1; attempt <= opts.maxRetries; attempt++) {
    try {
      const result = await invoke<T>(command, args);
      return result;
    } catch (error) {
      lastError = error;
      console.error(`[invokeWithRetry] Attempt ${attempt} failed:`, error);

      // Letzter Versuch - kein Retry mehr
      if (attempt === opts.maxRetries) {
        console.error(`[invokeWithRetry] All ${opts.maxRetries} attempts failed!`);
        break;
      }

      // Prüfe ob Fehler retry-fähig ist
      if (!isRetryableError(error)) {
        console.warn(
          `[invokeWithRetry] Error is not retryable, aborting after attempt ${attempt}`
        );
        break;
      }

      // Exponential Backoff berechnen
      const delay = opts.initialDelay * Math.pow(opts.backoffMultiplier, attempt - 1);

      // onRetry callback
      opts.onRetry(attempt, error);

      // Warten vor nächstem Versuch
      await sleep(delay);
    }
  }

  // Alle Versuche fehlgeschlagen
  throw lastError;
}
