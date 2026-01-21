import { motion } from "framer-motion";

interface ScriptListProps {
  title: string;
  scripts: string[];
  selected?: string | null;
  onSelect?: (script: string) => void;
  emptyText?: string;
  viewMode?: "list" | "grid";
}

export default function ScriptList({
  title,
  scripts,
  selected = null,
  onSelect,
  emptyText = "Brak skryptów",
  viewMode = "list",
}: ScriptListProps) {
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <motion.div className="script-list" initial={{ x: -20, opacity: 0 }} animate={{ x: 0, opacity: 1 }}>
      {children}
    </motion.div>
  );

  if (!scripts.length) {
    return (
      <Wrapper>
        <h2 className="text-xl font-semibold mb-2">{title}</h2>
        <div className="p-4 text-gray-500 dark:text-gray-400 border rounded-lg dark:border-gray-700">
          {emptyText}
        </div>
      </Wrapper>
    );
  }

  return (
    <Wrapper>
      <h2 className="text-xl font-semibold mb-2">{title}</h2>
      <div
        className={`scripts-container grid gap-2 ${
          viewMode === "grid" ? "grid-cols-2 md:grid-cols-3" : "grid-cols-1"
        }`}
      >
        {scripts.map((script) => (
          <motion.button
            key={script}
            className={`script-item flex items-center justify-between px-4 py-3 rounded-lg border dark:border-gray-700 bg-white dark:bg-gray-800 shadow-sm hover:shadow transition ${
              selected === script ? "ring-2 ring-blue-500" : ""
            } ${onSelect ? "cursor-pointer" : "cursor-default"}`}
            onClick={() => onSelect && onSelect(script)}
            whileHover={{ scale: onSelect ? 1.02 : 1 }}
            whileTap={{ scale: onSelect ? 0.98 : 1 }}
          >
            <span className="script-name text-left">{script}</span>
            {onSelect && <span className="script-status text-blue-600">▶</span>}
          </motion.button>
        ))}
      </div>
    </Wrapper>
  );
}
