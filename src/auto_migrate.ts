// Auto-run migrations on app startup
import { invoke } from '@tauri-apps/api/core';

export async function runMigrationsIfNeeded() {
  console.log('🔄 AUTO_MIGRATE: Starting migration check...');

  try {
    // Try to run each migration (they're idempotent - safe to run multiple times)
    console.log('📦 Running Guest Credit migration...');
    await invoke('run_guest_credit_migration');
    console.log('✅ Guest Credit migration done');
  } catch (e) {
    console.log('⚠️ Guest Credit migration skipped or failed:', e);
  }

  try {
    console.log('📦 Running Cleaning Tasks migration...');
    await invoke('run_cleaning_tasks_migration');
    console.log('✅ Cleaning Tasks migration done');
  } catch (e) {
    console.log('⚠️ Cleaning Tasks migration skipped or failed:', e);
  }

  try {
    console.log('📦 Running Email Automation migration...');
    await invoke('run_email_automation_migration');
    console.log('✅ Email Automation migration done');
  } catch (e) {
    console.log('⚠️ Email Automation migration skipped or failed:', e);
  }

  console.log('🎉 Migrations check complete!');
}
