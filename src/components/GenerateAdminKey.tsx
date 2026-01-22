import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Copy, Check } from 'lucide-react';
import { Stack, Button, Alert, Group, Container, Text, Badge, Code, List } from '@mantine/core';

interface GenerateAdminKeyProps {
  onGenerated: () => void;
}

export default function GenerateAdminKey({ onGenerated }: GenerateAdminKeyProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [keyPath, setKeyPath] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async () => {
    setLoading(true);
    setError(null);
    try {
      const path: string = await invoke('generate_admin_key');
      setKeyPath(path);
      setSuccess(true);
      setTimeout(() => onGenerated(), 1500);
    } catch (e) {
      console.error('Failed to generate key:', e);
      setError(`Błąd: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = () => {
    if (keyPath) {
      navigator.clipboard.writeText(keyPath);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <Container size="md" py="xl">
      <Stack gap="lg" style={{ border: '2px solid #0066cc', borderRadius: '8px', padding: '32px' }}>
        <Stack gap="md" align="center">
          <Badge size="xl">🔐</Badge>
          <Text size="lg" fw={700} ta="center">
            Wygeneruj klucz administratora
          </Text>
          <Text size="sm" c="dimmed" ta="center">
            Klucz administratora pozwala na zarządzanie skryptami oficjalnymi i ustawieniami systemu.
          </Text>
        </Stack>

        {error && (
          <Alert color="red" title="Błąd">
            {error}
          </Alert>
        )}

        {success && keyPath ? (
          <Stack gap="md">
            <Alert color="green" title="✅ Klucz został wygenerowany!">
              <Stack gap="sm">
                <Text size="sm">Klucz znajduje się w:</Text>
                <Code block p="xs" style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                  {keyPath}
                </Code>
                <Button
                  fullWidth
                  onClick={copyToClipboard}
                  leftSection={copied ? <Check size={18} /> : <Copy size={18} />}
                  color={copied ? 'green' : 'blue'}
                >
                  {copied ? 'Skopiowano!' : 'Kopiuj ścieżkę'}
                </Button>
              </Stack>
            </Alert>

            <Alert color="blue" title="Instrukcja">
              <List type="ordered" withPadding>
                <List.Item>Klucz jest zapisany w pliku JSON na Twoim pulpicie</List.Item>
                <List.Item>Trzymaj go w bezpiecznym miejscu</List.Item>
                <List.Item>Możesz skopiować go na inne komputery (Ubuntu, macOS)</List.Item>
                <List.Item>
                  Umieść go w folderze <Code>~/Desktop</Code>
                </List.Item>
              </List>
            </Alert>

            <Text size="sm" c="dimmed" ta="center">
              Aplikacja zaraz się przeładuje...
            </Text>
          </Stack>
        ) : (
          <Stack gap="md">
            <Alert color="yellow" title="⚠️ Brak klucza administratora">
              Wygeneruj nowy, aby uzyskać dostęp do funkcji administratora.
            </Alert>

            <Button
              fullWidth
              size="lg"
              onClick={handleGenerate}
              loading={loading}
              color="blue"
            >
              🔑 Wygeneruj klucz administratora
            </Button>

            <Text size="sm" c="dimmed" ta="center">
              Klucz będzie zapisany na Twoim pulpicie (Desktop)
            </Text>
          </Stack>
        )}
      </Stack>
    </Container>
  );
}
