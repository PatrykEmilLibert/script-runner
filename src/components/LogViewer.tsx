import { useState, useEffect } from "react";
import { invoke } from '@tauri-apps/api/core';
import { Stack, Code, Title, Loader, Center, Alert } from "@mantine/core";

interface LogViewerProps {
  scriptName: string;
}

export default function LogViewer({ scriptName }: LogViewerProps) {
  const [logs, setLogs] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchLogs = async () => {
      try {
        const result: string = await invoke("get_script_logs", { scriptName });
        setLogs(result);
        setError(null);
      } catch (error) {
        setError(`No logs available for ${scriptName}`);
        setLogs("");
      } finally {
        setIsLoading(false);
      }
    };

    fetchLogs();
  }, [scriptName]);

  return (
    <Stack gap="md">
      <Title order={2}>Logs - {scriptName}</Title>

      {isLoading ? (
        <Center p="xl">
          <Loader />
        </Center>
      ) : error ? (
        <Alert color="red" title="Error">
          {error}
        </Alert>
      ) : (
        <Code block p="md" style={{ maxHeight: "500px", overflow: "auto" }}>
          {logs || "No logs available"}
        </Code>
      )}
    </Stack>
  );
}
