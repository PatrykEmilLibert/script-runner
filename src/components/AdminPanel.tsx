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
  Checkbox,
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
import { useNotifications } from '../hooks/useNotifications';

interface AdminPanelProps {
  isAdmin: boolean;
  scriptsDir: string;
  officialDir: string;
  onRefreshScripts: () => void;
}

interface ScriptInfo {
  name: string;
  subdir: 'official' | 'scripts';
  path: string;
  size: number;
  modified: string;
  description: string;
  author: string;
  version: string;
  encrypted: boolean;
}

interface KillSwitchStatus {
  blocked: boolean;
  reason?: string;
  scheduledFor?: string;
  blocked_until?: string;
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

interface AppSettings {
  dark_mode: boolean;
  auto_update_enabled: boolean;
}

interface BulkScriptOperationResult {
  requested: number;
  processed: number;
  skipped: string[];
}

export default function AdminPanel({ isAdmin, scriptsDir, officialDir, onRefreshScripts }: AdminPanelProps) {
  const { showSuccess, showError } = useNotifications();

  const readInputValue = (payload: unknown): string => {
    if (typeof payload === 'string') return payload;
    if (payload && typeof payload === 'object') {
      const eventLike = payload as { currentTarget?: { value?: unknown }; target?: { value?: unknown } };
      const current = eventLike.currentTarget?.value;
      if (typeof current === 'string') return current;
      const target = eventLike.target?.value;
      if (typeof target === 'string') return target;
    }
    return '';
  };
  
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
  const [scriptPreview, setScriptPreview] = useState({
    open: false,
    title: '',
    content: '',
    subdir: 'official' as 'official' | 'scripts',
  });
  const [replaceModal, setReplaceModal] = useState({
    open: false,
    scriptName: '',
    content: '',
  });
  const [editModal, setEditModal] = useState({
    open: false,
    originalName: '',
    name: '',
    description: '',
    author: '',
    version: '',
    content: '',
  });
  const [selectedOfficialScripts, setSelectedOfficialScripts] = useState<string[]>([]);
  const [bulkMetadataModal, setBulkMetadataModal] = useState({
    open: false,
    author: '',
    version: '',
    descriptionPrefix: '',
  });
  
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
    loadAppSettings();
  }, []);

  useEffect(() => {
    const officialNames = new Set(
      scripts.filter((script) => script.subdir === 'official').map((script) => script.name)
    );

    setSelectedOfficialScripts((prev) => {
      const filtered = prev.filter((name) => officialNames.has(name));
      if (filtered.length === prev.length && filtered.every((name, index) => name === prev[index])) {
        return prev;
      }
      return filtered;
    });
  }, [scripts]);

  // ==================== DATA LOADING ====================
  
  const loadStats = async () => {
    try {
      const allScripts = await invoke<ScriptInfo[]>('get_all_scripts_info', { scriptsDir });
      const official = allScripts.filter((s) => s.subdir === 'official');
      const user = allScripts.filter((s) => s.subdir === 'scripts');

      const killSwitch = await invoke<{ blocked?: boolean }>('check_kill_switch_status').catch(() => ({ blocked: false }));
      const appBlocked = killSwitch?.blocked === true;
      
      setStats({
        totalScripts: official.length + user.length,
        officialScripts: official.length,
        userScripts: user.length,
        activeSessions: 1, // Current session
        lastSync: new Date().toLocaleString(),
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
      setScripts(
        basic.map((name) => ({
          name,
          subdir: 'official' as const,
          path: `official/${name}`,
          size: 0,
          modified: 'Unknown',
          description: '',
          author: '',
          version: '1.0.0',
          encrypted: false,
        }))
      );
    }
  };

  const loadKillSwitchStatus = async () => {
    try {
      const status = await invoke<KillSwitchStatus>('get_kill_switch_status');
      setKillSwitchEnabled(status.blocked);
      setKillSwitchReason(status.reason || '');
      setScheduledDate(status.blocked_until || status.scheduledFor || '');
      setWhitelist(status.whitelist || []);
    } catch (error) {
      console.error('Failed to load kill switch status:', error);
    }
  };

  const loadAppSettings = async () => {
    try {
      const settings = await invoke<AppSettings>('get_settings');
      setAutoUpdateEnabled(settings.auto_update_enabled ?? true);
    } catch (error) {
      console.error('Failed to load app settings:', error);
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
      await invoke('delete_script', {
        scriptName: confirmModal.data,
        scriptsDir,
        subdir: 'official',
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
      await invoke('encrypt_official_script', { scriptName });
      showSuccess('Script Encrypted', `Script "${scriptName}" encrypted successfully`);
      await loadScripts();
    } catch (error) {
      showError('Encryption Failed', `Failed to encrypt script: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handlePreviewScript = async (scriptName: string, subdir: 'official' | 'scripts') => {
    try {
      setLoading(true);
      const content = await invoke<string>('get_script_source', { scriptName, scriptsDir, subdir });
      setScriptPreview({
        open: true,
        title: scriptName,
        content,
        subdir,
      });
    } catch (error) {
      showError('Preview Failed', `Failed to load script preview: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const openReplaceModal = async (scriptName: string) => {
    try {
      setLoading(true);
      const content = await invoke<string>('get_script_source', {
        scriptName,
        scriptsDir,
        subdir: 'official',
      });
      setReplaceModal({
        open: true,
        scriptName,
        content,
      });
    } catch (error) {
      showError('Load Failed', `Failed to load script content: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const confirmReplaceScript = async () => {
    if (!replaceModal.scriptName.trim()) return;

    try {
      setLoading(true);
      await invoke('replace_official_script_content', {
        scriptName: replaceModal.scriptName,
        scriptContent: replaceModal.content,
        scriptsDir,
      });
      showSuccess('Script Updated', `Code replaced for "${replaceModal.scriptName}"`);
      setReplaceModal({ open: false, scriptName: '', content: '' });
      await loadScripts();
      await loadStats();
      if (onRefreshScripts) onRefreshScripts();
    } catch (error) {
      showError('Replace Failed', `Failed to replace script code: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const openEditModal = async (script: ScriptInfo) => {
    try {
      setLoading(true);
      const content = await invoke<string>('get_script_source', {
        scriptName: script.name,
        scriptsDir,
        subdir: 'official',
      });
      setEditModal({
        open: true,
        originalName: script.name,
        name: script.name,
        description: script.description || '',
        author: script.author || '',
        version: script.version || '1.0.0',
        content,
      });
    } catch (error) {
      showError('Load Failed', `Failed to load script for editing: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const confirmFullEditScript = async () => {
    if (!editModal.name.trim()) {
      showError('Validation Error', 'Script name cannot be empty');
      return;
    }

    try {
      setLoading(true);
      await invoke('update_official_script_full', {
        currentName: editModal.originalName,
        newName: editModal.name,
        scriptContent: editModal.content,
        description: editModal.description,
        author: editModal.author,
        version: editModal.version,
        scriptsDir,
      });
      showSuccess('Script Saved', `Official script "${editModal.name}" updated`);
      setEditModal({
        open: false,
        originalName: '',
        name: '',
        description: '',
        author: '',
        version: '',
        content: '',
      });
      await loadScripts();
      await loadStats();
      if (onRefreshScripts) onRefreshScripts();
    } catch (error) {
      showError('Save Failed', `Failed to save script: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const toggleOfficialScriptSelection = (scriptName: string, selected: boolean) => {
    setSelectedOfficialScripts((prev) => {
      if (selected) {
        if (prev.includes(scriptName)) {
          return prev;
        }
        return [...prev, scriptName];
      }
      return prev.filter((name) => name !== scriptName);
    });
  };

  const selectAllOfficialScripts = () => {
    const officialNames = scripts
      .filter((script) => script.subdir === 'official')
      .map((script) => script.name);
    setSelectedOfficialScripts(officialNames);
  };

  const selectOnlyUnencryptedOfficialScripts = () => {
    const unencrypted = scripts
      .filter((script) => script.subdir === 'official' && !script.encrypted)
      .map((script) => script.name);
    setSelectedOfficialScripts(unencrypted);
  };

  const clearOfficialSelection = () => {
    setSelectedOfficialScripts([]);
  };

  const handleBulkEncryptScripts = async () => {
    if (selectedOfficialScripts.length === 0) {
      showError('No Selection', 'Select at least one official script first');
      return;
    }

    try {
      setLoading(true);
      const result = await invoke<BulkScriptOperationResult>('bulk_encrypt_official_scripts', {
        scriptNames: selectedOfficialScripts,
        scriptsDir,
      });

      const suffix = result.skipped.length > 0 ? `, skipped ${result.skipped.length}` : '';
      showSuccess(
        'Bulk Encryption Complete',
        `Encrypted ${result.processed}/${result.requested} official scripts${suffix}`
      );
      await loadScripts();
    } catch (error) {
      showError('Bulk Encryption Failed', `Failed to encrypt selected scripts: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleBulkDeleteScripts = () => {
    if (selectedOfficialScripts.length === 0) {
      showError('No Selection', 'Select at least one official script first');
      return;
    }

    setConfirmModal({
      open: true,
      action: 'bulk_delete_scripts',
      data: [...selectedOfficialScripts],
    });
  };

  const confirmBulkDeleteScripts = async () => {
    const selectedNames = Array.isArray(confirmModal.data) ? confirmModal.data : [];
    if (selectedNames.length === 0) {
      setConfirmModal({ open: false, action: '', data: null });
      return;
    }

    try {
      setLoading(true);
      const result = await invoke<BulkScriptOperationResult>('bulk_delete_official_scripts', {
        scriptNames: selectedNames,
        scriptsDir,
      });

      const suffix = result.skipped.length > 0 ? `, skipped ${result.skipped.length}` : '';
      showSuccess(
        'Bulk Delete Complete',
        `Deleted ${result.processed}/${result.requested} official scripts${suffix}`
      );

      setSelectedOfficialScripts((prev) => prev.filter((name) => !selectedNames.includes(name)));
      await loadScripts();
      await loadStats();
      if (onRefreshScripts) onRefreshScripts();
    } catch (error) {
      showError('Bulk Delete Failed', `Failed to delete selected scripts: ${error}`);
    } finally {
      setLoading(false);
      setConfirmModal({ open: false, action: '', data: null });
    }
  };

  const openBulkMetadataModal = () => {
    if (selectedOfficialScripts.length === 0) {
      showError('No Selection', 'Select at least one official script first');
      return;
    }

    setBulkMetadataModal({
      open: true,
      author: '',
      version: '',
      descriptionPrefix: '',
    });
  };

  const confirmBulkMetadataUpdate = async () => {
    const author = bulkMetadataModal.author.trim();
    const version = bulkMetadataModal.version.trim();
    const descriptionPrefix = bulkMetadataModal.descriptionPrefix.trim();

    if (!author && !version && !descriptionPrefix) {
      showError('Validation Error', 'Provide at least one metadata field to update');
      return;
    }

    try {
      setLoading(true);
      const result = await invoke<BulkScriptOperationResult>('bulk_update_official_metadata', {
        scriptNames: selectedOfficialScripts,
        scriptsDir,
        author: author || null,
        version: version || null,
        descriptionPrefix: descriptionPrefix || null,
      });

      const suffix = result.skipped.length > 0 ? `, skipped ${result.skipped.length}` : '';
      showSuccess(
        'Bulk Metadata Updated',
        `Updated ${result.processed}/${result.requested} official scripts${suffix}`
      );

      setBulkMetadataModal({
        open: false,
        author: '',
        version: '',
        descriptionPrefix: '',
      });
      await loadScripts();
      await loadStats();
      if (onRefreshScripts) onRefreshScripts();
    } catch (error) {
      showError('Bulk Update Failed', `Failed to update metadata: ${error}`);
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
      await invoke('sync_scripts');
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

  const officialScripts = scripts.filter((script) => script.subdir === 'official');
  const userScripts = scripts.filter((script) => script.subdir === 'scripts');
  const selectedOfficialSet = new Set(selectedOfficialScripts);

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

            <Alert color="blue" variant="light">
              This view shows <Code>official/</Code> scripts and your own namespace in <Code>scripts/</Code>.
              User scripts are pushed to GitHub, but the app only exposes your own user scripts on this installation/account.
            </Alert>

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
                <Group justify="space-between">
                  <Title order={4}>Official Scripts</Title>
                  <Badge color="pink" variant="light">
                    {officialScripts.length}
                  </Badge>
                </Group>

                <Card withBorder p="md">
                  <Stack gap="sm">
                    <Group justify="space-between" align="center">
                      <Text fw={600}>Bulk Actions</Text>
                      <Badge
                        color={selectedOfficialScripts.length > 0 ? 'pink' : 'gray'}
                        variant="light"
                      >
                        {selectedOfficialScripts.length} selected
                      </Badge>
                    </Group>

                    <Group>
                      <Button
                        size="xs"
                        variant="light"
                        onClick={selectAllOfficialScripts}
                        disabled={officialScripts.length === 0}
                      >
                        Select All
                      </Button>
                      <Button
                        size="xs"
                        variant="light"
                        onClick={selectOnlyUnencryptedOfficialScripts}
                        disabled={officialScripts.length === 0}
                      >
                        Select Unencrypted
                      </Button>
                      <Button
                        size="xs"
                        variant="subtle"
                        onClick={clearOfficialSelection}
                        disabled={selectedOfficialScripts.length === 0}
                      >
                        Clear Selection
                      </Button>
                    </Group>

                    <Group>
                      <Button
                        size="xs"
                        variant="light"
                        color="violet"
                        leftSection={<Lock size={14} />}
                        onClick={handleBulkEncryptScripts}
                        disabled={selectedOfficialScripts.length === 0}
                        loading={loading}
                      >
                        Encrypt Selected
                      </Button>
                      <Button
                        size="xs"
                        variant="light"
                        color="pink"
                        leftSection={<Edit size={14} />}
                        onClick={openBulkMetadataModal}
                        disabled={selectedOfficialScripts.length === 0}
                        loading={loading}
                      >
                        Edit Metadata
                      </Button>
                      <Button
                        size="xs"
                        variant="light"
                        color="red"
                        leftSection={<Trash2 size={14} />}
                        onClick={handleBulkDeleteScripts}
                        disabled={selectedOfficialScripts.length === 0}
                        loading={loading}
                      >
                        Delete Selected
                      </Button>
                    </Group>
                  </Stack>
                </Card>

                {officialScripts.length === 0 ? (
                  <Text c="dimmed" ta="center" py="lg">
                    No official scripts found
                  </Text>
                ) : (
                  officialScripts.map((script) => (
                      <Card key={`official-${script.name}`} withBorder p="md">
                        <Group justify="space-between" align="flex-start" wrap="nowrap">
                          <Group gap="sm" align="flex-start" style={{ flex: 1 }}>
                            <Checkbox
                              checked={selectedOfficialSet.has(script.name)}
                              onChange={(event) =>
                                toggleOfficialScriptSelection(
                                  script.name,
                                  event.currentTarget.checked
                                )
                              }
                              mt={4}
                              aria-label={`Select script ${script.name}`}
                            />
                            <Stack gap={4} style={{ flex: 1 }}>
                              <Group gap="xs">
                                <Text fw={600}>{script.name}</Text>
                                {script.encrypted && (
                                  <Badge size="xs" color="violet" variant="light">Encrypted</Badge>
                                )}
                                <Badge size="xs" color="pink" variant="light">v{script.version || '1.0.0'}</Badge>
                              </Group>
                              <Text size="sm" c="dimmed">{script.description || 'No description'}</Text>
                              <Group gap="xs">
                                <Badge size="sm" variant="light">{(script.size / 1024).toFixed(1)} KB</Badge>
                                <Text size="xs" c="dimmed">Author: {script.author || 'Unknown'}</Text>
                                <Text size="xs" c="dimmed">Modified: {script.modified || 'Unknown'}</Text>
                              </Group>
                            </Stack>
                          </Group>

                          <Group gap="xs">
                            <Tooltip label="Preview script">
                              <ActionIcon
                                variant="light"
                                color="gray"
                                onClick={() => handlePreviewScript(script.name, 'official')}
                              >
                                <Eye size={16} />
                              </ActionIcon>
                            </Tooltip>

                            <Tooltip label="Replace code only (keep metadata)">
                              <ActionIcon
                                variant="light"
                                color="blue"
                                onClick={() => openReplaceModal(script.name)}
                              >
                                <RefreshCw size={16} />
                              </ActionIcon>
                            </Tooltip>

                            <Tooltip label="Edit all data + code">
                              <ActionIcon
                                variant="light"
                                color="pink"
                                onClick={() => openEditModal(script)}
                              >
                                <Edit size={16} />
                              </ActionIcon>
                            </Tooltip>

                            <Tooltip label="Encrypt">
                              <ActionIcon
                                variant="light"
                                color="violet"
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

            <Card className="glass-pink" p="lg">
              <Stack gap="md">
                <Group justify="space-between">
                  <Title order={4}>User Scripts (from repository)</Title>
                  <Badge color="grape" variant="light">
                    {userScripts.length}
                  </Badge>
                </Group>

                {userScripts.length === 0 ? (
                  <Text c="dimmed" ta="center" py="lg">
                    No user scripts found in repository
                  </Text>
                ) : (
                  userScripts.map((script) => (
                      <Card key={`user-${script.name}`} withBorder p="md">
                        <Group justify="space-between" align="flex-start">
                          <Stack gap={4} style={{ flex: 1 }}>
                            <Group gap="xs">
                              <Text fw={600}>{script.name}</Text>
                              <Badge size="xs" color="grape" variant="light">User</Badge>
                              <Badge size="xs" color="gray" variant="light">v{script.version || '1.0.0'}</Badge>
                            </Group>
                            <Text size="sm" c="dimmed">{script.description || 'No description'}</Text>
                            <Group gap="xs">
                              <Badge size="sm" variant="light">{(script.size / 1024).toFixed(1)} KB</Badge>
                              <Text size="xs" c="dimmed">Author: {script.author || 'Unknown'}</Text>
                              <Text size="xs" c="dimmed">Modified: {script.modified || 'Unknown'}</Text>
                            </Group>
                          </Stack>

                          <Tooltip label="Preview script">
                            <ActionIcon
                              variant="light"
                              color="gray"
                              onClick={() => handlePreviewScript(script.name, 'scripts')}
                            >
                              <Eye size={16} />
                            </ActionIcon>
                          </Tooltip>
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
                  onChange={(e) => setKillSwitchReason(readInputValue(e))}
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
                  onChange={(e) => setScheduledDate(readInputValue(e))}
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
                    onChange={(e) => setNewWhitelistItem(readInputValue(e))}
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

      <Modal
        opened={scriptPreview.open}
        onClose={() =>
          setScriptPreview({ open: false, title: '', content: '', subdir: 'official' })
        }
        title={`Preview: ${scriptPreview.title} (${scriptPreview.subdir})`}
        size="xl"
        centered
      >
        <Stack gap="md">
          <Code block>{scriptPreview.content || '# Empty script'}</Code>
        </Stack>
      </Modal>

      <Modal
        opened={replaceModal.open}
        onClose={() => setReplaceModal({ open: false, scriptName: '', content: '' })}
        title={`Replace Code: ${replaceModal.scriptName}`}
        size="xl"
        centered
      >
        <Stack gap="md">
          <Alert color="blue" variant="light">
            This updates only script code and keeps existing metadata.
          </Alert>
          <Textarea
            value={replaceModal.content}
            onChange={(e) =>
              setReplaceModal((prev) => ({
                ...prev,
                content: readInputValue(e),
              }))
            }
            minRows={16}
            autosize
            label="main.py content"
          />
          <Group justify="flex-end">
            <Button
              variant="light"
              onClick={() => setReplaceModal({ open: false, scriptName: '', content: '' })}
            >
              Cancel
            </Button>
            <Button className="btn-pink" leftSection={<Save size={16} />} loading={loading} onClick={confirmReplaceScript}>
              Replace Code
            </Button>
          </Group>
        </Stack>
      </Modal>

      <Modal
        opened={editModal.open}
        onClose={() =>
          setEditModal({
            open: false,
            originalName: '',
            name: '',
            description: '',
            author: '',
            version: '',
            content: '',
          })
        }
        title={`Edit Script: ${editModal.originalName}`}
        size="xl"
        centered
      >
        <Stack gap="md">
          <TextInput
            label="Script Name"
            value={editModal.name}
            onChange={(e) => setEditModal((prev) => ({ ...prev, name: readInputValue(e) }))}
          />
          <TextInput
            label="Description"
            value={editModal.description}
            onChange={(e) =>
              setEditModal((prev) => ({ ...prev, description: readInputValue(e) }))
            }
          />
          <Group grow>
            <TextInput
              label="Author"
              value={editModal.author}
              onChange={(e) =>
                setEditModal((prev) => ({ ...prev, author: readInputValue(e) }))
              }
            />
            <TextInput
              label="Version"
              value={editModal.version}
              onChange={(e) =>
                setEditModal((prev) => ({ ...prev, version: readInputValue(e) }))
              }
            />
          </Group>
          <Textarea
            label="main.py content"
            value={editModal.content}
            onChange={(e) => setEditModal((prev) => ({ ...prev, content: readInputValue(e) }))}
            minRows={16}
            autosize
          />
          <Group justify="flex-end">
            <Button
              variant="light"
              onClick={() =>
                setEditModal({
                  open: false,
                  originalName: '',
                  name: '',
                  description: '',
                  author: '',
                  version: '',
                  content: '',
                })
              }
            >
              Cancel
            </Button>
            <Button className="btn-pink" leftSection={<Save size={16} />} loading={loading} onClick={confirmFullEditScript}>
              Save All Changes
            </Button>
          </Group>
        </Stack>
      </Modal>

      <Modal
        opened={bulkMetadataModal.open}
        onClose={() =>
          setBulkMetadataModal({
            open: false,
            author: '',
            version: '',
            descriptionPrefix: '',
          })
        }
        title={`Bulk Metadata Update (${selectedOfficialScripts.length} selected)`}
        centered
      >
        <Stack gap="md">
          <Alert color="blue" variant="light">
            Fill only the fields you want to apply to all selected official scripts.
          </Alert>
          <TextInput
            label="Author"
            placeholder="Set author for all selected scripts"
            value={bulkMetadataModal.author}
            onChange={(e) =>
              setBulkMetadataModal((prev) => ({ ...prev, author: readInputValue(e) }))
            }
          />
          <TextInput
            label="Version"
            placeholder="Set version for all selected scripts"
            value={bulkMetadataModal.version}
            onChange={(e) =>
              setBulkMetadataModal((prev) => ({ ...prev, version: readInputValue(e) }))
            }
          />
          <TextInput
            label="Description Prefix"
            placeholder="Prefix added to beginning of each description"
            value={bulkMetadataModal.descriptionPrefix}
            onChange={(e) =>
              setBulkMetadataModal((prev) => ({
                ...prev,
                descriptionPrefix: readInputValue(e),
              }))
            }
          />
          <Group justify="flex-end">
            <Button
              variant="light"
              onClick={() =>
                setBulkMetadataModal({
                  open: false,
                  author: '',
                  version: '',
                  descriptionPrefix: '',
                })
              }
            >
              Cancel
            </Button>
            <Button
              className="btn-pink"
              leftSection={<Save size={16} />}
              loading={loading}
              onClick={confirmBulkMetadataUpdate}
            >
              Apply to Selected
            </Button>
          </Group>
        </Stack>
      </Modal>

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

          {confirmModal.action === 'bulk_delete_scripts' && (
            <>
              <Alert icon={<AlertTriangle size={16} />} color="red">
                Are you sure you want to delete
                {' '}
                {Array.isArray(confirmModal.data) ? confirmModal.data.length : 0}
                {' '}
                selected official scripts? This action cannot be undone.
              </Alert>
              {Array.isArray(confirmModal.data) && confirmModal.data.length > 0 && (
                <Code block>
                  {confirmModal.data.slice(0, 10).join('\n')}
                  {confirmModal.data.length > 10 ? '\n...' : ''}
                </Code>
              )}
              <Group justify="flex-end">
                <Button variant="light" onClick={() => setConfirmModal({ open: false, action: '', data: null })}>
                  Cancel
                </Button>
                <Button color="red" onClick={confirmBulkDeleteScripts} loading={loading}>
                  Delete Selected
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
