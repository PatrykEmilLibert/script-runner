import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, CheckCircle, XCircle, AlertTriangle, Info, Shield } from 'lucide-react';
import { NotificationType } from '../types/notification';

interface ToastProps {
  id: string;
  title: string;
  message: string;
  type: NotificationType;
  onClose: (id: string) => void;
  duration?: number;
}

const iconMap = {
  success: CheckCircle,
  error: XCircle,
  warning: AlertTriangle,
  info: Info,
  admin: Shield,
};

const colorMap = {
  success: 'from-green-500/20 to-pink-500/20 border-green-500/50',
  error: 'from-red-500/20 to-pink-500/20 border-red-500/50',
  warning: 'from-yellow-500/20 to-pink-500/20 border-yellow-500/50',
  info: 'from-blue-500/20 to-pink-500/20 border-blue-500/50',
  admin: 'from-purple-500/20 to-pink-500/20 border-purple-500/50',
};

const iconColorMap = {
  success: 'text-green-400',
  error: 'text-red-400',
  warning: 'text-yellow-400',
  info: 'text-blue-400',
  admin: 'text-purple-400',
};

const progressColorMap = {
  success: 'bg-gradient-to-r from-green-500 to-pink-500',
  error: 'bg-gradient-to-r from-red-500 to-pink-500',
  warning: 'bg-gradient-to-r from-yellow-500 to-pink-500',
  info: 'bg-gradient-to-r from-blue-500 to-pink-500',
  admin: 'bg-gradient-to-r from-purple-500 to-pink-500',
};

export function Toast({ id, title, message, type, onClose, duration = 4000 }: ToastProps) {
  const [progress, setProgress] = useState(100);
  const [isPaused, setIsPaused] = useState(false);
  const Icon = iconMap[type];

  useEffect(() => {
    if (isPaused) return;

    const interval = setInterval(() => {
      setProgress((prev) => {
        if (prev <= 0) {
          onClose(id);
          return 0;
        }
        return prev - (100 / (duration / 50));
      });
    }, 50);

    return () => clearInterval(interval);
  }, [isPaused, duration, id, onClose]);

  return (
    <motion.div
      initial={{ opacity: 0, x: 300, scale: 0.3 }}
      animate={{ opacity: 1, x: 0, scale: 1 }}
      exit={{ opacity: 0, x: 300, scale: 0.5, transition: { duration: 0.2 } }}
      className={`relative w-80 rounded-lg border-2 backdrop-blur-md shadow-2xl overflow-hidden ${colorMap[type]}`}
      onMouseEnter={() => setIsPaused(true)}
      onMouseLeave={() => setIsPaused(false)}
      style={{
        boxShadow: `0 0 20px rgba(236, 72, 153, 0.3)`,
      }}
    >
      <div className="p-4">
        <div className="flex items-start gap-3">
          <Icon className={`w-6 h-6 mt-0.5 ${iconColorMap[type]}`} />
          <div className="flex-1 min-w-0">
            <h4 className="font-semibold text-white text-sm mb-1">{title}</h4>
            <p className="text-gray-300 text-xs leading-relaxed">{message}</p>
          </div>
          <button
            onClick={() => onClose(id)}
            className="text-gray-400 hover:text-white transition-colors p-1 rounded hover:bg-white/10"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
      
      {/* Progress Bar */}
      <div className="h-1 bg-gray-800/50">
        <motion.div
          className={`h-full ${progressColorMap[type]}`}
          style={{ width: `${progress}%` }}
          initial={{ width: '100%' }}
        />
      </div>
    </motion.div>
  );
}

interface ToastContainerProps {
  toasts: Array<{
    id: string;
    title: string;
    message: string;
    type: NotificationType;
  }>;
  onClose: (id: string) => void;
}

export function ToastContainer({ toasts, onClose }: ToastContainerProps) {
  return (
    <div className="fixed top-4 right-4 z-[9999] flex flex-col gap-3">
      <AnimatePresence mode="popLayout">
        {toasts.map((toast) => (
          <Toast
            key={toast.id}
            id={toast.id}
            title={toast.title}
            message={toast.message}
            type={toast.type}
            onClose={onClose}
          />
        ))}
      </AnimatePresence>
    </div>
  );
}
