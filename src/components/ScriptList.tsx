import { Card, Button, Group, SimpleGrid, Stack, Title, Text, Badge } from "@mantine/core";
import { Play, Trash, Lock } from "lucide-react";
import { useTranslation } from "react-i18next";

interface ScriptListProps {
  title: string;
  scripts: string[];
  selected?: string | null;
  onSelect?: (script: string) => void;
  onDelete?: (script: string) => void;
  onEncrypt?: (script: string) => void;
  emptyText?: string;
  viewMode?: "list" | "grid";
}

export default function ScriptList({
  title,
  scripts,
  selected = null,
  onSelect,
  onDelete,
  onEncrypt,
  emptyText = "Brak skryptów",
  viewMode = "list",
}: ScriptListProps) {
  const { t } = useTranslation();

  if (!scripts.length) {
    return (
      <Stack gap="md">
        <Title order={2}>{title}</Title>
        <Card p="md" radius="md" withBorder>
          <Text c="dimmed">{emptyText}</Text>
        </Card>
      </Stack>
    );
  }

  const ScriptCard = ({ script }: { script: string }) => (
    <Card
      key={script}
      p="md"
      radius="md"
      withBorder
      shadow={selected === script ? "md" : "xs"}
      style={{
        cursor: onSelect ? "pointer" : "default",
        border: selected === script ? "2px solid #4dabf7" : undefined,
        transition: "all 0.2s ease",
      }}
      onClick={() => onSelect && onSelect(script)}
      onMouseEnter={(e) => {
        if (onSelect) {
          (e.currentTarget as HTMLElement).style.boxShadow = "var(--shadow-md)";
        }
      }}
      onMouseLeave={(e) => {
        if (onSelect && selected !== script) {
          (e.currentTarget as HTMLElement).style.boxShadow = "var(--shadow-xs)";
        }
      }}
    >
      <Group justify="space-between" wrap="nowrap">
        <div style={{ flex: 1 }}>
          <Text fw={500} size="sm">
            {script}
          </Text>
        </div>
        <Group gap="xs" wrap="nowrap">
          {onSelect && (
            <Badge leftSection={<Play size={12} />} variant="light" color="blue">
              {t("buttons.run", { defaultValue: "Uruchom" })}
            </Badge>
          )}
          {onEncrypt && (
            <Button
              variant="subtle"
              color="yellow"
              size="xs"
              onClick={(e) => {
                e.stopPropagation();
                onEncrypt(script);
              }}
              aria-label="Encrypt script"
              title="Encrypt this script to protect it from viewing/editing"
            >
              <Lock size={16} />
            </Button>
          )}
          {onDelete && (
            <Button
              variant="subtle"
              color="red"
              size="xs"
              onClick={(e) => {
                e.stopPropagation();
                onDelete(script);
              }}
              aria-label={t("buttons.delete", { defaultValue: "Usuń" })}
            >
              <Trash size={16} />
            </Button>
          )}
        </Group>
      </Group>
    </Card>
  );

  const layoutComponent = viewMode === "grid" ? SimpleGrid : Stack;
  const layoutProps = viewMode === "grid" ? { cols: { base: 1, sm: 2, md: 3 }, spacing: "md" } : { gap: "md" };

  return (
    <Stack gap="md">
      <Title order={2}>{title}</Title>
      {layoutComponent === SimpleGrid ? (
        <SimpleGrid {...layoutProps}>{scripts.map((script) => <ScriptCard script={script} />)}</SimpleGrid>
      ) : (
        <Stack {...layoutProps}>{scripts.map((script) => <ScriptCard script={script} />)}</Stack>
      )}
    </Stack>
  );
}
