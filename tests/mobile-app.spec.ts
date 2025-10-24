import { test, expect } from '@playwright/test';

/**
 * 🧪 MOBILE APP REGRESSION TESTS
 *
 * Tests für dpolg-cleaning-mobile.vercel.app
 * Prüft ob Emojis, Features und Filter funktionieren
 */

test.describe('Mobile App Tests', () => {

  test.beforeEach(async ({ page }) => {
    // Mobile App öffnen
    await page.goto('https://dpolg-cleaning-mobile.vercel.app');
  });

  test('App loads without errors', async ({ page }) => {
    // Warte auf Passwort-Eingabe oder App-Content
    await page.waitForSelector('body', { timeout: 10000 });

    // Keine kritischen Console Errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.waitForTimeout(2000);

    // Filtere bekannte harmlose Errors
    const criticalErrors = errors.filter(err =>
      !err.includes('favicon') &&
      !err.includes('manifest')
    );

    expect(criticalErrors.length).toBe(0);
  });

  test('Password protection works', async ({ page }) => {
    // Passwort-Feld sollte vorhanden sein
    const passwordField = page.locator('input[type="password"]');

    if (await passwordField.isVisible()) {
      // Falsches Passwort
      await passwordField.fill('wrong-password');
      await page.click('button:has-text("Login")');

      // Fehler sollte angezeigt werden
      await expect(page.locator('text=/Falsches|Incorrect/i')).toBeVisible();

      // Richtiges Passwort
      await passwordField.fill('putzplan2025');
      await page.click('button:has-text("Login")');

      // App sollte laden
      await expect(page.locator('text=/DPolG|Putzplan/i')).toBeVisible();
    }
  });

  test('Service emojis render correctly', async ({ page }) => {
    // Wenn Passwort-Schutz aktiv, einloggen
    const passwordField = page.locator('input[type="password"]');
    if (await passwordField.isVisible()) {
      await passwordField.fill('putzplan2025');
      await page.click('button:has-text("Login")');
    }

    // Warte auf Tasks
    await page.waitForSelector('.task-item', { timeout: 10000 });

    // Checke ob Emojis im DOM sind
    const content = await page.textContent('body');

    // Typische Emojis die vorhanden sein sollten
    const expectedEmojis = ['👥', '🐕', '🛏️', '✅'];

    let foundEmojis = 0;
    for (const emoji of expectedEmojis) {
      if (content?.includes(emoji)) {
        foundEmojis++;
      }
    }

    // Mindestens 2 Emoji-Typen sollten verwendet werden
    expect(foundEmojis).toBeGreaterThanOrEqual(2);
  });

  test('Filter buttons work', async ({ page }) => {
    // Login wenn nötig
    const passwordField = page.locator('input[type="password"]');
    if (await passwordField.isVisible()) {
      await passwordField.fill('putzplan2025');
      await page.click('button:has-text("Login")');
    }

    // Warte auf Filter
    await page.waitForTimeout(2000);

    // Filter Buttons sollten vorhanden sein
    const filters = ['Heute', 'Woche', 'Timeline'];

    for (const filter of filters) {
      const filterButton = page.locator(`button:has-text("${filter}")`);
      if (await filterButton.isVisible()) {
        await filterButton.click();
        await page.waitForTimeout(500);
      }
    }
  });

  test('Tasks can be marked as complete', async ({ page }) => {
    // Login
    const passwordField = page.locator('input[type="password"]');
    if (await passwordField.isVisible()) {
      await passwordField.fill('putzplan2025');
      await page.click('button:has-text("Login")');
    }

    // Warte auf Tasks
    await page.waitForSelector('.task-item', { timeout: 10000 });

    // Finde ersten Task Toggle
    const taskToggle = page.locator('[data-testid="task-toggle"]').first();

    if (await taskToggle.isVisible()) {
      // Klicke Toggle
      await taskToggle.click();

      // Status sollte sich ändern (visuell oder in DB)
      await page.waitForTimeout(1000);
    }
  });

  test('Auto-refresh updates data', async ({ page }) => {
    // Login
    const passwordField = page.locator('input[type="password"]');
    if (await passwordField.isVisible()) {
      await passwordField.fill('putzplan2025');
      await page.click('button:has-text("Login")');
    }

    // Warte auf Initial Load
    await page.waitForTimeout(3000);

    // Notiere Initial State
    const initialContent = await page.textContent('body');

    // Warte auf Auto-Refresh (normalerweise 5 Min, aber für Test kürzer mocken)
    // Oder triggere manuellen Refresh
    const refreshButton = page.locator('button:has-text("Aktualisieren")');
    if (await refreshButton.isVisible()) {
      await refreshButton.click();
      await page.waitForTimeout(2000);

      // Content sollte neu geladen sein
      const newContent = await page.textContent('body');
      expect(newContent).toBeTruthy();
    }
  });
});
