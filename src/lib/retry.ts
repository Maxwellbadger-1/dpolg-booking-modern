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
    console.log(`= Retry attempt ${attempt}:`, error);
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

  console.log('PPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPP');
  console.log(`=€ [invokeWithRetry] Command: ${command}`);
  console.log(`=æ Args:`, args);
  console.log(`™  Options:`, opts);
  console.log('PPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPP');

  let lastError: unknown;

  for (let attempt = 1; attempt <= opts.maxRetries; attempt++) {
    try {
      console.log(`<¯ [invokeWithRetry] Attempt ${attempt}/${opts.maxRetries}...`);
      const result = await invoke<T>(command, args);
      console.log(` [invokeWithRetry] Success on attempt ${attempt}`);
      return result;
    } catch (error) {
      lastError = error;
      console.error(`L [invokeWithRetry] Attempt ${attempt} failed:`, error);

      // Letzter Versuch - kein Retry mehr
      if (attempt === opts.maxRetries) {
        console.error(`=¥ [invokeWithRetry] All ${opts.maxRetries} attempts failed!`);
        break;
      }

      // Prüfe ob Fehler retry-fähig ist
      if (!isRetryableError(error)) {
        console.warn(
          `   [invokeWithRetry] Error is not retryable, aborting after attempt ${attempt}`
        );
        break;
      }

      // Exponential Backoff berechnen
      const delay = opts.initialDelay * Math.pow(opts.backoffMultiplier, attempt - 1);
      console.log(`ð [invokeWithRetry] Waiting ${delay}ms before retry...`);

      // onRetry callback
      opts.onRetry(attempt, error);

      // Warten vor nächstem Versuch
      await sleep(delay);
    }
  }

  // Alle Versuche fehlgeschlagen
  throw lastError;
}

/**
 * Batch Retry - Führt mehrere Commands parallel aus mit Retry Logic
 *
 * @example
 * const results = await batchInvokeWithRetry([
 *   { command: 'get_booking_command', args: { id: 1 } },
 *   { command: 'get_booking_command', args: { id: 2 } },
 * ]);
 */
export async function batchInvokeWithRetry<T>(
  requests: Array<{ command: string; args?: Record<string, unknown> }>,
  options?: RetryOptions
): Promise<T[]> {
  console.log(`=€ [batchInvokeWithRetry] Starting ${requests.length} parallel requests...`);

  const promises = requests.map((req) =>
    invokeWithRetry<T>(req.command, req.args, options)
  );

  try {
    const results = await Promise.all(promises);
    console.log(` [batchInvokeWithRetry] All ${requests.length} requests succeeded`);
    return results;
  } catch (error) {
    console.error(`L [batchInvokeWithRetry] At least one request failed:`, error);
    throw error;
  }
}

/**
 * Retry mit Circuit Breaker Pattern (optional, für zukünftige Verwendung)
 *
 * Circuit Breaker verhindert, dass bei dauerhaft fehlenden Services
 * unnötig viele Requests gesendet werden.
 *
 * States: CLOSED (normal), OPEN (fehlerhaft), HALF_OPEN (testing)
 */
class CircuitBreaker {
  private failureCount = 0;
  private lastFailureTime?: number;
  private state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';

  constructor(
    private threshold: number = 5, // Nach 5 Fehlern ’ OPEN
    private timeout: number = 60000 // Nach 60s ’ HALF_OPEN
  ) {}

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.state === 'OPEN') {
      if (Date.now() - (this.lastFailureTime || 0) > this.timeout) {
        console.log('= [CircuitBreaker] Transitioning to HALF_OPEN');
        this.state = 'HALF_OPEN';
      } else {
        throw new Error('Circuit breaker is OPEN - service unavailable');
      }
    }

    try {
      const result = await fn();
      if (this.state === 'HALF_OPEN') {
        console.log(' [CircuitBreaker] Transitioning to CLOSED');
        this.state = 'CLOSED';
        this.failureCount = 0;
      }
      return result;
    } catch (error) {
      this.failureCount++;
      this.lastFailureTime = Date.now();

      if (this.failureCount >= this.threshold) {
        console.error('=¥ [CircuitBreaker] Transitioning to OPEN');
        this.state = 'OPEN';
      }

      throw error;
    }
  }

  reset() {
    this.state = 'CLOSED';
    this.failureCount = 0;
    this.lastFailureTime = undefined;
  }
}

export const circuitBreaker = new CircuitBreaker();
