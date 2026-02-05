export type NotificationType = 'success' | 'error' | 'warning' | 'info' | 'admin';

export interface Notification {
  id: string;
  title: string;
  message: string;
  type: NotificationType;
  timestamp: number;
  read: boolean;
  sound?: boolean;
}

export interface NotificationOptions {
  type?: NotificationType;
  sound?: boolean;
  desktop?: boolean;
}
