import { useState } from "react";
import { invoke } from '@tauri-apps/api/core';
import { motion } from "framer-motion";
import { Play, Square } from "lucide-react";

interface ScriptExecutorProps {
  script: string;
  onOutput: (output: string) => void;
}

export default function ScriptExecutor({ script, onOutput }: ScriptExecutorProps) {
  const [isRunning, setIsRunning] = useState(false);
  const [output, setOutput] = useState("");

  const handleRun = async () => {
    setIsRunning(true);
    setOutput("Running...\n");

    try {
      const result: string = await invoke("run_script", { scriptName: script });
      setOutput(result);
      onOutput(result);
    } catch (error) {
      setOutput(`Error: ${error}`);
      onOutput(`Error: ${error}`);
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <motion.div className="script-executor" initial={{ x: 20, opacity: 0 }} animate={{ x: 0, opacity: 1 }}>
      <h2>{script}</h2>

      <div className="executor-controls">
        <motion.button
          className={`run-btn ${isRunning ? "running" : ""}`}
          onClick={handleRun}
          disabled={isRunning}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
        >
          {isRunning ? (
            <>
              <Square size={18} /> Stop
            </>
          ) : (
            <>
              <Play size={18} /> Run Script
            </>
          )}
        </motion.button>
      </div>

      <div className="output-section">
        <h3>Output</h3>
        <pre className="output-box">{output}</pre>
      </div>
    </motion.div>
  );
}
