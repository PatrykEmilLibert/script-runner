# System Notyfikacji - Dokumentacja i Przykłady

## 📋 Struktura Notification Object

```typescript
interface Notification {
  id: string;              // Unikalne ID (auto-generated)
  title: string;           // Tytuł notyfikacji
  message: string;         // Treść wiadomości
  type: NotificationType;  // 'success' | 'error' | 'warning' | 'info' | 'admin'
  timestamp: number;       // Unix timestamp
  read: boolean;           // Status przeczytania
  sound?: boolean;         // Czy odtworzyć dźwięk
}
```

## 🎨 Typy Notyfikacji

### 1. **Success** (Sukces)
- Kolor: Zielono-różowy gradient
- Ikona: CheckCircle ✓
- Użycie: Potwierdzenia, zakończone operacje

### 2. **Error** (Błąd)
- Kolor: Czerwono-różowy gradient
- Ikona: XCircle ✗
- Użycie: Błędy, nieudane operacje

### 3. **Warning** (Ostrzeżenie)
- Kolor: Żółto-różowy gradient
- Ikona: AlertTriangle ⚠
- Użycie: Ostrzeżenia, ważne informacje

### 4. **Info** (Informacja)
- Kolor: Niebiesko-różowy gradient
- Ikona: Info ℹ
- Użycie: Ogólne informacje

### 5. **Admin** (Administracyjna)
- Kolor: Fioletowo-różowy gradient
- Ikona: Shield 🛡
- Użycie: Akcje administratora, ważne systemowe

## 🚀 Przykłady Użycia

### Podstawowe Użycie

```typescript
// W komponencie
import { useNotifications } from './hooks/useNotifications';

function MyComponent() {
  const { sendNotification } = useNotifications();

  const handleClick = async () => {
    // Prosta notyfikacja (domyślnie: type='info')
    await sendNotification(
      'Operation Complete',
      'Your task has finished successfully'
    );
  };

  return <button onClick={handleClick}>Run Task</button>;
}
```

### Notyfikacja Sukcesu

```typescript
const handleScriptExecution = async () => {
  try {
    await runScript();
    
    await sendNotification(
      'Script Completed! 🎉',
      'Your script executed successfully in 2.3 seconds',
      'success',
      { sound: true, desktop: true }
    );
  } catch (error) {
    // ...
  }
};
```

### Notyfikacja Błędu

```typescript
const handleScriptExecution = async () => {
  try {
    await runScript();
  } catch (error) {
    await sendNotification(
      'Script Failed ❌',
      `Error: ${error.message}`,
      'error',
      { sound: true, desktop: true }
    );
  }
};
```

### Notyfikacja Ostrzeżenia

```typescript
const checkDiskSpace = async () => {
  const freeSpace = await getFreeSpace();
  
  if (freeSpace < 1000000000) { // < 1GB
    await sendNotification(
      'Low Disk Space ⚠️',
      'You have less than 1GB of free space remaining',
      'warning',
      { sound: true }
    );
  }
};
```

### Notyfikacja Informacyjna

```typescript
const onNewUpdate = async () => {
  await sendNotification(
    'Update Available 📦',
    'Version 2.0.0 is now available for download',
    'info',
    { desktop: false } // Tylko in-app, bez desktop
  );
};
```

### Notyfikacja Administracyjna

```typescript
const onAdminAction = async () => {
  await sendNotification(
    'Admin Access Granted 🔓',
    'You now have full administrative privileges',
    'admin',
    { sound: true, desktop: true }
  );
};
```

## 🎛️ Opcje Notyfikacji

```typescript
interface NotificationOptions {
  type?: NotificationType;  // Domyślnie: 'info'
  sound?: boolean;          // Domyślnie: true
  desktop?: boolean;        // Domyślnie: true
}
```

### Przykład z Opcjami

```typescript
// Tylko toast (bez dźwięku i desktop)
await sendNotification(
  'Background Task',
  'Processing in the background...',
  'info',
  { sound: false, desktop: false }
);

// Z dźwiękiem ale bez desktop
await sendNotification(
  'File Saved',
  'Your changes have been saved',
  'success',
  { sound: true, desktop: false }
);

// Desktop notification bez dźwięku
await sendNotification(
  'Reminder',
  'Don\'t forget to backup your data',
  'warning',
  { sound: false, desktop: true }
);
```

## 📱 Desktop Notifications (Natywne)

Desktop notifications są obsługiwane przez Rust backend i działają na:
- ✅ Windows (PowerShell Toast)
- ✅ macOS (osascript)
- ✅ Linux (notify-send)

```typescript
// Desktop notification jest automatycznie wysyłana
await sendNotification(
  'System Alert',
  'Your computer will restart in 5 minutes',
  'warning'
);
```

## 🔔 Zarządzanie Notyfikacjami

### Mark as Read/Unread

```typescript
const { markAsRead, markAsUnread } = useNotifications();

// Oznacz jako przeczytane
markAsRead('notification-id-123');

// Oznacz jako nieprzeczytane
markAsUnread('notification-id-123');
```

### Clear All

```typescript
const { clearAll } = useNotifications();

// Usuń wszystkie notyfikacje
const handleClearAll = () => {
  if (confirm('Delete all notifications?')) {
    clearAll();
  }
};
```

### Unread Count

```typescript
const { unreadCount } = useNotifications();

// Wyświetl licznik nieprzeczytanych
return (
  <div>
    You have {unreadCount} unread notifications
  </div>
);
```

## 💾 Persistence

Notyfikacje są automatycznie zapisywane w `localStorage`:
- **Klucz**: `scriptrunner_notifications`
- **Max liczba**: 100 notyfikacji
- **Auto-cleanup**: Starsze niż 7 dni są usuwane

## 🎵 Dźwięki

System generuje różne tony dla różnych typów:
- Success: 800 Hz (wysoki, przyjemny)
- Error: 400 Hz (niski, alarmujący)
- Warning: 600 Hz (średni)
- Info: 500 Hz (neutralny)
- Admin: 700 Hz (ważny)

Możesz wyłączyć dźwięk:
```typescript
await sendNotification(
  'Silent Update',
  'App updated silently',
  'success',
  { sound: false }
);
```

## 🎨 UI Components

### Toast (Auto-dismiss)
- Pozycja: Prawy górny róg
- Czas trwania: 4 sekundy
- Hover: Zatrzymuje countdown
- Progress bar: Różowy gradient

### Notification Center (Persistent)
- Przycisk: Prawy dolny róg z różowym glow
- Panel: Slide-in z prawej strony
- Features:
  - Wyszukiwanie
  - Filtrowanie po typie
  - Mark as read/unread
  - Clear all
  - Timestamps relative

## 🔥 Real-time Examples

### Script Execution Workflow

```typescript
const executeScript = async (scriptName: string) => {
  // Start notification
  await sendNotification(
    'Starting Script',
    `Executing ${scriptName}...`,
    'info',
    { desktop: false }
  );

  try {
    const result = await invoke('run_script', { scriptName });
    
    // Success notification
    await sendNotification(
      'Script Completed! 🎉',
      `${scriptName} finished successfully`,
      'success'
    );
  } catch (error) {
    // Error notification
    await sendNotification(
      'Script Failed',
      `${scriptName}: ${error}`,
      'error'
    );
  }
};
```

### File Upload Workflow

```typescript
const uploadFile = async (file: File) => {
  await sendNotification(
    'Uploading File',
    `Uploading ${file.name}...`,
    'info',
    { sound: false, desktop: false }
  );

  try {
    await upload(file);
    
    await sendNotification(
      'Upload Complete',
      `${file.name} uploaded successfully`,
      'success'
    );
  } catch (error) {
    await sendNotification(
      'Upload Failed',
      `Failed to upload ${file.name}`,
      'error'
    );
  }
};
```

### Dependency Installation

```typescript
const installDependencies = async (packages: string[]) => {
  await sendNotification(
    'Installing Packages',
    `Installing ${packages.length} packages...`,
    'info'
  );

  try {
    await install(packages);
    
    await sendNotification(
      'Installation Complete',
      `Successfully installed ${packages.join(', ')}`,
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

## 🎯 Best Practices

1. **Use appropriate types**: Match notification type to action severity
2. **Keep messages short**: Title max 50 chars, message max 100 chars
3. **Use emojis sparingly**: 1-2 per notification max
4. **Disable sound for frequent events**: Set `sound: false` for high-frequency notifications
5. **Desktop only for important**: Use `desktop: true` only for critical notifications
6. **Provide context**: Include relevant details (file name, count, time, etc.)

## 🔧 Customization

### Custom Sound Files (Future)

```typescript
// Możliwość dodania custom audio w przyszłości
await sendNotification(
  'Achievement Unlocked!',
  'You ran 100 scripts!',
  'success',
  { sound: '/sounds/achievement.mp3' } // Custom audio
);
```

### Custom Duration

```typescript
// Toast component przyjmuje duration prop
<Toast duration={6000} /> // 6 sekund zamiast 4
```

---

## 📊 Complete Hook API

```typescript
const {
  notifications,      // Notification[] - wszystkie notyfikacje
  toasts,            // Notification[] - aktywne toasty
  sendNotification,  // Funkcja wysyłania
  markAsRead,        // (id: string) => void
  markAsUnread,      // (id: string) => void
  clearAll,          // () => void
  removeToast,       // (id: string) => void
  unreadCount,       // number - liczba nieprzeczytanych
} = useNotifications();
```

---

**Gotowe do użycia!** 🚀
