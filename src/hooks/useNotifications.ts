import { useEffect, useCallback } from 'react';

export function useNotifications() {
  const sendNotification = useCallback(async (title: string, body: string) => {
    try {
      const { sendNotification } = await import('@tauri-apps/plugin-notification');
      await sendNotification({
        title,
        body,
      });
    } catch (e) {
      console.error('Failed to send notification:', e);
      // Fallback to browser notification
      if ('Notification' in window && Notification.permission === 'granted') {
        new Notification(title, { body });
      }
    }
  }, []);

  useEffect(() => {
    // Request notification permission on mount
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }
  }, []);

  return { sendNotification };
}
