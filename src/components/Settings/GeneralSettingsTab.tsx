import { useState } from 'react';
import { Building2, Save } from 'lucide-react';

export default function GeneralSettingsTab() {
  const [hotelName, setHotelName] = useState('');
  const [address, setAddress] = useState('');
  const [plz, setPlz] = useState('');
  const [ort, setOrt] = useState('');
  const [telefon, setTelefon] = useState('');
  const [email, setEmail] = useState('');
  const [website, setWebsite] = useState('');
  const [loading, setLoading] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setSuccessMessage(null);

    // TODO: Implementiere Backend-Speicherung
    setTimeout(() => {
      setLoading(false);
      setSuccessMessage('Allgemeine Einstellungen erfolgreich gespeichert!');
      setTimeout(() => setSuccessMessage(null), 3000);
    }, 500);
  };

  return (
    <form onSubmit={handleSave} className="space-y-6">
      <div>
        <h3 className="text-lg font-bold text-white mb-4">Unterkunftsdaten</h3>
        <div className="space-y-4">
          {/* Hotel Name */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              <div className="flex items-center gap-2">
                <Building2 className="w-4 h-4" />
                Name der Unterkunft *
              </div>
            </label>
            <input
              type="text"
              required
              value={hotelName}
              onChange={(e) => setHotelName(e.target.value)}
              placeholder="DPolG Stiftung Gästehaus"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Address */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Straße & Hausnummer *
            </label>
            <input
              type="text"
              required
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              placeholder="Musterstraße 123"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* PLZ & Ort */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                PLZ *
              </label>
              <input
                type="text"
                required
                value={plz}
                onChange={(e) => setPlz(e.target.value)}
                placeholder="12345"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Ort *
              </label>
              <input
                type="text"
                required
                value={ort}
                onChange={(e) => setOrt(e.target.value)}
                placeholder="Musterstadt"
                className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          {/* Telefon */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Telefon
            </label>
            <input
              type="tel"
              value={telefon}
              onChange={(e) => setTelefon(e.target.value)}
              placeholder="+49 123 456789"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Email */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Email
            </label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="info@beispiel.de"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          {/* Website */}
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Website
            </label>
            <input
              type="url"
              value={website}
              onChange={(e) => setWebsite(e.target.value)}
              placeholder="https://www.beispiel.de"
              className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </div>
      </div>

      {/* Success Message */}
      {successMessage && (
        <div className="bg-emerald-500/10 border border-emerald-500/30 rounded-lg p-4">
          <p className="text-sm text-emerald-300">{successMessage}</p>
        </div>
      )}

      {/* Save Button */}
      <div className="flex justify-end pt-4">
        <button
          type="submit"
          disabled={loading}
          className="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 text-white rounded-lg font-semibold transition-colors flex items-center gap-2"
        >
          {loading ? (
            <>
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
              Speichert...
            </>
          ) : (
            <>
              <Save className="w-4 h-4" />
              Speichern
            </>
          )}
        </button>
      </div>
    </form>
  );
}
