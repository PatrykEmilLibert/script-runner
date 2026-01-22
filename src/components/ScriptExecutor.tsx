import { useState } from "react";
import { invoke } from '@tauri-apps/api/core';
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { Play, Square, AlertTriangle } from "lucide-react";
import { useNotifications } from "../hooks/useNotifications";

interface ScriptExecutorProps {
  script: string;
  onOutput: (output: string) => void;
}

export default function ScriptExecutor({ script, onOutput }: ScriptExecutorProps) {
  const { t } = useTranslation();
  const { sendNotification } = useNotifications();
  const [isRunning, setIsRunning] = useState(false);
  const [output, setOutput] = useState("");
  const [compatibilityWarning, setCompatibilityWarning] = useState<string[]>([]);
  const [scriptArgs, setScriptArgs] = useState("");

  const handleRun = async () => {
    // Check compatibility before running
    try {
      const issues: string[] = await invoke("check_script_compatibility", { scriptName: script });
      if (issues.length > 0) {
        setCompatibilityWarning(issues);
        const proceed = window.confirm(
          `⚠️ This script contains Windows-specific libraries:\n\n${issues.join("\n")}\n\nThis may not work correctly on your system. Continue anyway?`
        );
        if (!proceed) return;
      }
    } catch (err) {
      console.error("Failed to check compatibility:", err);
      // Proceed anyway if check fails
    }

    setIsRunning(true);
    setOutput(`${t('scripts.running')}\n`);

    try {
      // Parse args from input (space-separated)
      const args = scriptArgs.trim() ? scriptArgs.trim().split(/\s+/) : null;
      const result: string = await invoke("run_script", { scriptName: script, args });
      setOutput(result);
      onOutput(result);
      await sendNotification(t('messages.scriptCompleted'), `${script}`);
    } catch (error) {
      const errorMsg = `${t('app.error')}: ${error}`;
      setOutput(errorMsg);
      onOutput(errorMsg);
      await sendNotification(t('messages.scriptFailed'), `${script}`);
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <motion.div className="script-executor bg-white dark:bg-gray-800 rounded-lg p-4 border dark:border-gray-700" initial={{ x: 20, opacity: 0 }} animate={{ x: 0, opacity: 1 }}>
      <h2 className="text-xl font-bold mb-4 text-gray-800 dark:text-gray-100">{script}</h2>

      {compatibilityWarning.length > 0 && (
        <div className="mb-4 p-3 bg-yellow-100 dark:bg-yellow-900/30 border border-yellow-400 dark:border-yellow-600 rounded flex gap-3">
          <AlertTriangle className="text-yellow-600 dark:text-yellow-400 flex-shrink-0" size={20} />
          <div>
            <p className="font-semibold text-yellow-800 dark:text-yellow-200">Platform Compatibility Warning</p>
            <ul className="text-sm text-yellow-700 dark:text-yellow-300 mt-2">
              {compatibilityWarning.map((issue, idx) => (
                <li key={idx}>• {issue}</li>
              ))}
            </ul>
            <p className="text-xs text-yellow-700 dark:text-yellow-400 mt-2">This script may not work correctly on your system.</p>
          </div>
        </div>
      )}

      <div className="space-y-3 mb-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Arguments (optional)
          </label>
          <input
            type="text"
            value={scriptArgs}
            onChange={(e) => setScriptArgs(e.target.value)}
            placeholder="e.g., --date 2024-01-01 --input file.txt"
            disabled={isRunning}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50"
          />
        </div>
      </div>

      <div className="executor-controls mb-4">
        <motion.button
          className={`flex items-center gap-2 px-4 py-2 rounded-lg font-semibold transition-all ${
            isRunning
              ? "bg-red-600 hover:bg-red-700 text-white"
              : "bg-green-600 hover:bg-green-700 text-white"
          } disabled:opacity-50`}
          onClick={handleRun}
          disabled={isRunning}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
        >
          {isRunning ? (
            <>
              <Square size={18} /> {t('buttons.cancel')}
            </>
          ) : (
            <>
              <Play size={18} /> {t('buttons.run')}
            </>
          )}
        </motion.button>
      </div>

      <div className="output-section">
        <h3 className="font-semibold mb-2 text-gray-800 dark:text-gray-100">{t('dashboard.output')}</h3>
        <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-auto max-h-96 font-mono text-sm">{output}</pre>
      </div>
    </motion.div>
  );
}
