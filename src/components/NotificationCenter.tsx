import { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Bell,
  X,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Info,
  Shield,
  Trash2,
  Filter,
  Search,
  Check,
  Circle,
} from 'lucide-react';
import { Notification, NotificationType } from '../types/notification';

interface NotificationCenterProps {
  notifications: Notification[];
  onMarkAsRead: (id: string) => void;
  onMarkAsUnread: (id: string) => void;
  onClearAll: () => void;
  unreadCount: number;
}

const iconMap = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
  admin: Shield,
};

const colorMap = {
  success: 'text-green-400',
  error: 'text-red-400',
  warning: 'text-yellow-400',
  info: 'text-blue-400',
  admin: 'text-purple-400',
};

const bgColorMap = {
  success: 'bg-green-500/10 border-green-500/30',
  error: 'bg-red-500/10 border-red-500/30',
  warning: 'bg-yellow-500/10 border-yellow-500/30',
  info: 'bg-blue-500/10 border-blue-500/30',
  admin: 'bg-purple-500/10 border-purple-500/30',
};

function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
  if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  return 'Just now';
}

export function NotificationCenter({
  notifications,
  onMarkAsRead,
  onMarkAsUnread,
  onClearAll,
  unreadCount,
}: NotificationCenterProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [filterType, setFilterType] = useState<NotificationType | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');

  const filteredNotifications = useMemo(() => {
    let filtered = notifications;

    // Filter by type
    if (filterType !== 'all') {
      filtered = filtered.filter((n) => n.type === filterType);
    }

    // Filter by search
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (n) =>
          n.title.toLowerCase().includes(query) ||
          n.message.toLowerCase().includes(query)
      );
    }

    return filtered;
  }, [notifications, filterType, searchQuery]);

  return (
    <>
      {/* Floating Button */}
      <motion.button
        onClick={() => setIsOpen(!isOpen)}
        className="fixed bottom-6 right-6 z-[1400] p-4 rounded-full bg-gradient-to-r from-pink-500 to-purple-500 text-white shadow-2xl hover:shadow-pink-500/50 transition-all"
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.95 }}
        style={{
          boxShadow: '0 0 30px rgba(236, 72, 153, 0.6)',
        }}
      >
        <Bell className="w-6 h-6" />
        {unreadCount > 0 && (
          <motion.span
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            className="absolute -top-1 -right-1 bg-red-500 text-white text-xs font-bold rounded-full w-6 h-6 flex items-center justify-center border-2 border-gray-900"
          >
            {unreadCount > 99 ? '99+' : unreadCount}
          </motion.span>
        )}
      </motion.button>

      {/* Slide-in Panel */}
      <AnimatePresence>
        {isOpen && (
          <>
            {/* Backdrop */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => setIsOpen(false)}
              className="fixed inset-0 bg-black/60 backdrop-blur-sm z-[1450]"
            />

            {/* Panel */}
            <motion.div
              initial={{ x: '100%' }}
              animate={{ x: 0 }}
              exit={{ x: '100%' }}
              transition={{ type: 'spring', damping: 25, stiffness: 200 }}
              className="fixed right-0 top-0 bottom-0 w-full max-w-md bg-gray-900 border-l-2 border-pink-500/30 shadow-2xl z-[1500] flex flex-col"
              style={{
                boxShadow: '-10px 0 50px rgba(236, 72, 153, 0.3)',
              }}
            >
              {/* Header */}
              <div className="p-6 border-b border-gray-800 bg-gradient-to-r from-pink-500/10 to-purple-500/10">
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-2xl font-bold text-white flex items-center gap-2">
                    <Bell className="w-6 h-6 text-pink-400" />
                    Notifications
                  </h2>
                  <button
                    onClick={() => setIsOpen(false)}
                    className="text-gray-400 hover:text-white transition-colors p-2 rounded-lg hover:bg-white/10"
                  >
                    <X className="w-5 h-5" />
                  </button>
                </div>

                {/* Search */}
                <div className="relative mb-3">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
                  <input
                    type="text"
                    placeholder="Search notifications..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full pl-10 pr-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-pink-500/50"
                  />
                </div>

                {/* Filter Buttons */}
                <div className="flex gap-2 flex-wrap">
                  {(['all', 'success', 'error', 'warning', 'info', 'admin'] as const).map(
                    (type) => (
                      <button
                        key={type}
                        onClick={() => setFilterType(type)}
                        className={`px-3 py-1 rounded-full text-xs font-medium transition-all ${
                          filterType === type
                            ? 'bg-pink-500 text-white'
                            : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                        }`}
                      >
                        {type.charAt(0).toUpperCase() + type.slice(1)}
                      </button>
                    )
                  )}
                </div>
              </div>

              {/* Actions Bar */}
              {notifications.length > 0 && (
                <div className="px-6 py-3 border-b border-gray-800 flex items-center justify-between">
                  <span className="text-sm text-gray-400">
                    {filteredNotifications.length} notification
                    {filteredNotifications.length !== 1 ? 's' : ''}
                  </span>
                  <button
                    onClick={onClearAll}
                    className="flex items-center gap-2 text-sm text-red-400 hover:text-red-300 transition-colors"
                  >
                    <Trash2 className="w-4 h-4" />
                    Clear All
                  </button>
                </div>
              )}

              {/* Notifications List */}
              <div className="flex-1 overflow-y-auto p-4 space-y-3">
                {filteredNotifications.length === 0 ? (
                  <div className="flex flex-col items-center justify-center h-full text-gray-500">
                    <Bell className="w-16 h-16 mb-4 opacity-20" />
                    <p className="text-lg">No notifications</p>
                  </div>
                ) : (
                  filteredNotifications.map((notification) => {
                    const Icon = iconMap[notification.type];
                    return (
                      <motion.div
                        key={notification.id}
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        className={`relative p-4 rounded-lg border ${
                          notification.read ? 'opacity-60' : ''
                        } ${bgColorMap[notification.type]} backdrop-blur-sm transition-all hover:scale-[1.02]`}
                      >
                        {/* Unread indicator */}
                        {!notification.read && (
                          <div className="absolute top-2 right-2 w-2 h-2 bg-pink-500 rounded-full animate-pulse" />
                        )}

                        <div className="flex gap-3">
                          <Icon className={`w-5 h-5 mt-0.5 flex-shrink-0 ${colorMap[notification.type]}`} />
                          <div className="flex-1 min-w-0">
                            <h4 className="font-semibold text-white text-sm mb-1">
                              {notification.title}
                            </h4>
                            <p className="text-gray-300 text-xs mb-2 leading-relaxed">
                              {notification.message}
                            </p>
                            <div className="flex items-center justify-between">
                              <span className="text-xs text-gray-500">
                                {formatRelativeTime(notification.timestamp)}
                              </span>
                              <button
                                onClick={() =>
                                  notification.read
                                    ? onMarkAsUnread(notification.id)
                                    : onMarkAsRead(notification.id)
                                }
                                className="text-xs text-pink-400 hover:text-pink-300 transition-colors flex items-center gap-1"
                              >
                                {notification.read ? (
                                  <>
                                    <Circle className="w-3 h-3" />
                                    Mark unread
                                  </>
                                ) : (
                                  <>
                                    <Check className="w-3 h-3" />
                                    Mark read
                                  </>
                                )}
                              </button>
                            </div>
                          </div>
                        </div>
                      </motion.div>
                    );
                  })
                )}
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </>
  );
}
