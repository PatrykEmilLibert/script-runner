import { useState } from "react";
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from "react-i18next";
import { Play, Star } from "lucide-react";
import { useNotifications } from "../hooks/useNotifications";
import { Stack, Button, Alert, Group, Text, ActionIcon } from '@mantine/core';

interface ScriptExecutorProps {
  script: string;
  onOutput: (output: string) => void;
  onFavoriteToggle?: (scriptName: string, isFav: boolean) => void;
  isFavorite?: boolean;
}

export default function ScriptExecutor({ script, onOutput, onFavoriteToggle, isFavorite = false }: ScriptExecutorProps) {
  const { t } = useTranslation();
  const { sendNotification } = useNotifications();
  const [isRunning, setIsRunning] = useState(false);

  const handleRun = async () => {
    setIsRunning(true);

    // Check compatibility
    try {
      const issues: string[] = await invoke("check_script_compatibility", { scriptName: script });
      if (issues.length > 0 && !window.confirm(
        `⚠️ This script contains Windows-specific libraries:\n\n${issues.join("\n")}\n\nContinue anyway?`
      )) {
        setIsRunning(false);
        return;
      }
    } catch (err) {
      // Proceed if check fails
    }

    // Stream output live: append each line the script prints as it arrives,
    // instead of only showing everything once the script finishes.
    let streamed = "";
    let unlisten: (() => void) | undefined;
    try {
      unlisten = await listen<{ script: string; stream: string; line: string }>(
        "script-output",
        (event) => {
          if (event.payload.script !== script) return;
          streamed += event.payload.line + "\n";
          onOutput(streamed);
        }
      );

      const result: string = await invoke("run_script", { scriptName: script, args: null });
      onOutput(result || streamed);
      await sendNotification(t('messages.scriptCompleted'), `${script}`, 'success', { sound: false });
    } catch (error) {
      const errorMsg = streamed ? `${streamed}\nError: ${error}` : `Error: ${error}`;
      onOutput(errorMsg);
      await sendNotification(t('messages.scriptFailed'), `${script}`, 'error', { sound: false });
    } finally {
      if (unlisten) unlisten();
      setIsRunning(false);
    }
  };

  return (
    <Stack gap="md">
      <Group justify="space-between" align="center">
        <div>
          <Text fw={500} size="lg">{script}</Text>
          <Text size="sm" c="dimmed">Click to execute this script</Text>
        </div>
        <ActionIcon
          size="lg"
          variant="light"
          color={isFavorite ? "pink" : "gray"}
          onClick={() => onFavoriteToggle?.(script, !isFavorite)}
        >
          <Star size={18} fill={isFavorite ? "#FF1493" : "none"} color="#FF1493" />
        </ActionIcon>
      </Group>

      <Button
        leftSection={<Play size={20} />}
        size="lg"
        variant="gradient"
        gradient={{ from: "pink", to: "grape", deg: 135 }}
        fullWidth
        onClick={handleRun}
        loading={isRunning}
        className="pink-glow"
      >
        {isRunning ? "Running..." : "▶ Run Script"}
      </Button>

      {isRunning && (
        <Alert color="blue" title="Running..." variant="light">
          Script is being executed...
        </Alert>
      )}
    </Stack>
  );
}
