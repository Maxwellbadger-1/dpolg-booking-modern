import { useState } from 'react';
import { X, Mail, FileText, CreditCard, Building2, Bell, HardDrive, DollarSign, Users } from 'lucide-react';
import EmailConfigTab from './EmailConfigTab';
import EmailTemplatesTab from './EmailTemplatesTab';
import PaymentSettingsTab from './PaymentSettingsTab';
import GeneralSettingsTab from './GeneralSettingsTab';
import NotificationsTab from './NotificationsTab';
import BackupTab from './BackupTab';
import PricingSettingsTab from './PricingSettingsTab';
import PaymentRecipientsTab from './PaymentRecipientsTab';

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

type SettingsTab = 'email' | 'templates' | 'payment' | 'payment_recipients' | 'pricing' | 'general' | 'notifications' | 'backup';

export default function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');

  if (!isOpen) return null;

  const tabs = [
    // 🏢 GRUNDEINSTELLUNGEN (meist verwendet)
    { id: 'general' as SettingsTab, label: 'Allgemein', icon: Building2 },
    { id: 'pricing' as SettingsTab, label: 'Preise', icon: DollarSign },

    // 💳 BUSINESS LOGIC (täglich verwendet)
    { id: 'payment' as SettingsTab, label: 'Zahlungseinstellungen', icon: CreditCard },
    { id: 'payment_recipients' as SettingsTab, label: 'Rechnungsempfänger', icon: Users },

    // 📧 KOMMUNIKATION (regelmäßig verwendet)
    { id: 'email' as SettingsTab, label: 'Email-Konfiguration', icon: Mail },
    { id: 'templates' as SettingsTab, label: 'Email-Templates', icon: FileText },

    // 🔧 ERWEITERT (seltener verwendet)
    { id: 'notifications' as SettingsTab, label: 'Benachrichtigungen', icon: Bell },
    { id: 'backup' as SettingsTab, label: 'Backup & Sicherheit', icon: HardDrive },
  ];

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 max-w-5xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-white">Einstellungen</h2>
          <button
            onClick={onClose}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-slate-300" />
          </button>
        </div>

        {/* Tabs Navigation - Professional Grouping */}
        <div className="flex-shrink-0 border-b border-slate-600 mb-6 relative">
          <div className="overflow-x-auto no-scrollbar min-w-0">
            <div className="flex gap-1 pb-px">
              {tabs.map((tab, index) => {
                const Icon = tab.icon;
                // Add visual separation after groups
                const isGroupEnd = index === 1 || index === 3 || index === 5; // After Grundeinstellungen, Business Logic, Kommunikation
                return (
                  <div key={tab.id} className="flex items-center">
                    <button
                      onClick={() => setActiveTab(tab.id)}
                      className={`flex items-center gap-2 px-4 py-3 font-medium transition-colors whitespace-nowrap relative flex-shrink-0 ${
                        activeTab === tab.id
                          ? 'text-blue-400'
                          : 'text-slate-400 hover:text-slate-200'
                      }`}
                    >
                      <Icon className="w-4 h-4" />
                      {tab.label}
                      {activeTab === tab.id && (
                        <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-400"></div>
                      )}
                    </button>
                    {isGroupEnd && (
                      <div className="mx-2 h-6 w-px bg-slate-600"></div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
          {/* Fade-out indicator on the right */}
          <div className="absolute right-0 top-0 bottom-0 w-12 bg-gradient-to-l from-slate-800 to-transparent pointer-events-none"></div>
          {/* Group legend (subtle) */}
          <div className="absolute top-0 right-0 text-xs text-slate-500 bg-slate-800 px-2 py-1 rounded-bl-lg">
            Wichtig → Erweitert
          </div>
        </div>

        {/* Tab Content */}
        <div className="flex-1 overflow-y-auto px-1 min-h-0">
          {activeTab === 'email' && <EmailConfigTab />}
          {activeTab === 'templates' && <EmailTemplatesTab />}
          {activeTab === 'payment' && <PaymentSettingsTab />}
          {activeTab === 'payment_recipients' && <PaymentRecipientsTab />}
          {activeTab === 'pricing' && <PricingSettingsTab />}
          {activeTab === 'general' && <GeneralSettingsTab />}
          {activeTab === 'notifications' && <NotificationsTab />}
          {activeTab === 'backup' && <BackupTab />}
        </div>
        {/* Footer */}
        <div className="flex justify-end gap-3 mt-6 pt-6 border-t border-slate-700">
          <button
            onClick={onClose}
            className="px-6 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
          >
            Schließen
          </button>
        </div>
      </div>
    </div>
  );
}
