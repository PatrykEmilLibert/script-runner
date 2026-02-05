import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  LineChart,
  Line,
  BarChart,
  Bar,
  PieChart,
  Pie,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  Cell,
  Label,
} from 'recharts';
import { Paper, Title, Select, Button, Group, Grid, Text, Badge, Stack } from '@mantine/core';
import { notifications } from '@mantine/notifications';
import { Download, RefreshCw, Calendar } from 'lucide-react';

interface DailyUsage {
  date: string;
  executions: number;
  successes: number;
  failures: number;
}

interface ScriptStat {
  name: string;
  executions: number;
  avg_duration: number;
  success_rate: number;
  category: string;
}

interface ExecutionTimeDistribution {
  bucket: string;
  count: number;
}

interface CategoryStats {
  category: string;
  official_count: number;
  user_count: number;
}

interface AnalyticsData {
  usage_over_time: DailyUsage[];
  top_scripts: ScriptStat[];
  success_rate: number;
  avg_execution_time: number;
  total_executions: number;
  total_failures: number;
  execution_time_distribution: ExecutionTimeDistribution[];
  category_stats: CategoryStats[];
}

const Analytics: React.FC = () => {
  const [data, setData] = useState<AnalyticsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [dateRange, setDateRange] = useState<string>('30');

  const loadAnalytics = async (days?: number) => {
    try {
      setLoading(true);
      const analyticsData = await invoke<AnalyticsData>('get_analytics_data', {
        days: days || parseInt(dateRange),
      });
      setData(analyticsData);
    } catch (error) {
      notifications.show({
        title: 'Error',
        message: `Failed to load analytics: ${error}`,
        color: 'red',
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadAnalytics();
  }, [dateRange]);

  const handleExport = async (format: 'json' | 'csv') => {
    try {
      const exported = await invoke<string>('export_analytics', {
        format,
        days: parseInt(dateRange),
      });

      const blob = new Blob([exported], {
        type: format === 'json' ? 'application/json' : 'text/csv',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `analytics_${new Date().toISOString().split('T')[0]}.${format}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      notifications.show({
        title: 'Success',
        message: `Analytics exported as ${format.toUpperCase()}`,
        color: 'green',
      });
    } catch (error) {
      notifications.show({
        title: 'Error',
        message: `Failed to export: ${error}`,
        color: 'red',
      });
    }
  };

  if (loading) {
    return (
      <div style={{ padding: '2rem', textAlign: 'center' }}>
        <Text size="lg">Loading analytics...</Text>
      </div>
    );
  }

  if (!data || data.total_executions === 0) {
    return (
      <div style={{ padding: '2rem', textAlign: 'center' }}>
        <Text size="lg">No analytics data available yet. Run some scripts to see statistics!</Text>
      </div>
    );
  }

  // Prepare pie chart data for success/failure
  const successFailureData = [
    { name: 'Success', value: data.total_executions - data.total_failures, color: '#ec4899' },
    { name: 'Failed', value: data.total_failures, color: '#f87171' },
  ];

  // Prepare execution time distribution data with proper sorting
  const timeBucketOrder = ['0-1s', '1-5s', '5-10s', '10-30s', '30s+'];
  const sortedTimeDistribution = [...data.execution_time_distribution].sort(
    (a, b) => timeBucketOrder.indexOf(a.bucket) - timeBucketOrder.indexOf(b.bucket)
  );

  // Prepare category stacked data
  const categoryStackedData = data.category_stats.map((cat) => ({
    category: cat.category,
    Official: cat.official_count,
    User: cat.user_count,
  }));

  return (
    <div style={{ padding: '1.5rem' }}>
      {/* Header with Controls */}
      <Group justify="space-between" mb="xl">
        <Title order={2}>📊 Analytics Dashboard</Title>
        <Group>
          <Select
            value={dateRange}
            onChange={(value) => setDateRange(value || '30')}
            data={[
              { value: '7', label: 'Last 7 days' },
              { value: '30', label: 'Last 30 days' },
              { value: '90', label: 'Last 90 days' },
            ]}
            leftSection={<Calendar size={16} />}
            style={{ width: 150 }}
          />
          <Button
            variant="light"
            color="pink"
            leftSection={<RefreshCw size={16} />}
            onClick={() => loadAnalytics()}
          >
            Refresh
          </Button>
          <Button
            variant="outline"
            color="pink"
            leftSection={<Download size={16} />}
            onClick={() => handleExport('json')}
          >
            Export JSON
          </Button>
          <Button
            variant="outline"
            color="pink"
            leftSection={<Download size={16} />}
            onClick={() => handleExport('csv')}
          >
            Export CSV
          </Button>
        </Group>
      </Group>

      {/* Key Metrics */}
      <Grid mb="xl">
        <Grid.Col span={{ base: 12, md: 3 }}>
          <Paper shadow="sm" p="md" withBorder>
            <Stack gap="xs">
              <Text size="sm" c="dimmed">
                Total Executions
              </Text>
              <Text size="xl" fw={700}>
                {data.total_executions.toLocaleString()}
              </Text>
            </Stack>
          </Paper>
        </Grid.Col>
        <Grid.Col span={{ base: 12, md: 3 }}>
          <Paper shadow="sm" p="md" withBorder>
            <Stack gap="xs">
              <Text size="sm" c="dimmed">
                Success Rate
              </Text>
              <Text size="xl" fw={700} c={data.success_rate >= 80 ? 'green' : 'orange'}>
                {data.success_rate.toFixed(1)}%
              </Text>
            </Stack>
          </Paper>
        </Grid.Col>
        <Grid.Col span={{ base: 12, md: 3 }}>
          <Paper shadow="sm" p="md" withBorder>
            <Stack gap="xs">
              <Text size="sm" c="dimmed">
                Avg Execution Time
              </Text>
              <Text size="xl" fw={700}>
                {(data.avg_execution_time / 1000).toFixed(2)}s
              </Text>
            </Stack>
          </Paper>
        </Grid.Col>
        <Grid.Col span={{ base: 12, md: 3 }}>
          <Paper shadow="sm" p="md" withBorder>
            <Stack gap="xs">
              <Text size="sm" c="dimmed">
                Total Failures
              </Text>
              <Text size="xl" fw={700} c="red">
                {data.total_failures.toLocaleString()}
              </Text>
            </Stack>
          </Paper>
        </Grid.Col>
      </Grid>

      {/* Charts Grid */}
      <Grid>
        {/* Usage Over Time - Line Chart */}
        <Grid.Col span={{ base: 12, lg: 8 }}>
          <Paper shadow="sm" p="md" withBorder>
            <Title order={4} mb="md">
              Usage Over Time
            </Title>
            <ResponsiveContainer width="100%" height={300}>
              <AreaChart data={data.usage_over_time}>
                <defs>
                  <linearGradient id="colorExecutions" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#ec4899" stopOpacity={0.8} />
                    <stop offset="95%" stopColor="#ec4899" stopOpacity={0.1} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis
                  dataKey="date"
                  stroke="#6b7280"
                  fontSize={12}
                  tickFormatter={(value) => {
                    const date = new Date(value);
                    return `${date.getMonth() + 1}/${date.getDate()}`;
                  }}
                />
                <YAxis stroke="#6b7280" fontSize={12} />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#fff',
                    border: '1px solid #e5e7eb',
                    borderRadius: '8px',
                  }}
                  labelFormatter={(value) => `Date: ${value}`}
                />
                <Legend />
                <Area
                  type="monotone"
                  dataKey="executions"
                  stroke="#ec4899"
                  strokeWidth={2}
                  fillOpacity={1}
                  fill="url(#colorExecutions)"
                  name="Executions"
                />
              </AreaChart>
            </ResponsiveContainer>
          </Paper>
        </Grid.Col>

        {/* Success vs Failed - Pie Chart */}
        <Grid.Col span={{ base: 12, lg: 4 }}>
          <Paper shadow="sm" p="md" withBorder>
            <Title order={4} mb="md">
              Success vs Failed
            </Title>
            <ResponsiveContainer width="100%" height={300}>
              <PieChart>
                <Pie
                  data={successFailureData}
                  cx="50%"
                  cy="50%"
                  innerRadius={60}
                  outerRadius={90}
                  paddingAngle={5}
                  dataKey="value"
                >
                  {successFailureData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                  <Label
                    value={`${data.success_rate.toFixed(1)}%`}
                    position="center"
                    style={{ fontSize: '24px', fontWeight: 'bold', fill: '#ec4899' }}
                  />
                </Pie>
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#fff',
                    border: '1px solid #e5e7eb',
                    borderRadius: '8px',
                  }}
                />
                <Legend />
              </PieChart>
            </ResponsiveContainer>
          </Paper>
        </Grid.Col>

        {/* Top Scripts - Horizontal Bar Chart */}
        <Grid.Col span={{ base: 12, lg: 6 }}>
          <Paper shadow="sm" p="md" withBorder>
            <Title order={4} mb="md">
              Top 10 Scripts
            </Title>
            <ResponsiveContainer width="100%" height={350}>
              <BarChart data={data.top_scripts} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis type="number" stroke="#6b7280" fontSize={12} />
                <YAxis
                  type="category"
                  dataKey="name"
                  stroke="#6b7280"
                  fontSize={11}
                  width={120}
                  tickFormatter={(value) =>
                    value.length > 15 ? `${value.substring(0, 15)}...` : value
                  }
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#fff',
                    border: '1px solid #e5e7eb',
                    borderRadius: '8px',
                  }}
                  formatter={(value: any, name: string) => {
                    if (name === 'executions') return [value, 'Executions'];
                    return [value, name];
                  }}
                />
                <Bar dataKey="executions" fill="#ec4899" radius={[0, 8, 8, 0]}>
                  {data.top_scripts.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={`rgba(236, 72, 153, ${1 - index * 0.08})`} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </Paper>
        </Grid.Col>

        {/* Execution Time Distribution - Area Chart */}
        <Grid.Col span={{ base: 12, lg: 6 }}>
          <Paper shadow="sm" p="md" withBorder>
            <Title order={4} mb="md">
              Execution Time Distribution
            </Title>
            <ResponsiveContainer width="100%" height={350}>
              <BarChart data={sortedTimeDistribution}>
                <defs>
                  <linearGradient id="colorTime" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#f472b6" stopOpacity={0.9} />
                    <stop offset="95%" stopColor="#ec4899" stopOpacity={0.6} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis dataKey="bucket" stroke="#6b7280" fontSize={12} />
                <YAxis stroke="#6b7280" fontSize={12} />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#fff',
                    border: '1px solid #e5e7eb',
                    borderRadius: '8px',
                  }}
                />
                <Bar dataKey="count" fill="url(#colorTime)" radius={[8, 8, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </Paper>
        </Grid.Col>

        {/* Scripts by Category - Stacked Bar */}
        {data.category_stats.length > 0 && (
          <Grid.Col span={12}>
            <Paper shadow="sm" p="md" withBorder>
              <Title order={4} mb="md">
                Scripts by Category
              </Title>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={categoryStackedData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis dataKey="category" stroke="#6b7280" fontSize={12} />
                  <YAxis stroke="#6b7280" fontSize={12} />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#fff',
                      border: '1px solid #e5e7eb',
                      borderRadius: '8px',
                    }}
                  />
                  <Legend />
                  <Bar dataKey="Official" stackId="a" fill="#ec4899" radius={[8, 8, 0, 0]} />
                  <Bar dataKey="User" stackId="a" fill="#f472b6" radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </Paper>
          </Grid.Col>
        )}
      </Grid>

      {/* Script Details Table */}
      <Paper shadow="sm" p="md" withBorder mt="xl">
        <Title order={4} mb="md">
          Script Performance Details
        </Title>
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ borderBottom: '2px solid #e5e7eb' }}>
                <th style={{ padding: '12px', textAlign: 'left' }}>Script</th>
                <th style={{ padding: '12px', textAlign: 'left' }}>Category</th>
                <th style={{ padding: '12px', textAlign: 'right' }}>Executions</th>
                <th style={{ padding: '12px', textAlign: 'right' }}>Avg Time</th>
                <th style={{ padding: '12px', textAlign: 'right' }}>Success Rate</th>
              </tr>
            </thead>
            <tbody>
              {data.top_scripts.map((script, index) => (
                <tr
                  key={index}
                  style={{ borderBottom: '1px solid #f3f4f6', backgroundColor: index % 2 === 0 ? '#fafafa' : '#fff' }}
                >
                  <td style={{ padding: '12px' }}>
                    <Text fw={500}>{script.name}</Text>
                  </td>
                  <td style={{ padding: '12px' }}>
                    <Badge color="pink" variant="light" size="sm">
                      {script.category}
                    </Badge>
                  </td>
                  <td style={{ padding: '12px', textAlign: 'right' }}>{script.executions}</td>
                  <td style={{ padding: '12px', textAlign: 'right' }}>
                    {(script.avg_duration / 1000).toFixed(2)}s
                  </td>
                  <td style={{ padding: '12px', textAlign: 'right' }}>
                    <Badge color={script.success_rate >= 80 ? 'green' : 'orange'} variant="filled" size="sm">
                      {script.success_rate.toFixed(1)}%
                    </Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Paper>
    </div>
  );
};

export default Analytics;
