import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { AppShell, Tabs, Alert, Button, Stack, Group } from "@mantine/core";
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
      <AppShell
        header={{ height: 120 }}
        navbar={{ width: 300, breakpoint: "sm", collapsed: { mobile: true } }}
        padding="md"
      >
        <AppShell.Header className="dark:bg-gray-800 border-b dark:border-gray-700">
          <div className="flex items-center justify-between h-full px-4">
            <div>
              <h1>🚀 {t('app.title')} <span className="text-sm text-gray-500">(User Mode)</span></h1>
              <p className="text-gray-600 dark:text-gray-400">{t('app.subtitle')}</p>
            </div>
            <DarkModeToggle />
          </div>
        </AppShell.Header>

        <AppShell.Main>
          <Tabs value={activeTab} onChange={(value) => setActiveTab(value as any)} className="w-full">
            <Tabs.List className="mb-4 flex flex-wrap gap-2">
              <Tabs.Tab value="dashboard">{t('nav.dashboard')}</Tabs.Tab>
              <Tabs.Tab value="scripts">{t('nav.scripts')}</Tabs.Tab>
              <Tabs.Tab value="history">{t('nav.history')}</Tabs.Tab>
            </Tabs.List>

            <Tabs.Panel value="dashboard">
              <Dashboard
                scripts={scripts}
                officialScripts={officialScripts}
                onAddScript={() => setShowAddScript(true)}
                isAdmin={false}
              />
            </Tabs.Panel>

            <Tabs.Panel value="scripts">
              <div className="scripts-section space-y-4">
                <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
                  <div className="flex-1 md:max-w-md">
                    <SearchBox onSearch={setScriptSearch} />
                  </div>
                  <Button
                    onClick={() => setShowAddScript(true)}
                    color="blue"
                  >
                    {t('scripts.addScript')}
                  </Button>
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
            </Tabs.Panel>

            <Tabs.Panel value="history">
              <History />
            </Tabs.Panel>
          </Tabs>

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
        </AppShell.Main>
      </AppShell>
    );
  }

  return (
    <AppShell
      header={{ height: 120 }}
      padding="md"
    >
      <AppShell.Header className="dark:bg-gray-800 border-b dark:border-gray-700">
        <div className="flex items-center justify-between h-full px-4">
          <div>
            <h1>🚀 {t('app.title')}</h1>
            <p className="text-gray-600 dark:text-gray-400">{t('app.subtitle')}</p>
          </div>
          <DarkModeToggle />
        </div>
      </AppShell.Header>

      <AppShell.Main>
        {errorMessage && (
          <Alert 
            title="Error" 
            color="yellow"
            mb="md"
            withCloseButton
            onClose={() => setErrorMessage("")}
          >
            {errorMessage}
          </Alert>
        )}

        <Tabs value={activeTab} onChange={(value) => setActiveTab(value as any)} className="w-full">
          <Tabs.List className="mb-4 flex flex-wrap gap-2">
            <Tabs.Tab value="dashboard">{t('nav.dashboard')}</Tabs.Tab>
            <Tabs.Tab value="scripts">{t('nav.scripts')}</Tabs.Tab>
            <Tabs.Tab value="history">{t('nav.history')}</Tabs.Tab>
            <Tabs.Tab value="logs">{t('logs.title')}</Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel value="dashboard">
            <Dashboard
              scripts={scripts}
              officialScripts={officialScripts}
              onAddScript={() => isAdmin && setShowAddScript(true)}
              isAdmin={isAdmin}
            />
          </Tabs.Panel>

          <Tabs.Panel value="scripts">
            <div className="scripts-section space-y-4">
              <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
                <Group>
                  <Button
                    variant={viewMode === "list" ? "filled" : "default"}
                    color="blue"
                    onClick={() => setViewMode("list")}
                  >
                    {t('scripts.viewList')}
                  </Button>
                  <Button
                    variant={viewMode === "grid" ? "filled" : "default"}
                    color="blue"
                    onClick={() => setViewMode("grid")}
                  >
                    {t('scripts.viewGrid')}
                  </Button>
                </Group>
                <div className="flex-1 md:max-w-md">
                  <SearchBox onSearch={setScriptSearch} />
                </div>
                {isAdmin && (
                  <Button
                    onClick={() => setShowAddScript(true)}
                    color="blue"
                  >
                    {t('scripts.addScript')}
                  </Button>
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
          </Tabs.Panel>

          <Tabs.Panel value="history">
            <History />
          </Tabs.Panel>

          <Tabs.Panel value="logs">
            {selectedScript && (
              <LogViewer scriptName={selectedScript} />
            )}
          </Tabs.Panel>
        </Tabs>

        {showAddScript && (
          <AddScript
            onScriptAdded={handleScriptAdded}
            onClose={() => setShowAddScript(false)}
            scriptsDir={scriptsDir}
          />
        )}

        {output && (
          <motion.div
            className="output-panel bg-gray-900 text-gray-100 rounded-lg border dark:border-gray-700 mt-6"
            initial={{ y: 100, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
          >
            <h3 className="font-bold p-4 border-b dark:border-gray-700">{t('dashboard.output')}</h3>
            <pre className="p-4 overflow-auto max-h-60">{output}</pre>
          </motion.div>
        )}
      </AppShell.Main>
    </AppShell>
  );
}
