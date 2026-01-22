import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Table, Badge, Title, Group, Button, Stack, Code, Alert, Text } from '@mantine/core';
import { Download } from 'lucide-react';
import SearchBox from './SearchBox';

interface RunRecord {
  id: string;
  script_name: string;
  start_time: string;
  end_time: string;
  duration_ms: number;
  status: string;
  output: string;
  error?: string;
}

export default function History() {
  const { t } = useTranslation();
  const [records, setRecords] = useState<RunRecord[]>([]);
  const [filtered, setFiltered] = useState<RunRecord[]>([]);
  const [expanded, setExpanded] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadHistory();
    const interval = setInterval(loadHistory, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadHistory = async () => {
    try {
      const result = await invoke('get_run_history', { limit: 100 });
      setRecords(result as RunRecord[]);
      setFiltered(result as RunRecord[]);
    } catch (e) {
      console.error('Failed to load history:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = (query: string) => {
    if (!query) {
      setFiltered(records);
    } else {
      setFiltered(records.filter(r =>
        r.script_name.toLowerCase().includes(query.toLowerCase())
      ));
    }
  };

  const exportCsv = async () => {
    try {
      const csv = await invoke('export_history_as_csv', { limit: 100 });
      const blob = new Blob([csv as string], { type: 'text/csv;charset=utf-8;' });
      const link = document.createElement('a');
      const url = URL.createObjectURL(blob);
      link.setAttribute('href', url);
      link.setAttribute('download', `historia_${new Date().toISOString().slice(0, 10)}.csv`);
      link.click();
    } catch (e) {
      console.error('Failed to export CSV:', e);
    }
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  const formatTime = (iso: string) => {
    try {
      const date = new Date(iso);
      return date.toLocaleString('pl-PL');
    } catch {
      return iso;
    }
  };

  if (loading) {
    return <Text ta="center" c="dimmed">{t('app.loading')}</Text>;
  }

  return (
    <Stack gap="md">
      <Group justify="space-between">
        <Title order={2}>{t('history.title')}</Title>
        <Button
          leftSection={<Download size={16} />}
          onClick={exportCsv}
          disabled={filtered.length === 0}
        >
          {t('history.csv')}
        </Button>
      </Group>

      <SearchBox onSearch={handleSearch} />

      {filtered.length === 0 ? (
        <Alert title={t('history.noHistory')} color="gray">
          {records.length === 0 ? t('history.noHistory') : t('history.noHistory')}
        </Alert>
      ) : (
        <Stack gap="md">
          {filtered.map(record => (
            <Stack key={record.id} gap="xs">
              <Group
                justify="space-between"
                style={{ cursor: 'pointer', padding: '12px', borderRadius: '8px' }}
                className="hover:bg-gray-50 dark:hover:bg-gray-700"
                onClick={() => setExpanded(expanded === record.id ? null : record.id)}
              >
                <Group gap="lg" wrap="wrap">
                  <Text fw={500} size="sm">
                    {record.script_name}
                  </Text>
                  <Badge
                    color={record.status === 'success' ? 'green' : 'red'}
                    variant="light"
                  >
                    {record.status === 'success' ? t('history.success') : t('history.failed')}
                  </Badge>
                  <Text size="sm" c="dimmed">
                    {formatTime(record.start_time)}
                  </Text>
                  <Text size="sm" c="dimmed">
                    {formatDuration(record.duration_ms)}
                  </Text>
                </Group>
              </Group>

              {expanded === record.id && (
                <Stack gap="md" p="md" style={{ backgroundColor: 'var(--mantine-color-gray-0)' }} className="dark:bg-gray-900 rounded-md">
                  {record.output && (
                    <Stack gap="xs">
                      <Text fw={600} size="sm">
                        {t('dashboard.output')}
                      </Text>
                      <Code block p="md" style={{ maxHeight: '240px', overflow: 'auto' }}>
                        {record.output}
                      </Code>
                    </Stack>
                  )}
                  {record.error && (
                    <Stack gap="xs">
                      <Text fw={600} size="sm" c="red">
                        {t('app.error')}
                      </Text>
                      <Alert color="red" title={t('app.error')}>
                        <Code block p="md" style={{ maxHeight: '160px', overflow: 'auto' }} color="red">
                          {record.error}
                        </Code>
                      </Alert>
                    </Stack>
                  )}
                </Stack>
              )}
            </Stack>
          ))}
        </Stack>
      )}
    </Stack>
  );
}
