import React from "react";
import ReactDOM from "react-dom/client";
import { MantineProvider } from "@mantine/core";
import App from "./App";
import "./index.css";
import '@mantine/core/styles.css';
import './i18n';

const isDark = typeof window !== 'undefined' && localStorage.getItem('sr-theme') === 'dark';

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MantineProvider
      theme={{
        primaryColor: 'blue',
        colors: {
          blue: [
            '#e7f5ff',
            '#d0ebff',
            '#a5d8ff',
            '#74c0fc',
            '#4dabf7',
            '#339af0',
            '#228be6',
            '#1c7ed6',
            '#1971c2',
            '#0d47a1',
          ],
        },
        fontFamily: 'system-ui, -apple-system, sans-serif',
        headings: { fontFamily: 'system-ui, -apple-system, sans-serif' },
      }}
      defaultColorScheme={isDark ? 'dark' : 'light'}
    >
      <App />
    </MantineProvider>
  </React.StrictMode>,
);
