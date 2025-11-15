import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright Config für automatisiertes Debugging
 * Optimiert für lokale Tauri App Testing
 */
export default defineConfig({
  testDir: './tests',

  // Parallel execution ausschalten für konsistente Logs
  fullyParallel: false,
  workers: 1,

  // Fehlerbehandlung
  forbidOnly: !!process.env.CI,
  retries: 0, // Keine Retries beim Debugging

  // Reporter für detaillierte Ausgabe
  reporter: [
    ['list'],
    ['html', { open: 'never' }]
  ],

  // Shared settings für alle tests
  use: {
    // Base URL der lokalen Tauri App
    baseURL: 'http://localhost:1420',

    // Screenshots bei Fehlern
    screenshot: 'only-on-failure',

    // Video bei Fehlern
    video: 'retain-on-failure',

    // Langsamer für besseres Debugging
    actionTimeout: 10000,
    navigationTimeout: 30000,
  },

  // Projekte (Browser-Konfigurationen)
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        // Console Logs immer capturen
        launchOptions: {
          args: ['--enable-logging', '--v=1'],
        }
      },
    },
  ],

  // Dev Server (Tauri App muss bereits laufen)
  webServer: undefined, // App läuft bereits via npm run tauri dev
});
