import { useState, useCallback, type ReactNode } from 'react';
import { translations, type Lang } from '../i18n/translations';
import { AppContext, type Theme } from './AppContextValue';

export type { Theme } from './AppContextValue';

export function AppProvider({ children }: { children: ReactNode }) {
  const [lang, setLang] = useState<Lang>('zh');
  const [theme, setTheme] = useState<Theme>('auto');

  const toggleTheme = useCallback(() => {
    setTheme(prev => {
      if (prev === 'auto') {
        const isDark = window.matchMedia('(prefers-color-scheme:dark)').matches;
        return isDark ? 'light' : 'dark';
      }
      return prev === 'dark' ? 'light' : 'dark';
    });
  }, []);

  const t = useCallback((key: string): string => {
    return translations[lang][key] ?? key;
  }, [lang]);

  return (
    <AppContext.Provider value={{ lang, theme, setLang, setTheme, toggleTheme, t }}>
      {children}
    </AppContext.Provider>
  );
}
