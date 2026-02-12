import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

interface UserContextType {
  userName: string | null;
  changeUserName: (newName: string) => void;
}

const UserContext = createContext<UserContextType | undefined>(undefined);

export function UserProvider({ children }: { children: ReactNode }) {
  const [userName, setUserName] = useState<string | null>(null);

  useEffect(() => {
    // Load from localStorage (like devtools-enabled pattern)
    let name = localStorage.getItem('audit-user-name');

    // Prompt if missing or empty
    while (!name || name.trim() === '') {
      name = window.prompt('Bitte geben Sie Ihren Namen ein:');
    }

    localStorage.setItem('audit-user-name', name);
    setUserName(name);
  }, []);

  const changeUserName = (newName: string) => {
    if (newName && newName.trim() !== '') {
      localStorage.setItem('audit-user-name', newName);
      setUserName(newName);
    }
  };

  return (
    <UserContext.Provider value={{ userName, changeUserName }}>
      {children}
    </UserContext.Provider>
  );
}

export function useUser() {
  const context = useContext(UserContext);
  if (!context) {
    throw new Error('useUser must be used within UserProvider');
  }
  return context;
}
