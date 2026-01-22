import { Card, SimpleGrid, Group, Stack, Title, Text, Button, ThemeIcon, Badge } from "@mantine/core";
import { BarChart3, Zap, GitBranch, Plus, Lock, FileText } from "lucide-react";
import AdminKeyDiagnostics from "./AdminKeyDiagnostics";

interface DashboardProps {
  scripts: string[];
  onAddScript: () => void;
  isAdmin: boolean;
  officialScripts: string[];
}

export default function Dashboard({ scripts, onAddScript, isAdmin, officialScripts }: DashboardProps) {
  const StatCard = ({ icon: Icon, title, value }: { icon: any; title: string; value: string | number }) => (
    <Card withBorder p="lg" radius="md" style={{ cursor: "pointer" }} className="hover:shadow-md transition-all">
      <Group justify="space-between" mb="xs">
        <Title order={4}>{title}</Title>
        <ThemeIcon variant="light" size="lg" radius="md">
          <Icon size={18} />
        </ThemeIcon>
      </Group>
      <Text fw={700} size="xl">
        {value}
      </Text>
    </Card>
  );

  return (
    <Stack gap="lg">
      <AdminKeyDiagnostics />

      <SimpleGrid cols={{ base: 1, sm: 3 }} spacing="lg">
        <StatCard icon={BarChart3} title="Total Scripts" value={scripts.length + officialScripts.length} />
        <StatCard icon={Zap} title="Status" value="Active" />
        <StatCard icon={GitBranch} title="Last Sync" value="Now" />
      </SimpleGrid>

      <Stack gap="lg">
        <Group justify="space-between">
          <Title order={2}>Official Scripts</Title>
          {isAdmin && (
            <Button
              leftSection={<Plus size={18} />}
              onClick={onAddScript}
              variant="gradient"
              gradient={{ from: "blue", to: "purple" }}
            >
              Add Script
            </Button>
          )}
        </Group>

        <Stack gap="xs">
          {officialScripts.slice(0, 5).map((script) => (
            <Group key={script} p="sm" style={{ borderRadius: "8px" }} className="hover:bg-gray-50 dark:hover:bg-gray-700">
              <ThemeIcon variant="light" color="blue" size="lg">
                <Lock size={18} />
              </ThemeIcon>
              <Text fw={500}>{script}</Text>
              <Badge size="sm" variant="light" ml="auto">
                Official
              </Badge>
            </Group>
          ))}
        </Stack>

        <Title order={3} mt="lg">
          User Scripts
        </Title>
        <Stack gap="xs">
          {scripts.slice(0, 5).map((script) => (
            <Group key={script} p="sm" style={{ borderRadius: "8px" }} className="hover:bg-gray-50 dark:hover:bg-gray-700">
              <ThemeIcon variant="light" color="gray" size="lg">
                <FileText size={18} />
              </ThemeIcon>
              <Text fw={500}>{script}</Text>
            </Group>
          ))}
        </Stack>
      </Stack>
    </Stack>
  );
}
