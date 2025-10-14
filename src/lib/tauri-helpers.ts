/**
 * Tauri Invoke Helpers - Automatische Parameter-Konvertierung
 *
 * PROBLEM: Tauri konvertiert nur bei konsistentem camelCase automatisch zu snake_case
 * LÖSUNG: Diese Helper-Funktionen validieren und konvertieren automatisch
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

/**
 * Konvertiert snake_case zu camelCase
 */
function snakeToCamel(str: string): string {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

/**
 * Konvertiert camelCase zu snake_case
 */
function camelToSnake(str: string): string {
  return str.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`);
}

/**
 * Prüft ob ein String snake_case enthält (Unterstriche)
 */
function hasSnakeCase(str: string): boolean {
  return str.includes('_');
}

/**
 * Konvertiert ein Objekt rekursiv von snake_case zu camelCase
 */
function convertObjectToCamelCase(obj: any): any {
  if (obj === null || obj === undefined) return obj;
  if (typeof obj !== 'object') return obj;
  if (Array.isArray(obj)) return obj.map(convertObjectToCamelCase);

  const result: any = {};
  for (const [key, value] of Object.entries(obj)) {
    const camelKey = snakeToCamel(key);
    result[camelKey] = typeof value === 'object' ? convertObjectToCamelCase(value) : value;
  }
  return result;
}

/**
 * Validiert dass ALLE Keys in camelCase sind (keine Unterstriche!)
 */
function validateCamelCase(obj: any, path: string = 'root'): string[] {
  const errors: string[] = [];

  if (obj === null || obj === undefined || typeof obj !== 'object') {
    return errors;
  }

  if (Array.isArray(obj)) {
    obj.forEach((item, index) => {
      errors.push(...validateCamelCase(item, `${path}[${index}]`));
    });
    return errors;
  }

  for (const [key, value] of Object.entries(obj)) {
    // Prüfe ob Key snake_case hat
    if (hasSnakeCase(key)) {
      errors.push(`❌ SNAKE_CASE DETECTED: "${key}" at ${path}.${key} - USE camelCase instead: "${snakeToCamel(key)}"`);
    }

    // Rekursiv für nested objects
    if (typeof value === 'object') {
      errors.push(...validateCamelCase(value, `${path}.${key}`));
    }
  }

  return errors;
}

/**
 * Sicherer Wrapper für invoke() der automatisch Parameter validiert und konvertiert
 *
 * @param command - Tauri Command Name
 * @param args - Parameter-Objekt (wird automatisch zu camelCase konvertiert falls nötig)
 * @param options - Optionen { strict: boolean, autoConvert: boolean }
 *
 * @example
 * // Strict Mode (Standard) - Wirft Fehler bei snake_case
 * await safeInvoke('update_booking', {
 *   roomId: 1,
 *   guestId: 1,
 *   paymentRecipientId: 1  // ✅ Alles camelCase
 * });
 *
 * // Auto-Convert Mode - Konvertiert automatisch
 * await safeInvoke('update_booking', {
 *   room_id: 1,              // wird zu roomId
 *   guest_id: 1,             // wird zu guestId
 *   payment_recipient_id: 1  // wird zu paymentRecipientId
 * }, { autoConvert: true });
 */
export async function safeInvoke<T>(
  command: string,
  args?: Record<string, any>,
  options: { strict?: boolean; autoConvert?: boolean } = {}
): Promise<T> {
  const { strict = true, autoConvert = false } = options;

  if (!args) {
    return tauriInvoke<T>(command);
  }

  // Validierung: Prüfe auf snake_case
  const validationErrors = validateCamelCase(args, 'args');

  if (validationErrors.length > 0) {
    const errorMessage = [
      '🚨 TAURI PARAMETER NAMING ERROR DETECTED!',
      '',
      'Tauri requires ALL parameters to be in camelCase!',
      'Mixed naming (camelCase + snake_case) breaks parameter conversion!',
      '',
      'Errors found:',
      ...validationErrors,
      '',
      autoConvert
        ? '✅ Auto-converting to camelCase...'
        : '❌ Set { autoConvert: true } to auto-fix, or change parameter names manually.',
    ].join('\n');

    if (strict && !autoConvert) {
      // Strict Mode: Fehler werfen
      console.error(errorMessage);
      throw new Error('Parameter naming convention violated. See console for details.');
    } else {
      // Warning ausgeben
      console.warn(errorMessage);
    }
  }

  // Auto-Convert wenn aktiviert
  const finalArgs = autoConvert ? convertObjectToCamelCase(args) : args;

  // Log für Debugging (in Development)
  if (process.env.NODE_ENV === 'development' && validationErrors.length > 0) {
    console.log('📤 [safeInvoke] Converted parameters:');
    console.log('  Before:', args);
    console.log('  After:', finalArgs);
  }

  return tauriInvoke<T>(command, finalArgs);
}

/**
 * Erstellt einen Type-Safe Invoker für einen spezifischen Command
 *
 * @example
 * interface UpdateBookingArgs {
 *   roomId: number;
 *   guestId: number;
 *   paymentRecipientId?: number | null;
 * }
 *
 * const updateBooking = createInvoker<BookingWithDetails, UpdateBookingArgs>('update_booking');
 * const result = await updateBooking({ roomId: 1, guestId: 1, paymentRecipientId: 1 });
 */
export function createInvoker<TResult, TArgs extends Record<string, any>>(
  command: string,
  options: { strict?: boolean; autoConvert?: boolean } = {}
) {
  return (args: TArgs): Promise<TResult> => {
    return safeInvoke<TResult>(command, args, options);
  };
}

/**
 * Utility: Konvertiert ein Objekt automatisch zu camelCase
 * Nützlich wenn man Daten vom Backend erhält die snake_case sind
 */
export function ensureCamelCase<T>(obj: any): T {
  return convertObjectToCamelCase(obj) as T;
}

/**
 * Utility: Validiert ob ein Objekt nur camelCase Keys hat
 * Gibt true zurück wenn alles OK ist, sonst false + console.error mit Details
 */
export function validateParameterNaming(obj: any, context: string = 'parameters'): boolean {
  const errors = validateCamelCase(obj, context);

  if (errors.length > 0) {
    console.error(`❌ Parameter naming validation failed in ${context}:`, errors);
    return false;
  }

  return true;
}

// Export auch die originale invoke für backwards compatibility
export { invoke } from '@tauri-apps/api/core';
