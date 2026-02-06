import { useEffect, useState } from 'react';
import { Moon, Sun } from 'lucide-react';
import { useMantineColorScheme } from '@mantine/core';
import { invoke } from '@tauri-apps/api/core';

export default function DarkModeToggle() {
  const [isDark, setIsDark] = useState(false);
  const { setColorScheme, colorScheme } = useMantineColorScheme();

  useEffect(() => {
    // Load from localStorage
    const saved = localStorage.getItem('sr-theme');
    const dark = saved === 'dark';
    setIsDark(dark);
    setColorScheme(dark ? 'dark' : 'light');
    applyTheme(dark);
  }, []);

  const applyTheme = (dark: boolean) => {
    if (dark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  };

  const toggle = async () => {
    const newDark = !isDark;
    setIsDark(newDark);
    setColorScheme(newDark ? 'dark' : 'light');
    localStorage.setItem('sr-theme', newDark ? 'dark' : 'light');
    applyTheme(newDark);

    // Sync with backend
    try {
      invoke('toggle_dark_mode');
    } catch (e) {
      console.error('Failed to sync dark mode:', e);
    }
  };

  return (
    <button
      onClick={toggle}
      className="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
      title={isDark ? 'Jasny tryb' : 'Ciemny tryb'}
    >
      {isDark ? <Sun size={20} /> : <Moon size={20} />}
    </button>
  );
}
