import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X, Building2, Save } from 'lucide-react';
import type { PaymentRecipient } from '../../types/booking';

interface PaymentRecipientDialogProps {
  isOpen: boolean;
  onClose: (saved: boolean) => void;
  recipient?: PaymentRecipient | null;
}

export default function PaymentRecipientDialog({ isOpen, onClose, recipient }: PaymentRecipientDialogProps) {
  const [name, setName] = useState('');
  const [company, setCompany] = useState('');
  const [street, setStreet] = useState('');
  const [plz, setPlz] = useState('');
  const [city, setCity] = useState('');
  const [country, setCountry] = useState('Deutschland');
  const [contactPerson, setContactPerson] = useState('');
  const [notes, setNotes] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (recipient) {
      // Edit mode - populate form with existing data
      setName(recipient.name);
      setCompany(recipient.company || '');
      setStreet(recipient.street || '');
      setPlz(recipient.plz || '');
      setCity(recipient.city || '');
      setCountry(recipient.country || 'Deutschland');
      setContactPerson(recipient.contact_person || '');
      setNotes(recipient.notes || '');
    } else {
      // Create mode - reset form
      setName('');
      setCompany('');
      setStreet('');
      setPlz('');
      setCity('');
      setCountry('Deutschland');
      setContactPerson('');
      setNotes('');
    }
  }, [recipient, isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validation
    if (!name.trim()) {
      alert('Bitte geben Sie einen Namen ein.');
      return;
    }

    setLoading(true);

    try {
      if (recipient) {
        // Update existing recipient
        await invoke<PaymentRecipient>('update_payment_recipient', {
          id: recipient.id,
          name: name.trim(),
          company: company.trim() || null,
          street: street.trim() || null,
          plz: plz.trim() || null,
          city: city.trim() || null,
          country: country.trim() || null,
          contactPerson: contactPerson.trim() || null,
          notes: notes.trim() || null,
        });
      } else {
        // Create new recipient
        await invoke<PaymentRecipient>('create_payment_recipient', {
          name: name.trim(),
          company: company.trim() || null,
          street: street.trim() || null,
          plz: plz.trim() || null,
          city: city.trim() || null,
          country: country.trim() || null,
          contactPerson: contactPerson.trim() || null,
          notes: notes.trim() || null,
        });
      }

      window.dispatchEvent(new CustomEvent('refresh-data'));
      onClose(true); // Signal that save was successful
    } catch (error) {
      console.error('Fehler beim Speichern:', error);
      alert(`Fehler beim Speichern: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    onClose(false);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-blue-600 rounded-lg">
              <Building2 className="w-6 h-6 text-white" />
            </div>
            <h2 className="text-2xl font-bold text-white">
              {recipient ? 'Empfänger bearbeiten' : 'Neuen Empfänger erstellen'}
            </h2>
          </div>
          <button
            onClick={handleCancel}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-slate-300" />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Name (Required) */}
          <div className="space-y-2">
            <label htmlFor="name" className="block text-sm font-medium text-slate-300">
              Name / Organisation *
            </label>
            <input
              id="name"
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="z.B. Polizeidirektion München"
            />
          </div>

          {/* Company (Optional) */}
          <div className="space-y-2">
            <label htmlFor="company" className="block text-sm font-medium text-slate-300">
              Firma / Abteilung
            </label>
            <input
              id="company"
              type="text"
              value={company}
              onChange={(e) => setCompany(e.target.value)}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="z.B. Verwaltungsabteilung"
            />
          </div>

          {/* Address Grid */}
          <div className="grid grid-cols-2 gap-4">
            {/* Street */}
            <div className="col-span-2 space-y-2">
              <label htmlFor="street" className="block text-sm font-medium text-slate-300">
                Straße und Hausnummer
              </label>
              <input
                id="street"
                type="text"
                value={street}
                onChange={(e) => setStreet(e.target.value)}
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="z.B. Musterstraße 123"
              />
            </div>

            {/* PLZ */}
            <div className="space-y-2">
              <label htmlFor="plz" className="block text-sm font-medium text-slate-300">
                PLZ
              </label>
              <input
                id="plz"
                type="text"
                value={plz}
                onChange={(e) => setPlz(e.target.value)}
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="z.B. 80331"
              />
            </div>

            {/* City */}
            <div className="space-y-2">
              <label htmlFor="city" className="block text-sm font-medium text-slate-300">
                Stadt
              </label>
              <input
                id="city"
                type="text"
                value={city}
                onChange={(e) => setCity(e.target.value)}
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="z.B. München"
              />
            </div>
          </div>

          {/* Country */}
          <div className="space-y-2">
            <label htmlFor="country" className="block text-sm font-medium text-slate-300">
              Land
            </label>
            <input
              id="country"
              type="text"
              value={country}
              onChange={(e) => setCountry(e.target.value)}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Deutschland"
            />
          </div>

          {/* Contact Person */}
          <div className="space-y-2">
            <label htmlFor="contactPerson" className="block text-sm font-medium text-slate-300">
              Ansprechpartner
            </label>
            <input
              id="contactPerson"
              type="text"
              value={contactPerson}
              onChange={(e) => setContactPerson(e.target.value)}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="z.B. Max Mustermann"
            />
          </div>

          {/* Notes */}
          <div className="space-y-2">
            <label htmlFor="notes" className="block text-sm font-medium text-slate-300">
              Notizen
            </label>
            <textarea
              id="notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              rows={3}
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
              placeholder="Optionale Notizen oder zusätzliche Informationen..."
            />
          </div>

          {/* Footer Buttons */}
          <div className="flex gap-3 pt-4 border-t border-slate-700">
            <button
              type="button"
              onClick={handleCancel}
              className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
              disabled={loading}
            >
              Abbrechen
            </button>
            <button
              type="submit"
              disabled={loading}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors disabled:bg-blue-800 disabled:cursor-not-allowed"
            >
              {loading ? (
                <>
                  <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
                  <span>Speichert...</span>
                </>
              ) : (
                <>
                  <Save className="w-5 h-5" />
                  <span>Speichern</span>
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
