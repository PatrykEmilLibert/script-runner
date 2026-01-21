import { useState } from "react";
import { invoke } from '@tauri-apps/api/core';
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { Play, Square } from "lucide-react";
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

  const handleRun = async () => {
    setIsRunning(true);
    setOutput(`${t('scripts.running')}\n`);

    try {
      const result: string = await invoke("run_script", { scriptName: script });
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
