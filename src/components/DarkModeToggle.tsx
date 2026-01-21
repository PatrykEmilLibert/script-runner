import { useEffect, useState } from 'react';
import { Moon, Sun } from 'lucide-react';

export default function DarkModeToggle() {
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    // Load from localStorage
    const saved = localStorage.getItem('sr-theme');
    const dark = saved === 'dark';
    setIsDark(dark);
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
    localStorage.setItem('sr-theme', newDark ? 'dark' : 'light');
    applyTheme(newDark);

    // Sync with backend
    try {
      const { invoke } = await import('@tauri-apps/api/tauri');
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
