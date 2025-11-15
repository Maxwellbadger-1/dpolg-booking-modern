// Modern Select Dropdown Styles
// Wiederverwendbare Styles für alle Select-Dropdowns in der App

export const SELECT_STYLES = "px-4 py-2.5 bg-white border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 appearance-none cursor-pointer text-slate-700 font-medium hover:border-slate-400 transition-colors";

export const SELECT_BACKGROUND_STYLE = {
  backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%23475569'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'%3E%3C/path%3E%3C/svg%3E")`,
  backgroundRepeat: 'no-repeat' as const,
  backgroundPosition: 'right 0.75rem center',
  backgroundSize: '1.25rem',
  paddingRight: '2.5rem'
};

export const SELECT_SMALL_STYLES = "px-3 py-2 bg-white border border-slate-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 appearance-none cursor-pointer text-slate-700 text-sm font-medium hover:border-slate-400 transition-colors";

export const SELECT_SMALL_BACKGROUND_STYLE = {
  backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%23475569'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'%3E%3C/path%3E%3C/svg%3E")`,
  backgroundRepeat: 'no-repeat' as const,
  backgroundPosition: 'right 0.5rem center',
  backgroundSize: '1rem',
  paddingRight: '2rem'
};
