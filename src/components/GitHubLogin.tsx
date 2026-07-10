import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Card,
  Stack,
  TextInput,
  Button,
  Alert,
  Group,
  Avatar,
  Text,
  Badge,
  Code,
  Collapse,
  ActionIcon,
  Tooltip,
  CopyButton,
  Loader,
  Divider,
} from '@mantine/core';
import {
  Github,
  LogIn,
  LogOut,
  Shield,
  ChevronDown,
  ChevronUp,
  ExternalLink,
  Copy,
  Check,
} from 'lucide-react';

interface GitHubUser {
  login: string;
  id: number;
  name: string | null;
  avatar_url: string;
}

interface AuthSession {
  token: string;
  user: GitHubUser;
  is_admin: boolean;
  created_at: string;
}

interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

// Tagged union returned by poll_github_device_flow (serde tag = "status")
type DeviceFlowPoll =
  | { status: 'pending' }
  | { status: 'slow_down'; interval: number }
  | { status: 'authorized'; session: AuthSession };

interface GitHubLoginProps {
  onAuthChange?: (isAdmin: boolean) => void;
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export default function GitHubLogin({ onAuthChange }: GitHubLoginProps) {
  const [token, setToken] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [user, setUser] = useState<GitHubUser | null>(null);
  const [isAdmin, setIsAdmin] = useState(false);
  const [showTokenInput, setShowTokenInput] = useState(false);
  const [showInstructions, setShowInstructions] = useState(false);

  // Device flow state
  const [device, setDevice] = useState<DeviceCodeResponse | null>(null);
  const [polling, setPolling] = useState(false);
  const cancelledRef = useRef(false);

  useEffect(() => {
    loadCurrentUser();
    // Stop any in-flight polling if the component unmounts
    return () => {
      cancelledRef.current = true;
    };
  }, []);

  const loadCurrentUser = async () => {
    try {
      const currentUser = await invoke<GitHubUser | null>('get_github_user');
      if (currentUser) {
        setUser(currentUser);
        const adminStatus = await invoke<boolean>('check_admin_status');
        setIsAdmin(adminStatus);
        onAuthChange?.(adminStatus);
      }
    } catch (err) {
      console.error('Failed to load user:', err);
    }
  };

  const finishLogin = (session: AuthSession) => {
    setUser(session.user);
    setIsAdmin(session.is_admin);
    setToken('');
    setShowTokenInput(false);
    setDevice(null);
    setPolling(false);
    onAuthChange?.(session.is_admin);

    if (!session.is_admin) {
      setError('Login successful, but you are not an admin. You can still use the app normally.');
    }
  };

  // --- Device Flow (recommended) ---

  const startDeviceFlow = async () => {
    setError(null);
    setLoading(true);
    cancelledRef.current = false;

    try {
      const data = await invoke<DeviceCodeResponse>('start_github_device_flow');
      setDevice(data);
      setLoading(false);

      // Open the verification page automatically as a convenience
      try {
        window.open(data.verification_uri, '_blank');
      } catch {
        /* user can still open it manually */
      }

      pollForToken(data);
    } catch (err) {
      setError(`Could not start GitHub login: ${err}`);
      setLoading(false);
    }
  };

  const pollForToken = async (data: DeviceCodeResponse) => {
    setPolling(true);
    let intervalMs = Math.max(data.interval, 1) * 1000;
    const deadline = Date.now() + data.expires_in * 1000;

    while (!cancelledRef.current) {
      await sleep(intervalMs);
      if (cancelledRef.current) return;

      if (Date.now() > deadline) {
        setError('The verification code expired. Please start the login again.');
        setDevice(null);
        setPolling(false);
        return;
      }

      try {
        const result = await invoke<DeviceFlowPoll>('poll_github_device_flow', {
          deviceCode: data.device_code,
        });

        if (result.status === 'authorized') {
          finishLogin(result.session);
          return;
        }
        if (result.status === 'slow_down') {
          intervalMs = Math.max(result.interval, 1) * 1000;
        }
        // 'pending' -> keep waiting
      } catch (err) {
        // Terminal errors (expired_token, access_denied, ...) come back as Err
        setError(`Login failed: ${err}`);
        setDevice(null);
        setPolling(false);
        return;
      }
    }
  };

  const cancelDeviceFlow = () => {
    cancelledRef.current = true;
    setDevice(null);
    setPolling(false);
    setError(null);
  };

  // --- Personal Access Token (fallback) ---

  const handleLogin = async () => {
    if (!token.trim()) {
      setError('Please enter a GitHub Personal Access Token');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const session = await invoke<AuthSession>('github_login', { token: token.trim() });
      finishLogin(session);
    } catch (err) {
      setError(`Login failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleLogout = async () => {
    setLoading(true);
    try {
      await invoke('github_logout');
      setUser(null);
      setIsAdmin(false);
      setToken('');
      onAuthChange?.(false);
    } catch (err) {
      setError(`Logout failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  if (user) {
    return (
      <Card withBorder p="md" className="glass-pink">
        <Group justify="space-between">
          <Group gap="md">
            <Avatar src={user.avatar_url} alt={user.login} size="lg" radius="md" />
            <Stack gap={4}>
              <Group gap="xs">
                <Text fw={600}>{user.name || user.login}</Text>
                {isAdmin && (
                  <Badge
                    leftSection={<Shield size={12} />}
                    variant="gradient"
                    gradient={{ from: 'pink', to: 'grape', deg: 135 }}
                  >
                    Admin
                  </Badge>
                )}
              </Group>
              <Text size="sm" c="dimmed">@{user.login}</Text>
            </Stack>
          </Group>

          <Tooltip label="Logout from GitHub">
            <ActionIcon
              variant="light"
              color="red"
              size="lg"
              onClick={handleLogout}
              loading={loading}
            >
              <LogOut size={18} />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Card>
    );
  }

  return (
    <Card withBorder p="lg" className="glass-pink">
      <Stack gap="md">
        <Group justify="space-between">
          <Group gap="xs">
            <Github size={24} />
            <Text fw={600} size="lg">GitHub Login (Optional)</Text>
          </Group>
          <Badge variant="light" color="blue">Not required</Badge>
        </Group>

        <Alert color="blue" variant="light">
          Login with GitHub to access Admin Panel. Regular features work without login.
        </Alert>

        {error && (
          <Alert color="red" variant="light" onClose={() => setError(null)} withCloseButton>
            {error}
          </Alert>
        )}

        {device ? (
          // --- Active device flow: show the user code and wait ---
          <Card withBorder p="md" bg="gray.0">
            <Stack gap="md" align="center">
              <Text size="sm" ta="center">
                In your browser, open the page below and enter this code:
              </Text>

              <Group gap="xs" justify="center">
                <Code style={{ fontSize: 28, letterSpacing: 4, padding: '8px 16px' }}>
                  {device.user_code}
                </Code>
                <CopyButton value={device.user_code}>
                  {({ copied, copy }) => (
                    <Tooltip label={copied ? 'Copied!' : 'Copy code'}>
                      <ActionIcon variant="light" size="lg" color={copied ? 'green' : 'blue'} onClick={copy}>
                        {copied ? <Check size={18} /> : <Copy size={18} />}
                      </ActionIcon>
                    </Tooltip>
                  )}
                </CopyButton>
              </Group>

              <Button
                variant="light"
                rightSection={<ExternalLink size={14} />}
                onClick={() => window.open(device.verification_uri, '_blank')}
              >
                Open {device.verification_uri.replace('https://', '')}
              </Button>

              <Divider w="100%" />

              <Group gap="xs">
                {polling && <Loader size="xs" />}
                <Text size="sm" c="dimmed">
                  {polling ? 'Waiting for you to authorize in the browser…' : 'Preparing…'}
                </Text>
              </Group>

              <Button variant="subtle" color="red" size="xs" onClick={cancelDeviceFlow}>
                Cancel
              </Button>
            </Stack>
          </Card>
        ) : !showTokenInput ? (
          <Stack gap="md">
            <Button
              fullWidth
              leftSection={<Github size={18} />}
              onClick={startDeviceFlow}
              loading={loading}
              variant="gradient"
              gradient={{ from: 'pink', to: 'grape', deg: 135 }}
            >
              Login with GitHub
            </Button>

            <Button
              fullWidth
              variant="subtle"
              size="xs"
              leftSection={<LogIn size={14} />}
              onClick={() => setShowTokenInput(true)}
            >
              Use a Personal Access Token instead
            </Button>
          </Stack>
        ) : (
          <Stack gap="md">
            <TextInput
              label="GitHub Personal Access Token"
              description="Token with 'repo' scope to access private admin config"
              placeholder="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
              value={token}
              onChange={(e) => setToken(e.currentTarget.value)}
              type="password"
              autoFocus
            />

            <Button
              variant="subtle"
              size="xs"
              leftSection={showInstructions ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
              onClick={() => setShowInstructions(!showInstructions)}
            >
              How to get a GitHub token?
            </Button>

            <Collapse in={showInstructions}>
              <Card withBorder p="sm" bg="gray.0">
                <Stack gap="xs">
                  <Text size="sm" fw={600}>Steps to create a GitHub Personal Access Token:</Text>
                  <Text size="sm" component="ol" style={{ paddingLeft: 20 }}>
                    <li>Go to GitHub → Settings → Developer settings</li>
                    <li>Click "Personal access tokens" → "Tokens (classic)"</li>
                    <li>Click "Generate new token (classic)"</li>
                    <li>Name it "ScriptRunner Admin" and select scopes:
                      <Code block mt="xs">repo (Full control of private repositories)</Code>
                    </li>
                    <li>Click "Generate token" and copy it</li>
                  </Text>
                  <Button
                    size="xs"
                    variant="light"
                    rightSection={<ExternalLink size={14} />}
                    onClick={() => window.open('https://github.com/settings/tokens/new', '_blank')}
                  >
                    Open GitHub Token Page
                  </Button>
                </Stack>
              </Card>
            </Collapse>

            <Group grow>
              <Button
                variant="default"
                onClick={() => {
                  setShowTokenInput(false);
                  setToken('');
                  setError(null);
                }}
              >
                Cancel
              </Button>
              <Button
                leftSection={<LogIn size={18} />}
                onClick={handleLogin}
                loading={loading}
                color="blue"
              >
                Login
              </Button>
            </Group>
          </Stack>
        )}
      </Stack>
    </Card>
  );
}
