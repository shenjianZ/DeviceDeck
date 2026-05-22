import { createContext } from 'react';
import type { Lang } from '../i18n/translations';

export type Theme = 'auto' | 'light' | 'dark';

export interface AppState {
  lang: Lang;
  theme: Theme;
  setLang: (lang: Lang) => void;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
  t: (key: string) => string;
}

export const AppContext = createContext<AppState | null>(null);
