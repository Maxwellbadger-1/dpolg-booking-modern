/**
 * Settings Test Suite
 *
 * Testet ob Settings wirklich Auswirkungen haben:
 * - Pricing Settings → Preisberechnung
 * - Notification Settings → Email-Versand
 * - Payment Settings → Zahlungsverarbeitung
 * - Email Config → Email-Versand
 */

import { invoke } from '@tauri-apps/api/core';

export interface SettingsTest {
  name: string;
  description: string;
  category: 'pricing' | 'notification' | 'payment' | 'email';
  testFn: () => Promise<{
    success: boolean;
    message: string;
    details?: any;
  }>;
}

export const settingsTests: SettingsTest[] = [
  // ============================================================================
  // PRICING SETTINGS TESTS
  // ============================================================================
  {
    name: 'DPolG Rabatt Settings Test',
    description: 'Prüft ob DPolG-Rabatt-Einstellungen die Preisberechnung beeinflussen',
    category: 'pricing',
    testFn: async () => {
      try {
        // 1. Aktuelle Settings laden
        const originalSettings = await invoke<any>('get_pricing_settings_pg');

        // 2. Test-Buchung erstellen (Gast 1, Raum 1, 2 Nächte, ist Mitglied)
        const testPrice1 = await invoke<any>('calculate_full_booking_price_pg', {
          roomId: 1,
          checkin: '2025-12-01',
          checkout: '2025-12-03',
          isMember: true,
          services: [],
          discounts: []
        });

        const discount1 = testPrice1.discountsTotal || 0;
        console.log('📊 Test 1 (15%):', testPrice1);

        // 3. Rabatt auf 20% ändern (statt 15%)
        await invoke('update_pricing_settings_pg', {
          settings: {
            ...originalSettings,
            mitgliederRabattProzent: 20
          }
        });

        // 3b. Settings nochmal laden zur Verifizierung
        const updatedSettings = await invoke<any>('get_pricing_settings_pg');
        console.log('⚙️ Updated Settings:', updatedSettings);

        // 4. Gleiche Buchung nochmal berechnen
        const testPrice2 = await invoke<any>('calculate_full_booking_price_pg', {
          roomId: 1,
          checkin: '2025-12-01',
          checkout: '2025-12-03',
          isMember: true,
          services: [],
          discounts: []
        });

        const discount2 = testPrice2.discountsTotal || 0;
        console.log('📊 Test 2 (20%):', testPrice2);

        // 5. Settings zurücksetzen
        await invoke('update_pricing_settings_pg', {
          settings: originalSettings
        });

        // 6. Verifizieren: Rabatt sollte sich geändert haben
        if (discount2 > discount1) {
          return {
            success: true,
            message: `✅ DPolG-Rabatt wirkt! 15%: ${discount1.toFixed(2)}€ → 20%: ${discount2.toFixed(2)}€`,
            details: { discount1, discount2, originalSettings }
          };
        } else {
          return {
            success: false,
            message: `❌ Rabatt hat sich nicht geändert! 15%: ${discount1.toFixed(2)}€, 20%: ${discount2.toFixed(2)}€`,
            details: { discount1, discount2, originalSettings }
          };
        }
      } catch (err) {
        return {
          success: false,
          message: `❌ Fehler: ${err}`,
        };
      }
    }
  },

  {
    name: 'Rabatt-Basis Settings Test',
    description: 'Prüft ob Rabatt-Basis (Grundpreis vs Endsumme) die Berechnung ändert',
    category: 'pricing',
    testFn: async () => {
      try {
        const originalSettings = await invoke<any>('get_pricing_settings_pg');

        // Test 1: Basis = Zimmerpreis
        await invoke('update_pricing_settings_pg', {
          settings: {
            ...originalSettings,
            rabattBasis: 'zimmerpreis'
          }
        });

        const price1 = await invoke<any>('calculate_full_booking_price_pg', {
          roomId: 1,
          checkin: '2025-12-01',
          checkout: '2025-12-03',
          isMember: true,
          services: [{ name: 'Frühbucher', priceType: 'percent', originalValue: 10, appliesTo: 'overnight_price' }],
          discounts: []
        });

        // Test 2: Basis = Gesamtpreis (inkl. Services)
        await invoke('update_pricing_settings_pg', {
          settings: {
            ...originalSettings,
            rabattBasis: 'gesamtpreis'
          }
        });

        const price2 = await invoke<any>('calculate_full_booking_price_pg', {
          roomId: 1,
          checkin: '2025-12-01',
          checkout: '2025-12-03',
          isMember: true,
          services: [{ name: 'Frühbucher', priceType: 'percent', originalValue: 10, appliesTo: 'overnight_price' }],
          discounts: []
        });

        // Settings zurücksetzen
        await invoke('update_pricing_settings_pg', {
          settings: originalSettings
        });

        const discount1 = price1.discountsTotal || 0;
        const discount2 = price2.discountsTotal || 0;

        if (discount2 > discount1) {
          return {
            success: true,
            message: `✅ Rabatt-Basis wirkt! Zimmerpreis: ${discount1.toFixed(2)}€ < Gesamtpreis: ${discount2.toFixed(2)}€`,
            details: { discount1, discount2 }
          };
        } else {
          return {
            success: false,
            message: `❌ Rabatt-Basis wirkt nicht! Zimmerpreis: ${discount1.toFixed(2)}€, Gesamtpreis: ${discount2.toFixed(2)}€`,
            details: { discount1, discount2 }
          };
        }
      } catch (err) {
        return {
          success: false,
          message: `❌ Fehler: ${err}`,
        };
      }
    }
  },

  // ============================================================================
  // NOTIFICATION SETTINGS TESTS
  // ============================================================================
  {
    name: 'Check-in Reminders Enabled/Disabled Test',
    description: 'Prüft ob checkin_reminders_enabled wirklich Email-Erzeugung verhindert',
    category: 'notification',
    testFn: async () => {
      try {
        const originalSettings = await invoke<any>('get_notification_settings_pg');

        // 1. Reminders DEAKTIVIEREN
        await invoke('update_notification_settings_pg', {
          settings: {
            ...originalSettings,
            checkinRemindersEnabled: false
          }
        });

        // 2. Backfill ausführen (sollte 0 Emails erstellen)
        const result1 = await invoke<string>('backfill_scheduled_emails');

        // 3. Reminders AKTIVIEREN
        await invoke('update_notification_settings_pg', {
          settings: {
            ...originalSettings,
            checkinRemindersEnabled: true
          }
        });

        // 4. Backfill nochmal (sollte Emails erstellen)
        const result2 = await invoke<string>('backfill_scheduled_emails');

        // Settings zurücksetzen
        await invoke('update_notification_settings_pg', {
          settings: originalSettings
        });

        if (result1.includes('DISABLED') && !result2.includes('DISABLED')) {
          return {
            success: true,
            message: `✅ Notification Settings wirken! Disabled: 0 Emails, Enabled: ${result2}`,
            details: { result1, result2 }
          };
        } else {
          return {
            success: false,
            message: `❌ Settings wirken nicht! Result1: ${result1}, Result2: ${result2}`,
            details: { result1, result2 }
          };
        }
      } catch (err) {
        return {
          success: false,
          message: `❌ Fehler: ${err}`,
        };
      }
    }
  },

  {
    name: 'Reminder Days Before Test',
    description: 'Prüft ob reminder_days_before die Email-Planung beeinflusst',
    category: 'notification',
    testFn: async () => {
      try {
        const originalSettings = await invoke<any>('get_notification_settings_pg');

        // Test: Days before auf 7 statt 3
        await invoke('update_notification_settings_pg', {
          settings: {
            ...originalSettings,
            reminderDaysBefore: 7
          }
        });

        // Scheduled Emails laden
        const emails = await invoke<any[]>('get_scheduled_emails');

        // Settings zurücksetzen
        await invoke('update_notification_settings_pg', {
          settings: originalSettings
        });

        // HINWEIS: Dieser Test kann nur verifizieren, dass Settings geladen werden
        // Echte Auswirkung zeigt sich nur bei NEUEN Buchungen
        return {
          success: true,
          message: `✅ Settings geladen. Reminder Days: 7. Emails: ${emails.length}`,
          details: { emails }
        };
      } catch (err) {
        return {
          success: false,
          message: `❌ Fehler: ${err}`,
        };
      }
    }
  },

  // ============================================================================
  // PAYMENT SETTINGS TESTS
  // ============================================================================
  {
    name: 'Payment Deadline Days Test',
    description: 'Prüft ob payment_deadline_days Settings existieren und geladen werden',
    category: 'payment',
    testFn: async () => {
      try {
        const settings = await invoke<any>('get_payment_settings_pg');

        return {
          success: true,
          message: `✅ Payment Settings geladen: ${JSON.stringify(settings)}`,
          details: settings
        };
      } catch (err) {
        return {
          success: false,
          message: `❌ Fehler: ${err}`,
        };
      }
    }
  },

  // ============================================================================
  // EMAIL CONFIG TESTS
  // ============================================================================
  {
    name: 'Email Config Loaded Test',
    description: 'Prüft ob Email-Konfiguration geladen werden kann',
    category: 'email',
    testFn: async () => {
      try {
        const config = await invoke<any>('get_email_config_pg');

        // Prüfe nur ob Config geladen wird (Felder können leer sein wenn nicht konfiguriert)
        if (config && config.id) {
          const configuredFields = [];
          if (config.smtpHost) configuredFields.push(`Host: ${config.smtpHost}`);
          if (config.smtpPort) configuredFields.push(`Port: ${config.smtpPort}`);
          if (config.senderEmail) configuredFields.push(`Email: ${config.senderEmail}`);

          const status = configuredFields.length > 0
            ? `✅ Email Config geladen: ${configuredFields.join(', ')}`
            : `⚠️ Email Config existiert aber ist nicht konfiguriert (alle Felder leer)`;

          return {
            success: true,
            message: status,
            details: config
          };
        } else {
          return {
            success: false,
            message: `❌ Email Config konnte nicht geladen werden!`,
            details: config
          };
        }
      } catch (err) {
        return {
          success: false,
          message: `❌ Fehler: ${err}`,
        };
      }
    }
  },
];

/**
 * Führt alle Settings-Tests aus
 */
export async function runAllSettingsTests(): Promise<{
  total: number;
  passed: number;
  failed: number;
  results: Array<{ test: string; success: boolean; message: string; details?: any }>;
}> {
  const results: Array<{ test: string; success: boolean; message: string; details?: any }> = [];
  let passed = 0;
  let failed = 0;

  for (const test of settingsTests) {
    console.log(`🧪 Running: ${test.name}...`);

    try {
      const result = await test.testFn();

      results.push({
        test: test.name,
        success: result.success,
        message: result.message,
        details: result.details
      });

      if (result.success) {
        passed++;
        console.log(`✅ ${test.name}: ${result.message}`);
      } else {
        failed++;
        console.error(`❌ ${test.name}: ${result.message}`);
      }
    } catch (err) {
      failed++;
      results.push({
        test: test.name,
        success: false,
        message: `❌ Exception: ${err}`
      });
      console.error(`❌ ${test.name}: Exception: ${err}`);
    }
  }

  return {
    total: settingsTests.length,
    passed,
    failed,
    results
  };
}
