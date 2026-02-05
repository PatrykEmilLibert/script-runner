# 🔔 Advanced Notification System - Quick Start

## System został pomyślnie zaimplementowany!

### 📦 Utworzone Pliki

1. **p:\python_runner_github\script-runner\src\types\notification.ts**
   - Typy TypeScript dla notyfikacji
   - Interface dla Notification i NotificationOptions

2. **p:\python_runner_github\script-runner\src\components\Toast.tsx**
   - Toast notifications (auto-dismiss)
   - Różowe gradienty i animacje
   - Progress bar z hover pause

3. **p:\python_runner_github\script-runner\src\components\NotificationCenter.tsx**
   - Floating button z badge
   - Slide-in panel z prawej
   - Search, filter, mark as read/unread
   - Relative timestamps

4. **p:\python_runner_github\script-runner\src\hooks\useNotifications.ts** (rozbudowany)
   - localStorage persistence
   - Auto-cleanup (7 dni)
   - Desktop notifications
   - Sound effects
   - Max 100 notyfikacji

5. **p:\python_runner_github\script-runner\src-tauri\src\main.rs** (zaktualizowany)
   - Rust command: `send_desktop_notification`
   - Windows: PowerShell Toast
   - macOS: osascript
   - Linux: notify-send

6. **p:\python_runner_github\script-runner\src\components\NotificationDemo.tsx**
   - Komponent demo do testowania

7. **p:\python_runner_github\script-runner\NOTIFICATION_SYSTEM_EXAMPLES.md**
   - Pełna dokumentacja z przykładami

---

## 🚀 Jak Używać

### Krok 1: Import Hook

```typescript
import { useNotifications } from './hooks/useNotifications';
```

### Krok 2: Użyj w Komponencie

```typescript
function MyComponent() {
  const { sendNotification } = useNotifications();
  
  const handleClick = async () => {
    await sendNotification(
      'Success!',           // Tytuł
      'Operation complete', // Wiadomość
      'success',           // Typ
      { sound: true }      // Opcje
    );
  };
  
  return <button onClick={handleClick}>Click Me</button>;
}
```

---

## 📱 Struktura Notification Object

```typescript
{
  id: "1738737600000-abc123",        // Auto-generated
  title: "Script Completed!",        // Twój tytuł
  message: "Ran in 2.3 seconds",    // Twoja wiadomość
  type: "success",                   // success | error | warning | info | admin
  timestamp: 1738737600000,          // Unix timestamp
  read: false,                       // false dla nowej notyfikacji
  sound: true                        // Czy odtwarzać dźwięk
}
```

---

## 🎨 Typy Notyfikacji

| Type | Kolor | Ikona | Użycie |
|------|-------|-------|---------|
| `success` | Zielono-różowy | ✓ | Sukces, potwierdzenia |
| `error` | Czerwono-różowy | ✗ | Błędy, niepowodzenia |
| `warning` | Żółto-różowy | ⚠ | Ostrzeżenia |
| `info` | Niebiesko-różowy | ℹ | Informacje |
| `admin` | Fioletowo-różowy | 🛡 | Akcje admina |

---

## 💡 Przykłady Użycia w ScriptRunner

### 1. Po Wykonaniu Skryptu

```typescript
const executeScript = async (scriptName: string) => {
  try {
    const result = await invoke('run_script', { scriptName });
    
    await sendNotification(
      'Script Completed! 🎉',
      `${scriptName} finished successfully`,
      'success',
      { sound: true, desktop: true }
    );
  } catch (error) {
    await sendNotification(
      'Script Failed ❌',
      `Error: ${error}`,
      'error'
    );
  }
};
```

### 2. Po Instalacji Dependencies

```typescript
const installPackages = async (packages: string[]) => {
  await sendNotification(
    'Installing Packages',
    `Installing ${packages.length} packages...`,
    'info',
    { sound: false }
  );

  try {
    await install(packages);
    
    await sendNotification(
      'Installation Complete',
      `Successfully installed all packages`,
      'success'
    );
  } catch (error) {
    await sendNotification(
      'Installation Failed',
      'Some packages failed to install',
      'error'
    );
  }
};
```

### 3. Przy Aktualizacji Aplikacji

```typescript
const checkUpdates = async () => {
  const hasUpdate = await invoke('check_for_updates');
  
  if (hasUpdate) {
    await sendNotification(
      'Update Available! 📦',
      'A new version is available for download',
      'info',
      { desktop: true }
    );
  }
};
```

### 4. Low Disk Space Warning

```typescript
const checkDiskSpace = async () => {
  const freeSpace = await getFreeSpace();
  
  if (freeSpace < 1000000000) { // < 1GB
    await sendNotification(
      'Low Disk Space ⚠️',
      'You have less than 1GB free',
      'warning',
      { desktop: true }
    );
  }
};
```

---

## 🎯 API Reference

### sendNotification()

```typescript
await sendNotification(
  title: string,          // Wymagane
  message: string,        // Wymagane
  type?: NotificationType, // Opcjonalne (domyślnie: 'info')
  options?: {
    sound?: boolean,      // Domyślnie: true
    desktop?: boolean     // Domyślnie: true
  }
)
```

### Inne Funkcje

```typescript
const {
  sendNotification,   // Wyślij notyfikację
  markAsRead,        // (id: string) => void
  markAsUnread,      // (id: string) => void
  clearAll,          // Usuń wszystkie
  notifications,     // Notification[] - lista wszystkich
  toasts,           // Notification[] - aktywne toasty
  unreadCount,      // number - nieprzeczytane
} = useNotifications();
```

---

## 🎬 Features

✅ **Toast Notifications** - Auto-dismiss po 4s  
✅ **Notification Center** - Historia wszystkich notyfikacji  
✅ **Desktop Notifications** - Natywne notyfikacje (Windows/macOS/Linux)  
✅ **Sound Effects** - Różne tony dla każdego typu  
✅ **localStorage Persistence** - Notyfikacje zapisywane lokalnie  
✅ **Auto-cleanup** - Starsze niż 7 dni automatycznie usuwane  
✅ **Max 100 Notifications** - Limit zapobiega overflow  
✅ **Search & Filter** - Wyszukiwanie i filtrowanie po typie  
✅ **Mark as Read/Unread** - Zarządzanie statusem  
✅ **Unread Badge** - Różowy badge z licznikiem  
✅ **Framer Motion Animations** - Smooth slide-in/fade  
✅ **Pink Theme** - Różowe gradienty i glow effects  

---

## 🧪 Testowanie

Użyj komponentu demo:

```typescript
import { NotificationDemo } from './components/NotificationDemo';

// W App.tsx lub innym komponencie
<NotificationDemo />
```

---

## 📝 Notatki

- Desktop notifications wymagają uprawnień systemowych
- Sound effects są generowane w czasie rzeczywistym (Web Audio API)
- Notifications są zapisywane w localStorage pod kluczem: `scriptrunner_notifications`
- Rust backend obsługuje desktop notifications natywnie

---

**System gotowy do użycia!** 🚀
