import { useState, useEffect } from 'react';
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
  Tooltip
} from '@mantine/core';
import { 
  Github, 
  LogIn, 
  LogOut, 
  Shield, 
  User, 
  ChevronDown,
  ChevronUp,
  ExternalLink
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

interface GitHubLoginProps {
  onAuthChange?: (isAdmin: boolean) => void;
}

export default function GitHubLogin({ onAuthChange }: GitHubLoginProps) {
  const [token, setToken] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [user, setUser] = useState<GitHubUser | null>(null);
  const [isAdmin, setIsAdmin] = useState(false);
  const [showTokenInput, setShowTokenInput] = useState(false);
  const [showInstructions, setShowInstructions] = useState(false);

  useEffect(() => {
    loadCurrentUser();
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

  const handleLogin = async () => {
    if (!token.trim()) {
      setError('Please enter a GitHub Personal Access Token');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const session = await invoke<AuthSession>('github_login', { token: token.trim() });
      setUser(session.user);
      setIsAdmin(session.is_admin);
      setToken('');
      setShowTokenInput(false);
      onAuthChange?.(session.is_admin);
      
      if (!session.is_admin) {
        setError('Login successful, but you are not an admin. You can still use the app normally.');
      }
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

        {!showTokenInput ? (
          <Stack gap="md">
            <Button
              fullWidth
              leftSection={<LogIn size={18} />}
              onClick={() => setShowTokenInput(true)}
              variant="light"
              color="blue"
            >
              Login with GitHub Personal Access Token
            </Button>

            <Button
              fullWidth
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
