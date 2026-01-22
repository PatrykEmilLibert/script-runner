import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Button, Alert, Stack, Badge, Text, Code, List, Group } from '@mantine/core';

interface Candidate {
  path: string;
  exists: boolean;
  valid: boolean;
}

export default function AdminKeyDiagnostics() {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);
  const [info, setInfo] = useState<{ candidates: Candidate[] } | null>(null);
  const [loading, setLoading] = useState(false);

  const checkPaths = async () => {
    setLoading(true);
    try {
      const result = await invoke('get_admin_key_info');
      setInfo(result as { candidates: Candidate[] });
      setExpanded(true);
    } catch (e) {
      console.error('Failed to get admin key info:', e);
    } finally {
      setLoading(false);
    }
  };

  const getStatusBadge = (candidate: Candidate) => {
    if (candidate.valid) return { label: '✅ Valid', color: 'green' };
    if (candidate.exists) return { label: '⚠️ Invalid Format', color: 'yellow' };
    return { label: '❌ Not Found', color: 'gray' };
  };

  const getAlertColor = (candidate: Candidate) => {
    if (candidate.valid) return 'green';
    if (candidate.exists) return 'yellow';
    return 'gray';
  };

  return (
    <Stack gap="md">
      <Button
        onClick={checkPaths}
        loading={loading}
        fullWidth
        variant={expanded ? 'light' : 'filled'}
        color="blue"
      >
        🔐 {t('admin.title')} - Diagnostyka klucza
      </Button>

      {expanded && info && (
        <Stack gap="md">
          {info.candidates.map((candidate, idx) => {
            const status = getStatusBadge(candidate);
            return (
              <Alert
                key={idx}
                color={getAlertColor(candidate)}
                title={
                  <Group gap="xs">
                    <Badge color={status.color}>{status.label}</Badge>
                  </Group>
                }
              >
                <Stack gap="sm">
                  <Code block p="sm" style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                    {candidate.path}
                  </Code>
                  <Text size="sm" c="dimmed">
                    {candidate.valid
                      ? 'Klucz znaleziony i ważny ✓'
                      : candidate.exists
                      ? 'Plik istnieje, ale nie jest ważny (zły format)'
                      : 'Plik nie istnieje'}
                  </Text>
                </Stack>
              </Alert>
            );
          })}

          <Alert color="blue" title="Jak to działa?">
            <List type="ordered" withPadding>
              <List.Item>Sprawdza zmienną env: <Code>SR_ADMIN_KEY_PATH</Code></List.Item>
              <List.Item>Desktop w profilu użytkownika</List.Item>
              <List.Item><Code>USERPROFILE\Desktop</Code> (Windows)</List.Item>
              <List.Item><Code>AppData\script-runner</Code> (Windows)</List.Item>
              <List.Item><Code>~/Desktop</Code> (Linux/macOS)</List.Item>
              <List.Item><Code>/tmp/sr-admin.key</Code> (fallback)</List.Item>
            </List>
          </Alert>
        </Stack>
      )}
    </Stack>
  );
}
