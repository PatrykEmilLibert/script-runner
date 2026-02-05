import { useEffect, useCallback, useState, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Notification, NotificationType, NotificationOptions } from '../types/notification';

const MAX_NOTIFICATIONS = 100;
const CLEANUP_DAYS = 7;
const STORAGE_KEY = 'scriptrunner_notifications';

// Sound effects (you can replace with actual audio files)
const playNotificationSound = (type: NotificationType) => {
  try {
    const audioContext = new AudioContext();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    // Different frequencies for different types
    const frequencies: Record<NotificationType, number> = {
      success: 800,
      error: 400,
      warning: 600,
      info: 500,
      admin: 700,
    };

    oscillator.frequency.value = frequencies[type];
    oscillator.type = 'sine';

    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.3);

    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.3);
  } catch (e) {
    console.warn('Could not play notification sound:', e);
  }
};

export function useNotifications() {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [toasts, setToasts] = useState<Notification[]>([]);

  // Load notifications from localStorage on mount
  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored) as Notification[];
        // Cleanup old notifications
        const cutoff = Date.now() - CLEANUP_DAYS * 24 * 60 * 60 * 1000;
        const cleaned = parsed.filter((n) => n.timestamp > cutoff);
        setNotifications(cleaned);
      }
    } catch (e) {
      console.error('Failed to load notifications:', e);
    }
  }, []);

  // Save notifications to localStorage whenever they change
  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(notifications));
    } catch (e) {
      console.error('Failed to save notifications:', e);
    }
  }, [notifications]);

  // Auto-cleanup old notifications
  useEffect(() => {
    const interval = setInterval(() => {
      const cutoff = Date.now() - CLEANUP_DAYS * 24 * 60 * 60 * 1000;
      setNotifications((prev) => prev.filter((n) => n.timestamp > cutoff));
    }, 60 * 60 * 1000); // Check every hour

    return () => clearInterval(interval);
  }, []);

  // Request notification permission on mount
  useEffect(() => {
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }
  }, []);

  const sendNotification = useCallback(
    async (
      title: string,
      message: string,
      type: NotificationType = 'info',
      options: NotificationOptions = {}
    ) => {
      const notification: Notification = {
        id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        title,
        message,
        type,
        timestamp: Date.now(),
        read: false,
        sound: options.sound ?? true,
      };

      // Add to notifications list
      setNotifications((prev) => {
        const updated = [notification, ...prev];
        // Keep only MAX_NOTIFICATIONS
        return updated.slice(0, MAX_NOTIFICATIONS);
      });

      // Add to toasts for visual display
      setToasts((prev) => [...prev, notification]);

      // Play sound if enabled
      if (notification.sound) {
        playNotificationSound(type);
      }

      // Send desktop notification if enabled
      if (options.desktop !== false) {
        try {
          await invoke('send_desktop_notification', {
            title,
            body: message,
            notificationType: type,
          });
        } catch (e) {
          console.error('Failed to send desktop notification:', e);
          // Fallback to browser notification
          if ('Notification' in window && Notification.permission === 'granted') {
            new Notification(title, { body: message });
          }
        }
      }

      return notification.id;
    },
    []
  );

  const markAsRead = useCallback((id: string) => {
    setNotifications((prev) =>
      prev.map((n) => (n.id === id ? { ...n, read: true } : n))
    );
  }, []);

  const markAsUnread = useCallback((id: string) => {
    setNotifications((prev) =>
      prev.map((n) => (n.id === id ? { ...n, read: false } : n))
    );
  }, []);

  const clearAll = useCallback(() => {
    setNotifications([]);
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const unreadCount = useMemo(
    () => notifications.filter((n) => !n.read).length,
    [notifications]
  );

  // Helper functions for common notification types
  const showSuccess = useCallback((title: string, message: string, options?: NotificationOptions) => {
    return sendNotification(title, message, 'success', options);
  }, [sendNotification]);

  const showError = useCallback((title: string, message: string, options?: NotificationOptions) => {
    return sendNotification(title, message, 'error', options);
  }, [sendNotification]);

  const showWarning = useCallback((title: string, message: string, options?: NotificationOptions) => {
    return sendNotification(title, message, 'warning', options);
  }, [sendNotification]);

  const showInfo = useCallback((title: string, message: string, options?: NotificationOptions) => {
    return sendNotification(title, message, 'info', options);
  }, [sendNotification]);

  return {
    notifications,
    toasts,
    sendNotification,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    markAsRead,
    markAsUnread,
    clearAll,
    removeToast,
    unreadCount,
  };
}
