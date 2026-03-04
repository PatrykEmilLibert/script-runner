import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { Alert, Button, Group, Stack, Modal, Text } from '@mantine/core';
import { AlertCircle, Download, ExternalLink, RefreshCw } from 'lucide-react';

interface UpdateNotificationProps {
  onUpdateNotified?: () => void;
}

interface AppSettings {
  dark_mode: boolean;
  auto_update_enabled: boolean;
}

interface AvailableUpdateInfo {
  version: string;
  current_version: string;
  notes: string | null;
  pub_date: string | null;
  download_url: string;
}

export function UpdateNotification({ onUpdateNotified }: UpdateNotificationProps) {
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [downloadUrl, setDownloadUrl] = useState('');
  const [updateInfo, setUpdateInfo] = useState<AvailableUpdateInfo | null>(null);
  const [supportsInAppInstall, setSupportsInAppInstall] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const loadAutoUpdateEnabled = async (): Promise<boolean> => {
      try {
        const settings = await invoke<AppSettings>('get_settings');
        return settings.auto_update_enabled ?? true;
      } catch {
        return true;
      }
    };

    const checkUpdates = async () => {
      setError(null);

      const autoUpdateEnabled = await loadAutoUpdateEnabled();
      if (!autoUpdateEnabled || cancelled) {
        return;
      }

      try {
        const detailedUpdate = await invoke<AvailableUpdateInfo | null>('check_for_updates_with_details');
        if (detailedUpdate && !cancelled) {
          setUpdateInfo(detailedUpdate);
          setDownloadUrl(detailedUpdate.download_url);
          setSupportsInAppInstall(true);
          setUpdateAvailable(true);
          onUpdateNotified?.();
          return;
        }
      } catch (err) {
        console.warn('Native updater check failed, using fallback flow:', err);
      }

      try {
        const hasUpdate = await invoke<boolean>('check_for_updates');
        if (hasUpdate && !cancelled) {
          const url = await invoke<string>('get_download_url');
          setDownloadUrl(url);
          setUpdateInfo(null);
          setSupportsInAppInstall(false);
          setUpdateAvailable(true);
          onUpdateNotified?.();
        }
      } catch (err) {
        console.error('Failed to check for updates:', err);
      }
    };

    // Check on mount
    void checkUpdates();

    // Check every 60 minutes
    const interval = setInterval(() => {
      void checkUpdates();
    }, 60 * 60 * 1000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [onUpdateNotified]);

  const handleOpenDownload = () => {
    if (!downloadUrl) {
      setError('Download URL is not available');
      return;
    }

    try {
      window.open(downloadUrl, '_blank');
    } catch (err) {
      setError('Failed to open download page');
    }
  };

  const handleInstallUpdate = async () => {
    setInstalling(true);
    setError(null);

    try {
      const installed = await invoke<boolean>('install_update');
      if (!installed) {
        setUpdateAvailable(false);
        return;
      }

      await relaunch();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(`Automatic installation failed: ${message}`);
    } finally {
      setInstalling(false);
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
              {updateInfo
                ? `Version ${updateInfo.version} is available (current: ${updateInfo.current_version}).`
                : 'A new version of ScriptRunner is available.'}
            </Alert>

            {updateInfo?.notes && (
              <Alert color="gray" variant="light" title="Release Notes">
                {updateInfo.notes}
              </Alert>
            )}

            {!supportsInAppInstall && (
              <Text size="sm" c="dimmed">
                Automatic install is unavailable. Open the release page to download the update manually.
              </Text>
            )}

            {error && (
              <Alert icon={<AlertCircle size={16} />} title="Error" color="red">
                {error}
              </Alert>
            )}

            <Group justify="flex-end">
              <Button
                variant="default"
                onClick={() => setUpdateAvailable(false)}
                disabled={installing}
              >
                Later
              </Button>

              {supportsInAppInstall ? (
                <Button
                  onClick={handleInstallUpdate}
                  rightSection={<RefreshCw size={14} />}
                  color="blue"
                  loading={installing}
                >
                  Install Update
                </Button>
              ) : (
                <Button
                  onClick={handleOpenDownload}
                  rightSection={<ExternalLink size={14} />}
                  color="blue"
                >
                  Download Update
                </Button>
              )}

              {supportsInAppInstall && error && (
                <Button
                  variant="light"
                  onClick={handleOpenDownload}
                  rightSection={<ExternalLink size={14} />}
                >
                  Open Release Page
                </Button>
              )}
            </Group>
          </Stack>
        </Modal>
      )}
    </>
  );
}
