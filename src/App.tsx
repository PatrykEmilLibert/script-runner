import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import Dashboard from "./components/Dashboard";
import ScriptList from "./components/ScriptList";
import ScriptExecutor from "./components/ScriptExecutor";
import LogViewer from "./components/LogViewer";
import History from "./components/History";
import SearchBox from "./components/SearchBox";
import { AddScript } from "./components/AddScript";
import { AdminDropzone } from "./components/AdminDropzone";
import GenerateAdminKey from "./components/GenerateAdminKey";
import DarkModeToggle from "./components/DarkModeToggle";
import { useNotifications } from "./hooks/useNotifications";
import "./App.css";

export default function App() {
  const { t } = useTranslation();
  const { sendNotification } = useNotifications();
  const [appBlocked, setAppBlocked] = useState(false);
  const [networkError, setNetworkError] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string>("");
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
  const [scriptSearch, setScriptSearch] = useState("");
  const [viewMode, setViewMode] = useState<"list" | "grid">("list");

  useEffect(() => {
    console.log("[APP] Render - activeTab:", activeTab, "isAdmin:", isAdmin);
  }, [activeTab, isAdmin]);

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
        console.log("Scripts directory:", dir);

        // Sync scripts from GitHub (clone if missing)
        try {
          const syncResult: string = await invoke("sync_scripts");
          console.log("Sync result:", syncResult);
          
          // Check for new scripts notification
          if (syncResult.includes("|new_scripts:")) {
            const parts = syncResult.split("|new_scripts:");
            const newCount = parseInt(parts[1]);
            if (newCount > 0) {
              await sendNotification(
                "New Scripts Available!",
                `${newCount} new script${newCount > 1 ? 's' : ''} added to your library`
              );
            }
          }
        } catch (syncError) {
          console.error("Sync error:", syncError);
          // Don't fail completely - maybe scripts are already present
          setErrorMessage(`Warning: Could not sync scripts from GitHub: ${syncError}`);
        }

        // List available scripts
        const scriptList: string[] = await invoke("list_scripts");
        setScripts(scriptList);
        console.log("Found scripts:", scriptList);

        // Load local scripts with the resolved path
        await loadLocalScripts(dir);
        await loadOfficialScripts(dir);
      } catch (error) {
        console.error("Initialization error:", error);
        setErrorMessage(String(error));
        // Check if it's a network error
        if (error && typeof error === 'string' && (error.includes('internet') || error.includes('network') || error.includes('connection'))) {
          setNetworkError(true);
        }
      } finally {
        setIsLoading(false);
      }
    };

    initApp();
  }, []);

  const loadLocalScripts = async (baseDir?: string) => {
    try {
      const dir = baseDir ?? scriptsDir;
      if (!dir) return;
      const localScripts: any[] = await invoke("get_local_scripts", { scriptsDir: dir, subdir: "scripts" });
      const names = localScripts.map((s: any) => s.name ?? "");
      setScripts(names);
    } catch (error) {
      console.error("Failed to load local scripts:", error);
    }
  };

  const loadOfficialScripts = async (baseDir?: string) => {
    try {
      const dir = baseDir ?? officialDir;
      if (!dir) return;
      const items: any[] = await invoke("get_local_scripts", { scriptsDir: dir, subdir: "official" });
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

  const deleteScript = async (script: string, folder: "scripts" | "official") => {
    const confirmMsg = t('scripts.deleteConfirm', { defaultValue: `Na pewno usunąć ${script}?` });
    if (!window.confirm(confirmMsg)) return;

    try {
      await invoke("delete_script", { scriptName: script, scriptsDir, subdir: folder });
      if (folder === "official") {
        await loadOfficialScripts();
      } else {
        await loadLocalScripts();
      }
      if (selectedScript === script) {
        setSelectedScript(null);
      }
    } catch (err) {
      const msg = String(err);
      setErrorMessage(msg);
      console.error("Delete failed:", err);
    }
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

  const encryptScript = async (scriptName: string) => {
    const confirmed = window.confirm(
      `Are you sure you want to encrypt "${scriptName}"?\n\nThis will:\n- Make the script unreadable outside the app\n- Prevent editing\n- Delete the original .py file\n\nThis action cannot be undone!`
    );

    if (!confirmed) return;

    try {
      const result: string = await invoke("encrypt_official_script", { scriptName });
      console.log("Encrypt result:", result);
      await loadOfficialScripts(officialDir);
      await sendNotification("Script Encrypted", `${scriptName} is now protected`);
    } catch (error) {
      console.error("Encryption error:", error);
      alert(`Failed to encrypt script: ${error}`);
    }
  };

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

  // Jeśli brak klucza administratora - pokaż app w user mode (read-only)
  if (!isAdmin) {
    return (
      <div className="app-container dark:bg-gray-900 dark:text-white">
        <header className="app-header bg-white dark:bg-gray-800 border-b dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div>
              <h1>🚀 {t('app.title')} <span className="text-sm text-gray-500">(User Mode)</span></h1>
              <p className="text-gray-600 dark:text-gray-400">{t('app.subtitle')}</p>
            </div>
            <DarkModeToggle />
          </div>
        </header>

        <nav className="app-nav bg-white dark:bg-gray-800 border-b dark:border-gray-700 px-4 py-3 flex gap-2 flex-wrap">
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
        </nav>

        <main className="app-main p-4">
          {activeTab === "dashboard" && (
            <Dashboard
              scripts={scripts}
              officialScripts={officialScripts}
              onAddScript={() => setShowAddScript(true)}
              isAdmin={false}
            />
          )}
          {activeTab === "scripts" && (
            <div className="scripts-section space-y-4">
              <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
                <div className="flex-1 md:max-w-md">
                  <SearchBox onSearch={setScriptSearch} />
                </div>
                <button
                  onClick={() => setShowAddScript(true)}
                  className="px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700"
                >
                  {t('scripts.addScript')}
                </button>
              </div>

              <ScriptList
                title={t('scripts.officialScripts')}
                scripts={officialScripts.filter(s => s.toLowerCase().includes(scriptSearch.toLowerCase()))}
                emptyText={t('scripts.noScripts')}
                selected={selectedScript}
                onSelect={setSelectedScript}
                viewMode={viewMode}
              />

              <ScriptList
                title={t('scripts.userScripts')}
                scripts={scripts.filter(s => s.toLowerCase().includes(scriptSearch.toLowerCase()))}
                selected={selectedScript}
                onSelect={setSelectedScript}
                emptyText={t('scripts.noScripts')}
                viewMode={viewMode}
                onDelete={(s) => deleteScript(s, "scripts")}
              />
            </div>
          )}
          {activeTab === "history" && <History />}

          {selectedScript && (
            <motion.div className="mt-6" initial={{ y: 20, opacity: 0 }} animate={{ y: 0, opacity: 1 }}>
              <ScriptExecutor
                script={selectedScript}
                onOutput={(output: string) => {
                  setOutput(output);
                  setActiveTab("scripts");
                }}
              />
            </motion.div>
          )}

          {showAddScript && (
            <AddScript
              onScriptAdded={() => {
                setShowAddScript(false);
                loadLocalScripts(scriptsDir);
              }}
              onClose={() => setShowAddScript(false)}
              scriptsDir={scriptsDir}
            />
          )}
        </main>
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

      {errorMessage && (
        <div className="bg-yellow-100 dark:bg-yellow-900 border-l-4 border-yellow-500 text-yellow-700 dark:text-yellow-200 p-4 mx-4 mt-4 rounded">
          <div className="flex items-start">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-yellow-500" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-3">
              <p className="text-sm font-medium">{errorMessage}</p>
            </div>
            <button 
              onClick={() => setErrorMessage("")}
              className="ml-auto flex-shrink-0 text-yellow-500 hover:text-yellow-600"
            >
              <span className="sr-only">Dismiss</span>
              <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
            </button>
          </div>
        </div>
      )}

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
          <div className="scripts-section space-y-4">
            <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
              <div className="flex items-center gap-2">
                <button
                  className={`px-3 py-2 rounded-lg border dark:border-gray-700 ${
                    viewMode === "list" ? "bg-blue-600 text-white" : "bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-100"
                  }`}
                  onClick={() => setViewMode("list")}
                >
                  {t('scripts.viewList')}
                </button>
                <button
                  className={`px-3 py-2 rounded-lg border dark:border-gray-700 ${
                    viewMode === "grid" ? "bg-blue-600 text-white" : "bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-100"
                  }`}
                  onClick={() => setViewMode("grid")}
                >
                  {t('scripts.viewGrid')}
                </button>
              </div>
              <div className="flex-1 md:max-w-md">
                <SearchBox onSearch={setScriptSearch} />
              </div>
              {isAdmin && (
                <button
                  onClick={() => setShowAddScript(true)}
                  className="px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700"
                >
                  {t('scripts.addScript')}
                </button>
              )}
            </div>

            {isAdmin && (
              <AdminDropzone onUploaded={handleOfficialAdded} scriptsDirOfficial={officialDir} />
            )}

            <ScriptList
              title={t('scripts.officialScripts')}
              scripts={officialScripts.filter(s => s.toLowerCase().includes(scriptSearch.toLowerCase()))}
              emptyText={t('scripts.noScripts')}
              selected={selectedScript}
              onSelect={setSelectedScript}
              onDelete={isAdmin ? (s) => deleteScript(s, "official") : undefined}
              onEncrypt={isAdmin ? encryptScript : undefined}
              viewMode={viewMode}
            />

            <ScriptList
              title={t('scripts.userScripts')}
              scripts={scripts.filter(s => s.toLowerCase().includes(scriptSearch.toLowerCase()))}
              selected={selectedScript}
              onSelect={setSelectedScript}
              onDelete={isAdmin ? (s) => deleteScript(s, "scripts") : undefined}
              emptyText={t('scripts.noScripts')}
              viewMode={viewMode}
            />

            <p className="text-sm text-gray-600 dark:text-gray-400">
              {t('scripts.clickToRun', { defaultValue: 'Kliknij skrypt, aby uruchomić' })}
            </p>

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
