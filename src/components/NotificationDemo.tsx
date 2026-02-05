import { useNotifications } from '../hooks/useNotifications';

/**
 * DEMONSTRATION COMPONENT
 * Shows all notification types and features
 */
export function NotificationDemo() {
  const { sendNotification } = useNotifications();

  const demos = [
    {
      title: 'Success Demo',
      action: () =>
        sendNotification(
          'Script Completed! 🎉',
          'Your Python script ran successfully in 2.3 seconds',
          'success',
          { sound: true, desktop: true }
        ),
      color: 'bg-green-500',
    },
    {
      title: 'Error Demo',
      action: () =>
        sendNotification(
          'Script Failed ❌',
          'ImportError: No module named "pandas"',
          'error',
          { sound: true, desktop: true }
        ),
      color: 'bg-red-500',
    },
    {
      title: 'Warning Demo',
      action: () =>
        sendNotification(
          'Low Disk Space ⚠️',
          'You have less than 1GB free space remaining',
          'warning',
          { sound: true }
        ),
      color: 'bg-yellow-500',
    },
    {
      title: 'Info Demo',
      action: () =>
        sendNotification(
          'Update Available 📦',
          'Version 2.0.0 is now available for download',
          'info'
        ),
      color: 'bg-blue-500',
    },
    {
      title: 'Admin Demo',
      action: () =>
        sendNotification(
          'Admin Access Granted 🔓',
          'You now have full administrative privileges',
          'admin',
          { sound: true, desktop: true }
        ),
      color: 'bg-purple-500',
    },
    {
      title: 'Silent Notification',
      action: () =>
        sendNotification(
          'Background Task',
          'Processing data in the background...',
          'info',
          { sound: false, desktop: false }
        ),
      color: 'bg-gray-500',
    },
  ];

  return (
    <div className="p-8 bg-gray-900 min-h-screen">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold text-white mb-4">
          🔔 Notification System Demo
        </h1>
        <p className="text-gray-400 mb-8">
          Click the buttons below to test different notification types
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {demos.map((demo, index) => (
            <button
              key={index}
              onClick={demo.action}
              className={`${demo.color} hover:opacity-80 text-white font-semibold py-4 px-6 rounded-lg shadow-lg transition-all transform hover:scale-105`}
            >
              {demo.title}
            </button>
          ))}
        </div>

        <div className="mt-12 bg-gray-800 rounded-lg p-6">
          <h2 className="text-2xl font-bold text-white mb-4">
            Quick Integration Example
          </h2>
          <pre className="text-sm text-gray-300 overflow-x-auto">
            {`// In your component
import { useNotifications } from './hooks/useNotifications';

function MyComponent() {
  const { sendNotification } = useNotifications();

  const handleAction = async () => {
    try {
      await someOperation();
      
      await sendNotification(
        'Success!',
        'Operation completed',
        'success'
      );
    } catch (error) {
      await sendNotification(
        'Error!',
        error.message,
        'error'
      );
    }
  };

  return <button onClick={handleAction}>Do Something</button>;
}`}
          </pre>
        </div>
      </div>
    </div>
  );
}
