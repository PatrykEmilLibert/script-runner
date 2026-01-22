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
}

export default function ScriptList({
  title,
  scripts,
  selected = null,
  onSelect,
  onDelete,
  onEncrypt,
  emptyText = "Brak skryptów",
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
      p="sm"
      radius="md"
      withBorder
      shadow={selected === script ? "md" : "xs"}
      style={{
        cursor: onSelect ? "pointer" : "default",
        border: selected === script ? "2px solid #4dabf7" : undefined,
        transition: "all 0.2s ease",
        minHeight: "80px",
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
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
      <Stack gap="xs">
        <Text fw={500} size="sm" lineClamp={2} style={{ wordBreak: "break-word" }}>
          {script}
        </Text>
        <Group gap="xs" justify="center" wrap="wrap">
          {onSelect && (
            <Badge 
              leftSection={<Play size={12} />} 
              variant="light" 
              color="blue"
              size="xs"
              style={{ cursor: "pointer" }}
            >
              {t("buttons.run", { defaultValue: "Uruchom" })}
            </Badge>
          )}
          {onEncrypt && (
            <Badge
              variant="light"
              color="yellow"
              size="xs"
              style={{ cursor: "pointer" }}
              onClick={(e) => {
                e.stopPropagation();
                onEncrypt(script);
              }}
              leftSection={<Lock size={12} />}
            >
              Encrypt
            </Badge>
          )}
          {onDelete && (
            <Badge
              variant="light"
              color="red"
              size="xs"
              style={{ cursor: "pointer" }}
              onClick={(e) => {
                e.stopPropagation();
                onDelete(script);
              }}
              leftSection={<Trash size={12} />}
            >
              {t("buttons.delete", { defaultValue: "Usuń" })}
            </Badge>
          )}
        </Group>
      </Stack>
    </Card>
  );

  return (
    <Stack gap="md">
      <Title order={2}>{title}</Title>
      <SimpleGrid 
        cols={{ base: 2, xs: 3, sm: 4, md: 5, lg: 6, xl: 7 }} 
        spacing="md"
      >
        {scripts.map((script) => <ScriptCard key={script} script={script} />)}
      </SimpleGrid>
    </Stack>
  );
}
