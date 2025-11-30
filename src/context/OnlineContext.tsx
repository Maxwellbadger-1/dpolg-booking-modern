import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface OnlineContextType {
  isOnline: boolean;
  checkConnection: () => Promise<void>;
  setOffline: () => void;
}

const OnlineContext = createContext<OnlineContextType | undefined>(undefined);

export function OnlineProvider({ children }: { children: ReactNode }) {
  const [isOnline, setIsOnline] = useState(true);

  const checkConnection = async () => {
    try {
      const connected = await invoke<boolean>('check_db_connection');
      setIsOnline(connected);
    } catch (error) {
      console.error('Connection check failed:', error);
      setIsOnline(false);
    }
  };

  const setOffline = () => {
    setIsOnline(false);
  };

  useEffect(() => {
    // Initial check
    checkConnection();

    // Check every 30 seconds
    const interval = setInterval(checkConnection, 30000);

    return () => clearInterval(interval);
  }, []);

  return (
    <OnlineContext.Provider value={{ isOnline, checkConnection, setOffline }}>
      {children}
    </OnlineContext.Provider>
  );
}

export function useOnline() {
  const context = useContext(OnlineContext);
  if (context === undefined) {
    throw new Error('useOnline must be used within OnlineProvider');
  }
  return context;
}
