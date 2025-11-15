import { useState } from 'react';
import { Briefcase, Tag } from 'lucide-react';
import ServiceTemplateList from './ServiceTemplateList';
import DiscountTemplateList from './DiscountTemplateList';

export default function TemplatesManagement() {
  const [activeTab, setActiveTab] = useState<'services' | 'discounts'>('services');

  return (
    <div className="h-full overflow-y-auto bg-gradient-to-br from-slate-800 to-slate-900 p-6">
      <div className="space-y-6">
      {/* Tab Navigation */}
      <div className="flex gap-2 border-b border-slate-700">
        <button
          onClick={() => setActiveTab('services')}
          className={`flex items-center gap-2 px-6 py-3 font-semibold border-b-2 transition-colors ${
            activeTab === 'services'
              ? 'border-emerald-500 text-emerald-400'
              : 'border-transparent text-slate-400 hover:text-slate-300'
          }`}
        >
          <Briefcase className="w-5 h-5" />
          Zusatzleistungen
        </button>
        <button
          onClick={() => setActiveTab('discounts')}
          className={`flex items-center gap-2 px-6 py-3 font-semibold border-b-2 transition-colors ${
            activeTab === 'discounts'
              ? 'border-amber-500 text-amber-400'
              : 'border-transparent text-slate-400 hover:text-slate-300'
          }`}
        >
          <Tag className="w-5 h-5" />
          Rabatte
        </button>
      </div>

      {/* Tab Content */}
      <div>
        {activeTab === 'services' ? (
          <ServiceTemplateList />
        ) : (
          <DiscountTemplateList />
        )}
      </div>
      </div>
    </div>
  );
}
