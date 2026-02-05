import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Card,
  Stack,
  Group,
  Title,
  Text,
  Button,
  Badge,
  Switch,
  Textarea,
  Tabs,
  SimpleGrid,
  ThemeIcon,
  Loader,
  Modal,
  TextInput,
  Select,
  Alert,
  Divider,
  ActionIcon,
  Tooltip,
  Code,
} from '@mantine/core';
import {
  BarChart3,
  Shield,
  Key,
  Power,
  Upload,
  Settings,
  GitBranch,
  Lock,
  Unlock,
  Trash2,
  Eye,
  RefreshCw,
  Calendar,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Edit,
  Save,
} from 'lucide-react';
import { AdminDropzone } from './AdminDropzone';
import AdminKeyDiagnostics from './AdminKeyDiagnostics';
import { useNotifications } from '../hooks/useNotifications';

interface AdminPanelProps {
  isAdmin: boolean;
  scriptsDir: string;
  officialDir: string;
  onRefreshScripts: () => void;
}

interface ScriptInfo {
  name: string;
  path: string;
  size: number;
  modified: string;
}

interface KillSwitchStatus {
  blocked: boolean;
  reason?: string;
  scheduledFor?: string;
  whitelist: string[];
}

interface AppStats {
  totalScripts: number;
  officialScripts: number;
  userScripts: number;
  activeSessions: number;
  lastSync: string;
  appStatus: 'active' | 'blocked';
}

export default function AdminPanel({ isAdmin, scriptsDir, officialDir, onRefreshScripts }: AdminPanelProps) {
  const { showSuccess, showError } = useNotifications();
  
  // State management
  const [activeTab, setActiveTab] = useState<string | null>('overview');
  const [loading, setLoading] = useState(false);
  const [stats, setStats] = useState<AppStats>({
    totalScripts: 0,
    officialScripts: 0,
    userScripts: 0,
    activeSessions: 0,
    lastSync: 'Never',
    appStatus: 'active',
  });
  
  // Script Management
  const [scripts, setScripts] = useState<ScriptInfo[]>([]);
  const [showUploader, setShowUploader] = useState(false);
  
  // Kill Switch
  const [killSwitchEnabled, setKillSwitchEnabled] = useState(false);
  const [killSwitchReason, setKillSwitchReason] = useState('');
  const [scheduledDate, setScheduledDate] = useState('');
  const [whitelist, setWhitelist] = useState<string[]>([]);
  const [newWhitelistItem, setNewWhitelistItem] = useState('');
  
  // Settings
  const [githubSyncEnabled, setGithubSyncEnabled] = useState(true);
  const [autoUpdateEnabled, setAutoUpdateEnabled] = useState(true);
  const [encryptionEnabled, setEncryptionEnabled] = useState(false);
  
  // Confirmation modals
  const [confirmModal, setConfirmModal] = useState({ open: false, action: '', data: null as any });

  // Load initial data
  useEffect(() => {
    loadStats();
    loadScripts();
    loadKillSwitchStatus();
  }, []);

  // ==================== DATA LOADING ====================
  
  const loadStats = async () => {
    try {
      const [official, user] = await Promise.all([
        invoke<string[]>('list_official_scripts', { scriptsDir }),
        invoke<string[]>('list_scripts', { scriptsDir }),
      ]);
      
      const lastSyncTime = await invoke<string>('get_last_sync_time').catch(() => 'Never');
      const appBlocked = await invoke<boolean>('check_kill_switch_status').catch(() => false);
      
      setStats({
        totalScripts: official.length + user.length,
        officialScripts: official.length,
        userScripts: user.length,
        activeSessions: 1, // Current session
        lastSync: lastSyncTime,
        appStatus: appBlocked ? 'blocked' : 'active',
      });
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  };

  const loadScripts = async () => {
    try {
      const scriptList = await invoke<ScriptInfo[]>('get_all_scripts_info', { scriptsDir });
      setScripts(scriptList);
    } catch (error) {
      console.error('Failed to load scripts:', error);
      // Fallback to basic list
      const basic = await invoke<string[]>('list_official_scripts', { scriptsDir });
      setScripts(basic.map(name => ({ name, path: name, size: 0, modified: 'Unknown' })));
    }
  };

  const loadKillSwitchStatus = async () => {
    try {
      const status = await invoke<KillSwitchStatus>('get_kill_switch_status');
      setKillSwitchEnabled(status.blocked);
      setKillSwitchReason(status.reason || '');
      setScheduledDate(status.scheduledFor || '');
      setWhitelist(status.whitelist || []);
    } catch (error) {
      console.error('Failed to load kill switch status:', error);
    }
  };

  // ==================== SCRIPT MANAGEMENT ====================
  
  const handleDeleteScript = async (scriptName: string) => {
    setConfirmModal({
      open: true,
      action: 'delete_script',
      data: scriptName,
    });
  };

  const confirmDeleteScript = async () => {
    try {
      setLoading(true);
      await invoke('delete_official_script', {
        scriptName: confirmModal.data,
        scriptsDir,
      });
      showSuccess('Script Deleted', `Script "${confirmModal.data}" deleted successfully`);
      await loadScripts();
      await loadStats();
      if (onRefreshScripts) onRefreshScripts();
    } catch (error) {
      showError('Delete Failed', `Failed to delete script: ${error}`);
    } finally {
      setLoading(false);
      setConfirmModal({ open: false, action: '', data: null });
    }
  };

  const handleEncryptScript = async (scriptName: string) => {
    try {
      setLoading(true);
      await invoke('encrypt_script', { scriptName, scriptsDir });
      showSuccess('Script Encrypted', `Script "${scriptName}" encrypted successfully`);
      await loadScripts();
    } catch (error) {
      showError('Encryption Failed', `Failed to encrypt script: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  // ==================== KILL SWITCH CONTROL ====================
  
  const handleToggleKillSwitch = async (enabled: boolean) => {
    if (enabled) {
      setConfirmModal({
        open: true,
        action: 'enable_kill_switch',
        data: null,
      });
    } else {
      await confirmToggleKillSwitch(false);
    }
  };

  const confirmToggleKillSwitch = async (enabled: boolean) => {
    try {
      setLoading(true);
      await invoke('toggle_kill_switch', {
        enabled,
        reason: killSwitchReason || 'Manual toggle by admin',
      });
      setKillSwitchEnabled(enabled);
      showSuccess('Kill Switch Toggled', enabled ? 'App blocked successfully' : 'App unblocked successfully');
      await loadStats();
    } catch (error) {
      showError('Toggle Failed', `Failed to toggle kill switch: ${error}`);
    } finally {
      setLoading(false);
      setConfirmModal({ open: false, action: '', data: null });
    }
  };

  const handleScheduleBlock = async () => {
    if (!scheduledDate) {
      showError('Invalid Date', 'Please select a date and time');
      return;
    }
    
    try {
      setLoading(true);
      await invoke('schedule_kill_switch', {
        scheduledFor: scheduledDate,
        reason: killSwitchReason || 'Scheduled maintenance',
      });
      showSuccess('Scheduled', `Kill switch scheduled for ${scheduledDate}`);
    } catch (error) {
      showError('Schedule Failed', `Failed to schedule: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleAddToWhitelist = async () => {
    if (!newWhitelistItem.trim()) return;
    
    try {
      await invoke('add_to_whitelist', { item: newWhitelistItem });
      setWhitelist([...whitelist, newWhitelistItem]);
      setNewWhitelistItem('');
      showSuccess('Whitelist Updated', 'Added to whitelist');
    } catch (error) {
      showError('Add Failed', `Failed to add to whitelist: ${error}`);
    }
  };

  const handleRemoveFromWhitelist = async (item: string) => {
    try {
      await invoke('remove_from_whitelist', { item });
      setWhitelist(whitelist.filter(i => i !== item));
      showSuccess('Whitelist Updated', 'Removed from whitelist');
    } catch (error) {
      showError('Remove Failed', `Failed to remove from whitelist: ${error}`);
    }
  };

  // ==================== ADVANCED SETTINGS ====================
  
  const handleSyncGitHub = async () => {
    try {
      setLoading(true);
      await invoke('sync_github_scripts');
      showSuccess('Sync Complete', 'GitHub sync completed successfully');
      await loadScripts();
      await loadStats();
    } catch (error) {
      showError('Sync Failed', `GitHub sync failed: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleAutoUpdate = async (enabled: boolean) => {
    try {
      await invoke('set_auto_update', { enabled });
      setAutoUpdateEnabled(enabled);
      showSuccess('Auto-Update', `Auto-update ${enabled ? 'enabled' : 'disabled'}`);
    } catch (error) {
      showError('Toggle Failed', `Failed to toggle auto-update: ${error}`);
    }
  };

  const handleToggleEncryption = async (enabled: boolean) => {
    try {
      await invoke('set_encryption_enabled', { enabled });
      setEncryptionEnabled(enabled);
      showSuccess('Encryption', `Encryption ${enabled ? 'enabled' : 'disabled'}`);
    } catch (error) {
      showError('Toggle Failed', `Failed to toggle encryption: ${error}`);
    }
  };

  // ==================== UI COMPONENTS ====================
  
  const StatCard = ({ icon: Icon, title, value, color }: any) => (
    <Card className="glass-pink" p="lg" radius="md">
      <Group justify="space-between" mb="xs">
        <Text size="sm" fw={500} c="dimmed">
          {title}
        </Text>
        <ThemeIcon size="lg" radius="md" variant="light" color={color}>
          <Icon size={20} />
        </ThemeIcon>
      </Group>
      <Text size="xl" fw={700}>
        {value}
      </Text>
    </Card>
  );

  return (
    <Stack gap="lg" p="md">
      <Group justify="space-between">
        <Title order={2} style={{ color: 'var(--pink-primary)' }}>
          🛡️ Admin Panel
        </Title>
        <Badge size="lg" variant="gradient" gradient={{ from: 'pink', to: 'grape' }}>
          Administrator
        </Badge>
      </Group>

      <Tabs value={activeTab} onChange={setActiveTab} color="pink">
        <Tabs.List>
          <Tabs.Tab value="overview" leftSection={<BarChart3 size={16} />}>
            Overview
          </Tabs.Tab>
          <Tabs.Tab value="scripts" leftSection={<Upload size={16} />}>
            Script Management
          </Tabs.Tab>
          <Tabs.Tab value="killswitch" leftSection={<Power size={16} />}>
            Kill Switch
          </Tabs.Tab>
          <Tabs.Tab value="settings" leftSection={<Settings size={16} />}>
            Advanced Settings
          </Tabs.Tab>
        </Tabs.List>

        {/* ==================== TAB: OVERVIEW ==================== */}
        <Tabs.Panel value="overview" pt="lg">
          <Stack gap="lg">
            <SimpleGrid cols={{ base: 1, sm: 2, md: 3 }} spacing="lg">
              <StatCard
                icon={BarChart3}
                title="Total Scripts"
                value={stats.totalScripts}
                color="blue"
              />
              <StatCard
                icon={Shield}
                title="Official Scripts"
                value={stats.officialScripts}
                color="pink"
              />
              <StatCard
                icon={GitBranch}
                title="User Scripts"
                value={stats.userScripts}
                color="grape"
              />
              <StatCard
                icon={Key}
                title="Active Sessions"
                value={stats.activeSessions}
                color="teal"
              />
              <StatCard
                icon={RefreshCw}
                title="Last Sync"
                value={stats.lastSync}
                color="cyan"
              />
              <StatCard
                icon={stats.appStatus === 'active' ? CheckCircle : XCircle}
                title="App Status"
                value={stats.appStatus === 'active' ? 'Active' : 'Blocked'}
                color={stats.appStatus === 'active' ? 'green' : 'red'}
              />
            </SimpleGrid>

            <Card className="glass-pink" p="lg">
              <Title order={4} mb="md">
                Quick Actions
              </Title>
              <Group>
                <Button
                  className="btn-pink"
                  leftSection={<Upload size={18} />}
                  onClick={() => setShowUploader(true)}
                >
                  Upload Script
                </Button>
                <Button
                  className="btn-pink-outline"
                  leftSection={<RefreshCw size={18} />}
                  onClick={handleSyncGitHub}
                  loading={loading}
                >
                  Sync GitHub
                </Button>
              </Group>
            </Card>

            <AdminKeyDiagnostics />
          </Stack>
        </Tabs.Panel>

        {/* ==================== TAB: SCRIPT MANAGEMENT ==================== */}
        <Tabs.Panel value="scripts" pt="lg">
          <Stack gap="lg">
            <Group justify="space-between">
              <Title order={3}>Manage Scripts</Title>
              <Button
                className="btn-pink"
                leftSection={<Upload size={18} />}
                onClick={() => setShowUploader(true)}
              >
                Upload New Script
              </Button>
            </Group>

            {showUploader && (
              <Card className="glass-pink" p="lg">
                <AdminDropzone
                  scriptsDirOfficial={scriptsDir}
                  onUploaded={() => {
                    setShowUploader(false);
                    loadScripts();
                    loadStats();
                    if (onRefreshScripts) onRefreshScripts();
                  }}
                />
              </Card>
            )}

            <Card className="glass-pink" p="lg">
              <Stack gap="md">
                {scripts.length === 0 ? (
                  <Text c="dimmed" ta="center" py="xl">
                    No scripts found
                  </Text>
                ) : (
                  scripts.map((script) => (
                    <Card key={script.name} withBorder p="md">
                      <Group justify="space-between">
                        <Stack gap={4}>
                          <Text fw={600}>{script.name}</Text>
                          <Group gap="xs">
                            <Badge size="sm" variant="light">
                              {(script.size / 1024).toFixed(1)} KB
                            </Badge>
                            <Text size="xs" c="dimmed">
                              Modified: {script.modified}
                            </Text>
                          </Group>
                        </Stack>
                        <Group gap="xs">
                          <Tooltip label="Encrypt">
                            <ActionIcon
                              variant="light"
                              color="blue"
                              onClick={() => handleEncryptScript(script.name)}
                            >
                              <Lock size={16} />
                            </ActionIcon>
                          </Tooltip>
                          <Tooltip label="Delete">
                            <ActionIcon
                              variant="light"
                              color="red"
                              onClick={() => handleDeleteScript(script.name)}
                            >
                              <Trash2 size={16} />
                            </ActionIcon>
                          </Tooltip>
                        </Group>
                      </Group>
                    </Card>
                  ))
                )}
              </Stack>
            </Card>
          </Stack>
        </Tabs.Panel>

        {/* ==================== TAB: KILL SWITCH ==================== */}
        <Tabs.Panel value="killswitch" pt="lg">
          <Stack gap="lg">
            <Card className="glass-pink" p="lg">
              <Group justify="space-between" mb="lg">
                <Stack gap={4}>
                  <Title order={3}>Kill Switch Control</Title>
                  <Text size="sm" c="dimmed">
                    Remotely block or enable the application
                  </Text>
                </Stack>
                <Group gap="md">
                  <Badge
                    size="lg"
                    color={killSwitchEnabled ? 'red' : 'green'}
                    variant="filled"
                    leftSection={killSwitchEnabled ? '🔴' : '🟢'}
                  >
                    {killSwitchEnabled ? 'BLOCKED' : 'ACTIVE'}
                  </Badge>
                  <Switch
                    size="lg"
                    color="pink"
                    checked={killSwitchEnabled}
                    onChange={(e) => handleToggleKillSwitch(e.currentTarget.checked)}
                    thumbIcon={
                      killSwitchEnabled ? (
                        <Lock size={12} style={{ color: 'var(--pink-primary)' }} />
                      ) : (
                        <Unlock size={12} style={{ color: 'var(--pink-primary)' }} />
                      )
                    }
                  />
                </Group>
              </Group>

              <Divider my="md" />

              <Stack gap="md">
                <Textarea
                  label="Block Reason"
                  placeholder="Enter reason for blocking the application..."
                  value={killSwitchReason}
                  onChange={(e) => setKillSwitchReason(e.currentTarget.value)}
                  minRows={3}
                />

                <Group grow>
                  <Button
                    className="btn-pink"
                    leftSection={<Power size={18} />}
                    onClick={() => confirmToggleKillSwitch(true)}
                    loading={loading}
                    disabled={killSwitchEnabled}
                  >
                    Block Now
                  </Button>
                  <Button
                    className="btn-pink-outline"
                    leftSection={<Calendar size={18} />}
                    onClick={handleScheduleBlock}
                    loading={loading}
                  >
                    Schedule Block
                  </Button>
                </Group>

                <TextInput
                  label="Schedule for"
                  type="datetime-local"
                  value={scheduledDate}
                  onChange={(e) => setScheduledDate(e.currentTarget.value)}
                />
              </Stack>
            </Card>

            <Card className="glass-pink" p="lg">
              <Title order={4} mb="md">
                Whitelist Management
              </Title>
              <Stack gap="md">
                <Group>
                  <TextInput
                    placeholder="User ID or email..."
                    value={newWhitelistItem}
                    onChange={(e) => setNewWhitelistItem(e.currentTarget.value)}
                    style={{ flex: 1 }}
                  />
                  <Button className="btn-pink" onClick={handleAddToWhitelist}>
                    Add
                  </Button>
                </Group>

                {whitelist.length > 0 && (
                  <Stack gap="xs">
                    {whitelist.map((item) => (
                      <Card key={item} withBorder p="sm">
                        <Group justify="space-between">
                          <Code>{item}</Code>
                          <ActionIcon
                            variant="light"
                            color="red"
                            onClick={() => handleRemoveFromWhitelist(item)}
                          >
                            <Trash2 size={16} />
                          </ActionIcon>
                        </Group>
                      </Card>
                    ))}
                  </Stack>
                )}
              </Stack>
            </Card>
          </Stack>
        </Tabs.Panel>

        {/* ==================== TAB: ADVANCED SETTINGS ==================== */}
        <Tabs.Panel value="settings" pt="lg">
          <Stack gap="lg">
            <Card className="glass-pink" p="lg">
              <Title order={4} mb="md">
                GitHub Integration
              </Title>
              <Stack gap="md">
                <Group justify="space-between">
                  <Stack gap={4}>
                    <Text fw={500}>Auto-sync with GitHub</Text>
                    <Text size="sm" c="dimmed">
                      Automatically sync scripts from GitHub repository
                    </Text>
                  </Stack>
                  <Switch
                    size="lg"
                    color="pink"
                    checked={githubSyncEnabled}
                    onChange={(e) => setGithubSyncEnabled(e.currentTarget.checked)}
                  />
                </Group>
                <Button
                  className="btn-pink"
                  leftSection={<GitBranch size={18} />}
                  onClick={handleSyncGitHub}
                  loading={loading}
                  fullWidth
                >
                  Sync Now
                </Button>
              </Stack>
            </Card>

            <Card className="glass-pink" p="lg">
              <Title order={4} mb="md">
                Security Settings
              </Title>
              <Stack gap="md">
                <Group justify="space-between">
                  <Stack gap={4}>
                    <Text fw={500}>Script Encryption</Text>
                    <Text size="sm" c="dimmed">
                      Encrypt scripts before storage
                    </Text>
                  </Stack>
                  <Switch
                    size="lg"
                    color="pink"
                    checked={encryptionEnabled}
                    onChange={(e) => handleToggleEncryption(e.currentTarget.checked)}
                  />
                </Group>
              </Stack>
            </Card>

            <Card className="glass-pink" p="lg">
              <Title order={4} mb="md">
                Application Updates
              </Title>
              <Stack gap="md">
                <Group justify="space-between">
                  <Stack gap={4}>
                    <Text fw={500}>Auto-update</Text>
                    <Text size="sm" c="dimmed">
                      Automatically check for and install updates
                    </Text>
                  </Stack>
                  <Switch
                    size="lg"
                    color="pink"
                    checked={autoUpdateEnabled}
                    onChange={(e) => handleToggleAutoUpdate(e.currentTarget.checked)}
                  />
                </Group>
              </Stack>
            </Card>

            <Card className="glass-pink" p="lg">
              <Title order={4} mb="md">
                System Logs
              </Title>
              <Button
                className="btn-pink-outline"
                leftSection={<Eye size={18} />}
                fullWidth
                onClick={() => {
                  // Navigate to logs viewer
                  setActiveTab('logs');
                }}
              >
                View Application Logs
              </Button>
            </Card>
          </Stack>
        </Tabs.Panel>
      </Tabs>

      {/* ==================== CONFIRMATION MODAL ==================== */}
      <Modal
        opened={confirmModal.open}
        onClose={() => setConfirmModal({ open: false, action: '', data: null })}
        title="Confirm Action"
        centered
      >
        <Stack gap="md">
          {confirmModal.action === 'delete_script' && (
            <>
              <Alert icon={<AlertTriangle size={16} />} color="red">
                Are you sure you want to delete script "{confirmModal.data}"? This action cannot be undone.
              </Alert>
              <Group justify="flex-end">
                <Button variant="light" onClick={() => setConfirmModal({ open: false, action: '', data: null })}>
                  Cancel
                </Button>
                <Button color="red" onClick={confirmDeleteScript} loading={loading}>
                  Delete
                </Button>
              </Group>
            </>
          )}

          {confirmModal.action === 'enable_kill_switch' && (
            <>
              <Alert icon={<AlertTriangle size={16} />} color="red">
                Are you sure you want to block the application? This will prevent all users from accessing it.
                {killSwitchReason && (
                  <Text size="sm" mt="sm">
                    Reason: {killSwitchReason}
                  </Text>
                )}
              </Alert>
              <Group justify="flex-end">
                <Button variant="light" onClick={() => setConfirmModal({ open: false, action: '', data: null })}>
                  Cancel
                </Button>
                <Button color="red" onClick={() => confirmToggleKillSwitch(true)} loading={loading}>
                  Block Application
                </Button>
              </Group>
            </>
          )}
        </Stack>
      </Modal>
    </Stack>
  );
}
