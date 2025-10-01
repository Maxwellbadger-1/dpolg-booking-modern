/**
 * GuestDialog Test Page
 *
 * Simple test page to verify the GuestDialog component functionality.
 * This can be used as a temporary route or integrated into your existing UI.
 *
 * Usage:
 * 1. Import this component into your App.tsx
 * 2. Add a route or render it conditionally
 * 3. Test both Create and Edit modes
 */

import { useState } from 'react';
import GuestDialog from './GuestDialog';
import { UserPlus, Edit, Users } from 'lucide-react';

export default function GuestDialogTestPage() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [guestToEdit, setGuestToEdit] = useState<any>(null);
  const [testLog, setTestLog] = useState<string[]>([]);

  // Mock guest data for testing edit mode
  const mockGuest = {
    id: 1,
    vorname: 'Max',
    nachname: 'Mustermann',
    email: 'max@beispiel.de',
    telefon: '+49 123 456789',
    dpolgMitglied: true,
    strasse: 'Musterstraße 123',
    plz: '12345',
    ort: 'Musterstadt',
    mitgliedsnummer: '87654321',
    notizen: 'Test-Notiz für Entwicklungszwecke',
  };

  const addLog = (message: string) => {
    const timestamp = new Date().toLocaleTimeString('de-DE');
    setTestLog(prev => [`[${timestamp}] ${message}`, ...prev]);
  };

  const handleOpenCreate = () => {
    addLog('Opening dialog in CREATE mode');
    setGuestToEdit(null);
    setIsDialogOpen(true);
  };

  const handleOpenEdit = () => {
    addLog('Opening dialog in EDIT mode with mock guest data');
    setGuestToEdit(mockGuest);
    setIsDialogOpen(true);
  };

  const handleClose = () => {
    addLog('Dialog closed');
    setIsDialogOpen(false);
    setGuestToEdit(null);
  };

  const handleSuccess = () => {
    addLog('SUCCESS: Guest saved successfully!');
    // In a real app, you would refresh your guest list here
  };

  const clearLog = () => {
    setTestLog([]);
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 p-8">
      {/* Header */}
      <div className="max-w-4xl mx-auto mb-8">
        <div className="bg-gradient-to-r from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 border border-slate-700">
          <div className="flex items-center gap-4 mb-4">
            <div className="bg-gradient-to-br from-blue-500 to-blue-600 p-3 rounded-2xl">
              <Users className="w-8 h-8 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-white">
                GuestDialog Test Page
              </h1>
              <p className="text-slate-300 mt-1">
                Phase 3.1a - Component Testing
              </p>
            </div>
          </div>

          <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4 mt-4">
            <p className="text-blue-200 text-sm">
              <strong>Testing Instructions:</strong><br />
              1. Click "Create New Guest" to test creation mode<br />
              2. Click "Edit Mock Guest" to test edit mode<br />
              3. Fill in the form and submit<br />
              4. Check the event log below for callback confirmations
            </p>
          </div>
        </div>
      </div>

      {/* Test Buttons */}
      <div className="max-w-4xl mx-auto mb-8">
        <div className="bg-white rounded-xl shadow-lg p-6 border border-slate-200">
          <h2 className="text-lg font-bold text-slate-800 mb-4">
            Test Actions
          </h2>

          <div className="flex flex-wrap gap-4">
            <button
              onClick={handleOpenCreate}
              className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white rounded-lg font-medium transition-all shadow-lg hover:shadow-xl"
            >
              <UserPlus className="w-5 h-5" />
              Create New Guest
            </button>

            <button
              onClick={handleOpenEdit}
              className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-600 hover:to-emerald-700 text-white rounded-lg font-medium transition-all shadow-lg hover:shadow-xl"
            >
              <Edit className="w-5 h-5" />
              Edit Mock Guest
            </button>
          </div>
        </div>
      </div>

      {/* Event Log */}
      <div className="max-w-4xl mx-auto">
        <div className="bg-white rounded-xl shadow-lg p-6 border border-slate-200">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-bold text-slate-800">
              Event Log
            </h2>
            <button
              onClick={clearLog}
              className="px-4 py-2 text-sm text-slate-600 hover:text-slate-800 hover:bg-slate-100 rounded-lg transition-all"
            >
              Clear Log
            </button>
          </div>

          <div className="bg-slate-50 rounded-lg p-4 font-mono text-sm max-h-96 overflow-y-auto">
            {testLog.length === 0 ? (
              <p className="text-slate-400 text-center py-8">
                No events yet. Click a button above to start testing.
              </p>
            ) : (
              <div className="space-y-1">
                {testLog.map((log, index) => (
                  <div
                    key={index}
                    className="text-slate-700 hover:bg-slate-100 px-2 py-1 rounded transition-colors"
                  >
                    {log}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Mock Guest Data Display */}
      <div className="max-w-4xl mx-auto mt-8">
        <div className="bg-white rounded-xl shadow-lg p-6 border border-slate-200">
          <h2 className="text-lg font-bold text-slate-800 mb-4">
            Mock Guest Data (for Edit mode)
          </h2>

          <div className="bg-slate-50 rounded-lg p-4">
            <pre className="text-sm text-slate-700 overflow-x-auto">
              {JSON.stringify(mockGuest, null, 2)}
            </pre>
          </div>

          <p className="text-sm text-slate-500 mt-3">
            This mock data is used when you click "Edit Mock Guest". You can modify
            this data in the component source to test different scenarios.
          </p>
        </div>
      </div>

      {/* Testing Checklist */}
      <div className="max-w-4xl mx-auto mt-8">
        <div className="bg-white rounded-xl shadow-lg p-6 border border-slate-200">
          <h2 className="text-lg font-bold text-slate-800 mb-4">
            Testing Checklist
          </h2>

          <div className="space-y-2">
            {[
              'Dialog opens in create mode with empty form',
              'Dialog opens in edit mode with pre-filled data',
              'Required fields show validation errors when empty',
              'Email validation triggers on blur',
              'Invalid email shows error message',
              'DPolG Stiftung Mitglied checkbox shows/hides Mitgliedsnummer field',
              'Submit button shows loading state during save',
              'Success toast appears after successful save',
              'Error toast appears on failure',
              'Dialog closes after successful save',
              'onSuccess callback is invoked',
              'Form resets after close',
            ].map((item, index) => (
              <label key={index} className="flex items-start gap-3 group cursor-pointer">
                <input
                  type="checkbox"
                  className="mt-1 w-4 h-4 text-blue-600 rounded border-slate-300 focus:ring-2 focus:ring-blue-500"
                />
                <span className="text-slate-700 group-hover:text-slate-900 transition-colors">
                  {item}
                </span>
              </label>
            ))}
          </div>
        </div>
      </div>

      {/* Expected Backend Commands */}
      <div className="max-w-4xl mx-auto mt-8 mb-8">
        <div className="bg-gradient-to-r from-purple-50 to-pink-50 rounded-xl shadow-lg p-6 border border-purple-200">
          <h2 className="text-lg font-bold text-purple-900 mb-4">
            Expected Backend Commands
          </h2>

          <div className="space-y-3 text-sm">
            <div className="bg-white rounded-lg p-3 border border-purple-200">
              <code className="text-purple-700 font-mono">
                validate_email_command(email: String)
              </code>
              <p className="text-slate-600 mt-1">
                Called on email field blur and before submit
              </p>
            </div>

            <div className="bg-white rounded-lg p-3 border border-purple-200">
              <code className="text-purple-700 font-mono">
                create_guest_command(...)
              </code>
              <p className="text-slate-600 mt-1">
                Called on submit in create mode
              </p>
            </div>

            <div className="bg-white rounded-lg p-3 border border-purple-200">
              <code className="text-purple-700 font-mono">
                update_guest_command(id, ...)
              </code>
              <p className="text-slate-600 mt-1">
                Called on submit in edit mode
              </p>
            </div>
          </div>

          <p className="text-purple-700 text-sm mt-4">
            <strong>Note:</strong> Make sure your Tauri backend is running to test
            the actual API calls. Check the browser console and Tauri logs for
            detailed output.
          </p>
        </div>
      </div>

      {/* GuestDialog Component */}
      <GuestDialog
        isOpen={isDialogOpen}
        onClose={handleClose}
        onSuccess={handleSuccess}
        guestToEdit={guestToEdit}
      />
    </div>
  );
}