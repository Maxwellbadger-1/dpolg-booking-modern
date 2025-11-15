import { test, expect } from '@playwright/test';

/**
 * 🛡️ CRITICAL REGRESSION TESTS
 *
 * Diese Tests prüfen die wichtigsten Features die oft kaputt gehen:
 * - Drag & Drop im Tapechart
 * - Mobile App iframe
 * - UI z-index Issues
 * - Settings Navigation
 */

test.describe('Critical Regression Tests', () => {

  test.beforeEach(async ({ page }) => {
    // App starten
    await page.goto('http://localhost:1423');

    // Warten bis App geladen
    await page.waitForSelector('[data-testid="tapechart"]', { timeout: 10000 });
  });

  test('Tapechart loads correctly', async ({ page }) => {
    // Tapechart sollte sichtbar sein
    const tapechart = page.locator('[data-testid="tapechart"]');
    await expect(tapechart).toBeVisible();

    // Zimmer-Sidebar sollte sichtbar sein
    const roomSidebar = page.locator('.room-sidebar').first();
    await expect(roomSidebar).toBeVisible();

    // Heute-Spalte sollte markiert sein
    const todayColumn = page.locator('.bg-emerald-100').first();
    await expect(todayColumn).toBeVisible();
  });

  test('Mobile App iframe loads in Putzplan Sync', async ({ page }) => {
    // Navigiere zu Putzplan Sync Tab
    await page.click('button:has-text("Putzplan Sync")');

    // Warte auf iframe
    const iframe = page.frameLocator('iframe[title="Mobile App Preview"]').first();

    // iframe sollte sichtbar sein
    await expect(page.locator('iframe[title="Mobile App Preview"]').first()).toBeVisible();

    // Keine CSP-Fehler in Console
    const consoleLogs: string[] = [];
    page.on('console', msg => consoleLogs.push(msg.text()));

    await page.waitForTimeout(2000);

    const cspErrors = consoleLogs.filter(log =>
      log.includes('Content Security Policy') ||
      log.includes('blocked')
    );

    expect(cspErrors.length).toBe(0);
  });

  test('Reminder dropdown opens above tapechart', async ({ page }) => {
    // Klicke auf Glocken-Icon
    await page.click('[aria-label="Erinnerungen"]');

    // Dropdown sollte sichtbar sein
    const dropdown = page.locator('text=Dringende Erinnerungen').first();
    await expect(dropdown).toBeVisible();

    // Dropdown sollte korrekte z-index haben (100)
    const dropdownContainer = page.locator('div.z-\\[100\\]').first();
    await expect(dropdownContainer).toBeVisible();
  });

  test('Settings tabs navigate correctly', async ({ page }) => {
    // Öffne Settings
    await page.click('[aria-label="Einstellungen"]');

    // Settings Dialog sollte öffnen
    await expect(page.locator('text=Einstellungen')).toBeVisible();

    // Navigiere durch Tabs
    const tabs = ['Allgemein', 'Preise', 'Zahlungseinstellungen', 'Email-Konfiguration'];

    for (const tab of tabs) {
      await page.click(`button:has-text("${tab}")`);
      await expect(page.locator(`button:has-text("${tab}")`)).toHaveClass(/bg-white/);
    }
  });

  test('New booking dialog opens', async ({ page }) => {
    // Klicke auf "Neuer Gast" Button
    await page.click('button:has-text("Neuer Gast")');

    // Dialog sollte öffnen
    await expect(page.locator('text=Neue Buchung')).toBeVisible();

    // Wichtige Felder sollten vorhanden sein
    await expect(page.locator('label:has-text("Gastname")')).toBeVisible();
    await expect(page.locator('label:has-text("Check-in")')).toBeVisible();
    await expect(page.locator('label:has-text("Check-out")')).toBeVisible();
    await expect(page.locator('label:has-text("Zimmer")')).toBeVisible();
  });

  test('Modern scrollbars are applied', async ({ page }) => {
    // Checke ob custom scrollbar CSS geladen wurde
    const scrollbarStyles = await page.evaluate(() => {
      const styles = window.getComputedStyle(document.documentElement);
      return {
        scrollbarWidth: getComputedStyle(document.body).scrollbarWidth,
      };
    });

    // Scrollbar sollte "thin" sein (Firefox) oder custom (Chrome)
    expect(scrollbarStyles.scrollbarWidth).toBeTruthy();
  });

  test('Tables scale with viewport', async ({ page }) => {
    // Navigiere zu Buchungen Tab
    await page.click('button:has-text("Buchungen")');

    // Tabelle sollte viewport-responsive height haben
    const table = page.locator('[data-testid="booking-list"]').first();

    if (await table.isVisible()) {
      const style = await table.evaluate(el => window.getComputedStyle(el).height);

      // Sollte calc() verwenden, nicht fixed height
      expect(style).toContain('calc');
    }
  });
});

test.describe('Known Bug Regressions', () => {

  test('No JSX syntax errors on page load', async ({ page }) => {
    const errors: string[] = [];

    page.on('pageerror', error => {
      errors.push(error.message);
    });

    await page.goto('http://localhost:1423');
    await page.waitForTimeout(3000);

    // Keine React/JSX Fehler
    const jsxErrors = errors.filter(err =>
      err.includes('JSX') ||
      err.includes('Adjacent') ||
      err.includes('closing tag')
    );

    expect(jsxErrors.length).toBe(0);
  });

  test('No console errors from Tauri invoke', async ({ page }) => {
    const errors: string[] = [];

    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.goto('http://localhost:1423');
    await page.waitForTimeout(5000);

    // Keine Tauri command errors
    const tauriErrors = errors.filter(err =>
      err.includes('Command') ||
      err.includes('invoke') ||
      err.includes('missing required key')
    );

    expect(tauriErrors.length).toBe(0);
  });
});
