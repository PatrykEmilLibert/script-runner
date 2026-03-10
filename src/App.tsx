import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getVersion } from "@tauri-apps/api/app";
import { useTranslation } from "react-i18next";
import { 
  AppShell, 
  Tabs, 
  Alert, 
  Button, 
  Badge, 
  Group, 
  ActionIcon, 
  Indicator,
  Stack,
  Title,
  Text,
  Box,
  Tooltip,
  Container,
  Flex,
  Drawer
} from "@mantine/core";
import { 
  LayoutDashboard as IconDashboard, 
  FileText as IconScript, 
  BarChart3 as IconChartBar, 
  Shield as IconShield, 
  History as IconHistory, 
  FileCode as IconFileText,
  Bell as IconBell,
  Moon as IconMoon,
  Sun as IconSun,
  Command as IconCommand,
  Sparkles as IconSparkles,
  FolderOpen as IconFolderOpen,
  Github as IconGithub,
  Minus as IconWindowMinimize,
  Square as IconWindowMaximize,
  X as IconWindowClose
} from "lucide-react";
import Dashboard from "./components/Dashboard";
import ScriptList from "./components/ScriptList";
import ScriptExecutor from "./components/ScriptExecutor";
import LogViewer from "./components/LogViewer";
import History from "./components/History";
import SearchBox from "./components/SearchBox";
import { AddScript } from "./components/AddScript";
import { AdminDropzone } from "./components/AdminDropzone";
import AdminPanel from "./components/AdminPanel";
import Analytics from "./components/Analytics";
import DarkModeToggle from "./components/DarkModeToggle";
import GitHubLogin from "./components/GitHubLogin";
import { UpdateNotification } from "./components/UpdateNotification";
import { NotificationCenter } from "./components/NotificationCenter";
import { ToastContainer } from "./components/Toast";
import { useNotifications } from "./hooks/useNotifications";
import "./App.css";
import './styles/pink-theme.css';

export default function App() {
  const { t } = useTranslation();
  const {
    sendNotification,
    notifications,
    toasts,
    markAsRead,
    markAsUnread,
    clearAll,
    removeToast,
    unreadCount,
  } = useNotifications();
  const [appBlocked, setAppBlocked] = useState(false);
  const [blockReason, setBlockReason] = useState<string>("");
  const [networkError, setNetworkError] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [scripts, setScripts] = useState<string[]>([]);
  const [selectedScript, setSelectedScript] = useState<string | null>(null);
  const [output, setOutput] = useState("");
  const [runningScripts, setRunningScripts] = useState<Record<string, { output: string; isRunning: boolean }>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"dashboard" | "scripts" | "analytics" | "admin" | "history" | "logs" | "output">("dashboard");
  const [showAddScript, setShowAddScript] = useState(false);
  const [scriptsDir, setScriptsDir] = useState<string>("");
  const [officialDir, setOfficialDir] = useState<string>("");
  const [isAdmin, setIsAdmin] = useState(false);
  const [officialScripts, setOfficialScripts] = useState<string[]>([]);
  const [scriptSearch, setScriptSearch] = useState("");
  const [customScriptNames, setCustomScriptNames] = useState<Record<string, string>>({});
  const [darkMode, setDarkMode] = useState(false);
  const [showNotifications, setShowNotifications] = useState(false);
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [favoritesStorageKey, setFavoritesStorageKey] = useState("favScripts");
  const [favorites, setFavorites] = useState<string[]>([]);
  const [showGitHubAuth, setShowGitHubAuth] = useState(false);
  const [isWindowsFrameless, setIsWindowsFrameless] = useState(false);
  const [isWindowMaximized, setIsWindowMaximized] = useState(false);
  const [appVersion, setAppVersion] = useState<string>("6.0.35");
  const hasBootstrappedRef = useRef(false);

  useEffect(() => {
    getVersion()
      .then(setAppVersion)
      .catch(() => setAppVersion("6.0.35"));
  }, []);

  useEffect(() => {
    const saved = localStorage.getItem(favoritesStorageKey);
    if (!saved) {
      setFavorites([]);
      return;
    }

    try {
      const parsed = JSON.parse(saved);
      setFavorites(Array.isArray(parsed) ? parsed : []);
    } catch (error) {
      console.error("Failed to load favorites:", error);
      setFavorites([]);
    }
  }, [favoritesStorageKey]);

  useEffect(() => {
    console.log("[APP] Render - activeTab:", activeTab, "isAdmin:", isAdmin);
  }, [activeTab, isAdmin]);

  // Dark mode sync
  useEffect(() => {
    const theme = localStorage.getItem('sr-theme');
    setDarkMode(theme === 'dark');
  }, []);

  // Hybrid titlebar: custom (frameless) on Windows, native overlay on macOS
  useEffect(() => {
    const isWindows = navigator.userAgent.toLowerCase().includes("windows");
    if (!isWindows) return;

    const appWindow = getCurrentWindow();

    appWindow
      .setDecorations(false)
      .then(() => setIsWindowsFrameless(true))
      .catch((error) => {
        setIsWindowsFrameless(false);
        console.warn("Failed to disable window decorations on Windows:", error);
      });

    appWindow
      .isMaximized()
      .then(setIsWindowMaximized)
      .catch(() => setIsWindowMaximized(false));

    let unlistenResized: (() => void) | undefined;
    appWindow
      .onResized(async () => {
        try {
          const maximized = await appWindow.isMaximized();
          setIsWindowMaximized(maximized);
        } catch (error) {
          console.warn("Failed to read window maximize state:", error);
        }
      })
      .then((unlisten) => {
        unlistenResized = unlisten;
      })
      .catch((error) => {
        console.warn("Failed to subscribe to resize events:", error);
      });

    return () => {
      unlistenResized?.();
    };
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+K: Command palette
      if (e.ctrlKey && e.key === 'k') {
        e.preventDefault();
        setShowCommandPalette(true);
      }
      // Ctrl+Shift+P: Admin Panel (admin only)
      if (e.ctrlKey && e.shiftKey && e.key === 'P' && isAdmin) {
        e.preventDefault();
        setActiveTab('admin');
      }
      // Ctrl+N: Notifications
      if (e.ctrlKey && e.key === 'n') {
        e.preventDefault();
        setShowNotifications(!showNotifications);
      }
      // Ctrl+D: Toggle dark mode
      if (e.ctrlKey && e.key === 'd') {
        e.preventDefault();
        const newMode = !darkMode;
        setDarkMode(newMode);
        localStorage.setItem('sr-theme', newMode ? 'dark' : 'light');
        // Removed automatic reload - theme will apply on next app restart
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isAdmin, showNotifications, darkMode]);

  // Analytics tracking
  useEffect(() => {
    if (activeTab === 'analytics') {
      console.log('[Analytics] Tab opened');
    }
  }, [activeTab]);

  useEffect(() => {
    // Load custom script names from localStorage
    const saved = localStorage.getItem('customScriptNames');
    if (saved) {
      try {
        setCustomScriptNames(JSON.parse(saved));
      } catch (e) {
        console.error('Failed to load custom script names:', e);
      }
    }
  }, []);

  const updateScriptName = (scriptId: string, customName: string) => {
    const updated = { ...customScriptNames, [scriptId]: customName };
    setCustomScriptNames(updated);
    localStorage.setItem('customScriptNames', JSON.stringify(updated));
  };

  const toggleFavorite = (scriptName: string, isFav: boolean) => {
    setFavorites((prev) => {
      const updated = isFav 
        ? [...prev, scriptName]
        : prev.filter((s) => s !== scriptName);
      localStorage.setItem(favoritesStorageKey, JSON.stringify(updated));
      return updated;
    });
  };

  const enforceKillSwitchGate = async () => {
    try {
      const blocked = await invoke<boolean>("check_kill_switch");
      if (!blocked) {
        setAppBlocked(false);
        return;
      }

      const isGithubAdmin = await invoke<boolean>("check_github_admin_status").catch(() => false);
      if (isGithubAdmin) {
        setAppBlocked(false);
        return;
      }

      const killSwitchConfig: any = await invoke("check_kill_switch_status").catch(() => null);
      const reason =
        killSwitchConfig?.message ||
        killSwitchConfig?.reason ||
        "Application access is currently restricted";

      setBlockReason(reason);
      setAppBlocked(true);
    } catch (error) {
      console.warn("Failed to enforce kill switch gate:", error);
    }
  };

  const handleAuthChange = async (isAdminStatus: boolean) => {
    setIsAdmin(isAdminStatus);
    console.log("Admin status updated:", isAdminStatus);

    if (!isAdminStatus) {
      await enforceKillSwitchGate();
    } else {
      setAppBlocked(false);
    }
  };

  const handleWindowMinimize = async () => {
    try {
      await getCurrentWindow().minimize();
    } catch (error) {
      console.error("Failed to minimize window:", error);
    }
  };

  const handleWindowToggleMaximize = async () => {
    try {
      const appWindow = getCurrentWindow();
      const maximized = await appWindow.isMaximized();
      if (maximized) {
        await appWindow.unmaximize();
        setIsWindowMaximized(false);
      } else {
        await appWindow.maximize();
        setIsWindowMaximized(true);
      }
    } catch (error) {
      console.error("Failed to toggle maximize window:", error);
    }
  };

  const handleWindowClose = async () => {
    try {
      await getCurrentWindow().close();
    } catch (error) {
      console.error("Failed to close window:", error);
    }
  };

  const runScriptDirectly = async (scriptName: string) => {
    // Initialize running state
    setRunningScripts(prev => ({
      ...prev,
      [scriptName]: { output: "Starting script...\n", isRunning: true }
    }));
    setSelectedScript(scriptName);

    try {
      // Check compatibility
      try {
        const issues: string[] = await invoke("check_script_compatibility", { scriptName });
        if (issues.length > 0 && !window.confirm(
          `\u26A0\uFE0F This script contains Windows-specific libraries:\n\n${issues.join("\n")}\n\nContinue anyway?`
        )) {
          setRunningScripts(prev => ({
            ...prev,
            [scriptName]: { output: "Script execution cancelled by user.\n", isRunning: false }
          }));
          return;
        }
      } catch (err) {
        // Proceed if check fails
      }

      const result: string = await invoke("run_script", { scriptName, args: null });
      setRunningScripts(prev => ({
        ...prev,
        [scriptName]: { output: result, isRunning: false }
      }));
      setOutput(result);
      await sendNotification("Script Completed", `${scriptName} finished successfully`, 'success', { sound: false });
    } catch (error) {
      const errorMsg = `Error: ${error}`;
      setRunningScripts(prev => ({
        ...prev,
        [scriptName]: { output: errorMsg, isRunning: false }
      }));
      setOutput(errorMsg);
      await sendNotification("Script Failed", `${scriptName} encountered an error`, 'error', { sound: false });
    }
  };

  const openScriptFolder = async () => {
    try {
      if (selectedScript) {
        const subdir = officialScripts.includes(selectedScript) ? "official" : "scripts";
        const openedPath: string = await invoke("open_script_folder", {
          scriptName: selectedScript,
          subdir,
        });
        await sendNotification("Folder opened", openedPath, "info", { sound: false });
        return;
      }

      const openedRoot: string = await invoke("open_scripts_root_folder");
      await sendNotification("Scripts folder opened", openedRoot, "info", { sound: false });
    } catch (error) {
      setErrorMessage(`Failed to open folder: ${error}`);
    }
  };

  const initializeApp = async (showBlockingLoader = false) => {
    const shouldBlockUi = showBlockingLoader || !hasBootstrappedRef.current;
    if (shouldBlockUi) {
      setIsLoading(true);
    }

    try {
      let killSwitchBlocked = false;
      let killSwitchConfig: any = null;

      // Check kill switch first
      try {
        killSwitchBlocked = await invoke<boolean>("check_kill_switch");
        if (killSwitchBlocked) {
          killSwitchConfig = await invoke("check_kill_switch_status").catch(() => null);
        }
      } catch (ksError) {
        console.warn("Kill switch check failed:", ksError);
      }

      if (killSwitchBlocked) {
        const reason =
          killSwitchConfig?.message ||
          killSwitchConfig?.reason ||
          "Application access is currently restricted";
        setBlockReason(reason);

        let githubAdmin = false;
        try {
          githubAdmin = await invoke<boolean>("check_github_admin_status");
        } catch (adminError) {
          console.error("Strict GitHub admin check error:", adminError);
          githubAdmin = false;
        }

        if (!githubAdmin) {
          setIsAdmin(false);
          setAppBlocked(true);
          return;
        }

        setIsAdmin(true);
        setAppBlocked(false);
      } else {
        setAppBlocked(false);

        // Standard admin status check (GitHub auth)
        try {
          const admin: boolean = await invoke("check_admin_status");
          setIsAdmin(admin);
          console.log("Admin check result:", admin);
        } catch (adminError) {
          console.error("Admin status check error:", adminError);
          setIsAdmin(false);
        }
      }

      // Resolve scripts directory from backend
      const dir: string = await invoke("get_scripts_dir");
      setScriptsDir(dir);
      setOfficialDir(dir);
      console.log("Scripts directory:", dir);

      // Resolve unique app instance id for per-install local storage
      try {
        const instanceId: string = await invoke("get_app_instance_id");
        setFavoritesStorageKey(`favScripts:${instanceId}`);
      } catch (instanceError) {
        console.warn("Failed to resolve app instance id, using default favorites key:", instanceError);
      }

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
              `${newCount} new script${newCount > 1 ? 's' : ''} added to your library`,
              'success',
              { sound: true, desktop: true }
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
      if (shouldBlockUi) {
        setIsLoading(false);
      }
      hasBootstrappedRef.current = true;
    }
  };

  const handleBlockedAuthChange = async (isAdminStatus: boolean) => {
    await handleAuthChange(isAdminStatus);
    if (isAdminStatus) {
      setAppBlocked(false);
      setErrorMessage("");
      await initializeApp(false);
    }
  };

  useEffect(() => {
    initializeApp(true);
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
      
      // Auto-encrypt official scripts if admin
      if (isAdmin) {
        for (const scriptName of names) {
          try {
            // Try to encrypt - backend will handle if already encrypted
            await invoke("encrypt_official_script", { scriptName });
            console.log(`Encrypted: ${scriptName}`);
          } catch (err) {
            // Ignore errors for already encrypted scripts
            if (!String(err).includes("Script not found")) {
              console.log(`Encryption check for ${scriptName}:`, err);
            }
          }
        }
      }
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
    const confirmMsg = t('scripts.deleteConfirm', { defaultValue: `Na pewno usunÄ…Ä‡ ${script}?` });
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
      <div className="flex items-center justify-center min-h-screen bg-gray-900 dark:bg-gray-950">
        <div className="text-center">
          <h1 className="text-4xl font-bold text-white mb-4">{"\u26A0\uFE0F"} {t('messages.errorOccurred')}</h1>
          <p className="text-gray-400">{t('app.title')}</p>
        </div>
      </div>
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
      await sendNotification("Script Encrypted", `${scriptName} is now protected`, 'success');
    } catch (error) {
      console.error("Encryption error:", error);
      await sendNotification("Encryption Failed", String(error), 'error');
      alert(`Failed to encrypt script: ${error}`);
    }
  };

  const githubAuthDrawer = (
    <Drawer
      opened={showGitHubAuth}
      onClose={() => setShowGitHubAuth(false)}
      position="right"
      size="md"
      title="GitHub Login"
      overlayProps={{ opacity: 0.35, blur: 2 }}
    >
      <GitHubLogin onAuthChange={handleAuthChange} />
    </Drawer>
  );

  if (appBlocked) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-gray-900 to-gray-950">
        <div className="w-full max-w-2xl p-8">
          <div className="mb-6 text-center">
            <IconShield size={64} className="mx-auto text-red-500 mb-4" />
            <h1 className="text-3xl font-bold text-white mb-2">Kill Switch Active</h1>
            <p className="text-gray-400">
              Application can be opened only after GitHub admin login.
            </p>
          </div>

          <div className="bg-gray-800 border-2 border-red-500/50 rounded-lg p-6 mb-6">
            <p className="text-gray-300 text-lg">{blockReason}</p>
          </div>

          <div className="bg-white dark:bg-gray-900 rounded-lg p-4 mb-4">
            <GitHubLogin onAuthChange={handleBlockedAuthChange} />
          </div>

          <div className="flex justify-center">
            <Button color="pink" variant="light" onClick={() => initializeApp(false)}>
              Retry Access Check
            </Button>
          </div>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gradient-to-br from-pink-50 to-purple-50 dark:from-gray-900 dark:to-gray-950">
        <div className="text-center">
          <div className="relative">
            <div className="animate-spin rounded-full h-16 w-16 border-4 border-pink-200 border-t-pink-500 mx-auto mb-4"></div>
            <IconSparkles className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-pink-500 animate-pulse" size={24} />
          </div>
          <Text size="sm" c="dimmed" className="animate-pulse">{t('app.loading')}</Text>
          <Text size="xs" c="pink" mt="xs">Initializing pink magic {"\u2728"}</Text>
        </div>
      </div>
    );
  }

  // If GitHub admin permissions are missing, show app in user mode
  if (!isAdmin) {
    return (
      <AppShell
        header={{ height: 70 }}
        padding="md"
        className="pink-theme-app"
      >
        <AppShell.Header className="border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
          <Container fluid className="h-full px-6">
            <Flex justify="space-between" align="center" className="h-full">
              <Group gap="md" data-tauri-drag-region style={{ flex: 1, minWidth: 0 }}>
                <IconSparkles size={28} className="text-pink-500 animate-pulse" />
                <div>
                  <Text size="lg" fw={700} className="text-pink-500">
                    {t('app.title')}
                  </Text>
                  <Badge size="xs" color="gray" variant="light">User Mode</Badge>
                </div>
              </Group>
              
              <Group gap="sm">
                <Tooltip label="GitHub Login">
                  <ActionIcon
                    variant="subtle"
                    color="pink"
                    onClick={() => setShowGitHubAuth(true)}
                  >
                    <IconGithub size={20} />
                  </ActionIcon>
                </Tooltip>

                <Indicator inline label={unreadCount} size={16} color="pink" disabled={unreadCount === 0}>
                  <ActionIcon 
                    variant="subtle" 
                    color="pink"
                    onClick={() => setShowNotifications(!showNotifications)}
                  >
                    <IconBell size={20} />
                  </ActionIcon>
                </Indicator>
                <DarkModeToggle />

                {isWindowsFrameless && (
                  <Group gap={4} ml="xs">
                    <ActionIcon
                      variant="subtle"
                      color="gray"
                      onClick={handleWindowMinimize}
                      aria-label="Minimize"
                    >
                      <IconWindowMinimize size={16} />
                    </ActionIcon>
                    <ActionIcon
                      variant="subtle"
                      color="gray"
                      onClick={handleWindowToggleMaximize}
                      aria-label={isWindowMaximized ? "Restore" : "Maximize"}
                    >
                      <IconWindowMaximize size={14} />
                    </ActionIcon>
                    <ActionIcon
                      variant="subtle"
                      color="red"
                      onClick={handleWindowClose}
                      aria-label="Close"
                    >
                      <IconWindowClose size={16} />
                    </ActionIcon>
                  </Group>
                )}
              </Group>
            </Flex>
          </Container>
        </AppShell.Header>

        <AppShell.Main className="bg-gray-50 dark:bg-gray-900">
          <Container fluid className="px-6">
            <Tabs 
              value={activeTab} 
              onChange={(value) => setActiveTab(value as any)} 
              className="pink-tabs"
              color="pink"
              variant="pills"
            >
              <Tabs.List mb="lg" className="sticky-tabs-nav">
                <Tabs.Tab value="dashboard" leftSection={<IconDashboard size={16} />}>
                  {t('nav.dashboard')}
                </Tabs.Tab>
                <Tabs.Tab value="scripts" leftSection={<IconScript size={16} />}>
                  {t('nav.scripts')}
                </Tabs.Tab>
                <Tabs.Tab value="history" leftSection={<IconHistory size={16} />}>
                  {t('nav.history')}
                </Tabs.Tab>
                <Tabs.Tab 
                  value="output" 
                  leftSection={<IconFileText size={16} />}
                  rightSection={
                    Object.keys(runningScripts).length > 0 && (
                      <Badge size="xs" color="pink" variant="filled">
                        {Object.keys(runningScripts).length}
                      </Badge>
                    )
                  }
                >
                  Script Output
                </Tabs.Tab>
              </Tabs.List>

              <Tabs.Panel value="dashboard">
                <Dashboard
                  scripts={scripts}
                  officialScripts={officialScripts}
                  onAddScript={() => setShowAddScript(true)}
                  isAdmin={false}
                  favorites={favorites}
                  onToggleFavorite={toggleFavorite}
                  onRunScript={runScriptDirectly}
                />
              </Tabs.Panel>

              <Tabs.Panel value="scripts">
                <div className="scripts-section space-y-4">
                  <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
                    <div className="flex-1 md:max-w-md">
                      <SearchBox onSearch={setScriptSearch} />
                    </div>
                    <Group gap="sm">
                      <Button
                        onClick={openScriptFolder}
                        variant="light"
                        color="pink"
                        leftSection={<IconFolderOpen size={16} />}
                      >
                        Open Script Folder
                      </Button>
                      <Button
                        onClick={() => setShowAddScript(true)}
                        color="pink"
                        leftSection={<IconSparkles size={16} />}
                      >
                        {t('scripts.addScript')}
                      </Button>
                    </Group>
                  </div>

                  <ScriptList
                    title={t('scripts.officialScripts')}
                    scripts={officialScripts.filter(s => s.toLowerCase().includes(scriptSearch.toLowerCase()))}
                    emptyText={t('scripts.noScripts')}
                    selected={selectedScript}
                    onSelect={runScriptDirectly}
                    onToggleFavorite={toggleFavorite}
                    favorites={favorites}
                  />

                  <ScriptList
                    title={t('scripts.userScripts')}
                    scripts={scripts.filter(s => s.toLowerCase().includes(scriptSearch.toLowerCase()))}
                    selected={selectedScript}
                    onSelect={runScriptDirectly}
                    emptyText={t('scripts.noScripts')}
                    onDelete={(s) => deleteScript(s, "scripts")}
                    onToggleFavorite={toggleFavorite}
                    favorites={favorites}
                  />
                </div>
              </Tabs.Panel>

              <Tabs.Panel value="history">
                <History />
              </Tabs.Panel>

              {Object.keys(runningScripts).length > 0 ? (
                <Tabs.Panel value="output">
                  <Stack gap="md">
                    <Group justify="space-between">
                      <Title order={3}>Script Outputs</Title>
                      <Badge color="pink" variant="light">
                        {Object.keys(runningScripts).length} script{Object.keys(runningScripts).length > 1 ? 's' : ''}
                      </Badge>
                    </Group>
                    
                    <Tabs defaultValue={selectedScript || Object.keys(runningScripts)[0]} color="pink" variant="pills">
                      <Tabs.List>
                        {Object.keys(runningScripts).map((scriptName) => (
                          <Tabs.Tab 
                            key={scriptName} 
                            value={scriptName}
                            rightSection={
                              runningScripts[scriptName].isRunning && (
                                <div className="animate-spin rounded-full h-3 w-3 border-2 border-pink-200 border-t-pink-500"></div>
                              )
                            }
                          >
                            {scriptName}
                          </Tabs.Tab>
                        ))}
                      </Tabs.List>

                      {Object.entries(runningScripts).map(([scriptName, data]) => (
                        <Tabs.Panel key={scriptName} value={scriptName} pt="md">
                          <div className="bg-white dark:bg-gray-800 rounded-lg border-2 border-pink-200 dark:border-pink-900 pink-glow">
                            <div className="p-4 border-b border-pink-100 dark:border-gray-700">
                              <Group justify="space-between">
                                <div>
                                  <Text fw={700} className="text-pink-600">
                                    {scriptName}
                                  </Text>
                                  <Text size="sm" c="dimmed">
                                    {data.isRunning ? "Running..." : "Completed"}
                                  </Text>
                                </div>
                                <Button
                                  size="xs"
                                  variant="light"
                                  color="pink"
                                  onClick={() => {
                                    setRunningScripts(prev => {
                                      const updated = { ...prev };
                                      delete updated[scriptName];
                                      return updated;
                                    });
                                  }}
                                >
                                  Clear
                                </Button>
                              </Group>
                            </div>
                            <pre className="p-4 overflow-auto max-h-96 text-sm font-mono bg-gray-50 dark:bg-gray-900 white-space-pre-wrap break-words">
                              {data.output}
                            </pre>
                          </div>
                        </Tabs.Panel>
                      ))}
                    </Tabs>
                  </Stack>
                </Tabs.Panel>
              ) : (
                <Tabs.Panel value="output">
                  <Alert color="pink" variant="light">
                    <Text size="sm">No scripts have been run yet. Click a script to execute it and see the output here.</Text>
                  </Alert>
                </Tabs.Panel>
              )}
            </Tabs>

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

            <UpdateNotification />
          </Container>
        </AppShell.Main>

        {/* Notification System */}
        <ToastContainer toasts={toasts} onClose={removeToast} />
        <NotificationCenter
          notifications={notifications}
          onMarkAsRead={markAsRead}
          onMarkAsUnread={markAsUnread}
          onClearAll={clearAll}
          unreadCount={unreadCount}
        />
        {githubAuthDrawer}
      </AppShell>
    );
  }

  return (
    <AppShell
      header={{ height: 70 }}
      padding="md"
      className="pink-theme-app"
    >
      <AppShell.Header className="border-b border-pink-100 dark:border-gray-700 bg-white dark:bg-gray-800 pink-header-glow">
        <Container fluid className="h-full px-6">
          <Flex justify="space-between" align="center" className="h-full">
            <Group gap="md" data-tauri-drag-region style={{ flex: 1, minWidth: 0 }}>
              <IconSparkles size={28} className="text-pink-500 animate-pulse" />
              <div>
                <Text size="lg" fw={700} className="bg-gradient-to-r from-pink-500 to-purple-500 bg-clip-text text-transparent">
                  {t('app.title')}
                </Text>
                <Badge size="xs" color="pink" variant="dot">
                  Admin Mode
                </Badge>
              </div>
            </Group>
            
            <Group gap="sm">
              <Tooltip label="Keyboard Shortcuts (Ctrl+K)">
                <ActionIcon 
                  variant="subtle" 
                  color="pink"
                  onClick={() => setShowCommandPalette(true)}
                >
                  <IconCommand size={20} />
                </ActionIcon>
              </Tooltip>
              
              <Indicator inline label={unreadCount} size={16} color="pink" disabled={unreadCount === 0}>
                <ActionIcon 
                  variant="subtle" 
                  color="pink"
                  onClick={() => setShowNotifications(!showNotifications)}
                  className="pink-pulse"
                >
                  <IconBell size={20} />
                </ActionIcon>
              </Indicator>
              
              <Badge color="pink" variant="light" className="pink-glow">
                <IconShield size={12} className="inline mr-1" />
                Admin
              </Badge>

              <Tooltip label="GitHub Login">
                <ActionIcon
                  variant="subtle"
                  color="pink"
                  onClick={() => setShowGitHubAuth(true)}
                >
                  <IconGithub size={20} />
                </ActionIcon>
              </Tooltip>
              
              <DarkModeToggle />

              {isWindowsFrameless && (
                <Group gap={4} ml="xs">
                  <ActionIcon
                    variant="subtle"
                    color="gray"
                    onClick={handleWindowMinimize}
                    aria-label="Minimize"
                  >
                    <IconWindowMinimize size={16} />
                  </ActionIcon>
                  <ActionIcon
                    variant="subtle"
                    color="gray"
                    onClick={handleWindowToggleMaximize}
                    aria-label={isWindowMaximized ? "Restore" : "Maximize"}
                  >
                    <IconWindowMaximize size={14} />
                  </ActionIcon>
                  <ActionIcon
                    variant="subtle"
                    color="red"
                    onClick={handleWindowClose}
                    aria-label="Close"
                  >
                    <IconWindowClose size={16} />
                  </ActionIcon>
                </Group>
              )}
            </Group>
          </Flex>
        </Container>
      </AppShell.Header>

      <AppShell.Main className="bg-gradient-to-br from-gray-50 to-pink-50/30 dark:from-gray-900 dark:to-gray-950">
        <Container fluid className="px-6">
          {errorMessage && (
            <Alert 
              title="Error" 
              color="pink"
              mb="md"
              withCloseButton
              onClose={() => setErrorMessage("")}
              className="pink-alert"
            >
              {errorMessage}
            </Alert>
          )}

          <Tabs
            value={activeTab} 
            onChange={(value) => setActiveTab(value as any)} 
            className="pink-tabs"
            color="pink"
            variant="pills"
          >
            <Tabs.List mb="lg" className="sticky-tabs-nav bg-white/50 dark:bg-gray-800/50 backdrop-blur-sm rounded-lg p-2">
              <Tabs.Tab 
                value="dashboard" 
                leftSection={<IconDashboard size={16} />}
                className="pink-tab-item"
              >
                {t('nav.dashboard')}
              </Tabs.Tab>
              
              <Tabs.Tab 
                value="scripts" 
                leftSection={<IconScript size={16} />}
                className="pink-tab-item"
              >
                {t('nav.scripts')}
              </Tabs.Tab>
              
              <Tabs.Tab 
                value="analytics" 
                leftSection={<IconChartBar size={16} />}
                className="pink-tab-item"
                rightSection={
                  <Badge size="xs" color="pink" variant="dot">New</Badge>
                }
              >
                Analytics
              </Tabs.Tab>
              
              <Tabs.Tab 
                value="admin" 
                leftSection={<IconShield size={16} />}
                className="pink-tab-item admin-tab"
                rightSection={
                  <IconSparkles size={12} className="text-pink-500" />
                }
              >
                Admin Panel
              </Tabs.Tab>
              
              <Tabs.Tab 
                value="history" 
                leftSection={<IconHistory size={16} />}
                className="pink-tab-item"
              >
                {t('nav.history')}
              </Tabs.Tab>
              
              <Tabs.Tab 
                value="logs" 
                leftSection={<IconFileText size={16} />}
                className="pink-tab-item"
              >
                {t('logs.title')}
              </Tabs.Tab>

              <Tabs.Tab 
                value="output" 
                leftSection={<IconFileText size={16} />}
                className="pink-tab-item"
                rightSection={
                  Object.keys(runningScripts).length > 0 && (
                    <Badge size="xs" color="pink" variant="filled">
                      {Object.keys(runningScripts).length}
                    </Badge>
                  )
                }
              >
                Script Output
              </Tabs.Tab>
            </Tabs.List>

            <Tabs.Panel value="dashboard">
              <Dashboard
                scripts={scripts}
                officialScripts={officialScripts}
                onAddScript={() => isAdmin && setShowAddScript(true)}
                isAdmin={isAdmin}
                favorites={favorites}
                onToggleFavorite={toggleFavorite}
                onRunScript={runScriptDirectly}
              />
            </Tabs.Panel>

            <Tabs.Panel value="scripts">
              <div className="scripts-section space-y-4">
                <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
                  <div className="flex-1 md:max-w-md">
                    <SearchBox onSearch={setScriptSearch} />
                  </div>
                  <Group gap="sm">
                    <Button
                      onClick={openScriptFolder}
                      variant="light"
                      color="pink"
                      leftSection={<IconFolderOpen size={16} />}
                    >
                      Open Script Folder
                    </Button>
                  {isAdmin && (
                    <Button
                      onClick={() => setShowAddScript(true)}
                      color="pink"
                      leftSection={<IconSparkles size={16} />}
                      className="pink-button"
                    >
                      {t('scripts.addScript')}
                    </Button>
                  )}
                  </Group>
                </div>

                {isAdmin && (
                  <AdminDropzone onUploaded={handleOfficialAdded} scriptsDirOfficial={officialDir} />
                )}

                <ScriptList
                  title={t('scripts.officialScripts')}
                  scripts={officialScripts.filter(s => s.toLowerCase().includes(scriptSearch.toLowerCase()))}
                  emptyText={t('scripts.noScripts')}
                  selected={selectedScript}
                  onSelect={runScriptDirectly}
                  onDelete={isAdmin ? (s) => deleteScript(s, "official") : undefined}
                  onEncrypt={isAdmin ? encryptScript : undefined}
                  onToggleFavorite={toggleFavorite}
                  favorites={favorites}
                />

                <ScriptList
                  title={t('scripts.userScripts')}
                  scripts={scripts.filter(s => s.toLowerCase().includes(scriptSearch.toLowerCase()))}
                  selected={selectedScript}
                  onSelect={runScriptDirectly}
                  onDelete={isAdmin ? (s) => deleteScript(s, "scripts") : undefined}
                  emptyText={t('scripts.noScripts')}
                  onToggleFavorite={toggleFavorite}
                  favorites={favorites}
                />
              </div>
            </Tabs.Panel>

            <Tabs.Panel value="analytics">
              <Analytics />
            </Tabs.Panel>

            <Tabs.Panel value="admin">
              <AdminPanel 
                isAdmin={isAdmin}
                scriptsDir={scriptsDir}
                officialDir={officialDir}
                onRefreshScripts={() => {
                  loadLocalScripts();
                  loadOfficialScripts();
                }}
              />
            </Tabs.Panel>

            <Tabs.Panel value="history">
              <History />
            </Tabs.Panel>

            <Tabs.Panel value="logs">
              {selectedScript ? (
                <LogViewer scriptName={selectedScript} />
              ) : (
                <Alert color="pink" variant="light">
                  <Text size="sm">Select a script to view its logs</Text>
                </Alert>
              )}
            </Tabs.Panel>

            {Object.keys(runningScripts).length > 0 ? (
              <Tabs.Panel value="output">
                <Stack gap="md">
                  <Group justify="space-between">
                    <Title order={3}>Script Outputs</Title>
                    <Badge color="pink" variant="light">
                      {Object.keys(runningScripts).length} script{Object.keys(runningScripts).length > 1 ? 's' : ''}
                    </Badge>
                  </Group>
                  
                  <Tabs defaultValue={selectedScript || Object.keys(runningScripts)[0]} color="pink" variant="pills">
                    <Tabs.List>
                      {Object.keys(runningScripts).map((scriptName) => (
                        <Tabs.Tab 
                          key={scriptName} 
                          value={scriptName}
                          rightSection={
                            runningScripts[scriptName].isRunning && (
                              <div className="animate-spin rounded-full h-3 w-3 border-2 border-pink-200 border-t-pink-500"></div>
                            )
                          }
                        >
                          {scriptName}
                        </Tabs.Tab>
                      ))}
                    </Tabs.List>

                    {Object.entries(runningScripts).map(([scriptName, data]) => (
                      <Tabs.Panel key={scriptName} value={scriptName} pt="md">
                        <div className="bg-white dark:bg-gray-800 rounded-lg border-2 border-pink-200 dark:border-pink-900 pink-glow">
                          <div className="p-4 border-b border-pink-100 dark:border-gray-700">
                            <Group justify="space-between">
                              <div>
                                <Text fw={700} className="text-pink-600">
                                  {scriptName}
                                </Text>
                                <Text size="sm" c="dimmed">
                                  {data.isRunning ? "Running..." : "Completed"}
                                </Text>
                              </div>
                              <Button
                                size="xs"
                                variant="light"
                                color="pink"
                                onClick={() => {
                                  setRunningScripts(prev => {
                                    const updated = { ...prev };
                                    delete updated[scriptName];
                                    return updated;
                                  });
                                }}
                              >
                                Clear
                              </Button>
                            </Group>
                          </div>
                          <pre className="p-4 overflow-auto max-h-96 text-sm font-mono bg-gray-50 dark:bg-gray-900 white-space-pre-wrap break-words">
                            {data.output}
                          </pre>
                        </div>
                      </Tabs.Panel>
                    ))}
                  </Tabs>
                </Stack>
              </Tabs.Panel>
            ) : (
              <Tabs.Panel value="output">
                <Alert color="pink" variant="light">
                  <Text size="sm">No scripts have been run yet. Click a script to execute it and see the output here.</Text>
                </Alert>
              </Tabs.Panel>
            )}
          </Tabs>

          {showAddScript && (
            <AddScript
              onScriptAdded={handleScriptAdded}
              onClose={() => setShowAddScript(false)}
              scriptsDir={scriptsDir}
            />
          )}

          <UpdateNotification />
        </Container>

        {/* Footer */}
        <Box 
          component="footer" 
          className="text-center py-4 mt-8 border-t border-pink-100 dark:border-gray-700"
        >
          <Text size="xs" c="dimmed">
            Script Runner v{appVersion} {"\u2022"} Made with{' '}
            <span className="text-pink-500 animate-pulse">{"\u2661"}</span> and{' '}
            <IconSparkles size={12} className="inline text-pink-500" />
          </Text>
        </Box>
      </AppShell.Main>

      {/* Notification System */}
      <ToastContainer toasts={toasts} onClose={removeToast} />
      <NotificationCenter
        notifications={notifications}
        onMarkAsRead={markAsRead}
        onMarkAsUnread={markAsUnread}
        onClearAll={clearAll}
        unreadCount={unreadCount}
      />
      {githubAuthDrawer}
    </AppShell>
  );
}

