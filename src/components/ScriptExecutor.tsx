import { useState } from "react";
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from "react-i18next";
import { Play, Square } from "lucide-react";
import { useNotifications } from "../hooks/useNotifications";
import { Stack, Button, TextInput, Alert, Group, Text, Code, List } from '@mantine/core';

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
    <div>
      <Stack gap="md">
        <div>
          <Text size="lg" fw={700}>{script}</Text>
        </div>

        {compatibilityWarning.length > 0 && (
          <Alert color="yellow" title="Platform Compatibility Warning">
            <Stack gap="sm">
              <Text size="sm">This script contains Windows-specific libraries:</Text>
              <List type="unordered" withPadding>
                {compatibilityWarning.map((issue, idx) => (
                  <List.Item key={idx}>{issue}</List.Item>
                ))}
              </List>
              <Text size="xs">This script may not work correctly on your system.</Text>
            </Stack>
          </Alert>
        )}

        <div>
          <TextInput
            label="Arguments (optional)"
            placeholder="e.g., --date 2024-01-01 --input file.txt"
            value={scriptArgs}
            onChange={(e) => setScriptArgs(e.target.value)}
            disabled={isRunning}
          />
        </div>

        <Group>
          <Button
            onClick={handleRun}
            disabled={isRunning}
            color={isRunning ? "red" : "green"}
            leftSection={isRunning ? <Square size={18} /> : <Play size={18} />}
          >
            {isRunning ? t('buttons.cancel') : t('buttons.run')}
          </Button>
        </Group>

        <div>
          <Text fw={600} mb="xs">{t('dashboard.output')}</Text>
          <Code block p="md" style={{ whiteSpace: 'pre-wrap', maxHeight: '400px', overflow: 'auto' }}>
            {output}
          </Code>
        </div>
      </Stack>
    </div>
  );
}
