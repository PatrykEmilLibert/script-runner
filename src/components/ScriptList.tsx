import { Card, Button, Group, SimpleGrid, Stack, Title, Text, Badge, ActionIcon, ThemeIcon } from "@mantine/core";
import { Play, Trash, Lock, Star } from "lucide-react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";

interface ScriptListProps {
  title: string;
  scripts: string[];
  selected?: string | null;
  onSelect?: (script: string) => void;
  onDelete?: (script: string) => void;
  onEncrypt?: (script: string) => void;
  onToggleFavorite?: (script: string, isFav: boolean) => void;
  favorites?: string[];
  emptyText?: string;
}

export default function ScriptList({
  title,
  scripts,
  selected = null,
  onSelect,
  onDelete,
  onEncrypt,
  onToggleFavorite,
  favorites = [],
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

  const isFavorite = (script: string) => favorites.includes(script);

  const renderScriptCard = (script: string, idx: number) => (
    <motion.div
      key={script}
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ delay: idx * 0.05 }}
    >
      <Card
        p="md"
        radius="md"
        withBorder
        className="glass-pink"
        style={{
          cursor: "pointer",
          background: isFavorite(script) ? "linear-gradient(135deg, rgba(255,20,147,0.15) 0%, rgba(219,112,147,0.1) 100%)" : undefined,
          border: selected === script ? "2px solid #FF1493" : "1px solid rgba(255,20,147,0.2)",
          transition: "all 0.3s ease",
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
          height: "100%",
        }}
        onClick={() => onSelect && onSelect(script)}
        onMouseEnter={(e) => {
          if (onSelect) {
            (e.currentTarget as HTMLElement).style.boxShadow = "0 8px 24px rgba(255,20,147,0.2)";
            (e.currentTarget as HTMLElement).style.transform = "translateY(-2px)";
          }
        }}
        onMouseLeave={(e) => {
          (e.currentTarget as HTMLElement).style.boxShadow = "none";
          (e.currentTarget as HTMLElement).style.transform = "translateY(0)";
        }}
      >
        <Group justify="space-between" mb="sm" align="flex-start">
          <Text fw={500} size="sm" lineClamp={2} style={{ flex: 1, wordBreak: "break-word" }}>
            {script}
          </Text>
          {onToggleFavorite && (
            <ActionIcon
              size="lg"
              variant="light"
              color={isFavorite(script) ? "pink" : "gray"}
              onClick={(e) => {
                e.stopPropagation();
                onToggleFavorite(script, !isFavorite(script));
              }}
            >
              <Star size={16} fill={isFavorite(script) ? "#FF1493" : "none"} color="#FF1493" />
            </ActionIcon>
          )}
        </Group>

        <Stack gap="xs">
          {onSelect && (
            <Button
              leftSection={<Play size={14} />}
              variant="gradient"
              gradient={{ from: "pink", to: "grape", deg: 135 }}
              size="sm"
              fullWidth
              className="pink-glow"
            >
              Run
            </Button>
          )}
          <Group gap="xs" justify="center" wrap="wrap">
            {onEncrypt && (
              <Button
                leftSection={<Lock size={12} />}
                variant="light"
                color="yellow"
                size="xs"
                onClick={(e) => {
                  e.stopPropagation();
                  onEncrypt(script);
                }}
              >
                Encrypt
              </Button>
            )}
            {onDelete && (
              <Button
                leftSection={<Trash size={12} />}
                variant="light"
                color="red"
                size="xs"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(script);
                }}
              >
                Delete
              </Button>
            )}
          </Group>
        </Stack>
      </Card>
    </motion.div>
  );

  return (
    <Stack gap="md">
      <Title order={2}>{title}</Title>
      <SimpleGrid 
        cols={{ base: 1, xs: 2, sm: 3, md: 4, lg: 5 }} 
        spacing="md"
      >
        {scripts.map((script, idx) => renderScriptCard(script, idx))}
      </SimpleGrid>
    </Stack>
  );
}
