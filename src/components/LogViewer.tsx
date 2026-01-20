import { useState, useEffect } from "react";
import { invoke } from '@tauri-apps/api/core';
import { motion } from "framer-motion";

interface LogViewerProps {
  scriptName: string;
}

export default function LogViewer({ scriptName }: LogViewerProps) {
  const [logs, setLogs] = useState("");
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchLogs = async () => {
      try {
        const result: string = await invoke("get_script_logs", { scriptName });
        setLogs(result);
      } catch (error) {
        setLogs(`No logs available for ${scriptName}`);
      } finally {
        setIsLoading(false);
      }
    };

    fetchLogs();
  }, [scriptName]);

  return (
    <motion.div className="log-viewer" initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
      <h2>Logs - {scriptName}</h2>

      {isLoading ? (
        <div className="loading">Loading logs...</div>
      ) : (
        <pre className="logs-box">{logs}</pre>
      )}
    </motion.div>
  );
}
