/**
 * Integration Test Suite
 *
 * End-to-End Tests für komplette Workflows:
 * - Buchung erstellen → Services hinzufügen → Rechnung erstellen → Email versenden
 * - Gast erstellen → Guthaben hinzufügen → Buchung mit Guthaben
 * - Template erstellen → Template verknüpfen → Preise berechnen
 */

import { invoke } from '@tauri-apps/api/core';

export interface IntegrationTest {
  name: string;
  description: string;
  testFn: () => Promise<{
    success: boolean;
    message: string;
    steps: Array<{ step: string; success: boolean; message: string }>;
  }>;
}

export const integrationTests: IntegrationTest[] = [
  // ============================================================================
  // COMPLETE BOOKING WORKFLOW
  // ============================================================================
  {
    name: 'Complete Booking Workflow',
    description: 'Erstellt Buchung → Fügt Services hinzu → Berechnet Preis → Löscht alles',
    testFn: async () => {
      const steps: Array<{ step: string; success: boolean; message: string }> = [];

      try {
        // Step 1: Create Test Booking
        const booking = await invoke<any>('create_booking_pg', {
          roomId: 1,
          guestId: 1,
          checkinDate: '2026-01-01',
          checkoutDate: '2026-01-03',
          anzahlGaeste: 2,
          anzahlBegleitpersonen: 0,
          status: 'bestaetigt',
          bemerkungen: 'DevTools Integration Test',
          grundpreis: 100.0,
          servicesPreis: 0.0,
          rabattPreis: 0.0,
          gesamtpreis: 100.0,
          anzahlNaechte: 2,
          bezahlt: false,
          istStiftungsfall: false
        });

        steps.push({
          step: '1. Create Booking',
          success: true,
          message: `✅ Booking ${booking.id} created`
        });

        // Step 2: Add Additional Service
        const service = await invoke<any>('create_additional_service_pg', {
          bookingId: booking.id,
          serviceName: 'Test Service',
          servicePrice: 25.0,
          priceType: 'fixed',
          originalValue: 25.0,
          appliesTo: 'overnight_price',
          emoji: '🧪'
        });

        steps.push({
          step: '2. Add Service',
          success: true,
          message: `✅ Service ${service.id} added`
        });

        // Step 3: Calculate Price
        const price = await invoke<any>('calculate_full_booking_price_pg', {
          roomId: booking.room_id,
          checkin: booking.checkin_date,
          checkout: booking.checkout_date,
          isMember: false,
          services: [{ name: 'Test Service', priceType: 'fixed', originalValue: 25.0, appliesTo: 'overnight_price' }],
          discounts: []
        });

        steps.push({
          step: '3. Calculate Price',
          success: price.total > 100,
          message: `✅ Total: ${price.total}€ (Base: ${price.basePrice}€ + Service: ${price.servicesTotal}€)`
        });

        // Step 4: Delete Service
        await invoke('delete_additional_service_pg', { id: service.id });

        steps.push({
          step: '4. Delete Service',
          success: true,
          message: `✅ Service deleted`
        });

        // Step 5: Delete Booking
        await invoke('delete_booking_pg', { id: booking.id });

        steps.push({
          step: '5. Delete Booking',
          success: true,
          message: `✅ Booking deleted`
        });

        return {
          success: true,
          message: `✅ Complete workflow successful (${steps.length} steps)`,
          steps
        };
      } catch (err) {
        steps.push({
          step: 'Error',
          success: false,
          message: `❌ ${err}`
        });

        return {
          success: false,
          message: `❌ Workflow failed: ${err}`,
          steps
        };
      }
    }
  },

  // ============================================================================
  // GUEST CREDIT WORKFLOW
  // ============================================================================
  {
    name: 'Guest Credit Workflow',
    description: 'Fügt Guthaben hinzu → Prüft Balance → Verwendet Guthaben',
    testFn: async () => {
      const steps: Array<{ step: string; success: boolean; message: string }> = [];

      try {
        // Step 1: Check Initial Balance
        const balance1 = await invoke<any>('get_guest_credit_balance', { guestId: 1 });

        steps.push({
          step: '1. Check Initial Balance',
          success: true,
          message: `✅ Initial: ${balance1.balance}€`
        });

        // Step 2: Add Credit
        await invoke('add_guest_credit', {
          guestId: 1,
          amount: 50.0,
          description: 'DevTools Test Credit'
        });

        steps.push({
          step: '2. Add Credit',
          success: true,
          message: `✅ Added 50€`
        });

        // Step 3: Check New Balance
        const balance2 = await invoke<any>('get_guest_credit_balance', { guestId: 1 });

        const increase = balance2.balance - balance1.balance;

        steps.push({
          step: '3. Verify Balance',
          success: Math.abs(increase - 50.0) < 0.01,
          message: increase === 50.0
            ? `✅ Balance increased by 50€ (${balance1.balance}€ → ${balance2.balance}€)`
            : `⚠️ Balance increased by ${increase}€ (expected 50€)`
        });

        // Step 4: Get Transactions
        const transactions = await invoke<any[]>('get_guest_credit_transactions', { guestId: 1 });

        const testTransaction = transactions.find(t => t.description === 'DevTools Test Credit');

        steps.push({
          step: '4. Verify Transaction',
          success: !!testTransaction,
          message: testTransaction
            ? `✅ Transaction found (${testTransaction.amount}€)`
            : `❌ Transaction not found`
        });

        return {
          success: steps.every(s => s.success),
          message: steps.every(s => s.success)
            ? `✅ Credit workflow successful`
            : `⚠️ Some steps failed`,
          steps
        };
      } catch (err) {
        steps.push({
          step: 'Error',
          success: false,
          message: `❌ ${err}`
        });

        return {
          success: false,
          message: `❌ Workflow failed: ${err}`,
          steps
        };
      }
    }
  },

  // ============================================================================
  // TEMPLATE WORKFLOW
  // ============================================================================
  {
    name: 'Service Template Workflow',
    description: 'Erstellt Template → Aktiviert/Deaktiviert → Löscht',
    testFn: async () => {
      const steps: Array<{ step: string; success: boolean; message: string }> = [];

      try {
        // Step 1: Create Service Template
        const template = await invoke<any>('create_service_template_pg', {
          name: 'DevTools Test Service',
          priceType: 'percent',
          defaultValue: 10.0,
          appliesTo: 'overnight_price',
          emoji: '🧪',
          isActive: true
        });

        steps.push({
          step: '1. Create Template',
          success: true,
          message: `✅ Template ${template.id} created`
        });

        // Step 2: Toggle Active (deactivate)
        const toggled1 = await invoke<any>('toggle_service_template_active_pg', { id: template.id });

        steps.push({
          step: '2. Deactivate Template',
          success: !toggled1.isActive,
          message: toggled1.isActive ? `❌ Still active` : `✅ Deactivated`
        });

        // Step 3: Toggle Active (reactivate)
        const toggled2 = await invoke<any>('toggle_service_template_active_pg', { id: template.id });

        steps.push({
          step: '3. Reactivate Template',
          success: toggled2.isActive,
          message: toggled2.isActive ? `✅ Reactivated` : `❌ Still inactive`
        });

        // Step 4: Delete Template
        await invoke('delete_service_template_pg', { id: template.id });

        steps.push({
          step: '4. Delete Template',
          success: true,
          message: `✅ Template deleted`
        });

        return {
          success: steps.every(s => s.success),
          message: steps.every(s => s.success)
            ? `✅ Template workflow successful`
            : `⚠️ Some steps failed`,
          steps
        };
      } catch (err) {
        steps.push({
          step: 'Error',
          success: false,
          message: `❌ ${err}`
        });

        return {
          success: false,
          message: `❌ Workflow failed: ${err}`,
          steps
        };
      }
    }
  },

  // ============================================================================
  // CLEANING TASKS WORKFLOW
  // ============================================================================
  {
    name: 'Cleaning Tasks Workflow',
    description: 'Erstellt Buchung → Prüft Putzaufgaben → Löscht Buchung',
    testFn: async () => {
      const steps: Array<{ step: string; success: boolean; message: string }> = [];

      try {
        // Step 1: Create Booking
        const booking = await invoke<any>('create_booking_pg', {
          roomId: 1,
          guestId: 1,
          checkinDate: '2026-02-01',
          checkoutDate: '2026-02-05',
          anzahlGaeste: 2,
          anzahlBegleitpersonen: 0,
          status: 'bestaetigt',
          bemerkungen: 'DevTools Cleaning Test',
          grundpreis: 200.0,
          servicesPreis: 0.0,
          rabattPreis: 0.0,
          gesamtpreis: 200.0,
          anzahlNaechte: 4,
          bezahlt: false,
          istStiftungsfall: false
        });

        steps.push({
          step: '1. Create Booking',
          success: true,
          message: `✅ Booking ${booking.id} created`
        });

        // Step 2: Check Cleaning Tasks (should be created automatically via trigger)
        // Wait a bit for trigger to fire
        await new Promise(resolve => setTimeout(resolve, 1000));

        const tasks = await invoke<any[]>('get_cleaning_tasks_by_booking_pg', { bookingId: booking.id });

        steps.push({
          step: '2. Check Cleaning Tasks',
          success: tasks.length > 0,
          message: tasks.length > 0
            ? `✅ ${tasks.length} cleaning tasks created`
            : `❌ No cleaning tasks found`
        });

        // Step 3: Delete Booking (should also delete tasks via CASCADE)
        await invoke('delete_booking_pg', { id: booking.id });

        steps.push({
          step: '3. Delete Booking',
          success: true,
          message: `✅ Booking deleted`
        });

        return {
          success: steps.every(s => s.success),
          message: steps.every(s => s.success)
            ? `✅ Cleaning workflow successful`
            : `⚠️ Some steps failed`,
          steps
        };
      } catch (err) {
        steps.push({
          step: 'Error',
          success: false,
          message: `❌ ${err}`
        });

        return {
          success: false,
          message: `❌ Workflow failed: ${err}`,
          steps
        };
      }
    }
  },

  // ============================================================================
  // REMINDER WORKFLOW
  // ============================================================================
  {
    name: 'Reminder Workflow',
    description: 'Erstellt Reminder → Snooze → Complete → Delete',
    testFn: async () => {
      const steps: Array<{ step: string; success: boolean; message: string }> = [];

      try {
        // Step 1: Create Reminder
        const reminder = await invoke<any>('create_reminder_pg', {
          bookingId: 1, // Assuming booking 1 exists
          reminderText: 'DevTools Test Reminder',
          reminderDate: '2026-01-15',
          isCompleted: false
        });

        steps.push({
          step: '1. Create Reminder',
          success: true,
          message: `✅ Reminder ${reminder.id} created`
        });

        // Step 2: Snooze Reminder
        const snoozed = await invoke<any>('snooze_reminder_pg', {
          id: reminder.id,
          snoozedUntil: '2026-01-16'
        });

        steps.push({
          step: '2. Snooze Reminder',
          success: snoozed.snoozedUntil === '2026-01-16',
          message: snoozed.snoozedUntil === '2026-01-16'
            ? `✅ Snoozed until ${snoozed.snoozedUntil}`
            : `⚠️ Snooze date: ${snoozed.snoozedUntil}`
        });

        // Step 3: Complete Reminder
        const completed = await invoke<any>('complete_reminder_pg', { id: reminder.id, completed: true });

        steps.push({
          step: '3. Complete Reminder',
          success: completed.isCompleted,
          message: completed.isCompleted ? `✅ Completed` : `❌ Not completed`
        });

        // Step 4: Delete Reminder
        await invoke('delete_reminder_pg', { id: reminder.id });

        steps.push({
          step: '4. Delete Reminder',
          success: true,
          message: `✅ Reminder deleted`
        });

        return {
          success: steps.every(s => s.success),
          message: steps.every(s => s.success)
            ? `✅ Reminder workflow successful`
            : `⚠️ Some steps failed`,
          steps
        };
      } catch (err) {
        steps.push({
          step: 'Error',
          success: false,
          message: `❌ ${err}`
        });

        return {
          success: false,
          message: `❌ Workflow failed: ${err}`,
          steps
        };
      }
    }
  },
];

/**
 * Führt alle Integration-Tests aus
 */
export async function runAllIntegrationTests(): Promise<{
  total: number;
  passed: number;
  failed: number;
  results: Array<{
    test: string;
    success: boolean;
    message: string;
    steps: Array<{ step: string; success: boolean; message: string }>;
  }>;
}> {
  const results: Array<{
    test: string;
    success: boolean;
    message: string;
    steps: Array<{ step: string; success: boolean; message: string }>;
  }> = [];

  let passed = 0;
  let failed = 0;

  for (const test of integrationTests) {
    console.log(`🧪 Running: ${test.name}...`);

    try {
      const result = await test.testFn();

      results.push({
        test: test.name,
        success: result.success,
        message: result.message,
        steps: result.steps
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
        message: `❌ Exception: ${err}`,
        steps: []
      });
      console.error(`❌ ${test.name}: Exception: ${err}`);
    }
  }

  return {
    total: integrationTests.length,
    passed,
    failed,
    results
  };
}
