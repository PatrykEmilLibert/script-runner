import { Card, SimpleGrid, Group, Stack, Title, Text, Button, ThemeIcon, Badge, Skeleton, Divider, Grid, ActionIcon } from "@mantine/core";
import { 
  BarChart3, Plus, Lock, FileText, Upload,
  CheckCircle, XCircle, Clock, RefreshCw,
  Star
} from "lucide-react";
import { useState, useEffect, useCallback, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";

interface DashboardProps {
  scripts: string[];
  onAddScript: () => void;
  isAdmin: boolean;
  officialScripts: string[];
  favorites?: string[];
  onToggleFavorite?: (script: string, isFav: boolean) => void;
  onRunScript?: (script: string) => void;
}

interface Stats {
  totalScripts: number;
  lastSync: number;
  appStatus: string;
}

interface RecentRun {
  timestamp: string;
  scriptName: string;
  status: "SUCCESS" | "FAILED";
  duration: number;
}

interface HistoryRunRecord {
  script_name: string;
  start_time: string;
  duration_ms: number;
  status: string;
}

export default function Dashboard({ scripts, onAddScript, isAdmin, officialScripts, favorites = [], onToggleFavorite, onRunScript }: DashboardProps) {
  const [stats, setStats] = useState<Stats>({
    totalScripts: scripts.length + officialScripts.length,
    lastSync: Date.now(),
    appStatus: "Healthy",
  });
  const [recentRuns, setRecentRuns] = useState<RecentRun[]>([]);
  const [loading, setLoading] = useState(true);
  const [syncCountdown, setSyncCountdown] = useState(0);

  // Load dashboard data once on mount
  const loadDashboardData = useCallback(async () => {
    try {
      setLoading(true);
      
      const historyRecords = await invoke<HistoryRunRecord[]>("get_run_history", { limit: 100 });
      const runsData: RecentRun[] = historyRecords.map((record) => ({
        timestamp: record.start_time,
        scriptName: record.script_name,
        status: record.status === "success" ? "SUCCESS" : "FAILED",
        duration: Math.round((record.duration_ms / 1000) * 10) / 10,
      }));

      setStats({
        totalScripts: scripts.length + officialScripts.length,
        lastSync: Date.now(),
        appStatus: "Healthy",
      });
      setRecentRuns(runsData);

    } catch (error) {
      console.error("Failed to load dashboard data:", error);
      setRecentRuns([]);
      setStats({
        totalScripts: scripts.length + officialScripts.length,
        lastSync: Date.now(),
        appStatus: "Healthy",
      });
    } finally {
      setLoading(false);
    }
  }, [scripts.length, officialScripts.length]);

  // Load data on mount and setup 30-minute sync interval
  useEffect(() => {
    loadDashboardData();
    
    // Setup 30-minute auto-sync interval
    const syncInterval = setInterval(() => {
      loadDashboardData();
      setSyncCountdown(1800); // 30 minutes
    }, 1800000); // 30 minutes in ms

    return () => clearInterval(syncInterval);
  }, [loadDashboardData]);

  // Countdown timer for next sync (non-blocking) - DISABLED to prevent jumping
  // useEffect(() => {
  //   if (syncCountdown <= 0) return;
  //   
  //   const timer = setTimeout(() => {
  //     setSyncCountdown(syncCountdown - 1);
  //   }, 1000);
  //   
  //   return () => clearTimeout(timer);
  // }, [syncCountdown]);

  const handleManualSync = async () => {
    await loadDashboardData();
    setSyncCountdown(1800);
  };

  const favoriteScripts = scripts.filter((s) => favorites.includes(s));
  const officialFavorites = officialScripts.filter((s) => favorites.includes(s));
  const allFavorites = [...favoriteScripts, ...officialFavorites];

  const formatTime = (seconds: number) => {
    if (seconds < 60) return `${seconds}s ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    return `${Math.floor(seconds / 3600)}h ago`;
  };

  const formatSyncCountdown = (seconds: number) => {
    if (seconds <= 0) return "Auto-sync enabled";
    const mins = Math.floor(seconds / 60);
    return `Next sync in ~${mins} minutes`;
  };

  // Stat Card component
  const StatCard = ({ 
    icon: Icon, 
    title, 
    value, 
    subtitle,
  }: { 
    icon: any; 
    title: string; 
    value: string | number; 
    subtitle?: ReactNode;
  }) => (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
    >
      <Card 
        withBorder 
        p="lg" 
        radius="md" 
        className="glass-pink card-pink"
        style={{ cursor: "pointer", position: "relative", overflow: "hidden" }}
      >
        <div 
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: "linear-gradient(135deg, rgba(255,20,147,0.1) 0%, rgba(219,112,147,0.05) 100%)",
            pointerEvents: "none",
          }}
        />
        <Group justify="space-between" mb="xs" style={{ position: "relative", zIndex: 1 }}>
          <Title order={4}>{title}</Title>
          <ThemeIcon 
            variant="gradient"
            gradient={{ from: "pink", to: "grape", deg: 135 }}
            size="lg" 
            radius="md"
            className="pink-glow"
          >
            <Icon size={18} />
          </ThemeIcon>
        </Group>
        <Text fw={700} size="xl" style={{ position: "relative", zIndex: 1 }}>
          {value}
        </Text>
        {subtitle && (
          <Text size="sm" c="dimmed" mt={4} style={{ position: "relative", zIndex: 1 }}>
            {subtitle}
          </Text>
        )}
      </Card>
    </motion.div>
  );

  if (loading) {
    return (
      <Stack gap="lg">
        <SimpleGrid cols={{ base: 1, sm: 2, md: 3 }} spacing="lg">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} height={120} radius="md" className="pink-glow" />
          ))}
        </SimpleGrid>
      </Stack>
    );
  }

  return (
    <Stack gap="xl">
      {/* Quick Stats - Only 2 cards */}
      <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="lg">
        <StatCard 
          icon={BarChart3} 
          title="Total Scripts" 
          value={stats.totalScripts}
          subtitle="Official + User scripts"
        />
        <StatCard 
          icon={Clock} 
          title="Last Sync" 
          value={formatTime(Math.floor((Date.now() - stats.lastSync) / 1000))}
          subtitle={
            <Group gap="xs" mt="xs">
              <Text size="sm" c="dimmed">{formatSyncCountdown(syncCountdown)}</Text>
              <Button
                leftSection={<RefreshCw size={14} />}
                size="xs"
                variant="light"
                color="pink"
                onClick={handleManualSync}
              >
                Sync Now
              </Button>
            </Group>
          }
        />
      </SimpleGrid>

      {/* Favorite Scripts Bar - Full Width */}
      <Card withBorder p="xl" radius="md" className="glass-pink">
        <Group justify="space-between" mb="md">
          <Group gap="xs">
            <Star size={24} color="#FF1493" fill="#FF1493" />
            <Title order={3}>⭐ Favorite Scripts</Title>
            <Badge variant="light" color="pink">{allFavorites.length} favorites</Badge>
          </Group>
        </Group>

        {allFavorites.length === 0 ? (
          <Text c="dimmed" ta="center" py="lg">
            No favorite scripts yet. Click the star icon next to a script to add it here.
          </Text>
        ) : (
          <SimpleGrid cols={{ base: 2, sm: 3, md: 4, lg: 5 }} spacing="md">
            {allFavorites.map((script) => (
              <motion.div
                key={script}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.2 }}
              >
                <Card
                  p="md"
                  radius="md"
                  withBorder
                  className="hover:shadow-lg transition-all cursor-pointer"
                  onClick={() => onRunScript?.(script)}
                  style={{
                    background: "linear-gradient(135deg, rgba(255,20,147,0.05) 0%, rgba(219,112,147,0.03) 100%)",
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    textAlign: "center",
                    cursor: onRunScript ? "pointer" : "default",
                  }}
                >
                  <Group justify="center" mb="xs" w="100%">
                    <ThemeIcon variant="light" color="pink" size="lg" radius="md">
                      <FileText size={18} />
                    </ThemeIcon>
                  </Group>
                  <Text size="sm" fw={500} lineClamp={2} mb="xs">
                    {script}
                  </Text>
                  <Button
                    size="xs"
                    variant="light"
                    color="pink"
                    fullWidth
                    mb="xs"
                    onClick={(e) => {
                      e.stopPropagation();
                      onRunScript?.(script);
                    }}
                  >
                    Run
                  </Button>
                  <ActionIcon
                    size="sm"
                    variant="light"
                    color="pink"
                    onClick={(e) => {
                      e.stopPropagation();
                      onToggleFavorite?.(script, false);
                    }}
                  >
                    <Star size={14} fill="#FF1493" color="#FF1493" />
                  </ActionIcon>
                </Card>
              </motion.div>
            ))}
          </SimpleGrid>
        )}
      </Card>

      {/* Recent Activity Feed - Expanded */}
      <Card withBorder p="xl" radius="md" className="glass-pink">
        <Group justify="space-between" mb="md">
          <Title order={3}>📋 Recent Activity</Title>
          <Badge variant="light" color="pink">{recentRuns.length} runs</Badge>
        </Group>
        <Stack gap="xs">
          {recentRuns.slice(0, 15).map((run, idx) => (
            <motion.div
              key={idx}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: idx * 0.05 }}
            >
              <Group 
                p="sm" 
                style={{ borderRadius: "8px" }} 
                className="hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                <ThemeIcon
                  variant="light"
                  color={run.status === "SUCCESS" ? "pink" : "red"}
                  size="md"
                  radius="xl"
                >
                  {run.status === "SUCCESS" ? (
                    <CheckCircle size={16} />
                  ) : (
                    <XCircle size={16} />
                  )}
                </ThemeIcon>
                <div style={{ flex: 1 }}>
                  <Text size="sm" fw={500}>{run.scriptName}</Text>
                  <Text size="xs" c="dimmed">
                    {new Date(run.timestamp).toLocaleTimeString()} • {run.duration}s
                  </Text>
                </div>
                <Badge 
                  size="sm" 
                  color={run.status === "SUCCESS" ? "pink" : "red"}
                  variant="light"
                >
                  {run.status}
                </Badge>
              </Group>
            </motion.div>
          ))}
        </Stack>
      </Card>
    </Stack>
  );
}
