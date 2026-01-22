import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Alert, Button, Group, Stack, Modal } from '@mantine/core';
import { AlertCircle, Download, ExternalLink } from 'lucide-react';

interface UpdateNotificationProps {
  onUpdateNotified?: () => void;
}

export function UpdateNotification({ onUpdateNotified }: UpdateNotificationProps) {
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [downloadUrl, setDownloadUrl] = useState('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const checkUpdates = async () => {
      try {
        const hasUpdate = await invoke<boolean>('check_for_updates');
        if (hasUpdate) {
          const url = await invoke<string>('get_download_url');
          setDownloadUrl(url);
          setUpdateAvailable(true);
          onUpdateNotified?.();
        }
      } catch (err) {
        console.error('Failed to check for updates:', err);
        // Silently fail - don't show error to user
      }
    };

    // Check on mount
    checkUpdates();

    // Check every 60 minutes
    const interval = setInterval(checkUpdates, 60 * 60 * 1000);

    return () => clearInterval(interval);
  }, [onUpdateNotified]);

  const handleOpenDownload = () => {
    try {
      window.open(downloadUrl, '_blank');
    } catch (err) {
      setError('Failed to open download page');
    }
  };

  return (
    <>
      {updateAvailable && (
        <Modal
          opened={updateAvailable}
          onClose={() => setUpdateAvailable(false)}
          title="Update Available"
          centered
        >
          <Stack gap="md">
            <Alert
              icon={<Download size={16} />}
              title="New Version Available"
              color="blue"
            >
              A new version of ScriptRunner is available. Click below to download and install it.
            </Alert>

            {error && (
              <Alert icon={<AlertCircle size={16} />} title="Error" color="red">
                {error}
              </Alert>
            )}

            <Group justify="flex-end">
              <Button
                variant="default"
                onClick={() => setUpdateAvailable(false)}
              >
                Later
              </Button>
              <Button
                onClick={handleOpenDownload}
                rightSection={<ExternalLink size={14} />}
                color="blue"
              >
                Download Update
              </Button>
            </Group>
          </Stack>
        </Modal>
      )}
    </>
  );
}
