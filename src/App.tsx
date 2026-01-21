import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import Dashboard from "./components/Dashboard";
import ScriptList from "./components/ScriptList";
import ScriptExecutor from "./components/ScriptExecutor";
import LogViewer from "./components/LogViewer";
import History from "./components/History";
import { AddScript } from "./components/AddScript";
import { AdminDropzone } from "./components/AdminDropzone";
import GenerateAdminKey from "./components/GenerateAdminKey";
import DarkModeToggle from "./components/DarkModeToggle";
import "./App.css";

export default function App() {
  const { t } = useTranslation();
  const [appBlocked, setAppBlocked] = useState(false);
  const [networkError, setNetworkError] = useState(false);
  const [scripts, setScripts] = useState<string[]>([]);
  const [selectedScript, setSelectedScript] = useState<string | null>(null);
  const [output, setOutput] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"dashboard" | "scripts" | "history" | "logs">("dashboard");
  const [showAddScript, setShowAddScript] = useState(false);
  const [scriptsDir, setScriptsDir] = useState<string>("");
  const [officialDir, setOfficialDir] = useState<string>("");
  const [isAdmin, setIsAdmin] = useState(false);
  const [officialScripts, setOfficialScripts] = useState<string[]>([]);

  useEffect(() => {
    const initApp = async () => {
      try {
        // Check kill switch
        const blocked: boolean = await invoke("check_kill_switch");
        if (blocked) {
          setAppBlocked(true);
          return;
        }

        // Check admin key
        try {
          const admin: boolean = await invoke("check_admin_key");
          setIsAdmin(admin);
          console.log("Admin check result:", admin);
        } catch (adminError) {
          console.error("Admin key check error:", adminError);
          setIsAdmin(false);
        }

        // Resolve scripts directory from backend
        const dir: string = await invoke("get_scripts_dir");
        setScriptsDir(dir);
        setOfficialDir(dir);

        // Sync scripts from GitHub (clone if missing)
        await invoke("sync_scripts");

        // List available scripts
        const scriptList: string[] = await invoke("list_scripts");
        setScripts(scriptList);
        
        // Load local scripts
        await loadLocalScripts();
        await loadOfficialScripts();
      } catch (error) {
        console.error("Initialization error:", error);
        // Check if it's a network error
        if (error && typeof error === 'string' && error.includes('internet')) {
          setNetworkError(true);
        }
      } finally {
        setIsLoading(false);
      }
    };

    initApp();
  }, []);

  const loadLocalScripts = async () => {
    try {
      if (!scriptsDir) return;
      const localScripts: any[] = await invoke("get_local_scripts", { scriptsDir, subdir: "scripts" });
      const names = localScripts.map((s: any) => s.name ?? "");
      setScripts(names);
    } catch (error) {
      console.error("Failed to load local scripts:", error);
    }
  };

  const loadOfficialScripts = async () => {
    try {
      if (!officialDir) return;
      const items: any[] = await invoke("get_local_scripts", { scriptsDir: officialDir, subdir: "official" });
      const names = items.map((s: any) => s.name ?? "");
      setOfficialScripts(names);
    } catch (error) {
      console.error("Failed to load official scripts:", error);
    }
  };

  const handleScriptAdded = async () => {
    await loadLocalScripts();
  };

  const handleOfficialAdded = async () => {
    await loadOfficialScripts();
  };

  if (networkError) {
    return (
      <motion.div className="blocked-screen" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
        <div className="blocked-content">
          <h1>⚠️ {t('messages.errorOccurred')}</h1>
          <p>{t('app.title')}</p>
        </div>
      </motion.div>
    );
  }

  if (appBlocked) {
    return (
      <motion.div className="blocked-screen" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
        <div className="blocked-content">
          <h1>{t('app.title')}</h1>
          <p>{t('messages.invalidAdminKey')}</p>
        </div>
      </motion.div>
    );
  }

  if (isLoading) {
    return (
      <motion.div className="loading-screen" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
        <div className="spinner"></div>
        <p>{t('app.loading')}</p>
      </motion.div>
    );
  }

  // Jeśli brak klucza administratora - pokaż formularz generowania
  if (!isAdmin) {
    return (
      <div className="app-container dark:bg-gray-900 dark:text-white flex items-center justify-center min-h-screen">
        <GenerateAdminKey onGenerated={() => window.location.reload()} />
      </div>
    );
  }

  return (
    <div className="app-container dark:bg-gray-900 dark:text-white">
      <header className="app-header bg-white dark:bg-gray-800 border-b dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div>
            <h1>🚀 {t('app.title')}</h1>
            <p className="text-gray-600 dark:text-gray-400">{t('app.subtitle')}</p>
          </div>
          <DarkModeToggle />
        </div>
      </header>

      <nav className="app-nav flex gap-2 bg-gray-100 dark:bg-gray-800 p-4 border-b dark:border-gray-700">
        <button
          className={`nav-btn px-4 py-2 rounded-lg transition-colors ${
            activeTab === "dashboard"
              ? "bg-blue-600 text-white"
              : "bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600"
          }`}
          onClick={() => setActiveTab("dashboard")}
        >
          {t('nav.dashboard')}
        </button>
        <button
          className={`nav-btn px-4 py-2 rounded-lg transition-colors ${
            activeTab === "scripts"
              ? "bg-blue-600 text-white"
              : "bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600"
          }`}
          onClick={() => setActiveTab("scripts")}
        >
          {t('nav.scripts')}
        </button>
        <button
          className={`nav-btn px-4 py-2 rounded-lg transition-colors ${
            activeTab === "history"
              ? "bg-blue-600 text-white"
              : "bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600"
          }`}
          onClick={() => setActiveTab("history")}
        >
          {t('nav.history')}
        </button>
        <button
          className={`nav-btn px-4 py-2 rounded-lg transition-colors ${
            activeTab === "logs"
              ? "bg-blue-600 text-white"
              : "bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600"
          }`}
          onClick={() => setActiveTab("logs")}
        >
          {t('logs.title')}
        </button>
      </nav>

      <main className="app-main p-4">
        {activeTab === "dashboard" && (
          <Dashboard
            scripts={scripts}
            officialScripts={officialScripts}
            onAddScript={() => isAdmin && setShowAddScript(true)}
            isAdmin={isAdmin}
          />
        )}
        {activeTab === "scripts" && (
          <div className="scripts-section">
            {isAdmin && (
              <div className="mb-4">
                <AdminDropzone onUploaded={handleOfficialAdded} scriptsDirOfficial={officialDir} />
              </div>
            )}
            <ScriptList scripts={scripts} selected={selectedScript} onSelect={setSelectedScript} />
            {selectedScript && (
              <ScriptExecutor script={selectedScript} onOutput={setOutput} />
            )}
          </div>
        )}
        {activeTab === "history" && <History />}
        {activeTab === "logs" && selectedScript && (
          <LogViewer scriptName={selectedScript} />
        )}
      </main>

      {showAddScript && (
        <AddScript
          onScriptAdded={handleScriptAdded}
          onClose={() => setShowAddScript(false)}
          scriptsDir={scriptsDir}
        />
      )}

      {output && (
        <motion.div
          className="output-panel bg-gray-900 text-gray-100 rounded-lg border dark:border-gray-700"
          initial={{ y: 100, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
        >
          <h3 className="font-bold p-4 border-b dark:border-gray-700">{t('dashboard.output')}</h3>
          <pre className="p-4 overflow-auto max-h-60">{output}</pre>
        </motion.div>
      )}
    </div>
  );
}
