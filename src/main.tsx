import React from "react";
import ReactDOM from "react-dom/client";
import { MantineProvider } from "@mantine/core";
import { Notifications } from "@mantine/notifications";
import App from "./App";
import { theme } from "./theme";
import "./index.css";
import '@mantine/core/styles.css';
import '@mantine/notifications/styles.css';
import './i18n';

const isDark = typeof window !== 'undefined' && localStorage.getItem('sr-theme') === 'dark';

// Error Boundary Component
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error?: Error }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Application Error:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-pink-50 to-purple-50 dark:from-gray-900 dark:to-gray-950">
          <div className="text-center p-8 bg-white dark:bg-gray-800 rounded-2xl shadow-2xl border-2 border-pink-200 dark:border-pink-900">
            <div className="text-6xl mb-4">💔</div>
            <h1 className="text-2xl font-bold text-pink-600 dark:text-pink-400 mb-4">
              Oops! Something went wrong
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mb-6">
              {this.state.error?.message || 'An unexpected error occurred'}
            </p>
            <button
              onClick={() => window.location.reload()}
              className="px-6 py-3 bg-gradient-to-r from-pink-500 to-purple-500 text-white rounded-lg font-semibold hover:shadow-lg transition-all"
            >
              Reload Application
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <ErrorBoundary>
    <MantineProvider
      theme={theme}
      defaultColorScheme={isDark ? 'dark' : 'light'}
    >
      <Notifications position="top-right" zIndex={1000} />
      <App />
    </MantineProvider>
  </ErrorBoundary>,
);
