import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import Dashboard from "./components/Dashboard";
import ScriptList from "./components/ScriptList";
import ScriptExecutor from "./components/ScriptExecutor";
import LogViewer from "./components/LogViewer";
import { AddScript } from "./components/AddScript";
import "./App.css";

export default function App() {
  const [appBlocked, setAppBlocked] = useState(false);
  const [networkError, setNetworkError] = useState(false);
  const [scripts, setScripts] = useState<string[]>([]);
  const [selectedScript, setSelectedScript] = useState<string | null>(null);
  const [output, setOutput] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"dashboard" | "scripts" | "logs">("dashboard");
  const [showAddScript, setShowAddScript] = useState(false);
  const [scriptsDir] = useState("P:\\python_runner_github\\script-runner-scripts");
  const [isAdmin, setIsAdmin] = useState(false);

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
        const admin: boolean = await invoke("check_admin_key");
        setIsAdmin(admin);

        // Sync scripts from GitHub
        await invoke("sync_scripts");

        // List available scripts
        const scriptList: string[] = await invoke("list_scripts");
        setScripts(scriptList);
        
        // Load local scripts
        await loadLocalScripts();
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
      const localScripts: any[] = await invoke("get_local_scripts", { scriptsDir });
      console.log("Local scripts:", localScripts);
    } catch (error) {
      console.error("Failed to load local scripts:", error);
    }
  };

  const handleScriptAdded = async () => {
    await loadLocalScripts();
    const scriptList: string[] = await invoke("list_scripts");
    setScripts(scriptList);
  };

  if (networkError) {
    return (
      <motion.div className="blocked-screen" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
        <div className="blocked-content">
          <h1>⚠️ No Internet Connection</h1>
          <p>ScriptRunner requires an active internet connection to:</p>
          <ul style={{ textAlign: 'left', marginTop: '20px' }}>
            <li>Verify security status (kill switch)</li>
            <li>Sync scripts from GitHub</li>
            <li>Install Python dependencies</li>
            <li>Download updates</li>
          </ul>
          <p style={{ marginTop: '20px' }}>Please connect to the internet and restart the application.</p>
        </div>
      </motion.div>
    );
  }

  if (appBlocked) {
    return (
      <motion.div className="blocked-screen" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
        <div className="blocked-content">
          <h1>Application Blocked</h1>
          <p>This application has been remotely disabled for security reasons.</p>
          <p>Please contact your administrator.</p>
        </div>
      </motion.div>
    );
  }

  if (isLoading) {
    return (
      <motion.div className="loading-screen" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
        <div className="spinner"></div>
        <p>Initializing ScriptRunner...</p>
      </motion.div>
    );
  }

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>🚀 ScriptRunner</h1>
        <p>Python Script Executor with Auto-Update</p>
      </header>

      <nav className="app-nav">
        <button
          className={`nav-btn ${activeTab === "dashboard" ? "active" : ""}`}
          onClick={() => setActiveTab("dashboard")}
        >
          Dashboard
        </button>
        <button
          className={`nav-btn ${activeTab === "scripts" ? "active" : ""}`}
          onClick={() => setActiveTab("scripts")}
        >
          Scripts
        </button>
        <button
          className={`nav-btn ${activeTab === "logs" ? "active" : ""}`}
          onClick={() => setActiveTab("logs")}
        >
          Logs
        </button>
      </nav>

      <main className="app-main">
        {activeTab === "dashboard" && (
          <Dashboard
            scripts={scripts}
            onAddScript={() => isAdmin && setShowAddScript(true)}
            isAdmin={isAdmin}
          />
        )}
        {activeTab === "scripts" && (
          <div className="scripts-section">
            <ScriptList scripts={scripts} selected={selectedScript} onSelect={setSelectedScript} />
            {selectedScript && (
              <ScriptExecutor script={selectedScript} onOutput={setOutput} />
            )}
          </div>
        )}
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
          className="output-panel"
          initial={{ y: 100, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
        >
          <h3>Output</h3>
          <pre>{output}</pre>
        </motion.div>
      )}
    </div>
  );
}
