import { test, expect, ConsoleMessage } from '@playwright/test';

/**
 * Automatisiertes Debugging für payment_recipient_id Bug
 *
 * Dieser Test:
 * 1. Öffnet die App
 * 2. Captured ALLE Console Logs
 * 3. Öffnet eine Buchung zum Bearbeiten
 * 4. Wählt einen Payment Recipient
 * 5. Klickt "Aktualisieren"
 * 6. Analysiert die Console Logs um das Problem zu finden
 */

test.describe('Payment Recipient Debugging', () => {
  // Array zum Speichern aller Console Logs
  const consoleLogs: ConsoleMessage[] = [];

  test.beforeEach(async ({ page }) => {
    // Console Log Listener hinzufügen (ALLE Logs capturen)
    page.on('console', msg => {
      consoleLogs.push(msg);
    });

    // Zu App navigieren
    await page.goto('/');

    // Warten bis App geladen ist
    await page.waitForLoadState('networkidle');

    console.log('✅ App geladen, Console Listener aktiv');
  });

  test('Debug payment_recipient_id flow - Update existing booking', async ({ page }) => {
    console.log('\n🚀 Starting automated debugging test...\n');

    // Schritt 0: Stay on Dashboard (default view shows TapeChart with all bookings)
    console.log('📝 Step 0: Waiting for Dashboard data to load...');

    // Wait for rooms to load (check header stats)
    await page.waitForSelector('text=/\\d+ Zimmer/', { timeout: 10000 });
    console.log('✅ Rooms loaded');

    await page.waitForTimeout(2000); // Additional wait for rendering

    // Schritt 1: Find and click on a booking in TapeChart to open BookingSidebar
    console.log('\n📝 Step 1: Looking for booking in TapeChart...');

    // Look for any booking block in TapeChart (they have data-booking-id attribute)
    const bookingBlock = page.locator('[data-booking-id]').first();
    const bookingCount = await bookingBlock.count();

    if (bookingCount === 0) {
      console.log('❌ No bookings found on Dashboard!');
      console.log('');
      console.log('MANUAL ACTION REQUIRED:');
      console.log('1. Open the app');
      console.log('2. Create at least one test booking');
      console.log('3. Re-run this test');
      throw new Error('No bookings available for testing. Please create a booking first.');
    }

    console.log(`✅ Found ${bookingCount} booking(s) in TapeChart`);

    // Click on first booking to open BookingSidebar
    await bookingBlock.click();
    console.log('✅ Clicked on booking to open sidebar');
    await page.waitForTimeout(1000);

    // Schritt 2: Click "Bearbeiten" button in BookingSidebar to enter edit mode
    console.log('\n📝 Step 2: Switching to edit mode in sidebar...');
    const editButton = page.locator('button:has-text("Bearbeiten")').first();
    await editButton.waitFor({ timeout: 5000 });
    await editButton.click();
    console.log('✅ Clicked "Bearbeiten" to enter edit mode');
    await page.waitForTimeout(1000);

    // Warten bis Dialog geöffnet ist
    await page.waitForTimeout(1000);

    // Schritt 3: Payment Recipient Dropdown finden und auswählen
    console.log('\n📝 Step 3: Selecting payment recipient from dropdown...');

    // Finde das Dropdown für Payment Recipient
    // Es sollte ein <select> mit payment_recipient oder ähnlichem sein
    const paymentRecipientDropdown = page.locator('select').filter({
      has: page.locator('option:has-text("Kein externer Empfänger")')
    }).or(
      page.locator('select').filter({
        has: page.locator('option:has-text("Rechnungsempfänger")')
      })
    );

    const dropdownCount = await paymentRecipientDropdown.count();
    console.log(`🔍 Found ${dropdownCount} payment recipient dropdown(s)`);

    if (dropdownCount > 0) {
      // Dropdown gefunden - wähle ersten echten Payment Recipient (nicht "Kein externer")
      await paymentRecipientDropdown.selectOption({ index: 1 }); // Index 1 = erster echter Empfänger
      console.log('✅ Selected payment recipient from dropdown (index 1)');

      // Kurz warten damit onChange-Handler feuern kann
      await page.waitForTimeout(500);
    } else {
      console.log('⚠️ Payment recipient dropdown not found! Dialog might not have this field yet.');
    }

    // Schritt 4: "Aktualisieren" Button klicken
    console.log('\n📝 Step 4: Clicking "Aktualisieren" button...');

    const updateButton = page.locator('button:has-text("Aktualisieren")');
    if (await updateButton.count() > 0) {
      await updateButton.click();
      console.log('✅ Clicked "Aktualisieren" button');

      // Warten bis Netzwerk-Request fertig ist
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
    } else {
      console.log('⚠️ "Aktualisieren" button not found');
    }

    // Schritt 5: Analysiere alle Console Logs
    console.log('\n' + '='.repeat(80));
    console.log('📊 CAPTURED CONSOLE LOGS ANALYSIS');
    console.log('='.repeat(80) + '\n');

    // Filtere relevante Logs
    const relevantLogs = consoleLogs.filter(msg => {
      const text = msg.text();
      return text.includes('payment_recipient') ||
             text.includes('DROPDOWN') ||
             text.includes('handleSubmit') ||
             text.includes('updatePayload') ||
             text.includes('DataContext') ||
             text.includes('🎯') ||
             text.includes('🚀') ||
             text.includes('📤') ||
             text.includes('🔍');
    });

    console.log(`Found ${relevantLogs.length} relevant console logs:\n`);

    relevantLogs.forEach((msg, index) => {
      console.log(`[${index + 1}] [${msg.type()}] ${msg.text()}`);
    });

    // Spezifische Suche nach payment_recipient_id Werten
    console.log('\n' + '='.repeat(80));
    console.log('🔍 PAYMENT_RECIPIENT_ID VALUE TRACKING');
    console.log('='.repeat(80) + '\n');

    const paymentRecipientLogs = consoleLogs.filter(msg =>
      msg.text().toLowerCase().includes('payment_recipient_id')
    );

    if (paymentRecipientLogs.length > 0) {
      paymentRecipientLogs.forEach((msg, index) => {
        console.log(`[${index + 1}] ${msg.text()}`);
      });
    } else {
      console.log('⚠️ NO LOGS FOUND containing "payment_recipient_id"!');
      console.log('This indicates the logging code may not be active (cache issue?)');
    }

    // Screenshot zum Schluss
    await page.screenshot({
      path: 'debug-payment-recipient-final.png',
      fullPage: true
    });
    console.log('\n📸 Screenshot saved: debug-payment-recipient-final.png');

    console.log('\n' + '='.repeat(80));
    console.log('✅ TEST COMPLETED');
    console.log('='.repeat(80));
  });

  test.afterEach(async () => {
    // Logs am Ende nochmal ausgeben
    console.log(`\n📊 Total console messages captured: ${consoleLogs.length}`);
  });
});
